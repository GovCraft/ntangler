use std::path::PathBuf;

use derive_new::new;

use crate::models::{CommitMessage, TimeStamp};

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone)]
pub(crate) struct FinalizedCommit {
    pub(crate) when: TimeStamp,
    pub(crate) target_file: PathBuf,
    pub(crate) repository_nickname: String,
    pub(crate) hash: String,
    pub(crate) commit_message: CommitMessage,
}
