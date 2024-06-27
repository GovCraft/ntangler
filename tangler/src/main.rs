#![allow(unused)]

use std::path::PathBuf;
use std::sync::Once;
use std::{env, fs};

use akton::prelude::*;
use anyhow::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tracing::{error, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::actors::Tangler;
use crate::models::config::TanglerConfig;

mod actors;

mod messages;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // TODO: this needs to move to an actor that reloads the config dynamically
    // Determine configuration file path according to XDG spec
    let config_path = get_config_file_path("tangler", "config.toml")?;

    // Read and parse the configuration file
    let config_content = fs::read_to_string(&config_path);
    if config_content.is_err() {
        error!("Configuration file not found at {:?}. Please create a configuration file at this location with the necessary settings.", config_path);
        return Err("Configuration file not found".into());
    }

    let tangler_config: TanglerConfig = toml::from_str(&config_content.unwrap())?;

    let (tangler, broker) = Tangler::init(tangler_config).await?;

    // Handle shutdown signal
    // need to move this to an actor
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("Shutting down gracefully. Your code is safe! Please wait a moment.");
            tangler.suspend_actor().await?;
            println!("All done! Tangler has shut down safely. Happy coding!");
        }
        Err(err) => {
            eprintln!("Oops! Couldn't catch the shutdown signal: {}. Don't worry, your code is safe! Wrapping things up... Please wait a moment.", err);
            tangler.suspend_actor().await?; // Shut down in case of error
        }
    }

    Ok(())
}

// Function to get the configuration file path following the XDG Base Directory Specification
fn get_config_file_path(
    app_name: &str,
    config_file: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(config_home).join(app_name).join(config_file))
    } else if let Ok(home) = env::var("HOME") {
        Ok(PathBuf::from(home)
            .join(".config")
            .join(app_name)
            .join(config_file))
    } else {
        Err("Could not determine configuration file path".into())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use akton::prelude::ActorContext;

    use crate::actors::Tangler;
    use crate::init_tracing;
    use crate::models::config::RepositoryConfig;
    use crate::models::config::TanglerConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_main() -> anyhow::Result<()> {
        init_tracing();

        // Read and parse the configuration file
        let tangler_config: TanglerConfig = toml::from_str(&fs::read_to_string("/config.toml")?)?;

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
            .add_directive("akton_core::common::context=error".parse().unwrap())
            .add_directive(
                "akton_core::common::context[emit_pool]=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("akton_core::traits=off".parse().unwrap())
            .add_directive("akton_core::traits::actor_context=off".parse().unwrap())
            .add_directive("akton_core::pool::builder=error".parse().unwrap())
            .add_directive("akton_core::actors::awake=error".parse().unwrap())
            .add_directive("akton_core::common::akton=error".parse().unwrap())
            .add_directive("akton_core::common::pool_builder=error".parse().unwrap())
            .add_directive("akton_core::common::system=error".parse().unwrap())
            .add_directive("akton_core::common::supervisor=error".parse().unwrap())
            .add_directive("akton_core::common::broker=error".parse().unwrap())
            .add_directive(
                "akton_core::common::broker[broadcast]=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("akton_core::message=error".parse().unwrap())
            .add_directive(
                "akton_core::message::outbound_envelope=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("akton_core::actors::actor=error".parse().unwrap())
            .add_directive("akton_core::actors::idle=error".parse().unwrap())
            .add_directive(
                "akton_core::message::outbound_envelope=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("tangler::actors::repositories=error".parse().unwrap())
            .add_directive(
                "tangler::actors::repositories[broadcast_futures]=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive(
                "tangler::actors::repositories[default_behavior]=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("tangler::actors::scribe=info".parse().unwrap())
            .add_directive(
                "tangler::actors::scribe[print_hero_message]=error"
                    .parse()
                    .unwrap(),
            )
            .add_directive("tangler::actors::tangler=error".parse().unwrap())
            .add_directive("tangler::models=error".parse().unwrap())
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
