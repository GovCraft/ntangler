use std::any::TypeId;
use std::fmt::Debug;
use std::time::Duration;

use akton::prelude::Subscribable;
use akton::prelude::*;
use futures::FutureExt;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::actors::repositories::GitRepository;
use crate::actors::scribe::Scribe;
use crate::actors::OpenAi;
use crate::messages::{
    AcceptBroker, DiffCalculated, NotifyChange, NotifyError, RepositoryPollRequested,
    SubscribeBroker, SystemStarted,
};
use crate::models::config::RepositoryConfig;
use crate::models::config::TanglerConfig;
use crate::models::TangledRepository;

/// Tangler is the name of the app and acts as the main orchestration point of this command line app and manages repository actors and a broker.
#[derive(Default, Debug, Clone)]
pub(crate) struct Tangler {
    git_repositories: Vec<Context>,
    diff_watchers: Vec<Context>,
    llm_pool: Vec<Context>,
    scribe: Context,
    generator: Context,
}

impl Tangler {
    #[instrument(skip(tangler_config))]
    pub(crate) async fn init(tangler_config: TanglerConfig) -> anyhow::Result<(Context, Context)> {
        let mut akton: AktonReady = Akton::launch().into();
        let broker = akton.get_broker();
        let actor_config =
            ActorConfig::new(Arn::with_root("tangler")?, None, Some(broker.clone()))?;

        let actor_context = akton
            .spawn_actor_with_setup::<Tangler>(actor_config, |mut actor| {
                Box::pin(async move {
                    let broker = actor.akton.get_broker().clone();

                    actor.state.scribe =
                        Scribe::initialize("scribe".to_string(), &mut actor.akton).await;

                    let generator_config = ActorConfig::new(
                        Arn::with_root("generator").expect("Failed to create generator Aktor-Arn"),
                        None,
                        Some(broker.clone()),
                    )
                    .expect("Failed to create generator config");
                    actor.state.generator = OpenAi::initialize(generator_config, &mut actor.akton)
                        .await
                        .expect("Failed to initialize generator actor");

                    actor
                        .setup
                        .act_on_async::<DiffCalculated>(|actor, event| {
                            let context = actor.context.clone();
                            let message = event.message.clone();
                            Box::pin(async move {
                                trace!("Diff submitted for LLM pool");
                                context.emit_async(message, Some("llm_pool")).await
                            })
                        })
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
                                        tokio::time::sleep(Duration::from_secs(180)).await;
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

                    for repo in &tangler_config.repositories {
                        let akton = &mut actor.akton.clone();
                        trace!(repo = ?repo, "Initializing a repository actor.");
                        let broker = broker.clone();
                        let tangled_repository: TangledRepository = repo.clone().into();
                        let watcher = GitRepository::init(tangled_repository, akton)
                            .await
                            .expect("Failed to start repository watcher");
                        actor.state.git_repositories.push(watcher.clone());
                        debug!(actor = watcher.key, "init repository");
                    }

                    // let pool_size = tangler_config.repositories.len() * 5;
                    // let pool_builder = PoolBuilder::default().add_pool::<OpenAi>(
                    //     "llm_pool",
                    //     pool_size,
                    //     LoadBalanceStrategy::RoundRobin,
                    // );

                    actor.context.subscribe::<SystemStarted>().await;
                    actor.context.subscribe::<NotifyError>().await;
                    actor.context.subscribe::<NotifyChange>().await;
                    actor.context.subscribe::<DiffCalculated>().await;

                    // let actor_context = actor.activate(Some(pool_builder)).await;
                    //
                    // for _ in 0..pool_size {
                    //     trace!("Sending broker to LLM Pool.");
                    //     let broker = broker.clone();
                    //     actor_context
                    //         .emit_async(AcceptBroker { broker }, Some("llm_pool"))
                    //         .await;
                    // }
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
