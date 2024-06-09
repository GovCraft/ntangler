use std::path::PathBuf;
use akton::prelude::*;

#[akton_message]
pub(crate) struct Diff {
    pub(crate) path: PathBuf,
}