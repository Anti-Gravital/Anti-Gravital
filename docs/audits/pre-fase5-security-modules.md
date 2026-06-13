# Pre-Phase 5 Per-Module Security (Stage 8)

> Stage 8 deliverable of the master audit plan. Systematic security review of each
> functional crate. Cross-references fixes already made in Stages 1, 3, 4 and 7.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`

## ag-core
- TLS 1.3 (rustls/ring), CORS, CSRF (double-submit), rate-limit (governor),
  payload validation, structured logging, request IDs — all present as Shield
  layers with secure defaults; config uses `deny_unknown_fields`.
- **Error sanitization (Stage 3, fixed):** 5xx responses no longer leak internal
  detail; generic message + stable code, real error logged.
- Malformed request does not crash the server (hyper/axum handle parse errors as
  4xx). 4xx/5xx are safe; logs do not leak secrets (Stage 3).
- Verdict: **pass.** Slowloris/size-limit deep load test deferred to Stage 6.

## ag-data
- PostgreSQL via sqlx with **parameterized queries** (no dynamic SQL string built
  from user input). No injection vector in the crate.
- Connection string **redacted** before logging (`sanitize_url`, Stage 3).
- `DataError -> AgError::Database` detail no longer reaches the client (Stage 3
  fix in ag-core).
- Verdict: **pass.**

## ag-dsl
- Arbitrary input is fuzzed (`fuzz_lexer/parser/compile`); a past lexer panic was
  fixed (ff85c6f). The one `unreachable!()` (ast.rs:134) is provably unreachable
  (Stage 1). Codegen emits `todo!()` only inside *generated stub* code, by design.
- Verdict: **pass** (deep 24h fuzz is the manual gate, DEBT-015).

## ag-auth
- **JWT:** `Validation::new(Algorithm::EdDSA)` pins the algorithm → no
  algorithm-confusion (`alg=none`/HS) attack; `exp` validated by default. Signing
  uses EdDSA (ed25519).
- **API keys:** hashed with BLAKE3; verification uses `constant_time_eq` → no
  timing attack.
- **WebAuthn:** attestation/assertion parsing has explicit length guards before
  every slice (Stage 1); challenge is supplied and consumed by the caller
  (single-use is caller state).
- **Refresh tokens:** revocation via `RwLock<HashSet>`; revoked tokens rejected.
- **Secrets:** none logged (Stage 3). OAuth client is injected (timeout is
  caller's responsibility; recommend examples set one).
- Verdict: **pass.**

## ag-cache
- **RESP2 DoS (Stage 4, fixed):** array/bulk lengths bounded (Redis-style limits)
  → malformed `$N` no longer triggers unbounded allocation. Parser fuzzed
  (853k runs, 0 crashes). TTL/flush/tag invalidation covered by example tests.
- Open: no connection cap on the native server (DEBT-013, Low).
- Verdict: **pass.**

## ag-realtime
- WebSocket/SSE/NATS; per-connection tasks (standard). External NATS via
  `async_nats` (reconnection handled by the client). No unbounded channels.
- Open: `event-persistence` `append` is synchronous per-event I/O (DEBT-012,
  Medium, opt-in feature).
- 50k-connection scalability is the manual load gate (Stage 6/7, DEBT-007 test
  exists, `#[ignore]`).
- Verdict: **pass** for security; scalability/performance proof is the Stage 6
  gate, tracked in issue #157 (the `#[ignore]` load test already exists).

## ag-storage
- **Path traversal:** `validate_key` rejects `..`/`//`/leading-trailing `/`/null/
  control bytes; `resolve_path` confines to the canonical root. **Property-tested**
  (Stage 5) over arbitrary keys; **fuzzed** (Stage 4, 4.4M runs).
- **Header-injection panic (Stage 1, fixed):** DEL-byte key no longer panics; safe
  header encoding.
- **Signed URLs:** HMAC-SHA256, constant-time compare; **property-tested** (Stage
  5: roundtrip, wrong key/secret, tamper, expiry).
- Payload size limit enforced (`TooLarge`); errors return status only (no path
  leak). Image decoding (decompression bombs) is a recommended fuzz follow-up
  (DEBT-014).
- Verdict: **pass.**

## ag-observe
- **Metric cardinality (Stage 8, fixed):** `record_request` now documents that
  `endpoint` must be the matched route pattern, never the raw path, to avoid
  unbounded label cardinality. No productive caller passes a raw path today.
- Logs do not leak tokens/passwords/API keys (Stage 3). Init is idempotent.
- Verdict: **pass.**

## ag-mail
- Outbound-only (not an MTA). Templates validate variables; malformed headers are
  handled (DEBT-002 closed). Queue is bounded with backpressure (Stage 7).
- **External timeout (Stage 7, fixed):** The provider HTTP adapter now has a 30s timeout.
- Secrets (SMTP creds, tokens) not logged.
- Verdict: **pass.**

## ag-domains
- DNS provider trait + Cloudflare adapter; ACME issue/renewal; SPF/DKIM/DMARC
  generation; propagation checks.
- **External timeout (Stage 7, fixed):** Cloudflare client now has a 30s timeout.
- ACME renewal sleeps to the renewal window (DEBT-005 closed); does not loop
  hot. Certificate parsing via x509-parser.
- Verdict: **pass** (DNS/cert malformed-input fuzz is a follow-up, DEBT-014).

## ag-cli
- Commands return clear errors and correct exit codes; generated templates
  compile; no secrets printed; writes confined to the target directory.
- Verdict: **pass.**

## Summary

All ten functional crates **pass** the per-module security review. Module-level
issues found during the audit were fixed in Stages 1/3/4/7/8; remaining items are
non-blocking debt (DEBT-012..015). No Blocker or Critical remains open.

## Gate rows advanced

- **Security:** pass (Stage 3 supply-chain/errors/secrets + Stage 8 per-module).
- **API/DX:** the ergonomics/type-safety review (Stage 2.2) plus these module
  APIs are consistent and hard to misuse; the one footgun (metric cardinality)
  is now documented. Marking API/DX pass.
