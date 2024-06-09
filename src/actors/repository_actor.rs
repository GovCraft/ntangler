use std::any::TypeId;
use std::future::Future;
use std::sync::{Arc, Mutex};

use akton::prelude::*;
use anyhow::anyhow;
use git2::{DiffOptions, Error, Repository};
use tracing::{debug, error, info, trace};

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

                    // Event: Generating Diff
                    // Description: Generating a diff for the repository.
                    // Context: Watch staged only configuration.
                    info!(file=?event.message.path, "Generating a diff for repository");

                    let mut diff_options = DiffOptions::new();
                    if let Some(path) = event.message.path.file_name().unwrap().to_str() {
                        diff_options.pathspec(path);
                        diff_options.minimal(true);
                    } else {
                        error!("Failed to convert PathBuf to str");
                        return Box::pin(async move {});
                    }

                    // Generate the diff
                    let diff = if actor.state.config.watch_staged_only {
                        let index = repo.index().expect("Failed to get index");
                        repo.diff_index_to_workdir(Some(&index), Some(&mut diff_options))
                    } else {
                        repo.diff_index_to_workdir(None, Some(&mut diff_options))
                    }.expect("Failed to get diff");

                    let mut diff_text = Vec::new();
                    diff.print(git2::DiffFormat::Patch, |_, _, line| {
                        diff_text.extend_from_slice(line.content());
                        true
                    }).expect("Failed to print diff");

                    let changes = String::from_utf8_lossy(&diff_text).to_string();

                    // Event: Diff Generated
                    // Description: The diff for the repository has been generated.
                    // Context: Diff text.
                    trace!(diff = changes, "Diff generated for the repository.");

                    changes
                } else {
                    error!("Received request for Diff but no repo diffs found.");
                    String::new()
                };

                let id = actor.state.config.id.clone();
                let broker = actor.state.broker.clone();
                let diff = diff.clone();

                Box::pin(async move {
                    if !diff.is_empty() {
                        let trace_id = id.clone();
                        broker.emit_async(SubmitDiff { diff, id }, None).await;
                        debug!(repo_id = trace_id, broker=?&broker, "Submitted retrieved Diff to broker.");
                    } else {
                        error!("Received request for Diff but no repo diffs found.");
                    }
                })
            })
            .act_on::<ResponseCommit>(|actor, event| {
                // Event: Received Commit Response
                // Description: Received a commit response and will commit changes to the repository.
                // Context: Commit message details.
                trace!(commit_message = ?event.message.commits, "Received ReponseCommit message and will attempt to commit changes to the repository.");

                // Received change, so we need to commit to this repo
                if let Some(repo) = &actor.state.repository {
                    let commit_message = &event.message.commits;
                    let repo = repo.lock().expect("Failed to lock repo mutex");
                    for commit in &commit_message.commits {
                        let sig = repo.signature().expect("Failed to get signature");

                        // Stage all modified files
                        let mut index = repo.index().expect("Failed to get index");
                        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).expect("Failed to add files to index");
                        index.write().expect("Failed to write index");

                        let tree_id = index.write_tree().expect("Failed to write tree");
                        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
                        let head = repo.head().expect("Failed to get HEAD");
                        let parent_commit = head.peel_to_commit().expect("Failed to get parent commit");
                        let commit = &format!("{}\n{}", commit.commit.heading, commit.commit.description);
                        repo.commit(
                            Some("HEAD"),
                            &sig,
                            &sig,
                            commit,
                            &tree,
                            &[&parent_commit],
                        ).expect("Failed to commit");
                    }

                    // Event: Changes Committed
                    // Description: Changes have been committed to the repository.
                    // Context: Commit message details.
                    info!("Committed changes {} locally", commit_message.commits.len());
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

    use crate::actors::BrokerActor;
    use crate::actors::repository_actor::RepositoryActor;
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
        let broker = BrokerActor::init().await?;

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
        let broker = BrokerActor::init().await?;

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
        let broker = BrokerActor::init().await?;

        let actor_context = RepositoryActor::init(&config, broker).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.suspend().await?;
        Ok(())
    }
}
