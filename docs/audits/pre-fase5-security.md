# Pre-Phase 5 Security & Supply Chain Audit (Stage 3)

> Stage 3 deliverable of the master audit plan. Removes security risks in our own
> code, dependencies, exposed errors and external input.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`

## 3.1 Unsafe code

`rg "\bunsafe\b|allow\(unsafe_code\)" crates examples` → **zero real `unsafe`**.
Only a doc-comment mention (`ag-core/src/lib.rs`) and a Spanish code comment
(`ag-storage/src/signed.rs`). Workspace lint `unsafe_code = "deny"` holds with no
per-crate override. No RFC/`// SAFETY:` needed because there is no `unsafe`.

## 3.2 Miri

Not run in this pass (deferred). Justification: there is no `unsafe` in the
workspace, so Miri's primary value (UB detection in `unsafe`) is low. Recommended
as a Low follow-up on pure crates (`ag-core`, `ag-dsl`) if `unsafe` is ever
introduced. Tracked as a non-blocking item.

## 3.3 Supply chain

| Tool | Command | Result |
| --- | --- | --- |
| Advisories | `cargo audit` | **pass** — 0 vulnerabilities across 591 deps, 2026-05-29 |
| Bans/licenses/sources/advisories | `cargo deny check` | **pass** — advisories/bans/licenses/sources ok, 2026-05-29 |

Notes:
- Removed an obsolete suppression: `RUSTSEC-2026-0119` (hickory-proto CPU
  exhaustion) no longer matches any crate after the resolver upgrade; the
  `cargo deny` `advisory-not-detected` warning is gone.
- One justified suppression remains: `RUSTSEC-2023-0071` (RSA Marvin attack),
  transitive via jsonwebtoken/sqlx. Anti-Gravital uses EdDSA exclusively; no code
  path invokes RSA. Kept with rationale in `deny.toml`.
- Duplicate transitive versions exist (`multiple-versions = "warn"`); see
  `pre-fase5-architecture.md` S2-1. Non-blocking.
- Crypto deps reviewed: EdDSA (ed25519-dalek), P-256 (p256), BLAKE3, SHA-2, HMAC,
  rustls (ring) — all current and maintained. Network/parsing deps (hyper, axum,
  hickory, lettre, x509-parser, ciborium) current.

## 3.4 Error handling (no leakage to the end user)

Reviewed every error type that reaches an HTTP client.

### Finding S3-1 — Server errors leaked internal detail (Critical, fixed)
- **Where:** `crates/ag-core/src/error.rs` `impl IntoResponse for AgError`.
- **Root cause:** the JSON body set `message = self.to_string()` for *all*
  variants. For 5xx variants this leaked internal detail to the client:
  - `Database(String)` is built from `DataError -> AgError::Database(err.to_string())`
    (`ag-data/src/lib.rs:67`), wrapping the **raw sqlx error** — which can contain
    SQL text, table/column/constraint names, and connection-failure detail
    (host/user).
  - `Io(std::io::Error)` can embed filesystem paths.
  - `Config`/`Tls`/`Other` expose internal config/TLS detail.
- **Fix:** for `status.is_server_error()` (5xx) the response now returns a generic
  `"internal server error"` message, logs the real error server-side
  (`tracing::error!`), and still returns the stable `code` (e.g.
  `database_error`) so clients can branch on the class. 4xx messages
  (actionable, no internal secrets) are unchanged.
- **Tests:** `server_errors_do_not_leak_internal_detail` (asserts a simulated DB
  credential string is absent from the body) and `client_errors_keep_their_message`.

### Other error surfaces (no issue)
- `ag-storage` `AppError::into_response` returns **status code only** (no body),
  so storage/S3 errors never leak detail. (Stage 1 already removed the DEL-byte
  panic in the same server.)
- `AgError::Auth` (401) messages describe the *failure*
  (`invalid token: <jsonwebtoken error kind>`, `missing Authorization header`),
  never the token value.

## 3.5 Secrets review

- **Logging:** scanned all `trace!/debug!/info!/warn!/error!/println!` for
  token/secret/password/key/credential/connection/authorization/bearer/cookie.
  No secret is logged. The only sensitive-term hits log a peer address (`%peer`)
  or an error message (`%err`), neither of which carries a secret.
- **Connection strings:** `ag-data` logs the pool URL through `sanitize_url`,
  which redacts `user:password@` to `<redacted>@` before any `debug!`.
- **Signed URLs:** `ag-storage` uses constant-time HMAC comparison
  (`pre-fase5-inventory.md`); signatures are not logged.
- **No hardcoded secrets:** OAuth/JWT/API-key material comes from config/runtime,
  not literals (spot-checked `ag-auth`).

## Findings summary

| ID | Severity | Status |
| --- | --- | --- |
| S3-1 | Critical | Fixed — 5xx error bodies sanitized + tests |
| (audit/deny stale ignore) | Low | Fixed — `RUSTSEC-2026-0119` removed |
| Miri on pure crates | Low | Deferred (no `unsafe` present) — tracked |
| Duplicate transitive versions | Low | Non-blocking (see S2-1) |

## Remaining debt

- Miri pass if `unsafe` is ever introduced (Low).
- `RUSTSEC-2023-0071` suppression to be retired when jsonwebtoken/sqlx drop the
  RSA transitive dependency (tracked in `deny.toml`).

## Gate rows advanced

- **Audit:** pass. **Deny:** pass. **Security:** pass for the supply-chain,
  unsafe, error-leakage and secrets review covered here. Per-module deep security
  (Stage 8) and fuzzing (Stage 4) further harden the same surfaces and are
  tracked separately in the release gate.
