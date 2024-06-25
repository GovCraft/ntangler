use std::fmt;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};
use derive_more::*;
use crate::models::{CommittedCommit, Description, DiffGeneratedCommit, Filename, FilenameTerminal, Footer, generate_id, Oid, PendingCommit, RepositoryTerminal, Scope, Status, TimeStamp, TimeStampTerminal};
use crate::models::semver_impact::SemVerImpact;
//use crate::models::traits::TanglerModel;
use std::default::Default;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CommitMessageGeneratedCommit {
    pub(crate) commit: CommittedCommit,
    pub(crate) status: Status,
}

impl CommitMessageGeneratedCommit {
    pub(crate) fn new(commit: CommittedCommit, message: String) -> Self {
        CommitMessageGeneratedCommit{
            commit,
            status: Status::Committing,
        }
    }
}
impl fmt::Display for CommitMessageGeneratedCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repository = &self.commit.repository.clone();
        let filename = &self.commit.filename.clone();
        let status = &self.status.clone();

        // convert to terminal formatted versions
        let filename: FilenameTerminal = filename.into();
        let repository: RepositoryTerminal = repository.into();
        let display = format!("{repository} {status} {filename}");

        write!(f, "{}", display)
    }
}

// impl From<PendingCommit> for CommitMessageGeneratedCommit {
//     fn from(value: PendingCommit) -> Self {
//         CommitMessageGeneratedCommit {
//             id: value.id,
//             repository: value.repository,
//             filename: value.filename,
//             status: Status::Committing,
//             ..Default::default()
//         }
//     }
// }


