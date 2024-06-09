use std::path::PathBuf;
use akton::prelude::*;

#[akton_message]
pub(crate) struct NotifyChange{
    pub(crate) repo_id: String,
    pub(crate) path: PathBuf
}
