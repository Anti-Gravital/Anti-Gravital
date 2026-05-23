# Spec: ai-backend — Example de streaming SSE simulado

**Fecha:** 2026-05-22
**Fase:** 4 (Examples)
**Ubicacion:** `examples/ai-backend/`
**Estado:** Aprobado para implementacion

---

## 1. Objetivo

Demostrar streaming Server-Sent Events (SSE) al estilo de una API de IA, usando respuestas pre-definidas emitidas token a token. Sin dependencias de APIs externas. Ejecutable con un solo comando.

## 2. Alcance

### En scope

- Servidor Axum con `POST /chat` que devuelve SSE token a token
- Respuestas pre-definidas mapeadas por palabras clave del prompt
- Delay configurable entre tokens (simula latencia de red/modelo)
- `ag-observe` para logging y metricas
- `GET /health` para verificacion de estado
- README con instrucciones y ejemplos `curl`

### Fuera de scope

- Integracion con API real de IA (Claude, OpenAI)
- Memoria de conversacion / contexto entre llamadas
- Autenticacion
- Rate limiting
- UI web

## 3. Arquitectura

### Estructura de archivos

```
examples/ai-backend/
  Cargo.toml
  README.md
  src/
    main.rs       router, main(), setup de observabilidad
    handlers.rs   chat_stream (SSE), health
    responses.rs  RESPONSES map + pick_response()
```

### Flujo de datos

```
Cliente                    Servidor
   |                          |
   | POST /chat               |
   | {"prompt": "que es rust?"}|
   |------------------------->|
   |                          | pick_response(prompt) -> Vec<String>
   |                          | stream tokens con delay 50ms
   |                          |
   | data: {"token":"Rust","done":false}
   |<-------------------------|
   | data: {"token":" es","done":false}
   |<-------------------------|
   | ...                      |
   | data: {"token":"","done":true}
   |<-------------------------|
```

## 4. API

### POST /chat

Inicia un stream SSE de respuesta simulada.

**Request body:**
```json
{ "prompt": "que es rust?" }
```

**Content-Type de respuesta:** `text/event-stream`

**Formato de cada evento SSE:**
```
data: {"token":"Rust","done":false}

data: {"token":" es","done":false}

data: {"token":"","done":true}

```

**Evento final:** `done: true` con `token: ""` — indica fin del stream.

**Comportamiento:**
- Si el prompt vacio: 422 Unprocessable Entity
- Si el prompt tiene match en RESPONSES: usa esa respuesta
- Si no hay match: usa respuesta por defecto sobre Anti-Gravital

### GET /health

**Respuesta:** 200 OK, body `{"status":"ok"}`

## 5. Respuestas pre-definidas

Mapa estatico en `responses.rs`. Busqueda case-insensitive por substring en el prompt.

| Palabra clave | Respuesta |
|---|---|
| `rust` | Descripcion de Rust como lenguaje de sistemas |
| `anti-gravital` | Descripcion del framework |
| `axum` | Descripcion de Axum como framework web |
| `async` | Explicacion de async/await en Rust |
| (default) | Descripcion generica de Anti-Gravital |

Cada respuesta es un `&str` de ~50-100 palabras que se divide en tokens por espacio.

## 6. Modelo de datos

```rust
// Request
#[derive(serde::Deserialize)]
struct ChatRequest {
    prompt: String,
}

// Cada evento SSE
#[derive(serde::Serialize)]
struct TokenEvent {
    token: String,
    done: bool,
}
```

## 7. Implementacion del stream

```rust
// handlers.rs (pseudocodigo)
async fn chat_stream(
    State(_state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let tokens = pick_response(&req.prompt);
    let delay = Duration::from_millis(50);

    let stream = async_stream::stream! {
        for token in tokens {
            tokio::time::sleep(delay).await;
            let ev = TokenEvent { token, done: false };
            yield Ok(Event::default().data(serde_json::to_string(&ev).unwrap()));
        }
        let end = TokenEvent { token: String::new(), done: true };
        yield Ok(Event::default().data(serde_json::to_string(&end).unwrap()));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

El crate `async_stream` se agrega como dependencia del example (no workspace — es especifico de este use case).

## 8. Dependencias del crate

```toml
[dependencies]
ag-observe = { path = "../../crates/ag-observe" }
axum       = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
tokio      = { workspace = true }
tracing    = { workspace = true }
tracing-subscriber = { workspace = true }
async-stream = "0.3"
futures-util = { workspace = true }
```

`async-stream` no se agrega al workspace — es una dependencia interna del example.
`futures-util` ya esta en el workspace como `futures-util = "0.3"`.

## 9. Ejecucion y prueba

```bash
cargo run -p ai-backend
```

Prueba con curl:
```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "que es rust?"}'
```

Variables de entorno:
- `PORT`: puerto del servidor (default: 3001, distinto al realtime-chat)
- `TOKEN_DELAY_MS`: delay entre tokens en milisegundos (default: 50)
- `LOG_FORMAT`: `pretty` o `json`

## 10. Observabilidad

`ag-observe::init()` se llama al inicio. Cada request `POST /chat` emite:
- Span de tracing con `prompt` (truncado a 100 chars) y `tokens_count`
- Log `info` al completar el stream

## 11. Tests

El crate compila y es verificado en CI con `cargo build -p ai-backend`. Sin tests unitarios propios — la correctitud se verifica con `curl`.

## 12. README minimo

El README incluye:
- Descripcion de una linea: "Streaming SSE token-a-token al estilo IA, sin API externa"
- Como ejecutar
- Ejemplos `curl` para cada tipo de respuesta
- Que crates demuestra (`ag-observe`)
- Nota sobre como conectar una API real en el futuro (TECH-DEBT)
