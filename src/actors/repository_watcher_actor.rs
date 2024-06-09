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

        actor.setup.act_on::<Watch>(|actor, _event| {
            let (tx, mut rx) = tokio::sync::mpsc::channel(200); // Increased channel capacity
            let repository_id = actor.state.repo.id.clone();
            let notify_config = notify::Config::default()
                .with_poll_interval(Duration::from_secs(5))
                .with_compare_contents(true);
            let debouncer_config = Config::default()
                .with_timeout(Duration::from_millis(2000)) // Increased debounce timeout
                .with_notify_config(notify_config);
            let repository_path = actor.state.repo.path.clone();
            let repository_path_trace = repository_path.clone();
            let watch_staged_only = actor.state.repo.watch_staged_only;

            let mut debouncer = match new_debouncer_opt::<_, PollWatcher>(
                debouncer_config,
                move |debounce_result: DebounceEventResult| {
                    match debounce_result {
                        Ok(events) => {
                            let mut walker = WalkBuilder::new(&repository_path)
                                .standard_filters(true)
                                .add_custom_ignore_filename(".ignore")
                                .add_custom_ignore_filename(".gitignore")
                                .build();
                            for event in events {
                                if let Ok(canonical_event_path) = PathBuf::from(event.path.clone()).canonicalize() {
                                    if walker.any(|entry| {
                                        entry.as_ref()
                                            .map(|e| e.path().canonicalize().unwrap_or_default() == canonical_event_path)
                                            .unwrap_or(false)
                                    }) {
                                        tracing::debug!(event=?event);
                                        // We only care about files
                                        if event.path.is_dir() {
                                            continue;
                                        }
                                        if let Err(e) = tx.blocking_send((repository_id.clone(), canonical_event_path.clone())) {
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
                            if e.to_string().contains("index.lock") {
                                // Ignore the specific error for index.lock not found
                                trace!("Ignoring index.lock not found error: {:?}", e);
                            } else {
                                error!("Debounce error: {:?}", e);
                            }
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
            info!("Setting up the watcher for the repository at path: {}", &repository_path_trace);

            if let Err(e) = debouncer.watcher().watch(
                (&actor.state.repo.path).as_ref(),
                RecursiveMode::Recursive,
            ) {
                trace!("Couldn't start watching modified files: {:?}", e);
                return;
            }

            actor.state.watcher = Some(debouncer);
            let notification_context = actor.state.broker.clone();
            let repo_id = actor.state.repo.id.clone();

            tokio::spawn(async move {
                while let Some((repo_id, path)) = rx.recv().await {
                    // Event: Change Detected
                    // Description: Detected a change in the repository.
                    // Context: Repository ID.
                    info!(repo_id = ?repo_id, "Detected a change in the repository.");
                    let change_message = NotifyChange { repo_id, path };
                    notification_context.emit_async(change_message, None).await;
                }
            });
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
    use std::path::PathBuf;

    use akton::prelude::*;
    use git2::{DiffOptions, IndexAddOption, Repository, StatusOptions};
    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::thread_rng;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tracing::{debug, error};

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
        watch_staged_only = false
        "#.to_string();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_watcher() -> anyhow::Result<()> {
        use rand::Rng;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        use std::path::PathBuf;

        init_tracing();

        let tangler_config: TanglerConfig = toml::from_str(&*TOML.clone()).unwrap();
        let config = tangler_config.repositories.first().unwrap().clone();
        let (tangler, broker) = TanglerActor::init(tangler_config).await?;
        let repo_id = config.id.clone();

        // Create a test file in ./mock-repo-working
        // Create a test file in ./mock-repo-working
        let test_file_path = "test_file.txt"; // Relative to the repository root
        {
            let mut file = File::create(format!("./mock-repo-working/{}", test_file_path)).await?;
            // Generate random string data
            let random_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();
            file.write_all(random_string.as_bytes()).await?;
            debug!("Wrote test file with random string data: {}", random_string);
        }
        let path = PathBuf::from(test_file_path);

        // Open the repository
        let repo = Repository::open("./mock-repo-working")?;


        // Remove any existing index.lock file
        // let index_lock_path = "./mock-repo-working/.git/index.lock";
        // if std::path::Path::new(index_lock_path).exists() {
        //     std::fs::remove_file(index_lock_path)?;
        // }

        // // Add the file to the repository index and commit if necessary
        // let mut index = repo.index()?;
        // index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        // index.write()?;
        // index.add_path(&path)?;
        // index.write()?;
        //
        // // Check repository status
        // let mut status_options = StatusOptions::new();
        // status_options.include_untracked(true);
        // let statuses = repo.statuses(Some(&mut status_options))?;
        // for entry in statuses.iter() {
        //     debug!("File: {:?}, Status: {:?}", entry.path(), entry.status());
        // }

        // let path = PathBuf::from("*.*");
        // Pretend we get a change and notify the broker
        broker.emit_async(NotifyChange { repo_id, path: PathBuf::from("test_file_path") }, None).await;

        // Additional debug information for diff generation
        let mut diff_options = DiffOptions::new();
        // diff_options.include_untracked(true);
        diff_options.minimal(true);
        diff_options.pathspec(test_file_path);
        diff_options.pathspec("file2.txt");
        // diff_options.pathspec("test_file.txt");

        let diff = repo.diff_index_to_workdir(None, Some(&mut diff_options))?;
        let mut diff_text = Vec::new();
        // Print the diff
        // diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        //     match line.origin() {
        //         '+' => print!("+"),
        //         '-' => print!("-"),
        //         ' ' => print!(" "),
        //         _ => (),
        //     }
        //     print!("{}", std::str::from_utf8(line.content()).unwrap());
        //     true
        // })?;
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
            diff_text.extend_from_slice(line.content());
            true
        })?;
        let changes = String::from_utf8_lossy(&*diff_text);
        debug!("Generated diff: {}", changes);

        // tangler.suspend().await?;


        // Remove the test file after actors are terminated
        // tokio::fs::remove_file(format!("./mock-repo-working/{}", test_file_path)).await?;
        // debug!("Removed test file");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_path() -> anyhow::Result<()> {
        use rand::Rng;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        use std::path::PathBuf;

        init_tracing();


        let test_file_path = "./mock-repo-working/main.rs"; // Relative to the repository root
        let path = PathBuf::from(test_file_path).canonicalize()?;

        // Open the repository
        let repo = Repository::open("./mock-repo-working")?;


        let mut diff_options = DiffOptions::new();

        // Get the repository root directory
        let repo_root = repo.workdir().unwrap();

        // Get the canonical path of the repository root
        let repo_root_canonical = repo_root.canonicalize()?;

        // Example canonical path to a file
        let binding = path.clone();
        // Get the relative path by stripping the repository root prefix
        let relative_path = binding.strip_prefix(&repo_root_canonical)?;

        debug!(file_name=?path, "Adding file to pathspec");
        diff_options.pathspec(relative_path);
        diff_options.include_untracked(true);
        // diff_options.minimal(true);
        let diff = repo.diff_index_to_workdir(None, Some(&mut diff_options)).expect("nope");
        let mut diff_text = Vec::new();
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
            diff_text.extend_from_slice(line.content());
            true
        })?;
        let changes = String::from_utf8_lossy(&*diff_text);
        debug!("Generated diff: {}", changes);
        // Print the relative path
        debug!("Relative path: {}", relative_path.display());

        Ok(())
    }
}