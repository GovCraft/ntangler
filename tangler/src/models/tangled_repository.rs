use crate::models::config::RepositoryConfig;
use crate::models::Oid;
use akton::prelude::Arn;
use derive_more;
use git2::Signature;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TangledRepository {
    pub(crate) akton_arn: Arn<'static>,
    pub(crate) nickname: String,
    pub(crate) path: PathBuf,
    pub(crate) branch_name: String,
}

impl From<RepositoryConfig> for TangledRepository {
    fn from(value: RepositoryConfig) -> Self {
        TangledRepository {
            akton_arn: Arn::with_root("tangled_repository").unwrap(),
            nickname: value.nickname,
            path: value.path,
            branch_name: value.branch_name,
        }
    }
}
