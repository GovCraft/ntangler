#![allow(unused)]

use std::collections::HashMap;
use std::fs;
use std::sync::Once;

use akton::prelude::*;
use anyhow::{anyhow, Result};
use async_openai::{
    Client,
    types::{
        AssistantStreamEvent, CreateMessageRequest, CreateRunRequest, CreateThreadRequest,
        MessageDeltaContent, MessageRole,
    },
};
use async_openai::types::CreateMessageRequestContent;
use futures::StreamExt;
use git2::{DiffOptions, Repository};
use notify::PollWatcher;
use notify_debouncer_mini::Debouncer;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tracing::{error, instrument, Level, trace};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_subscriber::fmt::format::FmtSpan;

use tangler_config::TanglerConfig;

use crate::actors::RepositoryWatcherActor;
use crate::actors::TanglerActor;
use crate::messages::LoadRepo;
use crate::repository_config::RepositoryConfig;

mod actors;
mod config_file;
mod tangler_config;
mod messages;
mod repository_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Read and parse the configuration file
    let tangler_config: TanglerConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

    let (tangler, broker) = TanglerActor::init(tangler_config).await?;

    // Handle shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            eprintln!("Shutting down");
            tangler.terminate().await?;
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            eprintln!("Unable to listen for shutdown signal: {}", err);
            tangler.terminate().await?; // Shut down in case of error
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::actors::TanglerActor;
    use crate::init_tracing;
    use crate::repository_config::RepositoryConfig;
    use crate::tangler_config::TanglerConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_main() -> anyhow::Result<()> {
        init_tracing();

        // Read and parse the configuration file
        let tangler_config: TanglerConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

        let (tangler_actor, _broker) = TanglerActor::init(tangler_config).await?;

        tangler_actor.terminate().await?;
        Ok(())
    }

    #[test]
    fn test_finder() {
        let repository_config = RepositoryConfig {
            path: "./tmp".to_string(),
            ..Default::default()
        };
        let config_clone = TanglerConfig {
            repositories: vec![repository_config],
        };
        let event_path = "./tmp/tmp.txt";
        // let config_clone = Some(repository_config);
        let repository = config_clone
            .repositories
            .iter()
            .find(|repo| event_path.starts_with(&repo.path));
        assert!(repository.is_some());
        let repository = repository.unwrap();
        println!("{:?}", repository);
    }
}

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        // Define an environment filter to suppress logs from specific functions
        let filter = EnvFilter::new("")
            .add_directive(
                "akton_core::common::context::peek_state_span=off"
                    .parse()
                    .unwrap(),
            )
            .add_directive("akton_core::common::context=off".parse().unwrap())
            .add_directive("akton_core::traits=off".parse().unwrap())
            .add_directive("akton_core::common::awake=off".parse().unwrap())
            .add_directive("akton_core::common::akton=off".parse().unwrap())
            .add_directive("akton_core::common::pool_builder=off".parse().unwrap())
            .add_directive("akton_core::common::system=off".parse().unwrap())
            .add_directive("akton_core::common::supervisor=off".parse().unwrap())
            .add_directive("akton_core::common::actor=trace".parse().unwrap())
            .add_directive("akton_core::common::idle=off".parse().unwrap())
            .add_directive("akton_core::common::outbound_envelope=off".parse().unwrap())
            .add_directive("tangler::actors::repository_actor=off".parse().unwrap())
            .add_directive("tangler::actors::tangler_actor=debug".parse().unwrap())
            .add_directive("tangler::actors::tangler_actor::tests=off".parse().unwrap())
            .add_directive("tangler::actors::broker_actor=debug".parse().unwrap())
            .add_directive("tangler::actors::ai_actor=off".parse().unwrap())
            .add_directive("hyper_util=off".parse().unwrap())
            .add_directive(Level::TRACE.into());

        // Set global log level to TRACE
        let subscriber = FmtSubscriber::builder()
            .with_span_events(FmtSpan::NONE)
            .with_max_level(Level::TRACE)
            .compact()
            .with_line_number(true)
            .without_time()
            .with_env_filter(filter)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}
