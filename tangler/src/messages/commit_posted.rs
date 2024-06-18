use akton::prelude::*;

use crate::models::{Commit, Oid, TimeStamp};

/// Represents a successful commit message with its details.
#[akton_message]
pub(crate) struct CommitPosted {
    pub(crate) commit: Commit,
}

impl CommitPosted {
    pub(crate) fn commit(&self) -> &Commit {
        &self.commit
    }
    pub(crate) fn new(mut commit: Commit, hash: String) -> CommitPosted {
        commit.timestamp = TimeStamp::new();
        commit.oid = Oid::new(&hash);
        CommitPosted { commit }
    }
}

