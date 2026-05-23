use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

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
        _prompt: &str,
        _model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!("implementado en Task 3")
    }
}
