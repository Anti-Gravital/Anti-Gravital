# Spec: realtime-chat — Example de chat en tiempo real

**Fecha:** 2026-05-22
**Fase:** 4 (Examples)
**Ubicacion:** `examples/realtime-chat/`
**Estado:** Aprobado para implementacion

---

## 1. Objetivo

Demostrar el uso de `ag-realtime` con Server-Sent Events (SSE) en un chat de sala unica. El example debe ser ejecutable con un solo comando, sin base de datos, sin autenticacion y sin dependencias de servicios externos.

## 2. Alcance

### En scope

- Un servidor Axum con 3 rutas: POST /messages, GET /events, GET /
- `AgRealtime` con `EventBus` in-process para broadcast de mensajes
- SSE via `tokio_stream::wrappers::BroadcastStream`
- UI de chat embebida como HTML en el binario (sin npm, sin archivos estaticos externos)
- `ag-observe` para logging estructurado
- README con instrucciones de ejecucion

### Fuera de scope

- Persistencia de mensajes
- Autenticacion de usuarios
- Multiples salas
- WebSocket (ag-realtime TECH-DEBT)
- Deploy / Docker

## 3. Arquitectura

### Estructura de archivos

```
examples/realtime-chat/
  Cargo.toml
  README.md
  src/
    main.rs      AppState, router, main()
    handlers.rs  post_message, get_events, index
```

### AppState

```rust
#[derive(Clone)]
struct AppState {
    realtime: Arc<AgRealtime>,
}
```

### Flujo de datos

```
Browser A          Servidor              Browser B
   |                   |                    |
   | POST /messages    |                    |
   | {"user":"alice",  |                    |
   |  "text":"hola"}   |                    |
   |------------------>|                    |
   |                   | broadcast_json     |
   |                   | "chat.message"     |
   |                   |----.               |
   |                   |    |EventBus       |
   |                   |<---'               |
   | 201 Created       |                    |
   |<------------------|                    |
   |                   | SSE data: {...}    |
   |<==================|===================>|
```

## 4. API

### POST /messages

Publica un mensaje en el chat.

**Request body:**
```json
{ "user": "alice", "text": "hola mundo" }
```

**Respuesta:** 201 Created (body vacio)

**Validacion:**
- `user` no vacio, max 32 chars
- `text` no vacio, max 512 chars
- Si falla: 422 Unprocessable Entity con mensaje de error

### GET /events

Stream SSE de mensajes del chat.

**Headers de respuesta:**
- `Content-Type: text/event-stream`
- `Cache-Control: no-cache`
- `X-Accel-Buffering: no`

**Formato de cada evento SSE:**
```
data: {"user":"alice","text":"hola mundo","ts":"2026-05-22T21:00:00Z"}

```

**Comportamiento:**
- Cada cliente recibe solo mensajes publicados DESPUES de conectarse (no hay historial)
- Mensajes perdidos por lag del receiver se ignoran silenciosamente
- Keep-alive automatico cada 15 segundos para mantener la conexion

### GET /

Sirve la UI de chat embebida.

**Content-Type:** `text/html; charset=utf-8`

## 5. UI embebida

HTML minimo (~80 lineas) embebido como string literal en `handlers.rs`. Contiene:

- Input para nombre de usuario (localStorage para persistencia entre recargas)
- Input para texto del mensaje + boton Enviar
- Area de mensajes (div con scroll automatico)
- JavaScript usando `EventSource` para conectar al SSE
- `fetch` para POST /messages
- Sin frameworks, sin CSS frameworks, sin dependencias externas

```html
<!-- estructura minima -->
<input id="user" placeholder="Tu nombre">
<div id="messages"></div>
<input id="text" placeholder="Mensaje">
<button onclick="send()">Enviar</button>
<script>
  const es = new EventSource('/events');
  es.onmessage = e => {
    const msg = JSON.parse(e.data);
    // agregar al div
  };
  function send() {
    fetch('/messages', { method: 'POST', body: JSON.stringify({...}) });
  }
</script>
```

## 6. Modelo de mensaje

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct ChatMessage {
    user: String,
    text: String,
    ts: String,  // ISO 8601 UTC, generado en el servidor
}
```

## 7. Dependencias del crate

```toml
[dependencies]
ag-realtime = { path = "../../crates/ag-realtime" }
ag-observe  = { path = "../../crates/ag-observe" }
axum        = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
tokio-stream = { workspace = true }
tracing     = { workspace = true }
tracing-subscriber = { workspace = true }
```

`tokio-stream` se agrega al workspace con version `"0.1"`.

## 8. Ejecucion

```bash
cargo run -p realtime-chat
# Abre http://localhost:3000 en dos ventanas para ver el chat en tiempo real
```

Variables de entorno:
- `PORT`: puerto del servidor (default: 3000)
- `LOG_FORMAT`: `pretty` (default) o `json`

## 9. Tests

El example no tiene tests unitarios propios — su correctitud se verifica ejecutandolo. El CI compila el crate (`cargo build -p realtime-chat`).

## 10. README minimo

El README del example debe incluir:
- Descripcion de una linea
- Como ejecutar
- Como probar (abrir dos ventanas del browser)
- Que crates de Anti-Gravital demuestra (`ag-realtime`, `ag-observe`)
