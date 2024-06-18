use std::fmt::{Display, Formatter};
use akton::prelude::*;

use crate::models::{Commit, Oid, PendingCommit, Status, TimeStamp};

/// Represents a successful commit message with its details.
#[akton_message]
pub(crate) struct CommitAuthoring {
    pub(crate) commit: PendingCommit,
}

impl From<PendingCommit> for CommitAuthoring {
    fn from(value: PendingCommit) -> Self {
        let mut updated = value.clone();
        updated.status = Status::Generating;
        CommitAuthoring { commit: updated }
    }
}

impl Display for CommitAuthoring {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.commit)
    }
}


