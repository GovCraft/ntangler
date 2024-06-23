use std::fmt;
use akton::prelude::*;
use crate::messages::commit_authoring::CommitAuthoring;
use crate::messages::CommitPosted;
use crate::models::{
    Commit, CommitHeadingTerminal, CommitTypeTerminal, DescriptionTerminal, FilenameTerminal,
    generate_id, GeneratingCommit, IsBreakingTerminal, Oid, OidTerminal, PendingCommit,
    RepositoryTerminal, ScopeTerminal, SemVerImpactTerminal, TAB_WIDTH, TimeStamp,
    TimeStampTerminal,
};

/// Represents a successful commit message with its details.
#[derive(Clone, Debug)]
pub(crate) enum CommitEventCategory {
    Pending(PendingCommit),
    Generating(GeneratingCommit),
    Commit(Commit),
}

#[akton_message]
pub(crate) struct CommitEvent {
    pub(crate) id: String,
    pub(crate) timestamp: TimeStamp,
    pub(crate) category: CommitEventCategory,
}

impl fmt::Display for CommitEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timestamp: TimeStampTerminal = (&self.timestamp).into();
        let tab = " ".repeat(TAB_WIDTH);
        let half_tab = " ".repeat(TAB_WIDTH / 2);

        let display = match &self.category {
            CommitEventCategory::Pending(event) => {
                let filename: FilenameTerminal = (&event.filename).into();
                let repository: RepositoryTerminal = (&event.repository).into();
                let status = &event.status;
                format!("{repository} {timestamp} {status} {filename}")
            }
            CommitEventCategory::Generating(event) => {
                let filename: FilenameTerminal = (&event.filename).into();
                let repository: RepositoryTerminal = (&event.repository).into();
                let status = &event.status;
                format!("{repository} {timestamp} {status} {filename}")
            }
            CommitEventCategory::Commit(commit) => {
                let oid: OidTerminal = (&commit.oid).into();
                let description: DescriptionTerminal = (&commit.description).into();
                let semver_impact: SemVerImpactTerminal = (&commit.semver_impact).into();
                let commit_heading: CommitHeadingTerminal = (
                    (&commit.commit_type).into(),
                    (&commit.scope).into(),
                    (&commit.is_breaking).into()
                ).into();
                let repository: RepositoryTerminal = (&commit.repository).into();
                let file_name: FilenameTerminal = (&commit.filename).into();
                format!(
                    "{repository} {timestamp} {oid} {semver_impact} {file_name} {commit_heading} {description}"
                )
            }
        };

        write!(f, "{}", display)
    }
}

impl CommitEvent {
    pub(crate) fn new(category: CommitEventCategory) -> CommitEvent {
        let (filename, repository) = match &category {
            CommitEventCategory::Pending(c) => {
                (&c.filename, &c.repository)
            }
            CommitEventCategory::Generating(c) => {
                (&c.filename, &c.repository)
            }
            CommitEventCategory::Commit(c) => {
                (&c.filename, &c.repository)
            }
        };
        let timestamp = TimeStamp::new();
        let id = generate_id(repository, filename.clone());
        CommitEvent {
            id,
            timestamp,
            category,
        }
    }
}