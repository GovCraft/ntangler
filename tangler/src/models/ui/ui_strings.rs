use crate::models::{REPO_COLOR, TIME_COLOR};
use lazy_static::lazy_static;
use owo_colors::OwoColorize;

pub(crate) const COLUMN_HEADING_ONE: &str = "REPOSITORY";
pub(crate) const COLUMN_HEADING_TWO: &str = "TIME";
pub(crate) const COLUMN_HEADING_THREE: &str = "OID";
pub(crate) const COLUMN_HEADING_FOUR: &str = "SEMVERS";
pub(crate) const COLUMN_HEADING_FIVE: &str = "FILENAME";
pub(crate) const COLUMN_HEADING_SIX: &str = "TYPE(SCOPE)";
pub(crate) const COLUMN_HEADING_SEVEN: &str = "SUMMARY";

lazy_static! {
    pub static ref COLUMN_HEADING_ONE_LENGTH: usize = COLUMN_HEADING_ONE.len();
    pub static ref COLUMN_HEADING_TWO_LENGTH: usize = COLUMN_HEADING_TWO.len() + 4;
    pub static ref COLUMN_HEADING_THREE_LENGTH: usize = COLUMN_HEADING_THREE.len() + 5;
    pub static ref COLUMN_HEADING_FOUR_LENGTH: usize = COLUMN_HEADING_FOUR.len();
    pub static ref COLUMN_HEADING_FIVE_LENGTH: usize = COLUMN_HEADING_FIVE.len();
    pub static ref COLUMN_HEADING_SIX_LENGTH: usize = COLUMN_HEADING_SIX.len();
    pub static ref COLUMN_HEADING_SEVEN_LENGTH: usize = COLUMN_HEADING_SEVEN.len();
}
