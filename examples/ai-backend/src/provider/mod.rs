//! AiProvider trait and dynamic registry of AI providers.

pub mod claude;
pub mod gemini;
pub mod openai;

use async_trait::async_trait;
use futures_util::stream::BoxStream;
use std::collections::HashMap;
use std::sync::Arc;

pub use claude::ClaudeProvider;
pub use gemini::GeminiProvider;
pub use openai::OpenAiProvider;

/// AI provider subsystem error.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("error HTTP: {0}")]
    Http(String),
    #[error("respuesta invalida del proveedor: {0}")]
    Parse(String),
}

/// Common interface for all AI providers.
///
/// Implement this trait to add a new provider.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider identifier: "claude", "gemini", "openai".
    fn name(&self) -> &'static str;

    /// Default model if the client does not specify one.
    fn default_model(&self) -> &'static str;

    /// Starts a completion stream. Each item is a text fragment.
    ///
    /// The stream ends when the provider closes the connection.
    /// Errors mid-stream are propagated as `Err(AiError)`.
    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError>;
}

/// Provider information for the GET /providers endpoint.
#[derive(serde::Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub default_model: String,
}

/// Registry of available providers based on the environment API keys.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    default: Option<String>,
}

impl ProviderRegistry {
    /// Builds the registry by detecting API keys in environment variables.
    ///
    /// Providers registered based on present keys:
    /// - `ANTHROPIC_API_KEY` -> `ClaudeProvider`
    /// - `GEMINI_API_KEY`    -> `GeminiProvider`
    /// - `OPENAI_API_KEY`    -> `OpenAiProvider`
    ///
    /// `AI_DEFAULT_PROVIDER` overrides the default provider.
    pub fn from_env() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("fallo al construir cliente HTTP");

        let mut providers: HashMap<String, Arc<dyn AiProvider>> = HashMap::new();
        let mut first: Option<String> = None;

        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            let p = Arc::new(ClaudeProvider::new(key, client.clone()));
            if first.is_none() {
                first = Some("claude".to_string());
            }
            providers.insert("claude".to_string(), p);
        }
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let p = Arc::new(GeminiProvider::new(key, client.clone()));
            if first.is_none() {
                first = Some("gemini".to_string());
            }
            providers.insert("gemini".to_string(), p);
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            let base = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com".to_string());
            let p = Arc::new(OpenAiProvider::new(key, base, client.clone()));
            if first.is_none() {
                first = Some("openai".to_string());
            }
            providers.insert("openai".to_string(), p);
        }

        let default = std::env::var("AI_DEFAULT_PROVIDER").ok().or(first);
        Self { providers, default }
    }

    /// Gets a provider by name. `None` if it is not registered.
    pub fn get(&self, name: &str) -> Option<Arc<dyn AiProvider>> {
        self.providers.get(name).cloned()
    }

    /// Name of the default provider. `None` if the registry is empty.
    pub fn default_name(&self) -> Option<&str> {
        self.default.as_deref()
    }

    /// Default provider. `None` if the registry is empty.
    pub fn default_provider(&self) -> Option<Arc<dyn AiProvider>> {
        self.default
            .as_ref()
            .and_then(|n| self.providers.get(n))
            .cloned()
    }

    /// Lists information for all available providers.
    pub fn available(&self) -> Vec<ProviderInfo> {
        let mut list: Vec<ProviderInfo> = self
            .providers
            .values()
            .map(|p| ProviderInfo {
                name: p.name().to_string(),
                default_model: p.default_model().to_string(),
            })
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// `true` if no provider is registered.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}
