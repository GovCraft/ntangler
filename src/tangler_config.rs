use serde::Deserialize;

use crate::repository_config::RepositoryConfig;

// TODO: Revisit and make sure we need this
#[derive(Deserialize, Debug)]
pub(crate) struct TanglerConfig {
    pub(crate) repositories: Vec<RepositoryConfig>,
}
