use std::path::PathBuf;
use akton::prelude::*;
use derive_more::*;
use derive_new::new;
use crate::models::{CommittedCommit, Oid, TimeStamp};

/// Represents a successful commit message with its details.
#[derive(new, Default, Debug, Clone, )]
pub(crate) struct DiffQueued {
    pub(crate) diff: String,
    pub(crate) target_file: PathBuf,
    pub(crate) repository_nickname: String,
    pub(crate) reply_address: Context,
}



