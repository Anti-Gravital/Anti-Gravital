# Pre-Phase 5 Build / Format / Lint / Docs Gate (Stage 1)

> Stage 1 deliverable of the master audit plan. Demonstrates that the workspace
> compiles, has no critical warnings, and that public documentation is not
> broken. All commands run locally on the audit baseline.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`
- **Baseline commit at gate close:** `635e623` (after Stage 1 fixes)
- **Toolchain:** stable (rustc/clippy 1.95.0), Docker running locally (used by
  the testcontainers Postgres/Redis tests).

## Results

| Check | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --all -- --check` | **pass** (clean) |
| Build / type-check | covered by clippy `--all-targets --all-features` | **pass** |
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **pass** (exit 0) |
| Docs | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` | **pass** (exit 0) |
| Tests | `cargo test --workspace --all-features` | **pass** — 564 passed, 0 failed, 7 ignored (52 suites) |

The 7 ignored tests are manual-gate tests (e.g. `ag-realtime` 50k load test,
real-NATS/JetStream tests that skip without a live server). They are addressed
in Stages 6/7.

## Findings fixed during Stage 1

All three were invisible to the previous CI because it never linted/compiled the
feature-gated code paths with `--all-features`. They are the kind of "green CI
that wasn't really green" gap the audit exists to surface.

### S1-1 — Panic / trivial DoS on object key with DEL byte (Critical/Blocker)
- **Where:** `crates/ag-storage/src/store/server.rs` `get_object`.
- **Root cause:** validation gap — `validate_key` rejects bytes `< 0x20` but
  allows `0x7F` (DEL); `http::HeaderValue` rejects `0x7F`. A key containing
  `%7F` (reachable from the URL path) passed validation, was stored, and on
  retrieval `Response::builder()...body().unwrap()` panicked when echoing the
  key in `X-AG-Store-Key` / `Content-Disposition`.
- **Fix:** echo the key verbatim only when it is a valid header value, else a
  percent-encoded fallback; RFC 5987 `filename*=UTF-8''` for non-ASCII
  filenames; map any residual builder error to 500 instead of panicking.
- **Regression test:** `get_object_with_del_byte_key_does_not_panic` plus unit
  tests for the encoders.
- **Commit:** `a16f771`.

### S1-2 — `ag-realtime` does not build standalone with `nats-external` (Blocker)
- **Where:** `crates/ag-realtime/Cargo.toml` / `src/external.rs`.
- **Root cause:** `external.rs` derives `thiserror::Error` but `thiserror` was
  never declared as a dependency. It only compiled when `thiserror` leaked into
  the build graph via `--all-targets` dev-dependency unification; `cargo doc`
  and `cargo test --all-features` both failed to compile the crate.
- **Fix:** `thiserror = { workspace = true, optional = true }` and add
  `dep:thiserror` to the `nats-external` feature.
- **Commit:** `ee7243b`.

### S1-3 — Clippy lints in `ag-realtime` under `nats-external` (Low)
- **Where:** `crates/ag-realtime/src/lib.rs` (1x `needless_return`) and
  `src/external.rs` tests (6x `field_reassign_with_default`).
- **Root cause:** these code paths are only compiled with `nats-external`, which
  the routine clippy never enabled.
- **Fix:** removed the needless return; converted `let mut c = Default::default();
  c.x = ...` to struct-update syntax.
- **Commit:** `635e623`.

## Panic policy audit

Raw grep counts 539 panic-family markers in `crates/`, but brace-tracking shows
360 are inside `#[cfg(test)]` modules (allowed) and 139 in `tests/`/`benches/`
(allowed). **40 are in productive code.** Breakdown:

- 1 was a reachable bug on external input (S1-1, fixed).
- ~7 are doc-comment examples (`//!`), not executed.
- ~24 are `expect(...)` on genuinely infallible invariants with explanatory
  messages (serde_json on a fixed type, HMAC accepting any key length, resolver
  config always valid, reqwest client construction with defaults, "polled future
  after completion" in `poll` impls, system-entropy failure, lock poisoning).
- 2 are fail-fast-at-boot in `ag-cli` (tokio runtime creation).
- ~6 are provably-unreachable arms / safe invariants (e.g. `ast.rs:134`
  `unreachable!()` after an early return handles the only reachable variants;
  `to_lowercase().next().unwrap()` always yields a char).

Per the plan's policy (`unwrap` forbidden in parsing/network/storage/auth/crypto/
CLI/DSL paths reachable from external input; `expect` allowed only with an
invariant message), the remaining 39 are compliant. No additional productive
panic is reachable from external input.

## Unsafe audit

`rg "\bunsafe\b|allow\(unsafe_code\)" crates examples` finds **zero** real
`unsafe` blocks — only a doc-comment mention in `ag-core/src/lib.rs` and a
Spanish code comment in `ag-storage/src/signed.rs`. The workspace lint
`unsafe_code = "deny"` holds with no per-crate override. (The plan's ideal
`#![forbid(unsafe_code)]` is effectively achieved via the workspace `deny` lint;
upgrading to `forbid` is a candidate Low improvement.)

## Housekeeping

- Removed stray `Cargo.toml.tmp.56172.091804dc0ca8` (gitignored atomic-write
  leftover) from the repo root.

## Gate conclusion

Stage 1 gate rows **Build, Clippy, Docs, Tests = pass** (see
`PRE_FASE5_RELEASE_GATE.md`). No blocking issue remains for this stage.
