use console::style;
use derive_more::*;
use std::fmt;

use crate::models::traits::TanglerModel;
use serde::Deserialize;
use tracing::{info, instrument};

use super::*;

/// Represents the type of a commit.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) struct CommitType(String);
impl TanglerModel for CommitType {}
impl fmt::Display for CommitType {
    /// Formats the `CommitType` for display.
    ///
    /// This method simply writes the inner `String`.
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for CommitType {
    /// Creates a `CommitType` from a `&str`.
    ///
    /// This function converts the input string to a `CommitType` and logs the event.
    #[instrument(level = "info", skip(s))]
    fn from(s: &str) -> Self {
        // Event: CommitType Created
        // Description: Triggered when a new CommitType instance is created from a &str.
        // Context: The string being converted to CommitType.
        info!(source = %s, "CommitType instance created from &str");
        CommitType(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_commit_type_from_str() {
        let commit: CommitType = "something".into();
        assert_eq!(commit.to_string(), "something");
    }

    #[test]
    #[traced_test]
    fn test_display_empty_commit_type() {
        let commit_type = CommitType(String::from(""));
        let expected_output = "";
        assert_eq!(commit_type.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_display_commit_type() {
        let commit_type = CommitType(String::from("feat"));
        let expected_output = "feat";
        assert_eq!(commit_type.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_commit_type_default() {
        let commit_type: CommitType = Default::default();
        assert_eq!(commit_type.to_string(), "");
    }

    #[test]
    #[traced_test]
    fn test_commit_type_from_str_empty() {
        let commit: CommitType = "".into();
        assert_eq!(commit.to_string(), "");
    }

    #[test]
    #[traced_test]
    fn test_commit_type_from_str_with_whitespace() {
        let commit: CommitType = " \n".into();
        assert_eq!(commit.to_string(), " \n");
    }

    #[test]
    #[traced_test]
    fn test_commit_type_from_str_special_chars() {
        let commit: CommitType = "!@#$%^&*()".into();
        assert_eq!(commit.to_string(), "!@#$%^&*()");
    }
}
