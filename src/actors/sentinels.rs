use std::path::{Path, PathBuf};
use std::time::Duration;

use akton::prelude::*;
use anyhow::Error;
use ignore::WalkBuilder;
use notify::{PollWatcher, RecursiveMode};
use notify_debouncer_mini::{Config, DebounceEventResult, Debouncer, new_debouncer_opt};
use tracing::{error, info, instrument, trace};

use crate::messages::{NotifyChange, Observe, Poll};
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
            let broker = actor.state.broker.clone();

            tokio::spawn(async move {
                let broker = broker.clone();
                loop {
                    let broker = broker.clone();
                    broker.emit_async(Poll, None).await;
                    tokio::time::sleep(Duration::from_secs(60)).await; // Poll every 3 seconds
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
        println!("Now watching your repository at {repo_name} and making commits to your {branch_name} branch. Just code—commits are taken care of. To stop Tangler, press Ctrl+C.");
        println!("Happy coding!");

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

    use akton::prelude::*;
    use git2::{DiffOptions, IndexAddOption, Repository, StatusOptions};
    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::thread_rng;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tokio::{task, time};
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

    #[tokio::test]
    async fn test_poll_modified_unstaged_files() -> anyhow::Result<()> {
        use git2::{Repository, Status, StatusOptions};
        use std::fs::{File, OpenOptions};
        use std::io::Write;
        use std::path::Path;
        use tokio::time::{self, Duration};
        use tracing::debug;
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;

        // Initialize tracing for logging
        init_tracing();

        // Step 1: Initialize a new repository
        let repo_path = "./mock-repo";
        let _ = std::fs::remove_dir_all(repo_path); // Clean up any previous test runs
        let repo = Repository::init(repo_path)?;

        // Step 2: Create a test file in the repository and commit it
        let test_file_path = Path::new(repo_path).join("test_file.txt");
        {
            let mut file = File::create(&test_file_path)?;
            writeln!(file, "Initial content")?;
        }

        // Add and commit the file
        let mut index = repo.index()?;
        index.add_path(Path::new("test_file.txt"))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let sig = repo.signature()?;
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

        // Step 3: Modify the test file
        {
            let mut file = OpenOptions::new().write(true).append(true).open(&test_file_path)?;
            let random_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();
            writeln!(file, "Modified content: {}", random_string)?;
            debug!("Modified test file with random string data: {}", random_string);
        }

        // Step 4: Check for modified but unstaged files
        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);

        // Polling mechanism (for demonstration, we'll just check once)
        time::sleep(Duration::from_secs(1)).await;

        let statuses = repo.statuses(Some(&mut status_options))?;
        let modified_files: Vec<_> = statuses
            .iter()
            .filter(|entry| entry.status().contains(Status::WT_MODIFIED))
            .map(|entry| entry.path().unwrap().to_string())
            .collect();

        debug!("Modified but unstaged files: {:?}", modified_files);

        // Assert that our test file is detected as modified
        assert!(modified_files.contains(&"test_file.txt".to_string()));

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