# Capitulo 6. Arquitectura del nucleo (ag-core): Shield y Core

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 6
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [05-ecosistema-modulos.md](./05-ecosistema-modulos.md)
> Siguiente: [07-anti-dsl.md](./07-anti-dsl.md)

## 6. Core architecture (`ag-core`): Shield and Core

The core of Anti-Gravital is organized into two conceptual layers within a single Rust process. The separation is not physical: there is no IPC, no FFI, no shared memory between runtimes. The two layers communicate through ordinary Rust function calls, with zero measurable overhead. The separation is logical and exists for two reasons: architectural clarity for the developer, and the future possibility of extracting the Shield as an independent gateway if a use case justifies it.

### 6.1 Core diagram

```
┌─────────────────────────────────────────────────────────────┐
│                  ag-core · Single Process                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA A — The Shield                  │       │
│   │  (Tower middleware composable pipeline)         │       │
│   │                                                 │       │
│   │   ┌─────────┐  ┌─────────┐  ┌─────────────┐     │       │
│   │   │ TLS 1.3 │─►│   JWT   │─►│ Rate Limit  │     │       │
│   │   │ rustls  │  │ Ed25519 │  │  governor   │     │       │
│   │   └─────────┘  └─────────┘  └──────┬──────┘     │       │
│   │                                    │            │       │
│   │   ┌─────────┐  ┌─────────┐  ┌──────▼──────┐     │       │
│   │   │  CORS   │◄─│  RBAC   │◄─│ Validación  │     │       │
│   │   │  CSRF   │  │ Guards  │  │  Schema     │     │       │
│   │   └─────────┘  └─────────┘  └─────────────┘     │       │
│   └─────────────────────┬───────────────────────────┘       │
│                         │                                   │
│              Llamada de función Rust (0ns)                  │
│                         ▼                                   │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA B — The Core                    │       │
│   │  (Axum router · Handlers · Estado)              │       │
│   │                                                 │       │
│   │   ┌────────────┐   ┌─────────────────────┐      │       │
│   │   │  Router    │   │  Business handlers  │      │       │
│   │   │  Axum      │──►│  (generados por DSL)│      │       │
│   │   └────────────┘   └──────────┬──────────┘      │       │
│   │                               │                 │       │
│   │   ┌────────────┐   ┌──────────▼──────────┐      │       │
│   │   │ Extractores│   │  Estado compartido  │      │       │
│   │   │  tipados   │   │  AppState           │      │       │
│   │   └────────────┘   └─────────────────────┘      │       │
│   └─────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                            │
                  cargo build --release
                            ▼
                ┌────────────────────────┐
                │  Single Static Binary  │
                │  FROM scratch Docker   │
                └────────────────────────┘
```

### 6.2 The Shield: the trust layer

The Shield is responsible for everything that happens before a request is considered trusted and delivered to the business code. It is implemented as a pipeline of Tower layers, the same composable model that Axum uses internally. Each layer is optional and is configured from the project's `schema.ag`.

The technical stack of the Shield is: Tokio as the M:N async runtime (it multiplexes millions of tasks over a fixed thread pool of size equal to available CPUs), Tower as the composable middleware model, rustls for TLS 1.3 without an OpenSSL dependency, serde and serde_json for zero-copy serialization where possible, ring for low-level cryptographic primitives, governor for rate limiting with a lock-free token bucket algorithm.

The standard layers of the Shield, in execution order over an incoming request:

The first layer is TLS termination, managed by rustls. It supports TLS 1.3 with modern cipher suites, OCSP stapling, and ALPN for HTTP/1.1 vs HTTP/2 negotiation. For environments where TLS termination is performed by an external load balancer (Cloudflare, AWS ALB, Nginx), this layer is disabled with an option in the schema.

The second layer is payload deserialization and validation. For requests with a body, the contract defined in the `.ag` is applied: types, length constraints, email format, regex, numeric ranges. A violation produces a 422 error with structured detail about which field failed and why.

The third layer is authentication. It supports JWT signed with Ed25519 (Edwards25519 curve, faster and more secure than RS256), Passkeys/WebAuthn (FIDO2), API keys, and cookie-based sessions. Verification is eager for endpoints marked as `auth required`.

The fourth layer is rate limiting. Implemented with governor over a token bucket algorithm, it supports limits per IP, per authenticated user, per endpoint, and per combinations. The limits are declared in the schema.

The fifth layer is RBAC authorization. The policies are declared in the `.ag` as expressions that are evaluated against the JWT claims and the request parameters. For example: `policy "user.role == ADMIN || user.id == params.id"`.

The sixth layer is CORS and CSRF. Configured by default with secure values (no wildcard); any deviation requires explicit declaration.

### 6.3 The Core: the business logic layer

The Core is where 80% of the application code that the developer writes lives. It is Axum with a thin layer of conventions on top.

The handlers have a signature generated by the DSL compiler from the declared endpoint:

```rust
// Generado automáticamente por `ag generate` desde schema.ag
// El desarrollador solo escribe el cuerpo del handler.
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedBody(req): ValidatedBody<CreateUserRequest>,
    Claims(claims): Claims<AuthClaims>,
) -> Result<Json<User>, AgError> {
    // El desarrollador solo escribe esto:
    let user = state.db.users()
        .create(CreateUserParams {
            email: req.email,
            name: req.name,
            created_by: claims.user_id,
        })
        .await?;

    state.events.emit("user.created", &user).await?;
    Ok(Json(user))
}
```

The type `ValidatedBody<T>` guarantees that the body already passed the Shield's validation. The type `Claims<T>` guarantees that the JWT was already verified. The type `AgError` is an enum that covers all the errors declared in the endpoint, and the conversion to an HTTP response is automatic via `IntoResponse`.

The application state (`AppState`) is a generated struct that contains clients to the project's resources: the database pool, the NATS client, the Redis client, the S3 client. It is built at binary startup and shared by reference (cheap `Arc` clones) among all handlers.

### 6.4 Error handling

The Anti-Gravital error system follows three principles. The first: each endpoint explicitly declares which errors it can produce in its `.ag` definition. This produces a typed `EndpointError` enum in which each variant is a specific error. The second: errors propagate with Rust's `?` operator, and the conversion to an HTTP response is automatic and consistent. The third: no error is silently discarded. Unexpected errors produce a structured 500 with a correlation ID that is linked to the stack trace in the tracing system.

### 6.5 Runtime and Tokio configuration

Anti-Gravital uses Tokio in multi-thread mode with default configuration: one worker per available CPU, a blocking pool of 512 threads. For standard IO-bound workloads this configuration is optimal. The schema allows adjustments:

```yaml
runtime:
  workers: auto              # Por defecto = núm CPUs
  blocking_threads: 512
  thread_stack: 2MB
  shutdown_timeout: 30s
```

---

