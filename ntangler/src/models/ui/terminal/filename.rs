use std::fmt;
use std::ops::Deref;

use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::{error, instrument, trace};

use crate::models::{Filename, FILENAME};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct FilenameTerminal(Filename);


impl Deref for FilenameTerminal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FilenameTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Err(e) = write!(f, "{:>15}", self.0.style(*FILENAME)) {
            error!("{:?}", e);
        }
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
