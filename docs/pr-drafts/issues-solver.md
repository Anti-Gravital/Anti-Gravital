# fix(ag-dsl): make generated Rust validation executable

## Summary

Completes the generated-Rust blocker by emitting executable, cached `@regex`
validation, rejecting malformed patterns during semantic analysis, and compiling
a representative generated project in CI.

## Phase affected

Phase 3 generated-project exit criterion, preserving Phase 4 DSL extensions.

## Type of change

- [ ] Security fix
- [x] Bug fix
- [x] Tests
- [x] Documentation
- [ ] New feature
- [ ] Breaking public API change

## Changes

- Validate `@regex` patterns during semantic analysis so malformed schemas fail
  before code generation.
- Emit one `OnceLock<regex::Regex>` per annotated field and execute it from the
  generated `validate()` method.
- Add an explicit `regex = "1"` dependency contract, justified by RFC-0014.
- Regenerate the representative Rust fixture and exercise accepted and rejected
  values in an integration test.
- Document executable validation and the generated-project dependency contract.

## Related documents

- `CLAUDE.md` sections 20, 26, 31, and 36
- `docs/rfc/RFC-0014-regex-runtime-validation.md`
- `docs/dsl/referencia-v01-v04.md`
- Issues #70, #59, and #60

## Test plan

- [x] `cargo test -p ag-dsl`
- [x] `cargo test -p ag-generated-rust-fixture`
- [x] `cargo clippy -p ag-dsl -p ag-generated-rust-fixture --all-targets -- -D warnings`
- [x] `cargo fmt --all`
- [x] `git diff --check`

## Exit criteria advanced

- [x] Generated modules use coherent ownership and imports.
- [x] Authenticated handlers import the existing `Claims` extractor.
- [x] A representative generated Rust project compiles and runs validation tests.
- [x] `@regex` validation is executable and its dependency is explicit.
- [x] Patterns are compiled once per field instead of once per request.
- [x] Malformed patterns fail semantic analysis.
- [x] Handler bodies remain developer-owned stubs.
- [x] Generated comments remain in English.

## Final checklist

- [x] Belongs to the approved Phase 3 scope.
- [x] New dependency is justified by RFC-0014.
- [x] Tests cover malformed, matching, and non-matching patterns.
- [x] Clippy passes with warnings denied.
- [x] No unrelated changes are included.
- [x] PR descriptor is present.

Closes #70
Closes #59
Closes #60
