# ag-workers: deployment configuration (WorkersConfig, RFC-0012 §28)

## Summary

Adds `WorkersConfig`, the deployment-level `[workers]` configuration for `ag-workers`
(RFC-0012 §28): mode, backend, shutdown timeout, payload limit, poison-guard threshold
and per-queue settings, loadable from a TOML document and overridable through
`AG_WORKERS_*` environment variables (env over TOML over defaults). This is the
foundation the upcoming `ag workers run`/`doctor` CLI reads. Additive; the existing
low-level `RuntimeConfig` is unchanged and is now derivable via
`WorkersConfig::runtime_config()`.

## Phase affected

Phase 4.6-D (`ag-workers` completion against the RFC-0012 blueprint). Additive,
native-by-default; no production/GA claim.

## Type of change

- [x] Feature implementation (additive public API: `WorkersConfig`, `WorkerMode`,
  `BackendKind`, `QueueConfig`, `ConfigError`, `parse_duration`)
- [x] Test (TOML parse, env override precedence, duration parsing, enum rejection)
- [x] Documentation (crate README configuration section)

## Related documents

- `docs/rfc/RFC-0012-ag-workers.md` (§28 Configuration)
- `crates/ag-workers/README.md` (new Configuration section)

## Test plan

- [ ] `cargo test -p ag-workers config` (default, empty/full TOML, unknown enum,
  duration units, env-over-TOML precedence, runtime-config derivation)
- [ ] `cargo clippy -p ag-workers --all-targets --all-features -- -D warnings`
- [ ] `cargo fmt --all -- --check`

## Exit criteria advanced (docs/roadmap/STATUS.md)

- Advances RFC-0012 §28 (configuration) toward the Phase 4.6-D complete scope. STATUS
  exit-criteria checkboxes are reconciled in the final completion PR.

## Final checklist

- [x] Belongs to the correct phase (4.6-D completion).
- [x] Respects documentation (RFC-0012 §28; README updated same change).
- [x] Does not break architecture (additive; no new cross-crate dependency beyond the
  workspace `toml` dep already used by `ag-core`).
- [x] No unnecessary complexity (one config layer; reuses workspace `toml`/`serde`).
- [x] No circular dependencies.
- [ ] Compiles.
- [ ] Tests pass.
- [ ] `cargo fmt` clean.
- [ ] `cargo clippy -D warnings` clean.
- [x] Documentation updated in the same change.
