# fix(ag-realtime): reject corrupt replay records

## Summary

Makes persisted event replay fail explicitly on corrupt records and publication
errors instead of creating empty events or hiding replay loss.

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

- Deserialize each NDJSON record into the strict PersistedEvent type with line
  context on InvalidData failures.
- Propagate replay publication failures as BrokenPipe with the affected subject.
- Test malformed JSON, missing fields, wrong field types, and a closed publisher.
- Document replay failure behavior for the event-persistence feature.

## Related documents

- CLAUDE.md sections 11, 13, 18, and 36
- crates/ag-realtime/README.md
- Issue #65

## Test plan

- [x] cargo test -p ag-realtime --features event-persistence
- [x] cargo clippy -p ag-realtime --features event-persistence --all-targets -- -D warnings
- [x] cargo fmt --all -- --check
- [x] git diff --check

## Exit criteria advanced

- [x] Invalid persisted fields return InvalidData with line context.
- [x] Replay no longer synthesizes empty events from corrupt records.
- [x] Publication errors stop replay and are returned to the caller.
- [x] Tests cover malformed records and a closed publisher.

## Final checklist

- [x] Belongs to the approved Phase 4 scope.
- [x] No new dependencies are introduced.
- [x] Public success-path behavior remains source-compatible.
- [x] Tests cover the reported regressions.
- [x] Clippy passes with warnings denied.
- [x] No unrelated changes are included.
- [x] PR descriptor is present.

Closes #65
