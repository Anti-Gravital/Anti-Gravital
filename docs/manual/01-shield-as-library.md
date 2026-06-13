English | Espanol

---

# Chapter 1. The `ag-core` Shield as a library

> Architectural source: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
> chapter 6 (Core architecture) and RFC-0002 (Shield MVP design).

This chapter describes how to use `ag-core::Shield` as a library from an
existing Rust application, without going through the `ag` CLI or the DSL
codegen. It covers everything from the minimal example to the full
configuration with TLS, JWT, CSRF, CORS, rate-limit and validation.

## 1.1 What the Shield is

The Shield is the first layer of the Anti-Gravital core: a Tower
middleware pipeline that processes every HTTP request before it reaches
the business handler. Its single responsibility is to decide whether a
request is **trustworthy** enough to hand to the application code.

The standard layers (outermost to innermost: logging, rate-limit, CORS,
auth-jwt, CSRF) compose over any `axum::Router` and operate at the
process level, with no IPC or FFI. The Shield is the mandatory piece of
the ecosystem; the remaining layers (Core, standard modules, optional
modules) are built on top of it or beside it.

## 1.2 Add `ag-core` to your project

`ag-core` is not published on crates.io during Phase 1. The way to
consume it is as a git dependency:

```toml
[dependencies]
ag-core = { git = "https://github.com/anti-gravital/anti-gravital", branch = "main" }
axum = "0.7"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

If you only need a subset of layers, you can disable features
explicitly:

```toml
[dependencies]
ag-core = { git = "https://github.com/anti-gravital/anti-gravital", branch = "main", default-features = false, features = ["validation", "cors"] }
```

Available features: `validation`, `cors`, `csrf`, `logging`,
`rate-limit`, `auth-jwt`, `tls`. All enabled by default.

## 1.3 Minimal server in five lines

The smallest use applies a Shield with the default configuration over a
trivial Axum router. Only the logging layer stays active; the rest expect
an explicit declaration in the configuration.

```rust,no_run
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shield = Shield::try_new(ShieldConfig::default())?;
    let app = shield.apply(
        Router::new().route("/", get(|| async { "hello, shield" })),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    shield.serve(listener, app).await?;
    Ok(())
}
```

This example is exactly what `crates/ag-core/examples/hello_world.rs`
ships. To run it locally:

```sh
cargo run --release -p ag-core --example hello_world
curl -i http://127.0.0.1:8080/
```

## 1.4 Load configuration from TOML

For production environments the configuration lives in a declarative
file. `ShieldConfig::from_path` loads it and applies
`#[serde(deny_unknown_fields)]`: any typo produces `AgError::Config` with
the exact name of the unknown field.

```rust,no_run
use ag_core::{Shield, ShieldConfig};

# async fn run() -> Result<(), ag_core::AgError> {
let config = ShieldConfig::from_path("config.toml")?;
let shield = Shield::try_new(config)?;
# let _ = shield;
# Ok(())
# }
```

The reference file with every section documented is at
`crates/ag-core/config.example.toml`. Section summary:

```toml
bind = "0.0.0.0:8080"

[runtime]
blocking_threads = 512

[cors]
enabled = false
allow_origins = []
allow_methods = ["GET", "POST"]
allow_headers = ["content-type", "authorization"]
allow_credentials = false

[csrf]
enabled = false
token_header = "x-csrf-token"
token_cookie = "ag_csrf"

[rate_limit]
enabled = false
per_ip_rps = 100
burst = 200

[auth]
enabled = false
# public_key_pem = "..."
# public_key_path = "/etc/anti-gravital/jwt.public.pem"
# expected_issuer = "https://auth.example.com/"
# expected_audience = "anti-gravital"

[tls]
enabled = false
# cert_path = "/etc/anti-gravital/tls.cert.pem"
# key_path = "/etc/anti-gravital/tls.key.pem"
```

The layers stay disabled by default. Activation is declarative and
explicit.

## 1.5 Layers by order of application

`Shield::apply(router)` wraps the router from the inside out. What
appears as the last `.layer()` is what the request sees first. For
operational safety, the current order is:

1. **Logging** (outermost): traces with method, path, status and
   latency. Active whenever the `logging` feature is present.
2. **Rate limit**: per-IP token bucket. Rejects before spending CPU on
   cryptographic validation.
3. **CORS**: responses for preflight and allowed-origin headers.
4. **Auth-JWT**: verifies `Authorization: Bearer <token>` (Ed25519) and
   injects `AuthContext` into the request extensions.
5. **CSRF** (innermost): double-submit cookie over mutating methods. It
   only runs once auth has passed, so an attacker without a token does
   not consume the CSRF cycle.

The **validation** layer is not Tower middleware; it is an extractor
(`ValidatedJson<T>`) used per handler.

## 1.6 Enable TLS with a self-signed cert in development

For local development with HTTPS, generate a self-signed certificate and
configure `[tls]`:

```sh
# Once, with openssl or mkcert.
mkcert -install
mkcert -cert-file dev-cert.pem -key-file dev-key.pem localhost 127.0.0.1
```

```toml
[tls]
enabled = true
cert_path = "dev-cert.pem"
key_path = "dev-key.pem"
```

When `[tls].enabled = true`, `Shield::serve` wraps each accepted TCP
connection with `tokio_rustls::TlsAcceptor`. It also injects the peer's
`ConnectInfo<SocketAddr>` into every request so the rate-limit layer can
identify the client.

In production behind a load balancer that already terminates TLS
(Cloudflare, AWS ALB, Nginx, Caddy), leave `[tls].enabled = false`. The
balancer does the termination and the Shield only serves plaintext
locally.

## 1.7 Enable JWT Ed25519 authentication

Generate the Ed25519 key pair outside Anti-Gravital (a separate
authentication service issues tokens) and configure the public key:

```toml
[auth]
enabled = true
public_key_path = "/etc/anti-gravital/jwt.public.pem"
expected_issuer = "https://auth.example.com/"
expected_audience = "anti-gravital"
```

In a handler, consume the claims via the `Claims<T>` extractor where `T`
is the struct describing your claims:

```rust
use ag_core::shield::Claims;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppClaims {
    sub: String,
    exp: u64,
    role: String,
}

async fn me_handler(Claims(c): Claims<AppClaims>) -> String {
    format!("hello {} (role: {})", c.sub, c.role)
}
```

`Claims<T>` fails with `AgError::Auth` if the `auth-jwt` layer is not
active or if the JWT claims cannot be deserialized to the type `T`. The
cryptographic verification happens in the layer, not in the extractor.

The `leeway` for `exp` and `nbf` is forced to 0 seconds to avoid silent
tolerance to clock drift. If your system needs a margin, open an RFC.

## 1.8 Per-handler payload validation

To validate a request body, implement the `Validate` trait on the
request type and use the `ValidatedJson<T>` extractor:

```rust
use ag_core::shield::{Validate, ValidatedJson, ValidationErrors};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreatePost {
    title: String,
}

impl Validate for CreatePost {
    fn validate(&self, errors: &mut ValidationErrors) {
        if self.title.is_empty() {
            errors.add("title", "must not be empty");
        }
        if self.title.len() > 200 {
            errors.add("title", "too long, max 200 chars");
        }
    }
}

async fn create_post(ValidatedJson(post): ValidatedJson<CreatePost>) -> String {
    format!("created \"{}\"", post.title)
}
```

A failure produces `AgError::Validation` with status 422 and a JSON body
that lists the per-field errors. From Phase 3 on, the DSL codegen
generates these `impl Validate` automatically from the annotations in
`schema.ag`.

## 1.9 Enable rate-limit and CSRF

Per-IP token-bucket rate limit:

```toml
[rate_limit]
enabled = true
per_ip_rps = 100
burst = 200
```

CSRF double-submit cookie (issuing the cookie is the project's
responsibility, typically a dedicated endpoint):

```toml
[csrf]
enabled = true
token_header = "x-csrf-token"
token_cookie = "ag_csrf"
```

CSRF is applied only to state-mutating methods (POST, PUT, PATCH,
DELETE). GET, HEAD, OPTIONS and TRACE pass without verification.

## 1.10 Errors and observability

Every error the pipeline produces implements
`axum::response::IntoResponse` via `AgError`. The JSON body has a stable
shape:

```json
{
  "code": "auth_error",
  "message": "invalid token: ExpiredSignature"
}
```

The `code` field is a stable `snake_case` identifier. Current codes:
`config_error`, `tls_error`, `auth_error`, `rate_limit_exceeded`,
`validation_error`, `cors_error`, `csrf_error`, `io_error`,
`internal_error`.

The logging layer emits one `tracing` event per request with `method`,
`path`, `status` and `latency_ms`. For full observability, the
`ag-observe` module (Phase 4) adds OpenTelemetry and Prometheus metrics.

## 1.11 Deployment

Recommendations for a production deployment:

- Use `cargo build --release` with the workspace profile (LTO `fat`,
  `codegen-units = 1`, `panic = abort`, `strip = symbols`,
  `opt-level = 3`).
- If you are behind a load balancer that terminates TLS, leave
  `[tls].enabled = false`.
- Set `RUST_LOG` to control tracing verbosity. Example:
  `RUST_LOG=info,ag_core=debug`.
- For HTTP/2, no special change: Axum negotiates HTTP/1.1 or HTTP/2 via
  ALPN when TLS is active, and supports both over plaintext via upgrade.
- To measure production, see `docs/benchmarks/measurement-template.md`
  and use `oha` or `wrk` against the release binary.

## 1.12 What the Shield is NOT

The Shield is not an application router. The router is Axum, composed
separately and passed to `Shield::apply`. It is also not a complete auth
system: the Shield verifies tokens issued by third parties, it does not
issue them. It does not handle sessions, stateful cookies, multi-tenancy
or complex RBAC: that belongs to the `ag-auth` module in Phase 4.

## 1.13 Cross references

- Code: `crates/ag-core/`.
- API documentation: run `cargo doc --workspace --no-deps --open`.
- Binary example: `crates/ag-core/examples/hello_world.rs`.
- TOML config example: `crates/ag-core/config.example.toml`.
- E2E tests that serve as practical reference:
  `crates/ag-core/tests/shield_full_pipeline.rs`,
  `crates/ag-core/tests/shield_tls.rs`,
  `crates/ag-core/tests/shield_auth.rs`,
  `crates/ag-core/tests/shield_csrf.rs`,
  `crates/ag-core/tests/shield_cors.rs`,
  `crates/ag-core/tests/shield_rate_limit.rs`,
  `crates/ag-core/tests/shield_validation.rs`.
- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- Master: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  chapter 6.
- Roadmap: `docs/roadmap/fase-01-shield-mvp.md`.

---

# Capitulo 1. La Shield de `ag-core` como libreria

> Fuente arquitectonica: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
> capitulo 6 (Arquitectura del nucleo) y RFC-0002 (Diseno del Shield MVP).

Este capitulo describe como usar `ag-core::Shield` como libreria desde
una aplicacion Rust existente, sin pasar por la CLI `ag` ni por el
codegen del DSL. Cubre desde el ejemplo minimo hasta la configuracion
completa con TLS, JWT, CSRF, CORS, rate-limit y validacion.

## 1.1 Que es la Shield

La Shield es la primera capa del nucleo de Anti-Gravital: una pipeline
de middleware Tower que procesa todo request HTTP antes de que llegue
al handler de negocio. Su unica responsabilidad es decidir si un
request es **confiable** para entregarselo al codigo de aplicacion.

Las capas estandar (en orden externo a interno: logging, rate-limit,
CORS, auth-jwt, CSRF) se componen sobre cualquier `axum::Router` y
operan a nivel de proceso, sin IPC ni FFI. La Shield es la pieza
obligatoria del ecosistema; el resto de capas (Core, modulos
estandar, modulos opcionales) se construyen sobre ella o a su lado.

## 1.2 Anadir `ag-core` a su proyecto

`ag-core` no esta publicado en crates.io durante Fase 1. La forma de
consumirlo es como dependencia git:

```toml
[dependencies]
ag-core = { git = "https://github.com/anti-gravital/anti-gravital", branch = "main" }
axum = "0.7"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Si solo necesita un subconjunto de capas, puede desactivar features
explicitamente:

```toml
[dependencies]
ag-core = { git = "https://github.com/anti-gravital/anti-gravital", branch = "main", default-features = false, features = ["validation", "cors"] }
```

Features disponibles: `validation`, `cors`, `csrf`, `logging`,
`rate-limit`, `auth-jwt`, `tls`. Todas activas por defecto.

## 1.3 Servidor minimo en cinco lineas

El uso mas pequeno aplica un Shield con configuracion por defecto
sobre un router Axum trivial. Solo la capa de logging queda activa;
el resto esperan declaracion explicita en la configuracion.

```rust,no_run
use ag_core::{Shield, ShieldConfig};
use axum::routing::get;
use axum::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shield = Shield::try_new(ShieldConfig::default())?;
    let app = shield.apply(
        Router::new().route("/", get(|| async { "hello, shield" })),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    shield.serve(listener, app).await?;
    Ok(())
}
```

Este ejemplo es exactamente lo que entrega
`crates/ag-core/examples/hello_world.rs`. Para ejecutarlo localmente:

```sh
cargo run --release -p ag-core --example hello_world
curl -i http://127.0.0.1:8080/
```

## 1.4 Cargar configuracion desde TOML

Para entornos de produccion la configuracion vive en un archivo
declarativo. `ShieldConfig::from_path` lo carga y aplica
`#[serde(deny_unknown_fields)]`: cualquier typo produce
`AgError::Config` con el nombre exacto del campo desconocido.

```rust,no_run
use ag_core::{Shield, ShieldConfig};

# async fn run() -> Result<(), ag_core::AgError> {
let config = ShieldConfig::from_path("config.toml")?;
let shield = Shield::try_new(config)?;
# let _ = shield;
# Ok(())
# }
```

El archivo de referencia con todas las secciones documentadas esta en
`crates/ag-core/config.example.toml`. Resumen de secciones:

```toml
bind = "0.0.0.0:8080"

[runtime]
blocking_threads = 512

[cors]
enabled = false
allow_origins = []
allow_methods = ["GET", "POST"]
allow_headers = ["content-type", "authorization"]
allow_credentials = false

[csrf]
enabled = false
token_header = "x-csrf-token"
token_cookie = "ag_csrf"

[rate_limit]
enabled = false
per_ip_rps = 100
burst = 200

[auth]
enabled = false
# public_key_pem = "..."
# public_key_path = "/etc/anti-gravital/jwt.public.pem"
# expected_issuer = "https://auth.example.com/"
# expected_audience = "anti-gravital"

[tls]
enabled = false
# cert_path = "/etc/anti-gravital/tls.cert.pem"
# key_path = "/etc/anti-gravital/tls.key.pem"
```

Las capas se mantienen deshabilitadas por defecto. La activacion es
declarativa y explicita.

## 1.5 Capas por orden de aplicacion

`Shield::apply(router)` envuelve el router de adentro hacia afuera.
Lo que aparece como ultima `.layer()` es lo que ve el request
primero. Por seguridad operacional, el orden actual es:

1. **Logging** (mas externa): trazas con metodo, path, status y
   latencia. Activa siempre que la feature `logging` este presente.
2. **Rate limit**: token bucket por IP. Rechaza antes de gastar CPU
   en validaciones criptograficas.
3. **CORS**: respuestas para preflight y headers de origenes
   permitidos.
4. **Auth-JWT**: verifica `Authorization: Bearer <token>` Ed25519 e
   inyecta `AuthContext` en las extensiones del request.
5. **CSRF** (mas interna): double-submit cookie sobre metodos
   mutantes. Solo corre cuando auth ya passed, asi un atacante sin
   token no consume el ciclo CSRF.

La capa de **validacion** no es Tower middleware; es un extractor
(`ValidatedJson<T>`) que se usa por handler.

## 1.6 Activar TLS con cert auto-firmado en desarrollo

Para desarrollo local con HTTPS, genere un certificado auto-firmado
y configure `[tls]`:

```sh
# Una vez, con openssl o mkcert.
mkcert -install
mkcert -cert-file dev-cert.pem -key-file dev-key.pem localhost 127.0.0.1
```

```toml
[tls]
enabled = true
cert_path = "dev-cert.pem"
key_path = "dev-key.pem"
```

Cuando `[tls].enabled = true`, `Shield::serve` envuelve cada conexion
TCP aceptada con `tokio_rustls::TlsAcceptor`. Inyecta tambien el
`ConnectInfo<SocketAddr>` del peer en cada request para que la capa
rate-limit pueda identificar al cliente.

En produccion detras de un balanceador que ya termina TLS (Cloudflare,
AWS ALB, Nginx, Caddy), deje `[tls].enabled = false`. El balanceador
hace la terminacion y la Shield solo sirve plano localmente.

## 1.7 Habilitar autenticacion JWT Ed25519

Genere el par de claves Ed25519 fuera de Anti-Gravital (un servicio
de autenticacion separado emite tokens) y configure la clave publica:

```toml
[auth]
enabled = true
public_key_path = "/etc/anti-gravital/jwt.public.pem"
expected_issuer = "https://auth.example.com/"
expected_audience = "anti-gravital"
```

En un handler, consuma los claims via el extractor `Claims<T>` donde
`T` es el struct que describe sus claims:

```rust
use ag_core::shield::Claims;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppClaims {
    sub: String,
    exp: u64,
    role: String,
}

async fn me_handler(Claims(c): Claims<AppClaims>) -> String {
    format!("hello {} (role: {})", c.sub, c.role)
}
```

`Claims<T>` falla con `AgError::Auth` si la capa `auth-jwt` no esta
activa o si los claims del JWT no se pueden deserializar al tipo
`T`. La verificacion criptografica ocurre en la capa, no en el
extractor.

El `leeway` para `exp` y `nbf` esta forzado a 0 segundos para evitar
tolerancia silenciosa a deriva de reloj. Si su sistema necesita
margen, abrir RFC.

## 1.8 Validacion de payload por handler

Para validar el body de una peticion, implemente el trait
`Validate` en el tipo de request y use el extractor
`ValidatedJson<T>`:

```rust
use ag_core::shield::{Validate, ValidatedJson, ValidationErrors};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreatePost {
    title: String,
}

impl Validate for CreatePost {
    fn validate(&self, errors: &mut ValidationErrors) {
        if self.title.is_empty() {
            errors.add("title", "must not be empty");
        }
        if self.title.len() > 200 {
            errors.add("title", "too long, max 200 chars");
        }
    }
}

async fn create_post(ValidatedJson(post): ValidatedJson<CreatePost>) -> String {
    format!("created \"{}\"", post.title)
}
```

Una falla produce `AgError::Validation` con status 422 y un body
JSON que enumera los errores por campo. A partir de Fase 3, el
codegen del DSL generara estos `impl Validate` automaticamente desde
las anotaciones del `schema.ag`.

## 1.9 Activar rate-limit y CSRF

Rate-limit con token bucket por IP:

```toml
[rate_limit]
enabled = true
per_ip_rps = 100
burst = 200
```

CSRF double-submit cookie (el emisor de la cookie es responsabilidad
del proyecto, tipicamente un endpoint dedicado):

```toml
[csrf]
enabled = true
token_header = "x-csrf-token"
token_cookie = "ag_csrf"
```

CSRF se aplica solo a metodos que mutan estado (POST, PUT, PATCH,
DELETE). GET, HEAD, OPTIONS y TRACE pasan sin verificacion.

## 1.10 Errores y observabilidad

Todos los errores que produce la pipeline implementan
`axum::response::IntoResponse` via `AgError`. El cuerpo JSON tiene
forma estable:

```json
{
  "code": "auth_error",
  "message": "invalid token: ExpiredSignature"
}
```

El campo `code` es un identificador estable `snake_case`. Codigos
actuales: `config_error`, `tls_error`, `auth_error`,
`rate_limit_exceeded`, `validation_error`, `cors_error`, `csrf_error`,
`io_error`, `internal_error`.

La capa de logging emite un evento `tracing` por request con
`method`, `path`, `status` y `latency_ms`. Para una observabilidad
completa, el modulo `ag-observe` (Fase 4) anadira OpenTelemetry y
metricas Prometheus.

## 1.11 Despliegue

Recomendaciones para deploy productivo:

- Use `cargo build --release` con el perfil del workspace (LTO `fat`,
  `codegen-units = 1`, `panic = abort`, `strip = symbols`,
  `opt-level = 3`).
- Si esta detras de un balanceador que termina TLS, deje
  `[tls].enabled = false`.
- Configure `RUST_LOG` para controlar la verbosidad de tracing.
  Ejemplo: `RUST_LOG=info,ag_core=debug`.
- Para HTTP/2, ningun cambio especial: Axum negocia HTTP/1.1 o HTTP/2
  via ALPN cuando TLS esta activo, y soporta ambos sobre plano via
  upgrade.
- Para medir produccion, vease `docs/benchmarks/measurement-template.md`
  y use `oha` o `wrk` contra el binario release.

## 1.12 Que NO es la Shield

La Shield no es un router de aplicacion. El router es Axum, que se
compone aparte y se pasa a `Shield::apply`. Tampoco es un sistema
de auth completo: la Shield verifica tokens emitidos por terceros,
no los emite. Tampoco maneja sesiones, cookies estatales,
multi-tenancy ni RBAC complejo: eso entra en el modulo `ag-auth` en
Fase 4.

## 1.13 Referencias cruzadas

- Codigo: `crates/ag-core/`.
- Documentacion API: ejecute `cargo doc --workspace --no-deps --open`.
- Ejemplo binario: `crates/ag-core/examples/hello_world.rs`.
- Ejemplo config TOML: `crates/ag-core/config.example.toml`.
- Tests E2E que sirven como referencia practica:
  `crates/ag-core/tests/shield_full_pipeline.rs`,
  `crates/ag-core/tests/shield_tls.rs`,
  `crates/ag-core/tests/shield_auth.rs`,
  `crates/ag-core/tests/shield_csrf.rs`,
  `crates/ag-core/tests/shield_cors.rs`,
  `crates/ag-core/tests/shield_rate_limit.rs`,
  `crates/ag-core/tests/shield_validation.rs`.
- RFC: `docs/rfc/RFC-0002-diseno-shield-mvp.md`.
- Maestro: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  capitulo 6.
- Hoja de Ruta: `docs/roadmap/fase-01-shield-mvp.md`.
