use std::env;
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;
use akton::prelude::*;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{AssistantEventStream, AssistantsApiResponseFormat, AssistantsApiResponseFormatOption, AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent, CreateRunRequest, CreateThreadRequest, MessageDeltaContent, MessageRole, ThreadObject};
use async_openai::types::AssistantsApiResponseFormatOption::Format;
use async_openai::types::AssistantsApiResponseFormatType::JsonObject;
use failsafe::Config;
use failsafe::futures::CircuitBreaker;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::messages::{CommitMessageGenerated, DiffQueued, GenerationStarted};
use crate::models::CommitMessage;

#[derive(Clone, Debug)]
pub struct LlmClient {
    client: Client,
    endpoint: String,
    api_key: Option<String>,
}

impl Default for LlmClient {
    fn default() -> Self {
        LlmClient {
            client: Client::new(),
            endpoint: String::default(),
            api_key: None,
        }
    }
}

impl LlmClient {
    #[instrument(skip(system, config))]
    pub async fn initialize(config: ActorConfig, system: &mut AktonReady) -> anyhow::Result<Context> {
        let mut actor = system.create_actor_with_config::<LlmClient>(config).await;

        // Initialize with default values, these will be set properly later
        actor.state.client = Client::new();
        // Read endpoint from environment variable
        actor.state.endpoint = env::var("NTANGLER_ENDPOINT")
            .unwrap_or_else(|_| {
                warn!("NTANGLER_ENDPOINT not set, using default endpoint");
                "https://api.openai.com/v1".to_string() // Default OpenAI endpoint
            });

        // Read API key from environment variable
        actor.state.api_key = env::var("NTANGLER_API_TOKEN").ok();

        if actor.state.api_key.is_none() {
            warn!("NTANGLER_API_TOKEN not set, API calls may fail");
        }

        actor.setup.act_on_async::<DiffQueued>(|actor, event| {
            let return_address = event.message.reply_address.clone();
            let message = event.message.clone();
            let client = actor.state.client.clone();
            let api_key = actor.state.api_key.clone();
            let endpoint = actor.state.endpoint.clone();
            info!("Received DiffQueued event: {:?}", event);
            Context::wrap_future(Self::handle_generate_commit_message(message, return_address, client, api_key, endpoint) )
        });

        actor.context.subscribe::<DiffQueued>().await;

        Ok(actor.activate(None).await)
    }

    #[instrument(skip(message, return_address, client, api_key))]
    async fn handle_generate_commit_message(message: DiffQueued, return_address: Context, client: Client, api_key: Option<String>, endpoint: String) {
        let return_address = return_address.clone();
        match Self::generate_commit_message(client, endpoint, api_key, message.diff).await {
            Ok(commit_message) => {
                return_address.emit_async(
                    CommitMessageGenerated { target_file: message.target_file.clone(), commit_message },
                    None,
                ).await
            }
            Err(e) => {
                error!("{e}");
            }
        }
    }

    pub fn configure(&mut self, endpoint: String, api_key: Option<String>) {
        self.endpoint = endpoint;
        self.api_key = api_key;
    }

    #[instrument(skip(client))]
    async fn generate_commit_message(client: Client, endpoint: String, api_key: Option<String>, diff: String) -> anyhow::Result<CommitMessage> {
        let request = CommitRequest { diff };
        let mut req_builder = client.post(&endpoint)
            .header("Content-Type", "application/json");

        if let Some(key) = &api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = req_builder
            .json(&request)
            .send()
            .await?;

        // Log response details
        debug!("Response status: {}", response.status());
        debug!("Response headers:");
        for (name, value) in response.headers().iter() {
            debug!("  {}: {}", name, value.to_str().unwrap_or("<non-string value>"));
        }

        // Handle streaming response
        let mut full_body = String::new();
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            full_body.push_str(&String::from_utf8_lossy(&chunk));
            debug!("Received chunk: {}", String::from_utf8_lossy(&chunk));
        }

        debug!("Full response body: {}", full_body);

        // Parse the full response body
        let commit_message: CommitMessage = serde_json::from_str(&full_body)?;

        Ok(commit_message)
    }
    #[instrument]
    async fn create_run_stream_with_circuit_breaker(
        circuit_breaker: &(impl CircuitBreaker + Debug),
        client: &async_openai::Client<OpenAIConfig>,
        conversation_thread_id: &str,
        format: Option<AssistantsApiResponseFormatOption>,
    ) -> anyhow::Result<AssistantEventStream> {
        match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().runs(conversation_thread_id).create_stream(CreateRunRequest {
            assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
            stream: Some(true),
            parallel_tool_calls: Some(true),
            response_format: format,
            ..Default::default()
        }))).await {
            Ok(result) => match result {
                Ok(stream) => {
                    info!("Run stream successfully created for conversation_thread_id: {}", conversation_thread_id);
                    Ok(stream)
                }
                Err(e) => {
                    error!("Timeout while creating run stream for conversation_thread_id: {}", conversation_thread_id);
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
        client: &async_openai::Client<OpenAIConfig>,
        conversation_thread_id: &str,
        diff: String,
    ) -> anyhow::Result<()> {
        match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().messages(conversation_thread_id).create(CreateMessageRequest {
            role: MessageRole::User,
            content: CreateMessageRequestContent::from(diff),
            ..Default::default()
        }))).await {
            Ok(result) => match result {
                Ok(_) => {
                    info!("Message successfully created in conversation_thread_id: {}", conversation_thread_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to create message in conversation_thread_id {}: {:?}", conversation_thread_id, e);
                    Err(anyhow::Error::from(e).into())
                }
            },
            Err(e) => {
                error!("Circuit breaker call failed while creating message in thread_id {}: {:?}", conversation_thread_id, e);
                Err(anyhow::Error::from(e))
            }
        }
    }

    #[instrument]
    async fn create_thread_with_circuit_breaker(circuit_breaker: &(impl CircuitBreaker + Debug), client: &async_openai::Client<OpenAIConfig>) -> anyhow::Result<ThreadObject> {
        match circuit_breaker.call(timeout(Duration::from_secs(10), client.threads().create(CreateThreadRequest::default()))).await {
            Ok(result) => match result {
                Ok(conversation_thread) => {
                    info!("Thread successfully created with id: {}", conversation_thread.id);
                    Ok(conversation_thread)
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
    #[instrument(skip(broker, tx, client))]
    async fn call_ai_endpoint(broker: Context, tx: Sender<String>, diff: String, repository_nickname: String, target_file_clone: PathBuf, client: async_openai::Client<OpenAIConfig>) {
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
        let thread = match Self::create_thread_with_circuit_breaker(&circuit_breaker, &client).await {
            Ok(thread) => thread,
            Err(e) => {
                // TODO: impl fallback logic
                error!("Error creating thread with circuit breaker for repository: {}, file: {}: {:?}", repository_nickname, target_file_display, e);
                return; // Fail gracefully by returning early
            }
        };

        let thread_id = thread.id.clone();
        trace!("Got thread id {} for repository: {}, file: {}", thread_id, repository_nickname, target_file_clone.display());
        match Self::create_message_with_circuit_breaker(&circuit_breaker, &client, &thread.id, diff).await {
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
        let mut event_stream = match Self::create_run_stream_with_circuit_breaker(&circuit_breaker, &client, &thread.id, Some(Format(format))).await {
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

#[derive(Deserialize)]
struct CommitResponse {
    commit_message: CommitMessage,
}

#[derive(Deserialize,Serialize)]
struct CommitRequest {
    diff: String,
}