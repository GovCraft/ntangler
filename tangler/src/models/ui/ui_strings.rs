use lazy_static::lazy_static;

use crate::models::TAB_WIDTH;

pub(crate) const COLUMN_HEADING_ONE: &str = "REPOSITORY";
pub(crate) const COLUMN_HEADING_TWO: &str = "TIME";
pub(crate) const COLUMN_HEADING_THREE: &str = "OID";
pub(crate) const COLUMN_HEADING_FOUR: &str = "SEMVERS";
pub(crate) const COLUMN_HEADING_FIVE: &str = "FILENAME";
pub(crate) const COLUMN_HEADING_SIX: &str = "TYPE(SCOPE)";
pub(crate) const COLUMN_HEADING_SEVEN: &str = "SUMMARY";
pub(crate) const EMDASH: &str = "\u{2022}";
lazy_static! {
    pub static ref TAB: String = " ".repeat(TAB_WIDTH);
    pub static ref HALFTAB: String = " ".repeat(TAB_WIDTH / 2);
    pub static ref COLUMN_HEADING_ONE_LENGTH: usize = COLUMN_HEADING_ONE.len() + 5;
    pub static ref COLUMN_HEADING_TWO_LENGTH: usize = COLUMN_HEADING_TWO.len() + 4;
    pub static ref COLUMN_HEADING_THREE_LENGTH: usize = COLUMN_HEADING_THREE.len() + 5;
    pub static ref COLUMN_HEADING_FOUR_LENGTH: usize = COLUMN_HEADING_FOUR.len();
    pub static ref COLUMN_HEADING_FIVE_LENGTH: usize = COLUMN_HEADING_FIVE.len();
    pub static ref COLUMN_HEADING_SIX_LENGTH: usize = COLUMN_HEADING_SIX.len();
    pub static ref COLUMN_HEADING_SEVEN_LENGTH: usize = COLUMN_HEADING_SEVEN.len();
}
