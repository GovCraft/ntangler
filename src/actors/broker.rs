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

use crate::messages::{BrokerSubscribe, BrokerUnsubscribe, ErrorNotification, NotifyChange, ResponseCommit, SubmitDiff};

#[akton_actor]
pub(crate) struct Broker {
    subscribers: DashMap<TypeId, HashSet<Context>>,
}

impl Broker {
    /// Initializes the BrokerActor.
    ///
    /// # Returns
    /// - `anyhow::Result<Context>`: The context of the initialized actor.
    #[instrument]
    pub(crate) async fn init() -> anyhow::Result<Context> {
        let mut actor = Akton::<Broker>::create_with_id("tangler_broker");

        actor.setup
            .act_on::<BrokerSubscribe>(|actor, event| {
                // Event: Broker Subscribe
                // Description: Triggered when a new subscriber is added.
                // Context: Subscriber's return address, message type ID, and subscriber context.
                let outbound_envelope = event.return_address.clone();
                let type_id = event.message.message_type_id;
                let subscriber = event.message.subscriber_context.clone();

                actor.state.subscribers
                    .entry(type_id)
                    .or_insert_with(HashSet::new)
                    .insert(subscriber.clone());

                // Event: Subscriber Added
                // Description: A new subscriber has been added to the broker.
                // Context: Type ID and subscriber context.
                info!(type_id=?type_id, subscriber=subscriber.key.value, "Subscriber added");
            })
            .act_on::<BrokerUnsubscribe>(|actor, event| {
                // Event: Broker Unsubscribe
                // Description: Triggered when a subscriber is removed.
                // Context: Subscriber's message type and subscriber context.
                let current_subscriber = event.message.subscriber.clone();
                let type_id = event.message.message_type;

                if let Some(mut subscribers) = actor.state.subscribers.get_mut(&type_id) {
                    subscribers.remove(&current_subscriber);
                    if subscribers.is_empty() {
                        actor.state.subscribers.remove(&type_id);
                    }
                }

                // Event: Subscriber Removed
                // Description: A subscriber has been removed from the broker.
                // Context: Type ID and current subscriber context.
                info!(type_id=?type_id, current_subscriber=current_subscriber.key.value, "Subscriber removed");
            })
            .act_on_async::<ErrorNotification>(|actor, event| {
                // Event: Error Notification
                // Description: Handling an error notification asynchronously.
                // Context: Error message details.
                info!("Handling an error notification asynchronously.");
                actor.state.load_subscriber_futures(event.message.clone())
            })
            .act_on_async::<SubmitDiff>(|actor, event| {
                debug!("Handling a submitted diff asynchronously.");
                actor.state.load_subscriber_futures(event.message.clone())
            })
            .act_on_async::<ResponseCommit>(|actor, event| {
                debug!("Handling a ResponseCommit message asynchronously.");
                actor.state.load_subscriber_futures(event.message.clone())
            })
            .act_on_async::<NotifyChange>(|actor, event| {
                // Event: Notify Change
                // Description: Handling a notify change event asynchronously.
                // Context: Change message details.
                info!("Handling a notify change event asynchronously.");
                actor.state.load_subscriber_futures(event.message.clone())
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
    fn load_subscriber_futures(
        &self,
        message: impl AktonMessage + Send + Sync + Clone,
    ) -> Pin<Box<impl Future<Output=()> + Sized>> {
        let mut futures = FuturesUnordered::new();
        let type_id = std::any::Any::type_id(&message).clone();

        if let Some(subscribers) = self.subscribers.get(&type_id) {
            for subscriber_context in subscribers.value().clone() {
                let type_id = type_id.clone();
                let subscriber_context = subscriber_context.clone();
                let message = message.clone();

                // Event: Subscriber Found
                // Description: A subscriber has been found for the message type.
                // Context: Subscriber context and message type ID.
                debug!(subscriber=&subscriber_context.key.value, message_type=?&type_id, "Subscriber found");

                futures.push(async move {
                    subscriber_context.emit_async(message, None).await;
                });
            }
        }

        Box::pin(async move {
            while let Some(_) = futures.next().await {
                // Event: Message Emitted
                // Description: The message has been emitted to a subscriber.
                // Context: Message type ID.
                info!(message_type=?type_id, "Message emitted");
            }
        })
    }
}