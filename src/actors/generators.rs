use std::any::TypeId;
use std::sync::Arc;
use std::thread;

use akton::prelude::*;
use akton::prelude::async_trait::async_trait;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{AssistantsApiResponseFormat, AssistantsApiResponseFormatOption, AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent, CreateRunRequest, CreateThreadRequest, MessageDeltaContent, MessageRole, ThreadObject};
use async_openai::types::AssistantsApiResponseFormatType::JsonObject;
use futures::StreamExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tracing::{debug, error, info, trace, warn};

use crate::commits::Commits;
use crate::messages::{AcceptParentBroker, BrokerSubscribe, ResponseCommit, SubmitDiff};

#[akton_actor]
pub(crate) struct OpenAi {
    client: Option<Arc<Client<OpenAIConfig>>>,
    broker: Context,
}

#[async_trait]
impl PooledActor for OpenAi {
    async fn initialize(&self, name: String, parent: &Context) -> Context {
        //TODO: expose broker through context
        let mut actor = Akton::<OpenAi>::create_with_id(&name);
        let client = Client::new();
        actor.state.client = Some(Arc::new(client));


        // Event: Setting up SubmitDiff Handler
        // Description: Setting up an actor to handle the `SubmitDiff` event asynchronously.
        // Context: None
        trace!("Setting up an actor to handle the `SubmitDiff` event asynchronously.");
        actor.setup
            .act_on_async::<AcceptParentBroker>(|actor, event| {
                actor.state.broker = event.message.broker.clone();
                let broker_context = actor.state.broker.clone();
                Box::pin(async move {
                    // broker_context.emit_async(subscription).await;
                })
            })
            .act_on_async::<SubmitDiff>(|actor, event| {
                let changes = event.message.diff.clone();
                let broker = actor.state.broker.clone();
                let path = event.message.path.clone();

                // Using Box::pin to handle the future.
                Box::pin(async move {
                    let (tx, mut rx) = mpsc::channel(32);
                    let broker = broker.clone();

                    task::spawn_blocking(move || {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            // Step 1: Create a new LLM thread via the API.
                            trace!("Step 1a: Create a new LLM thread via the API");
                            let client = Client::new();
                            trace!("Step 1b: Initiate conversation thread");
                            let thread = match client.threads().create(CreateThreadRequest::default()).await {
                                Ok(thread) => thread,
                                Err(e) => {
                                    // Event: Failed to Create Thread
                                    // Description: Failed to create a new LLM thread via the API.
                                    // Context: Error details.
                                    error!("Failed to create thread: {e}");
                                    return;
                                }
                            };
                            let thread_id = thread.id.clone();
                            trace!("Step 1c: Got thread id {}", thread_id);

                            // Step 2: Send changes as a new message in the thread.
                            trace!("Step 2: Send changes as a new message in the thread.");
                            if let Err(e) = client.threads().messages(&thread.id).create(CreateMessageRequest {
                                role: MessageRole::User,
                                content: CreateMessageRequestContent::from(changes),
                                ..Default::default()
                            }).await {
                                // Event: Failed to Create Message
                                // Description: Failed to send changes as a new message in the thread.
                                // Context: Error details.
                                error!("Failed to create message: {e}");
                                return;
                            }

                            let format = AssistantsApiResponseFormat { r#type: JsonObject };

                            // Step 3: Initiate a run and handle the event stream.
                            trace!("Step 3a: Initiate a run and handle the event stream.");
                            let mut event_stream = match client.threads().runs(&thread.id).create_stream(CreateRunRequest {
                                assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
                                stream: Some(true),
                                parallel_tool_calls: Some(true),
                                response_format: Some(AssistantsApiResponseFormatOption::Format(format)),
                                ..Default::default()
                            }).await {
                                Ok(stream) => {
                                    trace!("Run stream created");
                                    stream
                                }
                                Err(e) => {
                                    // Event: Failed to Create Run Stream
                                    // Description: Failed to initiate a run and handle the event stream.
                                    // Context: Error details.
                                    error!("Failed to create run stream: {e}");
                                    return;
                                }
                            };
                            let mut commit_message = String::new();
                            trace!("Step 3b: Processing events from the event stream.");

                            // Processing events from the event stream.
                            while let Some(event) = event_stream.next().await {
                                match event {
                                    Ok(event) => match event {
                                        AssistantStreamEvent::ThreadMessageDelta(message) => {
                                            if let Some(content) = message.delta.content {
                                                for item in content {
                                                    match item {
                                                        MessageDeltaContent::ImageFile(_) | MessageDeltaContent::ImageUrl(_) => {}
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
                                        AssistantStreamEvent::Done(_) => {}
                                        _ => {}
                                    },
                                    Err(e) => {
                                        // Event: Error in Event Stream
                                        // Description: An error occurred while processing the event stream.
                                        // Context: Error details.
                                        match e {
                                            OpenAIError::Reqwest(s) => {
                                                error!("Reqwest error: {s}");
                                            }
                                            OpenAIError::ApiError(_) => {}
                                            OpenAIError::JSONDeserialize(_) => {}
                                            OpenAIError::FileSaveError(_) => {}
                                            OpenAIError::FileReadError(_) => {}
                                            OpenAIError::StreamError(s) => {
                                                error!("Stream error: {s}");
                                            }
                                            OpenAIError::InvalidArgument(_) => {}
                                        }
                                    }
                                }
                            }

                            trace!("Step 4: Return commit msg: {}", &commit_message);
                            if let Err(e) = tx.send(commit_message).await {
                                // Event: Failed to Send Commit Message
                                // Description: Failed to send the commit message through the channel.
                                // Context: Error details.
                                error!("Failed to send commit msg: {e}");
                            }
                        });
                    });

                    // Await the result from the thread
                    if let Some(commit) = rx.recv().await {
                        // Event: Commit Message Received
                        // Description: A commit message has been received from the event stream.
                        // Context: Commit message details.
                        if !commit.is_empty() {
                            trace!(commit=?commit);
                            match serde_json::from_str(&*commit) {
                                Ok(commits) => {
                                    // let commits: Commits = serde_json::from_str(&*commit).expect("JSON was not well-formatted");
                                    broker.emit_async(ResponseCommit { commits, path }, None).await;
                                }
                                Err(e) => {
                                    error!(error=?e, "The json wasn't well formed");
                                }
                            };
                        } else {
                            error!("Commit message was empty. Check the logs.")
                        }
                    } else {
                        // Event: No Commit Message Received
                        // Description: No commit message was received from the event stream.
                        // Context: None
                        error!("No commit message received");
                    }
                })
            });


        let context = actor.activate(None).await.expect("Failed to activate OpenAi generator");
        // Event: Activating OpenAi generator
        // Description: Activating the OpenAi generator.
        // Context: None
        trace!(id=&context.key.value, "Activated OpenAi generator:");
        context
    }
}

#[cfg(test)]
mod unit_tests {
    use lazy_static::lazy_static;

    use crate::repository_config::RepositoryConfig;

    lazy_static! {
    static ref CONFIG: RepositoryConfig = RepositoryConfig {
        path: "./mock-repo-working".to_string(),
        branch_name: "new_branch".to_string(),
        api_url: "".to_string(),
        watch_staged_only: false,
        id: "any id".to_string(),
    };
        }

    lazy_static! {
    static ref DIFF: String = r#"diff --git a/test_file.txt b/test_file.txt
index 8430408..edc5728 100644
--- a/test_file.txt
+++ b/test_file.txt
@@ -1 +1,2 @@
Initial content
Modified content
"#.to_string();
}
}