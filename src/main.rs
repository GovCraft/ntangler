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
use tracing::{error, info, instrument, Level, trace};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_subscriber::fmt::format::FmtSpan;

use tangler_config::TanglerConfig;

use crate::actors::GitSentinel;
use crate::actors::Tangler;
use crate::messages::{LoadRepo, Poll};
use crate::repository_config::RepositoryConfig;

mod actors;
mod config_file;
mod tangler_config;
mod messages;
mod repository_config;
mod commits;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    println!("Welcome to Tangler! Now watching your repository and making commits to your local branch. Just code—commits are taken care of. Happy coding!");

    // Read and parse the configuration file
    let tangler_config: TanglerConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

    let (tangler, broker) = Tangler::init(tangler_config).await?;

    // Handle shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            eprintln!("Shutting down gracefully. Your code is safe! Please wait a moment.");
            tangler.suspend().await?;
            eprintln!("All done! Tangler has shut down safely. Happy coding!");
        }
        Err(err) => {
            eprintln!("Oops! Couldn't listen for the shutdown signal: {}. Don't worry, your code is safe! We're still wrapping things up... Please wait a moment.", err);
            tangler.suspend().await?; // Shut down in case of error
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use akton::prelude::ActorContext;

    use crate::actors::Tangler;
    use crate::init_tracing;
    use crate::repository_config::RepositoryConfig;
    use crate::tangler_config::TanglerConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_main() -> anyhow::Result<()> {
        init_tracing();

        // Read and parse the configuration file
        let tangler_config: TanglerConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

        let (tangler_actor, _broker) = Tangler::init(tangler_config).await?;

        tangler_actor.suspend().await?;
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
            .add_directive("akton_core::common::context=error".parse().unwrap())
            .add_directive("akton_core::common::context[emit_pool]=error".parse().unwrap())
            .add_directive("akton_core::traits=off".parse().unwrap())
            .add_directive("akton_core::pool::builder=error".parse().unwrap())
            .add_directive("akton_core::actors::awake=error".parse().unwrap())
            .add_directive("akton_core::common::akton=error".parse().unwrap())
            .add_directive("akton_core::common::pool_builder=error".parse().unwrap())
            .add_directive("akton_core::common::system=error".parse().unwrap())
            .add_directive("akton_core::common::supervisor=trace".parse().unwrap())
            .add_directive("akton_core::actors::actor=error".parse().unwrap())
            .add_directive("akton_core::actors::idle=error".parse().unwrap())
            .add_directive("akton_core::message::outbound_envelope=error".parse().unwrap())
            .add_directive("tangler::actors::repositories=info".parse().unwrap())
            .add_directive("tangler::actors::sentinels=info".parse().unwrap())
            .add_directive("tangler::actors::sentinels::tests=off".parse().unwrap())
            .add_directive("tangler::actors::tangler=info".parse().unwrap())
            .add_directive("tangler::actors::brokers=error".parse().unwrap())
            .add_directive("tangler::actors::brokers[load_subscriber_futures]=error".parse().unwrap())
            .add_directive("tangler::actors::brokers[load_subscriber_future_by_id]=error".parse().unwrap())
            .add_directive("tangler::actors::generators=error".parse().unwrap())
            .add_directive("tangler::tangler_config=error".parse().unwrap())
            .add_directive("tangler::repository_config=error".parse().unwrap())
            .add_directive("hyper_util=off".parse().unwrap())
            .add_directive("async_openai=off".parse().unwrap())
            .add_directive(Level::TRACE.into());
        // Set global log level to TRACE
        let subscriber = FmtSubscriber::builder()
            .with_span_events(FmtSpan::NONE)
            .with_max_level(Level::TRACE)
            .compact()
            .pretty()
            .with_line_number(true)
            .without_time()
            .with_env_filter(filter)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}
