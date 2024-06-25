use derive_more::*;
use derive_new::new;
use crate::models::Status;
use std::default::Default;
use serde::Deserialize;

#[derive(new, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) struct DiffGeneratedCommit {
    pub(crate) id: String,
    pub(crate) diff: String,
    pub(crate) filename: String,
    pub(crate) repository: String,
    pub(crate) status: Status,
}
