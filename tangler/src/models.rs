use std::hash::{DefaultHasher, Hash, Hasher};
pub(crate) use pending_commit::PendingCommit;
pub(crate) use commit::Commit;
pub(crate) use commit_type::CommitType;
pub(crate) use description::Description;
pub(crate) use footer::Footer;
pub(crate) use oid::Oid;
pub(crate) use scope::{OptionalScope, Scope};
pub(crate) use time_stamp::TimeStamp;
pub(crate) use ui::*;
pub(crate) use file_name::Filename;
mod footer;
mod commit_type;
mod scope;
mod commit;
pub(crate) mod config;
mod time_stamp;
mod oid;
mod description;
mod semver_impact;
mod ui;
mod traits;
mod file_name;
mod pending_commit;

/// Generates a unique ID based on the hash of the repository and filename combined.
pub(crate) fn generate_id(repository: &str, filename: Filename) -> String {
    let mut hasher = DefaultHasher::new();
    repository.hash(&mut hasher);
    filename.hash(&mut hasher);
    hasher.finish().to_string()
}


