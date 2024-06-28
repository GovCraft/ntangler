use std::{
    fmt::Debug,
    path::PathBuf,
    time::Duration
};

use akton::prelude::*;
use AssistantsApiResponseFormatOption::Format;
use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{AssistantEventStream, AssistantsApiResponseFormat, AssistantsApiResponseFormatOption, AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent, CreateRunRequest, CreateThreadRequest, MessageDeltaContent, MessageRole, ThreadObject},
    types::AssistantsApiResponseFormatType::JsonObject
};
use failsafe::{
    Config,
    futures::CircuitBreaker
};
use futures::StreamExt;
use tokio::{
    sync::mpsc,
    sync::mpsc::Sender,
    time::timeout
};
use tracing::{error, info, instrument, trace, warn};

use crate::messages::{CommitMessageGenerated, DiffQueued, GenerationStarted};

#[derive(Clone, Debug)]
pub(crate) struct OpenAi {
    client: Client<OpenAIConfig>,
}

impl Default for OpenAi {
    #[instrument]
    fn default() -> Self {
        info!("Initializing OpenAi actor with default configuration");
        let client = Client::new();
        Self {
            client,
        }
    }
}

#[instrument]
async fn create_run_stream_with_circuit_breaker(
    circuit_breaker: &(impl CircuitBreaker + Debug),
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    format: Option<AssistantsApiResponseFormatOption>,
) -> anyhow::Result<AssistantEventStream> {
    match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().runs(thread_id).create_stream(CreateRunRequest {
        assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
        stream: Some(true),
        parallel_tool_calls: Some(true),
        response_format: format,
        ..Default::default()
    }))).await {
        Ok(result) => match result {
            Ok(stream) => {
                info!("Run stream successfully created for thread_id: {}", thread_id);
                Ok(stream)
            }
            Err(e) => {
                error!("Timeout while creating run stream for thread_id: {}", thread_id);
                Err(anyhow::Error::from(e).into())
            }
        },
        Err(_) => {
            error!("Timeout while creating run stream");
            Err(anyhow::Error::msg("Timeout while creating run stream"))
        }
    }
}

#[instrument]
async fn create_message_with_circuit_breaker(
    circuit_breaker: &(impl CircuitBreaker + Debug),
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    diff: String,
) -> anyhow::Result<()> {
    match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().messages(thread_id).create(CreateMessageRequest {
        role: MessageRole::User,
        content: CreateMessageRequestContent::from(diff),
        ..Default::default()
    }))).await {
        Ok(result) => match result {
            Ok(_) => {
                info!("Message successfully created in thread_id: {}", thread_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create message in thread_id {}: {:?}", thread_id, e);
                Err(anyhow::Error::from(e).into())
            }
        },
        Err(e) => {
            error!("Circuit breaker call failed while creating message in thread_id {}: {:?}", thread_id, e);
            Err(anyhow::Error::from(e))
        }
    }
}

#[instrument]
async fn create_thread_with_circuit_breaker(circuit_breaker: &(impl CircuitBreaker + Debug), client: &Client<OpenAIConfig>) -> anyhow::Result<ThreadObject> {
    match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().create(CreateThreadRequest::default()))).await {
        Ok(result) => match result {
            Ok(thread) => {
                info!("Thread successfully created with id: {}", thread.id);
                Ok(thread)
            }
            Err(e) => {
                error!("Failed to create thread: {:?}", e);
                Err(anyhow::Error::from(e).into())
            }
        },
        Err(e) => {
            error!("Circuit breaker call failed while creating thread: {:?}", e);
            Err(anyhow::Error::from(e))
        }
    }
}

impl OpenAi {
    pub(crate) async fn initialize(
        config: ActorConfig,
        system: &mut AktonReady,
    ) -> anyhow::Result<Context> {
        info!("Initializing OpenAi actor with provided configuration");
        let mut actor = system.create_actor_with_config::<OpenAi>(config).await;
        trace!("Setting up SubmitDiff event handler for OpenAi actor");

        // Event: Setting up SubmitDiff Handler
        // Description: Setting up an actor to handle the `SubmitDiff` event asynchronously.
        // Context: None
        actor.setup.act_on_async::<DiffQueued>(|actor, event| {
            let reply_address = event.message.reply_address.clone();
            let broker = actor.akton.get_broker().clone();
            let message = event.message.clone();
            let client = actor.state.client.clone();
            info!("Received DiffQueued event: {:?}", event);

            Context::wrap_future(async move {
                Self::handle_diff_received(message, broker, reply_address, client).await;
            })
        });

        actor.context.subscribe::<DiffQueued>().await;
        let context = actor.activate(None).await;

        // Event: Activating OpenAi generator
        // Description: Activating the OpenAi generator.
        // Context: None
        info!(id = &context.key, "Activated OpenAi actor with id: {}", context.key);
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
        let target_file_display = &target_file.display().to_string();

        let client = client.clone();
        info!("Handling DiffQueued event for file: {}", target_file.display().to_string());
        tokio::spawn(Self::call_ai_endpoint(broker, tx, diff, repository_nickname, target_file_clone, client));

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
                        info!("Commit message generated and emitted for file: {}", target_file_display);
                    }
                    Err(e) => {
                        error!("Failed to deserialize commit message JSON for file {}: {:?}", target_file_display, e);
                    }
                };
            } else {
                error!("Commit message was empty for file: {}. Check the logs.", target_file_display);
            }
        } else {
            // Event: No Commit Message Received
            // Description: No commit message was received from the event stream.
            // Context: None
            error!("No commit message received for file: {}", target_file_display);
        }
    }

    #[instrument(skip(broker, tx, client))]
    async fn call_ai_endpoint(broker: Context, tx: Sender<String>, diff: String, repository_nickname: String, target_file_clone: PathBuf, client: Client<OpenAIConfig>) {
        let target_file_clone = target_file_clone.clone();
        let target_file_display = &target_file_clone.display().to_string();
        let msg = BrokerRequest::new(GenerationStarted::new(
            target_file_clone.clone(),
            repository_nickname.clone(),
        ));
        info!("AI endpoint called for repository: {}, file: {}", repository_nickname, target_file_clone.display());
        broker.emit_async(msg, None).await;

        let circuit_breaker = Config::new().build();

        let client = client.clone();
        let thread = match create_thread_with_circuit_breaker(&circuit_breaker, &client).await {
            Ok(thread) => thread,
            Err(e) => {
                // TODO: impl fallback logic
                error!("Error creating thread with circuit breaker for repository: {}, file: {}: {:?}", repository_nickname, target_file_display, e);
                return; // Fail gracefully by returning early
            }
        };

        let thread_id = thread.id.clone();
        trace!("Got thread id {} for repository: {}, file: {}", thread_id, repository_nickname, target_file_clone.display());
        match create_message_with_circuit_breaker(&circuit_breaker, &client, &thread.id, diff).await {
            Ok(message) => {
                trace!("Message created successfully in thread id {} for repository: {}, file: {}", thread_id, repository_nickname, target_file_display);
                message
            }
            Err(e) => {
                // TODO: impl fallback logic
                error!("Error creating message with circuit breaker for thread id {} in repository: {}, file: {}: {:?}", thread_id, repository_nickname, target_file_display, e);
                return; // Fail gracefully by returning early
            }
        };

        let format = AssistantsApiResponseFormat { r#type: JsonObject };

        // Step 3: Initiate a run and handle the event stream.
        let mut event_stream = match create_run_stream_with_circuit_breaker(&circuit_breaker, &client, &thread.id, Some(Format(format))).await {
            Ok(event_stream) => event_stream,
            Err(e) => {
                // TODO: impl fallback logic
                error!("Error creating run stream with circuit breaker for thread id {} in repository: {}, file: {}: {:?}", thread_id, repository_nickname, target_file_display, e);
                return; // Fail gracefully by returning early
            }
        };

        let mut commit_message = String::new();
        trace!("Processing events from the event stream for thread id {} in repository: {}, file: {}", thread_id, repository_nickname, target_file_display);

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
                    AssistantStreamEvent::Done(_) => {
                        trace!("Event stream completed for thread id {} in repository: {}, file: {}", thread_id, repository_nickname, target_file_display);
                    }
                    _ => {
                        warn!("Unhandled event type in the stream for thread id {} in repository: {}, file: {}", thread_id, repository_nickname, target_file_display);
                    }
                },
                Err(e) => {
                    // Event: Error in Event Stream
                    // Description: An error occurred while processing the event stream.
                    // Context: Error details.
                    match e {
                        OpenAIError::Reqwest(s) => {
                            error!("Reqwest error in event stream for thread id {} in repository: {}, file: {}: {}", thread_id, repository_nickname, target_file_display, s);
                        }
                        OpenAIError::ApiError(_) => {}
                        OpenAIError::JSONDeserialize(_) => {}
                        OpenAIError::FileSaveError(_) => {}
                        OpenAIError::FileReadError(_) => {}
                        OpenAIError::StreamError(s) => {
                            error!("Stream error in event stream for thread id {} in repository: {}, file: {}: {}", thread_id, repository_nickname, target_file_display, s);
                        }
                        OpenAIError::InvalidArgument(_) => {}
                    }
                }
            }
        }

        trace!("Returning commit message for repository: {}, file: {}", repository_nickname, target_file_display);
        if let Err(e) = tx.send(commit_message).await {
            error!("Failed to send commit message for repository: {}, file: {}: {}", repository_nickname, target_file_display, e);
        }
    }
}


