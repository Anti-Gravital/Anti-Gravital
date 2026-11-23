# Capitulo 19. Glosario tecnico

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 19
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)
> Siguiente: [20-apendice-comparativa.md](./20-apendice-comparativa.md)

The Rust ecosystem continues to evolve rapidly. Axum, Tokio, and sqlx may make breaking changes in future versions. The mitigation is conservative version pinning, exhaustive integration tests against each new version of the core dependencies, and active participation in their communities to anticipate changes.

### 18.6 Risk: community fragmentation

If the Anti-Gravital community fragments (for example, competing forks with divergent features emerge), the ecosystem weakens. The mitigation is an open RFC process that gives a real voice to the community, predictable releases, and a public roadmap.

### 18.7 Risk: post-launch security vulnerabilities

Although Rust eliminates many categories of vulnerabilities, it does not eliminate the logical ones (broken authorization, information leaks, application-level races). The mitigation is the external audit before 1.0, the responsible disclosure program, continuous fuzzing, and CI with static analysis (clippy, cargo-audit, cargo-deny).

---

## 19. Technical glossary

| Term                          | Definition                                                                                                                |
|-------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Anti-DSL (.ag)                | Domain definition language of the framework. Schema-first.                                                                |
| Axum                          | Rust HTTP framework built on Tokio and Tower. Base of the Core.                                                           |
| Backpressure                  | Mechanism by which the system rejects new work when it is saturated. Implemented natively in Tower.                       |
| Cargo                         | Rust's build system and package manager.                                                                                  |
| Cargo-fuzz                    | Fuzzing tool integrated with Cargo.                                                                                        |
| Core (layer B)                | Business logic layer of the core. Axum router, handlers, shared state.                                                    |
| Correlation ID                | Unique identifier per request that traverses all the logs, traces, and errors.                                            |
| Ed25519                       | Digital signature algorithm based on the Edwards25519 curve. Default for JWT in Anti-Gravital.                            |
| Flamegraph                    | CPU profiling visualization. With pure Rust it covers the whole application without gaps.                                 |
| Fuel (wasmtime)               | Quota of instructions that a WASM plugin can execute before being interrupted.                                            |
| GIL                           | Global Interpreter Lock. CPython mechanism that prevents real parallel execution.                                         |
| Governor                      | Rust crate for rate limiting based on token bucket. Thread-safe without contended locks.                                  |
| HTMX                          | Small JavaScript library that allows interactivity without SPA frameworks.                                                |
| JetStream                     | NATS message persistence system. Allows replay and durability.                                                            |
| Knowledge Graph               | Directed graph of the Anti-Gravital project. Indexes models, endpoints, events, dependencies.                             |
| LSP                           | Language Server Protocol. The `.ag` DSL offers LSP for autocompletion in editors.                                         |
| Moka                          | Concurrent Rust cache with TinyLFU. Thread-safe without contended locks.                                                  |
| NATS                          | pub/sub messaging system used by `ag-realtime`.                                                                           |
| OpenAPI                       | Standard specification to describe HTTP APIs. Anti-Gravital generates it automatically.                                   |
| Passkeys                      | FIDO2/WebAuthn standard for passwordless authentication.                                                                  |
| Ring                          | Low-level cryptography Rust crate. Maintained by members of the BoringSSL team.                                           |
| Rustls                        | TLS 1.3 implementation in pure Rust, without OpenSSL.                                                                     |
| Schema drift                  | Condition where the definition of a schema becomes desynchronized between layers. Anti-Gravital eliminates it by design.  |
| Schema-per-tenant             | Multi-tenant architecture where each client has its own schema in PostgreSQL.                                             |
| Shield (layer A)              | Trust layer of the core. Tower middleware pipeline: TLS, auth, validation, rate limit, RBAC, CORS.                        |
| sqlx                          | Rust database access crate with compile-time query verification.                                                          |
| TechEmpower                   | Industry-standard benchmark suite for comparing web frameworks.                                                           |
| Tokio                         | Rust async runtime. Provides M:N concurrency through lightweight tasks without GC.                                        |
| tokio-console                 | Live diagnostic tool for Tokio applications.                                                                              |
| Tower                         | Rust crate for composable services and middleware. Architectural base of the Shield.                                     |
| WASI                          | WebAssembly System Interface. Standard for WebAssembly modules with controlled access to the system.                     |
| wasmtime                      | WebAssembly runtime embeddable in Rust. Host of the plugin system.                                                        |
| WebAuthn                      | W3C standard for authentication with hardware factors (passkeys, security keys).                                          |
