use std::fmt;
use std::io::Write;
use std::ops::Deref;

use console::style;
use owo_colors::{OwoColorize, Style};
use serde::Deserialize;
use tracing::{info, instrument, trace};

use crate::models::{
    ConsoleStyle, Description, GRAY_10, GRAY_11, GRAY_12, GRAY_9, RED_9, TEAL_11, TEAL_12, TEAL_7,
};

#[derive(Debug, Clone)]
pub(crate) struct DimStatic {
    str: &'static str,
    color: Style,
}

impl ConsoleStyle for DimStatic {}

impl Deref for DimStatic {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.str
    }
}

impl fmt::Display for DimStatic {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create a ColorSpec with RGB values
        write!(f, "{}", &self.str.style(self.color)).unwrap();
        Ok(())
    }
}

impl From<(&'static str, Style)> for DimStatic {
    #[instrument(level = "info", skip(s))]
    fn from(s: (&'static str, Style)) -> Self {
        // Event: DescriptionTerminal Created
        // DescriptionTerminal: Triggered when a new DescriptionTerminal instance is created from a &str.
        // Context: The string being converted to DescriptionTerminal.
        let (str, color) = s;
        trace!(source = %str, "DescriptionTerminal instance created from &str");
        DimStatic { str, color }
    }
}
