# realtime-chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Crear el example `realtime-chat` — chat en sala unica via SSE usando `ag-realtime` EventBus, con UI embebida y sin dependencias de servicios externos.

**Architecture:** Un servidor Axum con `AppState { realtime: Arc<AgRealtime> }`. El cliente envia mensajes via `POST /messages`, el servidor los publica en el EventBus. Los clientes reciben mensajes via SSE (`GET /events`) usando `BroadcastStream` de `tokio-stream`. La UI de chat esta embebida como `&str` en el binario. La UI usa unicamente `createElement` y `textContent` — sin `innerHTML` — para evitar XSS.

**Tech Stack:** Rust 1.79, axum 0.7, ag-realtime, ag-observe, tokio-stream 0.1 (sync), serde, serde_json.

---

## Mapa de archivos

| Archivo | Accion | Responsabilidad |
|---|---|---|
| `Cargo.toml` (workspace) | Modificar | Agregar `tokio-stream` con feature `sync` |
| `examples/realtime-chat/Cargo.toml` | Crear | Dependencias del crate |
| `examples/realtime-chat/src/main.rs` | Crear | `AppState`, router, `main()` |
| `examples/realtime-chat/src/handlers.rs` | Crear | `index`, `post_message`, `get_events` |
| `examples/realtime-chat/README.md` | Crear | Instrucciones de ejecucion |

---

## Task 1: Workspace — agregar tokio-stream

**Files:**
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Agregar tokio-stream al workspace**

En `Cargo.toml` raiz, dentro de `[workspace.dependencies]`, agregar justo despues de `moka = ...`:

```toml
tokio-stream = { version = "0.1", features = ["sync"] }
```

La feature `sync` habilita `BroadcastStream` y `ReceiverStream` en `tokio_stream::wrappers`.

- [ ] **Step 2: Verificar que el workspace aun compila**

```bash
cargo check --workspace 2>&1 | grep "^error" | head -5
```

Resultado esperado: sin errores.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(workspace): agrega tokio-stream 0.1 con feature sync"
```

---

## Task 2: Cargo.toml del crate

**Files:**
- Create: `examples/realtime-chat/Cargo.toml`

- [ ] **Step 1: Crear el directorio y el Cargo.toml**

```bash
mkdir -p examples/realtime-chat/src
```

Crear `examples/realtime-chat/Cargo.toml`:

```toml
[package]
name = "realtime-chat"
version = "0.0.0"
edition = "2021"
publish = false
description = "Example: chat en tiempo real con ag-realtime y SSE"

[[bin]]
name = "realtime-chat"
path = "src/main.rs"

[dependencies]
ag-realtime   = { path = "../../crates/ag-realtime" }
ag-observe    = { path = "../../crates/ag-observe" }
axum          = { workspace = true }
futures-util  = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
tokio         = { workspace = true }
tokio-stream  = { workspace = true }
tracing       = { workspace = true }
tracing-subscriber = { workspace = true }
```

- [ ] **Step 2: Crear stub de main.rs para verificar que compila**

Crear `examples/realtime-chat/src/main.rs`:

```rust
fn main() {}
```

- [ ] **Step 3: Verificar que el crate se resuelve**

```bash
cargo check -p realtime-chat 2>&1 | grep "^error" | head -5
```

Resultado esperado: sin errores (stub vacio compila).

- [ ] **Step 4: Commit**

```bash
git add examples/realtime-chat/
git commit -m "chore(realtime-chat): Cargo.toml y estructura inicial del crate"
```

---

## Task 3: handlers.rs — index, post_message, get_events

**Files:**
- Create: `examples/realtime-chat/src/handlers.rs`

- [ ] **Step 1: Crear handlers.rs completo**

Crear `examples/realtime-chat/src/handlers.rs`:

```rust
//! Handlers HTTP del example realtime-chat.

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event as SseEvent, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};

use crate::AppState;

/// Mensaje de chat serializado en el bus y enviado via SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Nombre del usuario (max 32 chars).
    pub user: String,
    /// Texto del mensaje (max 512 chars).
    pub text: String,
    /// Timestamp Unix en segundos (el JS lo convierte con new Date(ts * 1000)).
    pub ts: u64,
}

/// Request body para POST /messages.
#[derive(Debug, Deserialize)]
pub struct PostMessageRequest {
    pub user: String,
    pub text: String,
}

/// HTML de la UI de chat embebido en el binario.
///
/// Usa unicamente `createElement` y `textContent` para renderizar
/// contenido del usuario — sin `innerHTML` — para evitar XSS.
const CHAT_HTML: &str = r##"<!DOCTYPE html>
<html lang="es">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>realtime-chat</title>
  <style>
    *{box-sizing:border-box;margin:0;padding:0}
    body{font-family:monospace;max-width:640px;margin:40px auto;padding:0 20px;background:#111;color:#eee}
    h2{margin-bottom:16px;color:#7cf}
    #messages{height:380px;overflow-y:auto;border:1px solid #333;padding:12px;margin-bottom:12px;background:#0d0d0d;border-radius:4px}
    .msg{margin:4px 0;line-height:1.5}
    .meta{color:#888;font-size:0.85em}
    .username{color:#7cf;font-weight:bold}
    input{width:100%;padding:8px;margin-bottom:8px;background:#1a1a1a;border:1px solid #333;color:#eee;border-radius:4px;font-family:monospace}
    .row{display:flex;gap:8px}
    .row input{flex:1;margin:0}
    button{padding:8px 20px;background:#7cf;color:#000;border:none;border-radius:4px;cursor:pointer;font-weight:bold}
    button:hover{background:#5ad}
    #status{font-size:0.8em;color:#666;margin-top:8px}
  </style>
</head>
<body>
  <h2>realtime-chat</h2>
  <div id="messages"></div>
  <input id="user" placeholder="Tu nombre" maxlength="32">
  <div class="row">
    <input id="text" placeholder="Mensaje..." maxlength="512">
    <button id="send-btn">Enviar</button>
  </div>
  <div id="status">conectando...</div>
<script>
"use strict";
const msgs = document.getElementById('messages');
const status = document.getElementById('status');
const userInput = document.getElementById('user');
const textInput = document.getElementById('text');

// Persistir nombre de usuario en localStorage
userInput.value = localStorage.getItem('chat-user') || '';
userInput.addEventListener('change', function() {
  localStorage.setItem('chat-user', userInput.value);
});

// SSE — todos los elementos creados con createElement/textContent (sin innerHTML)
const es = new EventSource('/events');

es.onopen = function() {
  status.textContent = 'conectado';
  status.style.color = '#4f4';
};

es.onmessage = function(e) {
  var m = JSON.parse(e.data);
  var row = document.createElement('div');
  row.className = 'msg';

  var meta = document.createElement('span');
  meta.className = 'meta';
  meta.appendChild(document.createTextNode('[' + new Date(m.ts * 1000).toLocaleTimeString() + '] '));

  var userSpan = document.createElement('span');
  userSpan.className = 'username';
  userSpan.textContent = m.user;  // textContent escapa automaticamente
  meta.appendChild(userSpan);
  meta.appendChild(document.createTextNode(': '));

  row.appendChild(meta);
  row.appendChild(document.createTextNode(m.text));  // textContent, sin XSS

  msgs.appendChild(row);
  msgs.scrollTop = msgs.scrollHeight;
};

es.onerror = function() {
  status.textContent = 'conexion perdida — reconectando...';
  status.style.color = '#f44';
};

function send() {
  var user = userInput.value.trim() || 'Anonimo';
  var text = textInput.value.trim();
  if (!text) return;
  fetch('/messages', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({user: user, text: text})
  });
  textInput.value = '';
  textInput.focus();
}

document.getElementById('send-btn').addEventListener('click', send);
textInput.addEventListener('keydown', function(e) {
  if (e.key === 'Enter') send();
});
</script>
</body>
</html>"##;

/// GET / — sirve la UI de chat embebida.
pub async fn index() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        CHAT_HTML,
    )
}

/// POST /messages — valida y publica un mensaje en el EventBus.
pub async fn post_message(
    State(state): State<AppState>,
    Json(req): Json<PostMessageRequest>,
) -> impl IntoResponse {
    let user = req.user.trim();
    let text = req.text.trim();

    if user.is_empty() || user.len() > 32 || text.is_empty() || text.len() > 512 {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let msg = ChatMessage {
        user: user.to_string(),
        text: text.to_string(),
        ts,
    };

    match state.realtime.broadcast_json("chat.message", &msg) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "error publicando mensaje en bus");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /events — stream SSE de mensajes del chat.
///
/// Cada evento SSE contiene el JSON de un [`ChatMessage`].
/// Los mensajes anteriores a la conexion NO se retransmiten.
/// Mensajes perdidos por lag del receiver se ignoran silenciosamente.
pub async fn get_events(
    State(state): State<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = state.realtime.bus().subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(ev) if ev.subject == "chat.message" => {
            let data = String::from_utf8_lossy(&ev.payload).into_owned();
            Some(Ok(SseEvent::default().data(data)))
        }
        // Lagged o subject diferente: ignorar sin desconectar al cliente
        _ => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

- [ ] **Step 2: Commit**

```bash
git add examples/realtime-chat/src/handlers.rs
git commit -m "feat(realtime-chat): handlers index/post_message/get_events — SSE + EventBus"
```

---

## Task 4: main.rs — AppState, router, main

**Files:**
- Modify: `examples/realtime-chat/src/main.rs`

- [ ] **Step 1: Reemplazar el stub**

```rust
//! realtime-chat: chat en tiempo real con ag-realtime y SSE.
//!
//! Ejecutar:
//! ```sh
//! cargo run -p realtime-chat
//! # Abrir http://localhost:3000 en dos ventanas del browser
//! ```

#![allow(missing_docs)]

mod handlers;

use ag_observe::{init as observe_init, ObserveConfig};
use ag_realtime::{AgRealtime, RealtimeConfig};
use axum::{http::StatusCode, routing::{get, post}, Router};
use std::sync::Arc;

/// Estado compartido entre todos los handlers.
#[derive(Clone)]
pub struct AppState {
    /// Subsistema de tiempo real para broadcast de mensajes.
    pub realtime: Arc<AgRealtime>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observe_init(&ObserveConfig::from_env()).ok();

    let realtime = Arc::new(AgRealtime::new(RealtimeConfig::default()));
    let state = AppState { realtime };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/events", get(handlers::get_events))
        .route("/messages", post(handlers::post_message))
        .route("/health", get(|| async { StatusCode::OK }))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!(addr = %addr, "realtime-chat iniciando");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 2: Build completo**

```bash
cargo build -p realtime-chat 2>&1 | grep "^error" | head -10
```

Resultado esperado: `Finished` sin errores.

- [ ] **Step 3: Clippy**

```bash
cargo clippy -p realtime-chat -- -D warnings 2>&1 | grep "^error" | head -10
```

Resultado esperado: sin errores.

- [ ] **Step 4: Commit**

```bash
git add examples/realtime-chat/src/main.rs
git commit -m "feat(realtime-chat): main.rs — AppState, router, servidor en puerto 3000"
```

---

## Task 5: README

**Files:**
- Create: `examples/realtime-chat/README.md`

- [ ] **Step 1: Crear README.md**

Crear `examples/realtime-chat/README.md`:

```markdown
# realtime-chat

Chat en tiempo real de sala unica. Demuestra `ag-realtime` con EventBus
in-process y streaming SSE. Sin base de datos, sin autenticacion, sin
servicios externos.

## Ejecucion

```bash
cargo run -p realtime-chat
```

Abrir **http://localhost:3000** en dos ventanas del browser para
ver los mensajes en tiempo real.

## Variables de entorno

| Variable     | Default  | Descripcion           |
|--------------|----------|-----------------------|
| `PORT`       | `3000`   | Puerto del servidor   |
| `LOG_FORMAT` | `pretty` | `pretty` o `json`     |

## API

| Metodo | Ruta        | Descripcion                    |
|--------|-------------|-------------------------------|
| GET    | `/`         | UI de chat (HTML embebido)    |
| GET    | `/events`   | Stream SSE de mensajes        |
| POST   | `/messages` | Publicar un mensaje           |
| GET    | `/health`   | Health check                  |

### Publicar con curl

```bash
curl -X POST http://localhost:3000/messages \
  -H "Content-Type: application/json" \
  -d '{"user":"alice","text":"hola desde curl"}'
```

### Escuchar eventos con curl

```bash
curl -N http://localhost:3000/events
```

## Crates demostrados

- `ag-realtime`: EventBus in-process pub/sub
- `ag-observe`: logging estructurado
```

- [ ] **Step 2: Verificacion final y commit**

```bash
cargo build -p realtime-chat 2>&1 | tail -3
git add examples/realtime-chat/README.md
git commit -m "docs(realtime-chat): README con instrucciones y API"
```

---

## Notas de implementacion

**XSS:** La UI usa `createElement` + `textContent` para todo contenido del usuario. El navegador escapa automaticamente cualquier caracter especial. Sin `innerHTML`.

**BroadcastStream lag:** Si un cliente SSE se atrasa, `BroadcastStream` devuelve `Err(BroadcastStreamRecvError::Lagged)`. El `filter_map` lo descarta silenciosamente — el cliente se pierde mensajes pero no se desconecta. Comportamiento correcto para chat best-effort.

**Sin historial:** Los mensajes anteriores a la conexion no se retransmiten. Diseno intencional — el example es minimal.

**Puerto 3000:** Distinto al 3001 de `ai-backend` para ejecutar ambos simultaneamente.
