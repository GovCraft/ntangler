use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

/// Represents a repository configuration.
#[derive(Serialize, Default, Clone)]
pub(crate) struct RepositoryConfig {
    pub(crate) path: String,
    pub(crate) branch_name: String,
    pub(crate) api_url: String,
    pub(crate) watch_staged_only: bool,
    pub(crate) id: String,
}

impl fmt::Debug for RepositoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RepositoryConfig")
            .field("path", &self.path)
            .field("branch_name", &self.branch_name)
            .field("api_url", &self.api_url)
            .field("watch_staged_only", &self.watch_staged_only)
            .field("id", &self.id)
            .finish()
    }
}

impl<'de> Deserialize<'de> for RepositoryConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RepositoryHelper {
            path: String,
            branch_name: String,
            api_url: String,
            watch_staged_only: bool,
        }

        let helper = RepositoryHelper::deserialize(deserializer)?;
        let id = calculate_id(&helper.path, &helper.branch_name);
        Ok(RepositoryConfig {
            path: helper.path,
            branch_name: helper.branch_name,
            api_url: helper.api_url,
            watch_staged_only: helper.watch_staged_only,
            id,
        })
    }
}

/// Calculates a unique ID based on the hash of the path and branch name.
fn calculate_id(path: &str, branch_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path);
    hasher.update(branch_name);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use toml;

    use super::*;

    #[test]
    fn test_calculate_id() {
        let path = "./repo1";
        let branch_name = "main";
        let expected_id = "0795541b2f2680e608aecced8c5d6b6ed85f2a2631794c9010a188f4083e4582";
        let id = calculate_id(path, branch_name);
        assert_eq!(id, expected_id);
    }

    #[test]
    fn test_deserialize_repository() {
        let toml_str = r#"
            [[repository]]
            path = "./repo1"
            branch_name = "main"
            api_url = "https://api.example.com/generate-commit-message"
            watch_staged_only = true

            [[repository]]
            path = "./repo2"
            branch_name = "develop"
            api_url = "https://api.example.com/generate-commit-message"
            watch_staged_only = true
        "#;

        #[derive(Debug, Deserialize)]
        struct Repositories {
            repository: Vec<RepositoryConfig>,
        }

        let _repositories: Repositories = toml::from_str(toml_str).unwrap();
        let _expected_repositories = vec![
            RepositoryConfig {
                path: "./repo1".to_string(),
                branch_name: "main".to_string(),
                api_url: "https://api.example.com/generate-commit-message".to_string(),
                watch_staged_only: true,
                id: calculate_id("./repo1", "main"),
            },
            RepositoryConfig {
                path: "./repo2".to_string(),
                branch_name: "develop".to_string(),
                api_url: "https://api.example.com/generate-commit-message".to_string(),
                watch_staged_only: true,
                id: calculate_id("./repo2", "develop"),
            },
        ];

        // TODO: revisit
        // assert_eq!(repositories.repository, expected_repositories);
    }

    #[test]
    fn test_deserialize_single_repository() {
        let toml_str = r#"
            [[repository]]
            path = "./repo1"
            branch_name = "main"
            api_url = "https://api.example.com/generate-commit-message"
            watch_staged_only = true
        "#;

        #[derive(Debug, Deserialize)]
        struct Repositories {
            repository: Vec<RepositoryConfig>,
        }

        let repositories: Repositories = toml::from_str(toml_str).unwrap();
        let expected_repositories = vec![RepositoryConfig {
            path: "./repo1".to_string(),
            branch_name: "main".to_string(),
            api_url: "https://api.example.com/generate-commit-message".to_string(),
            watch_staged_only: true,
            id: calculate_id("./repo1", "main"),
        }];

        // TODO: revisit
        assert_eq!(repositories.repository, expected_repositories);
    }
}
