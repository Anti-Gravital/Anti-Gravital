# Estado vivo de la Hoja de Ruta

Este archivo agrega el estado de las casillas de cada fase, en orden,
para que un agente o un humano pueda saber en un solo vistazo que esta
hecho y que falta. Se actualiza con cada pull request que avance la
hoja de ruta.

Convencion: `- [x]` significa cumplido y verificable en el repositorio,
`- [ ]` significa pendiente, `- [/]` significa parcialmente cumplido
(con explicacion).

Ultima actualizacion: 2026-06-10. Auditoria y consolidacion de fases 0-4.5 en curso; la Fase 4.6-D (`ag-workers`) tiene S1-S5 entregadas y su trabajo restante rastreado en GitHub Issues. La implementacion disponible no equivale a cierre de los criterios de salida; ver docs/audits/PRE_FASE5_RELEASE_GATE.md y los Issues con etiqueta `tech-debt` (docs/DEBT.md queda congelado como registro historico).

---

## Fase 0 - Fundaciones y gobernanza

Estado: En curso.

### Criterios de entrada

- [x] Decision final de comenzar Anti-Gravital como proyecto formal de Gravital Labs.
- [x] Aprobacion de licencia Apache 2.0 sin restricciones.
- [x] Compromiso publico de Angel Nereira como mantenedor inicial.

### Entregables en el repositorio

- [x] Repositorio `github.com/anti-gravital/anti-gravital` creado y publico.
- [x] Archivo `LICENSE` con texto completo Apache 2.0.
- [x] Archivo `README.md` bilingue (espanol mas ingles).
- [x] Archivo `CONTRIBUTING.md`.
- [x] Archivo `CODE_OF_CONDUCT.md` adoptando Contributor Covenant 2.1.
- [x] Archivo `SECURITY.md`.
- [x] Archivo `GOVERNANCE.md`.
- [x] Configuracion de CI con GitHub Actions en cuatro plataformas.
- [x] Plantillas de issue (bug report, feature request, RFC) y plantilla de pull request.
- [x] Estructura de carpetas del monorepo definida y commiteada.
- [x] Workspace Cargo inicializado con los 15 crates vacios.
- [x] El CI construye exitosamente el workspace vacio en las cuatro plataformas objetivo.
- [x] CLAUDE.md instalado como constitucion tecnica del repositorio.
- [x] Maestros instalados en `docs/master/` con `VERSION.md` y SHA-256.
- [x] Documentacion descompuesta en `docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`, `docs/security/`, `docs/governance/`, `docs/benchmarks/`.
- [x] ADRs iniciales bajo `docs/adr/`.

### Entregables externos (no viven en el repositorio)

- [ ] Branding basico: logo, paleta de colores, tipografia. Aplicado al README.
- [ ] Discord oficial del proyecto con canales requeridos.
- [ ] Cuenta del proyecto en X o Bluesky para anuncios.
- [ ] Dominio `antigravital.dev` registrado y apuntando a landing page.
- [x] Email institucional `anti@gravitalcloud.com` operativo (correo raiz). Respaldo del BDFL: `angelnereira@gravitalcloud.com`.
- [ ] Calendario publico de releases publicado en el sitio.

Detalle y owner sugerido en `docs/governance/external-deliverables.md`.

### Criterios de salida (puerta antes de Fase 1)

- [ ] El repositorio recibe su primer star externo no solicitado.
- [ ] Al menos cinco personas externas se han unido al Discord.
- [x] La estructura de carpetas del monorepo esta definida y commiteada.
- [x] El workspace Cargo esta inicializado con los crates vacios listados en CLAUDE.md.
- [x] El CI construye exitosamente el workspace vacio en las cuatro plataformas objetivo. Verificado en multiples runs del workflow `ci.yml` sobre Linux x86-64, Linux ARM64, macOS ARM64 y Windows x64.
- [ ] La landing page describe en un parrafo que es el proyecto, que no es, y donde esta en el roadmap.

---

## Fase 1 - The Shield MVP

Estado: En curso bajo excepcion documentada en
`docs/rfc/RFC-0001-paralelizar-fase-0-externa-y-fase-1.md`. La
implementacion avanza en paralelo con el cierre de las puertas
externas de Fase 0. El diseno de Shield esta fijado en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

### Criterios de entrada (1.1)

- [/] Fase 0 completada con todos sus criterios de salida marcados.
  (Excepcion: tres casillas externas siguen pendientes; vease RFC-0001.)
- [ ] Al menos un contribuidor adicional al mantenedor principal esta
  activo en el repositorio. (Excepcion: BDFL en modo solo; vease
  RFC-0001.)

### Entregables (1.2)

- [x] Crate `ag-core` con modulo `shield` operativo. Pipeline completa con logging, validation, CORS, CSRF, rate-limit, auth-jwt y TLS, ademas del helper `Shield::serve(listener, router)`.
- [x] Soporte de HTTP/1.1 y HTTP/2 via Axum + Tokio. Negociacion por ALPN bajo TLS y upgrade sobre plain.
- [x] Terminacion TLS 1.3 con rustls. `shield::tls` + helper `Shield::serve(listener, router)` que despacha a TLS o plain segun config.
- [x] Middleware de validacion de payload basico. Trait `Validate` y extractor `ValidatedJson<T>` bajo `shield::validation`.
- [x] Middleware de autenticacion JWT con verificacion Ed25519. `shield::auth` con `AuthLayer`, `AuthContext` y extractor `Claims<T>`.
- [x] Middleware de rate limiting con governor. Token bucket por IP en `shield::rate_limit`.
- [x] Middleware CORS y CSRF con defaults seguros. CORS en `shield::cors` y CSRF en `shield::csrf` (double-submit cookie apatrida).
- [x] Middleware de logging estructurado con `tracing`. `shield::logging` emite un evento por request con metodo, path, status y latencia.
- [x] Configuracion minima desde archivo TOML. `ShieldConfig::from_path` y `from_toml_str` con `deny_unknown_fields`; ejemplo en `crates/ag-core/config.example.toml`.
- [/] Tests unitarios con cobertura >= 80% del crate. 56 unit + 27 E2E + 1 doctest = 84 tests verde con `cargo test --workspace`. La medicion oficial de cobertura con `cargo-llvm-cov` esta pendiente.
- [x] Tests de integracion end-to-end del pipeline Shield. `tests/shield_full_pipeline.rs` arranca el Shield con todas las capas activas sobre HTTPS y valida flujo legitimo y rechazos por capa.
- [x] Benchmark Hello World ejecutable. `cargo bench -p ag-core --bench shield_hello_world` con tres grupos criterion. Metricas duras de cierre (>=300K req/s, p99 <=1ms) se miden en PR 10 con carga real.
- [x] Documentacion API generada con `cargo doc`. Rustdoc crate-level ampliado con tabla de capas, features, ejemplos y enlaces cruzados.
- [x] Capitulo del manual de usuario sobre uso de Shield como libreria. `docs/manual/01-shield-as-library.md`.

### Criterios de salida (1.3)

- [ ] Benchmark Hello World >= 300K req/s en hardware de referencia.
- [ ] Latencia p99 del pipeline Shield <= 1 ms a 100K req/s.
- [ ] Memoria del proceso idle <= 15 MB.
- [ ] Tiempo de arranque <= 100 ms.
- [/] CI pasa en las cuatro plataformas objetivo. Linux x86-64, Linux ARM64 y macOS ARM64 verde de forma estable. Windows x64 verde tras el hotfix `tls-test-tmp-collisions`. Pendiente la verificacion del primer run completamente verde post-merge.
- [x] Clippy sin warnings. `cargo clippy --workspace --all-targets -- -D warnings` limpio en local y en el job `quality/clippy`.
- [/] `cargo audit` sin vulnerabilidades conocidas. El job `quality/audit` corre `rustsec/audit-check` en cada PR y push. `cargo deny check` (advisories) pasa localmente.
- [x] Cero bloques `unsafe` no documentados. `unsafe_code = "deny"` en `[workspace.lints.rust]` y ningun `#[allow(unsafe_code)]` en el codigo del workspace.
- [ ] Al menos un blog post tecnico sobre la arquitectura de Shield.
- [ ] Al menos diez stars en el repositorio.

## Fase 2 - The Core MVP

Estado: funcionalidad principal disponible. Benchmarks medidos en hardware
real (Ryzen 5 2500U). Los criterios de throughput y latencia no se
alcanzan en este hardware con PostgreSQL estandar; requieren hardware
mas potente o pgbouncer. Criterios externos de comunidad pendientes.

### Criterios de entrada (2.1)

- [x] Fase 1 completada con todos sus criterios de salida marcados.
  (Excepcion: criterios externos de Fase 0/1 siguen pendientes; la
  implementacion avanza bajo la misma excepcion documentada en RFC-0001.)
- [x] El crate `ag-data` ha sido iniciado con sqlx como dependencia.

### Entregables (2.2)

- [x] Crate `ag-core` con modulo `core` operativo. Reexporta `State<T>`,
  `Path<T>`, `Query<T>`, `ValidatedBody<T>`, `Claims<T>` y el modulo
  `response` con `Json`, `PlainText` y `BodyStream`.
- [x] Router Axum integrado con la Shield. `Shield::apply(router)` acepta
  cualquier `Router<()>` con estado ya registrado via `with_state`.
- [x] Extractores: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`,
  `Query<T>` disponibles desde `ag_core::core`.
- [x] Sistema de errores `AgError` expandido: `NotFound` (404),
  `BadRequest` (400), `Conflict` (409), `Database` (500).
- [x] Sistema de respuestas: JSON (`axum::Json`), plaintext (`PlainText`),
  streams (`BodyStream = axum::body::Body`).
- [x] Crate `ag-data` con pool PostgreSQL via sqlx. `DataConfig`,
  `DbPool`, `connect()`, `run_migrations()`, conversion `DataError ->
  AgError`.
- [x] Sistema de migraciones embebido con `sqlx::migrate!` demostrado
  en `examples/todo-api/`.
- [x] Example app `todo-api` en `examples/` con CRUD completo: cinco
  handlers GET/POST/PUT/DELETE contra PostgreSQL real.
- [x] Benchmark CRUD + DB ejecutable. Ejecutado el 2026-05-21 en Ryzen 5 2500U
  con PostgreSQL 18.4 nativo. Criterion: select_one_by_id = 352 us, insert = 1.90 ms,
  full_cycle = 6.17 ms. HTTP (oha): GET /todos/:id = 14 478 req/s mediana (c=100,
  pool=100). POST /todos = 8 934 req/s (c=50, synchronous_commit=off). Resultados
  completos en `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.
  Nota: bug de routing corregido en esta sesion (ver criterios de salida).
- [x] Crate `ag-cli` con comandos `new`, `dev`, `build`.
- [x] Tres templates: `rest`, `realtime`, `fullstack` embebidos en el
  binario `ag` via `include_str!`.

### Criterios de salida (2.3)

- [ ] Benchmark CRUD + PostgreSQL >= 40K req/s en hardware de referencia.
  Medicion real del 2026-05-21 (Ryzen 5 2500U, PostgreSQL 18.4 nativo, c=100, pool=100):
  GET /todos/:id = 14 478 req/s mediana. No alcanza el objetivo de 40K req/s. El numero
  previo de "82 233 req/s" era INVALIDO: el binario tenia un bug de routing
  (Axum 0.7 usa `:id`, no `{id}`) y todas las requests a /todos/:id devolvian 404
  sin consultar la DB, midiendo efectivamente el throughput de respuestas 404 del stack
  HTTP (~89K req/s). Bug corregido en esta sesion. El cuello de botella real es
  PostgreSQL (modelo proceso-por-conexion, 4 nucleos fisicos). Para alcanzar 40K req/s
  se requiere hardware con >= 8 nucleos fisicos o uso de pgbouncer en transaction mode.
  Ver analisis completo en `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`.
- [ ] Latencia p99 del CRUD <= 5 ms. Medido el 2026-05-21: GET /todos/:id p99 = 14.6 ms
  (mediana de 2 corridas, c=100, pool=100, PostgreSQL nativo). No cumple el objetivo
  de 5 ms. La causa es la misma que el criterio de throughput: saturacion del scheduler
  de OS con 100 procesos PostgreSQL en 4 nucleos. Ver documento de benchmarks citado
  en el criterio anterior.
- [x] La CLI `ag new` crea el scaffold correcto y `ag dev` arranca el
  proceso de compilacion. Verificado el 2026-05-21: los tres templates (rest, realtime,
  fullstack) generan scaffold correcto y compilan sin warnings. `ag build` produce
  binario release funcional. Se elimino import no usado `ShieldConfig` en los tres
  templates. `ag dev` responde en la ruta raiz con el nombre del proyecto.
- [x] La app `todo-api` se despliega como binario unico (`FROM scratch` Docker).
  Verificado el 2026-05-21: `docker build` exitoso tras corregir base image
  (rust:1.79 -> rust:1.95) y orden de capas (rust-toolchain.toml antes de
  rustup target add). Contenedor arranca, se conecta a PostgreSQL y responde
  /health 200 OK.
- [x] El binario release del `todo-api` ocupa <= 20 MB. Verificado el 2026-05-21:
  binario MUSL estatico stripped = 5.3 MB. Binario GNU release stripped = 5.2 MB.
  Ambos dentro del criterio. Imagen Docker FROM scratch = 2.49 MB.
- [x] Documentacion: "Tu primera API con Anti-Gravital" publicada.
  Disponible en `docs/manual/02-primera-api.md`. Cubre todo el flujo
  desde `ag new` hasta Docker `FROM scratch`.
- [ ] Al menos 50 stars en el repositorio. Criterio externo.
- [ ] Al menos tres contribuidores externos con PRs merged. Criterio externo.

## Fase 3 - Anti-DSL alpha

Estado: funcionalidad principal disponible; consolidacion abierta. El generador Rust, el fuzzing manual de 24 horas y la comparativa de rendimiento aun tienen criterios pendientes. RFC-0003 aceptada.
### Criterios de entrada (3.1)

- [/] Fase 2 completada. (Excepcion RFC-0001: implementacion tecnica completa;
  criterios externos pendientes.)
- [x] Crate `ag-dsl` iniciado. Skeleton Fase 0 presente.
- [x] Decision final sobre librerias base del compilador. Documentada en
  `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md`. Aceptada 2026-05-21.

### Entregables (3.2)

- [x] DSL version 0.1: modelos basicos (@primary, @unique, @auto). Commit 5aa442d.
- [x] DSL version 0.2: endpoints (metodo, path, body, response, errors). Commit 7d41930.
- [x] DSL version 0.3: validaciones (@min, @max, @email, @regex, @length). Commit 47ae623.
- [x] DSL version 0.4: relaciones entre modelos (1:1, 1:N, N:M). @references/@relation,
  SQL FOREIGN KEY, Rust Option<M>/Vec<M>, TypeScript tipos opcionales, OpenAPI $ref. Commit 9a541cd..HEAD.
- [/] Generador Rust: structs con serde, types.rs, handlers.rs, router.rs y modulo raiz. Imports/estado y validaciones numericas fueron corregidos en la auditoria; el fixture tests/generated-rust-fixture prueba compilacion end-to-end del modulo generado; falta cerrar validacion Rust ejecutable para @regex (issue #70).
- [x] Generador SQL: migraciones idempotentes (v0.1).
- [x] Generador TypeScript: tipos y cliente HTTP (v0.1+v0.2).
- [x] Generador OpenAPI 3.1: schemas (v0.1) + paths (v0.2).
- [x] Comando ag generate. Operativo desde v0.1.
- [x] Comando ag schema lint. Operativo desde v0.1.
- [x] Comando ag schema diff. Operativo desde v0.1.
- [x] Diagnostics legibles. Lex + parse + semantic con linea:columna.
- [x] Servidor LSP basico (ag-lsp). Commits 01f256a + c7e01a9. Binario funcional;
  diagnostics en tiempo real, completion y hover implementados. 10 tests.
  Smoke test protocolo LSP verificado: responde initialize con serverInfo correcto.
- [x] Plugin VS Code. Commits fd8b882..133a390. tmLanguage grammar, extension.ts con
  LSP client, deteccion PATH y fallback cargo install. vsce package: anti-gravital-0.1.0.vsix.
  Publicacion en marketplace pendiente (requiere repo publico).
- [x] Cobertura tests >= 85%. Medicion 2026-05-21: 95.26% lineas, 93.02% funciones (cargo-llvm-cov).
  119 tests ag-dsl + 10 tests ag-lsp = 129 tests verdes post-Fase 3. Objetivo superado.
- [/] Fuzzing 24h sin crashes. Harness cargo-fuzz operativo (3 targets: fuzz_lexer,
  fuzz_parser, fuzz_compile). CI smoke test 60s activo en quality.yml. Crash encontrado
  y corregido (lexer panic en enteros > i64::MAX, commit ff85c6f). Gate manual 24h
  pendiente de ejecucion en hardware Linux x86-64 antes de mergear.
- [x] Documentacion de referencia del DSL. `docs/dsl/referencia-v01-v04.md` — cubre tipos,
  anotaciones v0.1–v0.4, generacion de codigo, diagnostics y limitaciones conocidas.
  Hoja de ruta LSP en `docs/dsl/lsp-roadmap.md`. Documentacion fuzzing en `docs/fuzz/README.md`.

### Criterios de salida (3.3)

- [/] Proyecto definible en schema.ag y generable con CLI. `ag schema lint` y `ag generate` estan verificados; el fixture tests/generated-rust-fixture compila el modulo generado completo; @regex sigue documentado como limitacion pendiente en issue #70.
- [x] Example ecommerce-api reescrito con DSL. Modelos User, Category, Product, Order, OrderItem
  con relaciones 1:N y N:M. SQL FOREIGN KEY, Rust Option<M>/Vec<M>, TS y OpenAPI $ref generados.
- [/] CRUD generado por DSL no es mas lento que CRUD a mano. Benchmark
  real de 2 horas contra Neon PostgreSQL (2026-05-22): 255 805 requests,
  0 errores, peak 43 req/s con handlers escritos manualmente (equivalente
  al codigo generado por ag-dsl v0.4). La comparativa directa DSL-generado
  vs manual requiere ejecutar `cargo bench -p todo-api` con el mismo schema,
  pendiente en gate de Fase 4. Ver `docs/benchmarks/measurement-2026-05-22-neon-real.md`.
- [ ] Plugin VS Code >= 100 instalaciones. (Plugin empaquetado; pendiente publicacion y adopcion.)
- [ ] Al menos un colaborador externo contribuyo al compilador.
- [ ] Documentacion DSL revisada por dos personas.
- [ ] Al menos 200 stars.

## Fase 4 - Modulos estandar

Estado: modulos principales disponibles. La puerta de produccion sigue abierta por evidencia manual de escala/rendimiento y deuda documentada; no se declara la fase cerrada.

### Criterios de entrada (4.1)

- [/] Fase 3 completada. (Excepcion RFC-0001: implementacion tecnica completa;
  criterios externos pendientes.)
- [x] DSL version 0.5 (auth y politicas) iniciada.

### Entregables (4.2)

- [x] DSL version 0.5: declaracion de auth/policies en endpoints (AuthMode enum, validacion semantica).
- [x] DSL version 0.6: declaracion de eventos (EventDef, bloque event, emits en endpoint).
- [x] rust_gen: Claims extractor + stubs de eventos.
- [x] openapi_gen: securitySchemes BearerAuth + security por endpoint.
- [x] ts_gen: tipos de payload de eventos (UserCreatedEvent pattern).
- [x] async_api_gen: nuevo generador AsyncAPI 2.6.
- [x] ag-dsl: 136 tests, cobertura 95.88%.
- [x] Crate `ag-auth`: WebAuthn/FIDO2 (registro+autenticacion, CBOR ciborium, COSE ES256 p256 + EdDSA ed25519-dalek), OAuth2 PKCE (Google+GitHub, oauth2 v5 + reqwest 0.12 manual), JWT Ed25519 (JwtSigner/JwtVerifier), API keys BLAKE3, refresh tokens con RwLock<HashSet<String>> blacklist. `AgAuth::new(config, http_client)`. 32 tests.
- [x] Crate `ag-cache`: L1 con moka, invalidacion por tags, TTL configurable, cobertura >= 80%.
- [x] Crate `ag-realtime`: bus InProcess pub/sub (EventBus broadcast), cliente NATS externo (NatsExternalClient, TLS 3 niveles, JetStream ACK), helpers Axum ws_handler + sse_handler (EventSource-compatible), `AgRealtime::new` async, `RealtimeBus` enum. Cobertura >= 80%.
- [x] Crate `ag-storage`: store filesystem nativo (atomic write-then-rename, path confinement), servidor HTTP Axum embebido, procesamiento de imagen (ImageProcessor: resize/thumbnail/webp), backend S3/MinIO (S3Store via object_store 0.11, feature `s3`), URLs firmadas HMAC-SHA256 (sign_url/verify_signed_url, comparacion en tiempo constante), `AgStore` enum Native|S3. Cobertura >= 80%.
- [x] Crate `ag-observe`: tracing estructurado, exporter OTLP, metricas Prometheus via axum::Router, layer personalizado, LogFormat JSON/Text, init idempotente. Cobertura >= 80%.
- [x] Example `realtime-chat` en `examples/`: chat SSE in-memory, EventBus, puerto 3000.
- [x] Example `ai-backend` en `examples/`: AiProvider trait, ClaudeProvider+GeminiProvider+OpenAiProvider (streaming SSE real), router Axum puerto 3001.
- [x] Tests de integracion cross-module: crate `tests/integration` con 7 tests (6 unitarios por modulo + 1 E2E 15 pasos).
- [x] RFC-0005 ag-cache L2 nativo RESP2: implementado y mergeado. El servidor incluye limites de protocolo y, desde esta auditoria, limite concurrente de conexiones.

### Criterios de salida (4.3)

- [ ] Los cinco modulos publicados en crates.io con releases independientes.
- [x] Cobertura de tests >= 80% en cada modulo. Verificada 2026-05-23.
- [/] Documentacion cada modulo: README actualizado (ag-storage completo; ag-auth, ag-cache, ag-realtime, ag-observe actualizados 2026-05-23). Guia de API pendiente de expansion.
- [ ] Performance: ag-realtime sostiene 50K conexiones WebSocket en 2 vCPU. Pendiente benchmark.
- [ ] Performance: ag-cache >= 1M ops/segundo en L1. Pendiente benchmark.
- [ ] Al menos cinco issues bug reports cerrados por la comunidad.
- [ ] Al menos 500 stars en el repositorio.

## Fase 4.5 - ag-mail y ag-domains

Estado: capacidades de correo y dominios disponibles, con consolidacion abierta. ag-domains tiene trabajo activo separado; esta auditoria no modifica su implementacion. La fase no se declara cerrada mientras el gate pre-Fase 5 permanezca abierto.
### Criterios de entrada (4.5.1)

- [/] Fase 4 tiene los entregables principales implementados, pero conserva criterios de salida y gate de produccion pendientes.
- [x] ag-auth expone hooks/eventos para verificacion de correo, recuperacion de contrasena y magic links. AuthMailer implementado con los tres flujos.
- [x] ag-observe registra metricas y trazas de jobs asincronos.
- [x] RFC aprobado para el alcance inicial de ag-mail. Vease RFC-0006.
- [x] RFC aprobado para el alcance inicial de ag-domains. Vease RFC-0007.

### Entregables (4.5.2)

- [x] Crate ag-mail (estandar diferido): MailSender trait + SmtpSender (lettre + rustls). 38 tests.
- [x] Templates HTML/plaintext: MailTemplate trait + StringTemplate con sustitucion {{var}}. Motor externo (askama, minijinja) integrable via trait. Validacion de vars en compile-time via template::validate.
- [x] Declaracion de correos en schema.ag (bloque mail). DSL v0.7.
- [x] Integracion ag-auth -> ag-mail para verificacion, recuperacion y magic links. AuthMailer con feature "mail".
- [x] Cola asincrona con reintentos y backoff exponencial. InMemoryQueue. Backend persistente via ag-data: diferido (TECH-DEBT documentado).
- [x] Metricas hacia ag-observe: ag_mail_sent_total, ag_mail_retry_total, ag_mail_send_latency_seconds (feature "metrics").
- [x] Crate ag-domains (opcional infra): DnsProvider trait + CloudflareProvider; A/AAAA/CNAME/TXT/MX. 28 tests.
- [x] Soporte ACME (Let's Encrypt) via instant-acme: issue() + issue_with_credentials() + spawn_renewal_task(). Challenge DNS-01. TECH-DEBT: parseo notAfter para renovacion exacta.
- [x] Generacion de SPF/DKIM/DMARC requeridos por ag-mail. apply_mail_records idempotente.
- [x] Verificacion de propagacion contra multiples resolvers publicos (hickory-resolver). PropagationChecker + DEFAULT_RESOLVERS.
- [x] DSL v0.7: bloques mail, domain, template. Compilador valida: from referencia domain declarado (warning), provider valido, vars en templates, politica DMARC valida.
- [x] Actualizacion del LSP ag-lsp para los bloques nuevos: hover y completions para mail/domain/template y sus 7 propiedades.
- [x] Comandos CLI: ag domains check, ag domains sync, ag mail test.
- [x] Example auth-mail-demo en examples/: tres flujos con NullSender.
- [x] Documentacion: "Configurar dominio, TLS y correo transaccional con Anti-Gravital". Vease docs/manual/03-dominio-tls-correo.md.

### Criterios de salida (4.5.3, puerta antes de Fase 5)

- [x] ag-mail envia correo transaccional HTML y plaintext via SmtpSender (relay nativo).
- [x] ag-auth usa ag-mail para los tres flujos en auth-mail-demo.
- [x] ag-domains implementa CloudflareProvider funcional con tests de contrato.
- [x] ag-domains implementa ACME completo (issue + renovacion automatica) contra Let's Encrypt staging/production.
- [x] ag-domains genera SPF/DKIM/DMARC requeridos por ag-mail.
- [x] ag domains check, ag domains sync y ag mail test compilan y pasan CI.
- [x] 14 tests E2E cross-module en tests/integration (7 Fase 4 + 7 Fase 4.5).
- [x] Cero dependencias circulares (CI verde).
- [/] Los gates historicos estuvieron verdes; deben reejecutarse sobre el commit final de consolidacion antes de cerrar la fase.

## Fase 4.6-D - ag-workers

Estado: En curso. Fase aditiva de extraccion/endurecimiento pre-Fase 5, hermana de
las sub-fases 4.6-A (`mta`) y 4.6-C (`api`) de `ag-mail`. Autorizada por el BDFL con
el gate pre-Fase 5 abierto (ADR-0013); no se hace ninguna afirmacion de produccion/GA
hasta que el gate cierre. Decision en `docs/rfc/RFC-0012-ag-workers.md` y
`docs/adr/0013-ag-workers-execution-model.md`. El alcance esta fijo; la entrega se
secuencia en etapas S1-S7 (RFC-0012 seccion 5), cada una verde.

S1-S5 estan implementadas y verificadas con CI verde (codigo + tests sobre el
backend nativo en memoria). S6 esta parcial (patron y ejemplos listos; falta el
wiring dedicado del feature `producer`). S7 esta completa: M0-M2 entregados y
M3/M4 verificados contra una base PostgreSQL viva (paridad probada, cola generica
duplicada eliminada). El CI por defecto no ejerce los tests `#[ignore]` que lo
prueban; su verificacion manual cerro los Issues #108 (verificacion PG), #109
(S7/M3) y #103 (S7/M4).

### Criterios de entrada (4.6-D.1)

- [x] RFC aprobado para `ag-workers`. Vease RFC-0012.
- [x] ADR que fija el modelo de ejecucion y la autorizacion. Vease ADR-0013.
- [x] `ag-mail` ya implementa el patron cola+reintentos+persistencia a extraer
  (`crates/ag-mail/src/queue/`).
- [x] `ag-data` expone pool y migraciones embebidas; `ag-observe` expone metricas/trazas.

### Entregables (4.6-D.2)

- [x] S1 Fundaciones: crate `ag-workers`; `ids`, `job` (envelope + maquina de estados
  + prioridad), `payload` (rmp + versionado + migrate), `error`/`outcome`, `handler`,
  `registry`, `context` (CancellationToken). Sin runtime.
- [x] S2 Runtime en memoria: `MemoryQueue`, workers estaticos, loop de dispatch,
  retry/backoff, timeout, DLQ en memoria, poison guard, shutdown gracioso, telemetria.
- [x] S3 Persistencia (codigo): `PostgresQueue` via `ag-data`, migraciones
  (`0001`-`0003`), leasing `FOR UPDATE SKIP LOCKED`, heartbeat + reaper,
  `enqueue_in_tx` (acepta `ag_data::AgTx`, Issue #110), DLQ persistente,
  admision/backpressure (feature `postgres`). La verificacion contra una base
  viva es criterio de salida y se rastrea en el Issue #108 (el entorno de CI por
  defecto no levanta PostgreSQL). La variante de admision `RejectedRateLimited`
  (RFC-0012 seccion 18) queda **reservada**: es vocabulario sancionado por la RFC
  y parte del espacio de etiquetas de `ag_workers_backpressure_total`, pero
  ningun camino de admision la produce hoy; un limitador de tasa por cola que la
  genere cambia el contrato de admision y va tras una RFC/ADR futura (Issue #113).
- [x] S4 Scheduling + dinamico: jobs por intervalo con claim singleton; pools dinamicos
  acotados; pool CPU-bound (`spawn_blocking` + semaforo).
- [x] S5 Superficies: declaracion `worker` en el Anti-DSL (v0.8) + generador
  `worker_gen` (payloads tipados, stubs `JobHandler`, `register_workers`). CLI
  `ag workers` completa tras el feature `workers-runtime`: `list` (compila el schema,
  sin infraestructura), `run`, `enqueue`, `queues`, `dlq` (`list`/`inspect`/`retry`/
  `purge`) y `doctor`; los subcomandos que tocan backend durable requieren
  `DATABASE_URL`. Ejemplos entregados: `workers-basic`, `workers-postgres`,
  `workers-scheduled`, `workers-mail-integration`, `workers-producer-edge`.
- [/] S6 Modo producer + edge: el feature `producer` (enqueue-only, sin runtime de
  workers) existe y el patron esta documentado y ejemplificado
  (`examples/workers-producer-edge`, RFC-0012 seccion 17.4). Pendiente: wiring
  dedicado del feature `producer` desde `ag-edge` (consumidor enqueue-only segun
  seccion 7); seguimiento en el Issue #112 (diferido hasta que exista un
  consumidor concreto).
- [x] S7 Migracion de `ag-mail`: M0-M4 (RFC-0012 seccion 5) tras feature `workers`.
  M0 (overlap documentado en la RFC) y M1 (ag-workers entregado en S1-S6) hechos.
  M2: feature `workers` en `ag-mail` + `MailDeliveryHandler` (payload `Email`,
  clasificacion retriable/permanente) + `WorkersMailQueue` (impl `MailQueue`
  enrutando la entrega a `ag-workers`); la logica de correo permanece en `ag-mail`;
  43 tests verdes. M3: tests de paridad Postgres (`tests/workers_postgres.rs`,
  feature `workers-postgres`, `#[ignore]` + `TEST_DATABASE_URL`) verificados contra
  una base viva: persistencia como `kind=mail.delivery`, entrega y supervivencia a
  reinicio (Issue #109). M4: la cola generica duplicada
  (`queue::store::PersistentQueue`, feature `queue-persistent`, migracion
  `0001_mail_queue.sql`) fue eliminada tras probar la paridad; el unico camino
  durable es ahora el backend compartido de `ag-workers` (Issue #103).

### Criterios de salida (4.6-D.3)

- [x] Jobs tipados se ejecutan sobre el backend en memoria (`ag dev`) con retry,
  backoff y DLQ (`tests/runtime_outcomes.rs`, `tests/retry_policy.rs`).
- [/] Backend PostgreSQL leasea con `FOR UPDATE SKIP LOCKED`, sobrevive reinicio, y
  `enqueue_in_tx` commitea job + escrituras del llamador de forma atomica (test de
  rollback). El codigo y los tests existen (`tests/postgres_queue.rs`), pero son
  `#[ignore]` y exigen `DATABASE_URL`; su ejecucion contra una base viva esta
  bloqueada por el entorno y se rastrea en el Issue #108.
- [x] El poison guard convierte un job en crash-loop en una entrada acotada del DLQ
  (`tests/poison_guard.rs`).
- [x] Los jobs por intervalo disparan una sola vez (claim singleton) sobre el backend
  en memoria (`tests/scheduler_dynamic.rs`). La verificacion cross-proceso sobre
  PostgreSQL forma parte del Issue #108.
- [x] La declaracion `worker` del DSL compila y genera payloads + stubs de handler
  (`ag-dsl` v0.8, `codegen/worker_gen.rs`).
- [x] El grupo de comandos `ag workers ...` compila y pasa CI (feature-gated
  `workers-runtime`).
- [x] Cobertura >= 80% (82.28% en `quality`); la gramatica `worker` se fuzzea via los
  targets unificados del DSL (`fuzz_parser`/`fuzz_compile`) y el decoder de payload
  via `fuzz_workers_payload` (`fuzz/fuzz_targets/`), integrado en el job de
  fuzz-smoke (60s) del workflow `quality` con CI verde.
- [x] Cero dependencias circulares (CI verde).
- [x] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit`,
  `cargo deny check` verdes.
- [x] Sin afirmacion de produccion/GA antes de que el gate pre-Fase 5 lo permita.

## Fase 5 - ag-cloud

Estado: Pendiente. Vease `docs/roadmap/fase-05-ag-cloud.md`.

## Fase 6 - ag-ai y Knowledge Graph

Estado: Pendiente. Vease `docs/roadmap/fase-06-ag-ai-knowledge-graph.md`.

## Fase 7 - ag-migrate

Estado: Pendiente. Vease `docs/roadmap/fase-07-ag-migrate.md`.

## Fase 8 - ag-mobile

Estado: Pendiente. Vease `docs/roadmap/fase-08-ag-mobile.md`.

## Fase 9 - Plugins WASI

Estado: Pendiente. Vease `docs/roadmap/fase-09-plugins-wasi.md`.

## Fase 10 - Endurecimiento y 1.0

Estado: Pendiente. Vease `docs/roadmap/fase-10-endurecimiento-y-1.0.md`.
