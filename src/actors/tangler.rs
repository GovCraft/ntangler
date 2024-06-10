use std::any::TypeId;
use std::fmt::Debug;

use akton::prelude::*;
use futures::FutureExt;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::actors::{OpenAi, Broker, GitSentinel};
use crate::actors::repositories::GitRepository;
use crate::messages::{AcceptParentBroker, BrokerSubscribe, Diff, ErrorNotification, NotifyChange, SubmitDiff, Observe};
use crate::repository_config::RepositoryConfig;
use crate::tangler_config::TanglerConfig;

/// Tangler manages repository actors and a broker.
#[akton_actor]
pub(crate) struct Tangler {
    git_repositories: Vec<Context>,
    diff_watchers: Vec<Context>,
    llm_pool: Vec<Context>,
    broker: Context,
}

impl Tangler {
    /// Initializes the Tangler with the given configuration.
    ///
    /// # Parameters
    /// - `tangler_config`: Configuration for the Tangler.
    ///
    /// # Returns
    /// - `anyhow::Result<(Context, Context)>`: A tuple containing the actor context and broker context.
    #[instrument(skip(tangler_config))]
    pub(crate) async fn init(tangler_config: TanglerConfig) -> anyhow::Result<(Context, Context)> {
        let mut actor = Akton::<Tangler>::create_with_id("tangler");

        trace!("Initializing the broker actor.");
        actor.state.broker = Broker::init().await?;
        let broker_context = actor.state.broker.clone();

        trace!("Setting up the error notification handler.");
        actor.setup
            .act_on_async::<SubmitDiff>(|actor, event| {
                let context = actor.context.clone();
                let message = event.message.clone();
                Box::pin(async move {
                    trace!("Diff submitted for LLM pool");
                    context.emit_async(message, Some("llm_pool")).await
                })
            })
            .act_on::<ErrorNotification>(|_, event| {
                let error_message = &event.message.error_message;
                warn!("Displayed user error: {:?}", &error_message);
                eprintln!("{}", error_message);
            })
            .on_before_stop_async(|actor| {
                let broker = actor.state.broker.clone();
                Box::pin(async move {
                    broker.suspend().await.expect("Failed shut down broker")
                })
            });

        for repo in &tangler_config.repositories {
            let broker = actor.state.broker.clone();

            trace!(repo = ?repo, "Initializing a repository actor.");
            if let Some(repo_actor) = GitRepository::init(repo, broker.clone()).await {
                actor.state.git_repositories.push(repo_actor);
            }
            trace!(repo = ?repo, "Initializing a diff watcher actor.");
            let watcher = GitSentinel::init(repo, broker).await?;
            watcher.emit_async(Observe, None).await;
            actor.state.diff_watchers.push(watcher);
        }
        let pool_size = tangler_config.repositories.len() * 3;
        let pool_builder = PoolBuilder::default().add_pool::<OpenAi>("llm_pool", pool_size, LoadBalanceStrategy::RoundRobin);
        let pool_broker = actor.state.broker.clone();
        trace!("Activating the Tangler actor.");
        let actor_context = actor.activate(Some(pool_builder)).await?;

        //pass the broker to the internal pool actors
        for _ in 0..pool_size {
            trace!("Sending broker to LLM Pool.");
            let broker = pool_broker.clone();
            actor_context.emit_async(AcceptParentBroker { broker }, Some("llm_pool")).await;
        }

        let subscription = BrokerSubscribe {
            subscriber_id: actor_context.key.value.clone(),
            message_type_id: TypeId::of::<ErrorNotification>(),
            subscriber_context: actor_context.clone(),
        };
        broker_context.emit_async(subscription, None).await;
        trace!(type_id=?TypeId::of::<ErrorNotification>(),"Subscribed to ErrorNotification:");

        let subscription = BrokerSubscribe {
            subscriber_id: actor_context.key.value.clone(),
            message_type_id: TypeId::of::<NotifyChange>(),
            subscriber_context: actor_context.clone(),
        };
        broker_context.emit_async(subscription, None).await;
        trace!(type_id=?TypeId::of::<NotifyChange>(),"Subscribed to NotifyChange:");

        let subscription = BrokerSubscribe {
            subscriber_id: actor_context.key.value.clone(),
            message_type_id: TypeId::of::<SubmitDiff>(),
            subscriber_context: actor_context.clone(),
        };
        broker_context.emit_async(subscription, None).await;
        trace!(type_id=?TypeId::of::<SubmitDiff>(),"Subscribed to SubmitDiff:");

        Ok((actor_context, broker_context))
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use akton::prelude::ActorContext;
    use lazy_static::lazy_static;
    use tracing::{debug, info, trace};

    use crate::actors::repositories::GitRepository;
    use crate::actors::Tangler;
    use crate::init_tracing;
    use crate::messages::ErrorNotification;
    use crate::repository_config::RepositoryConfig;
    use crate::tangler_config::TanglerConfig;

    lazy_static! {
    static ref CONFIG: RepositoryConfig = RepositoryConfig {
        path: "./mock-repo-working".to_string(),
        branch_name: "new_branch".to_string(),
        api_url: "".to_string(),
        watch_staged_only: false,
        id: "anyid".to_string(),
    };
        }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_broker_subscription() -> anyhow::Result<()> {
        init_tracing();
        let config = CONFIG.clone();
        let tangler_config = TanglerConfig {
            repositories: vec![config],
        };

        // Event: Tangler Initialization
        // Description: Initializing the Tangler with the given configuration.
        // Context: Tangler configuration details.
        info!(tangler_config = ?tangler_config, "Initializing the Tangler.");
        let (tangler, broker) = Tangler::init(tangler_config).await?;

        // Event: Constructing Error Notification Message
        // Description: Constructing an error notification message to broadcast through the broker.
        // Context: Error message details.
        info!("Constructing an error notification message.");
        let error_msg = ErrorNotification {
            error_message: "Hello world".to_string(),
        };

        // Event: Sending Message to Broker
        // Description: Sending the constructed message to the broker.
        // Context: Broker emit message details.
        info!("Sending the constructed message to the broker.");
        broker.emit_async(error_msg, None).await;

        // Event: Terminating Broker
        // Description: Terminating the broker actor.
        // Context: None
        info!("Terminating the broker actor.");
        broker.suspend().await?;

        // Event: Terminating Tangler Actor
        // Description: Terminating the Tangler actor.
        // Context: None
        info!("Terminating the Tangler actor.");
        tangler.suspend().await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_receives_error_notification() -> anyhow::Result<()> {
        init_tracing();

        let config = CONFIG.clone();
        let tangler_config = TanglerConfig {
            repositories: vec![config],
        };

        // this actor subscribes to ErrorNotification messages
        let (tangle, broker) = Tangler::init(tangler_config).await?;

        // get a copy of the context

        // construct a message to broadcast through the broker
        let error_msg = ErrorNotification { error_message: "Hello world".to_string() };

        // send the message to the broker
        tangle.emit_async(error_msg, None).await;


        tangle.suspend().await?;
        Ok(())
    }
}