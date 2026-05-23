//! Proveedor OpenAI-compatible (OpenAI, Ollama, LM Studio, etc.) para ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct OpenAiProvider {
    pub(super) api_key: String,
    pub(super) base_url: String,
    pub(super) client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, client: reqwest::Client) -> Self {
        Self {
            api_key,
            base_url,
            client,
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn default_model(&self) -> &'static str {
        "gpt-4o-mini"
    }

    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "model": model,
            "stream": true,
            "messages": [{"role": "user", "content": prompt}]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                    if let Some(token) = parse_openai_line(&line) {
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

/// Extrae contenido de una linea SSE OpenAI-compatible.
///
/// Formato: `data: {"choices":[{"delta":{"content":"token"},"finish_reason":null}]}`
/// Fin:     `data: [DONE]`
fn parse_openai_line(line: &str) -> Option<String> {
    let data = line.strip_prefix("data: ")?;
    if data.trim() == "[DONE]" {
        return None;
    }
    let val: serde_json::Value = serde_json::from_str(data).ok()?;
    let content = val["choices"][0]["delta"]["content"].as_str()?;
    if content.is_empty() {
        None
    } else {
        Some(content.to_string())
    }
}
