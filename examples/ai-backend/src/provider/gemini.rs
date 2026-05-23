//! Proveedor Gemini (Google AI) para ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct GeminiProvider {
    pub(super) api_key: String,
    pub(super) client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, client: reqwest::Client) -> Self {
        Self { api_key, client }
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn default_model(&self) -> &'static str {
        "gemini-2.0-flash"
    }

    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}",
            model, self.api_key
        );
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });

        let response = self
            .client
            .post(&url)
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
                    if let Some(token) = parse_gemini_line(&line) {
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

/// Extrae texto de una linea SSE de Gemini.
///
/// Formato: `data: {"candidates":[{"content":{"parts":[{"text":"..."}]}}]}`
fn parse_gemini_line(line: &str) -> Option<String> {
    let data = line.strip_prefix("data: ")?;
    let val: serde_json::Value = serde_json::from_str(data).ok()?;
    let text = val["candidates"][0]["content"]["parts"][0]["text"].as_str()?;
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}
