use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

pub struct OpenAiProvider {
    pub(super) api_key: String,
    pub(super) base_url: String,
    pub(super) client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, client: reqwest::Client) -> Self {
        Self { api_key, base_url, client }
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
        _prompt: &str,
        _model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!("implementado en Task 5")
    }
}
