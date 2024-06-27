use std::fmt;

use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};

use crate::models::{CommitType, Description, Filename, Footer, generate_id, Oid, Scope, TimeStamp};
use crate::models::semver_impact::SemVerImpact;
use crate::models::traits::TanglerModel;
use derive_more::*;

impl From<CommittedCommit> for String {
    fn from(commit_details: CommittedCommit) -> Self {
        commit_details.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct CommittedCommit {
    pub(crate) id: String,
    pub(crate) repository: String,
    pub(crate) commit_type: CommitType,
    pub(crate) scope: Option<Scope>,
    pub(crate) description: Description,
    pub(crate) filename: Filename,
    pub(crate) body: String,
    pub(crate) is_breaking: bool,
    pub(crate) footers: Vec<Footer>,
    pub(crate) timestamp: TimeStamp,
    pub(crate) oid: Oid,
    pub(crate) semver_impact: SemVerImpact,
}

impl TanglerModel for CommittedCommit {}

impl From<&str> for CommittedCommit {
    fn from(s: &str) -> Self {
        let commit_details: CommittedCommit = serde_json::from_str(s).unwrap_or_default();
        commit_details
    }
}

impl<'de> Deserialize<'de> for CommittedCommit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CommitHelper {
            #[serde(skip)]
            id: String,
            //            #[serde(rename = "type")]
            commit_type: String, // Adjusted to String
            scope: Option<Scope>,
            description: Description,
            body: String,
            #[serde(skip)]
            file_name: Filename,
            is_breaking: bool,
            footers: Vec<Footer>,
        }

        struct CommitVisitor;

        impl<'de> Visitor<'de> for CommitVisitor {
            type Value = CommittedCommit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Commit")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CommittedCommit, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut commit_type = None;
                let mut scope = None;
                let mut description = None;
                let mut body = None;
                let mut breaking = None;
                let mut footers: Vec<Footer> = Vec::new();

                while let Some(key) = map.next_key()? {
                    match key {
                        "type" => {
                            if commit_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            commit_type = Some(map.next_value()?);
                        }
                        "scope" => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            scope = Some(map.next_value()?);
                        }
                        "description" => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value()?);
                        }
                       "body" => {
                            if body.is_some() {
                                return Err(de::Error::duplicate_field("body"));
                            }
                            body = Some(map.next_value()?);
                        }
                        "breaking" => {
                            if breaking.is_some() {
                                return Err(de::Error::duplicate_field("breaking"));
                            }
                            breaking = Some(map.next_value()?);
                        }
                        "footers" => {
                            footers = map.next_value()?;
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let commit_type: CommitType = commit_type.ok_or_else(|| de::Error::missing_field("type"))?;
                let scope = scope.unwrap_or(None);
                let description = description.ok_or_else(|| de::Error::missing_field("description"))?;
                let body = body.ok_or_else(|| de::Error::missing_field("body"))?;
                let breaking = breaking.ok_or_else(|| de::Error::missing_field("breaking"))?;

                let semver_impact = CommittedCommit::calculate_semver_impact(&commit_type, breaking);

                CommittedCommit::calculate_footers(&mut footers, &commit_type, breaking);

                Ok(CommittedCommit {
                    commit_type,
                    scope,
                    description,
                    body,
                    is_breaking: breaking,
                    footers,
                    timestamp: TimeStamp::default(),
                    oid: Oid::default(),
                    semver_impact,
                    ..Default::default()
                })
            }
        }

        const FIELDS: &[&str] = &[
            "type", "scope", "description", "body", "breaking", "footers",
        ];
        deserializer.deserialize_struct("Commit", FIELDS, CommitVisitor)
    }
}

impl CommittedCommit {
    pub(crate) fn set_id(&mut self, repository:String, filename: &str ) {


        let id = generate_id(&repository, Filename::from(filename));
        self.repository = repository;
        self.filename = Filename::from(filename);
       self.id = id;
    }
    pub(crate) fn calculate_footers(footers: &mut Vec<Footer>, commit_type: &CommitType, breaking: bool) {
        if breaking {
            if !footers.iter().any(|footer| footer.token == "BREAKING CHANGES") {
                footers.push(Footer {
                    token: "BREAKING CHANGES".to_string(),
                    value: "You have made changes that may break backward compatibility. According to Semantic Versioning (SemVer), this requires a major version update. Please verify and update your version number accordingly.".to_string(),
                });
            }
        } else {
            match commit_type.to_string().as_str() {
                "fix" => {
                    footers.push(Footer {
                        token: "BUG FIX".to_string(),
                        value: "You appear to have made one or more backward-compatible bug fixes. According to Semantic Versioning (SemVer), this requires a patch version update. Please verify and update your version number accordingly.".to_string(),
                    });
                }
                "feat" => {
                    footers.push(Footer {
                        token: "NEW FEATURE".to_string(),
                        value: "You appear to have introduced one or more new features that are backward-compatible. According to Semantic Versioning (SemVer), this requires a minor version update. Please verify and update your version number accordingly.".to_string(),
                    });
                }
                _ => (),
            }
        };
    }

    pub(crate) fn calculate_semver_impact(commit_type: &CommitType, breaking: bool) -> SemVerImpact {
        if breaking {
            SemVerImpact::Major
        } else {
            match commit_type.to_string().as_str() {
                "fix" => {
                    SemVerImpact::Patch
                }
                "feat" => {
                    SemVerImpact::Minor
                }
                _ => SemVerImpact::NoImpact,
            }
        }
    }
}

impl fmt::Display for CommittedCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope_display = self.scope.as_deref().map_or_else(String::new, |s| format!("({})", s));
        let breaking_marker = if self.is_breaking { "!" } else { "" };
        let footers = self.footers.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");

        write!(
            f,
            "{}{}{}: {}\n\n{}\n\n{}",
            self.commit_type, scope_display, breaking_marker, self.description, self.body, footers
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tracing::info;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    fn test_deserialize_footer_with_breaking_change() {
        let json_data = json!({
            "token": "breaking change",
            "value": "The api is different"
        });

        let footer: Footer = serde_json::from_value(json_data).unwrap();
        assert_eq!(footer.token, "BREAKING CHANGE");
        assert_eq!(footer.value, "The api is different");
    }

    #[test]
    fn test_deserialize_footer_without_breaking_change() {
        let json_data = json!({
            "token": "Reviewed-by",
            "value": "Z"
        });

        let footer: Footer = serde_json::from_value(json_data).unwrap();
        assert_eq!(footer.token, "Reviewed-by");
        assert_eq!(footer.value, "Z");
    }

    #[test]
    fn test_display_commit_details() {
        let commit_type: CommitType = "fix".into();
        let commit_details = CommittedCommit {
            commit_type: commit_type.clone(),
            scope: Some("api".into()),
            description: "prevent racing of requests".into(),
            body: "Introduce a request id and a reference to latest request. Dismiss incoming responses other than from latest request.".to_string(),
            footers: vec![
                Footer { token: "Reviewed-by".to_string(), value: "Z".to_string() },
                Footer { token: "Refs".to_string(), value: "#123".to_string() },
            ],
            semver_impact: CommittedCommit::calculate_semver_impact(&commit_type, false),
            ..Default::default()
        };


        let expected_output = "🩹 fix(api): prevent racing of requests\n\nIntroduce a request id and a reference to latest request. Dismiss incoming responses other than from latest request.\n\nReviewed-by: Z\nRefs: #123";
        assert_eq!(format!("{}", &commit_details), expected_output);
    }

    #[test]
    fn test_deserialize_commit_details_with_scope() {
        let json_data = r#"
{ "type": "fix", "scope": "actors", "description": "fix commit message deserialization", "body": "Corrected the deserialization logic to properly handle commit messages within the PooledActor for OpenAi. Previously, it was attempting to deserialize into a 'commits' structure, which has been fixed to deserialize into a 'commit' structure.", "breaking": false, "footers": [] }
        "#;

        let commit_details: CommittedCommit = serde_json::from_str(json_data).unwrap();

        assert_eq!(commit_details.commit_type, "fix".into());
        assert_eq!(commit_details.scope, Some("actors".into()));
        assert_eq!(
            commit_details.description,
            "fix commit message deserialization".into()
        );
        assert_eq!(commit_details.body, "Corrected the deserialization logic to properly handle commit messages within the PooledActor for OpenAi. Previously, it was attempting to deserialize into a 'commits' structure, which has been fixed to deserialize into a 'commit' structure.");
        assert_eq!(commit_details.footers.len(), 1); // the "fix" commit type generates a "BUG FIX" semver footer
    }

    #[test]
    fn test_deserialize_commit_details_without_scope() {
        let json_data = r#"
        {
            "type": "fix",
            "description": "prevent racing of requests",
            "body": "Introduce a request id and a reference to latest request. Dismiss incoming responses other than from latest request.",
            "footers": [
                { "token": "Reviewed-by", "value": "Z" },
                { "token": "Refs", "value": "123" }
            ],
            "breaking":false
        }
        "#;

        let commit_details: CommittedCommit = serde_json::from_str(json_data).unwrap();

        assert_eq!(commit_details.commit_type, "fix".into());
        assert_eq!(commit_details.scope, None);
        assert_eq!(commit_details.description, "prevent racing of requests".into());
        assert_eq!(commit_details.body, "Introduce a request id and a reference to latest request. Dismiss incoming responses other than from latest request.");
        assert_eq!(commit_details.footers.len(), 3); //2 plus one more because the "fix" commit type would generate a BUG FIX semver footer
        assert_eq!(commit_details.footers[0].token, "Reviewed-by");
        assert_eq!(commit_details.footers[0].value, "Z");
        assert_eq!(commit_details.footers[1].token, "Refs");
        assert_eq!(commit_details.footers[1].value, "123");
        assert_eq!(commit_details.semver_impact.to_string(), "🩹");
    }

    #[test]
    #[traced_test]
    fn test_display_commit_details_without_footers() {
        let commit_details = CommittedCommit {
            commit_type: "feat".into(),
            scope: None,
            description: "add new feature".into(),
            body: "Implemented a new feature without any known issues.".to_string(),
            footers: vec![],
            ..Default::default()
        };

        let expected_output =
            "✅ feat: add new feature\n\nImplemented a new feature without any known issues.\n\n";
        assert_eq!(format!("{}", commit_details), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_display_commit_details_long_scope() {
        let commit_details = CommittedCommit {
            commit_type: "refactor".into(),
            scope: Some("very_long_and_descriptive_scope_name".into()),
            description: "refactor component".into(),
            body: "This refactor improves the component by a significant margin.".to_string(),
            footers: vec![],
            ..Default::default()
        };

        let expected_output = "✅ refactor(very_long_and_descriptive_scope_name): refactor component\n\nThis refactor improves the component by a significant margin.\n\n";
        assert_eq!(format!("{}", commit_details), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_deserialize_commit_with_empty_body() {
        let json_data = r#"
        {
            "type": "docs",
            "scope": null,
            "description": "update documentation",
            "body": "",
            "footers": [
                { "token": "Reviewed-by", "value": "X" }
            ],
            "breaking": false
        }
        "#;

        let commit_details: CommittedCommit = serde_json::from_str(json_data).unwrap();

        info!(
            commit_type = %commit_details.commit_type,
            scope = ?commit_details.scope,
            description = %commit_details.description,
            body = %commit_details.body,
            footers = ?commit_details.footers,
            "Commit details deserialized"
        );

        assert_eq!(commit_details.commit_type, "docs".into());
        assert_eq!(commit_details.scope, None);
        assert_eq!(commit_details.description, "update documentation".into());
        assert_eq!(commit_details.body, "");
        assert_eq!(commit_details.footers.len(), 1);
        assert_eq!(commit_details.footers[0].token, "Reviewed-by");
        assert_eq!(commit_details.footers[0].value, "X");
    }

    #[test]
    #[traced_test]
    fn test_commit_details_with_no_scope() {
        let commit_details = CommittedCommit {
            commit_type: "chore".into(),
            scope: None,
            description: "maintenance tasks".into(),
            body: "Performed various maintenance tasks across the project.".to_string(),
            footers: vec![Footer {
                token: "Refs".to_string(),
                value: "#456".to_string(),
            }],
            ..Default::default()
        };

        let expected_output = "✅ chore: maintenance tasks\n\nPerformed various maintenance tasks across the project.\n\nRefs: #456";
        assert_eq!(format!("{}", commit_details), expected_output);
    }

    #[test]
    #[traced_test]
    fn test_commit_details_partial() {
        let footer = Footer {
            token: "Co-authored-by".to_string(),
            value: "Jane Doe".to_string(),
        };
        assert_eq!(footer.token, "Co-authored-by");
        assert_eq!(footer.value, "Jane Doe");

        let commit = CommittedCommit {
            commit_type: "fix".into(),
            scope: Some("ui".into()),
            description: "fix button alignment".into(),
            body: "Fixed the misalignment of buttons on the toolbar.".to_string(),
            footers: vec![footer],
            ..Default::default()
        };

        let first_footer = &commit.footers[0];
        assert_eq!(first_footer.token, "Co-authored-by");
        assert_eq!(first_footer.value, "Jane Doe");
    }
}
