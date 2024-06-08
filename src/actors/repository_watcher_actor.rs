use std::path::{Path, PathBuf};
use std::time::Duration;

use akton::prelude::*;
use anyhow::Error;
use ignore::WalkBuilder;
use notify::{PollWatcher, RecursiveMode};
use notify_debouncer_mini::{Config, DebounceEventResult, Debouncer, new_debouncer_opt};
use tracing::{error, info, instrument, trace};

use crate::messages::{NotifyChange, Watch};
use crate::repository_config::RepositoryConfig;

#[akton_actor]
pub(crate) struct RepositoryWatcherActor {
    repo: RepositoryConfig,
    watcher: Option<Debouncer<PollWatcher>>,
    broker: Context,
}

impl RepositoryWatcherActor {
    /// Initializes the RepositoryWatcherActor with the given configuration and broker context.
    ///
    /// # Parameters
    /// - `config`: Configuration for the repository.
    /// - `broker`: Context for the broker.
    ///
    /// # Returns
    /// - `anyhow::Result<Context>`: The context of the initialized actor.
    #[instrument(skip(config, broker))]
    pub(crate) async fn init(config: &RepositoryConfig, broker: Context) -> anyhow::Result<Context> {
        let mut actor = Akton::<RepositoryWatcherActor>::create_with_id(&config.id);
        actor.state.repo = config.clone();
        actor.state.broker = broker.clone();
        // Event: Setting up Watch Handler
        // Description: Setting up the handler for Watch events.
        // Context: Repository configuration details.
        info!(config = ?config, "Setting up the handler for Watch events.");

        actor.setup.act_on::<Watch>(|actor, event| {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let message = &actor.state.repo;
            let repository_id = actor.state.repo.id.clone();
            let notify_config = notify::Config::default()
                .with_poll_interval(Duration::from_secs(1))
                .with_compare_contents(true);
            let debouncer_config = Config::default()
                .with_timeout(Duration::from_millis(1000))
                .with_notify_config(notify_config);
            let repository_path = actor.state.repo.path.clone();
            let watch_staged_only = actor.state.repo.watch_staged_only;
            let mut debouncer = match new_debouncer_opt::<_, PollWatcher>(
                debouncer_config,
                move |debounce_result: DebounceEventResult| {
                    match debounce_result {
                        Ok(events) => {
                            let mut walker = if watch_staged_only {
                                WalkBuilder::new(&repository_path)
                                    .add_custom_ignore_filename(".gitignore")
                                    .add_custom_ignore_filename(".ignore")
                                    .standard_filters(false)
                                    .hidden(false)
                                    .build()
                            } else {
                                WalkBuilder::new(&repository_path)
                                    .add_custom_ignore_filename(".gitignore")
                                    .add_custom_ignore_filename(".ignore")
                                    .standard_filters(true)
                                    .build()
                            };
                            for event in events {
                                if let Ok(canonical_event_path) = PathBuf::from(event.path.clone()).canonicalize() {
                                    if walker.any(|entry| {
                                        entry.as_ref()
                                            .map(|e| e.path().canonicalize().unwrap_or_default() == canonical_event_path)
                                            .unwrap_or(false)
                                    }) {
                                        if let Err(e) = tx.blocking_send(repository_id.clone()) {
                                            // Event: Failed to Send Change Notification
                                            // Description: Failed to send change notification through the channel.
                                            // Context: Error details.
                                            error!("Failed to send change notification: {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Event: Debounce Error
                            // Description: Error occurred during the debounce process.
                            // Context: Error details.
                            error!("Debounce error: {:?}", e);
                        }
                    }
                },
            ) {
                Ok(debouncer) => debouncer,
                Err(e) => {
                    trace!("Couldn't set up watcher: {:?}", e);
                    return;
                }
            };

            // Event: Setting up Watcher
            // Description: Setting up the watcher for the repository.
            // Context: Repository path details.
            info!("Setting up the watcher for the repository at path: {}", &actor.state.repo.path);

            if let Err(e) = debouncer.watcher().watch(
                &Path::new(&actor.state.repo.path).join(".git/index"),
                RecursiveMode::NonRecursive,
            ) {
                trace!("Couldn't start watching git repo: {:?}", e);
                return;
            }

            if !&actor.state.repo.watch_staged_only {
                if let Err(e) = debouncer.watcher().watch(
                    (&actor.state.repo.path).as_ref(),
                    RecursiveMode::Recursive,
                ) {
                    trace!("Couldn't start watching modified files: {:?}", e);
                    return;
                }
            }
            actor.state.watcher = Some(debouncer);

            let notification_context = actor.state.broker.clone();
            let repo_id = actor.state.repo.id.clone();
            tokio::spawn(async move {
                while let Some(repo_id) = rx.recv().await {
                    // Event: Change Detected
                    // Description: Detected a change in the repository.
                    // Context: Repository ID.
                    info!(repo_id = ?repo_id, "Detected a change in the repository.");
                    let change_message = NotifyChange { repo_id };
                    notification_context.emit_async(change_message, None).await;
                }
            });

            // Event: Watching for Changes
            // Description: Watching for changes in the repository.
            // Context: Repository path.
            info!("Watching for changes in {}...", &actor.state.repo.path);
        });

        // Event: Activating RepositoryWatcherActor
        // Description: Activating the RepositoryWatcherActor.
        // Context: None
        info!("Activating the RepositoryWatcherActor.");
        let context = actor.activate(None).await?;
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use akton::prelude::*;
    use lazy_static::lazy_static;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tracing::debug;

    use crate::actors::TanglerActor;
    use crate::init_tracing;
    use crate::messages::NotifyChange;
    use crate::tangler_config::TanglerConfig;

    lazy_static! {
        static ref TOML: String = r#"
[[repositories]]
path = "./mock-repo-working"
branch_name = "new_branch"
api_url = "https://api.example.com/generate-commit-message"
watch_staged_only = true
        "#.to_string();
}

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_watcher() -> anyhow::Result<()> {
        init_tracing();


        let tangler_config: TanglerConfig = toml::from_str(&*TOML.clone()).unwrap();
        let config = tangler_config.repositories.first().unwrap().clone();
        let (tangler, broker) = TanglerActor::init(tangler_config).await?;
        let repo_id = config.id.clone();

        // Create a test file in ./mock-repo-working
        let test_file_path = "./mock-repo-working/test_file.txt";
        {
            let mut file = File::create(test_file_path).await?;
            file.write_all("This is a test file.".as_ref()).await?;
            debug!("Wrote test file");
        }
        // // Pretend we get a change and notify the broker
        broker.emit_async(NotifyChange { repo_id }, None).await;

        tangler.suspend().await?;

        // Remove the test file after actors are terminated
        // tokio::fs::remove_file(test_file_path).await?;
        // debug!("Removed test file");

        Ok(())
    }
}