use std::any::TypeId;
use std::marker::PhantomData;
use akton::prelude::*;

#[akton_message]
pub(crate) struct BrokerSubscribe {
    pub(crate) message_type_id: TypeId,
    pub(crate) subscriber_context: Context
}