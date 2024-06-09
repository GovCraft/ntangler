use std::path::{Path, PathBuf};
use std::time::Duration;

use akton::prelude::*;
use anyhow::Error;
use ignore::WalkBuilder;
use notify::{PollWatcher, RecursiveMode};
use notify_debouncer_mini::{Config, DebounceEventResult, Debouncer, new_debouncer_opt};
use tracing::{error, info, instrument, trace};

use crate::messages::{NotifyChange, Observe};
use crate::repository_config::RepositoryConfig;

#[akton_actor]
pub(crate) struct GitSentinel {
    repo: RepositoryConfig,
    watcher: Option<Debouncer<PollWatcher>>,
    broker: Context,
}

impl GitSentinel {
    /// Initializes the GitSentinel with the given configuration and broker context.
    ///
    /// # Parameters
    /// - `config`: Configuration for the repository.
    /// - `broker`: Context for the broker.
    ///
    /// # Returns
    /// - `anyhow::Result<Context>`: The context of the initialized actor.
    #[instrument(skip(config, broker))]
    pub(crate) async fn init(config: &RepositoryConfig, broker: Context) -> anyhow::Result<Context> {
        let mut actor = Akton::<GitSentinel>::create_with_id(&config.id);
        actor.state.repo = config.clone();
        actor.state.broker = broker.clone();

        // Event: Setting up Watch Handler
        // Description: Setting up the handler for Watch events.
        // Context: Repository configuration details.
        trace!(config = ?config, "Setting up the handler for Watch events.");

        actor.setup.act_on::<Observe>(|actor, _event| {
            let (tx, mut rx) = tokio::sync::mpsc::channel(200); // Increased channel capacity
            let repository_id = actor.state.repo.id.clone();

            let notify_config = notify::Config::default()
                .with_poll_interval(Duration::from_secs(3))
                .with_compare_contents(true);

            let debouncer_config = Config::default()
                .with_timeout(Duration::from_millis(1500)) // Increased debounce timeout
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
                                        tracing::trace!(event=?event);
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
                                // Event: Debounce Error
                                // Description: Error occurred during the debounce process.
                                // Context: Error details.
                                error!("Debounce error: {:?}", e);
                            }
                        }
                    }
                },
            ) {
                Ok(debouncer) => debouncer,
                Err(e) => {
                    // Event: Watcher Setup Failed
                    // Description: Failed to set up the watcher for the repository.
                    // Context: Error details.
                    trace!("Couldn't set up watcher: {:?}", e);
                    return;
                }
            };

            // Event: Setting up Watcher
            // Description: Setting up the watcher for the repository.
            // Context: Repository path details.
            trace!("Setting up the watcher for the repository at path: {}", &repository_path_trace);

            if let Err(e) = debouncer.watcher().watch(
                (&actor.state.repo.path).as_ref(),
                RecursiveMode::Recursive,
            ) {
                // Event: Watcher Start Failed
                // Description: Failed to start watching modified files.
                // Context: Error details.
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
                    info!(file = ?path, "Change in");
                    let change_message = NotifyChange { repo_id, path };
                    notification_context.emit_async(change_message, None).await;
                }
            });
        });
        let repo_name = actor.state.repo.path.clone();
        let branch_name = actor.state.repo.branch_name.clone();
        let context = actor.activate(None).await?;

        // Event: Activating GitSentinel
        // Description: Activating the GitSentinel.
        // Context: None
        trace!(repository=repo_name,"GitSentinel activated for");
        info!("Watching repository at {} on branch '{}'.",repo_name, branch_name);

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use akton::prelude::*;
    use git2::{DiffOptions, IndexAddOption, Repository, StatusOptions};
    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::thread_rng;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tokio::time;
    use tracing::{debug, error};

    use crate::actors::Tangler;
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

    async fn poll_repository_for_changes(repo_path: &str) -> Result<(), git2::Error> {
        let repo = Repository::open(repo_path)?;

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);
        status_options.recurse_untracked_dirs(true);

        loop {
            let statuses = repo.statuses(Some(&mut status_options))?;
            for entry in statuses.iter() {
                if entry.status().is_wt_modified() {
                    println!("Modified but unstaged file: {:?}", entry.path());
                    // Here you can handle the modified file as needed
                }
            }
            time::sleep(Duration::from_secs(3)).await; // Poll every 3 seconds
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_poll_capability() -> anyhow::Result<()> {
        use std::fs;
        use std::path::PathBuf;
        use rand::Rng;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        use git2::{Repository, RepositoryInitOptions};
        use tokio::time::{self, Duration};

        init_tracing();

        // Step 1: Set up a bare repository
        let bare_repo_path = "./mock-bare-repo";
        let bare_repo = Repository::init_opts(bare_repo_path, RepositoryInitOptions::new().bare(true))?;

        // Step 2: Clone the bare repository into a working repository
        let working_repo_path = "./mock-repo-working";
        let working_repo = Repository::clone(bare_repo_path, working_repo_path)?;

        // Create Tangler config for the working repository
        let tangler_config: TanglerConfig = toml::from_str(&*TOML.clone()).unwrap();
        let config = tangler_config.repositories.first().unwrap().clone();
        let (tangler, broker) = Tangler::init(tangler_config).await?;
        let repo_id = config.id.clone();

        // Start polling for changes in the working repository
        let notification_context = broker.clone();
        tokio::spawn(async move {
            if let Err(e) = poll_repository_for_changes(working_repo_path, repo_id.clone(), notification_context).await {
                error!("Error polling repository: {}", e);
            }
        });

        // Step 3: Create and modify a test file in the working repository
        let test_file_path = "test_file.txt"; // Relative to the repository root
        {
            let mut file = File::create(format!("{}/{}", working_repo_path, test_file_path)).await?;
            // Generate random string data
            let random_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();
            file.write_all(random_string.as_bytes()).await?;
            debug!("Wrote test file with random string data: {}", random_string);
        }

        // Allow some time for the polling mechanism to detect the change
        time::sleep(Duration::from_secs(5)).await;

        // Verify that the change was detected
        // (This part of the test depends on how you handle notifications in your actor system. You might check logs, messages, or some shared state.)

        // Clean up: remove the test repositories
        fs::remove_dir_all(bare_repo_path)?;
        fs::remove_dir_all(working_repo_path)?;

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