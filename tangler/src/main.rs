#![allow(unused)]

use std::path::PathBuf;
use std::sync::Once;
use std::{env, fs};

use akton::prelude::*;
use anyhow::Result;
use console::Term;
use futures::StreamExt;
use indicatif::TermLike;
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
    setup_tracing();

    if check_openai_api_key() {
        Term::stderr().write_line("API Key Detected: The OPENAI_API_KEY environment variable is set.")?;
    } else {
        Term::stderr().write_line("Startup Error: The OPENAI_API_KEY environment variable is not set. Please set it to proceed. Consult the documentation to set the API key.")?;
        std::process::exit(1);
    }

    let config_path = find_config_file_path("tangler", "config.toml")?;
    let config_content = fs::read_to_string(&config_path)?;

    let tangler_config: TanglerConfig = toml::from_str(&config_content)?;
    Term::stderr().write_line(&format!("Configuration Loaded: Config found at {}. Initializing...", config_path.display()))?;

    let (tangler, broker) = Tangler::initialize(tangler_config).await?;

    match signal::ctrl_c().await {
        Ok(()) => {
            Term::stderr().write_line("Shutting down gracefully. Please wait...")?;
            tangler.suspend_actor().await?;
            Term::stdout().show_cursor()?;
            Term::stdout().write_line("Shutdown complete. All operations halted safely.");
        }
        Err(err) => {
            Term::stderr().write_line(&format!("Error capturing shutdown signal: {}. Terminating safely...", err))?;
            tangler.suspend_actor().await?;
            Term::stdout().show_cursor()?;
        }
    }

    Ok(())
}

fn check_openai_api_key() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
}

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::FULL)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn find_config_file_path(app_name: &str, config_file: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(config_home).join(app_name).join(config_file))
    } else if let Ok(home_dir) = env::var("HOME") {
        Ok(PathBuf::from(home_dir).join(".config").join(app_name).join(config_file))
    } else {
        Err("Could not determine the configuration file path.".into())
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

        let (tangler_actor, _broker) = Tangler::initialize(tangler_config).await?;

        tangler_actor.suspend().await?;
        Ok(())
    }

    #[test]
    fn test_finder() {
        let repository_config = RepositoryConfig {
            path: "./tmp".to_string().parse().unwrap(),
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
            .find(|repo| event_path.starts_with(&repo.path.display().to_string()));
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
