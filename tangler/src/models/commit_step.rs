use std::fmt;
use std::time::Duration;
use owo_colors::OwoColorize;
use crate::models::STATUS;

use indicatif::ProgressBar;
use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) enum CommitStep {
    #[default]
    Queued,
    GeneratingMessage,
    Finalizing,
}

impl fmt::Display for CommitStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            CommitStep::Queued => "QUEUED",
            CommitStep::GeneratingMessage => "GENERATING",
            CommitStep::Finalizing => "FINALIZING",
        };
        write!(f, "{:8}", symbol)
    }
}
