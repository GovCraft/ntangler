use std::fmt;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};

use crate::models::{Description, Filename, FilenameTerminal, Footer, generate_id, Oid, RepositoryTerminal, Scope, Status, TimeStamp, TimeStampTerminal};
use crate::models::semver_impact::SemVerImpact;
//use crate::models::traits::TanglerModel;



#[derive(Debug, Clone, Default)]
pub(crate) struct PendingCommit {
    pub(crate) id: String,
    pub(crate) repository: String,
    pub(crate) filename: Filename,
    pub(crate) status: Status,
}
impl fmt::Display for PendingCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repository = &self.repository.clone();
        let filename = &self.filename.clone();
        let status = &self.status.clone();

        // convert to terminal formatted versions
        let filename: FilenameTerminal = filename.into();
        let repository: RepositoryTerminal = repository.into();
        let display = format!("{repository} {status} {filename}");

        write!(f, "{}", display)
    }
}

impl PendingCommit {
    pub(crate) fn new(repository: String, filename: Filename) -> PendingCommit {
        let id = generate_id(&repository, filename.clone());
        PendingCommit {
            id,
            repository,
            filename,
            ..Default::default()
        }
    }
}


