use akton::prelude::*;

#[akton_message]
pub(crate) struct AcceptParentBroker {
    pub(crate) broker: Context
}