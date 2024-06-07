use akton::prelude::*;

use crate::repository_config::RepositoryConfig;

#[akton_message]
pub(crate) struct Watch {
    pub(crate) repo: RepositoryConfig,
}