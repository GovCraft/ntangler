use akton::prelude::*;

use crate::models::{Commit, Oid, TimeStamp};

/// Represents a successful commit message with its details.
#[akton_message]
pub(crate) struct CommitSuccess {
    commit: Commit,
}

impl CommitSuccess {
    pub(crate) fn commit(&self) -> &Commit {
        &self.commit
    }
    pub(crate) fn new(mut commit: Commit, hash: String) -> CommitSuccess {
        commit.time_stamp = TimeStamp::new();
        commit.oid = Oid::new(&hash);
        CommitSuccess { commit }
    }
}

