use std::fmt;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{instrument, trace};
use crate::models::{TEAL_9, TEAL_12, ConsoleStyle, Description, DescriptionTerminal, GRAY_11, GRAY_12, Scope, RED_9, TEAL_11, WHITE_PURE, AMBER_9, AMBER_12, GRAY_9, GRAY_10, BG_DARK, ACCENT, MAJOR, MINOR, PATCH};
use crate::models::semver_impact::SemVerImpact;
use crate::models::traits::TanglerModel;
use std::io::Write;
use owo_colors::OwoColorize;

#[derive(Clone, Debug, Default)]
pub(crate) struct SemVerImpactTerminal(SemVerImpact);

impl ConsoleStyle for SemVerImpactTerminal {}

impl fmt::Display for SemVerImpactTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.to_string().as_str() {
            "MAJOR" => {

                write!(f, "{:5}", "MAJOR".style(MAJOR.clone().bold()));
            }
            "MINOR" => {

                write!(f, "{:5}", "MINOR".style(MINOR.clone()));
            }
            "PATCH" => {

                write!(f, "{:5}", "PATCH".style(PATCH.clone()));
            }
            _ => {
                write!(f, "{:^5}", "\u{2014}".style(GRAY_9.clone()));
            }
        };

        // Write colored text to stderr using termcolor
        Ok(())
    }
}

impl From<&SemVerImpact> for SemVerImpactTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &SemVerImpact) -> Self {
        // Event: SemVerImpactTerminal Created
        // SemVerImpactTerminal: Triggered when a new SemVerImpactTerminal instance is created from a &str.
        // Context: The string being converted to SemVerImpactTerminal.
        trace!(source = %s, "SemVerImpactTerminal instance created from &str");
        SemVerImpactTerminal(s.clone())
    }
}