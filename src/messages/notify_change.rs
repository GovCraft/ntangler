use akton::prelude::*;

#[akton_message]
pub(crate) struct NotifyChange{
    pub(crate) repo_id: String
}
