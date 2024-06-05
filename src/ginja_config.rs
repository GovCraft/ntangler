use crate::repository_config::RepositoryConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct GinjaConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}
