use std::default::Default;
use std::fmt;
use std::ops::Deref;

use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::{error, info, instrument};

use crate::models::{
    Scope,
    SCOPE_COLOR,
};

#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct ScopeTerminal(Scope);


impl Deref for ScopeTerminal {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ScopeTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Err(e) = write!(f, "{}", self.0.style(*SCOPE_COLOR)) {
            error!("{:?}", e);
        }
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
