use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Represents a repository configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct RepositoryConfig {
    pub(crate) nickname: String,
    pub(crate) path: PathBuf,
    pub(crate) branch_name: String,
}
