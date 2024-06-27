use std::path::{Path, PathBuf};
use akton::prelude::Arn;
use git2::Signature;
use crate::models::config::RepositoryConfig;
use crate::models::Oid;
use derive_more;
use serde::Deserialize;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TangledRepository {
    pub(crate) akton_arn: Arn<'static>,
    pub(crate) nickname: String,
    pub(crate) path: PathBuf,
    pub(crate) branch_name: String,
}

impl From<RepositoryConfig> for TangledRepository{
    fn from(value: RepositoryConfig) -> Self {
        TangledRepository{
            akton_arn: Arn::with_root("tangled_repository").unwrap(),
            nickname: value.nickname,
            path: value.path,
            branch_name: value.branch_name,
        }
    }
}
