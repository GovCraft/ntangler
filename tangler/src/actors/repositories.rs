use crate::messages::{
    CommitMessageGenerated, CommitPosted,  DiffQueued, FileChangeDetected,
    FinalizedCommit, RepositoryPollRequested, SystemStarted,
};
use crate::models::config::RepositoryConfig;
use crate::models::config::TanglerConfig;
use crate::models::{
    CommitType, CommittedCommit, Description, Filename, Oid, RepositoryEvent, Scope, TangledCommit,
    TangledRepository, TangledSignature, TimeStamp,
};
use akton::prelude::*;
use anyhow::anyhow;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use git2::{DiffOptions, Error, IndexAddOption, Repository, Status as GitStatus, Status, StatusOptions};
use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::any::TypeId;
use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

#[akton_actor]
pub(crate) struct GitRepository {
    repo_info: TangledRepository,
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
        config: TangledRepository,
        system: &mut AktonReady,
    ) -> anyhow::Result<Context> {
        // Define the default behavior as an async closure that takes a reference to the repository configuration.
        let mut system = system.clone();
        let default_behavior = |config: TangledRepository, system: AktonReady| async move {
            GitRepository::default_behavior(&config, system).await
        };

        // Call the `init_with_custom_behavior` function with the default behavior closure and the configuration.
        GitRepository::init_with_custom_behavior(default_behavior, config.clone(), system).await
    }

    pub(crate) async fn init_with_custom_behavior<F, Fut>(
        custom_behavior: F,
        config: TangledRepository,
        system: AktonReady,
    ) -> anyhow::Result<Context>
    where
        F: Fn(TangledRepository, AktonReady) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<Context>> + Send,
    {
        // Execute the custom behavior and await its result
        custom_behavior(config, system).await
    }

    /// Example custom behavior function to be passed into the `init` function.
    pub(crate) async fn default_behavior(
        tangled_repository: &TangledRepository,
        mut system: AktonReady,
    ) -> anyhow::Result<Context> {
        let akton = system.clone();
        let actor_name = Arn::with_account("repository")?
            .add_part(tangled_repository.akton_arn.root.to_string())?;

        let actor_config = ActorConfig::new(actor_name, None, Some(system.clone().get_broker()))
            .expect("Failed to build repository config");
        let mut actor = system
            .create_actor_with_config::<GitRepository>(actor_config)
            .await;
        actor.broker = system.get_broker().clone();
        // debug!(path = &tangled_repository.path, "Open repo '{}' at", &tangled_repository.nickname);
        let repo = Repository::open(&tangled_repository.path)?;

        actor.state.repo_info = tangled_repository.clone();

        actor
            .setup
            .act_on::<SystemStarted>(|actor, _event| {
                actor.state.broker = actor.akton.get_broker().clone();
            })
            .act_on_async::<RepositoryPollRequested>(|actor, event| {
                debug!(
                    sender = event.return_address.sender,
                    "Poll changes received for"
                );
                let reply_to = event.return_address.clone();
                let broker = actor.akton.get_broker().clone();
                let mut futures = actor.state.handle_poll_request(reply_to);
                actor.state.broadcast_futures(futures)
            })
            .act_on_async::<FileChangeDetected>(|actor, event| {
                let actor_context = actor.context.clone();
                let reply_to = event.return_address.clone();
                let repository_path = &actor.state.repo_info.path;
                let target_file = &event.message.path;

                let repo = Repository::open(repository_path).expect("Failed to open repository");

                let mut diff_options = DiffOptions::new();
                diff_options.pathspec(target_file.as_os_str());
                diff_options.include_untracked(true);
                diff_options.recurse_untracked_dirs(true);
                diff_options.disable_pathspec_match(true);

                // Generate the diff
                let diff = repo
                    .diff_index_to_workdir(None, Some(&mut diff_options))
                    .expect("nope");
                let mut diff_text = Vec::new();
                diff.print(git2::DiffFormat::Patch, |_, _, line| {
                    diff_text.extend_from_slice(line.content());
                    true
                })
                .expect("Failed to print diff");
                let changes = String::from_utf8_lossy(&diff_text).to_string();

                let repository_event = BrokerRequest::new(DiffQueued::new(
                    changes,
                    target_file.clone(),
                    actor.state.repo_info.nickname.clone(),
                    actor.context.clone(),
                ));
                let broker = actor.akton.get_broker().clone();
                Context::wrap_future(async move {
                    broker.emit_async(repository_event, None).await;
                })
            })
            .act_on_async::<CommitMessageGenerated>(|actor, event| {
                // Event: Received Commit Response
                // Description: Received a commit response and will commit changes to the repository.
                // Context: Commit message details.
                let message = &event.message;
                let repository_nickname = actor.state.repo_info.nickname.clone();
                let repository_path = &actor.state.repo_info.path;
                let repo = Repository::open(repository_path).expect("Failed to open repository");
                let broker = actor.akton.get_broker();

                // Received commit message, so we need to commit to this repo
                let commit_message = &message.commit_message;
                let target_file = message.target_file.clone();

                let sig = repo.signature().expect("Failed to get signature");

                // Stage all modified files
                let mut index = repo.index().expect("Failed to get index");
                trace!(file=?target_file, "Repo index add");
                index
                    .add_path(target_file.as_ref())
                    .expect("Failed to add files to index");
                index.write().expect("Failed to write index");

                let tree_id = index.write_tree().expect("Failed to write tree");
                let tree = repo.find_tree(tree_id).expect("Failed to find tree");
                let head = repo.head().expect("Failed to get HEAD");
                let parent_commit = head.peel_to_commit().expect("Failed to get parent commit");

                let when: TimeStamp = (&sig.when()).into();
                let message_string = &commit_message.to_string();
                // TODO: optionally sign commits
                let hash = repo
                    .commit(
                        Some("HEAD"),
                        &sig,
                        &sig,
                        message_string,
                        &tree,
                        &[&parent_commit],
                    )
                    .expect("Failed to commit");
                let hash = hash.to_string();

                let commit_message = commit_message.clone();
                Context::wrap_future(async move {
                    debug!("Local commit: {:?}", &target_file);
                    let broker = broker.clone();
                    let msg = FinalizedCommit::new(
                        when,
                        target_file.clone(),
                        repository_nickname,
                        hash,
                        commit_message,
                    );
                    broker.emit_async(BrokerRequest::new(msg), None).await;
                })
            });

        // actor.context.subscribe::<CommitEvent>().await;
        actor.context.subscribe::<SystemStarted>().await;
        actor.context.subscribe::<RepositoryPollRequested>().await;
        actor.context.subscribe::<FileChangeDetected>().await;
        actor.context.subscribe::<CommitMessageGenerated>().await;

        Ok(actor.activate(None).await)
    }

    #[instrument(skip(self, futures))]
    fn broadcast_futures<T>(
        &self,
        mut futures: FuturesUnordered<impl Future<Output = T> + Sized>,
    ) -> Pin<Box<impl Future<Output = ()> + Sized>> {
        // Event: Broadcasting Futures
        // Description: Broadcasting futures to be processed.
        // Context: Number of futures.
        trace!(
            futures_count = futures.len(),
            "Broadcasting futures to be processed."
        );

        Box::pin(async move {
            let mut i = 0;
            while futures.next().await.is_some() {
                i += 1;
            }
            // Event: Futures Broadcast Completed
            // Description: All futures have been processed.
            // Context: None
            debug!("{i} future(s) sent");
        })
    }
    pub(crate) fn handle_poll_request(
        &self,
        outbound_envelope: OutboundEnvelope,
    ) -> FuturesUnordered<impl Future<Output = ()> + 'static> {
        let self_key = &self.repo_info.akton_arn;
        debug!(self = self.repo_info.nickname, "Received Poll request");
        let mut futures = FuturesUnordered::new();
        let repository_path = &self.repo_info.path;
        let repo = Repository::open(repository_path).expect("Failed to open repository");

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.recurse_untracked_dirs(true);
        status_options.include_unreadable_as_untracked(true);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .expect("Couldn't get repo statuses");

        let modified_files: Vec<String> = statuses
            .iter()
            .filter(|f| f.status() != (Status::INDEX_DELETED | Status::WT_DELETED))
            .map(|entry| entry.path().unwrap().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        debug!("modified files vec {:?}", &modified_files);

        let signature = repo
            .signature()
            .expect("Obtaining a signature from the repo failed");
        let repository = self.repo_info.clone();
        let id = self.repo_info.nickname.clone();

        let signature: TangledSignature = signature.into();
        let broker = self.broker.clone();
        for file in modified_files {
            let broker = broker.clone();
            let outbound_envelope = outbound_envelope.clone();
            let signature = signature.clone();
            let oid = file.as_str().into();
            let repository = repository.clone();
            let actor_context = outbound_envelope.clone();
            let path = file.clone();
            let trace_id = id.clone();
            let tangled_commit = TangledCommit::new(oid, signature, None, None);
            let repository_event = FileChangeDetected::new(file.into());
            futures.push(async move {
                outbound_envelope.reply_async(repository_event, None).await;
            });

            debug!(
                repo_id = trace_id,
                path = path,
                "Submitted initializing event to broker."
            );
        }
        futures
    }
}
