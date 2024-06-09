use std::any::TypeId;
use std::future::Future;
use std::sync::{Arc, Mutex};

use akton::prelude::*;
use anyhow::anyhow;
use git2::{DiffOptions, Error, IndexAddOption, Repository};
use tracing::{debug, error, info, trace, warn};

use crate::messages::{BrokerSubscribe, CheckoutBranch, Diff, NotifyChange, ResponseCommit, SubmitDiff};
use crate::repository_config::RepositoryConfig;
use crate::tangler_config::TanglerConfig;

#[akton_actor]
pub(crate) struct RepositoryActor {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
    broker: Context,
}

impl RepositoryActor {
    /// Initializes the repository actor with the provided custom behavior.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to the repository configuration.
    ///
    /// # Returns
    ///
    /// An optional `Context`, which is `Some` if the actor was successfully activated, or `None` otherwise.
    pub(crate) async fn init(config: &RepositoryConfig, broker: Context) -> Option<Context> {
        // Define the default behavior as an async closure that takes a reference to the repository configuration.
        let default_behavior = |config: RepositoryConfig, broker: Context| async move {
            RepositoryActor::default_behavior(&config, broker.clone()).await
        };

        // Call the `init_with_custom_behavior` function with the default behavior closure and the configuration.
        RepositoryActor::init_with_custom_behavior(default_behavior, config.clone(), broker).await
    }

    pub(crate) async fn init_with_custom_behavior<F, Fut>(
        custom_behavior: F,
        config: RepositoryConfig,
        broker: Context,
    ) -> Option<Context>
        where
            F: Fn(RepositoryConfig, Context) -> Fut + Send + Sync + 'static,
            Fut: Future<Output=Option<Context>> + Send,
    {
        // Execute the custom behavior and await its result
        custom_behavior(config, broker).await
    }

    /// Example custom behavior function to be passed into the `init` function.
    pub(crate) async fn default_behavior(config: &RepositoryConfig, broker: Context) -> Option<Context> {
        let mut actor = Akton::<RepositoryActor>::create_with_id(&config.id);
        actor.state.config = config.clone();
        actor.state.broker = broker;

        let repo = match Repository::open(&config.path) {
            Ok(repo) => repo,
            Err(e) => {
                error!("Failed to open repository: {}", e);
                return None;
            }
        };

        actor.state.repository = Some(Arc::new(Mutex::new(repo)));

        actor.setup
            .act_on::<CheckoutBranch>(|actor, _event| {
                trace!("Received CheckoutBranch message");
                actor.state.checkout_branch();
            })
            .act_on_async::<Diff>(|actor, event| {
                let diff: String = if let Some(repo) = &actor.state.repository {
                    let repo = repo.lock().expect("Couldn't lock repository mutex");

                    trace!(file=?event.message.path, "Generating a diff for repository");

                    let mut diff_options = DiffOptions::new();

                    // Get the repository root directory
                    let repo_root = repo.workdir().unwrap();
                    // Get the canonical path of the repository root
                    let repo_root_canonical = repo_root.canonicalize().unwrap();
                    // Canonicalize the event file path
                    let path = event.message.path.canonicalize().unwrap();

                    // Get the relative path by stripping the repository root prefix
                    let relative_path = path.strip_prefix(&repo_root_canonical).unwrap();
                    debug!(file_name=?relative_path, "Adding file to pathspec");

                    // Set the pathspec to the relative path
                    diff_options.pathspec(relative_path);
                    diff_options.include_untracked(true);

                    let mut index = repo.index().unwrap();
                    trace!(path=?index.path().unwrap(), "Diffing against index:");
                    debug!(working_dir=?repo.workdir().unwrap(), "...in");

                    // Generate the diff
                    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_options)).expect("nope");
                    let mut diff_text = Vec::new();
                    diff.print(git2::DiffFormat::Patch, |_, _, line| {
                        diff_text.extend_from_slice(line.content());
                        true
                    }).expect("Failed to print diff");

                    trace!(raw_diff = ?diff_text, "Raw diff generated for the repository.");
                    let changes = String::from_utf8_lossy(&diff_text).to_string();
                    trace!(diff = changes, "Diff generated for the repository.");
                    changes
                } else {
                    error!("Received request for Diff but the string was empty.");
                    String::new()
                };

                let id = actor.state.config.id.clone();
                let broker = actor.state.broker.clone();
                let diff = diff.clone();
                let path = event.message.path.clone();

                Box::pin(async move {
                    if !diff.is_empty() {
                        let trace_id = id.clone();
                        broker.emit_async(SubmitDiff { diff, id, path }, None).await;
                        trace!(repo_id = trace_id, broker=?&broker, "Submitted retrieved Diff to broker.");
                    } else {
                        warn!("Received request for Diff but no repo diffs found.");
                    }
                })
            })
            .act_on::<ResponseCommit>(|actor, event| {
                // Event: Received Commit Response
                // Description: Received a commit response and will commit changes to the repository.
                // Context: Commit message details.
                trace!(commit_message = ?event.message.commits, "Received ReponseCommit message and will attempt to commit changes to the repository.");

                // Canonicalize the event file path
                let target_file = event.message.path.canonicalize().expect("Failed to canonicalize path.");

                // Received change, so we need to commit to this repo
                if let Some(repo) = &actor.state.repository {
                    let response_commit = event.message;
                    let commit_message = &event.message.commits;
                    let repo = repo.lock().expect("Failed to lock repo mutex");

                    // Get the repository root directory
                    let repo_root = repo.workdir().unwrap();
                    // Get the canonical path of the repository root
                    let repo_root_canonical = repo_root.canonicalize().unwrap();

                    for commit in &commit_message.commits {
                        // Get the relative path by stripping the repository root prefix
                        let relative_path = target_file.strip_prefix(&repo_root_canonical).unwrap();
                        let sig = repo.signature().expect("Failed to get signature");

                        // Stage all modified files
                        let mut index = repo.index().expect("Failed to get index");
                        debug!(file=?relative_path, "Repo index add");
                        index.add_path(relative_path).expect("Failed to add files to index");
                        index.write().expect("Failed to write index");

                        let tree_id = index.write_tree().expect("Failed to write tree");
                        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
                        let head = repo.head().expect("Failed to get HEAD");
                        let parent_commit = head.peel_to_commit().expect("Failed to get parent commit");

                        let commit_message = &format!("{}\n{}", commit.commit.heading, commit.commit.description);
                        repo.commit(
                            Some("HEAD"),
                            &sig,
                            &sig,
                            commit_message,
                            &tree,
                            &[&parent_commit],
                        ).expect("Failed to commit");
                    }

                    info!(commits=commit_message.commits.len(),"Local");
                }
            });

        info!("Subscribing to broker for commit message response notifications.");
        let subscription = BrokerSubscribe {
            message_type_id: TypeId::of::<ResponseCommit>(),
            subscriber_context: actor.context.clone(),
        };

        actor.state.broker.emit_async(subscription, None).await;


        let context = actor.activate(None).await;


        match context {
            Ok(context) => {
                trace!(
                    "Activated RepositoryActor, attempting to checkout branch {}",
                    &config.branch_name
                );
                context.emit_async(CheckoutBranch, None).await;
                Some(context)
            }
            Err(_) => {
                error!(
                    "Failed to activate RepositoryActor for repository at {}",
                    &config.path
                );
                None
            }
        }
    }

    fn checkout_branch(&mut self) {
        if let Some(repository) = &self.repository {
            let repo = repository.lock().expect("Couldn't lock repository mutex");
            match repo.find_branch(&self.config.branch_name, git2::BranchType::Local) {
                Ok(_branch_ref) => {
                    trace!("Found branch: {}", &self.config.branch_name);
                    let checkout_result = repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::new().path(&self.config.path),
                    ));
                    match checkout_result {
                        Ok(_) => {
                            //set the repo to this branch
                        }
                        Err(e) => {
                            error!("Failed to checkout head: {}", e)
                        }
                    }
                }
                Err(e) => {
                    error!(
                            "Failed to find branch: {}. Logged error is: {}",
                            &self.config.branch_name, e
                        );
                }
            };
        } else {
            error!("Failed to find repository: {}", &self.config.path);
        }
    }
    /// Opens an existing repository and checks out a specific branch.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the repository.
    /// * `branch_name` - The name of the branch to checkout.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - An empty result indicating success or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let my_struct = MyStruct { repository: Some(Arc::new(Mutex::new(Repository::open("/path/to/repo").unwrap()))) };
    /// my_struct.open_repository_to_branch("feature-branch").unwrap();
    /// ```
    fn open_repository_to_branch(&self) -> anyhow::Result<()> {
        // Check if the repository is available
        if let Some(repository) = &self.repository {
            // Lock the repository mutex
            let repo = repository.lock().expect("Couldn't lock repository mutex");

            // Find the branch reference
            let branch_ref = repo
                .find_branch(&self.config.branch_name, git2::BranchType::Local)?
                .into_reference();

            // Set the HEAD to point to the branch reference
            repo.set_head(branch_ref.name().unwrap())?;

            // Checkout the branch
            let checkout_result =
                repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use akton::prelude::{ActorContext, Akton, Context};
    use anyhow::anyhow;
    use git2::{DiffOptions, Repository};
    use pretty_assertions::assert_eq;
    use tokio::sync::oneshot;
    use tracing::{error, info, trace};

    use crate::actors::Broker;
    use crate::actors::git_repository::RepositoryActor;
    use crate::init_tracing;
    use crate::messages::{NotifyChange, ResponseCommit};
    use crate::repository_config::RepositoryConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_non_existing_branch() -> anyhow::Result<()> {
        init_tracing();
        let config = RepositoryConfig {
            path: "./mock-repo-working".to_string(),
            branch_name: "non_existing_branch".to_string(),
            api_url: "".to_string(),
            watch_staged_only: false,
            id: "any id".to_string(),
        };
        let broker = Broker::init().await?;

        let actor_context = RepositoryActor::init(&config, broker).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.suspend().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "Deprecated in favor of test_commit for now"]
    async fn test_notify_change() -> anyhow::Result<()> {
        use std::fs::{self, File, OpenOptions};
        use std::io::Write;
        use std::path::Path;
        use std::sync::{Arc, Mutex};
        use tokio::sync::oneshot;

        init_tracing();

        let (sender, receiver) = oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender))); // Wrap the sender in Arc<Mutex<Option>>

        let config = RepositoryConfig {
            path: "./mock-repo-working".to_string(),
            branch_name: "new_branch".to_string(),
            api_url: "".to_string(),
            watch_staged_only: false,
            id: "any id".to_string(),
        };

        let test_behavior = {
            let sender = sender.clone(); // Clone the Arc to use in the closure
            move |config: RepositoryConfig, broker: Context| {
                let sender = sender.clone(); // Clone the Arc again for use inside the async block
                async move {
                    let mut actor = Akton::<RepositoryActor>::create_with_id(&config.id);
                    let repo = Repository::open(&config.path).expect("Failed to open mock repo");
                    actor.state.repository = Some(Arc::new(Mutex::new(repo)));
                    actor.state.config = config;
                    trace!("Running test in {}", &actor.state.config.path);
                    actor.state.open_repository_to_branch();
                    actor.setup.act_on::<NotifyChange>(move |actor, _event| {
                        info!("Received NotifyChange message");
                        if let Some(repo) = &actor.state.repository {
                            let repo = repo.lock().expect("Couldn't lock repository mutex");
                            let diff = repo.diff_index_to_workdir(None, Some(&mut DiffOptions::new()))
                                .expect("Failed to get diff");
                            let mut diff_text = Vec::new();
                            diff.print(git2::DiffFormat::Patch, |_, _, line| {
                                diff_text.extend_from_slice(line.content());
                                true
                            }).expect("Failed to print diff");
                            let changes = String::from_utf8(diff_text).expect("Failed to convert diff to string");
                            let mut sender_guard = sender.lock().unwrap();
                            if let Some(sender) = sender_guard.take() { // Properly take the sender out of the Option
                                sender.send(changes).expect("Couldn't send test diff");
                            } else {
                                error!("Sender has already been taken or was never initialized.");
                            }
                        } else {
                            error!("No test repo found.");
                        }
                    });

                    let context = actor.activate(None).await.expect("Couldn't activate RepositoryActor");
                    Some(context)
                }
            }
        };
        let broker = Broker::init().await?;

        let actor_context = RepositoryActor::init_with_custom_behavior(test_behavior, config.clone(), broker).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();

        // Add and modify a file in the ./mock-repo-working directory
        let file_path = Path::new(&config.path).join("test_file.txt");
        {
            trace!("Creating test file in {}", file_path.display());
            let mut file = File::create(&file_path)?;
            writeln!(file, "Initial content")?;
        }
        {
            trace!("Modifying test file at {}", file_path.display());
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&file_path)?;
            writeln!(file, "Modified content")?;
        }

        let repo_id = config.id;
        let path = file_path.clone();
        trace!("Notifying actor of change");
        context.emit_async(NotifyChange { repo_id, path }, None).await;

        let result = receiver.await?;

// Expected string formatted as a multiline string for clarity and accuracy
        let expected_diff = r#"diff --git a/test_file.txt b/test_file.txt
index 8430408..edc5728 100644
--- a/test_file.txt
+++ b/test_file.txt
@@ -1 +1,2 @@
Initial content
Modified content
"#;

// Print both strings to compare visually in the test output
        println!("Expected:\n{}", expected_diff);
        println!("Actual:\n{}", result);

// Assert that the actual diff matches the expected diff
        assert_eq!(expected_diff, &result);
        trace!("Removing test file");
        fs::remove_file(&file_path)?;

        context.suspend().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_existing_branch() -> anyhow::Result<()> {
        init_tracing();
        let config = RepositoryConfig {
            path: "./mock-repo-working".to_string(),
            branch_name: "new_branch".to_string(),
            api_url: "".to_string(),
            watch_staged_only: false,
            id: "any id".to_string(),
        };
        let broker = Broker::init().await?;

        let actor_context = RepositoryActor::init(&config, broker).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.suspend().await?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::PathBuf;
        use crate::commits::{Commit, CommitDetails, Commits};
        use pretty_assertions::assert_eq;

        #[test]
        fn test_squash_commits() {
            // Sample data for testing
            let commit1 = Commit {
                commit: CommitDetails {
                    heading: String::from("Initial commit"),
                    description: String::from("This is the initial commit."),
                },
            };
            let commit2 = Commit {
                commit: CommitDetails {
                    heading: String::from("Added feature"),
                    description: String::from("Implemented the new feature."),
                },
            };
            let commit3 = Commit {
                commit: CommitDetails {
                    heading: String::from("Fixed bug"),
                    description: String::from("Fixed a critical bug."),
                },
            };

            let commits = Commits {
                commits: vec![commit1, commit2, commit3],
            };

            let response_commit = ResponseCommit {
                path: PathBuf::from("src/main.rs"),
                commits,
            };

            // Squash the commits
            let squashed_commit = response_commit.squash_commits();

            // Expected result
            let expected_commit = "Initial commit\n\nThis is the initial commit.\n\nAdded feature: Implemented the new feature.\n\nFixed bug: Fixed a critical bug.";

            // Assert the result
            assert_eq!(squashed_commit, expected_commit);
        }
    }
}
