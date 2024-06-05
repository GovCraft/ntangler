use serde::Deserialize;
use crate::repository_config::RepositoryConfig;

#[derive(Deserialize)]
pub(crate) struct GinjaConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}