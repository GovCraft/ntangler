use akton::prelude::*;
use std::any::TypeId;
use std::marker::PhantomData;

#[akton_message]
pub(crate) struct SubscribeBroker {
    pub(crate) subscriber_id: String,
    pub(crate) message_type_id: TypeId,
    pub(crate) subscriber_context: Context,
}
