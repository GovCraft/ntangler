use crate::models::{CommitMessage, CommittedCommit, Oid, TimeStamp};
use akton::prelude::*;
use derive_more::*;
use derive_new::new;
use std::path::PathBuf;

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone)]
pub(crate) struct FinalizedCommit {
    pub(crate) when: TimeStamp,
    pub(crate) target_file: PathBuf,
    pub(crate) repository_nickname: String,
    pub(crate) hash: String,
    pub(crate) commit_message: CommitMessage,
}
