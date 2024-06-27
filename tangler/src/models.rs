use std::hash::{DefaultHasher, Hash, Hasher};
// pub(crate) use pending_commit::PendingCommit;
pub(crate) use commit_type::CommitType;
pub(crate) use committed_commit::CommittedCommit;
pub(crate) use description::Description;
pub(crate) use file_name::Filename;
pub(crate) use footer::Footer;
pub(crate) use oid::Oid;
pub(crate) use scope::{OptionalScope, Scope};
pub(crate) use semver_impact::SemVerImpact;
pub(crate) use time_stamp::TimeStamp;
pub(crate) use ui::*;

mod commit_type;
mod committed_commit;
pub(crate) mod config;
mod description;
mod file_name;
mod footer;
mod oid;
mod scope;
mod semver_impact;
mod time_stamp;
mod traits;
mod ui;
// mod pending_commit;
// mod diff_generated_commit;
mod tangled_commit;
// mod commit_step;
mod commit_message;
mod signature;
mod tangled_repository;

pub(crate) use commit_message::CommitMessage;
pub(crate) use signature::TangledSignature;
pub(crate) use tangled_repository::TangledRepository;
// pub(crate) use commit_step::CommitStep;
pub(crate) use tangled_commit::TangledCommit;
// pub(crate) use diff_generated_commit::DiffGeneratedCommit;

// mod commit_step;
//
// pub(crate) use commit_step::CommitStep;

/// Generates a unique ID based on the hash of the repository and filename combined.
pub(crate) fn generate_id(repository: &str, filename: Filename) -> String {
    let mut hasher = DefaultHasher::new();
    repository.hash(&mut hasher);
    filename.hash(&mut hasher);
    hasher.finish().to_string()
}
