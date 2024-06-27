use std::fmt;
use std::time::Duration;
use owo_colors::OwoColorize;
use crate::models::STATUS;

use indicatif::ProgressBar;
use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) enum Status {
    #[default]
    Pending,
    Thinking,
    Committing,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       let symbol = match self {
           Status::Pending => { "PENDING" }
           Status::Thinking => { "WRITING" }
            Status::Committing => { "STAGING" }
        };
        write!(f, "{:8}", symbol)
    }
}
