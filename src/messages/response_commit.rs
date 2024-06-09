use akton::prelude::*;
use crate::commits::Commits;

#[akton_message]
pub(crate) struct ResponseCommit {
    pub(crate) commits: Commits
}



