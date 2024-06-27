use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use tracing::{instrument, trace};

use derive_more::DebugCustom;
/// Represents a repository configuration.
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct RepositoryConfig {
    pub(crate) nickname: String,
    pub(crate) path: PathBuf,
    pub(crate) branch_name: String,
}
