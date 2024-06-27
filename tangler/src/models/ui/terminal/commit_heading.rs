use std::fmt;
use std::io::Write;
use std::ops::Deref;

use console::style;
use owo_colors::OwoColorize;
use serde::Deserialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tracing::{info, instrument, trace};

use crate::models::{
    CommitType, CommitTypeTerminal, ConsoleStyle, IsBreakingTerminal, Scope, ScopeTerminal,
    GRAY_11, GRAY_12, SCOPE_PUNCTUATION_COLOR, TEAL_11, TEAL_12,
};

#[derive(Debug, Clone)]
pub(crate) struct CommitHeadingTerminal((CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal));

impl ConsoleStyle for CommitHeadingTerminal {}

impl Deref for CommitHeadingTerminal {
    type Target = (CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for CommitHeadingTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (commit_type, scope, warning) = &self.0;
        let left_parens = "(".style(*SCOPE_PUNCTUATION_COLOR);
        let right_parens = ")".style(*SCOPE_PUNCTUATION_COLOR);
        write!(
            f,
            "{commit_type}{left_parens}{scope}{right_parens}{warning}"
        );
        Ok(())
    }
}

impl From<(CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal)> for CommitHeadingTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: (CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal)) -> Self {
        CommitHeadingTerminal(s)
    }
}
