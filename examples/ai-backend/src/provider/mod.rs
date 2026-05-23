//! Trait AiProvider y registro dinamico de proveedores de IA.

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

/// Error del subsistema de proveedores de IA.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("error HTTP: {0}")]
    Http(String),
    #[error("respuesta invalida del proveedor: {0}")]
    Parse(String),
}

/// Interfaz comun para todos los proveedores de IA.
///
/// Implementar este trait para agregar un nuevo proveedor.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Identificador del proveedor: "claude", "gemini", "openai".
    fn name(&self) -> &'static str;

    /// Modelo por defecto si el cliente no especifica uno.
    fn default_model(&self) -> &'static str;

    /// Inicia un stream de completion. Cada item es un fragmento de texto.
    ///
    /// El stream termina cuando el proveedor cierra la conexion.
    /// Los errores a mitad del stream se propagan como `Err(AiError)`.
    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError>;
}

/// Informacion de un proveedor para el endpoint GET /providers.
#[derive(serde::Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub default_model: String,
}

/// Registro de proveedores disponibles segun las API keys del entorno.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    default: Option<String>,
}

impl ProviderRegistry {
    /// Construye el registry detectando API keys en variables de entorno.
    ///
    /// Proveedores registrados segun keys presentes:
    /// - `ANTHROPIC_API_KEY` -> `ClaudeProvider`
    /// - `GEMINI_API_KEY`    -> `GeminiProvider`
    /// - `OPENAI_API_KEY`    -> `OpenAiProvider`
    ///
    /// `AI_DEFAULT_PROVIDER` sobreescribe el proveedor por defecto.
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

    /// Obtiene un proveedor por nombre. `None` si no esta registrado.
    pub fn get(&self, name: &str) -> Option<Arc<dyn AiProvider>> {
        self.providers.get(name).cloned()
    }

    /// Nombre del proveedor por defecto. `None` si el registry esta vacio.
    pub fn default_name(&self) -> Option<&str> {
        self.default.as_deref()
    }

    /// Proveedor por defecto. `None` si el registry esta vacio.
    pub fn default_provider(&self) -> Option<Arc<dyn AiProvider>> {
        self.default
            .as_ref()
            .and_then(|n| self.providers.get(n))
            .cloned()
    }

    /// Lista informacion de todos los proveedores disponibles.
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

    /// `true` si no hay ningun proveedor registrado.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}
