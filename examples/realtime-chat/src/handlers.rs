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
  userSpan.textContent = m.user;
  meta.appendChild(userSpan);
  meta.appendChild(document.createTextNode(': '));

  row.appendChild(meta);
  row.appendChild(document.createTextNode(m.text));

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
  }).catch(function(err) { console.error('error enviando mensaje:', err); });
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

    if user.is_empty() || user.chars().count() > 32 || text.is_empty() || text.chars().count() > 512
    {
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
