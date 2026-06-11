# Capitulo 15. Modelo de seguridad

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 15
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [14-observabilidad-ag-observe.md](./14-observabilidad-ag-observe.md)
> Siguiente: [16-rendimiento-y-validacion.md](./16-rendimiento-y-validacion.md)

## 15. Security model

Security is a cross-cutting concern, not a module. This section documents the project's guarantees and practices.

### 15.1 Guarantees by construction

Rust eliminates by construction four categories of bugs that historically represent more than 70% of the critical vulnerabilities in systems software: use-after-free, buffer overflows, data races, and null pointer dereferences. These guarantees are at the compiler level, not at runtime; they do not require GC or runtime checks.

Anti-Gravital prohibits the use of `unsafe` in all the framework code except in blocks that are explicitly justified, documented, and reviewed by at least two maintainers. Each `unsafe` block comes accompanied by a comment that explains why it is necessary and which invariants it preserves.

### 15.2 Cryptography practices

The cryptographic primitives are imported from the `ring` crate, maintained by members of Google's BoringSSL team. No custom cryptography is rolled. The default algorithms are Ed25519 for signatures, ChaCha20-Poly1305 for AEAD, Argon2id for password hashing, and TLS 1.3 for transport. Legacy algorithms (RSA, AES-CBC, SHA-1) are available only for explicit interoperability.

### 15.3 Responsible disclosure policy

The repository maintains a `SECURITY.md` file with contact addresses (primary `anti@gravitalcloud.com`, backup `angelnereira@gravitalcloud.com`) and a clear policy: vulnerabilities are reported privately, the team confirms receipt within 48 hours, publishes a patch within 30 days for critical vulnerabilities, and a CVE with credit to the reporter.

### 15.4 Audits

Before the stable 1.0 version, the Shield component of the framework undergoes an external audit by a company specialized in Rust systems security (Trail of Bits, NCC Group, or equivalent). The audit report is published with the release.

### 15.5 Continuous fuzzing

The DSL parser and the HTTP parser undergo continuous fuzzing with `cargo-fuzz`. The CI runs fuzzing corpora on each PR; before 1.0, at least 72 hours of fuzzing without crashes are completed on each parser.

---

