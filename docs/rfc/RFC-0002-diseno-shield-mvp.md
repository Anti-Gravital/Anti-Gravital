# RFC-0002 - Diseno del Shield MVP (Fase 1)

- Estado: aceptado
- Autor: Angel Nereira (BDFL inicial)
- Fecha de borrador: 2026-05-19
- Fecha de aceptacion: 2026-05-19
- Fase objetivo: Fase 1
- Modulos o crates afectados: `ag-core`
- RFC predecesora: RFC-0001 (paralelizacion)
- Periodo de comentarios: omitido por decision del BDFL en modo solo

## 1. Motivacion

La Fase 1 entrega la capa Shield del nucleo: una pipeline de
middleware Tower que valida, autentica basicamente, aplica rate
limiting y entrega requests a un handler placeholder. El maestro de
arquitectura define el alcance (seccion 6) y la Hoja de Ruta define
los entregables y los criterios de salida medibles (Fase 1).

Esta RFC fija las decisiones tecnicas concretas necesarias para
iniciar la implementacion: stack, organizacion del codigo,
dependencias permitidas y politicas de evolucion.

## 2. Problema

El maestro y la hoja de ruta describen el que. Falta el como, en
detalle suficiente para que la implementacion no improvise
arquitectura. Concretamente, falta decidir:

- Que crates externos se usan y en que version.
- Como se organiza el codigo dentro de `ag-core`.
- Que tipo de configuracion publica expone el Shield.
- Como se mide el cumplimiento de los criterios de salida.

## 3. Alternativas consideradas

### 3.1 Stack basado en Axum + Tower

Axum sobre Hyper, middleware via Tower, runtime Tokio. Es la opcion
descrita en el maestro. Es el estandar de facto del ecosistema Rust
para servicios HTTP y es lo que cualquier contribuidor con experiencia
Rust espera.

### 3.2 Stack basado en Actix-Web

Actor model, performance excepcional historicamente. Pros: muy
rapido. Contras: contradice la decision arquitectonica del maestro;
modelo de actores es mas complejo de razonar para middleware
compositional; menor familiaridad cross-team.

### 3.3 Implementar el servidor HTTP desde cero

Solo Hyper y traits propios. Maxima libertad. Contras: reinventa
ruedas; pierde la composibilidad de Tower que necesitamos para los
modulos posteriores; viola la Regla 12 (interoperabilidad: integrar,
no reemplazar).

Se elige 3.1 por alineacion con el maestro, ecosistema y filosofia
del proyecto.

## 4. Diseno propuesto

### 4.1 Stack y dependencias

Dependencias del crate `ag-core` en Fase 1:

| Crate externo | Version minima | Proposito |
| --- | --- | --- |
| `tokio` | 1.40 | Runtime async multi-thread con feature `full`. |
| `axum` | 0.7 | Router HTTP/1.1 y HTTP/2 sobre Hyper. |
| `tower` | 0.5 | Modelo de middleware compositional. |
| `tower-http` | 0.6 | Capas estandar: trace, cors, timeout, request-id. |
| `hyper` | 1.4 | HTTP base; expuesto a traves de Axum. |
| `hyper-util` | 0.1 | Helpers de servidor para Axum. |
| `rustls` | 0.23 | TLS 1.3 sin OpenSSL. Activado en capa de TLS. |
| `tokio-rustls` | 0.26 | Integracion de rustls con Tokio. |
| `rustls-pemfile` | 2 | Carga de certificados PEM. |
| `governor` | 0.7 | Token bucket para rate limiting. |
| `jsonwebtoken` | 9 | Verificacion JWT (Ed25519 via feature). |
| `ring` | 0.17 | Primitivas criptograficas Ed25519. |
| `serde` | 1 | Serializacion. |
| `serde_json` | 1 | JSON. |
| `toml` | 0.8 | Configuracion. |
| `tracing` | 0.1 | Logging estructurado. |
| `tracing-subscriber` | 0.3 | Suscriptor por defecto con formato. |
| `thiserror` | 1 | Tipos de error derivados. |
| `bytes` | 1 | Buffers eficientes. |
| `http` | 1 | Tipos HTTP estandar. |
| `pin-project-lite` | 0.2 | Pin projection sin dep de proc-macro. |

Dependencias de desarrollo:

| Crate | Proposito |
| --- | --- |
| `criterion` | Benchmarks. |
| `tokio-test` | Utilidades async para tests. |
| `reqwest` | Cliente HTTP para tests E2E. |
| `tower-test` | Mock services para Tower. |

Cada dependencia debe estar listada en `[workspace.dependencies]` del
`Cargo.toml` raiz, con su feature set explicito, para garantizar
versionado coherente cross-crate.

### 4.2 Organizacion del codigo

```
crates/ag-core/
  src/
    lib.rs            // Re-exports publicos y modulos
    error.rs          // AgError, AgResult, IntoResponse impl
    runtime/
      mod.rs          // Configuracion Tokio
    config/
      mod.rs          // Tipos de configuracion deserializados desde TOML
    shield/
      mod.rs          // Pipeline Tower; Shield::builder()
      tls.rs          // Capa TLS 1.3 (rustls). Feature `tls`.
      auth.rs         // Capa JWT Ed25519. Feature `auth-jwt`.
      rate_limit.rs   // Token bucket por IP. Feature `rate-limit`.
      validation.rs   // Validacion de payload.
      cors.rs         // CORS con defaults seguros.
      csrf.rs         // CSRF con defaults seguros.
      logging.rs      // Tracing layer.
    core/             // Placeholder; implementacion completa en Fase 2.
      mod.rs
  benches/
    shield_hello_world.rs
  tests/
    shield_e2e.rs
```

### 4.3 Features Cargo

- `default = ["rate-limit", "validation", "cors", "csrf", "logging"]`.
- `tls`: activa TLS y dependencias `rustls`, `tokio-rustls`, `rustls-pemfile`.
- `auth-jwt`: activa JWT y dependencias `jsonwebtoken`, `ring`.
- `rate-limit`: activa `governor`.
- `validation`: activa restricciones declarativas basicas en `serde`.
- `cors`: activa `tower-http` cors.
- `csrf`: activa proteccion CSRF.
- `logging`: activa `tracing` y `tracing-subscriber`.

Esto cumple la Regla 15 (features bien aisladas).

### 4.4 API publica minima de Fase 1

```rust
use ag_core::{Shield, ShieldConfig};

let config = ShieldConfig::from_path("ag.toml")?;
let shield = Shield::builder(config).build();
let app = axum::Router::new()
    .route("/", get(|| async { "hello, shield" }))
    .layer(shield.into_layer());

let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
axum::serve(listener, app).await?;
```

Esto es el contrato que la Fase 1 debe sostener. Los modulos
posteriores (Core en Fase 2) componen sobre esta misma firma.

### 4.5 Configuracion TOML

```toml
[bind]
addr = "0.0.0.0:8080"

[runtime]
workers = "auto"
blocking_threads = 512
shutdown_timeout = "30s"

[shield.rate_limit]
enabled = true
per_ip_rps = 100
burst = 200

[shield.cors]
enabled = true
allow_origins = ["https://example.com"]
allow_methods = ["GET", "POST"]
allow_headers = ["content-type", "authorization"]
allow_credentials = false

[shield.csrf]
enabled = true
token_header = "X-CSRF-Token"

[shield.tls]
enabled = false
cert_path = "/etc/ag/cert.pem"
key_path = "/etc/ag/key.pem"

[shield.auth.jwt]
enabled = false
public_key_path = "/etc/ag/jwt-ed25519.pub"
```

Todos los campos opcionales tienen defaults seguros. La carga falla
con error preciso si el TOML contiene claves desconocidas.

### 4.6 Sistema de errores

`AgError` es un enum con variantes por dominio (`Config`, `Tls`,
`Auth`, `RateLimit`, `Validation`, `Cors`, `Csrf`, `Io`, `Other`).
Implementa `axum::response::IntoResponse` para mapeo automatico a
respuesta HTTP. Cada variante lleva un codigo de error estable
(string snake_case) que se serializa en el body JSON. Un `correlation_id`
generado por el middleware de logging aparece en todas las respuestas
de error.

### 4.7 Configuracion Tokio

Runtime multi-thread por defecto: un worker por CPU. Stack size de
2MB por thread. Blocking pool de 512. Shutdown graceful con timeout
configurable. Estos parametros vienen de la seccion 6.5 del maestro.

## 5. Plan de implementacion

PRs incrementales, cada uno con tests, fmt y clippy estrictos:

1. Bootstrap `ag-core`: dependencias, modulos vacios, error.rs basico,
   runtime.rs basico, ejemplo Hello World HTTP/1.1+HTTP/2 sin TLS.
2. Capa de logging con tracing y request-id.
3. Capa de validacion de payload.
4. Capa CORS.
5. Capa CSRF.
6. Capa de rate limiting con governor.
7. Capa de autenticacion JWT Ed25519.
8. Capa TLS 1.3 con rustls.
9. Configuracion TOML completa.
10. Benchmark Hello World con criterion.
11. Tests E2E con reqwest cubriendo el pipeline completo.

## 6. Riesgos

- Versiones de dependencias rapidas en evolucion (axum, hyper).
  Mitigacion: pin estricto y actualizaciones semanales monitorizadas.
- Targets de rendimiento ambiciosos (300K req/s). Mitigacion: medir
  desde el primer PR, publicar cifras honestas; si no se alcanzan,
  documentar la brecha en STATUS.md y abrir RFC de optimizacion.
- Curva de aprendizaje de rustls 0.23. Mitigacion: capa de TLS llega
  en PR 8, con tiempo de estudio previo.
- Compatibilidad CI ARM64. Mitigacion: ya configurado en
  `.github/workflows/ci.yml`.

## 7. Impacto

- Sobre el alcance: ninguno; estrictamente dentro de Fase 1.
- Sobre el cronograma: 11 PRs estimados a 1-2 semanas cada uno = 3
  meses, consistente con el rango de Hoja de Ruta (2-3 meses).
- Sobre la API publica: define la primera API publica del proyecto.
- Sobre la documentacion: cada PR actualiza el capitulo
  correspondiente bajo `docs/architecture/06-nucleo-shield-y-core.md`
  (siempre via regeneracion desde el maestro si la decision afecta al
  maestro).

## 8. Rollback

Si el stack elegido demuestra ser inviable durante la implementacion
(por ejemplo, axum no escala a los targets), se abre RFC sucesora que
proponga el cambio. Mientras tanto, los PRs ya merged se revierten en
orden inverso. El crate `ag-core` se restaura al estado Fase 0
(esqueleto vacio).

## 9. Decision

Aceptada por el BDFL inicial junto con RFC-0001.

## 10. Referencias

- `docs/architecture/06-nucleo-shield-y-core.md`.
- `docs/roadmap/fase-01-shield-mvp.md`.
- `docs/architecture/15-seguridad.md`.
- `docs/architecture/16-rendimiento-y-validacion.md`.
- RFC-0001 (paralelizacion).
- ADR-0001 (monorepo workspace).
