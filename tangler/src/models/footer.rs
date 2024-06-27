use std::fmt;

use regex::Regex;
use serde::{Deserialize, Deserializer};
use tracing::{info, instrument};

/// Represents a footer in a commit message, which consists of a token and a value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Footer {
    pub(crate) token: String,
    pub(crate) value: String,
}


impl From<&str> for Footer {
    fn from(s: &str) -> Self {
        // Assuming the input string is formatted as "token:value"
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let token = parts.first().unwrap_or(&"").to_string();
        let value = parts.get(1).unwrap_or(&"").to_string();

        Footer { token, value }
    }
}

impl AsRef<str> for Footer {
    fn as_ref(&self) -> &str {
        // Create a formatted string with token and value
        let formatted = format!("{}:{}", &self.token, &self.value);
        // Convert the formatted string to a &str
        Box::leak(formatted.into_boxed_str())
    }
}

impl<'de> Deserialize<'de> for Footer {
    /// Deserializes a `Footer` instance from the given deserializer.
    ///
    /// This method checks for specific terms in the token and modifies it if necessary.
    #[instrument(level = "info", skip(deserializer))]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FooterData {
            token: String,
            value: String,
        }

        let mut footer = FooterData::deserialize(deserializer)?;
        let re = Regex::new(r"(?i)\b(breaking|change)\b").unwrap();
        if re.is_match(&footer.token) {
            footer.token = "BREAKING CHANGE".to_string();
        }

        // Event: Footer Deserialized
        // Description: Triggered after deserializing a Footer instance.
        // Context: Token and value of the footer.
        info!(
            token = %footer.token,
            value = %footer.value,
            "Footer instance deserialized"
        );

        Ok(Footer {
            token: footer.token,
            value: footer.value,
        })
    }
}

impl fmt::Display for Footer {
    /// Formats the `Footer` for display.
    ///
    /// This method formats the footer as "token: value".
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.token, self.value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn deserialize_footer_with_breaking() {
        let json_data = json!({ "token": "breaking news", "value": "Changes in the API" });
        let footer: Footer = serde_json::from_value(json_data).unwrap();
        assert_eq!(footer.token, "BREAKING CHANGE");
        assert_eq!(footer.value, "Changes in the API");
    }

    #[test]
    #[traced_test]
    fn deserialize_footer_with_change() {
        let json_data = json!({ "token": "this is a change", "value": "Data format updated" });
        let footer: Footer = serde_json::from_value(json_data).unwrap();
        assert_eq!(footer.token, "BREAKING CHANGE");
        assert_eq!(footer.value, "Data format updated");
    }

    #[test]
    #[traced_test]
    fn deserialize_footer_without_keyword() {
        let json_data = json!({ "token": "note", "value": "General information" });
        let footer: Footer = serde_json::from_value(json_data).unwrap();
        assert_eq!(footer.token, "note");
        assert_eq!(footer.value, "General information");
    }

    #[test]
    #[traced_test]
    fn display_footer() {
        let footer = Footer {
            token: String::from("BREAKING CHANGE"),
            value: String::from("This will break the API"),
        };
        assert_eq!(
            format!("{}", footer),
            "BREAKING CHANGE: This will break the API"
        );
    }

    #[test]
    #[traced_test]
    fn display_footer_regular() {
        let footer = Footer {
            token: String::from("info"),
            value: String::from("Routine update"),
        };
        assert_eq!(format!("{}", footer), "info: Routine update");
    }
}
