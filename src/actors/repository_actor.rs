use std::future::Future;
use std::sync::{Arc, Mutex};

use akton::prelude::*;
use anyhow::anyhow;
use git2::{DiffOptions, Error, Repository};
use tracing::{error, trace};

use crate::ginja_config::GinjaConfig;
use crate::messages::{CheckoutBranch, NotifyChange};
use crate::repository_config::RepositoryConfig;

#[akton_actor]
pub(crate) struct RepositoryActor {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
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
    pub(crate) async fn init(config: &RepositoryConfig) -> Option<Context> {
        // Define the default behavior as an async closure that takes a reference to the repository configuration.
        let default_behavior = |config: RepositoryConfig| async move {
            RepositoryActor::default_behavior(&config).await
        };

        // Call the `init_with_custom_behavior` function with the default behavior closure and the configuration.
        RepositoryActor::init_with_custom_behavior(default_behavior, config.clone()).await
    }

    pub(crate) async fn init_with_custom_behavior<F, Fut>(
        custom_behavior: F,
        config: RepositoryConfig,
    ) -> Option<Context>
        where
            F: Fn(RepositoryConfig) -> Fut + Send + Sync + 'static,
            Fut: Future<Output=Option<Context>> + Send,
    {
        // Execute the custom behavior and await its result
        custom_behavior(config).await
    }

    /// Example custom behavior function to be passed into the `init` function.
    pub(crate) async fn default_behavior(config: &RepositoryConfig) -> Option<Context> {
        let mut actor = Akton::<RepositoryActor>::create_with_id(&config.id);
        actor.state.config = config.clone();

        let repo = match Repository::open(&config.path) {
            Ok(repo) => repo,
            Err(e) => {
                error!("Failed to open repository: {}", e);
                return None;
            }
        };

        actor.state.repository = Some(Arc::new(Mutex::new(repo)));

        actor.setup.act_on::<CheckoutBranch>(|actor, _event| {
            trace!("Received CheckoutBranch message");
            if let Some(repository) = &actor.state.repository {
                let repo = repository.lock().expect("Couldn't lock repository mutex");
                match repo.find_branch(&actor.state.config.branch_name, git2::BranchType::Local) {
                    Ok(_branch_ref) => {
                        trace!("Found branch: {}", &actor.state.config.branch_name);
                        let checkout_result = repo.checkout_head(Some(
                            git2::build::CheckoutBuilder::new().path(&actor.state.config.path),
                        ));
                        if let Err(e) = checkout_result {
                            error!("Failed to checkout head: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to find branch: {}. Logged error is: {}",
                            &actor.state.config.branch_name, e
                        );
                    }
                };
            } else {
                error!("Failed to find repository: {}", &actor.state.config.path);
            }
        });

        actor.setup.act_on::<NotifyChange>(|actor, _event| {
            if let Some(repo) = &actor.state.repository {
                let repo = repo.lock().expect("Couldn't lock repository mutex");
                let diff = if actor.state.config.watch_staged_only {
                    repo.diff_index_to_workdir(
                        Some(&repo.index().expect("Failed to get index")),
                        Some(&mut DiffOptions::new()),
                    )
                } else {
                    repo.diff_index_to_workdir(None, Some(&mut DiffOptions::new()))
                }
                    .expect("Failed to get diff");

                let mut diff_text = Vec::new();
                diff.print(git2::DiffFormat::Patch, |_, _, line| {
                    diff_text.extend_from_slice(line.content());
                    true
                })
                    .expect("Failed to print diff");

                let changes =
                    String::from_utf8(diff_text).expect("Failed to convert diff to string");
                trace!("Diff: {changes}");
            }
        });

        let context = actor.activate(None).await;
        match context {
            Ok(context) => {
                trace!(
                    "Activated RepositoryActor, attempting to checkout branch {}",
                    &config.branch_name
                );
                context.emit_async(CheckoutBranch).await;
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
    fn open_repository_to_branch(&self, branch_name: &str) -> anyhow::Result<()> {
        // Check if the repository is available
        if let Some(repository) = &self.repository {
            // Lock the repository mutex
            let repo = repository.lock().expect("Couldn't lock repository mutex");

            // Find the branch reference
            let branch_ref = repo
                .find_branch(branch_name, git2::BranchType::Local)?
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
    use std::sync::{Arc, Mutex};
    use akton::prelude::{ActorContext, Akton};
    use git2::{DiffOptions, Repository};
    use tracing::{error, info, trace};
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::path::Path;
    use tokio::sync::oneshot;
    use pretty_assertions::assert_eq;

    use crate::actors::repository_actor::RepositoryActor;
    use crate::init_tracing;
    use crate::messages::NotifyChange;
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
        let actor_context = RepositoryActor::init(&config).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.terminate().await?;
        Ok(())
    }


    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
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
            move |config: RepositoryConfig| {
                let sender = sender.clone(); // Clone the Arc again for use inside the async block
                async move {
                    let mut actor = Akton::<RepositoryActor>::create_with_id(&config.id);
                    let repo = Repository::open(&config.path).expect("Failed to open mock repo");
                    actor.state.repository = Some(Arc::new(Mutex::new(repo)));
                    actor.state.config = config;
                    trace!("Running test in {}", &actor.state.config.path);

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

        let actor_context = RepositoryActor::init_with_custom_behavior(test_behavior, config.clone()).await;
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

        trace!("Notifying actor of change");
        context.emit_async(NotifyChange).await?;

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

// Check if the result contains any content
        assert!(result.len() > 0);

// Assert that the actual diff matches the expected diff
        assert_eq!(expected_diff, &result);
        trace!("Removing test file");
        fs::remove_file(&file_path)?;

        context.terminate().await?;
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
        let actor_context = RepositoryActor::init(&config).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.terminate().await?;
        Ok(())
    }
}
