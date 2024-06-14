use akton::prelude::*;

#[akton_message]
pub(crate) struct AcceptBroker {
    pub(crate) broker: Context
}