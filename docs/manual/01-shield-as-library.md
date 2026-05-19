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
