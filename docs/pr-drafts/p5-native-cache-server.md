# P5 — ag-cache Native RESP2 Server (RFC-0005)

## Summary

Implements RFC-0005: a native in-process TCP server inside `ag-cache` that
speaks the RESP2 protocol, enabling standard Redis clients to connect without
requiring an actual Redis process. Closes DEBT-003 (P5 of the pre-Phase-5
corrective audit).

Changes:

- `crates/ag-cache/src/server/resp.rs`: RESP2 frame parser + response writer.
- `crates/ag-cache/src/server/cmd.rs`: command dispatch (GET, SET, DEL, EXISTS,
  MGET, MSET, EXPIRE, TTL, KEYS, PING, FLUSHDB, DBSIZE, COMMAND).
- `crates/ag-cache/src/server/mod.rs`: `NativeCacheServer` accept loop, one
  tokio task per connection.
- `crates/ag-cache/src/l1.rs`: added `flush()`.
- `crates/ag-cache/src/tags.rs`: added `clear()`.
- `crates/ag-cache/src/config.rs`: added `native_server_enabled`, `native_server_port`.
- `crates/ag-cache/Cargo.toml`: `native-server` feature + dashmap 6 dep.
- `crates/ag-cache/tests/resp2_compat.rs`: 13 integration tests via raw TcpStream.
- `docs/rfc/RFC-0005-ag-cache-native-l2.md`: status updated to Implemented.
- `crates/ag-cache/README.md`: added Native RESP2 server section.

## Phase affected

Pre-Phase-5 corrective — P5 (ag-cache RESP2 L2).

## Type of change

- Feature implementation (behind `native-server` Cargo feature, opt-in)
- Test

## Related documents

- `docs/rfc/RFC-0005-ag-cache-native-l2.md` — RFC
- `docs/DEBT.md` — DEBT-003 closed by this PR

## Test plan

- [x] `cargo build -p ag-cache --features native-server` — compiles clean
- [x] `cargo test -p ag-cache` — 20 existing tests pass (no regression)
- [x] `cargo test -p ag-cache --features native-server --test resp2_compat` — 13 passed
- [x] `cargo fmt -p ag-cache -- --check` — no diffs
- [x] `cargo clippy -p ag-cache --features native-server -- -D warnings` — clean
- [x] `cargo check --workspace` — no errors

## Exit criteria advanced

- DEBT-003: ag-cache native RESP2 server operational
- RFC-0005: status Implemented

## Final checklist

- [x] Belongs to correct phase
- [x] Respects documentation
- [x] Does not break architecture
- [x] No unnecessary complexity added
- [x] No circular dependencies
- [x] Compiles
- [x] Tests pass
- [x] `cargo fmt` passes
- [x] `cargo clippy` passes
- [x] Documentation updated in same PR
- [x] No emojis
- [x] No AI attribution
- [x] Commit messages under 256 characters
- [x] PR descriptor present
