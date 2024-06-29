use std::{env, fs};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Once;

use akton::prelude::*;
use anyhow::Result;
use console::Term;
use serde::Deserialize;
use tokio::signal;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::actors::Ntangler;
use crate::models::config::NtanglerConfig;

mod actors;
mod messages;
mod models;

#[derive(Debug, Deserialize)]
struct LogConfig {
    log_directives: Vec<String>,
}

fn read_log_config(config_path: &PathBuf) -> LogConfig {
    let config_content =
        fs::read_to_string(config_path).expect("Unable to read log configuration file");
    toml::from_str(&config_content).expect("Invalid configuration format")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing("ntangler", "config.toml");

    if check_openai_api_key() {
        Term::stderr()
            .write_line("API Key Detected: The OPENAI_API_KEY environment variable is set.")?;
    } else {
        Term::stderr().write_line("Startup Error: The OPENAI_API_KEY environment variable is not set. Please set it to proceed. Consult the documentation to set the API key.")?;
        std::process::exit(1);
    }

    let config_path = find_config_path("ntangler", "config.toml")?;
    let config_content = fs::read_to_string(&config_path)?;

    let ntangler_config: NtanglerConfig = toml::from_str(&config_content)?;
    Term::stderr().write_line(&format!(
        "Configuration Loaded: Config found at {}. Initializing...",
        config_path.display()
    ))?;

    let (ntangler, _broker) = Ntangler::initialize(ntangler_config).await?;

    match signal::ctrl_c().await {
        Ok(()) => {
            Term::stderr().write_line("Shutting down gracefully. Please wait...")?;
            ntangler.suspend_actor().await?;
            Term::stdout().show_cursor()?;
            Term::stdout().write_line("Shutdown complete. All operations halted safely.")?;
        }
        Err(err) => {
            Term::stderr().write_line(&format!(
                "Error capturing shutdown signal: {}. Terminating safely...",
                err
            ))?;
            ntangler.suspend_actor().await?;
            Term::stdout().show_cursor()?;
        }
    }

    Ok(())
}

fn check_openai_api_key() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
}

fn find_config_path(
    app_name: &str,
    config_file: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(config_home).join(app_name).join(config_file))
    } else if let Ok(home_dir) = env::var("HOME") {
        Ok(PathBuf::from(home_dir)
            .join(".config")
            .join(app_name)
            .join(config_file))
    } else {
        Err("Could not determine the configuration file path.".into())
    }
}

fn find_logs_path(app_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_home) = env::var("XDG_CACHE_HOME") {
        Ok(PathBuf::from(config_home).join(app_name).join("logs"))
    } else if let Ok(home_dir) = env::var("HOME") {
        Ok(PathBuf::from(home_dir)
            .join(".cache")
            .join(app_name)
            .join("logs"))
    } else {
        Err("Could not determine the logs directory path.".into())
    }
}

fn create_log_path(config_path: &Path) -> PathBuf {
    let file_name = config_path.file_name().unwrap();
    let mut new_file_name = OsString::from("log_");
    new_file_name.push(file_name);

    let mut new_path = config_path.to_path_buf();
    new_path.set_file_name(new_file_name);
    new_path
}

static INIT: Once = Once::new();

pub fn setup_tracing(app_name: &str, config_file: &str) {
    INIT.call_once(|| {
        // Get the directory for logging using the logs path function
        let log_dir = find_logs_path(app_name).expect("Unable to find logs directory path");
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "app.log");

        let config_path =
            find_config_path(app_name, config_file).expect("Unable to find config file path");

        let log_config_path = create_log_path(&config_path);
        // Read initial log configuration directives
        let log_config = read_log_config(&log_config_path);

        // Closure to create the filter from the log configuration
        let create_filter = |log_config: &LogConfig| {
            let mut filter = EnvFilter::new("");
            for directive in &log_config.log_directives {
                filter = filter.add_directive(directive.parse().unwrap());
            }
            filter.add_directive(Level::TRACE.into())
        };

        let filter = create_filter(&log_config);

        // Set global log level to TRACE and direct logs to the file appender
        let subscriber = FmtSubscriber::builder()
            .with_span_events(FmtSpan::NONE)
            .with_max_level(Level::TRACE)
            .compact()
            .with_line_number(true)
            .with_env_filter(filter)
            .with_writer(file_appender) // Set the writer to the file appender
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

#[cfg(test)]
mod tests {
    use std::fs;

    use akton::prelude::ActorContext;

    use crate::actors::Ntangler;
    use crate::models::config::RepositoryConfig;
    use crate::models::config::NtanglerConfig;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_main() -> anyhow::Result<()> {
        setup_tracing("ntanger_test", "config.toml");

        // Read and parse the configuration file
        let ntangler_config: NtanglerConfig = toml::from_str(&fs::read_to_string("/config.toml")?)?;

        let (ntangler_actor, _broker) = Ntangler::initialize(ntangler_config).await?;

        ntangler_actor.suspend_actor().await?;
        Ok(())
    }

    #[test]
    fn test_finder() {
        let repository_config = RepositoryConfig {
            path: "./tmp".to_string().parse().unwrap(),
            ..Default::default()
        };
        let config_clone = NtanglerConfig {
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
