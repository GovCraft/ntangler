use serde::Deserialize;

use crate::repository_config::RepositoryConfig;

#[derive(Deserialize, Debug)]
pub(crate) struct TanglerConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}
