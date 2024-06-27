use console::style;
use owo_colors::colors::*;
use owo_colors::{OwoColorize, Style};
use std::default::Default;
use std::fmt;
use std::io::Write;
use std::ops::Deref;

use crate::models::{
    ConsoleStyle, DimStatic, OptionalScope, Scope, AMBER_12, GRAY_10, GRAY_11, GRAY_12, GRAY_3,
    GRAY_9, SCOPE_COLOR, TEAL_11, TEAL_12, TEAL_9, WHITE_PURE,
};
use serde::Deserialize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument};

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct ScopeTerminal(Scope);

impl ConsoleStyle for ScopeTerminal {}

impl Deref for ScopeTerminal {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ScopeTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.style(*SCOPE_COLOR));

        Ok(())
    }
}

impl From<&Scope> for ScopeTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &Scope) -> Self {
        // Event: ScopeTerminal Created
        // Description: Triggered when a new ScopeTerminal instance is created from a &str.
        // Context: The string being converted to ScopeTerminal.
        info!(source = %s, "ScopeTerminal instance created from &str");
        ScopeTerminal(s.clone())
    }
}

impl From<&Option<Scope>> for ScopeTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &Option<Scope>) -> Self {
        if let Some(s) = s {
            ScopeTerminal(s.clone())
        } else {
            ScopeTerminal(Scope::default())
        }
    }
}
