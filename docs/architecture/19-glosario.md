# Capitulo 19. Glosario tecnico

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 19
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [18-riesgos-y-mitigaciones.md](./18-riesgos-y-mitigaciones.md)
> Siguiente: [20-apendice-comparativa.md](./20-apendice-comparativa.md)

## 19. Glosario técnico

| Término                       | Definición                                                                                                                |
|-------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Anti-DSL (.ag)                | Lenguaje de definición de dominio del framework. Schema-first.                                                            |
| Axum                          | Framework HTTP de Rust construido sobre Tokio y Tower. Base del Core.                                                     |
| Backpressure                  | Mecanismo por el cual el sistema rechaza trabajo nuevo cuando está saturado. Implementado nativamente en Tower.            |
| Cargo                         | Sistema de build y gestor de paquetes de Rust.                                                                            |
| Cargo-fuzz                    | Herramienta de fuzzing integrada con Cargo.                                                                                |
| Core (capa B)                 | Capa de lógica de negocio del núcleo. Axum router, handlers, estado compartido.                                          |
| Correlation ID                | Identificador único por request que atraviesa todos los logs, traces y errores.                                          |
| Ed25519                       | Algoritmo de firma digital basado en la curva Edwards25519. Default para JWT en Anti-Gravital.                            |
| Flamegraph                    | Visualización de profiling de CPU. Con Rust puro cubre toda la aplicación sin gaps.                                       |
| Fuel (wasmtime)               | Cuota de instrucciones que un plugin WASM puede ejecutar antes de ser interrumpido.                                       |
| GIL                           | Global Interpreter Lock. Mecanismo de CPython que impide ejecución paralela real.                                         |
| Governor                      | Crate Rust para rate limiting basado en token bucket. Thread-safe sin locks contenciosos.                                 |
| HTMX                          | Librería JavaScript pequeña que permite interactividad sin frameworks SPA.                                                |
| JetStream                     | Sistema de persistencia de mensajes de NATS. Permite replay y durabilidad.                                                |
| Knowledge Graph               | Grafo dirigido del proyecto Anti-Gravital. Indexa modelos, endpoints, eventos, dependencias.                              |
| LSP                           | Language Server Protocol. El DSL `.ag` ofrece LSP para autocompletado en editores.                                        |
| Moka                          | Caché concurrente Rust con TinyLFU. Thread-safe sin locks contenciosos.                                                   |
| NATS                          | Sistema de mensajería pub/sub usado por `ag-realtime`.                                                                    |
| OpenAPI                       | Especificación estándar para describir APIs HTTP. Anti-Gravital la genera automáticamente.                                |
| Passkeys                      | Estándar FIDO2/WebAuthn para autenticación sin password.                                                                   |
| Ring                          | Crate Rust de criptografía de bajo nivel. Mantenido por miembros del equipo BoringSSL.                                    |
| Rustls                        | Implementación de TLS 1.3 en Rust puro, sin OpenSSL.                                                                       |
| Schema drift                  | Condición donde la definición de un schema queda desincronizada entre capas. Anti-Gravital la elimina por diseño.         |
| Schema-per-tenant             | Arquitectura multi-tenant donde cada cliente tiene su propio schema en PostgreSQL.                                       |
| Shield (capa A)               | Capa de confianza del núcleo. Pipeline de middleware Tower: TLS, auth, validation, rate limit, RBAC, CORS.                |
| sqlx                          | Crate Rust de acceso a bases de datos con verificación de queries en compile time.                                        |
| TechEmpower                   | Suite de benchmarks estándar de la industria para comparar frameworks web.                                                |
| Tokio                         | Runtime async de Rust. Provee concurrencia M:N mediante tasks livianas sin GC.                                            |
| tokio-console                 | Herramienta de diagnóstico en vivo para aplicaciones Tokio.                                                                |
| Tower                         | Crate Rust para servicios y middleware composables. Base arquitectónica del Shield.                                       |
| WASI                          | WebAssembly System Interface. Estándar para módulos WebAssembly con acceso controlado al sistema.                         |
| wasmtime                      | Runtime WebAssembly embebible en Rust. Host del sistema de plugins.                                                       |
| WebAuthn                      | Estándar W3C para autenticación con factores hardware (passkeys, security keys).                                          |
| Zero-copy                     | Transferencia de datos sin copiarlos en memoria. Reduce overhead de CPU.                                                  |
| Zero-overhead abstraction     | Principio de Rust: una abstracción no debe costar rendimiento frente al código manual equivalente.                       |

---

