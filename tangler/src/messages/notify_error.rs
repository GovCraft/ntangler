use akton::prelude::*;

#[akton_message]
pub(crate) struct NotifyError {
    pub(crate) error_message: String
}