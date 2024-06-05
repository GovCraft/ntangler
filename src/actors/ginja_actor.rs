use akton::prelude::*;
use crate::ginja_config::GinjaConfig;
use crate::repository_config::RepositoryConfig;

#[derive(Default, Debug)]
pub(crate) struct GinjaActor {}

impl GinjaActor {
    pub(crate) async fn init(ginja_config: GinjaConfig) -> anyhow::Result<Context> {
        let actor = Akton::<GinjaActor>::create();

        let context = actor.activate(None).await?;

        Ok(context)
    }
}