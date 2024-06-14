use std::any::TypeId;
use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use akton::prelude::*;
use anyhow::anyhow;
use git2::{DiffOptions, Error, IndexAddOption, Repository, Status, StatusOptions};
use git2::FileMode::Commit;
use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use tracing::{debug, error, info, trace, warn};

use crate::messages::{
    CommitMessageGenerated, CommitSuccess, DiffCalculated, NotifyChange, PollChanges,
    SubscribeBroker, SystemStarted,
};
use crate::models::{CommitType, Description, Oid, Scope};
use crate::models::config::RepositoryConfig;
use crate::models::config::TanglerConfig;

#[akton_actor]
pub(crate) struct GitRepository {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
    broker: Context,
    watching: AtomicBool,
}

impl GitRepository {
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
            GitRepository::default_behavior(&config, broker.clone()).await
        };

        // Call the `init_with_custom_behavior` function with the default behavior closure and the configuration.
        GitRepository::init_with_custom_behavior(default_behavior, config.clone(), broker).await
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
    pub(crate) async fn default_behavior(
        config: &RepositoryConfig,
        broker: Context,
    ) -> Option<Context> {
        let actor_config = ActorConfig::new(config.id.clone(), None, None);
        let mut actor = Akton::<GitRepository>::create_with_config(actor_config);
        actor.state.config = config.clone();
        actor.state.broker = broker;
        trace!(path = actor.state.config.path, "Open repo at");
        let repo = match Repository::open(&config.path) {
            Ok(repo) => repo,
            Err(e) => {
                use git2::ErrorCode::*;
                let error_message = match e.code() {
                    NotFound => format!(
                        "Oops! Couldn't find the requested object in the repository at {}. Check the path and try again. More details are in the local log.",
                        config.path
                    ),
                    BareRepo => format!(
                        "Heads up! The operation isn't allowed on a bare repository at {}. Please check your repository settings. More details are available in the local log.",
                        config.path
                    ),
                    UnbornBranch => format!(
                        "It looks like the branch at {} has no commits yet. Please commit something first. More details are available in the local log.",
                        config.path
                    ),
                    Unmerged => format!(
                        "There's an ongoing merge at {} that's preventing the operation. Please resolve the merge and try again. More details are available in the local log.",
                        config.path
                    ),
                    NotFastForward => format!(
                        "The reference at {} isn't fast-forwardable. Please perform a merge or rebase. More details are available in the local log.",
                        config.path
                    ),
                    Conflict => format!(
                        "Checkout conflicts are preventing the operation at {}. Please resolve them and try again. More details are available in the local log.",
                        config.path
                    ),
                    Auth => format!(
                        "Authentication error while accessing the repository at {}. Please check your credentials. More details are available in the local log.",
                        config.path
                    ),
                    Certificate => format!(
                        "The server certificate is invalid for the repository at {}. Please verify the certificate. More details are available in the local log.",
                        config.path
                    ),
                    MergeConflict => format!(
                        "A merge conflict exists at {} and we can't continue. Please resolve the conflict and try again. More details are available in the local log.",
                        config.path
                    ),
                    IndexDirty => format!(
                        "Unsaved changes in the index at {} would be overwritten. Please commit or stash your changes. More details are available in the local log.",
                        config.path
                    ),
                    _ => format!(
                        "An internal error occurred while accessing the repository at {}. Please check the local log for more details.",
                        config.path
                    ),
                };
                error!("{}", error_message);
                return None;
            }
        };

        actor.state.repository = Some(Arc::new(Mutex::new(repo)));


        actor.setup.act_on::<SystemStarted>(|actor, _event| {
            let broker = actor.state.broker.clone();
            trace!(
                    "Activated RepositoryActor, attempting to checkout branch {}",
                    &actor.state.config.branch_name
                );
            actor.state.checkout_branch();
        })
            .act_on_async::<PollChanges>(|actor, _event| {
                // use crate::models::Commit;
                // let mut rng = thread_rng();
                // let commit_types = ["fix", "feat", "other"];
                // let commit_type_str = commit_types.choose(&mut thread_rng()).unwrap();
                // let commit_type = CommitType::from(*commit_type_str);
                //
                // let repository = [
                //     "tangler", "akton", "website"
                // ];
                // let scope = [
                //     "api", "parser", "security", "docs", "actors", "readme", "models"
                // ];
                //
                // let lorem_ipsum = [
                //     "Lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", "do",
                // ];
                //
                // let mut rng = thread_rng();
                // let num_words = rng.gen_range(3..=5); // Randomly choose between 3 and 5 words
                // let mut repo_id = thread_rng();
                // let repository = repository.choose(&mut repo_id).unwrap().to_string();
                // let scope = scope.choose(&mut repo_id).unwrap().to_string();
                // let random_text: Vec<&str> = lorem_ipsum.choose_multiple(&mut rng, num_words).cloned().collect();
                // let random_text = random_text.join(" ");
                // let random_bool = rand::thread_rng().gen_bool(0.1);
                // //temporary simulation
                // let broker = actor.state.broker.clone();
                // Box::pin(async move {
                //     let commit = crate::models::Commit {
                //         repository,
                //         commit_type: commit_type.clone(),
                //         scope: Some(Scope::from(scope.as_str())),
                //         description: Description::from(random_text.as_str()),
                //         body: "".to_string(),
                //         is_breaking: random_bool,
                //         footers: vec![],
                //         semver_impact: Commit::calculate_semver_impact(&commit_type, random_bool),
                //         ..Default::default()
                //     };
                //     let hash: String = rand::thread_rng()
                //         .sample_iter(&Alphanumeric)
                //         .take(7)
                //         .map(char::from)
                //         .collect();
                //     let msg = CommitSuccess::new(commit, hash.to_string().to_lowercase());
                //     broker.emit_async(msg, None).await;
                // })
                debug!(actor=actor.state.config.id,"Received Poll request");
                let mut diffs: Vec<(String, String)> = Default::default();

                if let Some(repo) = &actor.state.repository {
                    let repo = repo.lock().expect("Couldn't lock repository mutex");
                    let mut status_options = StatusOptions::new();
                    status_options.include_untracked(true);

                    let statuses = repo.statuses(Some(&mut status_options)).expect("Couldn't get repo statuses");
                    let modified_files: Vec<_> = statuses
                        .iter()
                        .filter(|entry| entry.status().contains(Status::WT_MODIFIED))
                        .map(|entry| entry.path().unwrap().to_string())
                        .collect::<HashSet<_>>() // Collect into a HashSet to remove duplicates
                        .into_iter() // Convert back to an iterator
                        .collect(); // Collect into a Vec
                    debug!("modified files vec {:?}", &modified_files);
                    // notify for each file
                    for path in modified_files {
                        debug!(change_file=&path, "Unstaged files");

                        trace!(file=?path, "Generating a diff for repository");

                        let mut diff_options = DiffOptions::new();

                        // Get the relative path by stripping the repository root prefix
                        debug!(file_name=?path, "Adding file to pathspec");

                        // Set the pathspec to the relative path
                        // this is where we've been failing
                        diff_options.pathspec(path.clone());
                        diff_options.include_untracked(true);

                        let mut index = repo.index().unwrap();
                        debug!(path=?index.path().unwrap(), "Diffing against index:");
                        trace!(working_dir=?repo.workdir().unwrap(), "...in");

                        // Generate the diff
                        let diff = repo.diff_index_to_workdir(None, Some(&mut diff_options)).expect("nope");
                        let mut diff_text = Vec::new();
                        diff.print(git2::DiffFormat::Patch, |_, _, line| {
                            diff_text.extend_from_slice(line.content());
                            true
                        }).expect("Failed to print diff");

                        // trace!(raw_diff = ?diff_text, "Raw diff generated for the repository.");
                        let changes = String::from_utf8_lossy(&diff_text).to_string();
                        // trace!(diff = changes, "Diff generated for the repository.");
                        diffs.push((path.clone(), changes));
                    }
                }
                let id = actor.key.value.clone();
                let broker = actor.state.broker.clone();
                let path = actor.state.config.path.clone();
                Box::pin(async move {
                    for (file, diff) in diffs {
                        let id = id.clone();
                        let broker = broker.clone();
                        let path = file.clone();
                        let trace_id = id.clone();
                        let diff = diff.to_string();

                        debug_assert_ne!(diff, "./".to_string());
                        broker.emit_async(DiffCalculated { diff, id, path: path.clone() }, None).await;
                        tracing::debug!(repo_id = trace_id, path=path, "Submitted retrieved Diff to broker.");
                    }
                })
            })
            .act_on_async::<CommitMessageGenerated>(|actor, event| {
                // Event: Received Commit Response
                // Description: Received a commit response and will commit changes to the repository.
                // Context: Commit message details.
                trace!(commit_message = ?event.message.commit, "Received ReponseCommit message and will attempt to commit changes to the repository.");

                let broker = actor.state.broker.clone();
                // Received commit message, so we need to commit to this repo
                let hash = {
                    if let Some(repo) = &actor.state.repository {
                        let response_commit = event.message;
                        let commit_message = &event.message.commit;
                        let repo = repo.lock().expect("Failed to lock repo mutex");
                        let target_file = event.message.path.clone();

                        // Get the repository root directory
                        let repo_root = repo.workdir().unwrap();
                        // Get the canonical path of the repository root
                        let repo_root_canonical = repo_root.canonicalize().unwrap();
                        trace!(file=target_file, "Committing");

                        let sig = repo.signature().expect("Failed to get signature");
                        let path = PathBuf::from(target_file.clone());

                        // Stage all modified files
                        let mut index = repo.index().expect("Failed to get index");
                        trace!(file=?path, "Repo index add");
                        index.add_path(path.as_ref()).expect("Failed to add files to index");
                        index.write().expect("Failed to write index");

                        let tree_id = index.write_tree().expect("Failed to write tree");
                        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
                        let head = repo.head().expect("Failed to get HEAD");
                        let parent_commit = head.peel_to_commit().expect("Failed to get parent commit");

                        let commit_message = &commit_message.to_string();
                        // TODO: optionally sign commits
                        let hash = repo.commit(
                            Some("HEAD"),
                            &sig,
                            &sig,
                            commit_message,
                            &tree,
                            &[&parent_commit],
                        ).expect("Failed to commit");
                        hash.to_string()
                    } else {
                        "".to_string()
                    }
                };

                let commit = event.message.commit.clone();
                Box::pin(async move {
                    debug!("Local commit: {:?}", &commit);
                    let broker = broker.clone();
                    broker.emit_async(CommitSuccess::new(commit, hash), None).await;
                })
            });

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<CommitMessageGenerated>(),
            subscriber_context: actor.context.clone(),
        };

        actor.state.broker.emit_async(subscription, None).await;
        trace!(type_id=?TypeId::of::<CommitMessageGenerated>(),subscriber=actor.key.value.clone(),"Subscribed to ResponseCommit:");

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<PollChanges>(),
            subscriber_context: actor.context.clone(),
        };
        trace!(type_id=?TypeId::of::<PollChanges>(),subscriber=actor.key.value.clone(),"Subscribed to Poll:");

        actor.state.broker.emit_async(subscription, None).await;

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<SystemStarted>(),
            subscriber_context: actor.context.clone(),
        };
        trace!(type_id=?TypeId::of::<SystemStarted>(),subscriber=actor.key.value.clone(),"Subscribed to SystemStarted:");

        actor.state.broker.emit_async(subscription, None).await;

        let context = actor.activate(None).await;

        match context {
            Ok(context) => Some(context),
            Err(_) => {
                error!(
                    "Whoops! Something went wrong while activating the RepositoryActor for the repository at {}. Don't worry, more details are available in the local log. Hang in there!",
                    &config.path
                );
                None
            }
        }
    }

    fn checkout_branch(&mut self) {
        if let Some(repository) = &self.repository {
            match repository.lock() {
                Ok(repo) => {
                    match repo.find_branch(&self.config.branch_name, git2::BranchType::Local) {
                        Ok(_branch_ref) => {
                            trace!("Found branch: {}", &self.config.branch_name);
                            let checkout_result = repo.checkout_head(Some(
                                git2::build::CheckoutBuilder::new().path(&self.config.path),
                            ));
                            match checkout_result {
                                Ok(_) => {
                                    // Set the repo to this branch
                                }
                                Err(e) => {
                                    error!("Oops! Couldn't switch to the branch head: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Uh-oh! Couldn't find the branch: {}. Error details: {}",
                                &self.config.branch_name, e
                            );
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Yikes! The repository is locked and couldn't be accessed: {}",
                        e
                    );
                }
            }
        } else {
            error!(
                "Oh no! Couldn't locate the repository at: {}",
                &self.config.path
            );
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

    use akton::prelude::{ActorConfig, ActorContext, Akton, Context};
    use anyhow::anyhow;
    use git2::{DiffOptions, Repository};
    use pretty_assertions::assert_eq;
    use tokio::sync::oneshot;
    use tracing::{error, info, trace};

    use crate::actors::Broker;
    use crate::actors::repositories::GitRepository;
    use crate::init_tracing;
    use crate::messages::{CommitMessageGenerated, NotifyChange};
    use crate::models::config::RepositoryConfig;

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

        let actor_context = GitRepository::init(&config, broker).await;
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
                    let actor_config = ActorConfig::new(&config.id, None, None);
                    let mut actor = Akton::<GitRepository>::create_with_config(actor_config);
                    let repo = Repository::open(&config.path).expect("Failed to open mock repo");
                    actor.state.repository = Some(Arc::new(Mutex::new(repo)));
                    actor.state.config = config;
                    trace!("Running test in {}", &actor.state.config.path);
                    actor.state.open_repository_to_branch();
                    actor.setup.act_on::<NotifyChange>(move |actor, _event| {
                        info!("Received NotifyChange message");
                        if let Some(repo) = &actor.state.repository {
                            let repo = repo.lock().expect("Couldn't lock repository mutex");
                            let diff = repo
                                .diff_index_to_workdir(None, Some(&mut DiffOptions::new()))
                                .expect("Failed to get diff");
                            let mut diff_text = Vec::new();
                            diff.print(git2::DiffFormat::Patch, |_, _, line| {
                                diff_text.extend_from_slice(line.content());
                                true
                            })
                                .expect("Failed to print diff");
                            let changes = String::from_utf8(diff_text)
                                .expect("Failed to convert diff to string");
                            let mut sender_guard = sender.lock().unwrap();
                            if let Some(sender) = sender_guard.take() {
                                // Properly take the sender out of the Option
                                sender.send(changes).expect("Couldn't send test diff");
                            } else {
                                error!("Sender has already been taken or was never initialized.");
                            }
                        } else {
                            error!("No test repo found.");
                        }
                    });

                    let context = actor
                        .activate(None)
                        .await
                        .expect("Couldn't activate RepositoryActor");
                    Some(context)
                }
            }
        };
        let broker = Broker::init().await?;

        let actor_context =
            GitRepository::init_with_custom_behavior(test_behavior, config.clone(), broker).await;
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
        let path = "test_file.txt".to_string();
        trace!("Notifying actor of change");
        context
            .emit_async(NotifyChange { repo_id, path }, None)
            .await;

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

        let actor_context = GitRepository::init(&config, broker).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.suspend().await?;
        Ok(())
    }
}
