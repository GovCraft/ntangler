use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use futures::future::join_all;
use akton::prelude::*;
use dashmap::DashMap;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tracing::{debug, instrument, trace, field, trace_span, debug_span, info};

use crate::messages::{BrokerEmit, BrokerSubscribe, BrokerUnsubscribe, ErrorNotification};
// debug!("BROKER SUBSCRIPTION: {} for AktonMessage TypeId {:?}", &event.return_address.sender.value.clone(), &type_id);
// debug!("subscriber_count: {}", &actor.state.subscribers.len());
// debug!("BROKER EMIT for TypeId {:?}", &type_id);
// debug!("subscriber_count: {}", &actor.state.subscribers.len());
// debug!("NOTIFY SUBSCRIBER {} for TypeId {:?}", &subscriber_context.key.value, &type_id);

#[akton_actor]
pub(crate) struct BrokerActor {
    subscribers: DashMap<TypeId, HashSet<Context>>,
}

impl BrokerActor {
    #[instrument]
    pub(crate) async fn init() -> anyhow::Result<Context> {
        let mut actor = Akton::<BrokerActor>::create_with_id("tangler_broker");

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
                debug!(type_id=?type_id, subscriber=subscriber.key.value, "Subscriber added");
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
                info!(type_id=?type_id, current_subscriber=current_subscriber.key.value, "Subscriber removed");
            })
            .act_on_async::<BrokerEmit>(|actor, event| {
                // Event: Broker Emit
                // Description: Triggered when a message is emitted to subscribers.
                // Context: Message type ID and the message being emitted.
                let (type_id, message) = match event.message {
                    BrokerEmit::Error(msg) => {
                        let message = msg.clone();
                        (TypeId::of::<ErrorNotification>(), message)
                    }
                };
                let subscribers = actor.state.subscribers.clone();
                let mut futures = FuturesUnordered::new();

                if let Some(subscribers) = subscribers.get(&type_id) {
                    for subscriber_context in subscribers.value().clone() {
                        let type_id = type_id.clone();
                        let subscriber_context = subscriber_context.clone();
                        let message = message.clone();
                        debug!(subscriber=&subscriber_context.key.value, message_type=?&type_id,"Subscriber found");
                        futures.push(async move {
                            subscriber_context.emit_async(message.clone()).await;
                        });
                    }
                }

                Box::pin(async move {
                    while let Some(_) = futures.next().await {
                        // Each future is awaited here
                        debug!(message_type=?type_id, "Message emitted");
                    }
                })
            });

        Ok(actor.activate(None).await?)
    }
}