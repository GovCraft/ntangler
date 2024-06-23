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
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use tracing::{debug, error, info, trace, warn};

use crate::messages::{CommitEventCategory, CommitAuthoring, CommitEvent, CommitMessageGenerated, CommitPending, CommitPosted, DiffCalculated, NotifyChange, PollChanges, SubscribeBroker, SystemStarted};
use crate::models::{Commit, CommitType, Description, Filename, GeneratingCommit, Oid, Scope};
use crate::models::config::RepositoryConfig;
use crate::models::config::TanglerConfig;

#[akton_actor]
pub(crate) struct GitRepository {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
    broker: Context,
    watching: AtomicBool,
    pending: Vec<Commit>,
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
    pub(crate) async fn init(config: &RepositoryConfig, system: &mut AktonReady) -> anyhow::Result<Context> {
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

        let actor_config = ActorConfig::new(actor_name, None, Some(system.clone().get_broker())).expect("Failed to build repository config");
        let mut actor = system.create_actor_with_config::<GitRepository>(actor_config).await;

        debug!(path = &config.path, "Open repo '{}' at", &config.nickname);
        let repo = Repository::open(&config.path)?;

        actor.state.repository = Some(Arc::new(Mutex::new(repo)));
        actor.state.config = config.clone();


        actor.setup.act_on::<SystemStarted>(|actor, _event| {
            let broker = actor.state.broker.clone();
            if !&actor.state.config.branch_name.is_empty() {
                trace!(
                    "Activated RepositoryActor, attempting to checkout branch {}",
                    &actor.state.config.branch_name
                );
                actor.state.checkout_branch();
            } else {
                error!("Repository config branch name was empty!")
            }
        })
            .act_on_async::<CommitEvent>(|actor, event| {
                //temporary simulation
                use crate::models::Commit;
                let broker = actor.akton.get_broker().clone();

                match &event.message.category {
                    CommitEventCategory::Pending(pending_commit) => {
                        let mut rng = thread_rng();
                        let commit_types = ["fix", "feat", "other"];
                        let commit_type_str = commit_types.choose(&mut thread_rng()).unwrap();
                        let commit_type = CommitType::from(*commit_type_str);

                        let repository = actor.state.config.id.clone();
                        let scope = [
                            "api", "parser", "security", "docs", "actors", "readme", "models"
                        ];

                        let lorem_ipsum = [
                            "Lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", "do",
                        ];

                        let mut rng = thread_rng();
                        let num_words = rng.gen_range(3..=5); // Randomly choose between 3 and 5 words
                        let mut repo_id = thread_rng();
                        let scope = scope.choose(&mut repo_id).unwrap().to_string();
                        let random_text: Vec<&str> = lorem_ipsum.choose_multiple(&mut rng, num_words).cloned().collect();
                        let random_text = random_text.join(" ");
                        let random_bool = rand::thread_rng().gen_bool(0.1);

                        let pending_commit = pending_commit.clone();

                        Box::pin(async move {
                            let pending_commit = pending_commit.clone();
                            let authoring_commit: GeneratingCommit = pending_commit.clone().into();
                            let broker = broker.clone();
                            tokio::time::sleep(Duration::from_secs(4)).await; // Poll every 3 seconds
                            let msg = CommitEvent::new(CommitEventCategory::Generating(authoring_commit));
                            broker.emit_async(BrokerRequest::new(msg), None).await;

                            tokio::time::sleep(Duration::from_secs(1)).await; // Poll every 3 seconds
                            tokio::spawn(async move {
                                let broker = broker.clone();
                                let hash: String = rand::thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(7)
                                    .map(char::from)
                                    .collect();
                                let oid = Oid::from(hash.as_str());
                                let mut commit = crate::models::Commit {
                                    repository,
                                    commit_type: commit_type.clone(),
                                    scope: Some(Scope::from(scope.as_str())),
                                    description: Description::from(random_text.as_str()),
                                    body: "".to_string(),
                                    is_breaking: random_bool,
                                    footers: vec![],
                                    semver_impact: Commit::calculate_semver_impact(&commit_type, random_bool),
                                    oid,
                                    ..Default::default()
                                };
                                commit.set_id(pending_commit.repository.clone(), &pending_commit.filename);
                                let msg = CommitEvent::new(CommitEventCategory::Commit(commit));
                                tracing::debug!("Sending CommitPending");
                                broker.emit_async(BrokerRequest::new(msg), None).await;
                            });
                        })
                    }
                    _ => { Box::pin(async move {}) }
                }
            })
            .act_on_async::<PollChanges>(|actor, _event| {
                let actor_key = &actor.key;
                #[cfg(feature = "demo")]
                {
                    trace!(actor_id=&actor.key,"Received PollChanges from broker:");
                    //randomly have something to emit
                    let random_bool = rand::thread_rng().gen_bool(0.5);
                    if random_bool {
                        return Box::pin(async move {});
                    }
                    use crate::models::Commit;
                    let mut rng = thread_rng();
                    let filename = [
                        "main.rs",        // Rust
                        "lib.rs",         // Rust
                        "mod.rs",         // Rust
                        "config.ts",      // TypeScript
                        "utils.ts",       // TypeScript
                        "handlers.ts",    // TypeScript
                        "app.py",         // Python
                        "models.py",      // Python
                        "services.py",    // Python
                        "errors.py"       // Python
                    ];
                    let filename_str = filename.choose(&mut thread_rng()).unwrap();
                    let filename = Filename::from(*filename_str);
                    let repository = actor.state.config.nickname.clone();

                    //temporary simulation
                    let broker = actor.akton.get_broker().clone();
                    let actor_key = actor_key.clone();
                    Box::pin(async move {
                        let commit = crate::models::PendingCommit::new(
                            repository,
                            filename,
                        );
                        let hash: String = rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect();
                        let msg = CommitEvent::new(CommitEventCategory::Pending(commit));
                        debug!(actor_id=&actor_key,"Sending CommitEvent(Pending):");
                        broker.emit_async(BrokerRequest::new(msg), None).await;
                    })
                }

                #[cfg(not(feature = "demo"))]
                {
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
                            // trace!(diff = changes, "Diff generated for the repository.");
                            diffs.push((path.clone(), changes));
                        }
                    }
                    let id = actor.key.clone();
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
                }
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
                    let msg = CommitEvent::new(CommitEventCategory::Commit(commit));
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
            let checkout_result =
                repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }
        Ok(())
    }
}