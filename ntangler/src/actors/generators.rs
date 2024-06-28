use std::fmt::Debug;
use std::time::Duration;

use akton::prelude::*;
use async_openai::{Client};
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{AssistantsApiResponseFormat, AssistantsApiResponseFormatOption, AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent, CreateRunRequest, CreateThreadRequest, MessageDeltaContent, MessageRole, ThreadObject};
use async_openai::types::AssistantsApiResponseFormatType::JsonObject;
use failsafe::{Config, StateMachine};
use failsafe::futures::CircuitBreaker;
use futures::StreamExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::timeout;
use tracing::{error, instrument, trace};

use crate::messages::{CommitMessageGenerated, DiffQueued, GenerationStarted};

#[derive(Clone, Debug)]
pub(crate) struct OpenAi {
    client: Client<OpenAIConfig>,
}

impl Default for OpenAi {
    #[instrument]
    fn default() -> Self {
        tracing::trace!("Default called for OpenAi actor");
        let client = Client::new();
        Self {
            client,
        }
    }
}

#[instrument]
async fn create_message_with_circuit_breaker(
    circuit_breaker: &(impl CircuitBreaker + Debug),
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    diff: String
) -> anyhow::Result<()> {
    match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().messages(thread_id).create(CreateMessageRequest {
        role: MessageRole::User,
        content: CreateMessageRequestContent::from(diff),
        ..Default::default()
    }))).await {
        Ok(result) => match result {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to create message: {:?}", e);
                Err(anyhow::Error::from(e).into())
            }
        },
        Err(e) => {
            error!("Circuit breaker call failed: {:?}", e);
            Err(anyhow::Error::from(e))
        }
    }
}
#[instrument]
async fn create_thread_with_circuit_breaker(circuit_breaker: &(impl CircuitBreaker + Debug), client: &Client<OpenAIConfig>) -> anyhow::Result<ThreadObject> {
    match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().create(CreateThreadRequest::default()))).await {
        Ok(result) => match result {
            Ok(thread) => Ok(thread),
            Err(e) => {
                error!("Failed to create thread: {:?}", e);
                Err(anyhow::Error::from(e).into())
            }
        },
        Err(e) => {
            error!("Circuit breaker call failed: {:?}", e);
            Err(anyhow::Error::from(e))
        }
    }
}
impl OpenAi {
    pub(crate) async fn initialize(
        config: ActorConfig,
        system: &mut AktonReady,
    ) -> anyhow::Result<Context> {
        let mut actor = system.create_actor_with_config::<OpenAi>(config).await;
        // Event: Setting up SubmitDiff Handler
        // Description: Setting up an actor to handle the `SubmitDiff` event asynchronously.
        // Context: None
        trace!("Setting up an actor to handle the `SubmitDiff` event asynchronously.");
        actor.setup.act_on_async::<DiffQueued>(|actor, event| {
            let reply_address = event.message.reply_address.clone();
            let broker = actor.akton.get_broker().clone();
            let message = event.message.clone();
            let client = actor.state.client.clone();
            tracing::info!("Received DiffQueued event: {:?}", event);
            Context::wrap_future(async move {
                Self::handle_diff_received(message, broker, reply_address, client).await;
            })
        });

        actor.context.subscribe::<DiffQueued>().await;
        let context = actor.activate(None).await;

        // Event: Activating OpenAi generator
        // Description: Activating the OpenAi generator.
        // Context: None
        trace!(id = &context.key, "Activated OpenAi generator:");
        Ok(context)
    }
    #[instrument(skip(broker, return_address, client))]
    async fn handle_diff_received(message: DiffQueued, broker: Context, return_address: Context, client: Client<OpenAIConfig>) {
        let (tx, mut rx) = mpsc::channel(32);
        let return_address = return_address.clone();
        let diff = message.diff.clone();
        let target_file = message.target_file.clone();
        let repository_nickname = message.repository_nickname.clone();
        let target_file_clone = target_file.clone();

        task::spawn_blocking(move || {
            let client = client.clone();
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let target_file_clone = target_file_clone.clone();
                let msg = BrokerRequest::new(GenerationStarted::new(
                    target_file_clone.clone(),
                    repository_nickname.clone(),
                ));
                broker.emit_async(msg, None).await;

                let circuit_breaker = Config::new().build();

                let client = client.clone();
                let thread = match create_thread_with_circuit_breaker(&circuit_breaker, &client).await {
                    Ok(thread) => thread,
                    Err(e) => {
                        error!("Error creating thread with circuit breaker: {:?}", e);
                        return; // Fail gracefully by returning early
                    }
                };

                let thread_id = thread.id.clone();
                trace!("Step 1c: Got thread id {}", thread_id);

                // Set timeout for sending changes as a new message in the thread
                if let Err(e) = timeout(Duration::from_secs(10), client.threads().messages(&thread.id).create(CreateMessageRequest {
                    role: MessageRole::User,
                    content: CreateMessageRequestContent::from(diff),
                    ..Default::default()
                })).await {
                    // Event: Failed to Create Message
                    // Description: Failed to send changes as a new message in the thread.
                    // Context: Error details.
                    error!("Failed to create message: {e}");
                    return;
                }

                let format = AssistantsApiResponseFormat { r#type: JsonObject };

                // Step 3: Initiate a run and handle the event stream.

                // Set timeout for initiating a run and handling the event stream
                let mut event_stream = match timeout(Duration::from_secs(10), client.threads().runs(&thread.id).create_stream(CreateRunRequest {
                    assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
                    stream: Some(true),
                    parallel_tool_calls: Some(true),
                    response_format: Some(AssistantsApiResponseFormatOption::Format(format)),
                    ..Default::default()
                })).await {
                    Ok(Ok(stream)) => {
                        trace!("Run stream created");
                        stream
                    }
                    Ok(Err(e)) => {
                        // Event: Failed to Create Run Stream
                        // Description: Failed to initiate a run and handle the event stream.
                        // Context: Error details.
                        error!("Failed to create run stream: {e}");
                        return;
                    }
                    Err(_) => {
                        // Event: Timeout while Creating Run Stream
                        // Description: Timeout occurred while creating a run stream.
                        // Context: None
                        error!("Timeout while creating run stream");
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
                                            MessageDeltaContent::ImageFile(_)
                                            | MessageDeltaContent::ImageUrl(_) => {}
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
        if let Some(commit_message) = rx.recv().await {
            // Event: Commit Message Received
            // Description: A commit message has been received from the event stream.
            // Context: Commit message details.
            if !commit_message.is_empty() {
                match serde_json::from_str(&commit_message) {
                    Ok(commit) => {
                        let message = CommitMessageGenerated::new(target_file, commit);
                        return_address.emit_async(message, None).await;
                        trace!("Emitted commit message to broker");
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
    }
}


