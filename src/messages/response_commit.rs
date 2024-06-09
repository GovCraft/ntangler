use std::path::PathBuf;

use akton::prelude::*;

use crate::commits::Commits;

#[akton_message]
pub(crate) struct ResponseCommit {
    pub(crate) path: PathBuf,
    pub(crate) commits: Commits,
}



