use std::fmt;
use std::ops::Deref;

use console::style;
use owo_colors::OwoColorize;
use serde::Deserialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument, trace};

use crate::models::{
    ConsoleStyle,  ALERT_COLOR, BG_DARK, GRAY_10, GRAY_11, GRAY_12, GRAY_9,
    PUNCTUATION_COLOR, SCOPE_PUNCTUATION_COLOR, TEAL_11, TEAL_12, TIME_PUNCTUATION_COLOR,
    WHITE_PURE,
};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct IsBreakingTerminal(bool);

impl ConsoleStyle for IsBreakingTerminal {}

impl Deref for IsBreakingTerminal {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for IsBreakingTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 {
            write!(
                f,
                "{}{}",
                "!".style(*ALERT_COLOR).bold(),
                ":".style(*SCOPE_PUNCTUATION_COLOR)
            );
        } else {
            write!(f, "{}", ":".style(*SCOPE_PUNCTUATION_COLOR));
        }

        Ok(())
    }
}

impl From<&bool> for IsBreakingTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &bool) -> Self {
        // Event: IsBreakingTerminal Created
        // IsBreakingTerminal: Triggered when a new IsBreakingTerminal instance is created from a bool.
        // Context: The string being converted to IsBreakingTerminal.
        trace!(source = %s, "IsBreakingTerminal instance created from bool");
        IsBreakingTerminal(*s)
    }
}
