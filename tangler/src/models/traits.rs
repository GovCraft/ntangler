use crate::models::Description;
use std::fmt::Display;
use tracing::{instrument, trace};

pub(crate) trait TanglerModel: Display + for<'a> From<&'a str> {}
