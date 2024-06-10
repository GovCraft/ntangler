use std::path::PathBuf;

use akton::prelude::*;

// TODO: will revisit names shortly
#[akton_message]
pub(crate) struct SubmitDiff {
    pub(crate) diff: String,
    pub(crate) path: String,
    pub(crate) id: String,
}