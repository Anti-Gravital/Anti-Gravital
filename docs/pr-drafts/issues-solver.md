# fix(ag-storage): close critical storage security issues

## Summary

Resolves three security issues in the native storage backend and embedded HTTP
server: parent-symlink path escape, unauthenticated public binding, and inline
rendering of uploaded active content.

## Phase affected

Phase 4 production hardening for `ag-storage`.

## Type of change

- [x] Security fix
- [x] Bug fix
- [x] Tests
- [x] Documentation
- [ ] New feature
- [ ] Breaking public API change

## Changes

- Native filesystem operations walk parent directories through directory
  capabilities, reject symlink components, disable final symlink following,
  and keep temporary writes plus rename inside the opened parent capability.
- Server mode binds to `127.0.0.1` by default. A non-loopback bind without a
  token fails during construction unless
  `STORAGE_ALLOW_INSECURE_PUBLIC=true` is explicitly configured.
- Uploaded HTML, HTM, and SVG retain their media type but are always returned
  with `Content-Disposition: attachment`.

## Related documents

- `CLAUDE.md` sections 15, 16, 18, 19, 28, and 36
- `docs/rfc/RFC-0013-capability-filesystem-confinement.md`
- `docs/modules/ag-storage/README.md`
- Issues #62, #63, and #64

## Test plan

- [x] `cargo test -p ag-storage`
- [x] `cargo clippy -p ag-storage --all-targets -- -D warnings`
- [x] `cargo fmt --all -- --check`
- [x] `git diff --check`

Verified result: 72 unit/property/integration tests and 2 doctests pass.

## Exit criteria advanced

- [x] Parent symlinks cannot read or write outside the native storage root.
- [x] Native filesystem validation and I/O are no longer a separable
  canonicalize/open sequence.
- [x] Public unauthenticated server configuration fails with an actionable
  error unless an explicit unsafe override is present.
- [x] Loopback development without authentication remains supported.
- [x] HTML, HTM, and SVG responses include safe attachment headers.

## Final checklist

- [x] Belongs to the approved Phase 4 storage scope.
- [x] Security behavior and limitations are documented.
- [x] Architecture boundaries remain unchanged.
- [x] New dependencies are justified by RFC-0013.
- [x] Tests cover the reported regressions.
- [x] Clippy passes with warnings denied.
- [x] No unrelated changes are included.
- [x] PR descriptor is present.

Closes #62
Closes #63
Closes #64
