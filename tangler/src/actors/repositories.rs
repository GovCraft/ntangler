use std::any::TypeId;
use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use akton::prelude::*;
use anyhow::anyhow;
use git2::{DiffOptions, Error, IndexAddOption, Repository, StatusOptions, Status as GitStatus};
use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use tracing::{debug, error, info, trace, warn};

use crate::messages::{
    CommitAuthoring, CommitEvent, CommitEventCategory, CommitMessageGenerated, CommitPending,
    CommitPosted, DiffCalculated, NotifyChange, PollChanges, SubscribeBroker, SystemStarted,
};
use crate::models::config::RepositoryConfig;
use crate::models::config::TanglerConfig;
use crate::models::{CommittedCommit, CommitType, Description, Filename, CommitMessageGeneratedCommit, Oid, Scope, PendingCommit, DiffGeneratedCommit, Status};

#[akton_actor]
pub(crate) struct GitRepository {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
    broker: Context,
    watching: AtomicBool,
    pending: Vec<CommittedCommit>,
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
    pub(crate) async fn init(
        config: &RepositoryConfig,
        system: &mut AktonReady,
    ) -> anyhow::Result<Context> {
        // Define the default behavior as an async closure that takes a reference to the repository configuration.
        let mut system = system.clone();
        let default_behavior = |config: RepositoryConfig, system: AktonReady| async move {
            GitRepository::default_behavior(&config, system).await
        };

        // Call the `init_with_custom_behavior` function with the default behavior closure and the configuration.
        GitRepository::init_with_custom_behavior(default_behavior, config.clone(), system).await
    }

    pub(crate) async fn init_with_custom_behavior<F, Fut>(
        custom_behavior: F,
        config: RepositoryConfig,
        system: AktonReady,
    ) -> anyhow::Result<Context>
    where
        F: Fn(RepositoryConfig, AktonReady) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=anyhow::Result<Context>> + Send,
    {
        // Execute the custom behavior and await its result
        custom_behavior(config, system).await
    }

    /// Example custom behavior function to be passed into the `init` function.
    pub(crate) async fn default_behavior(
        config: &RepositoryConfig,
        mut system: AktonReady,
    ) -> anyhow::Result<Context> {
        let akton = system.clone();
        let actor_name = Arn::with_account("repository")?.add_part(&config.id)?;

        let actor_config = ActorConfig::new(actor_name, None, Some(system.clone().get_broker()))
            .expect("Failed to build repository config");
        let mut actor = system
            .create_actor_with_config::<GitRepository>(actor_config)
            .await;

        debug!(path = &config.path, "Open repo '{}' at", &config.nickname);
        let repo = Repository::open(&config.path)?;

        actor.state.repository = Some(Arc::new(Mutex::new(repo)));
        actor.state.config = config.clone();

        actor.setup.act_on::<SystemStarted>(|actor, _event| {
            let broker = actor.state.broker.clone();
            if !&actor.state.config.branch_name.is_empty() {
                debug!(
                    "Activated RepositoryActor, attempting to checkout branch {}",
                    &actor.state.config.branch_name
                );
                actor.state.checkout_branch();
            } else {
                error!("Repository config branch name was empty!")
            }
        })
            .act_on_async::<PollChanges>(|actor, _event| {
                let actor_key = &actor.key;
                debug!(actor=actor.state.config.nickname,"Received Poll request");
                let mut modified_files = Vec::new();
                if let Some(repo) = &actor.state.repository {
                    let repo = repo.lock().expect("Couldn't lock repository mutex");
                    let mut status_options = StatusOptions::new();
                    status_options.include_untracked(true);

                    let statuses = repo.statuses(Some(&mut status_options)).expect("Couldn't get repo statuses");
                    modified_files = statuses
                        .iter()
                        .filter(|entry| entry.status().contains(GitStatus::WT_MODIFIED))
                        .map(|entry| entry.path().unwrap().to_string())
                        .collect::<HashSet<_>>() // Collect into a HashSet to remove duplicates
                        .into_iter() // Convert back to an iterator
                        .collect(); // Collect into a Vec
                    debug!("modified files vec {:?}", &modified_files);
                }
                let id = actor.state.config.nickname.clone();
                let broker = actor.akton.get_broker();
                let path = actor.state.config.path.clone();
                Box::pin(async move {
                    for file in modified_files {
                        warn!("*");
                        let id = id.clone();
                        let broker = broker.clone();
                        let path = file.clone();
                        let trace_id = id.clone();

                        let msg= CommitEvent::new(CommitEventCategory::FilePending(PendingCommit::new(id, Filename::new(&path))));
                        broker.emit_async(BrokerRequest::new(msg), None).await;
                        // broker.emit_async(BrokerRequest::new(DiffCalculated { diff, id, path: path.clone() }), None).await;
                        tracing::debug!(repo_id = trace_id, path=path, "Submitted retrieved Diff to broker.");
                    }
                })
            })
            .act_on_async::<CommitEvent>(|actor, event| {
                //temporary simulation
                use crate::models::CommittedCommit;
                let broker = actor.akton.get_broker().clone();

                match &event.message.category {
                    CommitEventCategory::FilePending(pending_commit) => {
                        //make sure this message is for this repo
                        if event.message.id != actor.state.config.id {
                            trace!(message_id=event.message.id,repo_id=actor.state.config.id,"Rejecting message meant for a different actor");
                            return Box::pin(async move {  });
                        }
                        let mut rng = thread_rng();
                        let pending_commit = pending_commit.clone();
                        let pending_commit = pending_commit.clone();
                        let path = pending_commit.filename;
                        let broker = broker.clone();
                        let nickname = actor.state.config.nickname.clone();
                        let id = actor.state.config.id.clone();
                        if let Some(repo) = &actor.state.repository {
                            let repo = repo.lock().expect("Couldn't lock repository mutex");

                            debug!(change_file=?&path, "Unstaged files");

                            trace!(file=?path, "Generating a diff for repository");

                            let mut diff_options = DiffOptions::new();

                            // Get the relative path by stripping the repository root prefix
                            debug!(file_name=?path, "Adding file to pathspec");

                            // Set the pathspec to the relative path
                            // this is where we've been failing
                            diff_options.pathspec(path.to_string().clone());
                            diff_options.include_untracked(true);
                            diff_options.recurse_untracked_dirs(true);

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
                            let diff_event = DiffGeneratedCommit::new(id, changes, path.to_string(), nickname, Status::Thinking);
                            // trace!(diff = changes, "Diff generated for the repository.");
                            Box::pin(async move {
                                tokio::time::sleep(Duration::from_secs(4)).await; // Poll every 3 seconds

                                let msg = CommitEvent::new(CommitEventCategory::DiffGenerated(diff_event));
                                broker.emit_async(BrokerRequest::new(msg), None).await;
                            })
                        } else {
                            Box::pin(async move {})
                        }
                    },
                    _ => { Box::pin(async move {}) }
                }
            })
            .act_on_async::<CommitMessageGenerated>(|actor, event| {
                if event.message.id != actor.state.config.id {
                    trace!(message_id=event.message.id,repo_id=actor.state.config.id,"Rejecting message meant for a different actor");
                    return Box::pin(async move {  });
                }

                // Event: Received Commit Response
                // Description: Received a commit response and will commit changes to the repository.
                // Context: Commit message details.
                trace!(commit_message = ?event.message.commit, "Received ReponseCommit message and will attempt to commit changes to the repository.");

                let broker = actor.state.broker.clone();
                // Received commit message, so we need to commit to this repo
                let hash = {
                    if let Some(repo) = &actor.state.repository {
                        let response_commit = &event.message;
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
                        // let hash = repo.commit(
                        //     Some("HEAD"),
                        //     &sig,
                        //     &sig,
                        //     commit_message,
                        //     &tree,
                        //     &[&parent_commit],
                        // ).expect("Failed to commit");
                        let hash= "demohash";
                        hash.to_string()
                    } else {
                        "".to_string()
                    }
                };

                let commit = event.message.commit.clone();
                Box::pin(async move {
                    debug!("Local commit: {:?}", &commit);
                    let broker = broker.clone();
                    let msg = CommitEvent::new(CommitEventCategory::FileCommitted(commit));
                    broker.emit_async(BrokerRequest::new(msg), None).await;
                })
            });

        actor.context.subscribe::<CommitMessageGenerated>().await;
        actor.context.subscribe::<CommitEvent>().await;
        actor.context.subscribe::<PollChanges>().await;
        actor.context.subscribe::<SystemStarted>().await;

        Ok(actor.activate(None).await)
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

            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }
        Ok(())
    }
}
