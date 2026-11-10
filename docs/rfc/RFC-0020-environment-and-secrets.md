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
