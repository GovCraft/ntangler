use crate::repository_config::RepositoryConfig;
use akton::prelude::*;

#[akton_actor]
pub(crate) struct RepositoryWatcherActor {
    id: String,
}

impl RepositoryWatcherActor {
    pub(crate) async fn init(config: &RepositoryConfig) -> anyhow::Result<Context> {
        let actor = Akton::<RepositoryWatcherActor>::create_with_id(&config.id);

        let context = actor.activate(None).await?;
        Ok(context)
    }
}
