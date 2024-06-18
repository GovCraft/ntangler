use std::fmt;
use std::ops::Deref;

use serde::Deserialize;
use tracing::{info, instrument};

use crate::models::Footer;
use crate::models::traits::TanglerModel;

#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub(crate) struct Oid(String);

impl TanglerModel for Oid {}

impl Oid {
    pub(crate) fn new(oid: &str) -> Oid {
        Oid(oid.to_string())
    }
}

impl Deref for Oid {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Oid {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let oid = &self.0[..7];
        let oid = format!("{}", oid.to_lowercase());
        write!(f, "{}", oid)
    }
}

impl From<&str> for Oid {
    #[instrument(level = "info", skip(s))]
    fn from(s: &str) -> Self {
        info!(source = %s, "Oid instance created from &str");
        Oid(s.to_string())
    }
}
