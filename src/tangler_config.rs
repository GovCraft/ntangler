use crate::repository_config::RepositoryConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct TanglerConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}
