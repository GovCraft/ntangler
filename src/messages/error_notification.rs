use akton::prelude::*;

#[akton_message]
pub(crate) struct ErrorNotification  {
    pub(crate) error_message: String
}