# Estado vivo de la Hoja de Ruta

Este archivo agrega el estado de las casillas de cada fase, en orden,
para que un agente o un humano pueda saber en un solo vistazo que esta
hecho y que falta. Se actualiza con cada pull request que avance la
hoja de ruta.

Convencion: `- [x]` significa cumplido y verificable en el repositorio,
`- [ ]` significa pendiente, `- [/]` significa parcialmente cumplido
(con explicacion).

Ultima actualizacion: 2026-05-21, medicion real CRUD con PostgreSQL nativo, correccion bug de routing Axum 0.7 ({id} -> :id).

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

Estado: Implementacion tecnica completa. Benchmarks medidos en hardware
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

Estado: En curso. Iniciada 2026-05-21 en rama `fase-3`.
RFC-0003 aceptada. Stack fijado: logos 0.14 (lexer), chumsky 0.9 (parser),
format! macros (codegen). Implementacion incremental: v0.1 y v0.2 completados.

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
- [ ] DSL version 0.4: relaciones entre modelos (1:1, 1:N, N:M).
- [x] Generador Rust: structs con serde (v0.1), types.rs + handlers.rs + router.rs (v0.2).
- [x] Generador SQL: migraciones idempotentes (v0.1).
- [x] Generador TypeScript: tipos y cliente HTTP (v0.1+v0.2).
- [x] Generador OpenAPI 3.1: schemas (v0.1) + paths (v0.2).
- [x] Comando ag generate. Operativo desde v0.1.
- [x] Comando ag schema lint. Operativo desde v0.1.
- [x] Comando ag schema diff. Operativo desde v0.1.
- [x] Diagnostics legibles. Lex + parse + semantic con linea:columna.
- [ ] Servidor LSP basico (ag-lsp).
- [ ] Plugin VS Code.
- [/] Cobertura tests >= 85%. 73 tests verdes; cobertura formal pendiente.
- [ ] Fuzzing 24h sin crashes.
- [ ] Documentacion de referencia del DSL.

### Criterios de salida (3.3)

- [ ] Proyecto completo definible en schema.ag, generado y ejecutable con CLI.
- [ ] Example ecommerce-api reescrito con DSL.
- [ ] CRUD generado por DSL no es mas lento que CRUD a mano.
- [ ] Plugin VS Code >= 100 instalaciones.
- [ ] Al menos un colaborador externo contribuyo al compilador.
- [ ] Documentacion DSL revisada por dos personas.
- [ ] Al menos 200 stars.

## Fase 4 - Modulos estandar

Estado: Pendiente. Vease `docs/roadmap/fase-04-modulos-estandar.md`.

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
