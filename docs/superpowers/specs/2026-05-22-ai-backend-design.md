# Spec: ai-backend — Example de streaming SSE multi-proveedor

**Fecha:** 2026-05-22
**Fase:** 4 (Examples)
**Ubicacion:** `examples/ai-backend/`
**Estado:** Aprobado para implementacion

---

## 1. Objetivo

Demostrar streaming Server-Sent Events (SSE) con multiples proveedores de IA (Claude, Gemini, OpenAI-compatible) a traves de un trait comun extensible. El servidor detecta en tiempo de arranque que API keys estan disponibles y registra solo los proveedores configurados. Si no hay ninguna key: el servidor inicia pero `/chat` devuelve 503 con instrucciones claras.

## 2. Alcance

### En scope

- Trait `AiProvider` con metodo de streaming
- `ProviderRegistry`: deteccion dinamica de proveedores en el entorno
- Proveedores implementados: `claude`, `gemini`, `openai` (+ compatible)
- `POST /chat` con seleccion de proveedor por parametro
- `GET /providers` lista proveedores disponibles
- `GET /health` siempre 200
- `ag-observe` para logging y trazabilidad
- README con ejemplos `curl` para cada proveedor

### Fuera de scope

- Modo simulado (sin API keys → error explicito, no fallback)
- Memoria de conversacion entre llamadas
- Autenticacion del cliente
- Rate limiting
- UI web
- Fine-tuning o embeddings

## 3. Arquitectura

### Estructura de archivos

```
examples/ai-backend/
  Cargo.toml
  README.md
  src/
    main.rs              AppState, router, main(), registro de proveedores
    handlers.rs          chat_stream, list_providers, health
    provider/
      mod.rs             trait AiProvider, ProviderRegistry, AiError
      claude.rs          ClaudeProvider (Anthropic API)
      gemini.rs          GeminiProvider (Google AI)
      openai.rs          OpenAiProvider (OpenAI + endpoints compatibles)
```

### AppState

```rust
#[derive(Clone)]
struct AppState {
    registry: Arc<ProviderRegistry>,
}
```

### Flujo de arranque

```
main()
  └── ProviderRegistry::from_env()
        ├── ANTHROPIC_API_KEY presente → registra ClaudeProvider
        ├── GEMINI_API_KEY presente    → registra GeminiProvider
        ├── OPENAI_API_KEY presente    → registra OpenAiProvider
        └── Sin ninguna key            → registry vacio (proveedores=[])
```

### Flujo de request

```
POST /chat {"prompt": "...", "provider": "claude"}
                    │
                    ▼
         registry.get("claude")
                    │
         presente ──┤── ausente → 503 + lista de providers disponibles
                    │
                    ▼
         claude.stream_completion(prompt, model)
                    │
              Stream<String>
                    │
                    ▼
         SSE: data: {"token":"...", "done":false}
              data: {"token":"", "done":true}
```

## 4. Trait AiProvider

```rust
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Identificador del proveedor: "claude", "gemini", "openai".
    fn name(&self) -> &'static str;

    /// Modelo por defecto si el cliente no especifica uno.
    fn default_model(&self) -> &'static str;

    /// Inicia un stream de completion. Devuelve tokens de texto uno a uno.
    ///
    /// Cada item del stream es un fragmento de texto (puede ser una palabra,
    /// varios caracteres, o un solo caracter segun el proveedor).
    async fn stream_completion(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, AiError>>, AiError>;
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("error HTTP: {0}")]
    Http(String),
    #[error("respuesta invalida del proveedor: {0}")]
    Parse(String),
    #[error("proveedor no disponible: {0}")]
    Unavailable(String),
}
```

## 5. ProviderRegistry

```rust
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    default: Option<String>,  // primer proveedor registrado, o AI_DEFAULT_PROVIDER
}

impl ProviderRegistry {
    /// Construye el registry leyendo variables de entorno.
    pub fn from_env() -> Self { ... }

    /// Obtiene un proveedor por nombre. None si no esta registrado.
    pub fn get(&self, name: &str) -> Option<Arc<dyn AiProvider>>;

    /// Proveedor por defecto segun AI_DEFAULT_PROVIDER o el primero registrado.
    pub fn default_provider(&self) -> Option<Arc<dyn AiProvider>>;

    /// Lista los nombres de proveedores disponibles.
    pub fn available(&self) -> Vec<&str>;

    /// True si no hay ningun proveedor registrado.
    pub fn is_empty(&self) -> bool;
}
```

### Variables de entorno del registry

| Variable | Proveedor | Descripcion |
|---|---|---|
| `ANTHROPIC_API_KEY` | claude | API key de Anthropic |
| `GEMINI_API_KEY` | gemini | API key de Google AI Studio |
| `OPENAI_API_KEY` | openai | API key de OpenAI o compatible |
| `OPENAI_BASE_URL` | openai | Endpoint alternativo (Ollama, LM Studio, etc.) Default: `https://api.openai.com` |
| `AI_DEFAULT_PROVIDER` | — | Proveedor a usar cuando el cliente no especifica |

## 6. API

### POST /chat

**Request body:**
```json
{
  "prompt": "Explica que es Rust en tres oraciones",
  "provider": "claude",
  "model": "claude-3-5-haiku-20241022"
}
```

- `provider`: opcional. Si se omite, usa el proveedor por defecto.
- `model`: opcional. Si se omite, usa el modelo por defecto del proveedor.

**Respuestas:**
- `200 OK` → SSE stream de tokens
- `422 Unprocessable Entity` → prompt vacio
- `503 Service Unavailable` → proveedor no disponible o ningun proveedor configurado

**Formato SSE:**
```
data: {"token":"Rust","done":false}

data: {"token":" es","done":false}

data: {"token":"","done":true}

```

**Evento de error durante stream** (si el upstream falla a mitad):
```
data: {"error":"timeout del proveedor","done":true}

```

### GET /providers

Lista los proveedores disponibles en este servidor.

**Response 200:**
```json
{
  "providers": [
    {"name": "claude", "default_model": "claude-3-5-haiku-20241022"},
    {"name": "gemini", "default_model": "gemini-2.0-flash"}
  ],
  "default": "claude"
}
```

Si registry vacio:
```json
{
  "providers": [],
  "default": null,
  "hint": "Configure ANTHROPIC_API_KEY, GEMINI_API_KEY o OPENAI_API_KEY"
}
```

### GET /health

Siempre responde `200 OK`:
```json
{"status": "ok", "providers_available": 2}
```

## 7. Implementaciones de proveedores

### ClaudeProvider

- Endpoint: `https://api.anthropic.com/v1/messages`
- Auth header: `x-api-key: {ANTHROPIC_API_KEY}`
- Header adicional: `anthropic-version: 2023-06-01`
- Request body: `{"model": "...", "max_tokens": 1024, "stream": true, "messages": [{"role": "user", "content": "..."}]}`
- Modelo por defecto: `claude-3-5-haiku-20241022`
- Parsing SSE: busca eventos `content_block_delta` con `delta.type == "text_delta"`, extrae `delta.text`

### GeminiProvider

- Endpoint: `https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?key={GEMINI_API_KEY}`
- Auth: query param `key`
- Request body: `{"contents": [{"parts": [{"text": "..."}]}]}`
- Modelo por defecto: `gemini-2.0-flash`
- Parsing: respuesta NDJSON, cada linea tiene `candidates[0].content.parts[0].text`

### OpenAiProvider

- Endpoint: `{OPENAI_BASE_URL}/v1/chat/completions`
- Auth: `Authorization: Bearer {OPENAI_API_KEY}`
- Request body: `{"model": "...", "stream": true, "messages": [{"role": "user", "content": "..."}]}`
- Modelo por defecto: `gpt-4o-mini`
- Compatible con: Ollama (`http://localhost:11434`), LM Studio, cualquier API OpenAI-compatible
- Parsing SSE: extrae `choices[0].delta.content` de cada evento `data:`

## 8. SSE parsing de upstream

Funcion auxiliar compartida entre Claude y OpenAI (ambos usan SSE):

```rust
fn parse_sse_line(line: &str) -> Option<serde_json::Value> {
    let data = line.strip_prefix("data: ")?;
    if data == "[DONE]" { return None; }
    serde_json::from_str(data).ok()
}
```

## 9. Dependencias del crate

```toml
[dependencies]
ag-observe    = { path = "../../crates/ag-observe" }
axum          = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
tokio         = { workspace = true }
tracing       = { workspace = true }
tracing-subscriber = { workspace = true }
reqwest       = { workspace = true }           # HTTP client para APIs
async-trait   = "0.1"                          # trait AiProvider async
futures-util  = { workspace = true }
thiserror     = { workspace = true }
```

`async-trait` no esta en el workspace — se declara solo en este crate.

## 10. Variables de entorno de ejecucion

| Variable | Default | Descripcion |
|---|---|---|
| `PORT` | `3001` | Puerto del servidor |
| `LOG_FORMAT` | `pretty` | Formato de logs |
| `ANTHROPIC_API_KEY` | — | Habilita proveedor claude |
| `GEMINI_API_KEY` | — | Habilita proveedor gemini |
| `OPENAI_API_KEY` | — | Habilita proveedor openai |
| `OPENAI_BASE_URL` | `https://api.openai.com` | Endpoint OpenAI-compatible |
| `AI_DEFAULT_PROVIDER` | primer registrado | Proveedor por defecto |

## 11. Ejecucion y prueba

```bash
# Con Claude
ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend

# Con Gemini
GEMINI_API_KEY=AI... cargo run -p ai-backend

# Con Ollama (OpenAI-compatible local)
OPENAI_API_KEY=ollama OPENAI_BASE_URL=http://localhost:11434 cargo run -p ai-backend

# Consultar proveedores disponibles
curl http://localhost:3001/providers

# Usar Claude
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "que es Rust?", "provider": "claude"}'

# Usar el proveedor por defecto
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "hola"}'
```

## 12. CI

El CI ejecuta `cargo build -p ai-backend` y `GET /health`. El endpoint `/chat` no se prueba en CI (requiere keys reales). El servidor arranca limpiamente sin keys — solo `/chat` devuelve 503.

## 13. Observabilidad

`ag-observe::init()` al inicio. Cada request `POST /chat` emite:
- Span con `provider`, `model` y `prompt` (truncado a 100 chars)
- Log `info` al completar stream con `tokens_streamed: N`
- Log `warn` si el upstream devuelve error

## 14. TECH-DEBT

- motivo: modelos fijos por proveedor; el cliente no puede listar modelos disponibles.
- impacto: el developer debe conocer el nombre exacto del modelo.
- eliminacion esperada: endpoint `GET /models?provider=claude` en segunda iteracion.

## 15. README minimo

El README incluye:
- Que demuestra: streaming SSE multi-proveedor con trait extensible
- Como ejecutar con cada proveedor
- Como agregar un proveedor nuevo (implementar `AiProvider`)
- Ejemplos `curl` completos
- Que crates de Anti-Gravital demuestra (`ag-observe`)
