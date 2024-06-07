use akton::prelude::*;

#[akton_message]
pub(crate) struct SubmitDiff {
    pub(crate) diff: String,
    pub(crate) id: String,
}