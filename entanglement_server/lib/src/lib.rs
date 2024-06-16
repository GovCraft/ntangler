use futures_util::StreamExt;
use ollama_rs;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::Ollama;
use tokio::fs::read_to_string;
use tokio::io::AsyncWriteExt;
use dotenv::dotenv;
use std::env;
use mistralai_client::v1::chat::{ChatMessage, ChatMessageRole, ChatParams};
use mistralai_client::v1::client::Client;
use mistralai_client::v1::constants::Model;
use tracing::{instrument, trace};

pub async fn init() {
    dotenv().ok();

    let api_key = env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY must be set.");
    let endpoint = env::var("MISTRAL_ENDPOINT").expect("MISTRAL_ENDPOINT must be set.");

    let client = Client::new(Some(api_key), Some(endpoint), Some(3), Some(60)).unwrap();

    let messages = vec![ChatMessage {
        role: ChatMessageRole::User,
        content: "Just guess the next word: \"pub(crate) ...\"?".to_string(),
        tool_calls: None,
    }];
    let options = ChatParams {
        temperature: 0.0,
        random_seed: Some(42),
        ..Default::default()
    };

    let result = client
        .chat_async("model", messages, Some(options))
        .await
        .unwrap();
    println!(
        "{:?}: {}",
        result.choices[0].message.role, result.choices[0].message.content
    );
    assert!(client.is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIFF: &str = r#"
    diff --git a/src/lib.rs b/src/lib.rs
index 1a2b3c4..5d6e7f8 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,27 +1,34 @@
-// lib.rs

-pub fn fetch_data(url: &str, callback: fn(Result<String, reqwest::Error>)) {
-    let client = reqwest::blocking::Client::new();
-    match client.get(url).send() {
-        Ok(response) => {
-            if response.status().is_success() {
-                match response.text() {
-                    Ok(text) => callback(Ok(text)),
-                    Err(err) => callback(Err(err)),
-                }
-            } else {
-                callback(Err(reqwest::Error::new(
-                    reqwest::StatusCode::BAD_REQUEST,
-                    "Bad request",
-                )));
-            }
-        }
-        Err(err) => callback(Err(err)),
-    }
-}

-pub fn post_data(url: &str, body: &str, callback: fn(Result<String, reqwest::Error>)) {
-    let client = reqwest::blocking::Client::new();
-    match client.post(url).body(body.to_string()).send() {
-        Ok(response) => {
-            if response.status().is_success() {
-                match response.text() {
-                    Ok(text) => callback(Ok(text)),
-                    Err(err) => callback(Err(err)),
-                }
-            } else {
-                callback(Err(reqwest::Error::new(
-                    reqwest::StatusCode::BAD_REQUEST,
-                    "Bad request",
-                )));
-            }
-        }
-        Err(err) => callback(Err(err)),
-    }
+use reqwest;
+
+pub struct API {
+    base_url: String,
+}
+
+impl API {
+    pub fn new(base_url: &str) -> Self {
+        API {
+            base_url: base_url.to_string(),
+        }
+    }
+
+    pub fn fetch_data(&self, endpoint: &str) -> Result<String, reqwest::Error> {
+        let url = format!("{}{}", self.base_url, endpoint);
+        let response = reqwest::blocking::get(&url)?;
+        if response.status().is_success() {
+            response.text()
+        } else {
+            Err(reqwest::Error::new(
+                reqwest::StatusCode::BAD_REQUEST,
+                "Bad request",
+            ))
+        }
+    }
+
+    pub fn post_data(&self, endpoint: &str, body: &str) -> Result<String, reqwest::Error> {
+        let url = format!("{}{}", self.base_url, endpoint);
+        let client = reqwest::blocking::Client::new();
+        let response = client.post(&url).body(body.to_string()).send()?;
+        if response.status().is_success() {
+            response.text()
+        } else {
+            Err(reqwest::Error::new(
+                reqwest::StatusCode::BAD_REQUEST,
+                "Bad request",
+            ))
+        }
+    }
 }
+
+// Example usage
+fn main() {
+    let api = API::new("https://api.example.com");
+
+    match api.fetch_data("/data") {
+        Ok(data) => println!("Data: {}", data),
+        Err(err) => println!("Error: {}", err),
+    }
+
+    match api.post_data("/data", "{\"key\": \"value\"}") {
+        Ok(data) => println!("Data: {}", data),
+        Err(err) => println!("Error: {}", err),
+    }
+}

    "#;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn it_works() {
        let diff = "This is a test diff.";
        let _ = init().await;
    }
}
