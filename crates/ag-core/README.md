# ag-core

> Status: Phases 1-2 — implemented (Shield pipeline and Core runtime). Phase
> exit gates (reference performance, coverage certification, external
> adoption) remain open; see `docs/roadmap/STATUS.md`.
> Criticality: Core.
> Architecture chapter: docs/architecture/06-nucleo-shield-y-core.md
> Governing RFC: docs/rfc/RFC-0002-diseno-shield-mvp.md

## Domain

High-performance HTTP runtime. Combines the Shield layer (a Tower middleware
pipeline) with the Core layer (an Axum router with typed extractors, the error
system and shared state). It is the only mandatory dependency of the
Anti-Gravital ecosystem.

## Implemented

Shield pipeline (`src/shield/`), each layer with unit tests plus E2E tests
against a real server (`tests/shield_*.rs`, `tests/shield_full_pipeline.rs`):

- Structured logging (first layer of the pipeline).
- Payload validation layer.
- CORS layer.
- CSRF layer.
- Rate limiting (`governor`, behind the `rate-limit` feature).
- JWT authentication layer.
- TLS 1.3 (`rustls`, behind the `tls` feature).
- `ShieldConfig` deserializable from TOML.

Core layer (`src/core/`, `src/runtime/`, `src/error.rs`, `src/config/`):

- HTTP/1.1 and HTTP/2 via Axum + Tokio.
- `AgError` / `AgResult` with automatic mapping to HTTP responses.
- Router and typed extractors consumed by generated projects (Phase 2).

## Pending (phase gates, not code)

- Reference performance evidence on recorded hardware (Phase 2 targets).
- Coverage certification and external-adoption exit criteria.

Tracking lives in `docs/roadmap/STATUS.md` and GitHub Issues; this README does
not list debt (CLAUDE.md rule 29).

## References

- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` section 6.
- `docs/architecture/06-nucleo-shield-y-core.md`.
- `docs/roadmap/fase-01-shield-mvp.md`, `docs/roadmap/fase-02-core-mvp.md`.
- `docs/rfc/RFC-0002-diseno-shield-mvp.md`.

## Applicable rules

- Complies with CLAUDE.md rules 14 and 15 (crates and dependencies).
- `ag-core` depends on no other Anti-Gravital crate.
- Independent semantic versioning.
- No `unsafe`; see `unsafe_code = "deny"` in the root `Cargo.toml`.
