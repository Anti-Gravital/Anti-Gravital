# ai-backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Crear el example `ai-backend` — servidor de streaming SSE multi-proveedor (Claude, Gemini, OpenAI-compatible) con trait `AiProvider` extensible y registro dinamico por API keys del entorno.

**Architecture:** `ProviderRegistry::from_env()` detecta al arranque que API keys estan disponibles y registra solo esos proveedores. Cada proveedor implementa `AiProvider` con `stream_completion` que devuelve `BoxStream<Result<String, AiError>>` via un canal mpsc. El handler `chat_stream` convierte ese stream en SSE token a token. Sin proveedores = servidor arranca, `/chat` devuelve 503.

**Tech Stack:** Rust 1.79, axum 0.7, ag-observe, reqwest 0.12 (con feature stream), async-trait 0.1, tokio-stream 0.1 (sync), futures-util, serde_json.

---

## Mapa de archivos

| Archivo | Accion | Responsabilidad |
|---|---|---|
| `Cargo.toml` (workspace) | Modificar | Agregar feature `stream` a reqwest |
| `examples/ai-backend/Cargo.toml` | Crear | Dependencias del crate |
| `examples/ai-backend/src/main.rs` | Crear | `AppState`, router, `main()` |
| `examples/ai-backend/src/handlers.rs` | Crear | `chat_stream`, `list_providers`, `health` |
| `examples/ai-backend/src/provider/mod.rs` | Crear | `AiProvider` trait, `AiError`, `ProviderRegistry`, `ProviderInfo` |
| `examples/ai-backend/src/provider/claude.rs` | Crear | `ClaudeProvider` |
| `examples/ai-backend/src/provider/gemini.rs` | Crear | `GeminiProvider` |
| `examples/ai-backend/src/provider/openai.rs` | Crear | `OpenAiProvider` |
| `examples/ai-backend/README.md` | Crear | Instrucciones y ejemplos curl |

---

## Task 1: Workspace + Cargo.toml del crate

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `examples/ai-backend/Cargo.toml`

- [ ] **Step 1: Agregar feature `stream` a reqwest en el workspace**

En `Cargo.toml` raiz, cambiar la linea de reqwest:

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
```

La feature `stream` habilita `Response::bytes_stream()` necesario para leer SSE de las APIs de IA.

- [ ] **Step 2: Crear el directorio y Cargo.toml del crate**

```bash
mkdir -p examples/ai-backend/src/provider
```

Crear `examples/ai-backend/Cargo.toml`:

```toml
[package]
name = "ai-backend"
version = "0.0.0"
edition = "2021"
publish = false
description = "Example: streaming SSE multi-proveedor IA (Claude, Gemini, OpenAI-compatible)"

[[bin]]
name = "ai-backend"
path = "src/main.rs"

[dependencies]
ag-observe   = { path = "../../crates/ag-observe" }
axum         = { workspace = true }
futures-util = { workspace = true }
reqwest      = { workspace = true }
serde        = { workspace = true }
serde_json   = { workspace = true }
thiserror    = { workspace = true }
tokio        = { workspace = true }
tokio-stream = { workspace = true }
tracing      = { workspace = true }
tracing-subscriber = { workspace = true }
async-trait  = "0.1"
```

`async-trait` se declara solo en este crate (no workspace) — es especifico de este pattern.

- [ ] **Step 3: Crear stub main.rs y verificar que compila**

Crear `examples/ai-backend/src/main.rs`:

```rust
fn main() {}
```

```bash
cargo check -p ai-backend 2>&1 | grep "^error" | head -5
```

Resultado esperado: sin errores.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock examples/ai-backend/
git commit -m "chore(ai-backend): Cargo.toml, feature stream en reqwest workspace"
```

---

## Task 2: provider/mod.rs — AiProvider trait + ProviderRegistry

**Files:**
- Create: `examples/ai-backend/src/provider/mod.rs`

- [ ] **Step 1: Crear provider/mod.rs**

Crear `examples/ai-backend/src/provider/mod.rs`:

```rust
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
    /// - `ANTHROPIC_API_KEY` → `ClaudeProvider`
    /// - `GEMINI_API_KEY`    → `GeminiProvider`
    /// - `OPENAI_API_KEY`    → `OpenAiProvider`
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
            if first.is_none() { first = Some("claude".to_string()); }
            providers.insert("claude".to_string(), p);
        }
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let p = Arc::new(GeminiProvider::new(key, client.clone()));
            if first.is_none() { first = Some("gemini".to_string()); }
            providers.insert("gemini".to_string(), p);
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            let base = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com".to_string());
            let p = Arc::new(OpenAiProvider::new(key, base, client.clone()));
            if first.is_none() { first = Some("openai".to_string()); }
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
        self.default.as_ref().and_then(|n| self.providers.get(n)).cloned()
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
```

- [ ] **Step 2: Crear stubs de los tres proveedores para que compile**

Crear `examples/ai-backend/src/provider/claude.rs`:

```rust
use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

pub struct ClaudeProvider { api_key: String, client: reqwest::Client }
impl ClaudeProvider {
    pub fn new(api_key: String, client: reqwest::Client) -> Self { Self { api_key, client } }
}
#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &'static str { "claude" }
    fn default_model(&self) -> &'static str { "claude-3-5-haiku-20241022" }
    async fn stream_completion(&self, _prompt: &str, _model: &str) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!()
    }
}
```

Crear `examples/ai-backend/src/provider/gemini.rs`:

```rust
use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

pub struct GeminiProvider { api_key: String, client: reqwest::Client }
impl GeminiProvider {
    pub fn new(api_key: String, client: reqwest::Client) -> Self { Self { api_key, client } }
}
#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &'static str { "gemini" }
    fn default_model(&self) -> &'static str { "gemini-2.0-flash" }
    async fn stream_completion(&self, _prompt: &str, _model: &str) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!()
    }
}
```

Crear `examples/ai-backend/src/provider/openai.rs`:

```rust
use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::stream::BoxStream;

pub struct OpenAiProvider { api_key: String, base_url: String, client: reqwest::Client }
impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, client: reqwest::Client) -> Self { Self { api_key, base_url, client } }
}
#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &'static str { "openai" }
    fn default_model(&self) -> &'static str { "gpt-4o-mini" }
    async fn stream_completion(&self, _prompt: &str, _model: &str) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        todo!()
    }
}
```

- [ ] **Step 3: Verificar que compila**

```bash
cargo check -p ai-backend 2>&1 | grep "^error" | head -10
```

Resultado esperado: errores de `mod provider` no encontrado en main.rs — normal, main.rs es stub todavia.

- [ ] **Step 4: Commit**

```bash
git add examples/ai-backend/src/provider/
git commit -m "feat(ai-backend): AiProvider trait + ProviderRegistry + stubs Claude/Gemini/OpenAI"
```

---

## Task 3: ClaudeProvider — streaming real de Anthropic

**Files:**
- Modify: `examples/ai-backend/src/provider/claude.rs`

- [ ] **Step 1: Reemplazar el stub con la implementacion completa**

El enfoque: la respuesta HTTP de Anthropic es un stream SSE. Se lanza un `tokio::spawn` que lee chunks, los parte en lineas, parsea los eventos `content_block_delta` y envia los tokens via canal mpsc. El handler recibe del `ReceiverStream`.

```rust
//! Proveedor Claude (Anthropic) para ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct ClaudeProvider {
    api_key: String,
    client: reqwest::Client,
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

                // Procesar lineas completas (terminan en \n)
                while let Some(pos) = buf.find('\n') {
                    let line = buf[..pos].trim().to_string();
                    buf = buf[pos + 1..].to_string();
                    if let Some(token) = parse_claude_line(&line) {
                        if tx.send(Ok(token)).await.is_err() {
                            return; // cliente desconectado
                        }
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx).boxed())
    }
}

/// Extrae el token de texto de una linea SSE de Anthropic.
///
/// Formato esperado: `data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"..."}}`
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
```

- [ ] **Step 2: Commit**

```bash
git add examples/ai-backend/src/provider/claude.rs
git commit -m "feat(ai-backend): ClaudeProvider — streaming SSE Anthropic con mpsc channel"
```

---

## Task 4: GeminiProvider — streaming de Google AI

**Files:**
- Modify: `examples/ai-backend/src/provider/gemini.rs`

- [ ] **Step 1: Reemplazar el stub**

La API de Gemini con `streamGenerateContent` devuelve SSE con `data:` prefijado JSON.
Cada evento tiene `candidates[0].content.parts[0].text`.

```rust
//! Proveedor Gemini (Google AI) para ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
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
/// Formato esperado: `data: {"candidates":[{"content":{"parts":[{"text":"..."}]}}]}`
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
```

- [ ] **Step 2: Commit**

```bash
git add examples/ai-backend/src/provider/gemini.rs
git commit -m "feat(ai-backend): GeminiProvider — streaming SSE Google AI"
```

---

## Task 5: OpenAiProvider — streaming OpenAI-compatible

**Files:**
- Modify: `examples/ai-backend/src/provider/openai.rs`

- [ ] **Step 1: Reemplazar el stub**

Funciona con OpenAI, Ollama (`http://localhost:11434`), LM Studio y cualquier endpoint compatible.
El formato SSE es `data: {"choices":[{"delta":{"content":"token"}}]}` con `data: [DONE]` al final.

```rust
//! Proveedor OpenAI-compatible (OpenAI, Ollama, LM Studio, etc.) para ai-backend.

use super::{AiError, AiProvider};
use async_trait::async_trait;
use futures_util::{stream::BoxStream, StreamExt as _};
use tokio_stream::wrappers::ReceiverStream;

pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
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
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
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
```

- [ ] **Step 2: Commit**

```bash
git add examples/ai-backend/src/provider/openai.rs
git commit -m "feat(ai-backend): OpenAiProvider — streaming SSE OpenAI-compatible (Ollama, LM Studio)"
```

---

## Task 6: handlers.rs — chat_stream, list_providers, health

**Files:**
- Create: `examples/ai-backend/src/handlers.rs`

- [ ] **Step 1: Crear handlers.rs**

```rust
//! Handlers HTTP del example ai-backend.

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::{stream::BoxStream, StreamExt as _};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::{provider::AiError, AppState};

/// Request body para POST /chat.
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// Texto del prompt.
    pub prompt: String,
    /// Proveedor a usar: "claude", "gemini", "openai".
    /// Si se omite, usa el proveedor por defecto del registry.
    pub provider: Option<String>,
    /// Modelo especifico. Si se omite, usa el modelo por defecto del proveedor.
    pub model: Option<String>,
}

/// Cada evento SSE enviado al cliente.
#[derive(Serialize)]
struct TokenEvent {
    token: String,
    done: bool,
}

/// Evento de error enviado como ultimo evento SSE del stream.
#[derive(Serialize)]
struct ErrorEvent {
    error: String,
    done: bool,
}

/// POST /chat — inicia un stream SSE de tokens de IA.
///
/// Respuestas:
/// - `200` + SSE stream si hay proveedor disponible
/// - `422` si el prompt esta vacio
/// - `503` si el proveedor pedido no esta configurado
pub async fn chat_stream(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    // Seleccionar proveedor
    let provider = match req.provider.as_deref() {
        Some(name) => state.registry.get(name),
        None => state.registry.default_provider(),
    };

    let Some(provider) = provider else {
        let hint = if state.registry.is_empty() {
            "Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY"
        } else {
            "Proveedor no disponible. Consulta GET /providers para ver los disponibles."
        };
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": hint})),
        )
            .into_response();
    };

    let model = req
        .model
        .as_deref()
        .unwrap_or_else(|| provider.default_model());

    // Iniciar stream del proveedor
    let token_stream: BoxStream<'static, Result<String, AiError>> =
        match provider.stream_completion(prompt, model).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "error iniciando stream del proveedor");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        };

    // Evento final "done: true"
    let done_event = futures_util::stream::once(async {
        let data = serde_json::to_string(&TokenEvent {
            token: String::new(),
            done: true,
        })
        .unwrap_or_default();
        Ok::<SseEvent, Infallible>(SseEvent::default().data(data))
    });

    // Convertir stream de tokens a stream SSE
    let sse_stream = token_stream
        .map(|result| {
            let data = match result {
                Ok(token) => serde_json::to_string(&TokenEvent { token, done: false })
                    .unwrap_or_default(),
                Err(e) => serde_json::to_string(&ErrorEvent {
                    error: e.to_string(),
                    done: true,
                })
                .unwrap_or_default(),
            };
            Ok::<SseEvent, Infallible>(SseEvent::default().data(data))
        })
        .chain(done_event);

    Sse::new(sse_stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// Response body para GET /providers.
#[derive(Serialize)]
struct ProvidersResponse {
    providers: Vec<crate::provider::ProviderInfo>,
    default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'static str>,
}

/// GET /providers — lista los proveedores disponibles segun las keys configuradas.
pub async fn list_providers(State(state): State<AppState>) -> impl IntoResponse {
    let providers = state.registry.available();
    let default = state.registry.default_name().map(String::from);
    let hint = if providers.is_empty() {
        Some("Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY")
    } else {
        None
    };
    Json(ProvidersResponse { providers, default, hint })
}

/// GET /health — siempre 200.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.registry.available().len();
    Json(serde_json::json!({
        "status": "ok",
        "providers_available": count
    }))
}
```

- [ ] **Step 2: Commit**

```bash
git add examples/ai-backend/src/handlers.rs
git commit -m "feat(ai-backend): handlers chat_stream/list_providers/health — SSE multi-proveedor"
```

---

## Task 7: main.rs — AppState, router, main

**Files:**
- Modify: `examples/ai-backend/src/main.rs`

- [ ] **Step 1: Reemplazar el stub**

```rust
//! ai-backend: streaming SSE multi-proveedor (Claude, Gemini, OpenAI-compatible).
//!
//! Ejecutar con al menos una API key:
//! ```sh
//! ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend
//! GEMINI_API_KEY=AIza...       cargo run -p ai-backend
//! OPENAI_API_KEY=sk-...        cargo run -p ai-backend
//! # OpenAI-compatible (Ollama):
//! OPENAI_API_KEY=ollama OPENAI_BASE_URL=http://localhost:11434 cargo run -p ai-backend
//! ```

#![allow(missing_docs)]

mod handlers;
mod provider;

use ag_observe::{init as observe_init, ObserveConfig};
use axum::{
    routing::{get, post},
    Router,
};
use provider::ProviderRegistry;
use std::sync::Arc;

/// Estado compartido entre handlers.
#[derive(Clone)]
pub struct AppState {
    /// Registro dinamico de proveedores de IA disponibles.
    pub registry: Arc<ProviderRegistry>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observe_init(&ObserveConfig::from_env()).ok();

    let registry = Arc::new(ProviderRegistry::from_env());

    if registry.is_empty() {
        tracing::warn!(
            "Ningun proveedor de IA configurado. \
             Configura ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY. \
             El endpoint /chat devolvera 503."
        );
    } else {
        let names: Vec<_> = registry.available().into_iter().map(|p| p.name).collect();
        tracing::info!(providers = ?names, "proveedores de IA disponibles");
    }

    let state = AppState { registry };

    let app = Router::new()
        .route("/chat", post(handlers::chat_stream))
        .route("/providers", get(handlers::list_providers))
        .route("/health", get(handlers::health))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!(addr = %addr, "ai-backend iniciando");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 2: Build completo**

```bash
cargo build -p ai-backend 2>&1 | grep "^error" | head -10
```

Resultado esperado: `Finished` sin errores.

- [ ] **Step 3: Clippy**

```bash
cargo clippy -p ai-backend -- -D warnings 2>&1 | grep "^error" | head -10
```

Resultado esperado: sin errores.

- [ ] **Step 4: Commit**

```bash
git add examples/ai-backend/src/main.rs
git commit -m "feat(ai-backend): main.rs — AppState, router, puerto 3001"
```

---

## Task 8: README

**Files:**
- Create: `examples/ai-backend/README.md`

- [ ] **Step 1: Crear README.md**

```markdown
# ai-backend

Servidor de streaming SSE multi-proveedor de IA. Detecta las API keys
disponibles en el entorno y expone solo los proveedores configurados.
Agregar un proveedor nuevo = implementar el trait `AiProvider`.

## Ejecucion

```bash
# Claude (Anthropic)
ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend

# Gemini (Google)
GEMINI_API_KEY=AIza... cargo run -p ai-backend

# OpenAI
OPENAI_API_KEY=sk-... cargo run -p ai-backend

# Ollama (OpenAI-compatible local, modelo por defecto llama3)
OPENAI_API_KEY=ollama \
OPENAI_BASE_URL=http://localhost:11434 \
cargo run -p ai-backend

# Multiples proveedores simultaneos
ANTHROPIC_API_KEY=... GEMINI_API_KEY=... cargo run -p ai-backend
```

## Variables de entorno

| Variable              | Default                    | Descripcion                       |
|-----------------------|----------------------------|-----------------------------------|
| `ANTHROPIC_API_KEY`   | —                          | Habilita proveedor claude         |
| `GEMINI_API_KEY`      | —                          | Habilita proveedor gemini         |
| `OPENAI_API_KEY`      | —                          | Habilita proveedor openai         |
| `OPENAI_BASE_URL`     | `https://api.openai.com`   | Endpoint OpenAI-compatible        |
| `AI_DEFAULT_PROVIDER` | primer registrado          | Proveedor por defecto             |
| `PORT`                | `3001`                     | Puerto del servidor               |
| `LOG_FORMAT`          | `pretty`                   | `pretty` o `json`                 |

## API

| Metodo | Ruta         | Descripcion                          |
|--------|--------------|--------------------------------------|
| POST   | `/chat`      | Stream SSE de tokens                 |
| GET    | `/providers` | Lista proveedores disponibles        |
| GET    | `/health`    | Health check                         |

### Ver proveedores disponibles

```bash
curl http://localhost:3001/providers
```

### Chat con proveedor por defecto

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explica Rust en tres oraciones"}'
```

### Chat especificando proveedor y modelo

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "hola", "provider": "gemini", "model": "gemini-1.5-pro"}'
```

## Agregar un nuevo proveedor

1. Crear `src/provider/mi_proveedor.rs` implementando el trait `AiProvider`
2. Registrarlo en `ProviderRegistry::from_env()` segun la key del entorno
3. Re-exportarlo en `provider/mod.rs`

## Crates demostrados

- `ag-observe`: logging y trazabilidad
```

- [ ] **Step 2: Verificacion final**

```bash
cargo build -p ai-backend 2>&1 | tail -3
git add examples/ai-backend/README.md
git commit -m "docs(ai-backend): README con instrucciones multi-proveedor y ejemplos curl"
```

---

## Notas de implementacion

**SSE line parsing:** Cada proveedor implementa su propio `parse_*_line`. El patron es identico: `strip_prefix("data: ")` → parse JSON → extraer texto. La funcion es privada al modulo del proveedor.

**Canal mpsc en lugar de stream directo:** `bytes_stream()` de reqwest da chunks arbitrarios que pueden partir lineas SSE. El enfoque canal+spawn acumula en un buffer, procesa lineas completas, y envia tokens por el canal. `ReceiverStream` convierte el canal en `Stream`.

**Puerto 3001:** Distinto al 3000 de `realtime-chat` para ejecutar ambos ejemplos simultaneamente.

**Sin modo simulado:** El servidor arranca limpiamente sin API keys. Solo `/chat` devuelve 503 con instrucciones. `/health` y `/providers` responden siempre.

**Timeout del cliente HTTP:** 120 segundos por request. Suficiente para respuestas largas de modelos lentos.

**OPENAI_API_KEY para Ollama:** Ollama no requiere key real pero el campo es obligatorio en el protocolo. Usar cualquier valor, ej: `OPENAI_API_KEY=ollama`.
