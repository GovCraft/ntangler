use std::fmt::Debug;
use std::time::Duration;

use akton::prelude::*;
use akton::prelude::Subscribable;
use tracing::{debug, instrument, trace};

use crate::actors::{LlmClient};
use crate::actors::repositories::GitRepository;
use crate::actors::scribe::Scribe;
use crate::messages::{RepositoryPollRequested, SystemStarted};
use crate::models::config::NtanglerConfig;
use crate::models::NtangledRepository;

/// Tangler is the name of the app and acts as the main orchestration point of this command line app and manages repository actors and a broker.
#[derive(Default, Debug, Clone)]
pub(crate) struct Ntangler {
    git_repositories: Vec<Context>,
    scribe: Context,
    generator: Context,
}

impl Ntangler {
    #[instrument(skip(ntangler_config))]
    pub(crate) async fn initialize(
        ntangler_config: NtanglerConfig,
    ) -> anyhow::Result<(Context, Context)> {
        let mut akton: AktonReady = Akton::launch().into();
        let broker = akton.get_broker();
        let actor_config =
            ActorConfig::new(Arn::with_root("ntangler")?, None, Some(broker.clone()))?;

        let actor_context = akton
            .spawn_actor_with_setup::<Ntangler>(actor_config, |mut actor| {
                Box::pin(async move {
                    let broker = actor.akton.get_broker().clone();

                    actor.state.scribe =
                        Scribe::initialize("scribe".to_string(), &mut actor.akton).await;

                    let llm_config = ActorConfig::new(
                        Arn::with_root("llm_actor").expect("Failed to create generator Aktor-Arn"),
                        None,
                        Some(broker.clone()),
                    )
                        .expect("Failed to create generator config");
                    actor.state.generator = LlmClient::initialize(llm_config, &mut actor.akton)
                        .await
                        .expect("Failed to initialize generator actor");

                    actor
                        .setup
                        .act_on_async::<SystemStarted>(|actor, _event| {
                            let broker = actor.akton.get_broker().clone();
                            Box::pin(async move {
                                tokio::spawn(async move {
                                    let broker = broker.clone();
                                    debug!(broker_id = &broker.key, "Initiating polling");
                                    loop {
                                        let broker = broker.clone();
                                        broker
                                            .emit_async(
                                                BrokerRequest::new(RepositoryPollRequested),
                                                None,
                                            )
                                            .await;
                                        tokio::time::sleep(Duration::from_secs(10)).await;
                                        // Poll every 3 seconds
                                    }
                                });
                            })
                        })
                        .on_before_stop_async(|actor| {
                            let broker = actor.broker.clone();
                            Box::pin(async move {
                                broker
                                    .suspend_actor()
                                    .await
                                    .expect("Failed to shut down broker");
                            })
                        });

                    for repo in &ntangler_config.repositories {
                        let akton = &mut actor.akton.clone();
                        trace!(repo = ?repo, "Initializing a repository actor.");

                        let ntangled_repository: NtangledRepository = repo.clone().into();
                        let watcher = GitRepository::init(ntangled_repository, akton)
                            .await
                            .expect("Failed to start repository watcher");
                        actor.state.git_repositories.push(watcher.clone());
                        debug!(actor = watcher.key, "init repository");
                    }

                    actor.context.subscribe::<SystemStarted>().await;
                    actor.activate(None).await
                })
            })
            .await?;

        let broker_message = BrokerRequest::new(SystemStarted);
        //notify the system has started
        broker.emit_async(broker_message, None).await;

        Ok((actor_context, broker))
    }
}
