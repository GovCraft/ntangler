use std::fmt;

use chrono::{DateTime, Utc};
use chrono::format::StrftimeItems;
use console::style;
use serde::Deserialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument};
use crate::models::{TIME_COLOR, TEAL_11, TEAL_9, ConsoleStyle, DimStatic, GRAY_10, GRAY_11, Scope, TimeStamp, GRAY_9, GRAY_8, TIME_ELEMENTS_COLOR};
use crate::models::ui::terminal::ScopeTerminal;
use std::io::Write;
use owo_colors::OwoColorize;

/// A struct representing a timestamp in UTC.
#[derive(Clone, Default, Debug)]
pub(crate) struct TimeStampTerminal(TimeStamp);

impl ConsoleStyle for TimeStampTerminal {}

impl fmt::Display for TimeStampTerminal {
    /// Formats the `TimeStampTerminal` for display.
    ///
    /// This method converts the stored `DateTime<Utc>` to a string in the format "YYYY-MM-DD HH:MM:SS".
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Create a ColorSpec with RGB values
        let val = &self.0.to_string();

        let dim_open_bracket = DimStatic::from(("[", *TIME_ELEMENTS_COLOR));
        let dim_close_bracket = DimStatic::from(("]", *TIME_ELEMENTS_COLOR));
        let dim_colon = DimStatic::from((":", *TIME_ELEMENTS_COLOR));

// Write colored text to stderr using termcolor

// Write the open bracket
        write!(f, "{}", dim_open_bracket);

// Write each part of the formatted string
        let mut parts = val.split(':');
        if let Some(first_part) = parts.next() {
            write!(f, "{}", first_part.style(*TIME_COLOR));
        }
        for part in parts {
            write!(f, "{}", dim_colon.style(*TIME_ELEMENTS_COLOR));
            write!(f, "{}", part.style(*TIME_COLOR));
        }

// Write the close bracket
        write!(f, "{}", dim_close_bracket.style(*GRAY_9));

        Ok(())
    }
}

impl From<&TimeStamp> for TimeStampTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &TimeStamp) -> Self {
        // Event: ScopeTerminal Created
        // Description: Triggered when a new ScopeTerminal instance is created from a &str.
        // Context: The string being converted to ScopeTerminal.
        info!(source = %s, "ScopeTerminal instance created from &str");
        TimeStampTerminal(s.clone())
    }
}

