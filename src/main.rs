#![allow(unused)]

use std::collections::HashMap;
use std::fs;
use std::sync::Once;

use akton::prelude::*;
use anyhow::{anyhow, Result};
use async_openai::{
    Client,
    types::{
        AssistantStreamEvent, CreateMessageRequest, CreateRunRequest,
        CreateThreadRequest, MessageDeltaContent,
        MessageRole,
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
use ginja_config::GinjaConfig;

use crate::actors::GinjaActor;
use crate::messages::LoadRepo;
use crate::repository_config::RepositoryConfig;
use crate::actors::{RepositoryWatcherActor};

mod config_file;
mod repository_config;
mod messages;
mod ginja_config;
mod actors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Read and parse the configuration file
    let ginja_config: GinjaConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

    let ginga_actor = GinjaActor::init(ginja_config).await?;

    // Handle shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            eprintln!("Shutting down");
            ginga_actor.terminate().await?;
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            eprintln!("Unable to listen for shutdown signal: {}", err);
            ginga_actor.terminate().await?; // Shut down in case of error
        }
    }

    Ok(())
}

/// Handles changes in a repository by creating a diff, sending it to an API to get a commit message,
/// and then committing and pushing the changes.
///
/// # Arguments
///
/// * `repo_config` - The configuration of the repository.
/// * `staged_only` - A flag indicating whether to consider only staged changes.
///
/// # Returns
///
/// A result indicating success or failure.
///
/// # Errors
///
/// This function can return errors if repository operations, API requests, or commit actions fail.
#[instrument]
async fn handle_changes(_repo_id: String, repo_path: String, staged_only: bool) -> Result<()> {
    let client = Client::new();
    // Open the repository
    let repo =
        Repository::open(&repo_path).map_err(|e| anyhow!("Failed to open repository: {}", e))?;

    // Get the diff of changes (staged or unstaged based on staged_only flag)
    let diff = if staged_only {
        repo.diff_index_to_workdir(
            Some(
                &repo
                    .index()
                    .map_err(|e| anyhow!("Failed to get index: {}", e))?,
            ),
            Some(&mut DiffOptions::new()),
        )
    } else {
        repo.diff_index_to_workdir(None, Some(&mut DiffOptions::new()))
    }
        .map_err(|e| anyhow!("Failed to get diff: {}", e))?;

    // Collect the diff as text
    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        diff_text.extend_from_slice(line.content());
        true
    })
        .map_err(|e| anyhow!("Failed to print diff: {}", e))?;

    let changes = String::from_utf8(diff_text)
        .map_err(|e| anyhow!("Failed to convert diff to string: {}", e))?;
    trace!("Diff: {changes}");

    // Send changes to the API to get the commit message
    let thread = client
        .threads()
        .create(CreateThreadRequest::default())
        .await?;

    let _message = client
        .threads()
        .messages(&thread.id)
        .create(CreateMessageRequest {
            role: MessageRole::User,
            content: CreateMessageRequestContent::from(changes),
            ..Default::default()
        })
        .await?;

    //
    // Step 3: Initiate a Run
    //
    let mut event_stream = client
        .threads()
        .runs(&thread.id)
        .create_stream(CreateRunRequest {
            assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
            stream: Some(true),
            ..Default::default()
        })
        .await?;

    let mut task_handle = None;
    let mut commit_message = String::new();
    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => match event {
                AssistantStreamEvent::ThreadRunRequiresAction(run_object) => {
                    println!("thread.run.requires_action: run_id:{}", run_object.id);
                    let _client = client.clone();
                    task_handle = Some(tokio::spawn(async move {
                        // handle_requires_action(client, run_object).await
                    }));
                }
                // _ => { println!("\nEvent: {event:?}\n");
                //     commit_message.push_str(event.into());
                // },
                AssistantStreamEvent::TreadCreated(_) => {}
                AssistantStreamEvent::ThreadRunCreated(_) => {}
                AssistantStreamEvent::ThreadRunQueued(_) => {}
                AssistantStreamEvent::ThreadRunInProgress(_) => {}
                AssistantStreamEvent::ThreadRunCompleted(_) => {}
                AssistantStreamEvent::ThreadRunIncomplete(_) => {}
                AssistantStreamEvent::ThreadRunFailed(_) => {}
                AssistantStreamEvent::ThreadRunCancelling(_) => {}
                AssistantStreamEvent::ThreadRunCancelled(_) => {}
                AssistantStreamEvent::ThreadRunExpired(_) => {}
                AssistantStreamEvent::ThreadRunStepCreated(_) => {}
                AssistantStreamEvent::ThreadRunStepInProgress(_) => {}
                AssistantStreamEvent::ThreadRunStepDelta(_) => {}
                AssistantStreamEvent::ThreadRunStepCompleted(_) => {}
                AssistantStreamEvent::ThreadRunStepFailed(_) => {}
                AssistantStreamEvent::ThreadRunStepCancelled(_) => {}
                AssistantStreamEvent::ThreadRunStepExpired(_) => {}
                AssistantStreamEvent::ThreadMessageCreated(_) => {}
                AssistantStreamEvent::ThreadMessageInProgress(_) => {}
                AssistantStreamEvent::ThreadMessageDelta(message) => {
                    if let Some(content) = message.delta.content {
                        for item in content {
                            match item {
                                MessageDeltaContent::ImageFile(_) => {}
                                MessageDeltaContent::ImageUrl(_) => {}
                                MessageDeltaContent::Text(text) => {
                                    if let Some(text) = text.text {
                                        if let Some(text) = text.value {
                                            commit_message.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                AssistantStreamEvent::ThreadMessageCompleted(_) => {}
                AssistantStreamEvent::ThreadMessageIncomplete(_) => {}
                AssistantStreamEvent::ErrorEvent(_) => {}
                AssistantStreamEvent::Done(_) => {}
                _ => {}
            },
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    // wait for task to handle required action and submit tool outputs
    if let Some(task_handle) = task_handle {
        let _ = tokio::join!(task_handle);
    }

    // Commit and push changes
    let _sig = repo
        .signature()
        .map_err(|e| anyhow!("Failed to get signature: {}", e))?;
    let tree_id = repo
        .index()
        .map_err(|e| anyhow!("Failed to get index: {}", e))?
        .write_tree()
        .map_err(|e| anyhow!("Failed to write tree: {}", e))?;
    let _tree = repo
        .find_tree(tree_id)
        .map_err(|e| anyhow!("Failed to find tree: {}", e))?;
    let head = repo
        .head()
        .map_err(|e| anyhow!("Failed to get HEAD: {}", e))?;
    let _parent_commit = head
        .peel_to_commit()
        .map_err(|e| anyhow!("Failed to get parent commit: {}", e))?;

    // repo.commit(
    //     Some("HEAD"),
    //     &sig,
    //     &sig,
    //     &commit_message.message,
    //     &tree,
    //     &[&parent_commit],
    // )
    //     .map_err(|e| anyhow!("Failed to commit: {}", e))?;

    trace!("Committed changes locally with message: {}", commit_message);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::actors::GinjaActor;
    use crate::ginja_config::GinjaConfig;
    use crate::init_tracing;
    use crate::repository_config::RepositoryConfig;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_main() -> anyhow::Result<()> {
        init_tracing();

        // Read and parse the configuration file
        let ginja_config: GinjaConfig = toml::from_str(&fs::read_to_string("./src/config.toml")?)?;

        let ginga_actor = GinjaActor::init(ginja_config).await?;

        ginga_actor.terminate().await?;
        Ok(())
    }


    #[test]
    fn test_finder() {
        let repository_config = RepositoryConfig {
            path: "./tmp".to_string(),
            ..Default::default()
        };
        let config_clone = GinjaConfig {
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
            .add_directive("tests=off".parse().unwrap())
            .add_directive("actor_tests=off".parse().unwrap())
            .add_directive("akton_core::traits=off".parse().unwrap())
            .add_directive("akton_core::common::awake=off".parse().unwrap())
            .add_directive("akton_core::common::akton=off".parse().unwrap())
            .add_directive("akton_core::common::pool_builder=off".parse().unwrap())
            .add_directive("akton_core::common::system=off".parse().unwrap())
            .add_directive("akton_core::common::supervisor=off".parse().unwrap())
            .add_directive("akton_core::common::actor=off".parse().unwrap())
            .add_directive("akton_core::common::idle=off".parse().unwrap())
            .add_directive("akton_core::common::outbound_envelope=off".parse().unwrap())
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
