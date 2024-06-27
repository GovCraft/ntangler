use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use tracing::{instrument, trace};

/// Represents a repository configuration.
use std::path::Path;
use derive_more::DebugCustom;
use std::path::PathBuf;


#[derive(Serialize,Deserialize, Debug, Default, Clone, PartialEq)]
pub(crate) struct RepositoryConfig {
    pub(crate) nickname: String,
    pub(crate) path: PathBuf,
    pub(crate) branch_name: String,
}