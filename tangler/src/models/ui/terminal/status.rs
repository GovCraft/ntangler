use std::fmt;
use std::time::Duration;
use owo_colors::OwoColorize;
use crate::models::STATUS;

use indicatif::ProgressBar;
#[derive(Default, Clone, Debug)]
pub(crate) enum Status {
    #[default]
    Pending,
    Generating,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       let symbol = match self {
            Status::Pending => { "PENDING" }
            Status::Generating => { "EDITING" }
        };
        write!(f, "{:13}", symbol.style(*STATUS))
    }
}
