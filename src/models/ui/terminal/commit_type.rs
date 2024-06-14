use std::fmt;
use console::style;
use owo_colors::OwoColorize;


use serde::Deserialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument};
use crate::models::{TEAL_9, TEAL_12, CommitType, ConsoleStyle, TEAL_11, GRAY_9, GRAY_12, WHITE_PURE, GRAY_3, COMMIT_TYPE_COLOR};

use super::*;

/// Represents the type of a commit.
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct CommitTypeTerminal(CommitType);
impl ConsoleStyle for CommitTypeTerminal{}

impl fmt::Display for CommitTypeTerminal {
    /// Formats the `CommitTypeTerminal` for display.
    ///
    /// This method simply writes the inner `String`.
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Write colored text to stderr using termcolor
        write!(f, "{}", &self.0.style(COMMIT_TYPE_COLOR.clone()).bold()).unwrap();

        Ok(())
    }
}

impl From<&CommitType> for CommitTypeTerminal {
    /// Creates a `CommitTypeTerminal` from a `&str`.
    ///
    /// This function converts the input string to a `CommitTypeTerminal` and logs the event.
    #[instrument(level = "info", skip(s))]
    fn from(s: &CommitType) -> Self {
        // Event: CommitTypeTerminal Created
        // Description: Triggered when a new CommitTypeTerminal instance is created from a &str.
        // Context: The string being converted to CommitTypeTerminal.
        info!(source = %s, "CommitTypeTerminal instance created from &str");
        CommitTypeTerminal(s.clone())
    }
}
