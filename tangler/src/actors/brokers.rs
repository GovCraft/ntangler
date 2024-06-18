use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;

use akton::prelude::*;
use dashmap::DashMap;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tracing::{debug, debug_span, field, info, instrument, trace, trace_span};

use crate::messages::{CommitEvent, CommitMessageGenerated, CommitPending, CommitPosted, DiffCalculated, NotifyChange, NotifyError, PollChanges, SubscribeBroker, SystemStarted, UnsubscribeBroker};

// TODO: This needs to be generalized and included in the Akton framework
#[akton_actor]
pub(crate) struct Broker {
    subscribers: DashMap<TypeId, HashSet<(String, Context)>>,
}

impl Broker {
    /// Initializes the BrokerActor.
    ///
    /// # Returns
    /// - `anyhow::Result<Context>`: The context of the initialized actor.
    #[instrument]
    pub(crate) async fn init() -> anyhow::Result<Context> {
        let actor_config = ActorConfig::new("broker", None, None);
        let mut actor = Akton::<Broker>::create_with_config(actor_config);

        actor.setup
            .act_on::<SubscribeBroker>(|actor, event| {
                // Event: Broker Subscribe
                // Description: Triggered when a new subscriber is added.
                // Context: Subscriber's return address, message type ID, and subscriber context.
                let outbound_envelope = event.return_address.clone();
                let type_id = event.message.message_type_id;
                let subscriber = event.message.subscriber_context.clone();
                let id = event.message.subscriber_id.clone();

                actor.state.subscribers
                    .entry(type_id)
                    .or_insert_with(HashSet::new)
                    .insert((id.clone(), subscriber.clone()));

                // Event: Subscriber Added
                // Description: A new subscriber has been added to the broker.
                // Context: Type ID and subscriber context.
                trace!(type_id=?type_id, subscriber=subscriber.key.value, "Subscriber added");
            })
            .act_on::<UnsubscribeBroker>(|actor, event| {
                // Event: Broker Unsubscribe
                // Description: Triggered when a subscriber is removed.
                // Context: Subscriber's message type and subscriber context.
                let current_subscriber = event.message.subscriber.clone();
                let type_id = event.message.message_type;
                let id = event.message.subscriber_id.clone();

                if let Some(mut subscribers) = actor.state.subscribers.get_mut(&type_id) {
                    subscribers.remove(&(id, current_subscriber.clone()));
                    if subscribers.is_empty() {
                        actor.state.subscribers.remove(&type_id);
                    }
                }
                // Event: Subscriber Removed
                // Description: A subscriber has been removed from the broker.
                // Context: Type ID and current subscriber context.
                info!(type_id=?type_id, current_subscriber=current_subscriber.key.value, "Subscriber removed");
            })
            .act_on_async::<NotifyError>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<NotifyError>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<DiffCalculated>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<DiffCalculated>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<CommitEvent>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<CommitEvent>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<CommitPosted>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<CommitPosted>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<PollChanges>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<PollChanges>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<SystemStarted>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<SystemStarted>(event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<CommitMessageGenerated>(|actor, event| {
                let futures = actor.state.load_subscriber_future_by_id::<CommitMessageGenerated>(event.message.id.clone(), event.message.clone());
                Self::broadcast_futures(futures)
            })
            .act_on_async::<NotifyChange>(|actor, event| {
                let futures = actor.state.load_subscriber_futures::<NotifyChange>(event.message.clone());
                Self::broadcast_futures(futures)
            });

        // Event: Activating BrokerActor
        // Description: Activating the BrokerActor.
        // Context: None
        info!("Activating the BrokerActor.");
        Ok(actor.activate(None).await?)
    }

    /// Loads subscriber futures for the given message and emits the message to each subscriber.
    ///
    /// # Parameters
    /// - `message`: The message to be emitted to the subscribers.
    ///
    /// # Returns
    /// - `Pin<Box<impl Future<Output=()> + Sized>>`: A future that resolves when all subscriber futures have completed.
    fn load_subscriber_futures<T>(
        &self,
        message: impl AktonMessage + Send + Sync + Clone,
    ) -> FuturesUnordered<impl Future<Output = ()> + Sized> {
        let mut futures = FuturesUnordered::new();
        let type_id = std::any::Any::type_id(&message).clone();

        if let Some(subscribers) = self.subscribers.get(&type_id) {
            for (_, subscriber_context) in subscribers.value().clone() {
                let type_id = type_id.clone();
                let subscriber_context = subscriber_context.clone();
                let message = message.clone();

                // Event: Subscriber Found
                // Description: A subscriber has been found for the message type.
                // Context: Subscriber context and message type ID.
                trace!(
                    subscriber = &subscriber_context.key.value,
                    message_type = ?&type_id,
                    "Subscriber found"
                );

                futures.push(async move {
                    subscriber_context.emit_async(message, None).await;
                });
            }
        }

        // Event: Futures Loaded
        // Description: All subscriber futures have been loaded.
        // Context: Number of futures.
        trace!(
            futures_count = futures.len(),
            "All subscriber futures have been loaded."
        );

        futures
    }

    // TODO: need comments now
    #[instrument]
    fn load_subscriber_future_by_id<T>(
        &self,
        id: String,
        message: impl AktonMessage + Send + Sync + Clone + 'static,
    ) -> FuturesUnordered<impl Future<Output = ()> + 'static> {
        let type_id = std::any::Any::type_id(&message);
        let mut futures = FuturesUnordered::new();

        if let Some(subscribers) = self.subscribers.get(&type_id) {
            // Clone the subscribers to avoid lifetime issues
            let subscribers_cloned: Vec<_> = subscribers
                .value()
                .iter()
                .filter(|(subscriber_id, _)| **subscriber_id == id)
                .cloned()
                .collect();

            for (_, subscriber_context) in subscribers_cloned {
                let id = id.clone();
                let message = message.clone();

                // Event: Subscriber Found by ID
                // Description: A subscriber has been found for the message type by ID.
                // Context: Subscriber ID and message type ID.
                trace!(subscriber_id = id, message_type = ?type_id, "Subscriber found by ID.");

                futures.push(async move {
                    subscriber_context.clone().emit_async(message, None).await;
                });
            }
        }

        // Event: Futures Loaded by ID
        // Description: All subscriber futures have been loaded by ID.
        // Context: Number of futures.
        trace!(
            futures_count = futures.len(),
            "All subscriber futures have been loaded by ID."
        );

        debug_assert_ne!(
            futures.len(),
            0,
            "There were no subscribers found for id: {}",
            id.clone()
        );
        futures
    }

    fn broadcast_futures<T>(
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
            while futures.next().await.is_some() {}
            // Event: Futures Broadcast Completed
            // Description: All futures have been processed.
            // Context: None
            trace!("All futures have been processed.");
        })
    }
}

