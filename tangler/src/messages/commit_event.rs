use std::fmt;
use akton::prelude::*;
use crate::messages::commit_authoring::CommitAuthoring;
use crate::messages::CommitPosted;

use crate::models::{Commit, CommitHeadingTerminal, CommitTypeTerminal, DescriptionTerminal, FilenameTerminal, generate_id, GeneratingCommit, IsBreakingTerminal, Oid, OidTerminal, PendingCommit, RepositoryTerminal, ScopeTerminal, SemVerImpactTerminal, TAB_WIDTH, TimeStamp, TimeStampTerminal};

/// Represents a successful commit message with its details.
#[derive(Clone, Debug)]
pub(crate) enum Category {
    Pending(PendingCommit),
    Generating(GeneratingCommit),
    Commit(Commit),
}

#[akton_message]
pub(crate) struct CommitEvent {
    pub(crate) id: String,
    pub(crate) timestamp: TimeStamp,
    pub(crate) category: Category,
}

impl fmt::Display for CommitEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut display = String::default();
        let timestamp = &self.timestamp;
        let timestamp: TimeStampTerminal = timestamp.into();
        let tab = " ".repeat(TAB_WIDTH);
        let half_tab = " ".repeat(TAB_WIDTH / 2);

        match &self.category {
            Category::Pending(event) => {
                let repository = &event.repository.clone();
                let filename = &event.filename.clone();
                let status = &event.status.clone();

                // convert to terminal formatted versions
                let filename: FilenameTerminal = filename.into();
                let repository: RepositoryTerminal = repository.into();
                display = format!("{repository} {timestamp} {status} {filename}");
            }
            Category::Generating(event) => {
                let repository = &event.repository.clone();
                let filename = &event.filename.clone();
                let status = &event.status.clone();

                // convert to terminal formatted versions
                let filename: FilenameTerminal = filename.into();
                let repository: RepositoryTerminal = repository.into();
                display = format!("{repository} {timestamp} {status} {filename}");
            }
            Category::Commit(commit) => {
                let repository = &commit.repository;
                let oid = &commit.oid;
                let description = &commit.description;
                let scope = &commit.scope;
                let commit_type = &commit.commit_type;
                let semver_impact = &commit.semver_impact;
                let is_breaking = &commit.is_breaking;
                let file_name = &commit.filename;

                // convert to terminal formatted versions
                let oid: OidTerminal = oid.into();
                let description: DescriptionTerminal = description.into();
                let semver_impact: SemVerImpactTerminal = semver_impact.into();
                let commit_heading: (CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal) = (commit_type.into(), scope.into(), is_breaking.into());
                let commit_heading: CommitHeadingTerminal = commit_heading.into();
                let repository: RepositoryTerminal = repository.into();
                let file_name: FilenameTerminal = file_name.into();

                display = format!("{repository} {timestamp} {oid} {semver_impact} {file_name} {commit_heading} {description}");
            }
        }
        write!(f, "{}", display)
    }
}

impl CommitEvent {
    pub(crate) fn new(category: Category) -> CommitEvent {
        let (filename, repository) = match &category {
            Category::Pending(c) => {
                (&c.filename, &c.repository)
            }
            Category::Generating(c) => {
                (&c.filename, &c.repository)
            }
            Category::Commit(c) => {
                (&c.filename, &c.repository)
            }
        };
        let timestamp = TimeStamp::new();
        let id = generate_id(&repository, filename.clone());
        CommitEvent {
            id,
            timestamp,
            category,
        }
    }
}