# Fase 4 — Examples y Tests Cross-Module: Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar dos ejemplos completos (realtime-chat y ai-backend) y un test de integracion end-to-end que valida el stack completo con todos los modulos de Fase 4.

**Architecture:** Cada ejemplo es un proyecto Anti-Gravital completo con `schema.ag`, usando todos los modulos disponibles. Los tests cross-module levantan infra completa con testcontainers. Rama: `fase-4/examples`.

**Precondicion:** Todos los crates de Fase 4 mergeados a `fase-4` (ag-observe, ag-auth, ag-cache, ag-realtime, ag-storage).

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion Examples.
**Arquitectura:** `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` secciones 5-8.

---

## Mapa de archivos

### Crear
- `examples/realtime-chat/schema.ag`
- `examples/realtime-chat/src/main.rs`
- `examples/realtime-chat/src/handlers/mod.rs`
- `examples/realtime-chat/migrations/0001_messages.sql`
- `examples/realtime-chat/Cargo.toml`
- `examples/realtime-chat/README.md`
- `examples/ai-backend/schema.ag`
- `examples/ai-backend/src/main.rs`
- `examples/ai-backend/src/handlers/mod.rs`
- `examples/ai-backend/Cargo.toml`
- `examples/ai-backend/README.md`
- `tests/integration/fase4_e2e.rs` — test cross-module

### Modificar
- `Cargo.toml` (root) — agregar los dos ejemplos a `[workspace.members]`
- `docs/roadmap/STATUS.md` — marcar Fase 4 en curso
- `README.md` — actualizar estado

---

## Task 1: realtime-chat — schema.ag y migracion

**Files:** `examples/realtime-chat/schema.ag`, `examples/realtime-chat/migrations/0001_messages.sql`

- [ ] Escribir `schema.ag` con DSL v0.6:
  ```ag
  config { project_name "realtime-chat" database "postgres" }

  model Message {
      id         UUID      @primary @auto
      room_id    UUID      @references(Room.id) @on_delete(cascade)
      user_id    UUID      @references(User.id) @on_delete(cascade)
      content    String    @max(4096)
      created    DateTime  @auto
  }

  model Room {
      id      UUID   @primary @auto
      name    String @unique @max(100)
      created DateTime @auto
  }

  model User {
      id      UUID   @primary @auto
      name    String @max(100)
      created DateTime @auto
  }

  event message.sent {
      payload MessageResponse
      retain 7d
  }

  request SendMessageRequest { content String @max(4096) }
  response MessageResponse { id UUID content String user_id UUID created DateTime }

  endpoint SendMessage {
      method POST
      path /rooms/{room_id}/messages
      auth required
      body SendMessageRequest
      response MessageResponse
      events [message.sent]
  }

  endpoint GetMessages {
      method GET
      path /rooms/{room_id}/messages
      auth required
      response MessageResponse
  }
  ```
- [ ] Ejecutar: `ag schema lint examples/realtime-chat/schema.ag` — sin errores
- [ ] Ejecutar: `ag generate examples/realtime-chat/schema.ag` — genera artefactos
- [ ] Crear `migrations/0001_messages.sql` (puede reutilizar el SQL generado por DSL)
- [ ] Commit: `feat(example): realtime-chat schema.ag v0.6 + migracion`

---

## Task 2: realtime-chat — Cargo.toml y main.rs

**Files:** `examples/realtime-chat/Cargo.toml`, `examples/realtime-chat/src/main.rs`

- [ ] Crear `Cargo.toml` con deps: `ag-core`, `ag-data`, `ag-auth`, `ag-cache`, `ag-realtime`, `ag-observe`, `tokio`, `serde`, `axum`
- [ ] Implementar `main.rs`:
  ```rust
  #[tokio::main]
  async fn main() {
      ag_observe::init(ObserveConfig::from_env().unwrap()).unwrap();

      let pool = ag_data::connect(&DataConfig::from_env()).await.unwrap();
      let auth = AgAuth::new(AuthConfig::from_env().unwrap());
      let cache = AgCache::new(CacheConfig::from_env().unwrap()).await.unwrap();
      let realtime = AgRealtime::new(RealtimeConfig::from_env().unwrap()).await.unwrap();

      let state = AppState { pool, auth, cache, realtime };
      let router = build_router(state);

      let shield = Shield::from_config(ShieldConfig::default());
      let app = shield.apply(router);

      let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
      axum::serve(listener, app).await.unwrap();
  }
  ```
- [ ] Verificar que compila: `cargo build -p realtime-chat 2>&1 | grep "^error" | head -10`
- [ ] Commit: `feat(example): realtime-chat Cargo.toml + main.rs`

---

## Task 3: realtime-chat — handlers

**Files:** `examples/realtime-chat/src/handlers/mod.rs`

- [ ] Implementar handler `send_message`: lee del body, persiste en DB, emite evento `message.sent` via realtime, invalida cache de mensajes del room
- [ ] Implementar handler `get_messages`: busca en cache, si miss consulta DB y guarda en cache con tag `room:{room_id}`, retorna JSON
- [ ] Implementar handler `ws_chat`: monta el handler WebSocket de ag-realtime suscrito al subject `room.{room_id}`
- [ ] Test de integracion del ejemplo (requiere testcontainers PostgreSQL + NATS):
  - `send_message_broadcasts_to_ws_subscribers` — POST /rooms/1/messages, verificar que un cliente WebSocket conectado recibe el evento
- [ ] Correr test
- [ ] Commit: `feat(example): realtime-chat handlers — send, get, ws`

---

## Task 4: ai-backend — schema.ag y estructura

**Files:** `examples/ai-backend/schema.ag`, `examples/ai-backend/Cargo.toml`, `examples/ai-backend/src/main.rs`

- [ ] Escribir `schema.ag` con DSL v0.5+v0.6:
  ```ag
  config { project_name "ai-backend" database "postgres" }

  model Document {
      id           UUID     @primary @auto
      user_id      UUID
      storage_key  String   @max(512)
      filename     String   @max(255)
      created      DateTime @auto
  }

  event token.streamed {
      payload TokenPayload
  }

  request UploadDocumentRequest { filename String @max(255) }
  response DocumentResponse { id UUID filename String storage_key String }
  response TokenPayload { token String delta String }

  endpoint UploadDocument {
      method POST
      path /documents
      auth required
      body UploadDocumentRequest
      response DocumentResponse
  }

  endpoint StreamCompletion {
      method POST
      path /completions/stream
      auth required
      events [token.streamed]
  }
  ```
- [ ] Crear `Cargo.toml` con deps: `ag-core`, `ag-data`, `ag-auth`, `ag-storage`, `ag-realtime`, `ag-observe`
- [ ] Implementar `main.rs` siguiendo el patron de realtime-chat
- [ ] Verificar que compila
- [ ] Commit: `feat(example): ai-backend schema.ag + estructura`

---

## Task 5: ai-backend — handlers de upload y streaming SSE

**Files:** `examples/ai-backend/src/handlers/mod.rs`

- [ ] Implementar `upload_document`: recibe multipart, sube a ag-storage, persiste metadatos en DB, retorna `DocumentResponse`
- [ ] Implementar `stream_completion`: recibe el prompt, emite tokens simulados via SSE (sin LLM real en el ejemplo — simula un proveedor con texto de prueba), emite evento `token.streamed` por cada token, usa `ag_realtime::sse_handler`
- [ ] Test: `upload_document_stores_file_and_returns_key` — subir bytes de prueba, verificar que ag-storage los tiene y DB tiene el registro (testcontainers PostgreSQL + MinIO)
- [ ] Test: `stream_completion_emits_sse_events` — POST /completions/stream, consumir el stream SSE, verificar que llegan eventos con campo `data`
- [ ] Correr tests
- [ ] Commit: `feat(example): ai-backend handlers — upload + SSE streaming`

---

## Task 6: Test cross-module end-to-end

**Files:** `tests/integration/fase4_e2e.rs`

- [ ] Agregar `tests/integration/` al workspace si no existe, o usar el crate de tests existente
- [ ] TDD: test `e2e_authenticated_request_with_cache_and_event`:
  - Levanta testcontainers: PostgreSQL + Redis + NATS + MinIO
  - Inicia un servidor Anti-Gravital completo con todos los modulos en un `tokio::spawn`
  - Hace un POST autenticado (JWT valido generado con ag-auth)
  - Verifica que la respuesta es 200
  - Verifica que ag-cache tiene la entrada
  - Verifica que ag-realtime recibio el evento en NATS
  - Verifica que ag-observe registro el request en las metricas (`/metrics` contiene el counter)
- [ ] Correr para verificar que falla inicialmente (infra no levantada aun)
- [ ] Implementar el test paso a paso usando `testcontainers`
- [ ] Correr el test completo: `cargo test -p anti-gravital e2e_authenticated 2>&1 | tail -15`
- [ ] Commit: `test(e2e): test cross-module Fase 4 — auth+cache+realtime+observe`

---

## Task 7: Documentacion y cierre de Fase 4

- [ ] Escribir `examples/realtime-chat/README.md` con: descripcion, prerrequisitos, `ag dev`, endpoints disponibles
- [ ] Escribir `examples/ai-backend/README.md` con: descripcion, prerrequisitos, endpoints, como probar el streaming
- [ ] Actualizar `docs/roadmap/STATUS.md`: marcar todos los entregables de Fase 4 como `[x]`
- [ ] Actualizar `README.md` root: estado de Fase 4 (completada), nuevos comandos disponibles, enlace a los ejemplos
- [ ] `cargo fmt --all && cargo clippy --workspace -- -D warnings`
- [ ] `cargo test --workspace 2>&1 | tail -15`
- [ ] Crear `docs/pr-drafts/fase-4.md` con descriptor del PR final
- [ ] Commit: `docs(fase4): README ejemplos, STATUS.md, README root`
- [ ] Merge rama al integradora: `git checkout fase-4 && git merge --no-ff fase-4/examples -m "feat(examples): realtime-chat + ai-backend + test e2e cross-module"`
- [ ] Merge fase-4 a main: `git checkout main && git merge --no-ff fase-4 -m "feat: Fase 4 completa — ag-auth/cache/realtime/storage/observe + DSL v0.5+v0.6"`
