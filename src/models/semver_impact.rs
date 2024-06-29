use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SemVerImpact {
    #[default]
    NoImpact,
    Patch,
    Minor,
    Major,
}

impl fmt::Display for SemVerImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            SemVerImpact::NoImpact => "\u{2022}", // ✅
            SemVerImpact::Patch => "PATCH",       // 🩹
            SemVerImpact::Minor => "MINOR",       // 🔧
            SemVerImpact::Major => "MAJOR",       // 💥
        };
        write!(f, "{symbol}")
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
