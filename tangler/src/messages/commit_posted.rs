use akton::prelude::*;

use crate::models::{CommittedCommit, Oid, TimeStamp};

/// Represents a successful commit message with its details.
#[akton_message]
pub(crate) struct CommitPosted {
    pub(crate) commit: CommittedCommit,
}

impl CommitPosted {
    pub(crate) fn commit(&self) -> &CommittedCommit {
        &self.commit
    }
    pub(crate) fn new(mut commit: CommittedCommit, hash: String) -> CommitPosted {
        commit.timestamp = Default::default();
        commit.oid = Oid::new(&hash);
        CommitPosted { commit }
    }
}
