use ollama_rs;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
pub async fn prompt() {
    let model = "llama3:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();
// By default it will connect to localhost:11434
    let ollama = Ollama::default();
    let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

    let mut stdout = tokio::io::stdout();
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        for resp in responses {
            stdout.write(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn it_works() {
        prompt().await;
    }
}
