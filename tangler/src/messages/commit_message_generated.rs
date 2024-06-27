use std::path::PathBuf;

use derive_new::new;

use crate::models::CommitMessage;

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone)]
pub(crate) struct CommitMessageGenerated {
    pub(crate) target_file: PathBuf,
    pub(crate) commit_message: CommitMessage,
}
