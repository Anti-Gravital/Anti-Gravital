# Pre-Phase 5 Technical Inventory

> Stage 0 deliverable of the *Master audit, stabilization and hardening plan
> (pre-Phase 5)*. This document is an exact photograph of the real state of the
> repository at the moment of the audit. It does not propose changes; it records
> what exists.

- **Audit date:** 2026-05-29
- **Branch at capture:** `p5-native-cache-server`
- **Roadmap declared state (`docs/roadmap/STATUS.md`):** Phase 4.5 technical
  implementation complete; corrective work pre-Phase 5 in progress.
- **Workspace version:** `0.0.0` (Rust edition 2021, `rust-version = "1.79"`).
- **Method:** static read of every `crates/*/Cargo.toml`, `src/`, `tests/`, the
  three master documents, `README.md`, `docs/roadmap/STATUS.md`, `docs/DEBT.md`,
  and `.github/workflows/`. Line counts and test-function counts are raw
  `find`/`grep` measurements over `src/` and `tests/` (counting `#[test]` and
  `#[tokio::test]` attributes); they are indicative, not a coverage report.

## Summary

The workspace declares **18 crates**: 12 with functional code and tests, and 6
honest Phase 0 skeletons. The skeleton crates each carry a `//!` header that
explicitly states "this crate does not yet contain functional code", which is
consistent with ADR-0009 rule 1 (no crate may be mislabelled). No functional
crate falsely declares itself a skeleton, and no skeleton crate hides real code.

| Classification | Crates | Count |
| --- | --- | --- |
| Functional | ag-core, ag-dsl, ag-cli, ag-auth, ag-realtime, ag-cache, ag-storage, ag-lsp, ag-mail, ag-domains | 10 |
| Partial | ag-data, ag-observe | 2 |
| Skeleton (Phase 0) | ag-ai, ag-cloud, ag-migrate, ag-mobile, ag-ui, ag-wasm-host | 6 |

"Partial" means the crate is functional for its current declared scope but has
explicitly deferred capabilities tracked in `docs/DEBT.md` (ag-data: DSL-typed
ORM/RLS, DEBT-009; ag-observe: thin but complete for its current contract).

## Per-crate inventory

Legend for the `Class` column: **F** = functional, **P** = partial, **S** =
skeleton. LOC is `src/` only. "Tests" counts `#[test]`/`#[tokio::test]` in
`src/` + `tests/`.

### Core tier

#### ag-core — F
- **Real state:** Functional. Shield security pipeline (logging, validation,
  CORS, CSRF, rate-limit, auth-jwt, TLS) + Core router on Axum/Tokio.
- **Public API:** `Shield`, `ShieldConfig`, `Shield::serve`, `Shield::apply`;
  extractors `State<T>`, `Path<T>`, `Query<T>`, `ValidatedBody<T>`, `Claims<T>`;
  `response::{Json, PlainText, BodyStream}`; `AgError`.
- **Tests:** ~95 test fns; dedicated `tests/` dir and `benches/` (criterion).
- **Cargo features:** `default = [validation, cors, csrf, logging, rate-limit,
  auth-jwt, tls]`; each toggle is granular; `tls`/`auth-jwt`/`rate-limit` gate
  their heavy deps.
- **Main deps:** axum, tower, tower-http, hyper, hyper-util, rustls, governor,
  jsonwebtoken.
- **Open debt:** none crate-specific in DEBT.md.
- **Risks:** Phase 1 throughput/latency exit criteria (>=300K req/s, p99 <=1ms)
  not yet met on reference hardware (open in STATUS.md). LOC ~2658.

#### ag-dsl — F
- **Real state:** Functional. DSL v0.1 through v0.7 (models, endpoints,
  validations, relations, auth/policies, events, mail/domain/template blocks).
- **Public API:** lexer, parser, AST, semantic analysis, diagnostics, codegen
  for Rust/SQL/TypeScript/OpenAPI/AsyncAPI; `ag generate`/`schema lint`/`diff`
  consume it.
- **Tests:** ~153 test fns. Reported coverage 95%+ (cargo-llvm-cov, STATUS.md).
- **Cargo features:** none.
- **Main deps:** logos (lexer), chumsky (parser).
- **Open debt:** none crate-specific.
- **Risks:** 24h fuzz gate pending (STATUS.md Phase 3, `- [/]`; issue #156); a lexer panic
  on integers > i64::MAX was found and fixed (commit ff85c6f). LOC ~6829 (largest
  crate).

#### ag-cli — F
- **Real state:** Functional. Commands `new`, `dev`, `build`, `generate`,
  `schema lint`, `schema diff`, `domains check`, `domains sync`, `mail test`.
- **Public API:** binary `ag`; three embedded templates via `include_str!`.
- **Tests:** ~6 test fns.
- **Cargo features:** none.
- **Main deps:** clap.
- **Open debt:** none crate-specific.
- **Risks:** Lowest test density relative to surface area among functional
  crates (CLI is integration-heavy; candidate for more E2E coverage). LOC ~882.

#### ag-wasm-host — S
- **Real state:** Skeleton (Phase 0). `//!` declares no functional code. WASI
  plugin host deferred to Phase 9. LOC 5. No tests.

### Standard tier

#### ag-auth — F
- **Real state:** Functional. WebAuthn/FIDO2 (CBOR/COSE), OAuth2 PKCE
  (Google/GitHub), JWT Ed25519, API keys (BLAKE3), refresh-token blacklist.
- **Public API:** `AgAuth::new(config, http_client)`, `JwtSigner`/`JwtVerifier`,
  `AuthMailer` (under `mail` feature).
- **Tests:** ~37 test fns.
- **Cargo features:** `default = []`; `persistent = [ag-data]`; `mail = [ag-mail]`.
  ag-mail does NOT depend on ag-auth (dependency rule 6 respected).
- **Main deps:** jsonwebtoken, p256, ed25519-dalek, oauth2, passkey-types,
  ciborium, blake3, sha2, hmac.
- **Open debt:** none crate-specific.
- **Risks:** Security-critical surface; primary target for Stage 3/4/8 (fuzzing
  JWT/claims/WebAuthn payloads, secret-leak review). LOC ~2017.

#### ag-data — P
- **Real state:** Partial. PostgreSQL pool, declarative config, embedded
  migrations runner. Typed ORM / RLS / multi-tenancy are DSL-generated and
  deferred (DEBT-009).
- **Public API:** `DataConfig`, `DbPool`, `connect()`, `run_migrations()`,
  `DataError -> AgError`.
- **Tests:** ~5 test fns.
- **Cargo features:** none.
- **Main deps:** sqlx (postgres, migrate, macros).
- **Open debt:** DEBT-009 (open, Phase 5+).
- **Risks:** Thinnest functional crate; SQL error sanitization and connection
  string leakage are Stage 8 review items. LOC ~211.

#### ag-realtime — F
- **Real state:** Functional. InProcess pub/sub bus, external NATS client
  (TLS/JetStream), Axum WebSocket + SSE handlers, optional event persistence.
- **Public API:** `AgRealtime::new`, `RealtimeBus` enum, `EventBus`,
  `ws_handler`, `sse_handler`, `EventBuffer`/`replay_into_bus`.
- **Tests:** ~34 test fns; `tests/` dir incl. `load_50k.rs` (marked `#[ignore]`,
  manual gate).
- **Cargo features:** `nats-external = [async-nats]`; `event-persistence`.
- **Main deps:** async-nats, futures-util, axum (ws), tokio-stream.
- **Open debt:** DEBT-007 closed (50k load test exists but `#[ignore]`),
  DEBT-008 closed (event persistence).
- **Risks:** 50k-connection claim verified only in an ignored debug-mode test;
  Stage 7 must confirm backpressure and graceful shutdown under load. LOC ~1102.

#### ag-cache — F
- **Real state:** Functional. L1 moka in-memory; optional native in-process
  RESP2 server (RFC-0005) compatible with Redis clients; tag invalidation, TTL.
- **Public API:** L1 cache API, tag invalidation, `NativeCacheServer` (feature
  `native-server`): GET/SET/DEL/EXISTS/MGET/MSET/EXPIRE/TTL/KEYS/PING/FLUSHDB/
  DBSIZE/COMMAND.
- **Tests:** ~34 test fns; `tests/` dir incl. 13 RESP2 integration tests via raw
  TcpStream.
- **Cargo features:** `native-server = [dashmap]`.
- **Main deps:** moka, dashmap.
- **Open debt:** DEBT-004 closed (native L2). External multi-instance Redis L2
  remains a documented TECH-DEBT in `lib.rs`.
- **Risks:** Native RESP2 parser is external-input-facing; Stage 4 fuzz target
  recommended (malformed RESP2, oversized bulk strings, extreme TTL). LOC ~1168.

#### ag-storage — F
- **Real state:** Functional. Native filesystem store (atomic write-then-rename,
  path confinement), embedded Axum HTTP server, image processing, S3/MinIO
  backend, HMAC-SHA256 signed URLs (constant-time compare).
- **Public API:** `AgStore` enum (Native | S3), `ImageProcessor`,
  `sign_url`/`verify_signed_url`, `S3Store` (feature `s3`).
- **Tests:** ~57 test fns.
- **Cargo features:** `default = []`; `auth = [ag-auth]`; `s3 = [object_store]`.
- **Main deps:** image, hmac, sha2, object_store (s3).
- **Open debt:** none crate-specific.
- **Risks:** Path traversal, MIME handling, decompression bombs are explicit
  Stage 4/8 fuzz + review targets. LOC ~2100.

#### ag-observe — P
- **Real state:** Partial/functional for scope. Structured tracing, OTLP export,
  Prometheus metrics via Axum, idempotent init, JSON/Text log format.
- **Public API:** `ObserveConfig`, `init()`, metrics layer, Grafana dashboards
  in `dashboards/`.
- **Tests:** ~12 test fns.
- **Cargo features:** `dev-console = [console-subscriber]`.
- **Main deps:** tracing, tracing-subscriber, tracing-opentelemetry,
  opentelemetry, opentelemetry_sdk, metrics, metrics-exporter-prometheus.
- **Open debt:** workspace TECH-DEBT note on opentelemetry-otlp/protoc (Windows
  CI) in root `Cargo.toml`.
- **Risks:** Metric cardinality and PII/secret leakage in logs are Stage 8
  review items. LOC ~383.

### Deferred-standard tier

#### ag-mail — F
- **Real state:** Functional. `MailSender` trait + `SmtpSender` (lettre+rustls) +
  provider adapters; `StringTemplate` with `{{var}}`; async retry/backoff queue;
  persistent queue over ag-data; metrics.
- **Public API:** `MailSender`, `SmtpSender`, `MailTemplate`,
  `StringTemplate`, `InMemoryQueue`, `PersistentQueue`, `NullSender` (test-utils).
- **Tests:** ~42 test fns; `tests/` dir.
- **Cargo features:** `default = [smtp, templates, metrics]`; opt-in
  `mta` (mail-send/mail-auth/hickory), `queue-persistent` (ag-data), `test-utils`.
- **Main deps:** lettre (smtp, default-on protocol client).
- **Open debt:** DEBT-001 closed (persistent queue), DEBT-002 closed (custom
  headers), DEBT-003 open (external template engines, Phase 5+).
- **Risks:** Template rendering with missing/extra vars and malformed headers are
  Stage 4 fuzz targets. Outbound-only by design (not an MTA). LOC ~1942.

#### ag-domains — F
- **Real state:** Functional. `DnsProvider` trait + Cloudflare adapter; ACME
  (Let's Encrypt) issue + scheduled renewal; SPF/DKIM/DMARC generation; multi-
  resolver propagation checks.
- **Public API:** `DnsProvider`, `CloudflareProvider`, ACME `issue()` /
  `issue_with_credentials()` / `spawn_renewal_task()`, `apply_mail_records`,
  `PropagationChecker`.
- **Tests:** ~41 test fns.
- **Cargo features:** `default = [acme, propagation]`; opt-in `cloudflare`
  (reqwest).
- **Main deps:** instant-acme, rcgen, x509-parser, hickory-resolver (acme +
  propagation are protocol clients, default-on; the *provider* adapter is opt-in).
- **Open debt:** DEBT-005 closed (notAfter parsing for scheduled renewal),
  DEBT-006 open (additional DNS adapters, opt-in, non-blocking).
- **Risks:** ACME renewal loop scheduling, malformed cert/notAfter, IDN domains
  are Stage 4/7 targets. LOC ~2325.

### Optional / infrastructure-optional tier (skeletons)

| Crate | Class | LOC | Declared scope | Target phase |
| --- | --- | --- | --- | --- |
| ag-cloud | S | 5 | Deployment (compose, Fly.io, Railway, k8s) | Phase 5 |
| ag-ai | S | 5 | Knowledge graph + AI-assisted capabilities | Phase 6 |
| ag-migrate | S | 5 | Legacy-framework importers into Anti-DSL | Phase 7 |
| ag-mobile | S | 5 | Dart generator (freezed/dio) + RT clients | Phase 8 |
| ag-ui | S | 5 | SSR (askama) + progressive HTMX | optional |
| ag-wasm-host | S | 5 | WASI plugin host | Phase 9 |

All six declare Phase 0 status honestly in their `//!` header. None is the
target of the pre-Phase 5 hardening (scope is frozen; `ag-cloud` stays a
skeleton until the release gate closes).

## CI workflows present

- `.github/workflows/ci.yml`
- `.github/workflows/quality.yml` (clippy, audit, coverage gate via tarpaulin
  `--fail-under 80`, fuzz smoke)
- `.github/workflows/docs.yml`
- `.github/workflows/pr-autofill.yml`

## Cross-cutting observations (carried into later stages)

1. **Stray file:** `Cargo.toml.tmp.56172.091804dc0ca8` exists at repo root — a
   leftover temp file. Candidate for cleanup in Stage 1.
2. **Fuzz harness:** `fuzz/` exists with targets `fuzz_lexer`, `fuzz_parser`,
   `fuzz_compile` (DSL only). Stage 4 recommends new targets for ag-storage,
   ag-auth, ag-mail, ag-domains, ag-cache.
3. **Benchmarks:** only ag-core has a `benches/` dir. Stage 6 will need real
   HTTP benchmarks and per-crate criterion benches for the perf claims.
4. **Workspace lints:** `unsafe_code = "deny"` and `missing_docs = "warn"` set at
   workspace level; clippy `all = warn`. Stage 1 will tighten to `-D warnings`.
5. **Honesty of claims:** STATUS.md already downgrades several performance claims
   honestly (e.g. the invalid "82 233 req/s" routing-bug number was corrected to
   ~14 478 req/s). Stage 10 should verify all remaining README claims similarly.

## Acceptance criteria status (Stage 0)

This document records, per crate: real state, existing public API, existing
tests, Cargo features, main dependencies, risks, open debt, and the
functional/partial/skeleton classification — satisfying the Stage 0 acceptance
criteria of the master audit plan.
