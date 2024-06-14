use std::any::TypeId;

use akton::prelude::*;

#[akton_message]
pub(crate) struct UnsubscribeBroker {
    pub(crate) subscriber_id: String,
    pub(crate) message_type: TypeId,
    pub(crate) subscriber: Context,
}