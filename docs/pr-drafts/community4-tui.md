# Add `ag config` and the bare-`ag` home screen (RFC-0018 stages B and C)

## Resumen

Add ag config form and bare-ag home screen with shared typed requests (RFC-0018 stages B and C)

## Fase afectada

Phase 4.7 (additive, interactive terminal experience). Stages B and C of
RFC-0018. No master document under `docs/master/` was modified.

## Tipo de cambio

- [x] New command / CLI behavior
- [x] New module inside an existing crate
- [x] Dependency addition to an existing crate (`ag-core`, `toml_edit`)
- [x] Tests (unit, render, state transition, parity, non-interactive regression)
- [x] Documentation (root README EN+ES, CHANGELOG, RFC-0018 status, ag-cli
      READMEs, knowledge graph)
- [ ] Breaking change
- [ ] New crate

## Documentos relacionados

- `docs/rfc/RFC-0018-interactive-terminal-experience.md` (accepted), sections
  4.1 (flag parity, one executor), 4.2, 4.4, 4.5, 4.9.
- `CLAUDE.md` rules 15, 18, 19, 26, 27, 32, 36.
- `README.md`, `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`,
  `docs/graph/knowledge-graph.json`, `CHANGELOG.md`.

## Cambios principales

Stage B:

- `crates/ag-cli/src/config.rs`: `ConfigKey` registry mirroring
  `ag_core::ShieldConfig`, typed `ConfigRequest` (`Get`/`Set`/`Validate`),
  `execute_config` with validation before write and an atomic replace,
  format-preserving `toml_edit` editing, in-memory normalization of the legacy
  `[bind] host/port` template form, and no exposure of inline secret material.
- `crates/ag-cli/src/commands/config.rs`: flag parsing and output; flags and
  the form produce the same request shape and share the single executor.
- `crates/ag-cli/src/tui/config_form.rs`: form screen with inline validation,
  review step, equivalent-command output and no I/O in state or view.

Stage C:

- `crates/ag-cli/src/tui/home.rs`: home screen with keyboard navigation that
  returns a command line through a `Launch` effect and performs no work.
- `crates/ag-cli/src/main.rs`: the root command is optional; bare `ag` opens
  the home screen only when the session is interactive and prints help
  otherwise. Launched commands are re-parsed through the real CLI.

`ag new`, every existing command and the documented exit codes are unchanged.

## Plan de prueba

Commands run from worktree `worktrees4/tui`:

- `cargo test -p ag-cli --all-features`
- `cargo test --workspace --all-features`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- `cargo audit`
- `cargo deny check`

Verification used an isolated `CARGO_TARGET_DIR` because the shared target
directory is concurrently overwritten by other wave-4 roles building `ag`.

## Criterios de salida que avanza

- Phase 4.7 stage B (project configuration) implemented with non-interactive
  parity and validation before write.
- Phase 4.7 stage C (home screen) implemented as a launcher with no work.
- RFC-0018 principle 1 (flag parity) and principle 2 (one executor) enforced by
  tests: printed equivalent commands re-parse to the same request.

## Checklist

- [x] Compila
- [x] Pasa tests
- [x] Pasa fmt
- [x] Pasa clippy `-D warnings`
- [x] Pasa audit y deny
- [x] Documentacion actualizada en el mismo cambio

## Cierre de issues

Closes: none
