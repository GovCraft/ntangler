use std::fmt;
use std::ops::Deref;
use console::style;

use serde::Deserialize;
use tracing::{info, instrument};
use crate::models::Oid;
use crate::models::traits::TanglerModel;
use derive_more::*;
#[derive(Clone,Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub(crate) struct Scope(String);
impl TanglerModel for Scope {}
impl Deref for Scope {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Scope {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            write!(f, "{}", &self.0)
        } else {
            write!(f, "")
        }
    }
}

impl From<&str> for Scope {
    #[instrument(level = "info", skip(s))]
    fn from(s: &str) -> Self {
        // Event: Scope Created
        // Description: Triggered when a new Scope instance is created from a &str.
        // Context: The string being converted to Scope.
        info!(source = %s, "Scope instance created from &str");
        let cleaned = s.split_whitespace().collect::<String>();
        Scope(cleaned)
    }
}

// Wrapper type around Option<Scope>
#[derive(Clone, Debug, Default)]
pub(crate) struct OptionalScope(pub Option<Scope>);

impl fmt::Display for OptionalScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(scope) => {
                write!(f, "{}", scope)
            }
            None => write!(f, ""),
        }
    }
}

impl From<Option<Scope>> for OptionalScope {
    fn from(option: Option<Scope>) -> Self {
        OptionalScope(option)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_scope_from_str_with_whitespace() {
        let commit: Scope = " \n".into();
        assert_eq!(commit.to_string(), "");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str_with_single_word() {
        let commit: Scope = "word".into();
        assert_eq!(commit.to_string(), "(word)");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str_with_multiple_spaces() {
        let commit: Scope = "  multiple   spaces  ".into();
        assert_eq!(commit.to_string(), "(multiplespaces)");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str_with_newlines() {
        let commit: Scope = "line\nbreaks".into();
        assert_eq!(commit.to_string(), "(linebreaks)");
    }

    #[test]
    fn test_scope_display() {
        let some_scope = OptionalScope(Some(Scope("api".to_string())));
        let none_scope = OptionalScope(None);

        assert_eq!(some_scope.to_string(), "(api)");
        assert_eq!(none_scope.to_string(), "");
    }

    #[test]
    fn test_from_option_scope() {
        let some_scope: Option<Scope> = Some(Scope("api".to_string()));
        let none_scope: Option<Scope> = None;

        let some_optional_scope: OptionalScope = some_scope.into();
        let none_optional_scope: OptionalScope = none_scope.into();

        assert_eq!(some_optional_scope.to_string(), "(api)");
        assert_eq!(none_optional_scope.to_string(), "");
    }

    #[test]
    fn test_scope_deref() {
        let scope = Scope("api".to_string());
        assert_eq!(&*scope, "api");
    }

    #[test]
    fn test_option_scope_as_deref() {
        let some_scope: Option<Scope> = Some(Scope("api".to_string()));
        let none_scope: Option<Scope> = None;

        assert_eq!(some_scope.as_deref().unwrap_or(""), "api");
        assert_eq!(none_scope.as_deref().unwrap_or(""), "");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str() {
        let commit: Scope = "something".into();
        assert_eq!(commit.to_string(), "(something)");
    }


    #[test]
    #[traced_test]
    fn test_display_empty_scope() {
        let scope = Scope(String::from(""));
        let expected_output = "";
        assert_eq!(scope.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_display_scope() {
        let scope = Scope(String::from("feat"));
        let expected_output = "(feat)";
        assert_eq!(scope.to_string(), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_scope_default() {
        let scope: Scope = Default::default();
        assert_eq!(scope.to_string(), "");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str_empty() {
        let commit: Scope = "".into();
        assert_eq!(commit.to_string(), "");
    }

    #[test]
    #[traced_test]
    fn test_scope_from_str_special_chars() {
        let commit: Scope = "!@#$%^&*()".into();
        assert_eq!(commit.to_string(), "(!@#$%^&*())");
    }
}