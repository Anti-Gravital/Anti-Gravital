# Pre-Phase 5 Adversarial Resilience & Fuzzing (Stage 4)

> Stage 4 deliverable of the master audit plan. Demonstrates that malformed,
> extreme or malicious input does not break the framework.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`
- **Toolchain:** `cargo +nightly fuzz` (cargo-fuzz, libFuzzer), nightly present.

## 4.1 Existing targets (DSL)

`fuzz/fuzz_targets/`: `fuzz_lexer`, `fuzz_parser`, `fuzz_compile` (ag-dsl). These
predate the audit; a prior lexer panic on integers > i64::MAX was found and fixed
(commit ff85c6f, recorded in STATUS.md). CI runs a 60s smoke on them
(`quality.yml`).

## 4.2 New targets added this stage

Targets for the highest-risk external-input parsers (network and path/token
boundaries), wired in `fuzz/Cargo.toml`:

| Target | Function under test | Surface |
| --- | --- | --- |
| `fuzz_resp` | `ag_cache::server::resp::read_command` | RESP2 wire parser (native cache server, network input) |
| `fuzz_storage_key` | `ag_storage::store::validate_key` | object-key validation (path-traversal boundary) |
| `fuzz_signed_url` | `ag_storage::verify_signed_url` | signed-URL token parsing (HMAC verify) |

To make the RESP2 parser fuzzable from a `Cursor`, `read_command` was made
generic over `AsyncBufReadExt + AsyncReadExt + Unpin` (the live TCP caller is
unchanged — `BufReader<OwnedReadHalf>` still satisfies the bound).

## 4.3 Finding fixed: RESP2 unbounded-allocation DoS (Critical)

- **Where:** `crates/ag-cache/src/server/resp.rs` `read_command`.
- **Root cause:** the multibulk count (`*N`) drove `Vec::with_capacity(count)` and
  the bulk-string length (`$L`) drove `vec![0u8; len + 2]`, both with **no upper
  bound**. A tiny header such as `*1\r\n$999999999999\r\n` (reachable by any
  client connecting to the native cache port) triggered a ~1 TB allocation →
  OOM/abort. `len + 2` could also overflow `usize`. This is a trivial,
  amplified DoS.
- **Fix:** bound the bulk-string length to `MAX_BULK_LEN` (512 MiB, Redis
  `proto-max-bulk-len`) and the array count to `MAX_ARRAY_ELEMENTS` (1 Mi, Redis
  `proto-max-multibulk-len`), reject oversized headers before allocating, and
  never pre-allocate the attacker-controlled `count`.
- **Tests:** 6 unit tests in `resp.rs` including
  `oversized_bulk_len_is_rejected_without_allocating`,
  `oversized_array_count_is_rejected`, `bulk_len_at_usize_max_does_not_overflow`.
- **Commit:** `37c281f` (fix), `b6b8a50` (fuzz target).

Note: this was found by *reading the parser in order to fuzz it* — the fuzz
target now guards against regressions.

## 4.4 Smoke results (2026-05-29)

`cargo +nightly fuzz run <target> -- -max_total_time=60`:

| Target | Runs | Crashes | Exit |
| --- | --- | --- | --- |
| `fuzz_resp` | 853,386 | 0 | 0 |
| `fuzz_storage_key` | 4,417,791 | 0 | 0 |
| `fuzz_signed_url` | 4,045,871 | 0 | 0 |

~9.3M total executions, **0 crashes / panics / leaks / OOM**. No artifacts
written to `fuzz/artifacts/`.

## 4.5 Long (24h) manual gate

Not executed in this pass (impractical inline). Reproduction command per target:

```
cd fuzz
cargo +nightly fuzz run <target> -- -max_total_time=86400
```

Per the plan this is allowed to be documented as a blocking-pending item. The
release gate row **Fuzz 24h** stays `pending` until run on a Linux x86-64 host;
**Fuzz smoke** is `pass`.

## 4.6 Recommended additional targets (follow-up, Medium)

Not blocking, but recommended to broaden coverage in a later pass:
- `ag-auth`: WebAuthn attestation/assertion parsing (note: Stage 1 verified the
  `parse_raw_auth_data` length guards), JWT/claims, OAuth state, API keys.
- `ag-mail`: template rendering with missing/extra vars, header validation.
- `ag-domains`: DNS record validation, IDN, `notAfter`/certificate parsing.
- `ag-storage`: image decoding (decompression bombs) via `ImageProcessor`.

## 4.7 Rules applied

Every panic found by fuzzing is treated as a real bug unless proven unreachable
in production. The one crash-class issue found (RESP2 DoS) produced a fix + a
regression test, per the plan.

## Gate rows advanced

- **Fuzz smoke:** pass (existing DSL targets + 3 new targets, 0 crashes).
- **Fuzz 24h:** pending (manual gate, command documented above); tracked in issue #156.
