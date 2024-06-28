use std::fmt;
use std::ops::Deref;

use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::{error, instrument, trace};

use crate::models::{
    REPO_COLOR, TAB_WIDTH,
};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct RepositoryTerminal(String);


impl Deref for RepositoryTerminal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RepositoryTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let half_tab = " ".repeat(TAB_WIDTH / 2);
        match write!(
            f,
            "{half_tab}{:>width$}",
            self.0.style(*REPO_COLOR),
            width = 10 - self.0.len()
        ) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                error!("{:?}",e);
                Ok(())
            }
        }
    }
}

impl From<&String> for RepositoryTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &String) -> Self {
        // Event: RepositoryTerminal Created
        // RepositoryTerminal: Triggered when a new RepositoryTerminal instance is created from a &str.
        // Context: The string being converted to RepositoryTerminal.
        trace!(source = %s, "RepositoryTerminal instance created from &str");
        RepositoryTerminal(s.clone())
    }
}
