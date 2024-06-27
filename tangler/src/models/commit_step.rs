use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use crate::models::STATUS;

use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) enum Status {
    #[default]
    Pending,
    Thinking,
    Committing,
}
pub(crate) enum CommitStep {
    FileChangeDetected(PathBuf),
    DiffQueued(PathBuf),
    DiffGenerated(String),
    CommitMessageGenerated(String),
    #[default]
    Finalized,
}

impl fmt::Display for CommitStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            CommitStep::FileChangeDetected(_) => "DETECTED",
            CommitStep::DiffQueued(_) => "QUEUED",
            CommitStep::DiffGenerated(_) => "DIFFSUBMITTED",
            CommitStep::CommitMessageGenerated(_) => "GENERATED",
            CommitStep::Finalized => "FINALIZED",
        };
        write!(f, "{}", symbol)
    }
}
