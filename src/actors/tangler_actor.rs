use std::any::TypeId;
use crate::actors::repository_actor::RepositoryActor;
use crate::tangler_config::TanglerConfig;
use crate::repository_config::RepositoryConfig;
use akton::prelude::*;
use tracing::{debug, error, info, instrument, trace};
use crate::actors::BrokerActor;
use crate::messages::{BrokerEmit, BrokerSubscribe, ErrorNotification};


/// TanglerActor manages repository actors and a broker.
#[akton_actor]
pub(crate) struct TanglerActor {
    repository_actors: Vec<Context>,
    broker: Context,
}

impl TanglerActor {
    /// Initializes the TanglerActor with the given configuration.
    ///
    /// # Parameters
    /// - `tangler_config`: Configuration for the TanglerActor.
    ///
    /// # Returns
    /// - `anyhow::Result<(Context, Context)>`: A tuple containing the actor context and broker context.
    #[instrument(skip(tangler_config))]
    pub(crate) async fn init(tangler_config: TanglerConfig) -> anyhow::Result<(Context, Context)> {
        let mut actor = Akton::<TanglerActor>::create_with_id("tangler");

        // Event: Broker Initialization
        // Description: Initializing the broker actor.
        // Context: None
        info!("Initializing the broker actor.");
        actor.state.broker = BrokerActor::init().await?;
        let broker_context = actor.state.broker.clone();

        // Event: Setup Error Notification Handler
        // Description: Setting up the error notification handler.
        // Context: None
        info!("Setting up the error notification handler.");
        actor.setup.act_on::<ErrorNotification>(|_, event| {
            debug!("broker message received: {:?}", &event.message);
            debug!("Received message: {:?}", &event.message);
            let error_message = &event.message.error_message;
            eprintln!("{}", error_message);
        });

        for repo in &tangler_config.repositories {
            let broker = actor.state.broker.clone();

            // Event: Repository Actor Initialization
            // Description: Initializing a repository actor.
            // Context: Repository configuration details.
            info!(repo = ?repo, "Initializing a repository actor.");
            if let Some(repo_actor) = RepositoryActor::init(repo, broker).await {
                actor.state.repository_actors.push(repo_actor);
            }
        }

        // Event: Activating Tangler Actor
        // Description: Activating the Tangler actor.
        // Context: None
        info!("Activating the Tangler actor.");
        let actor_context = actor.activate(None).await?;

        // Event: Broker Subscription
        // Description: Subscribing to broker for error notifications.
        // Context: Subscription details.
        info!("Subscribing to broker for error notifications.");
        let subscription = BrokerSubscribe {
            message_type_id: TypeId::of::<ErrorNotification>(),
            subscriber_context: actor_context.clone(),
        };
        broker_context.emit_async(subscription).await;

        Ok((actor_context, broker_context))
    }
}
#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use akton::prelude::ActorContext;
    use lazy_static::lazy_static;
    use tracing::{debug, info, trace};
    use crate::actors::repository_actor::RepositoryActor;
    use crate::actors::TanglerActor;
    use crate::init_tracing;
    use crate::messages::{BrokerEmit, ErrorNotification};
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

        // Event: TanglerActor Initialization
        // Description: Initializing the TanglerActor with the given configuration.
        // Context: Tangler configuration details.
        info!(tangler_config = ?tangler_config, "Initializing the TanglerActor.");
        let (tangler, broker) = TanglerActor::init(tangler_config).await?;

        // Event: Constructing Error Notification Message
        // Description: Constructing an error notification message to broadcast through the broker.
        // Context: Error message details.
        info!("Constructing an error notification message.");
        let error_msg = ErrorNotification {
            error_message: "Hello world".to_string(),
        };
        let message_type_id = TypeId::of::<ErrorNotification>();
        let broker_emit_msg = BrokerEmit::Error(error_msg);

        // Event: Sending Message to Broker
        // Description: Sending the constructed message to the broker.
        // Context: Broker emit message details.
        info!("Sending the constructed message to the broker.");
        broker.emit_async(broker_emit_msg).await;

        // Event: Terminating Broker
        // Description: Terminating the broker actor.
        // Context: None
        info!("Terminating the broker actor.");
        broker.terminate().await?;

        // Event: Terminating Tangler Actor
        // Description: Terminating the Tangler actor.
        // Context: None
        info!("Terminating the Tangler actor.");
        tangler.terminate().await?;

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
        let (tangle, broker) = TanglerActor::init(tangler_config).await?;

        // get a copy of the context

        // construct a message to broadcast through the broker
        let error_msg = ErrorNotification { error_message: "Hello world".to_string() };

        // send the message to the broker
        tangle.emit_async(error_msg).await;


        tangle.terminate().await?;
        Ok(())
    }
}