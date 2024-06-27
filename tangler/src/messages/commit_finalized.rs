use std::path::PathBuf;
use akton::prelude::*;
use derive_more::*;
use derive_new::new;
use git2::Oid;
use crate::models::{CommitMessage, CommittedCommit, TimeStamp};

/// Represents a successful commit message with its details.
#[derive(new, Debug, Clone, )]
pub(crate) struct CommitFinalized {
    pub(crate) path: PathBuf,
    pub(crate) commit_message: CommitMessage,
    pub(crate) oid: Oid,
}



