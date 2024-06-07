use std::any::TypeId;
use std::fmt::Debug;

use akton::prelude::*;
use futures::FutureExt;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::actors::{BrokerActor, RepositoryWatcherActor};
use crate::actors::repository_actor::RepositoryActor;
use crate::messages::{BrokerSubscribe, Diff, ErrorNotification, NotifyChange, Watch};
use crate::repository_config::RepositoryConfig;
use crate::tangler_config::TanglerConfig;

/// TanglerActor manages repository actors and a broker.
#[akton_actor]
pub(crate) struct TanglerActor {
    git_repositories: Vec<Context>,
    diff_watchers: Vec<Context>,
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
            let error_message = &event.message.error_message;
            error!("Displayed error: {:?}", &error_message);
            eprintln!("{}", error_message);
        })
            .act_on_async::<NotifyChange>(|actor, event| {
                let repo_id = &event.message.repo_id;

                // Event: Change Detected
                // Description: A change has been detected in the repository.
                // Context: Repository ID.
                info!(repo_id = ?repo_id, "Change detected in repo: {:?}", repo_id);

                let repo = actor.state.git_repositories
                    .iter()
                    .find(|g| g.key.value.contains(repo_id))
                    .cloned();

                if let Some(repo) = repo {
                    // Event: Emitting Diff
                    // Description: Emitting a Diff message to the repository.
                    // Context: Repository ID.
                    info!(repo_id = ?repo_id, "Emitting Diff message to the repository.");
                    Box::pin(async move {
                        repo.emit_async(Diff).await
                    })
                } else {
                    // Event: Repository Not Found
                    // Description: No repository found matching the given ID.
                    // Context: Repository ID.
                    warn!(repo_id = ?repo_id, "No repository found matching the given ID.");
                    Box::pin(async move {})
                }
            });

        for repo in &tangler_config.repositories {
            let broker = actor.state.broker.clone();

            // Event: Repository Actor Initialization
            // Description: Initializing a repository actor.
            // Context: Repository configuration details.
            info!(repo = ?repo, "Initializing a repository actor.");
            if let Some(repo_actor) = RepositoryActor::init(repo, broker.clone()).await {
                actor.state.git_repositories.push(repo_actor);
            }
            info!(repo = ?repo, "Initializing a diff watcher actor.");
            actor.state.diff_watchers.push(RepositoryWatcherActor::init(repo, broker).await?);
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

        let subscription = BrokerSubscribe {
            message_type_id: TypeId::of::<NotifyChange>(),
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

        // Event: Sending Message to Broker
        // Description: Sending the constructed message to the broker.
        // Context: Broker emit message details.
        info!("Sending the constructed message to the broker.");
        broker.emit_async(error_msg).await;

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