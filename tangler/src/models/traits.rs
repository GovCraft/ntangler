use std::fmt::Display;
use tracing::{instrument, trace};
use crate::models::Description;

pub(crate) trait TanglerModel: Display + for<'a> From<&'a str>{}

