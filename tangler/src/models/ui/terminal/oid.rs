use std::fmt;
use std::io::Write;
use std::ops::Deref;

use console::style;
use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::{info, instrument};

use crate::models::{ConsoleStyle, GRASS_11, GRASS_12, GRASS_9, GRAY_10, Oid, OID_COLOR, TEAL_11, TEAL_12, TERTIARY_10};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct OidTerminal(Oid);
impl ConsoleStyle for OidTerminal{}
impl Deref for OidTerminal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for OidTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {



         write!(f, "{}", &self.0.style(*OID_COLOR));
        Ok(())

    }
}

impl From<&Oid> for OidTerminal {
    #[instrument(level = "info", skip(s))]
    fn from(s: &Oid) -> Self {
        info!(source = %s, "OidTerminal instance created from &str");
        OidTerminal(s.clone())
    }
}
