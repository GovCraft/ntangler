use akton::prelude::*;

use crate::models::CommittedCommit;

#[akton_message]
pub(crate) struct CommitMessageGenerated {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) commit: CommittedCommit,
}
