# ag-domains control plane + ag-edge data plane (ADR-0012 / RFC-0011, phases A-D)

## Summary

Extends `ag-domains` from a declarative DNS+TLS library into a native domain
attachment and serving control plane, and adds the `ag-edge` data-plane crate,
per ADR-0012 / RFC-0011 (phases A, B, C and D). Additive and non-regressive: the
existing declarative library and routing are unchanged.

Phase B adds runnable edge listeners in `ag-edge`: an HTTP listener (ACME
HTTP-01 responder + Host/:authority routing + canonical redirects, fail-closed)
and an HTTPS listener with SNI certificate selection from a rustls cert store
(PEM bridge from ACME issuance). Phase C adds the `/v1/domains/...` REST API in
`ag-domains` backed by the native store. Phase D adds an optional Postgres store
(`sql-store`). All have real TCP/TLS/HTTP tests; SQL via `#[ignore]` tests.

Changes:

- `crates/ag-domains/src/hostname.rs`: hostname normalization, IDN/Punycode
  identity, apex/subdomain/wildcard classification and validation.
- `crates/ag-domains/src/attachment.rs`: attachment state machine
  (ownership/dns/tls/routing dimensions + derived lifecycle + activation rule).
- `crates/ag-domains/src/store.rs`: `AttachmentStore` trait, native
  `InMemoryStore` and `JsonFileStore`, tombstones.
- `crates/ag-domains/src/ownership.rs`: TXT token generation + verification.
- `crates/ag-domains/src/instructions.rs`: DNS instruction engine + BIND export.
- `crates/ag-domains/src/caa.rs`: CAA preflight (pure decision + resolver query).
- `crates/ag-domains/src/diagnostics.rs`: expected vs observed comparison.
- `crates/ag-domains/Cargo.toml`, workspace `Cargo.toml`: `idna`, `uuid` deps.
- `crates/ag-edge/*`: new crate — pure logic (`router::resolve_hostname`,
  `tls::SniCertStore` + `allow_on_demand`, `redirect::CanonicalPolicy`,
  `challenge::Http01ChallengeStore`); `server` feature (`server::serve_http`);
  `tls` feature (`cert::CertStore` + `SniCertResolver` + `server::serve_https`).
- `crates/ag-domains/src/api.rs`: `api` feature — axum `/v1/domains/...` REST API.
- `crates/ag-domains/src/sql_store.rs`: `sql-store` feature — Postgres
  `SqlAttachmentStore` (sqlx) + `migrations/0001_*.sql`; `#[ignore]` tests.
- `AttachmentStore` trait is now async + `&self` (interior mutability); the REST
  API holds `Arc<dyn AttachmentStore>` (no external Mutex); the CLI runs domain
  commands on the tokio runtime.
- `crates/ag-cli/src/main.rs`: `ag domains attach|instructions|export-zone|status|list|verify|detach`.
- Workspace `Cargo.toml`: `hyper-util` gains `server-auto` + `service` features.
- Governance/docs: `RFC-0011`, `ADR-0012`, `docs/ag-domains/**`,
  `openapi/ag-domains.v1.yaml` (implemented contract), README/CHANGELOG/DEBT updates.

## Phase affected

Phase 5 (extends Phase 4.5 `ag-domains`). RFC-0011 phase A.

## Type of change

- Feature implementation (additive; new crate `ag-edge`; native-first defaults)
- Documentation
- Test

## Related documents

- `docs/rfc/RFC-0011-ag-domains-control-plane.md`
- `docs/adr/0012-ag-domains-control-plane.md`
- `docs/DEBT.md` — DEBT-017 (eTLD+1/PSL), DEBT-018 (deferred phases E-F)
- `docs/ag-domains/BACKLOG.md` — blueprint gap analysis

## Test plan

- [x] `cargo test -p ag-domains --all-features` — 84 unit + 4 REST + 3 ignored SQL + doctests pass
- [x] `cargo test -p ag-edge --features tls` — 25 unit + 5 HTTP + 1 real HTTPS/SNI pass
- [x] `cargo build -p ag-cli` — compiles; manual flow smoke-tested
      (attach, list, detach + tombstone re-claim block)
- [x] `cargo clippy -p ag-domains --all-features --all-targets -- -D warnings` — clean
- [x] `cargo clippy -p ag-edge --all-features --all-targets -- -D warnings` — clean
- [x] `cargo clippy -p ag-cli --all-targets -- -D warnings` — clean
- [x] `cargo build --workspace` — no errors

## Exit criteria advanced

- RFC-0011 phases A, B, C and D delivered: control-plane library + manual CLI
  flow, live edge listeners (HTTP-01 + routing + HTTPS/SNI), the REST API, and
  the optional Postgres store.
- Deferred phases E-F (provider automation, registrar) tracked in DEBT-018.

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation (RFC-0011 + ADR-0012 authorize the scope)
- [x] Does not break architecture (additive; existing API/CLI unchanged)
- [x] No unnecessary complexity added
- [x] No circular dependencies (ag-edge depends on ag-domains only)
- [x] Compiles
- [x] Tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
