# Fase 4 — Modulos Estandar: Diseno Completo

**Fecha:** 2026-05-22
**Autor:** Angel Nereira
**Estado:** Aprobado
**Rama integradora:** `fase-4`

---

## Contexto

Fase 3 entrego DSL v0.1–v0.4, ag-lsp, plugin VS Code y el harness de fuzzing. La
implementacion tecnica esta completa. Los criterios externos de comunidad (stars,
instalaciones, colaboradores) se cumplen en paralelo con el tiempo.

Fase 4 completa los modulos batteries-included: auth, realtime, cache, storage y
observe. Cada uno es un crate independiente con API publica, tests, documentacion
y ejemplos. Al final de la fase, los cinco crates se publican en crates.io.

---

## Alcance

- DSL v0.5: declaracion de auth y politicas RBAC en endpoints
- DSL v0.6: declaracion de eventos emitidos por endpoint
- `ag-auth`: WebAuthn, JWT Ed25519, OAuth2, API keys, refresh tokens con rotacion
- `ag-cache`: moka L1 + Redis L2 con fred, invalidacion por tags
- `ag-realtime`: WebSocket binario, SSE fallback, NATS embebido/externo
- `ag-storage`: adaptadores S3, MinIO, filesystem local, URLs firmadas, procesamiento imagen
- `ag-observe`: tracing, OpenTelemetry exporter, Prometheus, dashboards Grafana, tokio-console
- `examples/realtime-chat/`: chat completo con todos los modulos
- `examples/ai-backend/`: backend IA con streaming SSE
- Tests cross-module end-to-end con testcontainers

**Fuera de alcance:** ag-ui, ag-cloud, ag-ai, ag-mobile, ag-migrate, ag-wasm-host.

---

## Estructura de ramas

La rama `fase-4` es la rama integradora. Cada modulo tiene su propia rama feature.

```
main
 └── fase-4
       ├── fase-4/dsl-v05-v06
       ├── fase-4/ag-observe
       ├── fase-4/ag-auth
       ├── fase-4/ag-cache
       ├── fase-4/ag-realtime
       ├── fase-4/ag-storage
       └── fase-4/examples
```

**Orden de merge a `fase-4`:**

1. `fase-4/dsl-v05-v06` — primero; ag-auth y ag-realtime dependen del schema declarativo
2. `fase-4/ag-observe` — segundo; todos los crates posteriores instrumentan con tracing
3. `fase-4/ag-auth` — tercero; bloquea los casos de uso mas comunes
4. `fase-4/ag-cache` y `fase-4/ag-realtime` — cuarto, en paralelo (independientes entre si)
5. `fase-4/ag-storage` — quinto
6. `fase-4/examples` — ultimo; consume todos los modulos anteriores

Cada rama tiene su propio sub-plan bajo `docs/superpowers/plans/`.

---

## Diseno por modulo

### DSL v0.5 — Autenticacion y politicas RBAC

**Nuevas construcciones gramaticales:**

```ag
endpoint CreateUser {
    method   POST
    path     /users
    auth     optional     # opcional | required | none (default: none)
    policy   "user.role == ADMIN || user.id == params.id"
    ...
}
```

**Codegen:**
- Rust: activa `AuthLayer` en el handler generado cuando `auth required`
- OpenAPI: genera `securitySchemes` + campo `security` por endpoint
- SQL: sin cambios (politicas son runtime)

**Analisis semantico nuevo:**
- `policy` solo valido cuando `auth required`
- Expresiones de politica referencian solo `user.*` (claims JWT) o `params.*` (path params)
- Error si campo referenciado no existe en el modelo de claims

**Tests:** nuevos tests en la suite existente de ag-dsl. Cobertura objetivo: mantener >= 85%.

---

### DSL v0.6 — Eventos

**Nuevas construcciones gramaticales:**

```ag
event user.created {
    payload UserResponse
    retain  30d
}

endpoint CreateUser {
    ...
    events [user.created]
}
```

**Codegen:**
- Rust: stub de emision en el handler (`state.events.emit("user.created", &payload).await?`)
- AsyncAPI 2.6: spec completa del bus de eventos
- TypeScript: tipos del payload de cada evento

**Analisis semantico nuevo:**
- Todos los eventos en `events [...]` deben estar declarados en el schema
- El `payload` de cada evento debe ser un tipo definido (model, request o response)
- Nombres de eventos en formato `entidad.accion` (validacion con regex)

**Dependencias nuevas en ag-dsl:** ninguna. Extension pura de logos + chumsky + codegen existente.

---

### `ag-observe`

**API publica:**

```rust
ag_observe::init(ObserveConfig::from_env()?)?;
```

**Componentes:**

`ObserveConfig` — configuracion desde env:
- `OTEL_EXPORTER_OTLP_ENDPOINT` (traces)
- `PROMETHEUS_PORT` (default 9090)
- `LOG_FORMAT` = `json` | `pretty`
- `AG_DEV_CONSOLE` = `true` | `false` (tokio-console)

`init()` — configura el subscriber global de `tracing` con tres layers:
1. `fmt::Layer` — logs estructurados JSON a stdout
2. `OpenTelemetryLayer` — spans OTLP
3. `MetricsLayer` — histogramas y contadores Prometheus

**Metricas estandar incluidas:**
- `ag_request_duration_seconds{endpoint, method, status}` (histograma)
- `ag_requests_total{endpoint, method, status}` (counter)
- `ag_active_connections` (gauge)
- `ag_db_pool_size`, `ag_db_pool_idle` (gauge, alimentadas por ag-data)

**Dashboards Grafana:** JSON pre-configurados en `crates/ag-observe/dashboards/`:
- `overview.json` — latencia p50/p95/p99, throughput, errores
- `database.json` — pool size, query duration
- `runtime.json` — tareas Tokio

**tokio-console:** activado con feature `dev-console`. No incluido en binario release.

**Dependencias nuevas al workspace:**
- `opentelemetry`, `opentelemetry-otlp`, `opentelemetry_sdk`
- `tracing-opentelemetry`
- `metrics`, `metrics-exporter-prometheus`
- `console-subscriber` (feature-gated `dev-console`)

**Tests:** unit tests para config parsing + tests de integracion que verifican que
`/metrics` devuelve contadores correctos tras N requests. Sin testcontainers.

---

### `ag-auth`

**API publica:**

```rust
let auth = AgAuth::new(AuthConfig::from_env()?);
state.auth.verify_jwt(&token)?;
state.auth.create_session(&user_id).await?;
state.auth.revoke_token(&jti).await?;
```

**Flujos soportados:**

| Flujo | Crate base | Notas |
|---|---|---|
| JWT Ed25519 | `jsonwebtoken` + `ring` | Firma/verificacion. Reutiliza primitivas de ag-core |
| Passkeys / WebAuthn | `webauthn-rs` | Registro y autenticacion FIDO2 |
| OAuth2 (Google, GitHub) | `oauth2` | Providers preconfigurados |
| API keys | custom sobre `ring` | Hash BLAKE3, almacenadas en DB via ag-data |
| Refresh tokens con rotacion | custom | Rotacion en cada uso, revocacion por JTI |

**Integracion con ag-data:** feature-gated (`features = ["persistent"]`). Sin esa feature,
solo JWT stateless. Con la feature, persiste sesiones, passkey credentials y API keys.

**Migraciones SQL propias:** `crates/ag-auth/migrations/` contiene:
- `0001_ag_sessions.sql`
- `0002_ag_api_keys.sql`
- `0003_ag_webauthn_credentials.sql`

**Integracion con DSL v0.5:** ag-auth provee la clave publica que la Shield usa para
verificar JWTs. No hay dependencia directa entre ag-dsl y ag-auth.

**Integracion con ag-observe:** cada verificacion fallida genera un span y un counter
`ag_auth_failures_total{reason}`.

**Tests:**
- Unitarios: JWT (sin infra), WebAuthn (mocks de ceremonia FIDO2)
- Integracion con testcontainers (PostgreSQL): API keys, refresh tokens, persistencia de sesiones

**Dependencias nuevas:** `webauthn-rs`, `oauth2`, `blake3`, `base64ct`.

---

### `ag-cache`

**API publica:**

```rust
let cache = AgCache::new(CacheConfig::from_env()?);
cache.set("user:123", &user, Duration::from_secs(300)).await?;
let user: Option<User> = cache.get("user:123").await?;
cache.invalidate_tag("users").await?;
```

**Niveles:**

L1 — `moka` (TinyLFU, en memoria, sin locks contenciosos). Siempre disponible.

L2 — Redis via `fred` (async, connection pooling, cluster-ready). Opcional: si Redis
no esta configurado, el crate funciona solo con L1.

**Trait `CacheBackend`:** definido en ag-core para que ag-core no dependa de ag-cache.
ag-cache implementa el trait.

**Invalidacion por tags:** cada entrada puede tener uno o mas tags. `invalidate_tag("users")`
invalida todas las entradas con ese tag en ambos niveles.

**Tests:**
- L1: unitarios puros
- L2 + invalidacion distribuida: testcontainers (Redis)

**Dependencias nuevas:** `moka`, `fred`.

---

### `ag-realtime`

**API publica:**

```rust
let rt = AgRealtime::new(RealtimeConfig::from_env()?);
rt.on_connect(|conn| async { ... });
rt.broadcast("user.created", &payload).await?;
rt.sse_stream("events").filter(|e| e.user_id == claims.sub)
```

**Modalidades:**
- WebSocket binario con msgpack (overhead menor que JSON)
- SSE fallback automatico para clientes sin soporte WebSocket
- Bus pub/sub con NATS

**NATS embebido vs externo:** `nats_mode = embedded | external` en `RealtimeConfig`.
En modo embedded, el binario levanta un servidor NATS en memoria. En modo external,
conecta a cluster NATS externo. El codigo de aplicacion no cambia entre modos.

**Integracion con DSL v0.6:** los eventos declarados en el schema se emiten al bus
interno via ag-realtime. Los handlers generados por DSL reciben el stub de emision.

**Integracion con ag-observe:** counter `ag_realtime_messages_total{channel, direction}`
y gauge `ag_realtime_active_connections`.

**Tests:**
- WebSocket y SSE: servidor en memoria (sin testcontainers)
- NATS pub/sub distribuido: testcontainers (NATS)

**Dependencias nuevas:** `async-nats`, `rmp-serde`.

---

### `ag-storage`

**API publica:**

```rust
let storage = AgStorage::new(StorageConfig::from_env()?);
storage.put("avatars/user-123.webp", bytes, ContentType::WebP).await?;
let url = storage.signed_url("avatars/user-123.webp", Duration::from_secs(900)).await?;
storage.put_image("avatars/user-123", bytes, ImagePolicy {
    formats: &[WebP, Avif],
    sizes: &[(128, 128), (512, 512)],
}).await?;
```

**Backends:**
- S3/AWS: via `aws-sdk-s3`
- MinIO: mismo protocolo S3, endpoint configurable via `STORAGE_ENDPOINT`
- Filesystem local: para dev y tests sin infra

**Procesamiento de imagenes:** crate `image` para resize, compresion y conversion
(JPEG, PNG, WebP, AVIF). Thumbnails generados automaticamente en upload si la
politica esta activa en `StorageConfig`.

**URLs firmadas:** HMAC-SHA256 con expiracion. Para S3/MinIO se delega al SDK.
Para filesystem local se implementa custom (util en tests).

**Tests:**
- Filesystem local: unitarios puros
- S3-compatible: testcontainers (MinIO)

**Dependencias nuevas:** `aws-sdk-s3`, `image`.

---

### Examples

**`examples/realtime-chat/`**

Demuestra integracion completa:
- WebSocket binario con ag-realtime (mensajes en tiempo real)
- Auth con JWT via ag-auth (registro, login, token refresh)
- Historial de mensajes en PostgreSQL via ag-data
- Cache de ultimos 50 mensajes con ag-cache
- Schema completo en `schema.ag` con DSL v0.6 (evento `message.sent`)
- Observabilidad con ag-observe (`/metrics`, traces OTLP)

**`examples/ai-backend/`**

Demuestra:
- Streaming SSE de tokens a frontend via ag-realtime
- Almacenamiento de archivos de contexto con ag-storage (S3/MinIO)
- API keys para autenticacion servidor-servidor con ag-auth
- Observabilidad completa: latencia de stream, tokens por segundo

**Tests cross-module (en `fase-4/examples`):**

Test de integracion end-to-end que levanta el stack completo:
Shield -> Core -> ag-auth -> ag-data -> ag-cache -> ag-realtime -> ag-observe.
Requiere testcontainers (PostgreSQL + Redis + NATS + MinIO).
Verifica que un request autenticado, cacheado y emitido como evento funciona
de extremo a extremo.

---

## Dependencias nuevas al workspace (resumen)

| Crate | Dependencia nueva |
|---|---|
| ag-observe | opentelemetry, opentelemetry-otlp, opentelemetry_sdk, tracing-opentelemetry, metrics, metrics-exporter-prometheus, console-subscriber |
| ag-auth | webauthn-rs, oauth2, blake3, base64ct |
| ag-cache | moka, fred |
| ag-realtime | async-nats, rmp-serde |
| ag-storage | aws-sdk-s3, image |
| (todos los tests) | testcontainers |

Todas las dependencias nuevas se justifican por madurez, mantenimiento activo,
seguridad auditada y necesidad real. Ninguna es especulativa.

---

## Criterios de salida de Fase 4 (de la Hoja de Ruta)

- [ ] Los cinco modulos publicados en crates.io con releases independientes
- [ ] Cobertura de tests >= 80% en cada modulo
- [ ] Documentacion por modulo: README, guia de uso, referencia de API
- [ ] ag-realtime: 50K conexiones WebSocket en instancia 2 vCPU sin degradacion
- [ ] ag-cache: >= 1M ops/segundo en L1
- [ ] Al menos cinco issues bug reports cerrados por la comunidad
- [ ] Al menos 500 stars en el repositorio

---

## Riesgos y mitigaciones

**Fragmentacion del esfuerzo entre cinco modulos en paralelo.**
Mitigacion: el orden de merge garantiza que ag-observe este disponible antes de
implementar los otros. Cada modulo tiene su gate de calidad antes de integrarse.

**Dependencias externas (NATS, Redis, MinIO) en CI.**
Mitigacion: testcontainers-rs levanta las dependencias automaticamente. Sin infra
manual en CI. Los tests sin infra estan separados de los que requieren contenedores.

**Complejidad de WebAuthn.**
Mitigacion: usar webauthn-rs (crate mantenido por William Brown, contribuidor de
Fedora/SSSD con experiencia en FIDO2). No rodar criptografia propia.

**Scope creep dentro de los modulos.**
Mitigacion: cada crate implementa exactamente lo declarado en este spec.
Cualquier feature adicional requiere RFC antes de implementarse.
