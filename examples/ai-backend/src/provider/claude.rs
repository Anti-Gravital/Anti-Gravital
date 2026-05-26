//! Claude (Anthropic) provider for ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct ClaudeProvider {
    pub(super) api_key: String,
    pub(super) client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(api_key: String, client: reqwest::Client) -> Self {
        Self { api_key, client }
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn default_model(&self) -> &'static str {
        "claude-3-5-haiku-20241022"
    }

    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "stream": true,
            "messages": [{"role": "user", "content": prompt}]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::Http(format!("{status}: {text}")));
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<String, AiError>>(64);

        tokio::spawn(async move {
            let mut bytes_stream = response.bytes_stream();
            let mut buf = String::new();

            while let Some(chunk) = bytes_stream.next().await {
                let bytes = match chunk {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = tx.send(Err(AiError::Http(e.to_string()))).await;
                        return;
                    }
                };
                buf.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buf.find('\n') {
                    let line = buf[..pos].trim().to_string();
                    buf = buf[pos + 1..].to_string();
                    if let Some(token) = parse_claude_line(&line) {
                        if tx.send(Ok(token)).await.is_err() {
                            return;
                        }
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx).boxed())
    }
}

/// Extracts the text token from an Anthropic SSE line.
///
/// Format: `data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"..."}}`
fn parse_claude_line(line: &str) -> Option<String> {
    let data = line.strip_prefix("data: ")?;
    let val: serde_json::Value = serde_json::from_str(data).ok()?;
    if val["type"].as_str()? != "content_block_delta" {
        return None;
    }
    if val["delta"]["type"].as_str()? != "text_delta" {
        return None;
    }
    let text = val["delta"]["text"].as_str()?;
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}
