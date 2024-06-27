use std::fmt;
use std::ops::Deref;

use owo_colors::OwoColorize;
use tracing::{error, instrument};

use crate::models::{
    CommitTypeTerminal, IsBreakingTerminal, SCOPE_PUNCTUATION_COLOR,
    ScopeTerminal,
};

#[derive(Debug, Clone)]
pub(crate) struct CommitHeadingTerminal((CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal));


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
        if let Err(e) = write!(
            f,
            "{}{}{}{}{}",
            commit_type, left_parens, scope, right_parens, warning
        ) {
            error!("{:?}", e);
        }
        Ok(())
    }
}

impl From<(CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal)> for CommitHeadingTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: (CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal)) -> Self {
        CommitHeadingTerminal(s)
    }
}
