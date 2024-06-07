use akton::prelude::*;

#[akton_message]
pub(crate) struct ResponseCommit {
    pub(crate) commit: String
}

