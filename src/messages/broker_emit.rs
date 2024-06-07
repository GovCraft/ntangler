use std::any::TypeId;
use akton::prelude::*;
use crate::messages::ErrorNotification;

#[akton_message]
pub(crate) enum BrokerEmit {
    Error(ErrorNotification),
}