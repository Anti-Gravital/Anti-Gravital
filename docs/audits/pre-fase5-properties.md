# Pre-Phase 5 Property-Based Testing (Stage 5)

> Stage 5 deliverable of the master audit plan. Verifies invariants, not just
> concrete cases, on the most security-critical pure logic.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`
- **Tool:** `proptest` 1.x, added as a workspace dev-dependency (test-only, not a
  runtime dependency). 256 generated cases per property by default.

## Why ag-storage first

Property testing has the highest return on hand-written, security-critical pure
logic. `ag-storage` holds two such surfaces:
- **Signed URLs** — custom HMAC-SHA256 sign/verify (auth + integrity boundary).
- **Object-key path resolution** — the path-traversal safety boundary.

The cache (moka-backed) and most other roundtrips are already covered by example
tests and the Stage 4 fuzzers; property-testing them adds little over the
flakiness/async cost, so they are listed as follow-up (§ Recommended).

## Properties added

### `ag-storage` signed URLs (`signed.rs::prop_tests`)
| Property | Invariant |
| --- | --- |
| `fresh_token_always_verifies` | sign(secret,key,future) then verify(secret,key,token) = `Ok` |
| `different_key_is_rejected` | a token for key A never verifies for key B (A≠B) |
| `different_secret_is_rejected` | a token under secret S1 never verifies under S2 (S1≠S2) |
| `tampered_signature_is_rejected` | flipping any signature byte → verify errors |
| `expired_token_is_rejected` | a past `expires_at` → `Err(Expired)` |

### `ag-storage` path confinement (`store/mod.rs::prop_tests`)
| Property | Invariant |
| --- | --- |
| `resolve_path_never_escapes_root` | for ANY input key, `resolve_path` either errors or returns a path inside the canonical root — never an `Ok` path that escapes |

This is the strongest statement of the path-traversal guarantee: it holds over
arbitrary Unicode keys (`.*`), including `..`, `/`, control chars, and mixed
sequences, not just the hand-picked cases in the existing unit tests.

## Results

`cargo test -p ag-storage prop_tests` → **6 properties, 0 failures** (256 cases
each = ~1,536 generated inputs). Clippy `-D warnings` and `cargo fmt --check`
clean.

## Recommended follow-up (Medium, non-blocking)

Mirrors the plan's invariant list for the remaining crates; valuable but lower
marginal return than the security boundaries above, and partly covered by
existing tests/fuzzers:
- `ag-cache`: set→get roundtrip, TTL expiry removes key, tag invalidation removes
  associated keys, `mget` order, `flushdb` empties. (Covered by 34 example
  tests; would need async proptest harness.)
- `ag-dsl`: parse-then-generate never yields invalid structures; diagnostics have
  valid line/column; codegen never emits invalid Rust identifiers; OpenAPI/
  AsyncAPI output is valid JSON/YAML. (Arbitrary input already fuzzed.)
- `ag-auth`: tampered/expired JWT always fails; API key wrong-hash fails; revoked
  refresh token cannot be reused; WebAuthn challenge single-use.
- `ag-mail`: template with complete vars renders, missing vars fail; retries
  bounded; backoff monotonic.
- `ag-domains`: generated SPF/DKIM/DMARC syntactically valid; DNS sync
  idempotent; ACME renewal not before window.

## Gate rows advanced

Property testing for the critical `ag-storage` boundaries is in place and green.
This contributes to overall test confidence; it is not a standalone gate row but
strengthens **Tests** and **Security**. Broader property coverage is tracked as
follow-up above.
