use std::fmt;
use std::ops::Deref;

use console::style;
use owo_colors::OwoColorize;
use serde::Deserialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument, trace};

use crate::models::{ConsoleStyle, Filename, FILENAME, GRAY_11, GRAY_12, TEAL_11, TEAL_12};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct FilenameTerminal(Filename);

impl ConsoleStyle for FilenameTerminal {}

impl Deref for FilenameTerminal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FilenameTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //        write!(f, "{:>10}", self.0.style(REPO_COLOR.clone()));
        write!(f, "{:>15}", &self.0.style(*FILENAME));
        Ok(())
    }
}

impl From<&Filename> for FilenameTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &Filename) -> Self {
        // Event: FilenameTerminal Created
        // FilenameTerminal: Triggered when a new FilenameTerminal instance is created from a &str.
        // Context: The string being converted to FilenameTerminal.
        trace!(source = %s, "FilenameTerminal instance created from &str");
        FilenameTerminal(s.clone())
    }
}
