use std::fmt;
use crate::models::Scope;
use crate::models::traits::TanglerModel;

#[derive(Clone, Debug, Default)]
pub(crate) enum SemVerImpact {
    #[default]
    NoImpact,
    Patch,
    Minor,
    Major,
}
impl TanglerModel for SemVerImpact {}
impl fmt::Display for SemVerImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            SemVerImpact::NoImpact => "", // ✅
            SemVerImpact::Patch => "PATCH",    // 🩹
            SemVerImpact::Minor => "MINOR",    // 🔧
            SemVerImpact::Major => "MAJOR",    // 💥
        };
        write!(f, "{}", symbol)
    }
}

impl From<&str> for SemVerImpact {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "patch" => SemVerImpact::Patch,
            "minor" => SemVerImpact::Minor,
            "major" => SemVerImpact::Major,
            _ => SemVerImpact::NoImpact,
        }
    }
}