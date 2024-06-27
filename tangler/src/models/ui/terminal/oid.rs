use std::fmt;
use std::ops::Deref;

use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::{error, info, instrument};

use crate::models::{
    Oid, OID_COLOR,
};

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct OidTerminal(Oid);

impl Deref for OidTerminal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for OidTerminal {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Err(e) = write!(f, "{}", self.0.style(*OID_COLOR)) {
            error!("{:?}", e);
        }
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
