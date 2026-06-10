# fix(ag-realtime): bound persistent event appends

## Summary

Bounds async event-persistence work, preserves the persistent NDJSON handle, and
adds an explicit flush barrier for graceful shutdown.

## Phase affected

Phase 4 production hardening for ag-realtime event persistence.

## Type of change

- [ ] Security fix
- [x] Bug fix
- [x] Tests
- [x] Documentation
- [ ] New feature
- [ ] Breaking public API change

## Changes

- Keep EventBuffer::open source-compatible with a conservative default of 64
  pending async appends.
- Add open_with_max_pending_appends for explicit backpressure configuration.
- Acquire a semaphore permit before submitting blocking filesystem work.
- Add flush_async as a barrier for submitted writes during shutdown.
- Document replay, backpressure, flush, and handle-drop behavior.
- Test cap enforcement, permit reuse, concurrent integrity, and flushing.

## Related documents

- CLAUDE.md sections 11, 13, 18, and 36
- docs/DEBT.md DEBT-012
- docs/modules/ag-realtime/README.md
- Issue #73

## Test plan

- [x] cargo test -p ag-realtime --features event-persistence
- [x] cargo clippy -p ag-realtime --features event-persistence --all-targets -- -D warnings
- [x] cargo fmt --all -- --check
- [x] git diff --check

## Exit criteria advanced

- [x] Async appends do not block Tokio worker threads.
- [x] Repeated appends reuse the persistent file handle.
- [x] Pending blocking work is bounded and applies explicit backpressure.
- [x] Concurrent append integrity is tested.
- [x] Malformed and truncated replay behavior is documented and tested.
- [x] Flush and shutdown behavior is documented and tested.

## Final checklist

- [x] Belongs to the approved Phase 4 scope.
- [x] No new dependencies are introduced.
- [x] Existing EventBuffer::open and append_async callers remain compatible.
- [x] Tests cover the reported concurrency regressions.
- [x] Clippy passes with warnings denied.
- [x] No unrelated changes are included.
- [x] PR descriptor is present.

Closes #73
