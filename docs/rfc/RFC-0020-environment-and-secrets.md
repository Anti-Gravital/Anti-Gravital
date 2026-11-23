# RFC-0020 - Declared environment variables and secret management

- Status: accepted
- Decision date: 2026-11-10
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: additive Phase 4.8, stage B
- Affected modules/crates: `ag-core` (configuration loading, `Secret` type),
  `ag-cli` (new `ag env` command group), `templates/`
- Predecessor RFC: RFC-0019 (`ag.toml`), RFC-0018 (interactive forms)
- Comment period: waived by BDFL decision.

## Implementation status (stage B, 2026-11-21)

- Stage B / PR 1 is implemented on branch `community4/envsecrets`: `Secret<T>`
  and the declared environment loader live in `ag-core` with tests covering
  redaction, absence of serialization, zeroization on drop, explicit expose,
  type validation, declared-key isolation and fail-fast multi-error
  reporting without printing values.
- Resolution order currently implements the process environment and the
  declared default from section 4.2. The encrypted file source (4.4), the
  `[env]` declaration in `ag.toml` (4.1), the `ag env` commands (4.6), module
  migration off direct `std::env::var` reads (PR 4) and `ag env export` with
  the interactive form (PR 5) are still future stages.
- Existing `std::env::var` call sites are unchanged: this stage is additive.
  This note records stage progress only; the accepted design above is not
  modified.
- Stage B / PR 2 (CLI subset, wave 9, 2026-11-22): `ag env list` and
  `ag env check` are implemented in `ag-cli`. They build `EnvDeclaration`
  values from the `[env.*]` tables of `ag.toml` and resolve them through
  `ag_core::EnvLoader`, so `check` validates exactly as startup would. `list`
  shows name, type, required/secret flags, source and presence; no value,
  including a non-secret default, is ever printed. `--output json` emits
  versioned `env.list` / `env.check` envelopes. `--env NAME` is accepted as a
  label only: the encrypted secrets file, `config/<environment>.toml`,
  `set`/`unset`, `keygen`, `recipients` and `export` remain future stages and
  are not faked. No `[env]` parser was added to `ag-core`; the CLI owns the
  manifest parsing and uses only the public loader API.
- Stage B / PR 4 (wave 22, 2026-11-23): `ag-storage` is the first crate migrated
  off direct `std::env::var` reads. `crates/ag-storage/src/config.rs` declares
  its 17 keys through `EnvDeclaration`/`EnvLoader`, with secret classification
  (`STORE_TOKEN`, `STORAGE_JWT_PUBLIC_KEY`, `AWS_SECRET_ACCESS_KEY`,
  `STORAGE_SIGN_SECRET`), per-field declared defaults and process-environment
  isolation (only declared keys are read). `StorageConfig::from_env` keeps its
  signature and resolves through `load_process`; a test-only injected source
  removes the `set_var` races (audit A1). Parity tests in
  `crates/ag-storage/src/config_tests.rs` compare the declared path against the
  previous direct reads for every key. Defaults, resolution order and backend
  selection are unchanged; external S3/MinIO backends stay opt-in behind the
  `s3` feature (`ADR-0009`). The remaining crates in audit A2 (`ag-observe`,
  `ag-workers`, `ag-realtime`, `ag-auth`, `ag-cache`, `ag-cli`) are still
  pending. This note records stage progress only; the accepted design above
  is not modified.
- Stage B / PR 4 (wave 23, 2026-11-23): `ag-observe` is the second crate migrated
  off direct `std::env::var` reads. `crates/ag-observe/src/config.rs` declares
  its six keys through `EnvDeclaration`/`EnvLoader`, all as `EnvType::String`
  so the original per-field fallback on unset or malformed values is preserved.
  None is a secret (`AG_PROJECT_ID` is an opaque bounded label and the loader
  never prints a value). `ObserveConfig::from_env` keeps its signature and uses
  `load_process`; a test-only injected source removes the `set_var` races (audit
  A1). Parity tests in `crates/ag-observe/src/config_tests.rs` compare the
  declared path against the previous direct reads for every key. `SERVICE_NAME`
  is declared with its `ag-service` default but its read stays direct in the
  `otlp`-gated `layer::build_resource`: adding a public `ObserveConfig` field
  would be a public API change forbidden by this PR. Defaults, resolution order,
  log-format and dev-console semantics, project-scope validation and OTLP
  behavior are unchanged. `ag-core` is added as an internal dependency with the
  dependency-direction allow-list recording observability -> core (Arq-Tecnica
  5.3 regla 2). The remaining crates in audit A2 (`ag-workers`, `ag-realtime`,
  `ag-auth`, `ag-cache`, `ag-cli`) are still pending. This note records stage
  progress only; the accepted design above is not modified.
- Stage B / PR 4 (wave 24, 2026-11-23): `ag-workers` is the third crate migrated
  off direct `std::env::var` reads. `crates/ag-workers/src/config.rs` declares
  six fixed keys through `EnvDeclaration`/`EnvLoader`, all as `EnvType::String`
  so the original per-field fallback on unset or malformed values is preserved.
  Three of them (`AG_WORKERS_MODE`, `AG_WORKERS_BACKEND`,
  `AG_WORKERS_SHUTDOWN_TIMEOUT`) were the direct reads in `apply_env`; the other
  three (`AG_WORKERS_ENABLED`, `AG_WORKERS_MAX_PAYLOAD_BYTES`,
  `AG_WORKERS_POISON_GUARD_ATTEMPTS`) and the per-queue
  `AG_WORKERS_QUEUE_<NAME>_MIN`/`_MAX` keys were read through the
  `env_bool`/`env_parse` helpers, so declaring them too completes the crate's
  migration. None is a secret. `WorkersConfig::apply_env` keeps its signature
  and uses `load_process`; a test-only injected source removes the `set_var`
  races (audit A1). Parsing stays in the loader-to-config conversion, so enum
  validation, `parse_duration` and per-field fallbacks are byte-identical.
  Parity tests in `crates/ag-workers/src/config_tests.rs` compare the declared
  path against the previous direct reads for every key. `ag-core` is added as
  an internal dependency with the dependency-direction allow-list recording
  workers -> core (Arq-Tecnica 5.3 regla 2). The `postgres` feature and the
  optional `ag-data` dependency are unchanged. The remaining crates in audit A2
  (`ag-realtime`, `ag-auth`, `ag-cache`, `ag-cli`) are still pending. This note
  records stage progress only; the accepted design above is not modified.
- Stage B / PR 4 (wave 25, 2026-11-23): `ag-realtime` is the fourth crate migrated
  off direct `std::env::var` reads. `crates/ag-realtime/src/config.rs` declares
  its eleven keys through `EnvDeclaration`/`EnvLoader`, all as `EnvType::String`
  so the original per-field fallback on unset or malformed values is preserved.
  None is a secret: `NATS_TLS_CERT` and `NATS_TLS_KEY` are paths to a client
  certificate and private key, not credential bytes, so they are not marked
  secret; the loader never prints a value. `RealtimeConfig::from_env` keeps its
  signature and uses `load_process`; a test-only injected source removes the
  `set_var` races (audit A1). Parsing stays in `from_loaded_env`, so exact
  lowercase `external` mode, case-insensitive `true` flags, numeric fallbacks
  and optional TLS paths (including the empty string) are byte-identical.
  Parity tests in `crates/ag-realtime/src/config_tests.rs` compare the declared
  path against the previous direct reads for every key. The three `NATS_URL`
  reads inside `#[cfg(test)] mod tests` in `external.rs` are integration gates
  for a real NATS server and are intentionally untouched, like the S3/MinIO
  gates in `ag-storage`. `ag-core` is added as an internal dependency with the
  dependency-direction allow-list recording realtime -> core (Arq-Tecnica 5.3
  regla 2). The remaining crates in audit A2 (`ag-auth`, `ag-cache`, `ag-cli`)
  are still pending. This note records stage progress only; the accepted design
  above is not modified.
- Stage B / PR 4 (wave 26, 2026-11-23): `ag-auth` is the fifth crate migrated
  off direct `std::env::var` reads. `crates/ag-auth/src/config.rs` declares its
  eight keys through `EnvDeclaration`/`EnvLoader`, all as `EnvType::String` so
  the original per-field fallback on unset or non-UTF-8 values is preserved.
  Four are secrets and are declared `secret(true)`: `JWT_PRIVATE_KEY`,
  `JWT_PUBLIC_KEY`, `GOOGLE_CLIENT_SECRET` and `GITHUB_CLIENT_SECRET`; their
  bytes are read only through an explicit `expose()` call in `from_loaded_env`
  and never appear in a value-free error or a redacted `Debug`/`Display`.
  `AuthConfig::from_env` keeps its `Result<Self, AuthConfigError>` signature and
  uses `load_process`; absence of a JWT key still maps to
  `AuthConfigError::MissingVar` with the same message. A test-only injected
  source removes the `set_var` races (audit A1). Parsing and defaults stay in
  `from_loaded_env`, so the `localhost` / `http://localhost:8080` WebAuthn
  defaults and independent OAuth `Option` fields are byte-identical. Parity
  tests in `crates/ag-auth/src/config_tests.rs` compare the declared path
  against the previous direct reads for every key. `ag-core` is added as an
  internal dependency with the dependency-direction allow-list recording
  auth -> core (Arq-Tecnica 5.3 regla 2). The remaining crates in audit A2
  (`ag-cache`, `ag-cli`) are still pending. This note records stage progress
  only; the accepted design above is not modified.

## 1. Motivation

Configuration is where most production incidents and most leaks start: a
variable missing in one environment, a typo in a name, a secret committed to
the repository or printed in a log. A tool that promises secure-by-default
backends has to make the correct handling of configuration the easiest path,
and has to make the incorrect one hard.

## 2. Problem

Verified in the repository on 2026-11-10:

1. About forty distinct environment variables are read with direct
   `std::env::var` calls spread over ten crates (`DATABASE_URL`, `REDIS_URL`,
   `NATS_URL`, `STORAGE_SIGN_SECRET`, `STORAGE_JWT_PUBLIC_KEY`, and others).
   There is no single list of what a project needs.
2. A missing or malformed variable is discovered when the code path that reads
   it runs, which can be long after startup.
3. Nothing distinguishes a secret from an ordinary setting. A secret is a
   `String` and can be logged, formatted or serialized like any other.
4. There is no supported way to keep secrets with the project. Users will
   reach for an unencrypted `.env` file, and some will commit it.
5. The deployment targets of Phase 5 each have a different secret mechanism;
   without a declaration to generate from, each target would be wired by hand.

## 3. Alternatives considered

**A. Document a `.env` convention.** Familiar. Rejected: plain-text secrets on
disk next to the code, no typing, no validation.

**B. Require an external secret manager.** Strong security. Rejected as the
only path: it violates `ADR-0009` rule 2 (an external service must never be a
requirement). Kept as adapters (4.5).

**C. A declaration in `ag.toml`, typed loading with fail-fast validation, and
a native encrypted secrets file, with external managers as adapters
(chosen).**

## 4. Proposed design

### 4.1 Declaration

Every variable a project reads is declared once in `ag.toml`. Undeclared
variables are not readable through the configuration API.

```toml
[env.DATABASE_URL]
type = "url"                # string | int | bool | url | duration | path
required = true
secret = true
description = "PostgreSQL connection string"

[env.PORT]
type = "int"
default = 8080
description = "HTTP listen port"
```

Enabling a module in `ag.toml` (`RFC-0019`) contributes that module's
variables to the declaration, with their documentation. The modules stop
calling `std::env::var` themselves and receive typed configuration from the
composition root.

### 4.2 Loading and validation

- Resolution order, highest first: process environment, the decrypted secrets
  file for the selected environment, `config/<environment>.toml`, declared
  default.
- All variables are resolved and validated **at startup, before any listener
  opens**. On failure the process exits non-zero and prints every problem at
  once: name, expected type, and where it looked. Values are never printed.
- The loader produces a typed, immutable configuration value. There is no
  global mutable configuration.

### 4.3 The `Secret` type

Variables declared `secret = true` are delivered as `Secret<T>`:

- `Debug` and `Display` print `[redacted]`. It does not implement `Serialize`.
- The value is zeroed in memory when dropped.
- Reading the value requires an explicit `expose()` call, which makes every
  use of a secret searchable in review.
- `ag-observe` log and trace layers never receive the inner value because the
  type cannot be formatted.

### 4.4 Native secret storage

Non-secret settings live in `config/<environment>.toml`, committed in clear.

Secrets live in `config/<environment>.secrets`, an encrypted file that is safe
to commit:

- Encryption uses the `age` file format with X25519 recipients. Each person or
  server that must read an environment has a key pair; the file is encrypted
  to the list of public keys in `config/<environment>.recipients`.
- Private keys never enter the repository. On a developer machine the key is
  in the user's configuration directory with owner-only permissions; on a
  server it is provided by the deployment target.
- Adding or removing a recipient re-encrypts the file. Removing a recipient
  does not un-leak what they already read: the command says so and lists the
  secrets that must be rotated.
- The scaffolded `.gitignore` excludes `.env`, `*.key` and decrypted outputs.
  `ag env check` fails when a declared secret is found in a committed
  clear-text file.

### 4.5 External secret stores (adapters)

A `SecretSource` trait with the encrypted file as the default implementation.
Adapters behind Cargo features can read from the secret mechanism of the
deployment target (Kubernetes Secrets, the container runtime's secret mounts,
or a secret manager), following the adapter rule of `ADR-0009` and the naming
rule of `ADR-0011`.

### 4.6 Commands

| Command | Effect |
| --- | --- |
| `ag env list [--env NAME]` | Declared variables with type, source and whether a value is present. Secret values are never shown. |
| `ag env check [--env NAME]` | Validates the environment exactly as startup would. Usable in CI. |
| `ag env set NAME [--env NAME]` | Sets a value. For a secret the value is read from a hidden prompt or from stdin, never from an argument (arguments are visible in process listings and shell history). |
| `ag env unset NAME` | Removes a value. |
| `ag env recipients add/remove/list` | Manages who can decrypt an environment. |
| `ag env keygen` | Creates the local key pair. |
| `ag env export --format FORMAT` | Emits the **shape** for a target: a Kubernetes Secret manifest skeleton, compose `environment` keys, Terraform variable declarations marked `sensitive`. Values are included only with an explicit flag and only to stdout. |
| `ag env` (in a terminal) | Form per `RFC-0018`: variables grouped by module, validation inline, secrets entered through masked fields and never displayed again. |

### 4.7 Dependencies

| Dependency | Purpose | Justification (rule 15) |
| --- | --- | --- |
| `age` | File encryption format | Rust implementation of a small, audited-by-design format; avoids inventing a cryptographic container (rule 16). |
| `zeroize` | Clearing secret memory | Standard crate for this purpose; already present transitively through the cryptography stack. |

### 4.8 Testing

Unit tests for resolution order and every type parser; a test that formatting
a `Secret` in every formatting trait yields the redaction; round-trip and
wrong-key tests for the encrypted file; a test that startup with three invalid
variables reports all three; property tests on the parser; a regression test
that no command writes a secret value to stdout or stderr without the explicit
flag.

## 5. Implementation plan (only after approval)

1. PR 1 - `Secret<T>` and the typed loader in `ag-core`, with tests.
2. PR 2 - `[env]` declaration in `ag.toml`; `ag env list` and `ag env check`.
3. PR 3 - encrypted secrets file, `keygen`, `recipients`, `set`, `unset`.
4. PR 4 - migrate the modules from direct `std::env::var` reads to injected
   typed configuration, one crate per pull request.
5. PR 5 - `ag env export`; the interactive form.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| A user loses the only private key for an environment | Medium | High | `keygen` and `recipients` warn when an environment has a single recipient; documentation requires at least two. |
| Encrypted file gives a false sense of safety after a recipient leaves | Medium | High | `recipients remove` prints the rotation list (4.4). |
| Migrating ten crates off direct environment reads breaks existing users | Medium | Medium | One crate per pull request; variable names are preserved. |
| A cryptography dependency receives an advisory | Low | High | The format is isolated behind `SecretSource`; the weekly audit of `RFC-0022` detects it. |

## 7. Impact

- **Scope:** new `ag env` command group; configuration becomes declared.
- **Public APIs:** module constructors take configuration values instead of
  reading the environment (pre-release breaking change).
- **Security posture:** secrets become a distinct type end to end.
- **Documentation:** new manual chapter; each module README lists its
  variables from the same declaration.

## 8. Rollback

The loader keeps the process environment as the highest-priority source, so a
project can ignore the encrypted file and the `ag env` commands and supply
every variable from outside, as today. `Secret<T>` would be kept in any case.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-10.
- Result: accepted. The comment period was waived by BDFL decision.

Resolution of the open questions:

The native encrypted secrets file is the default storage. A project can still
supply every variable through the process environment, which keeps the highest
priority in the resolution order.

## 10. References

- `CLAUDE.md` rules 13, 15, 16, 21.
- `ADR-0009` rules 2 and 5; `ADR-0011`.
- `RFC-0018`, `RFC-0019`, `RFC-0021`.
