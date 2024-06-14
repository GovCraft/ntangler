use serde::Deserialize;

use crate::models::config::RepositoryConfig;

#[derive(Deserialize, Debug)]
pub(crate) struct TanglerConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}
