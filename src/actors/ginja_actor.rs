use akton::prelude::*;
use crate::actors::repository_actor::RepositoryActor;
use crate::ginja_config::GinjaConfig;
use crate::repository_config::RepositoryConfig;

#[akton_actor]
pub(crate) struct GinjaActor {
    repository_actors: Vec<Context>
}

impl GinjaActor {
    pub(crate) async fn init(ginja_config: GinjaConfig) -> anyhow::Result<Context> {
        let mut actor = Akton::<GinjaActor>::create();

        for repo in &ginja_config.repositories {
            if let Some(repo_actor) = RepositoryActor::init(&repo).await {
                actor.state.repository_actors.push(repo_actor);
            }
        }

        let context = actor.activate(None).await?;

        Ok(context)
    }
}