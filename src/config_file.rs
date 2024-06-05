use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::repository_config::RepositoryConfig;

#[derive(Serialize, Deserialize)]
pub(crate) struct ConfigFile {
    pub(crate) repositories: HashMap<String, RepositoryConfig>,
}
