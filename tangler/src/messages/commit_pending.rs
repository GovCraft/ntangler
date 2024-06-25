use akton::prelude::*;
use derive_more::*;

use crate::models::{Commit, Oid, PendingCommit, TimeStamp};

/// Represents a successful commit message with its details.
#[akton_message]
pub(crate) struct CommitPending {
    commit: PendingCommit
}

impl CommitPending {
    pub(crate) fn commit(&self) -> &PendingCommit {
        &self.commit
    }
    pub(crate) fn new(mut commit: PendingCommit, hash: String) -> CommitPending {
        CommitPending { commit }
    }
}

