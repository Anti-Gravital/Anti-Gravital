use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

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
        _prompt: &str,
        _model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!("implementado en Task 4")
    }
}
