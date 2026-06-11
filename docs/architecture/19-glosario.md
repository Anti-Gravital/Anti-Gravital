# Capitulo 19. Glosario tecnico

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 19
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)
> Siguiente: [20-apendice-comparativa.md](./20-apendice-comparativa.md)

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
| Zero-copy                     | Data transfer without copying it in memory. Reduces CPU overhead.                                                         |
| Zero-overhead abstraction     | Rust principle: an abstraction must not cost performance versus the equivalent manual code.                               |
| ACME                          | Automatic Certificate Management Environment. Protocol for automatic issuance and renewal of TLS certificates (Let's Encrypt). Used by `ag-domains` since Phase 4.5. |
| DKIM                          | DomainKeys Identified Mail. Mail authentication mechanism by cryptographic signature of the sender domain. Generated by `ag-domains` for `ag-mail`. |
| SPF                           | Sender Policy Framework. DNS record that enumerates the servers authorized to send mail on behalf of the domain. Generated by `ag-domains` for `ag-mail`. |
| DMARC                         | Domain-based Message Authentication, Reporting and Conformance. Policy that indicates how to treat mail that fails SPF or DKIM. Generated by `ag-domains`. |
| MTA                           | Mail Transfer Agent. Complete mail server (Postfix, Stalwart). `ag-mail` v1 is **NOT an MTA**: it only sends outbound, it does not receive inbound. |
| DnsProvider                   | `ag-domains` trait that abstracts DNS providers through adapters. Initial adapter: Cloudflare. Designed to add Route53, Namecheap, etc. with contract tests. |
| Deferred standard             | Crate classification introduced by `ADR-0007`. Crate with standard maturity that is NOT installed by default in official templates. `ag-mail` is the first case. |

---

