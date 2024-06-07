use std::sync::Arc;
use std::thread;

use akton::prelude::*;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent, CreateRunRequest, CreateThreadRequest, MessageDeltaContent, MessageRole, ThreadObject};
use futures::StreamExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tracing::{error, info, trace, warn};

use crate::messages::{ResponseCommit, SubmitDiff};

#[akton_actor]
pub(crate) struct AiActor {
    client: Option<Arc<Client<OpenAIConfig>>>,
}

impl AiActor {
    pub(crate) async fn init() -> anyhow::Result<Context> {
        let mut actor = Akton::<AiActor>::create();
        let client = Client::new();
        actor.state.client = Some(Arc::new(client));

        // Setting up an actor to handle the `SubmitDiff` event asynchronously.
        actor.setup.act_on_async::<SubmitDiff>(|actor, event| {
            let changes = event.message.diff.clone();
            let return_address = event.return_address.clone();
            // Using Box::pin to handle the future.
            Box::pin(async move {
                let (tx, mut rx) = mpsc::channel(32);
                let return_address  = return_address.clone();
                task::spawn_blocking(move || {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        // Step 1: Create a new LLM thread via the API.
                        trace!("Step 1a: Create a new LLM thread via the API");
                        let client = Client::new();
                        trace!("Step 1b: Initiate conversation thread");
                        let thread = client
                            .threads()
                            .create(CreateThreadRequest::default())
                            .await
                            .expect("Failed to create thread");

                        let thread_id = thread.id.clone();
                        trace!("Step 1c: Got thread id {}", thread_id);

                        trace!("Step 2: Send changes as a new message in the thread.");
                        // Step 2: Send changes as a new message in the thread.
                        let _message = client
                            .threads()
                            .messages(&thread.id)
                            .create(CreateMessageRequest {
                                role: MessageRole::User,
                                content: CreateMessageRequestContent::from(changes),
                                ..Default::default()
                            })
                            .await
                            .expect("Failed to create message");

                        trace!("Step 3a: Initiate a run and handle the event stream.");
                        // Step 3: Initiate a run and handle the event stream.
                        let mut event_stream = client
                            .threads()
                            .runs(&thread.id)
                            .create_stream(CreateRunRequest {
                                assistant_id: "asst_xiaBOCpksCenAMJSL2F0qqFL".to_string(),
                                stream: Some(true),
                                ..Default::default()
                            })
                            .await
                            .expect("Failed to create run stream");

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
                                    eprintln!("Error: {e}");
                                }
                            }
                        }
                        trace!("Step 4: Return commit msg: {}", &commit_message);
                        tx.send(commit_message).await.expect("Failed to send commit msg");
                    });
                });

                // Await the result from the thread
                let result = rx.recv().await;
                if let Some(commit) = result {
                    info!("Commit message: {}", commit);
                    let return_address = return_address;
                    let commit = commit.clone();
                    return_address.reply_async(ResponseCommit { commit }, None).await;
                } else {
                    error!("No commit message received");
                }
            })
        });

        let context = actor.activate(None).await.expect("Failed to activate AiActor");
        Ok(context)
    }
}

#[cfg(test)]
mod unit_tests {
    use akton::prelude::{ActorContext, Akton};
    use lazy_static::lazy_static;

    use crate::actors::ai_actor::AiActor;
    use crate::actors::RepositoryWatcherActor;
    use crate::init_tracing;
    use crate::messages::SubmitDiff;
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "Makes live call"]
    async fn test_commit_msg_retrieval() -> anyhow::Result<()> {
        init_tracing();

        let diff = DIFF.clone();
        let config = CONFIG.clone();
        let id = config.id.clone();

        let watcher = RepositoryWatcherActor::init(&config);
        let ai_context = AiActor::init().await?;

        ai_context.emit_async(SubmitDiff { diff, id }).await;
        ai_context.terminate().await?;

        Ok(())
    }
}