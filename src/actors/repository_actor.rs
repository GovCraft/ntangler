use std::sync::{Arc, Mutex};

use akton::prelude::*;
use git2::{Error, Repository};
use tracing::{error, trace};

use crate::ginja_config::GinjaConfig;
use crate::messages::CheckoutBranch;
use crate::repository_config::RepositoryConfig;

#[akton_actor]
pub(crate) struct RepositoryActor {
    repository: Option<Arc<Mutex<Repository>>>,
    config: RepositoryConfig,
}

impl RepositoryActor {
    pub(crate) async fn init(config: &RepositoryConfig) -> Option<Context> {
        let mut actor = Akton::<RepositoryActor>::create_with_id(&config.id);

        actor.state.config = config.clone();
        let repo = match Repository::open(&config.path) {
            Ok(repo) => repo,
            Err(e) => {
                error!("Failed to open repository: {}", e);
                return None;
            }
        };
        actor.state.repository = Some(Arc::new(Mutex::new(repo)));
        actor.setup.act_on::<CheckoutBranch>(|actor, event| {
            trace!("Received CheckoutBranch message");

            // Check if the repository is available
            if let Some(repository) = &actor.state.repository {
                // Lock the repository mutex
                let repo = repository.lock().expect("Couldn't lock repository mutex");

                // Find the branch reference
                match repo.find_branch(&actor.state.config.branch_name, git2::BranchType::Local) {
                    Ok(branch_ref) => {
                        trace!("Found branch: {}", &actor.state.config.branch_name);

                        let checkout_result = repo.checkout_head(Some(git2::build::CheckoutBuilder::new().path(&actor.state.config.path)));
                        match checkout_result {
                            Ok(_) => {
                                trace!(
                                "Checked out branch {} for repo {} at path {}",
                                &actor.state.config.branch_name,
                                &actor.state.config.id,
                                &actor.state.config.path
                            );
                            }
                            Err(e) => {
                                error!("Failed to checkout head: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to find branch: {}. Logged error is: {}", &actor.state.config.branch_name, e);
                        return;
                    }
                };
            } else {
                error!("Failed to find repository: {}", &actor.state.config.path);
            }
        });

        let context = actor.activate(None).await;
        match context {
            Ok(context) => {
                trace!("Activated RepositoryActor, attempting to checkout branch {}", &config.branch_name);
                context.emit_async(CheckoutBranch).await;
                Some(context)
            }
            Err(_) => {
                error!("Failed to activate RepositoryActor for repository at {}", &config.path);
                None
            }
        }
        // Checkout the branch
    }

    /// Opens an existing repository and checks out a specific branch.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the repository.
    /// * `branch_name` - The name of the branch to checkout.
    ///
    /// # Returns
    ///
    /// * `Result<(), Error>` - An empty result indicating success or an error.
    ///
    /// # Example
    ///
    /// ```
    /// let my_struct = MyStruct { repository: Some(Arc::new(Mutex::new(Repository::open("/path/to/repo").unwrap()))) };
    /// my_struct.open_repository_to_branch("feature-branch").unwrap();
    /// ```
    fn open_repository_to_branch(&self, branch_name: &str) -> anyhow::Result<()> {
        // Check if the repository is available
        if let Some(repository) = &self.repository {
            // Lock the repository mutex
            let repo = repository.lock().expect("Couldn't lock repository mutex");

            // Find the branch reference
            let branch_ref = repo.find_branch(branch_name, git2::BranchType::Local)?.into_reference();

            // Set the HEAD to point to the branch reference
            repo.set_head(branch_ref.name().unwrap())?;

            // Checkout the branch
            let checkout_result = repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::actors::repository_actor::RepositoryActor;
    use crate::init_tracing;
    use crate::repository_config::RepositoryConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_non_existing_branch() -> anyhow::Result<()> {
        init_tracing();
        let config = RepositoryConfig {
            path: "./mock-repo-working".to_string(),
            branch_name: "non_existing_branch".to_string(),
            api_url: "".to_string(),
            watch_staged_only: false,
            id: "any id".to_string(),
        };
        let actor_context = RepositoryActor::init(&config).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.terminate().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_existing_branch() -> anyhow::Result<()> {
        init_tracing();
        let config = RepositoryConfig {
            path: "./mock-repo-working".to_string(),
            branch_name: "new_branch".to_string(),
            api_url: "".to_string(),
            watch_staged_only: false,
            id: "any id".to_string(),
        };
        let actor_context = RepositoryActor::init(&config).await;
        assert!(actor_context.is_some());
        let context = actor_context.unwrap();
        context.terminate().await?;
        Ok(())
    }
}