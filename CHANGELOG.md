# Changelog

Formato basado en Keep a Changelog (https://keepachangelog.com/) y
semver (https://semver.org/). La primera version etiquetada es el
prerelease `0.1.0-alpha.1`; las entradas bajo `[Unreleased]` son cambios
aun no etiquetados.

## [0.1.0-alpha.1] - 2026-11-22

Primer prerelease etiquetado. Version `0.1.0-alpha.1` tomada del manifest
compartido del workspace; tag `v0.1.0-alpha.1`. Alpha para construir
prototipos e instalacion desde el codigo fuente, no un release de
produccion. Gobierno: RFC-0032 (aceptada, dispensa BDFL registrada).

### Added

- `.github/workflows/release.yml`: se dispara con `workflow_dispatch` (tag
  existente como entrada) o con tags `v*`. Compila los dos binarios del
  workspace, `ag` (CLI) y `ag-lsp` (language server del Anti-DSL), con
  `cargo build --locked --release` para cuatro targets:
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `aarch64-apple-darwin` y `x86_64-pc-windows-msvc`. Ocho artefactos por
  release.
- Un manifiesto `checksums.txt` con los SHA-256 de los ocho binarios,
  identificados por nombre de asset (p. ej. `ag-x86_64-unknown-linux-gnu`),
  generado y verificado con `sha256sum --check` dentro del job antes de
  crear cualquier asset. Es el manifiesto que consume el canal alpha de
  los instaladores.
- Creacion de un prerelease en modo DRAFT con los ocho binarios y el
  manifiesto. La publicacion es una accion manual del coordinador; el
  workflow nunca publica, firma, etiqueta ni hace push.

### Changed

- `Cargo.toml`: la version del workspace pasa de `0.0.0` a
  `0.1.0-alpha.1`. Se sincronizan los requisitos de version de las
  dependencias internas por ruta y se regenera `Cargo.lock`.
- `publish = false` se mantiene: no hay paquetes en crates.io. La disciplina
  SemVer de una API publica estable empieza mas adelante; este tag es una
  promesa de inestabilidad, no de compatibilidad.
- RFC-0032, indice de RFC y esta seccion del CHANGELOG.

### What works (honest alpha scope)

- El workspace compila y sus tests pasan en CI en las cuatro plataformas
  objetivo; `ag` ofrece scaffolding, ciclo de desarrollo, generacion desde
  el DSL, `ag dev`, `ag build` y los comandos de modulo documentados.
- Los crate `ag-core` (Shield), `ag-dsl`, `ag-data`, `ag-auth`, `ag-cache`,
  `ag-realtime`, `ag-storage`, `ag-observe`, `ag-mail`, `ag-workers`,
  `ag-domains` y `ag-edge` tienen codigo y pruebas; diez ejemplos son
  ejecutables. El DSL v0.1-v0.8 genera Rust, SQL, TypeScript, OpenAPI y
  AsyncAPI; `ag-lsp` acompana al compilador.
- La instalacion desde fuente funciona con Rust 1.99.1 o superior mediante
  `install.sh` / `install.ps1`, que conservan la verificacion fail-closed
  de `AG_INSTALLER_CHECKSUMS`.

### What does not work yet

- No hay API estable: los crates, el DSL y el codigo generado pueden cambiar
  hasta la puerta de 1.0. El texto "implemented" del README significa que
  existe codigo y tests, no certificacion de produccion.
- No hay despliegue (`ag deploy`), secretos, logs ni rollback: pertenecen a
  fases posteriores. `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate` y
  `ag-wasm-host` siguen siendo reservas de nombre.
- No hay publicacion en crates.io. El canal por defecto sigue siendo la
  instalacion desde el codigo fuente; el canal alpha opt-in de los
  instaladores descarga el asset `ag-<target>` y lo comprueba contra
  `checksums.txt` (solo checksum, sin firma).
- La puerta de release pre-Fase 5 sigue abierta: faltan la puerta de fuzz
  de 24 horas (CI corre una prueba de 60 segundos), la evidencia final de
  rendimiento y la reevaluacion sobre el commit de consolidacion.
- Los tests de integracion con PostgreSQL duradero siguen `#[ignore]` sin
  base de datos en CI; se verificaron manualmente contra PostgreSQL 16.
- Los criterios externos de comunidad y el comite tecnico no estan cerrados.

### Integrity and signing (explicit non-claim)

- `checksums.txt` es de INTEGRIDAD, no de AUTENTICIDAD: la pagina de release
  y el repositorio estan en el mismo dominio de confianza.
- No existe clave de firma de release. No se publica ninguna firma ni
  manifiesto firmado. RFC-0030 sigue siendo un diseno no aceptado y el
  issue #141 permanece abierto y bloqueado. No se afirma una instalacion
  "verificada" en sentido criptografico.

## [Unreleased]

### Template design conventions (wave 7)

Added:

- `docs/templates/TEMPLATE_DESIGN.md`: canonical English conventions shared by
  the template catalog and the wave-7 deployment work. Defines the standard
  project skeleton, naming and environment variables, the `/health` and
  `/ready` probe contract, structured logging fields, configuration precedence,
  secret handling (RFC-0020), the template-versus-adapter boundary, a
  composition model, the upgrade path and the checklist for new templates.
  Includes an honest Docker/Kubernetes compatibility matrix that separates
  generated shapes, operator responsibilities and the native no-service
  default, plus a list of known drift with existing templates. Documentation
  only; no template, CLI or master document is modified.
- `docs/INDEX.md`: entry for `docs/templates/`.

### Roadmap and STATUS reconciliation for the alpha (waves 4 and 5)

Changed:

- `docs/roadmap/STATUS.md` records the verifiable state after wave 4 and the
  alpha milestone: RFC-0018 stages B and C implemented (`ag config`, bare-`ag`
  home screen; `ag new --with` is still absent, so stage B stays partial),
  RFC-0021 stage E read-only `--output json` (contract 0.1.0), RFC-0019 stage A
  opt-in layered template, RFC-0020 stage B `Secret<T>` plus declared env
  loader. Phases 4.7 and 4.8 are not closed; RFC-0024 remains blocked on its
  seven prerequisites and no MCP code is claimed.
- The master `ANTI-GRAVITAL-Hoja-de-Ruta.md` status labels for Phases 4.7/4.8
  were corrected in EN and ES. SHA-256 updated in `docs/master/VERSION.md` and
  `.github/workflows/docs.yml` in the same change; derived files regenerated
  with `tools/split-masters.sh`. The pre-Phase-5 gate and Phase 5 stay open.

### Local checks: no silent emoji skip (issue #190)

Fixed:

- `scripts/ag-check` and `.githooks/commit-msg` now verify that a Perl
  interpreter actually runs before selecting it as the emoji detector, and
  fail loudly when neither a working Perl nor GNU `grep -P` is available. A
  missing detector can no longer be reported as `PASS: prohibited`.
- The hook test suite covers the no-detector path (36 assertions, all passing).

### Opt-in local verification workflow

Added:

- `.githooks/commit-msg` enforces the 256-character subject limit and rejects
  AI-tool attribution, chat-session URLs and emojis in commit messages.
- `.githooks/pre-push` runs `scripts/ag-check` and propagates its exit code, so
  a failed check blocks the push without a silent bypass.
- `scripts/ag-check` validates the branch descriptor and its closing keyword,
  `fmt`, `clippy`, `test`, `audit`, `deny`, master-document SHA-256 hashes and
  prohibited content. Skipped checks are always reported as `SKIP`, never as
  passed. `--only`, `--skip` and `--list` support targeted runs; command
  overrides (`AG_*_CMD`) keep the checks testable without a Rust toolchain.
- `scripts/install-hooks.sh` sets `core.hooksPath` in the local git config only,
  downloads nothing and prints the removal command. Documented in
  `CONTRIBUTING.md` (English and Spanish).
- `tests/hooks/test-hooks.sh`: 34 standalone assertions covering both hooks,
  descriptor logic, hash and policy checks, command overrides, install/removal
  and exit-code propagation. No network and no cargo required.

This is a local, opt-in developer aid. It does not modify `.github/` or any
master document, so no RFC is required.

### CI maintainability guardrails (RFC-0022)

Added:

- New `guardrails` workflow: `derived-docs` regenerates the verbatim files in a
  scratch copy and fails on any drift from the masters; `file-size` rejects
  tracked Rust files over 600 lines unless listed in the new
  `tools/file-size-allowlist.txt` with an issue number; `script-tests` runs the
  standalone script tests. Stale allowlist entries also fail, so the list can
  only shrink. The 14 current exceptions are seeded with the audit parent
  issue #142 until focused refactoring issues exist.
- New `security-audit` workflow: weekly (and manually triggered) `cargo audit`
  and `cargo deny check --all-features` on `main`. A new `tools/security-report.sh`
  maintains exactly one issue labelled `security`, updated in place, closed
  when both checks pass and reopened if a later run fails. The issue carries an
  ownership marker so human issues
  are never reused, and the scheduled job stays green after reporting.
- New `stale-prs` workflow: daily `tools/close-stale-prs.sh` labels, notes and
  closes pull requests with no activity for 30 days without deleting branches.
  A manual `dry_run` input previews candidates.
- `clippy.toml` now carries the configured 80-line `too_many_lines` threshold.
  The command-table check from RFC-0022 section 4.6 was deliberately not added:
  a literal comparison against clap help text is brittle.

Changed:

- `tools/README.md` documents every guardrail script; `CONTRIBUTING.md` explains
  how to react to each failure.

Tests:

- `tools/test-guardrails.sh` covers the size limit, allowlist format and stale
  entries, derived-doc drift and missing files, all four security-issue states
  (create, update, human-issue preservation, close), and stale-PR closure with
  and without dry run, using a fake `gh` CLI. No network and no cargo needed.

Refs #168 (context: the `--locked` gate it introduced; the issue is closed).

### Opt-in layered project template (RFC-0019 stage A)

Added:

- `ag new <name> --template layered` scaffolds a four-crate Cargo workspace
  (`domain`, `app`, `infra`, `api`) plus a binary composition root and an
  `ag.toml` manifest. Dependencies point inward only: `domain` is pure Rust,
  `app` declares trait ports, `infra` implements them with `ag-data`, and `api`
  owns HTTP transport through `ag-core`. Sibling crates are aliased in
  `Cargo.toml`, so generated Rust identifiers are stable for any project name.
- `crates/ag-cli/tests/layered.rs` scaffolds into a scratch directory and
  asserts the dependency direction, the manifest contents and that the three
  default templates stay unchanged.

Changed:

- The `--template` value parser, the `Template` enum and the interactive
  wizard now offer `layered` as a fourth, opt-in choice. `rest` remains the
  non-interactive default and `ag dev` is unaffected.
- Two TUI wizard test expectations were updated for the longer template list:
  the clamped last selection is now `layered`, and the template-step frame is
  tall enough to show the full file list.
- `templates/README.md` documents the new layout, the dependency rule and the
  manifest.

Scope note: stage A does not add an `ag.toml` reader, layer-aware generator
output or compact-to-layered promotion. A fresh scaffold passes
`cargo metadata --no-deps` and `cargo check` against the pinned `rev`.

Refs #154.

### Wave-4 CLI/config regression suite and STATUS reconciliation

Added:

- `crates/ag-cli/tests/wave4_regression.rs`: a suite for the wave-4 CLI/config
  and machine-output work. `new_no_interactive_is_unchanged` runs unconditionally
  and checks that `--no-interactive` is accepted at both positions, creates the
  scaffold and fails with exit 2 and no side effects when the name is missing.
  Five further tests cover `ag config get/set/validate`, refusal of
  `--output json` on mutating commands, a version field in JSON output from
  read commands, and the inward-only dependency direction of the generated
  layered workspace. Each of those detects its feature at runtime and prints a
  SKIP note when it is absent, so the suite is green on base `42db343` and
  activates without a source change once the relevant branch merges.
- `docs/audits/wave4-status-reconciliation.md`: maps wave-4 branches to
  `docs/roadmap/STATUS.md` boxes, lists the exact evidence commands required
  before each box can be checked, and records which boxes stay open even after
  the merges because of deliberate branch-scope limits (`ag env`, `ag describe`
  and plan/apply are not delivered by this wave).

`STATUS.md` and `docs/master/` are intentionally left unchanged: no roadmap box
is claimed as advanced before its code is merged and verified.

Closes: none.

### Versioned `--output json` for read commands (RFC-0021 stage E, RFC-0024 prerequisite)

Added:

- Global `--output json` for the read-only commands `ag schema lint`,
  `ag schema diff` and `ag workers list`. Each emits one compact, deterministic
  JSON line with `contract_version` (`0.1.0`), `command`, `status` and either
  `data` or a typed `error`. Prose output is unchanged when the flag is absent.
- Versioned contract under `docs/mcp/`: human-readable reference, JSON Schema
  (draft 2020-12) and byte-exact snapshots asserted by
  `crates/ag-cli/tests/json_output.rs`. The constant, schema and snapshots move
  together on every contract bump.
- Mutating commands and read commands without a versioned shape refuse
  `--output json` before dispatch (exit code 2, `unsupported_command`), so no
  file, database or network operation is reached. Refusal messages are closed
  strings; a canary-secret test proves no argument or environment value is
  echoed.
- Stable exit codes in JSON mode: 0 success, 1 read or validation failure,
  2 usage/unsupported. `schema.lint` reports diagnostics as data and exits 1
  when errors are present.

Changed:

- Schema diff now computes one structured change list shared by prose and JSON;
  the `[BREAKING]` / `[additive]` prefixes are derived from it, so the two
  outputs cannot drift.
- `ag generate` keeps its documented `--output DIR` flag. Because the global
  format selector owns that long name, this subcommand exposes the selector as
  `--output-format`; `ag --output json generate` still works and is refused.

Tests:

- 12 integration snapshot tests (per command, error paths, refusal, no side
  effects, prose parity, secret canary) plus parser and envelope unit tests.
- `cargo test --workspace --all-features`, `fmt`, `clippy -D warnings`, `audit`
  and `deny` run green; see the MCP readiness review for the prerequisites
  this change does **not** cover (plan/apply, audit sink, `Secret<T>`, typed
  requests for all groups, `ag describe`).

### Interactive terminal experience, stages B and C (RFC-0018)

Added:

- `ag config` with a full-screen form for `config.toml` in interactive
  terminals and exact non-interactive parity: `ag config get KEY`,
  `ag config set KEY VALUE` and `ag config validate`. `validate` exits
  non-zero on an invalid file so it can be used in CI.
- A shared typed `ConfigRequest` and a single `execute_config` executor used
  by both flags and the form (RFC-0018 principles 1 and 2). The edited keys
  mirror `ag_core::ShieldConfig`; every write is validated with that loader
  before an atomic replace, preserving unrelated keys and comments.
- Bare `ag` in a terminal opens a home screen with keyboard navigation that
  only launches existing commands and prints the command it runs.

Changed:

- The root `ag` command is now optional in the clap definition. Outside a
  terminal, with `CI`, `AG_NO_TUI`, `TERM=dumb` or `--no-interactive`, bare
  `ag` still prints help and every command behaves as before.
- `ag config` normalizes the legacy `[bind] host/port` table emitted by older
  project templates in memory, so scaffolded projects validate without
  changing the templates.

### Wave 4 technical review: security and low-level audit

Added:

- `docs/audits/wave4-review.md` with the read-only review of all seven wave-4
  branches (`tui`, `machinejson`, `layered`, `envsecrets`, `guardrails`,
  `hooks`, `qa`). It records the security-critical verifications, the
  low-level checks and three real defects with severity, affected branch and
  recommendation.
- Three focused issues: #187 (qa layered regression test probes the wrong
  templates and skips silently), #188 (`ag config set` silently corrupts
  integers above `i64::MAX`) and #190 (ag-check silently skips emoji detection
  without `grep -P`). The `ag-cli` area label was created for the CLI findings.
- `docs/INDEX.md` now references the new audit note.

No code owned by another wave-4 role was changed; no push or merge was
performed.

### Native OAuth2/PKCE client in ag-auth (issue #185, RFC-0029)

Added:

- `OAuthState` and `PkceVerifier`, local secret newtypes returned by
  `OAuthClient::authorization_url`. Both expose `expose_secret() -> &str` and
  redact their `Debug` output so a state or verifier cannot be logged by
  accident. Sizes are unchanged: 128-bit state (22 base64url chars) and a
  43-character S256 verifier from 32 CSPRNG bytes.
- `OAuthError::Entropy` for CSPRNG failure (the enum is `#[non_exhaustive]`).

Changed:

- Replaced the `oauth2` 5.0.0 dependency with a native Authorization Code +
  PKCE implementation built only from existing primitives (base64ct, sha2,
  getrandom, url, reqwest, serde_json). Removed `oauth2` from the workspace
  dependency table and from `Cargo.lock`; `cargo tree -i oauth2` is now empty
  and the `oauth2 -> chrono` edge is gone.
- The workspace and the DSL generator migrated from `chrono` to `time` 0.3
  (RFC-0026): models use `OffsetDateTime`, generated fields use RFC 3339 serde,
  and golden-string tests cover fractional seconds and UTC spelling. One
  advisory ignore remains because `object_store 0.14.2` has a non-optional
  chrono dependency behind the opt-in `ag-storage/s3` feature; issue #185 stays
  open until that edge disappears.
- `fuzz/Cargo.lock` was updated to `time` 0.3.55 so the fuzz targets compile
  against the migrated `SignedDuration` API; the fuzz smoke gate failed in CI
  until this separate lockfile was refreshed.
- Public API change in `ag-auth`: `authorization_url` now returns
  `(Url, OAuthState, PkceVerifier)` and `exchange_code` accepts
  `PkceVerifier` instead of oauth2's `CsrfToken` / `PkceCodeVerifier`.
  `OAuthProvider`, `OAuthUser`, both method names and the request/response wire
  format are unchanged. No crate outside `ag-auth` used those newtypes and
  `ag-auth` is `publish = false`.
- Query parameter order, form body order, endpoints, scopes and error mapping
  stay byte-compatible with the former oauth2 5.0.0 behavior (RFC-0029 4.3).

Tests:

- RFC 7636 Appendix B known-answer vector, CSRF state length/uniqueness,
  43-character URL-safe verifier, redacted `Debug`, golden authorization URL,
  form-encoding edge cases, and wiremock token/userinfo tests covering exact
  request body, non-2xx, malformed JSON, missing `access_token`, transport
  failure and the GitHub `User-Agent`.

Refs #185. This removes the blocking oauth2 edge only; issue #185 stays open
until RFC-0026 removes the remaining chrono edges and the deny ignore.

### CI: supported macOS runner (issue #186)

Changed:

- The macOS matrix job now uses `macos-15` instead of the unsupported
  `macos-14` image, restoring the four-platform CI matrix. RFC-0031 records the
  decision; no target, toolchain or test command changed.

### Design proposal: signed release manifest and trust anchor (issue #141)

Added (documentation only; no code, no key, no release assets, no phase change):

- `RFC-0030` proposes offline Ed25519 release-key custody, a signed SHA-256
  manifest published as a release asset, and an independently published public
  key, so installer verification does not depend on an external service.
- The RFC records key rotation and revocation, offline bootstrap, signature
  verification order, Windows and macOS parity, fail-closed behavior, and the
  exact evidence required before issue #141 can be closed.
- Roadmap sections added to `SECURITY.md` and
  `docs/security/INSTALLATION_INTEGRITY.md` state honestly that no key or
  signed artifact exists yet and that the issue stays open.
- Refs #141. The RFC is a draft: it advances the issue but does not close it.

### CI verification for the ag-observe OTLP feature (issue #149, RFC-0028)

Added:

- `RFC-0028` (accepted) records the BDFL decision to add a dedicated
  `ci / ag-observe otlp feature` job that checks, tests and lints
  `ag-observe` with the opt-in `otlp` feature enabled. The default-feature
  matrix is unchanged; the job makes neither `protoc` nor an external
  collector mandatory.
- `ADR-0019` now cross-references RFC-0028 for the verification coverage
  added after the original decision.

Purpose:

- Makes the closure evidence for issue #149 reproducible in CI. Local
  verification on 2026-11-22 passed: `cargo check`/`cargo test` with
  `--features otlp` (14 tests, 1 ignored doc-test), `cargo fmt --check`,
  clippy with `-D warnings` for both the feature and the full workspace,
  `cargo audit` (4 allowed warnings) and `cargo deny --all-features check`.
  A fresh target-dir build with `PROTOC=/nonexistent/protoc` also passed,
  confirming no protobuf compiler is needed.

### Committee decisions (RFC-0025, RFC-0027, issue #152)

Added (documentation only; no code, no dependency, no phase change):

- `ADR-0020` records the BDFL decision of 2026-11-22: RFC-0025 is accepted as
  design and its implementation is deferred until Phase 5 opens and the
  pre-Phase-5 release gate closes. Issue #154 stays open.
- `ADR-0021` records the BDFL decision of 2026-11-22: RFC-0027 is accepted with
  its recommendation to defer the chumsky upgrade. Keep `chumsky 0.9.3` until
  the DSL v1.0 grammar freeze or a concrete blocker. Issue #180 stays open.
- `docs/modules/ag-mail/managed-api-dx-design.md` records the 2026-11-22
  decision to defer Mail C/D until the RFC-0009 Mail A gate has evidence or an
  RFC-0009 amendment moves them out under Hoja de Ruta 4.6.3. Issue #152 stays
  open.

Changed:

- Status lines in RFC-0025 section 10 and RFC-0027 section 9 now carry date and
  decider (rule 28); the RFC and ADR indexes reflect the accepted statuses.

### Design proposal: ag-data ORM, RLS and tenancy (issue #154)

Added (documentation only; no code, no phase change):

- `RFC-0025` proposes a Phase 5+ design for DSL-generated typed repositories,
  explicit `TenantContext`, transaction-local RLS scope and expand/contract
  migrations. It is a proposal, not accepted and not implemented.
- The design incorporates the data-model review note at
  `docs/modules/ag-data/data-layer-rls-tenancy.md`, including the pooled
  connection hazard, fail-closed policies, generated output paths and the
  dependency on the issue #185 date/time decision.

### Crate roster: ag-ui removed (issue #182, RFC-0019)

Removed:

- `ag-ui` from the workspace, the crate roster (rule 14), the three master
  documents, the README tiers and the generated module scaffolding. The
  backend-only scope of `RFC-0019` section 4.6 now holds without exceptions;
  UI consumers use generated client SDKs instead.
- The empty `crates/ag-ui/` and `docs/modules/ag-ui/` skeletons.

Changed:

- Crate count 20 to 19 (placeholders 6 to 5) across masters, CLAUDE.md and
  README (EN+ES). Chapter 8.7 of the architecture master now records the scope
  decision rather than a UI module.
- Master hashes in `VERSION.md` and `.github/workflows/docs.yml`, derived files
  and the line ranges of `tools/split-masters.sh` were recalculated.

### Master documentation consistency (issues #170, #172)

Changed:

- Blueprint v4.1 now describes `ag-mail`'s native outbound MTA, the
  implemented `ag-cache` L1/L2 layers and the delivery state through Phase 4.7.
- The Hoja de Ruta master states that its boxes describe the contract and that
  verified state lives in `docs/roadmap/STATUS.md`; all boxes were reset to
  unchecked and the derived files were regenerated.
- Master hashes were updated in `VERSION.md` and `.github/workflows/docs.yml`.

### Proposal: shared MCP machine interface (RFC-0024)

Added (documentation only; no code):

- `RFC-0024` defines a versioned MCP contract shared by Anti-Gravital and the
  separate Gravital Fabric platform, with capability-scoped agents, plan/apply
  for mutations, an audit trail and an opt-in self-debug tool group.
- `ADR-0016` fixes the boundary: open versioned contracts and no compile-time
  dependency between `ag-*` and `fabric-*` crates.
- `ADR-0017` records that MCP never grants privilege by virtue of being a
  machine and that debug tools are opt-in and stdio-only.
- A readiness review recorded in `docs/audits/mcp-readiness-review.md` adds
  seven implementation prerequisites to RFC-0024: machine output, plan/apply,
  `Secret<T>`, typed executors for all command groups, a single plan model,
  a contract-version owner, and no DNS egress in the stdio-only debug tools.
- `docs/diagrams/mcp-layer-flow.md` shows the agent-to-executor flow and the
  boundary with Fabric.

### Native MTA and mixed TLS providers (issue #181)

Fixed:

- The native MTA now installs the process-level `ring` crypto provider before
  `mail-send` builds its rustls client. Combining `ag-mail/mta` with
  `ag-storage/s3` no longer panics with two linked crypto providers.
  `cargo test --workspace --all-features` passes again and the release gate
  Tests row was re-evaluated.

### Secure development default (issue #169)

Changed:

- `ag dev` now listens on `127.0.0.1:8080` by default instead of `0.0.0.0:8080`.
  Listening on all interfaces requires the explicit `--bind 0.0.0.0:8080`
  opt-in. Generated project templates keep `0.0.0.0:8080` for container use;
  the difference is documented in `crates/ag-cli/README.md`.

### Security and dependency remediation

Changed:

- The RustCrypto generation was upgraded together: `p256` 0.14,
  `elliptic-curve` 0.14, `jsonwebtoken` 12, `sha2` 0.11, `hmac` 0.13,
  `ed25519-dalek` 3, `rsa` 0.10, `rand` 0.10 and `getrandom` 0.4. WebAuthn
  ES256 verification now uses `Sec1Point` with explicit 32-byte coordinate
  checks. This removes `crypto-bigint` 0.5.5 (RUSTSEC-2026-0387).
- `sqlx` 0.9 replaces `runtime-tokio-rustls` with `runtime-tokio` plus
  `tls-rustls-ring-webpki`. Dynamic SQL in the `ag-workers` PostgreSQL backend
  is marked safe after audit: only static column lists are interpolated and
  every runtime value is bound.
- `lettre` 0.11.24 clears six SMTP advisories; `libsqlite3-sys` is locked at
  0.39.0. `cargo audit` reports zero vulnerabilities.
- The PowerShell installer now delimits the script name in the integrity error
  message (`${scriptName}`), fixing the Windows parser failure found in CI. The
  installer checksum file was regenerated.
- `deny.toml` records dated ignores for RUSTSEC-2026-0440 (only reachable from
  testcontainers dev-dependencies), RUSTSEC-2026-0543 (`chrono`, tracked in
  issue #185) and RUSTSEC-2026-0542 (`humantime` in adapter paths). It also
  allows NCSA and BSL-1.0 for all-features builds (both OSI-approved). The
  stale RUSTSEC-2026-0104 ignore was removed (issue #183).

### Quality gate

Changed:

- Every cargo invocation in CI that resolves dependencies now uses `--locked`,
  so a manifest change without a matching `Cargo.lock` fails the build (issue
  #168, guardrail anticipated in RFC-0022 section 7).

- The coverage gate now excludes the CLI dispatch and terminal plumbing files
  produced by the RFC-0018 split of `ag-cli/src/main.rs`: `cli.rs`,
  `commands/domains.rs`, `commands/mail.rs`, `commands/schema.rs`,
  `commands/workers.rs`, `interactive.rs`, `tui/mod.rs` and `tui/terminal.rs`.
  Project creation logic, `request.rs` and the TUI wizard state machine stay
  measured. The exclusion restores the original "library code only" intent of
  the gate after the refactor.

### ag-cli: interactive project creation wizard (`RFC-0018`, stage A)

Added:

- `ag new` opens a guided full-screen wizard when an input is missing and the
  session is an interactive terminal: project name with live validation,
  template selection with the files each template creates, and a review
  screen. Nothing is written before the review is confirmed. After creating
  the project, `ag` prints the equivalent non-interactive command.
- Global `--no-interactive` flag. `ag` also never prompts when standard input
  or output is not a terminal, when `CI` or `AG_NO_TUI` is set, or when `TERM`
  is `dumb`.
- Monochrome theme under `NO_COLOR`; sequential line prompts in terminals
  smaller than 80x24; terminal state restored on exit, error, `Ctrl-C` and
  panic.
- `tui` Cargo feature (default on). `--no-default-features` builds contain no
  interactive code.
- Dependency: `ratatui` 0.30 (optional, `tui` feature), with its re-exported
  `crossterm` backend.

Changed:

- `ag new`: `NAME` is now optional. With `NAME` and `--template` given the
  behavior is unchanged. In a non-interactive session a missing `NAME` is a
  usage error with exit code 2 (previously a clap error with the same code);
  a missing `--template` still defaults to `rest`.
- `ag-cli` source split into `cli.rs`, `commands/` (one module per command
  group), `request.rs` and `interactive.rs`; `main.rs` went from 1 829 lines
  to dispatch only. No behavior change in the other commands.
- `RFC-0018` to `RFC-0023` accepted by BDFL decision on 2026-11-10; their
  decision sections record the resolution of each open question. Two
  questions of `RFC-0021` (scope of `ag setup`, release key custody) remain
  open until its stage C.

Removed:

- `dialoguer` dependency. `ag-cli` has a single interaction library.

### Proposal: serverless functions (`RFC-0023`)

Added (proposed; no code):

- `RFC-0023`: serverless functions as sandboxed WebAssembly components run by
  `ag-wasm-host`, declared in `schema.ag`, triggered by HTTP, schedule, queue
  or events, with deny-by-default capabilities and per-invocation limits;
  external function platforms and containers as packaging adapters.
- Roadmap master: additive Phase 5.5 after the public beta.

### Toolchain and dependency refresh

Changed:

- Rust toolchain and MSRV: 1.95.0 to 1.99.1 (`rust-toolchain.toml`,
  `rust-version`, `clippy.toml`, installers and their checksums, CI, README,
  manual, CONTRIBUTING, `examples/todo-api/Dockerfile`).
- All semver-compatible updates applied (`cargo update`).
- Major upgrades: `axum` 0.7 to 0.8, `tower-http` 0.6 to 0.7, `thiserror` 1 to
  2, `toml` 0.8 to 1, `governor` 0.7 to 0.10, `jsonwebtoken` 10 to 11,
  `logos` 0.14 to 0.16, `opentelemetry` and `opentelemetry_sdk` 0.27 to 0.33,
  `tracing-opentelemetry` 0.28 to 0.34, `metrics-exporter-prometheus` 0.16 to
  0.18, `console-subscriber` 0.4 to 0.5, `async-nats` 0.48 to 0.52, `redis`
  0.27 to 1, `passkey-types` 0.5 to 0.6, `x509-parser` 0.16 to 0.18, `rcgen`
  0.13 to 0.14, `quick-xml` 0.41 to 0.42, `minijinja` 2 to 3, `base64` 0.22 to
  0.23, `criterion` 0.5 to 0.8.
- `axum` 0.8 migration: route captures use `{name}` and `{*name}` instead of
  `:name` and `*name` (`ag-domains` API, `ag-storage` server, examples,
  benchmark, generated router). The DSL already used `{name}`, so
  `ag_dsl::ast::dsl_path_to_axum` now returns the path unchanged and
  `ag generate` emits `{name}` routes. Shield extractors no longer use
  `axum::async_trait`.
- `ag-cache`: RESP bulk-string parsing rewritten with `?` (lint of the new
  toolchain; no behavior change).

Known issue surfaced by this refresh, already present on `main`: a workspace
build with `--all-features` links two `rustls` crypto providers and two
`ag-mail` MTA tests panic (#181). Default features and the per-crate feature
sets that CI runs are green.

Not upgraded (each tracked in its own issue): `sqlx` 0.9 (#175), the
RustCrypto generation (#176), `reqwest` 0.13 (#177), the mail protocol stack
(#178), `instant-acme` 0.8 (#179), `chumsky` 0.13 (#180). `testcontainers`
0.28 is blocked upstream by `testcontainers-modules` (#174). The line-prompt
library is left as is because `RFC-0018` removes it.

Note for generated projects: the templates under `templates/` pin `ag-core`
by commit and declare `axum` 0.7. They keep building against that commit.
After this change merges, the pin and the `axum` version in the three
templates must be bumped together (procedure in `templates/README.md`).

### Architecture proposals: project layers, environment, installation, guardrails

Added (all proposed; no code):

- `RFC-0019`: layered architecture of generated projects (domain, application,
  infrastructure, interface, composition root) enforced by Cargo, the
  `ag.toml` project manifest, and the backend-only interoperability scope
  (Docker, Kubernetes, Terraform, gRPC, Go, WebAssembly, Flutter).
- `RFC-0020`: declared environment variables with fail-fast typed loading, a
  `Secret<T>` type, an encrypted per-environment secrets file and the `ag env`
  command group.
- `RFC-0021`: verified one-command installation from signed static binaries,
  `ag setup`, a remote console over SSH (`ag connect`) and a machine interface
  (`--output json`, `ag describe`, plan and apply).
- `RFC-0022`: maintainability guardrails, each paired with the CI mechanism
  that enforces it.
- Roadmap master: additive Phase 4.8 (project foundations) with stages A-E;
  Phase 5 gains Phase 4.8 stages A and B as an entry criterion and a Terraform
  integration deliverable.

### Roadmap amendment: Phase 4.6 and 4.7 sections, staged later phases (`RFC-0018`)

Added:

- `docs/rfc/RFC-0018-interactive-terminal-experience.md` (proposed): an
  interactive full-screen terminal interface for the `ag` binary to create and
  configure a project, with exact non-interactive equivalents. No code yet.
- Roadmap master: a dedicated Phase 4.6 section (mail and workers tracks with
  per-stage state), a new additive Phase 4.7 (interactive terminal
  experience), and a Stages block plus RFC prerequisites in Phases 5 to 10.

Changed:

- Roadmap master: Phase 5 entry criteria now name the pre-Phase-5 release gate
  and its open issues; Phase 5 delivers the self-hosted target before hosted
  ones (`ADR-0009` rule 5).
- `tools/split-masters.sh` ranges recomputed; derived files under
  `docs/roadmap/` regenerated, with new files for Phases 4.6 and 4.7.
- `docs/modules/ag-cli/README.md` rewritten to the real command surface.

Fixed:

- `Cargo.lock` was missing the dependency tree of the `ag-storage` `avif`
  feature (`rav1e`, `ravif` and transitive crates), so a clean build rewrote it
  and `cargo audit` inspected an incomplete lockfile.
- Security: lockfile updates for advisories published since the last green
  `cargo audit` run: `rustls` 0.23.46, `rustls-webpki` 0.103.16, `quinn-proto`
  0.11.19, `h2` 0.4.20, `webpki-roots` and `webpki-root-certs` 1.0.11, and
  `tokio` 1.54.1. This resolves 13 of the 15 reported vulnerabilities; the two
  without a semver-compatible fix (`crypto-bigint`, `libsqlite3-sys`) are
  tracked in issue #173, and `cargo audit` stays red until it closes.
- Roadmap master: repository URL in Phase 0, importer count in Phase 7 (six,
  not five), and the retired `wasm32-wasi` target name in Phase 10
  (`wasm32-wasip1`).
- Root README: the Spanish Phase 4.6 row had drifted from the English one.

Removed:

- Six descriptors under `docs/pr-drafts/` whose pull requests were already
  merged (descriptor life-cycle rule).

### ag-storage: lossy WebP encoding with configurable quality (`webp-lossy`)

Added:

- `ag-storage`: the new opt-in `webp-lossy` feature makes `ImageProcessor::to_webp`
  honor its `quality` argument (0..=100; 100 = lossless) via the native `webp`
  encoder (bundled libwebp, built from source with `cc`; no system library
  required). Without the feature the default stays lossless via the `image`
  crate and `quality` is ignored. Note: `--all-features` builds now compile
  libwebp.

### ag-storage: implement the `auth` feature (JWT Ed25519 via ag-auth)

Added:

- `ag-storage`: with the `auth` feature, the HTTP server validates the Bearer
  as an Ed25519 JWT through `ag-auth` when a PEM public key is configured
  (`StorageConfig::jwt_public_key_pem`, env `STORAGE_JWT_PUBLIC_KEY` /
  `STORAGE_JWT_PUBLIC_KEY_PATH`). Static-token mode remains the default; the
  public-bind security check now treats a configured JWT key as authenticated.
  The middleware state changed from `Arc<String>` to `AuthState`.

### ag-auth: remove panic paths from the public WebAuthn API

Changed:

- `ag-auth`: `WebAuthnRp::start_registration` and `start_authentication` now
  return `Result<_, WebAuthnError>` and propagate a new `WebAuthnError::Entropy`
  variant instead of panicking when the system entropy source fails. This is a
  pre-1.0 breaking change to those two signatures.
- `ag-auth`: `parse_raw_auth_data` reads its fixed-size header via
  `split_first_chunk`, so the length invariant is proven by the type system and
  no slice `unwrap` panic is reachable; a short buffer returns a typed error
  (covered by a new test).
- `ag-auth`: the `RefreshBlacklist` lock-poisoning policy (fail fast on a
  poisoned lock for a security control) is documented once at module level.

### ag-storage: English docs and display strings (ADR-0008)

Changed:

- `ag-storage`: `crates/ag-storage/src/signed.rs` doc comments, `Display`
  messages and test messages are now English. The `Display` strings of
  `StorageError` (observable in logs and error payloads) were translated as
  well. No variant, field or public API changed. The remaining non-ASCII
  bytes in `ag-storage` source are deliberate Unicode test fixtures, not
  Spanish text.

### ag-auth: English residual comments and messages (ADR-0008)

Changed:

- `ag-auth`: residual Spanish comments and `expect`/`assert`/panic messages in
  `crates/ag-auth/src` are now English. The `Display` strings of `JwtError`,
  `AuthMailerError` and `AuthConfigError` (observable in logs and error
  payloads) were translated as well. No variant, field or public API changed.

### ag-mail: English error display strings (ADR-0008)

Changed:

- `ag-mail`: the `#[error(...)]` display strings of `AgMailError`
  (`crates/ag-mail/src/error.rs`) are now English. These strings are
  user-visible: they surface in logs, traces and REST error payloads.
  `SendExhausted`, `InvalidAddress`, `Template`, `VarMismatch`, `Config`,
  `Provider`, `Dns`, `NoMailHost` and `Dkim` changed their rendered text;
  no variant, field or public API changed.

### README raiz reestructurado (2026-06-12)

Cambiado:

- `README.md`: reestructuracion completa de la ventana publica del proyecto.
  Anade tabla "de un vistazo" con el estado real (pre-release, puerta
  pre-Fase 5 abierta), secciones explicitas "que se puede hacer hoy" y "que
  no se puede hacer todavia", extracto real del Anti-DSL, diagrama Mermaid
  de arquitectura, arbol del repositorio, principios de ingenieria, modelo
  de fases explicado (bloqueantes vs aditivas, implementada vs cerrada),
  tabla de fases con evidencia y bloqueadores, mapa de documentacion con
  orden de lectura para humanos y agentes automatizados, y espejo completo
  en espanol (ADR-0008). Sin cambios de comportamiento del codigo.

### Correctivo pre-Fase 5 - Observabilidad y toolchain (2026-06-10)

Corregido:

- `ag-observe`: `/metrics` ahora renderiza el snapshot real de Prometheus
  mediante `PrometheusHandle`; si el exporter no fue inicializado, responde de
  forma explicita en vez de devolver un cuerpo vacio.
- `ag-observe`: configurar `OTEL_EXPORTER_OTLP_ENDPOINT` ahora falla con
  `ObserveError::OtlpSetup` hasta que exista un exporter OTLP real, evitando
  aceptar una configuracion que descartaba trazas.

Cambiado:

- MSRV declarado alineado a Rust 1.95.0 en `Cargo.toml`, `clippy.toml`,
  documentacion de instalacion/contribucion y CI (`cargo check` dedicado).

### ag-workers: RFC-0012 + ADR-0013 (fundacion documental, Fase 4.6-D)

Anadido:

- `docs/rfc/RFC-0012-ag-workers.md`: motor de ejecucion en segundo plano nativo
  (jobs tipados, reintentos, DLQ, scheduling, worker pools). Estandar diferido,
  precedente `ag-mail` (`ADR-0007`). Alcance fijo; entrega secuenciada S1-S7.
- `docs/adr/0013-ag-workers-execution-model.md`: modelo de ejecucion (maquina de
  estados, dos backends, poison guard bajo `panic = "abort"`, scheduling singleton,
  semantica at-least-once) y autorizacion del BDFL para implementar con el gate
  pre-Fase 5 abierto, sin afirmacion de produccion/GA hasta que cierre.
- `docs/modules/ag-workers/README.md`: documentacion del modulo.
- `docs/architecture/05-ecosistema-modulos.md`: fila `ag-workers` (estandar diferido)
  y nota de conteo del ecosistema.
- `docs/architecture/08-modulos-batteries-included.md`: seccion 8.10.
- `docs/roadmap/STATUS.md`: Fase 4.6-D con criterios de entrada/entregables/salida.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`: fila 4.6 en el calendario y nota de fase.

Decisiones fijadas (RFC-0012 seccion 9, ADR-0013): documentacion primero y codigo en
etapas S1-S7; nueva dependencia de workspace `tokio-util` para `CancellationToken`;
`enqueue_in_tx` acepta `&mut sqlx::Transaction` tras la feature `postgres` (sin `AgTx`
en `ag-data`; brecha rastreada como Issue `tech-debt`, regla 29). No se anaden
entradas a `docs/DEBT.md` (congelado).

### ag-edge: listener de upgrade HTTP->HTTPS (cierra issue #91)

Anadido:

- `crates/ag-edge` feature `server`: `serve_http_redirect` / `http_redirect_router`
  - un listener de puerto 80 que sirve el challenge ACME HTTP-01 sobre HTTP plano
  (no se redirige, o se rompe la emision) y redirige todo lo demas a
  `https://<host><path>?<query>` con el status indicado (308/301). Tests de
  integracion sobre TCP real.

### Gobernanza: deuda tecnica en GitHub Issues (CLAUDE.md regla 29)

Cambiado:

- CLAUDE.md regla 29: la deuda tecnica, bugs, fallos y documentacion de
  problemas se registran como GitHub Issues con etiqueta `tech-debt`, no como
  archivos del repositorio. `docs/DEBT.md` queda congelado (historico); se
  prohiben nuevos archivos tipo `BACKLOG.md`/`TODO.md`/`*-DEBT.md`.
- Eliminado `docs/ag-domains/BACKLOG.md`; su contenido vive en el issue #76.

### ag-domains: endpoint verify + capacidades de proveedor + comando diagnose

Anadido:

- `crates/ag-domains/src/api.rs`: `POST /v1/domains/attachments/{id}/verify` con
  `OwnershipVerifier` inyectable (`NullVerifier` por defecto;
  `PropagationOwnershipVerifier` tras feature `propagation`). Emite
  `domain.ownership.verified` y registra fallos de verificacion. Cierra el
  hueco de que el evento estuviera modelado pero nunca emitido.

- `crates/ag-domains/src/provider/capabilities.rs`: registro de capacidades por
  proveedor (`ProviderCapabilities`, `known_provider_capabilities`,
  `capabilities_for`) — eje adaptador (manual/read_apply) vs features del
  proveedor (apex alias, CNAME flattening, DNS-01 auto). Lista conservadora.
- `crates/ag-domains/src/api.rs`: endpoint `GET /v1/domains/provider-capabilities`
  (feature `api`) + test de integracion.
- `crates/ag-cli`: comando `ag domains diagnose` — compara registros esperados vs
  observados en resolvers publicos y reporta hallazgos accionables; usa
  `propagation::lookup_observed` (nuevo, lookup A/AAAA/CNAME/TXT) + `diagnostics`.
- Docs: `reference/provider-capability-matrix.md`, `how-to/connect-providers.md`,
  `how-to/domain-connect.md`, `how-to/configure-wildcard.md`,
  `how-to/troubleshoot.md`; `reference/cli.md` documenta `diagnose`; OpenAPI con
  `/provider-capabilities`.

### ag-domains: orquestacion de emision de certificados (RFC-0011, fase 2)

Anadido:

- `crates/ag-domains/src/issuance.rs`: proteccion de rate-limit del blueprint
  §13.4. `CertIssuer` trait (el emisor ACME real se inyecta), `IssuedCertificate`,
  `san_key` (clave de deduplicacion canonica por conjunto SAN), `IssuanceLimiter`
  (dedup de ordenes en vuelo + contadores por dominio registrable) y
  `issue_certificate` (reserva, emite, cuenta exitos, libera). Logica nativa sin
  dependencias externas; tests con issuer mock.
- `crates/ag-edge/tests/issuance_to_edge.rs`: test E2E que emite via la
  orquestacion (issuer rcgen), carga el PEM en el `CertStore` del edge y lo
  selecciona por SNI; verifica la cadena emision -> serving completa.

### ag-domains: event log de dominio + metricas de control-plane

Anadido:

- `crates/ag-domains/src/events.rs`: `DomainEvent` (vocabulario blueprint
  §16.2: `domain.attachment.created`, `domain.ownership.verified`,
  `domain.detached`) y trait `EventSink` con `NullEventSink` (defecto),
  `InMemoryEventSink` (nativo/tests) y `TracingEventSink`. Sin broker externo
  (ADR-0009). La API REST emite eventos en create/detach (`ApiState::with_events`).
- `crates/ag-domains/src/metrics.rs`: contadores `ag_domains_attachments_total`,
  `ag_domains_detached_total`, `ag_domains_verification_failures_total`.
- Tests: unitarios del event log + test de integracion que captura los eventos
  emitidos por la API real.

### ag-domains: store SQL Postgres (RFC-0011, fase D)

Anadido:

- `crates/ag-domains` feature `sql-store`: `SqlAttachmentStore` (Postgres via
  sqlx) que implementa `AttachmentStore`. Almacena el attachment como JSONB con
  columnas indexadas (id, hostname, lifecycle) y tabla de tombstones; esquema
  embebido + `migrations/0001_ag_domains_attachments.sql`. Mapea violacion de
  unicidad a `AlreadyExists` y respeta tombstones en `create`. El store nativo
  (memoria/JSON) sigue siendo el predeterminado (ADR-0009). Tests de integracion
  `#[ignore]` que requieren `DATABASE_URL`.

Cambios internos (no rompen la API publica del crate):

- El trait `AttachmentStore` pasa a asincrono y `&self` (mutabilidad interior),
  para poder compartir un store como `Arc<dyn AttachmentStore>` entre tareas y
  permitir backends async. `InMemoryStore`/`JsonFileStore` adaptados; la API REST
  ya no envuelve el store en un `Mutex` externo; la CLI ejecuta los comandos de
  dominios sobre el runtime tokio.

### ag-domains/ag-edge: edge en vivo + API REST (RFC-0011, fases B y C)

Anadido:

- `crates/ag-edge` feature `server`: listener HTTP ejecutable (`serve_http`,
  axum) que sirve el challenge ACME HTTP-01 (`Http01ChallengeStore` +
  `acme_challenge_token`, con proteccion ante path traversal), aplica politica
  canonica/redireccion y enruta por `Host`/`:authority` con fail-closed para
  hosts desconocidos. Tests de integracion sobre TCP real.
- `crates/ag-edge` feature `tls`: almacen `CertStore` de certificados rustls
  (exacto + wildcard) con puente PEM desde la salida de emision ACME
  (`insert_pem`), `SniCertResolver` (RFC 6066) y `serve_https`. Test de
  integracion con handshake TLS real verificando seleccion por SNI.
- `crates/ag-domains` feature `api`: API REST `/v1/domains/attachments`
  (crear/listar/obtener/instructions/status/detach) respaldada por cualquier
  `AttachmentStore`, con mapeo de errores a codigos HTTP. Tests de integracion
  sobre HTTP real (`tests/api_rest.rs`).
- `openapi/ag-domains.v1.yaml` actualizado al contrato implementado.

Cambios de workspace:

- `hyper-util`: features `server-auto` y `service` anadidas (para el listener
  HTTPS del edge).

### ag-domains plano de control + ag-edge (ADR-0012 / RFC-0011, fase A)

Anadido:

- `crates/ag-domains`: capa de plano de control aditiva. Modulos `hostname`
  (normalizacion/clasificacion IDN via `idna`), `attachment` (maquina de estados
  con dimensiones ownership/dns/tls/routing), `store` (`AttachmentStore` trait +
  `InMemoryStore` + `JsonFileStore` nativos), `ownership` (token TXT + verificacion
  via propagacion), `instructions` (motor de registros DNS + exportacion zona BIND),
  `caa` (preflight CAA antes de ACME) y `diagnostics` (esperado vs observado). La
  libreria declarativa DNS+TLS previa queda intacta. Dependencias nuevas `idna`,
  `uuid`. 80 tests.

- `crates/ag-edge` (nuevo crate, plano de datos): `router::resolve_hostname`
  (precedencia exacto/wildcard/legacy con fail-closed para hostnames
  desconocidos), `tls::SniCertStore` + `allow_on_demand` (emision on-demand
  restringida), `redirect::CanonicalPolicy` (www<->apex preservando path/query).
  Libreria pura y probada; sin listeners en vivo todavia. 17 tests.

- `crates/ag-cli`: subcomandos `ag domains attach`, `instructions`, `export-zone`,
  `status`, `list`, `verify`, `detach`. Flujo manual sin credenciales de proveedor.

- Gobernanza: `docs/rfc/RFC-0011-ag-domains-control-plane.md`,
  `docs/adr/0012-ag-domains-control-plane.md`. Documentacion en
  `docs/ag-domains/` y esqueleto OpenAPI en `openapi/ag-domains.v1.yaml`.

Estado real tras las fases A-C: los listeners HTTP/HTTPS, el responder HTTP-01
y la API REST ya estan implementados. Permanecen diferidos el almacen SQL
(Postgres, fase D), Domain Connect y adaptadores de proveedor adicionales
(fase E; Cloudflare y exportacion BIND ya existen), wildcards DNS-01 y el modulo
`ag-registrars` (fase F).

### ag-mail - Motor de plantillas externo (minijinja) (2026-06-04)

Aditivo, tras la feature `minijinja` (apagada por defecto). Cierra DEBT-003.

- `template::jinja::MinijinjaTemplate`: implementa el trait `MailTemplate` sobre
  el motor `minijinja` (loops, condicionales, filtros), alternativa drop-in al
  `StringTemplate` por defecto. Dep `minijinja` tras la feature.

### Fase 4.6-C - MTA: webhooks firmados (2026-06-04)

Aditivo, tras la nueva feature `api` (apagada por defecto). Avanza DEBT-021.

- `api::webhook`: firma y verificacion de webhooks HMAC-SHA256 sobre
  `{id}.{timestamp}.{payload}` con secretos `whsec_`, cabecera `v1,<base64>`,
  soporte de multiples firmas (rotacion) y ventana de tolerancia anti-replay;
  verificacion en tiempo constante. Deps `hmac`/`sha2`/`base64` tras `api`.
- `.github/workflows/ci.yml`: el job `mail-mta` ahora ejercita `mta,api`.

### Fase 4.6-B - MTA: colas, shaping, egress y suppression (2026-06-04)

Aditivo, tras la feature `mta` (apagada por defecto). Cierra DEBT-019 (núcleo) y
avanza DEBT-020. Persistencia durable del spool queda como DEBT-023.

- `sender::mta::egress`: `EgressSource`/`EgressPool` con weighted round-robin
  suave (SWRR) para IP warming; `MtaSender` elige una fuente por mensaje y
  aplica EHLO + IP de origen al conectar.
- `sender::mta::shaping`: `ShapingLimits`/`ShapingConfig` por `site_name`
  (rate token-bucket determinista + cap de conexiones por semáforo).
- `sender::mta::queue`: cola de dos niveles en memoria (scheduled min-heap +
  ready batch `max_ready`) con `MtaRetryPolicy` (backoff exponencial, max-age),
  trait `DeliveryBackend` (lo implementa `MtaSender`), worker `run`, y reportes
  por ciclo. `MtaSender::build_jobs` genera jobs por dominio desde un `Email`.
- `sender::mta::suppress`: `SuppressionList` automática (hard bounce 5xx,
  límite de reintentos), case-insensitive; la cola la consulta y la alimenta.
- `sender::mta::dsn`: intake asíncrono de DSN (RFC 3464) y ARF (RFC 5965) vía
  `mail-parser`; `process_dsn`/`process_arf` alimentan la suppression list
  (cierra DEBT-020).
- `MtaSender` clasifica los fallos SMTP: solo un 5xx del destino es permanente
  (suprime); fallos propios (DKIM, MX, TLS) son transitorios (reintentables).
- Métricas de profundidad de cola; tests deterministas (reloj explícito).

### Fase 4.6-A - Endurecimiento: RSA DKIM, metricas y CI del MTA (2026-06-04)

Aditivo, cierra deuda de la 4.6-A (DEBT-017/018).

- `sender::mta::dkim`: firma DKIM ahora soporta Ed25519 **y RSA-SHA256**
  (PKCS#8 DER y PEM) via `mail_auth ... RsaKey::from_key_der` (no deprecado);
  tests positivos para ambos algoritmos. Nueva dep `rustls-pki-types` tras `mta`.
- `MtaSender::send`: emite metricas `ag_mail_sent_total` /
  `ag_mail_send_latency_seconds` y `ag_mail_retry_total` en failover de MX.
- `.github/workflows/ci.yml`: nuevo job `mail-mta` (build + test + clippy
  `--features mta`), antes el MTA no se compilaba en CI (RFC-0009 §4.8).
- `docs/DEBT.md`: renumeradas las entradas de MTA (colision de DEBT-012/013/014);
  DEBT-017/018 cerradas; 4.6-B/C/D y el test de entrega en vivo quedan como
  DEBT-019..022.

### Fase 4.6-A - Implementacion: motor MTA outbound nativo de ag-mail (opt-in)

Aditivo. Nueva feature de Cargo `mta` en `ag-mail`, apagada por defecto; el
sender por defecto y los adapters no cambian (ADR-0010 / RFC-0009).

Anadido:

- `crates/ag-mail` feature `mta`: `sender::mta::MtaSender` (implementa
  `MailSender`) con entrega directa al MX destino via ESMTP+STARTTLS
  (`mail-send`), agrupacion de destinatarios por dominio y construccion MIME
  RFC 5322 (`mail-builder`).
- `sender::mta::resolve`: resolucion MX (`hickory-resolver`) con orden por
  preferencia, rollup `site_name` y fallback de MX implicito (RFC 5321).
- `sender::mta::dkim`: firma DKIM Ed25519 (RFC 8463) aplicada al final;
  el material de clave lo aporta el llamador / `ag-domains`.
- `sender::mta::bounce`: clasificador de bounces SMTP/RFC 3463 (transitorio
  vs permanente), puro y unit-testeado.
- Nuevas variantes de error `AgMailError::{Dns, NoMailHost, Dkim}`.
- Dependencias opcionales tras `mta`: `mail-send 0.6` (features `builder`,
  `ring`, `tls12`), `mail-auth 0.9` (`ring`), `hickory-resolver 0.26`.

Notas: la ruta de entrega en vivo se cubre con tests `#[ignore]` (requiere
red/puerto 25). Deuda registrada en `docs/DEBT.md` (DEBT-012 RSA DKIM,
DEBT-013 metricas+CI, DEBT-014 colas/shaping/DSN-FBL).

### Gobernanza - Pivot ag-mail a MTA outbound nativo (2026-06-03)

Solo documentacion y gobernanza; sin codigo funcional nuevo.

- `docs/adr/0010-ag-mail-native-mta-pivot.md`: nuevo ADR que supersede el
  alcance v1 "NO es un MTA / inbound nunca" de `ADR-0007` y expande `ag-mail`
  a un MTA outbound nativo (resolucion MX, ESMTP+STARTTLS, firma DKIM,
  clasificacion de bounces), por fases y opt-in tras features de Cargo,
  conservando el patron Native | Adapter y la direccionalidad `ag-auth ->
  ag-mail`. `ADR-0007` sigue vigente para `ag-domains` y la clasificacion.
  Principio aditivo-only vinculante: el MTA se anade como `MtaSender` opt-in
  sin degradar el baseline (default `SmtpSender`, adapters de proveedor
  existentes y colas permanecen sin cambios). Se rechaza el "overwrite / MTA
  por defecto / degradar adapters / migrar la cola" del blueprint. Las
  plataformas externas citadas en la investigacion son solo referencia, no
  dependencias ni objetivos de integracion del proyecto.
- `docs/rfc/RFC-0009-ag-mail-native-mta.md`: plan tecnico (6 subsistemas,
  dependencias `mail-send`/`mail-builder`/`mail-auth`/`mail-parser`/
  `hickory-resolver` tras feature `mta`, cola de dos niveles, modelo de datos,
  superficie REST, webhooks firmados HMAC-SHA256, plan por fases 4.6-A..D + Fase 5+).
  Supersede a `RFC-0006` para el alcance de `ag-mail`.
- Alineacion documental: `docs/modules/ag-mail/README.md` reescrito (baseline
  implementado real + direccion MTA, fin del estado "Pendiente"); notas de
  actualizacion de alcance en Arquitectura `§8.8` (maestro EN+ES y derivado) y
  nota futura `§4.5.5` en Hoja-de-Ruta (maestro EN+ES y derivado de fase);
  README raiz (EN+ES); indices de ADR y RFC; `RFC-0006`/`ADR-0007` marcados
  superseded para el alcance de `ag-mail`.
- `docs/master/VERSION.md` y `.github/workflows/docs.yml`: hashes SHA-256 de
  los dos maestros recalculados.

### Fase 4.5 - Implementacion tecnica: ag-mail + ag-domains (2026-05-24)

Anadido:

- `crates/ag-mail` (estandar diferido): `MailSender` trait + `SmtpSender` (lettre + rustls).
  `InMemoryQueue` con reintentos y backoff exponencial.
  `StringTemplate` + `MailTemplate` trait. `NullSender` (feature `test-utils`). 38 tests.

- `crates/ag-domains` (opcional infra): `DnsProvider` trait + `CloudflareProvider` (reqwest).
  `apply_mail_records`: upsert idempotente de SPF/DKIM/DMARC. `PropagationChecker`
  con hickory-resolver. Soporte ACME completo via instant-acme + rcgen (DNS-01,
  renovacion automatica). 28 tests.

- `crates/ag-dsl` v0.7: tokens `mail`, `domain`, `template` en el lexer; nodos
  `MailBlock`, `DomainBlock`, `MailTemplateDef` en el AST; parsers y analisis
  semantico para los nuevos bloques. 151 tests.

- `crates/ag-auth` (feature `mail`): `AuthMailer` con `send_verification`,
  `send_password_reset`, `send_magic_link`. `AgAuth::with_mail()` builder. 37 tests.

- `crates/ag-cli`: subcomandos `ag domains check`, `ag domains sync`, `ag mail test`.

- `examples/auth-mail-demo`: ejemplo ejecutable de los tres flujos con `NullSender`.

- `tests/integration/tests/fase45_e2e.rs`: 7 tests E2E cross-module (total: 14).

### Fase 4.5 - Pendientes: ag-lsp v0.7 + DSL from->domain + ACME fix + docs (2026-05-24)

Anadido:

- `crates/ag-lsp/src/backend.rs`: hover y completions para los bloques DSL v0.7.
  `mail`, `domain`, `template` con ejemplos de codigo; propiedades `provider`,
  `from`, `subject`, `vars`, `dkim_selector`, `dmarc_policy`, `dmarc_rua`. 6 tests nuevos (15 total).

- `docs/manual/03-dominio-tls-correo.md`: guia completa "Configurar dominio, TLS
  y correo transaccional con Anti-Gravital". Seis secciones: declaracion DSL,
  DNS sync, verificacion de propagacion, ACME, correo transaccional, ag-auth+ag-mail.

Corregido:

- `crates/ag-dsl/src/semantic.rs`: warning cuando el hostname del campo `from` de
  un bloque `mail` no coincide con ningun `domain_name` declarado en los bloques
  `domain` del schema. 2 tests nuevos (153 total en ag-dsl).

- `crates/ag-domains/src/acme/renewal.rs`: bug de underflow en `spawn_renewal_task`.
  `check_interval - renew_threshold` panickea en debug mode si `renew_before_days >= 1`.
  Nuevo calculo: `sleep_secs = renew_before_days.max(1) * 86400`. 4 tests nuevos.

- `tools/vscode-anti-gravital/syntaxes/anti-gravital.tmLanguage.json`: keywords DSL v0.7
  anadidos al resaltado de sintaxis: `mail`, `domain`, `template`, `provider`, `from`,
  `subject`, `vars`, `dkim_selector`, `dmarc_policy`, `dmarc_rua`.

Cambiado:

- `docs/roadmap/STATUS.md`: Fase 4.5 marcada como completada con todos los criterios
  de salida marcados. Commits de referencia: PR #42 (implementacion) y PR #43 (pendientes).

- `README.md`: banner y tabla de calendario actualizados a "Fase 4.5 implementacion
  tecnica completa".

### Fase 4.5 - Integracion documental: ag-mail + ag-domains (2026-05-23)

Anadido:

- `docs/adr/0007-ag-mail-ag-domains.md`: ADR-0007 oficializa la Fase 4.5
  como fase aditiva entre la Fase 4 (completa) y la Fase 5 (pendiente).
  Introduce `ag-mail` (estandar diferido, outbound + adapters) y
  `ag-domains` (opcional infra, DNS + ACME + SPF/DKIM/DMARC). Alcance,
  restricciones, direccionalidad de dependencias (ag-auth -> ag-mail,
  ag-cloud -> ag-domains), consecuencias documentadas. Estado: Aprobado.

Cambiado:

- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`: insertada fila 4.5 en
  el resumen de fases entre Fase 4 y Fase 5; duracion total
  24-28 meses -> 25-30 meses; nueva seccion "Fase 4.5" completa con
  criterios de entrada, entregables, criterios de salida bloqueantes y
  riesgos. Hito v0.5 BETA permanece al final de la Fase 5.

- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`:
  - § 5.1: tabla del ecosistema actualizada a 17 crates con ag-mail
    (Estandar diferido) y ag-domains (Opcional infra). Parrafo de cierre
    explica la clasificacion "estandar diferido".
  - § 5.2: diagrama del ecosistema redibujado para incluir la columna
    "Estandar diferido" con ag-mail y la flecha ag-cloud -> ag-domains
    dentro de "Modulos opcionales".
  - § 5.3: sexta regla (ag-auth -> ag-mail, no ciclo) y septima regla
    (ag-cloud -> ag-domains, dependencia no rigida) anadidas.
  - § 5.4: estructura del monorepo incluye `crates/ag-mail/` y
    `crates/ag-domains/` con anotaciones de Fase 4.5.
  - § 7.2: tabla del DSL realineada. v0.5/v0.6 marcados "Fin Fase 4
    (entregado)"; v0.7 "Mail y dominios declarativos" (Fin Fase 4.5);
    v0.8 "Plugin hooks" (Fin Fase 9). Multi-tenancy y migracion de
    datos quedan diferidos para RFCs propios.
  - § 8.8 (nueva): especificacion completa de `ag-mail` (estandar
    diferido). Patron Native | Adapter, validacion build-time de
    templates, cola asincrona, integracion ag-auth.
  - § 8.9 (nueva): especificacion completa de `ag-domains` (opcional
    infra). Trait DnsProvider, ACME (instant-acme), cooperacion con
    ag-mail para SPF/DKIM/DMARC, verificacion de propagacion.
  - § 10: renombrado a "Subsistema de despliegue (ag-cloud + ag-domains)".
  - § 10.6 (nueva): integracion ag-cloud <- ag-domains. Flujo de seis
    pasos del deploy con dominio: validar control, configurar DNS,
    emitir/renovar TLS, asociar al target, materializar SPF/DKIM/DMARC,
    verificar propagacion.

- `docs/master/VERSION.md`: nuevos hashes SHA-256 de los dos maestros
  markdown. Blueprint PDF v4.0 registrado como deuda explicita pendiente
  de re-export a v4.1 (los maestros markdown gobiernan segun politica
  pre-existente).

Pendiente (deuda registrada):

- Re-export del Blueprint a `ANTI-GRAVITAL-Blueprint-v4.1.pdf` con los
  cambios de la Fase 4.5. Requiere herramienta de exportacion fuera del
  scope de esta rama documental.

### Fase 4 - Modulos estandar: implementacion tecnica completa (2026-05-23)

Anadido:

- `crates/ag-auth`: implementacion completa de autenticacion y autorizacion.
  JWT Ed25519 con firma/verificacion PEM, API keys con hash BLAKE3 y prefijo
  configurable, WebAuthn/Passkeys con CBOR (ciborium) y verificacion COSE
  (ES256 via p256, EdDSA via ed25519-dalek), OAuthClient para Google y GitHub
  con flujo PKCE, RefreshBlacklist en memoria con RwLock. 32 tests pasan.

- `crates/ag-realtime`: bus de eventos InProcess mas cliente NATS externo.
  EventBus broadcast en proceso, NatsExternalClient con async-nats 0.48,
  TLS en 3 niveles (CAs del sistema, CA custom, mTLS), JetStream con stream
  AG_EVENTS y publish con ACK, ws_handler Axum que conecta WebSocket al bus,
  sse_handler Axum que sirve stream SSE EventSource-compatible via BroadcastStream.
  AgRealtime::new es ahora async y retorna Result.

- `crates/ag-storage`: URLs firmadas HMAC-SHA256 y backend S3/MinIO.
  sign_url y verify_signed_url con comparacion en tiempo constante, token
  en formato {base64url_hmac}_{expires_at}, AgStore convertido de struct a
  enum (Native | S3), S3Store via object_store 0.11 con soporte MinIO
  via endpoint opcional.

- `crates/ag-cache`: cobertura de tests elevada a mas de 80% en todos los
  archivos. Tests adicionales para L1Cache incluyendo invalidacion por tags,
  expiracion, y operaciones concurrentes.

- `crates/ag-observe`: cobertura de tests elevada a mas de 80%. Tests para
  init del layer, metricas handler, y configuracion desde env vars.

- `tests/integration`: nuevo crate de tests E2E cross-module con 7 tests:
  6 tests unitarios por modulo (observe, auth JWT, auth API key, cache,
  realtime, storage) y 1 test E2E de 15 pasos que atraviesa ag-auth,
  ag-cache, ag-realtime, ag-storage y ag-observe en secuencia.

- `docs/rfc/RFC-0005-ag-cache-native-l2.md`: RFC para L2 cache nativo
  Anti-Gravital compatible con protocolo RESP2, sin dependencia de Redis
  como servicio externo. Estado: propuesto, pendiente de aprobacion.

Cambiado:

- `examples/realtime-chat`: actualizado para AgRealtime::new async y
  bus() que retorna Option<Arc<EventBus>>.

- READMEs de modulos actualizados de placeholder "Fase 0 - Vacio" a
  documentacion de uso real: ag-auth, ag-realtime, ag-cache, ag-observe,
  ag-storage.

- `docs/roadmap/STATUS.md`: Fase 4 marcada como completada con todos los
  entregables en [x].

### Benchmarks reales Fase 2 y correccion de routing (2026-05-21)

Corregido:

- `examples/todo-api/src/main.rs`: ruta parametrizada corregida de
  `"/todos/{id}"` (sintaxis Axum 0.8) a `"/todos/:id"` (sintaxis Axum 0.7,
  version en uso). El bug causaba que GET, PUT y DELETE /todos/:id devolvieran
  404 sin tocar la base de datos. Todas las mediciones previas de throughput
  para esas rutas eran invalidas (median velocidad de respuestas 404, no
  operaciones DB reales).

Documentado:

- `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`:
  primera medicion valida de CRUD con PostgreSQL real. Hardware: AMD Ryzen 5
  2500U, 4C/8T, PostgreSQL 18.4 nativo. Resultados: GET /todos/:id = 14 478
  req/s mediana (c=100); POST /todos = 8 934 req/s (c=50, synchronous_commit
  off); stack HTTP sin DB = 88 930 req/s. Los criterios de 40K req/s y p99
  <= 5 ms no se alcanzan en este hardware con configuracion estandar de
  PostgreSQL; el cuello de botella es el scheduler de OS con el modelo
  proceso-por-conexion de PG sobre 4 nucleos fisicos.

- `docs/roadmap/STATUS.md`: criterios 2.3 de benchmark y latencia actualizados
  a `[ ]` con numeros reales y nota explicativa del bug de routing invalidado.

### Verificacion y cierre tecnico de Fase 2 (2026-05-21)

Corregido:

- `examples/todo-api/Dockerfile`: base image actualizada de `rust:1.79-slim` a
  `rust:1.95-slim` para coincidir con `rust-toolchain.toml`. `rust-toolchain.toml`
  ahora se copia antes de `rustup target add` para que rustup instale el target
  MUSL contra el canal correcto (1.95.0). Build verificado: binario MUSL estatico
  5.3 MB, imagen `FROM scratch` 2.49 MB, `/health` 200 OK con PostgreSQL.

- `templates/rest/src/main.rs.tmpl`, `templates/realtime/src/main.rs.tmpl`,
  `templates/fullstack/src/main.rs.tmpl`: eliminado import no usado `ShieldConfig`.
  Los tres templates compilan sin warnings con la nueva version del binario `ag`.

- `deny.toml`: restaurado ignore de RUSTSEC-2023-0071 (`rsa` dep transitiva de
  `jsonwebtoken`). El advisory fue reactivado al actualizar jsonwebtoken a v10.
  Anti-Gravital usa exclusivamente EdDSA; el timing attack de RSA no aplica.

Anadido:

- `examples/todo-api/src/main.rs`: soporte de variable de entorno
  `DATABASE_MAX_CONNECTIONS` para ajustar el pool sin recompilar (por defecto 10).
  Con pool=50 el throughput de GET /todos/:id llega a 82K req/s en hardware
  informativo (Ryzen 5 2500U, Docker PostgreSQL).

Seguridad:

- `jsonwebtoken` actualizado de 9.3.1 a 10.4.0. Cierra CVE-2026-25537 /
  GHSA-h395-gr6q-cpjc (Type Confusion en validacion de claims nbf/exp que permite
  bypass de restricciones temporales). Feature `rust_crypto` anadida para activar
  el backend criptografico puro Rust requerido por v10 con default-features = false.
  Sin cambios en src/shield/auth.rs: la API publica de v10 es compatible con v9.

Documentacion:

- `docs/roadmap/STATUS.md` actualizado: criterios de Docker y binario marcados `[x]`,
  benchmarks actualizados con mediciones del 2026-05-21 (pool=50, Docker PostgreSQL).

### Alineacion documental de Fase 1

Cambiado:

- `docs/roadmap/STATUS.md` sincronizado con el estado real de Fase 1
  tras el cierre tecnico de los 11 PRs de RFC-0002 mas los dos
  hotfixes. Marcadas como `[x]` las casillas de `ag-core` operativo,
  HTTP/1.1+HTTP/2, logging estructurado, clippy sin warnings y
  ausencia de `unsafe`. Marcadas como `[/]` con explicacion las
  casillas de cobertura >=80%, CI verde en las cuatro plataformas y
  `cargo audit`, pendientes de validacion oficial. Las metricas
  duras (300K req/s, p99, idle, arranque), blog post y stars siguen
  `[ ]` por requerir hardware de referencia o eventos de comunidad.

### Hotfix de Fase 1 (segundo)

Cambiado:

- Generacion de rutas temporales en tests TLS (`tests/shield_full_pipeline.rs`,
  `tests/shield_tls.rs`, `src/shield/tls.rs::tests`) usa `AtomicUsize`
  + pid del proceso en lugar de `SystemTime::now().as_nanos()`. El
  timestamp en Windows tiene resolucion de ~15ms, lo que producia
  colisiones de archivo entre tests paralelos: dos tests escribian
  cert/key en el mismo path y uno sobreescribia al otro mid-test.
  Sintoma: 3 de 6 tests de `shield_full_pipeline` fallaban
  intermitentemente en `build (windows-x64)`. Verificado con
  `RUST_TEST_THREADS=16`.

### Hotfix de Fase 1

Cambiado:

- `shield::tls::build_acceptor` deja de depender del provider rustls
  global del proceso. Construye `ServerConfig` con
  `ServerConfig::builder_with_provider(ring::default_provider())` y
  `with_safe_default_protocol_versions()`. Cada `TlsAcceptor` lleva
  su propio provider via Arc. Cierra una race observada en CI
  macos-arm64 cuando varios tests E2E inicializaban TLS en paralelo:
  un test podia leer el provider global antes de que otro terminara
  de instalarlo. Los tests E2E ya no necesitan llamar
  `install_default()`; se eliminaron las llamadas en
  `tests/shield_full_pipeline.rs` y `tests/shield_tls.rs`.

### Fase 0 - Fundaciones y gobernanza

Anadido:

- Documentos maestros instalados en `docs/master/`:
  `ANTI-GRAVITAL-Blueprint-v4.0.pdf`,
  `ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
  `ANTI-GRAVITAL-Hoja-de-Ruta.md` y `VERSION.md` con hashes SHA-256.
- Constitucion tecnica del repositorio en `CLAUDE.md`.
- README bilingue espanol e ingles.
- Documentos de gobernanza: `CONTRIBUTING.md`, `GOVERNANCE.md`,
  `SECURITY.md`, `CODE_OF_CONDUCT.md`.
- Estructura de documentacion: `docs/architecture/`, `docs/roadmap/`,
  `docs/modules/`, `docs/dsl/`, `docs/benchmarks/`, `docs/security/`,
  `docs/governance/`, `docs/examples/`, `docs/rfc/`, `docs/adr/`,
  `docs/diagrams/`, `docs/graph/`, `docs/es/`, `docs/en/`.
- Descomposicion verbatim de los maestros en archivos navegables por
  capitulo, fase y modulo.
- Workspace Cargo con 15 crates vacios: `ag-core`, `ag-dsl`,
  `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`,
  `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`,
  `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- Configuracion de toolchain: `rust-toolchain.toml`, `rustfmt.toml`,
  `clippy.toml`, `deny.toml`.
- Workflows de CI multiplataforma: `ci.yml`, `quality.yml`, `docs.yml`.
- Plantillas de issue, pull request y RFC en `.github/`.
- ADRs iniciales: `0001-monorepo-workspace.md`,
  `0002-bilingual-documentation.md`, `0003-bdfl-governance.md`,
  `0004-descomposicion-de-maestros.md`,
  `0005-contact-identities.md`.
- Tablero vivo del proyecto en `docs/roadmap/STATUS.md`.
- Lista de entregables externos pendientes en
  `docs/governance/external-deliverables.md`.

Cambiado:

- Identidades de contacto oficiales del proyecto. Los placeholders
  `security@gravital.io` y `hello@antigravital.dev` de los maestros se
  reemplazan por `anti@gravitalcloud.com` (correo raiz) y
  `angelnereira@gravitalcloud.com` (BDFL inicial) en
  `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (15.3) y
  `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (Fase 0). Hashes
  recomputados en `docs/master/VERSION.md` con entrada de historial.
  Derivados verbatim regenerados. Registrado en
  `docs/adr/0005-contact-identities.md`.

Sin codigo funcional. El primer hito tecnico (Shield MVP) se entrega
en Fase 1.

### Fase 1 - The Shield MVP (en curso)

Anadido:

- RFC-0001 que autoriza la paralelizacion de las puertas externas de
  Fase 0 con la implementacion de Fase 1 mientras el BDFL trabaja en
  solitario.
- RFC-0002 con el diseno detallado del Shield MVP: stack, modulos,
  features Cargo, configuracion TOML, sistema de errores y plan de
  implementacion en 11 PRs incrementales.
- Estado vivo de Fase 1 reflejado en `docs/roadmap/STATUS.md`.
- Bootstrap del crate `ag-core` con HTTP/1.1 y HTTP/2 funcionales via
  Axum + Tokio (sin TLS aun): modulos `error`, `config`, `runtime`,
  `shield` (capa de logging estructurado), `core` (placeholder).
- `AgError` y `AgResult` con mapeo automatico a respuestas HTTP via
  `IntoResponse`.
- `ShieldConfig` deserializable desde TOML con defaults seguros.
- Dependencias compartidas del workspace declaradas en
  `[workspace.dependencies]` (axum, tokio, tower, tower-http, tracing,
  serde, thiserror, hyper, http, bytes, toml, pin-project-lite).
- Tests: 12 unit tests por modulo, 2 tests E2E con servidor real, 1
  doctest. Todos en verde con `cargo fmt`, `cargo clippy -D warnings`
  y `cargo doc --no-deps` limpios.
- Capa de validacion de payload (`shield::validation`) detras de la
  feature `validation` activa por defecto. Trait `Validate`, agregado
  `ValidationErrors` con `FieldError` serializable y extractor
  `ValidatedJson<T>` que mapea fallos a `AgError::Validation` con
  detalle estructurado por campo (status 422). 4 unit tests
  adicionales y 3 tests E2E sobre `/projects`.
- Capa CORS (`shield::cors`) detras de la feature `cors` activa por
  defecto. Wraps `tower_http::cors::CorsLayer` con configuracion
  declarativa via `CorsConfig` en `ShieldConfig`. Defaults seguros:
  CORS deshabilitado salvo declaracion explicita. Errores de
  configuracion mapeados a `AgError::Cors` con codigo `cors_error`
  (status 403). 4 unit tests sobre construccion y 4 tests E2E sobre
  preflight, origenes listados y rechazados.
- Tower-http feature `cors` activada en el workspace.
- Capa CSRF (`shield::csrf`) detras de la feature `csrf` activa por
  defecto. Patron double-submit cookie apatrida: en peticiones que
  mutan estado (POST, PUT, PATCH, DELETE) se exige que el header y la
  cookie configurados lleven el mismo valor opaco. Configuracion via
  `CsrfConfig` con header por defecto `x-csrf-token` y cookie
  `ag_csrf`. CSRF deshabilitado por defecto. 7 unit tests sobre
  parsing de cookies y validacion y 6 tests E2E sobre flujo completo.
- Capa rate-limit (`shield::rate_limit`) detras de la feature
  `rate-limit` activa por defecto. Token bucket por IP con `governor`
  (dashmap storage). Cuando una IP excede `per_ip_rps`/`burst`, las
  peticiones rebotan con `AgError::RateLimit` (status 429, codigo
  `rate_limit_exceeded`). Sin `ConnectInfo` la capa pasa transparente
  (compatibilidad con tests sin transporte). `RateLimitConfig`
  deshabilitada por defecto, configuracion validada al construir
  Shield. Dependencia opcional `governor = 0.7`. 6 unit tests y 3
  tests E2E.
- Capa de autenticacion JWT Ed25519 (`shield::auth`) detras de la
  feature `auth-jwt` activa por defecto. Verifica el header
  `Authorization: Bearer <token>` contra una clave publica Ed25519
  cargada al arranque desde `AuthConfig.public_key_pem` o
  `public_key_path`. Valida firma, expiracion, issuer opcional y
  audience opcional. Leeway forzado a 0 para evitar tolerancia
  silenciosa a deriva de reloj. Inyecta `AuthContext` en las
  extensiones del request; expone el extractor `Claims<T>` que
  deserializa los claims al tipo de la aplicacion. Fallos mapeados a
  `AgError::Auth` (status 401). Dependencia opcional
  `jsonwebtoken = 9` con feature `use_pem`. 6 unit tests sobre
  parsing y carga de claves y 6 tests E2E sobre flujo completo,
  incluyendo expiracion, firma incorrecta y issuer no esperado.
- Dev-dependencies `ed25519-dalek = 2` y `rand_core = 0.6` para
  generar pares de claves Ed25519 en tests.
- Capa TLS 1.3 (`shield::tls`) detras de la feature `tls` activa por
  defecto. Construye un `tokio_rustls::TlsAcceptor` desde cert/key
  PEM declarados en `TlsConfig`. Provider criptografico: `ring`.
  Cuando la feature `tls` esta presente y `TlsConfig.enabled` es
  cierto, `Shield::serve(listener, router)` opera el accept loop
  envolviendo cada conexion con TLS; en otro caso delega a
  `axum::serve`. Errores de carga mapeados a `AgError::Tls`.
  Dependencias opcionales `rustls = 0.23`, `rustls-pki-types = 1.10`,
  `tokio-rustls = 0.26`. Dev-dependency `rcgen = 0.13` para generar
  certificados auto-firmados en tests. 4 unit tests sobre carga de
  PEM y 3 tests E2E incluyendo handshake HTTPS real con reqwest.

Cambiado:

- Migracion de `rustls-pemfile` (archivado, RUSTSEC-2025-0134) al
  trait `PemObject` de `rustls-pki-types`. La superficie de API
  publica no se altera.
- Tests E2E del pipeline Shield completo en
  `crates/ag-core/tests/shield_full_pipeline.rs`. Arrancan un servidor
  con TODAS las capas activas simultaneamente (TLS, auth-jwt, csrf,
  cors, rate-limit, validation, logging) sobre HTTPS real con cert
  auto-firmado y par Ed25519 generados en setup. 6 tests cubren:
  request valido pasa por toda la pipeline (GET y POST), JWT invalido
  bloqueado por auth (401), CSRF ausente bloqueado en POST (403),
  payload invalido bloqueado por validation (422), origen no listado
  no recibe header allow-origin.
- Ejemplo binario release-ready
  `crates/ag-core/examples/hello_world.rs` para correr con
  `cargo run --release -p ag-core --example hello_world` y medir con
  `oha`, `wrk` o equivalente.
- Plantilla `docs/benchmarks/measurement-template.md` para registrar
  oficialmente las metricas duras de cierre de Fase 1 (throughput,
  p99, memoria idle, arranque) cumpliendo la regla 17. RFC-0002 PR 10
  de 11.
- Rustdoc enriquecido de `ag-core` con introduccion crate-level
  ampliada (tabla de capas, features, ejemplos de uso con
  `Shield::serve` y `ShieldConfig::from_path`, lista de tipos
  publicos clave y enlaces cruzados a maestro y manual). Dos
  doctests compilan en CI.
- Capitulo 1 del manual de usuario: `docs/manual/01-shield-as-library.md`
  con once secciones cubriendo instalacion como dependencia git,
  servidor minimo, carga de configuracion desde TOML, orden de
  capas, activacion de TLS / JWT / CSRF / rate-limit, extractor
  `Claims<T>` y `ValidatedJson<T>`, errores y observabilidad,
  recomendaciones de despliegue y referencias cruzadas. Cierra el
  ultimo entregable en-repo de Fase 1. RFC-0002 PR 11 de 11.

Cambiado:

- `Shield::serve` inyecta `ConnectInfo<SocketAddr>` en cada request
  tanto en transporte plano como TLS. En el camino plano se usa
  `Router::into_make_service_with_connect_info::<SocketAddr>()`; en
  el camino TLS se captura `peer_addr` antes del handshake y se
  inyecta como extension del request. Antes de este fix, la capa
  rate-limit pasaba transparente sobre cualquier servicio arrancado
  via `Shield::serve` porque la IP del cliente no llegaba al
  middleware. La firma publica de `Shield::serve` no cambia.

- Benchmark Hello World del Shield con criterion en
  `crates/ag-core/benches/shield_hello_world.rs`. Tres grupos
  comparables a nivel Tower: `bare_axum_hello` (linea base),
  `shield_default_hello` (Shield con solo logging) y
  `shield_full_default_hello` (Shield con CORS, CSRF y rate-limit
  activas). Documentado en `crates/ag-core/benches/README.md` con la
  regla 17 (hardware, OS, version Rust, commit, configuracion,
  metodologia, ejecuciones, desviacion estandar). Dev-dependency
  `criterion = 0.5`. RFC-0002 PR 9 de 11.

- Configuracion TOML completa del Shield: `ShieldConfig::from_path`
  carga la configuracion desde un archivo, `to_toml_string` permite
  round-trip estable, y todas las structs llevan
  `#[serde(deny_unknown_fields)]` para rechazar typos con
  `AgError::Config` en vez de ignorarlos silenciosamente. Ejemplo
  documentado en `crates/ag-core/config.example.toml` con todas las
  secciones (`bind`, `runtime`, `cors`, `csrf`, `rate_limit`, `auth`,
  `tls`) y sus defaults explicados. 14 unit tests sobre parsing por
  seccion, rechazo de claves desconocidas, round-trip y carga desde
  disco. RFC-0002 PR 8 de 11.

- Flujo de pull requests con autofill automatizado: cada rama trae su
  descriptor pre-rellenado bajo `docs/pr-drafts/<rama-aplanada>.md`
  (las `/` de la rama se convierten en `-`). El nuevo workflow
  `.github/workflows/pr-autofill.yml` se dispara al abrir o reabrir
  la pull request, busca el descriptor y reemplaza el cuerpo del PR
  con su contenido completo. Si no encuentra descriptor, comenta el
  PR avisando. La plantilla `.github/PULL_REQUEST_TEMPLATE.md` queda
  como aviso visible solo en ese caso. Regla incorporada a
  `CLAUDE.md` y `CONTRIBUTING.md`. Descriptor de la rama
  `phase-0/foundations-and-governance` publicado en
  `docs/pr-drafts/phase-0-foundations-and-governance.md`.

Cambiado:

- API publica de `Shield`: `Shield::layer()` reemplazado por
  `Shield::apply(router)`. La nueva firma oculta la complejidad de
  tipos de la pipeline y permite agregar capas sin romper la
  superficie publica en cada PR.
- `Shield::try_new(config)` valida la configuracion en construccion
  (origenes, metodos y headers de CORS); `Shield::new(config)` mantiene
  semantica de panic para casos de prototipado.
- Workflow `quality.yml`: `cargo deny` ya pasa tras anadir
  `Unicode-3.0` a `deny.toml` (commit anterior).

### MCP readiness review (RFC-0024 / ADR-0017)

Added (documentation only; no code):

- `docs/audits/mcp-readiness-review.md` maps every proposed MCP tool to the
  current CLI request/executor surface and records that the stage E substrate
  (`--output json`, `ag describe`, plan/apply, audit log), `Secret<T>` and
  `ag deploy` do not exist yet.
- The review proposes a versioned capability manifest schema, the per-tool
  security checks (capability gate, plan id, apply, redaction, audit record),
  self-debug boundaries, a prioritized implementation checklist and test cases
  including refusal of an agent without the required capability.
- Contract risks for arbitration: the two conflicting plan models in RFC-0024,
  the `deploy.*` tools versus Phase 5, and ownership of the shared versioned
  schema under ADR-0016. `docs/INDEX.md` links the new audit.

[Unreleased]: https://github.com/anti-gravital/anti-gravital/compare/HEAD..HEAD

### Data-layer design note (issue #154)

Added (documentation only; no code and no phase change):

- `docs/modules/ag-data/data-layer-rls-tenancy.md` records the design for the
  Phase 5+ typed ORM, row-level security and multi-tenancy: shared `TenantId`
  and explicit `TenantContext` propagation, fail-closed RLS policies with a
  transaction-local GUC set through `AgTx`, generated ORM layout under RFC-0019,
  expand/contract migrations and the sqlx 0.9 feature matrix.
- `docs/modules/ag-data/README.md` points to the design note. The design
  remains non-normative until the architect's RFC is accepted; it introduces no
  implementation and does not advance the roadmap state.
- The note records the data-model constraints for issue #185: the DSL
  `Timestamp` mapping is a hidden chrono dependency, sqlx 0.9 supports `time`
  but not `jiff`, and serialized date/time formats need golden-string tests.
  It includes the measured result that `time::OffsetDateTime` needs an explicit
  `serde` adapter to keep RFC 3339 strings, and that trailing-zero fractions
  still differ from chrono's output.

### Proposal: chumsky parser upgrade evaluation (RFC-0027, issue #180)

Added (documentation only; no code or dependency change):

- `RFC-0027` amends RFC-0003 with the evaluation of `chumsky` 0.9.3 versus the
  maintained 0.13+/1.0-line (current stable 0.14.2). It records the API and
  error-model differences, a compiled local API spike, a local baseline timing
  with full Rule 17 metadata, the upstream benchmark as context, and the
  impact on diagnostics, fuzzing and benchmarks.
- Recommendation: defer the migration until the DSL v1.0 grammar freeze or a
  concrete blocker (advisory, unfixable DX defect or measured bottleneck). No
  parser code, manifests or generated artifacts were changed by this proposal.

### Managed email API DX design (issue #152)

Added (documentation only; no code):

- `docs/modules/ag-mail/managed-api-dx-design.md` fixes the precise managed
  REST surface before implementation: versioned endpoints and marketing
  objects, request/response shapes, an RFC 9457 problem+json error model,
  tenant-scoped idempotency, capability-scoped agent access per RFC-0024,
  the `ag new -> ag dev` ergonomics guarantee, and the exact gate condition
  to start. Mail C/D remain design-only; the RFC-0009 Mail A deliverability
  gate has no external evidence yet and RFC-0024 needs a mail/marketing
  tool-group amendment. The module sheet now indexes the design.

### Blocked-issue evidence and tracking reconciliation (issues #87, #179, #142, #76)

Added (documentation only; no code):

- `docs/ag-domains/how-to/acme-staging-test.md` defines the reproducible
  credentialed Let's Encrypt staging run for the `#[ignore]` test of #87:
  prerequisites, environment variables, exact commands, SNI check, teardown
  and an evidence template. The test does not exist yet; both it and #179 stay
  open with the `blocked` label until real before/after evidence exists.
- `docs/ag-domains/reference/instant-acme-0.8-migration.md` records the nine
  API migration points for the blocked 0.8 port, with the eight compiler errors
  reproduced locally on 0.8.0 and 0.8.5 and the exact current source locations.
- Issue #142 and #76 child checklists were reconciled against the live issue
  board; #142's status line now names all six children that remain open.

### Infrastructure review of issues #112, #149 and #141

Added (review document only; no code, features or dependency changes):

- `docs/audits/infrastructure-review-112-149-141.md` records the edge-to-workers
  producer contract (acyclic direction, native default and the marker-feature
  caveat), the evidence that the protoc premise for OTLP is stale at the pinned
  dependency versions, a protoc-free path to re-enable the exporter, and an
  installer trust-anchor design. It includes verification criteria and an
  ADR-0009/ADR-0011 compliance check.
- `docs/INDEX.md` now lists the audits directory and this review. No issue is
  closed by this change: #112 stays blocked on a concrete consumer, #149 awaits
  the opt-in exporter wiring and CI evidence, and #141 awaits a release
  publication and signature path.

[Unreleased]: https://github.com/anti-gravital/anti-gravital/compare/HEAD..HEAD

### Low-level reviews (issues #178, #180, #181)

Added:

- `docs/audits/low-level-review-178-180.md` records the read-only review of
  the mail protocol stack upgrade, the DKIM and rustls provider semantics,
  the #181 process-level ring fix, and the chumsky 0.9 to 0.13 RFC inputs.
  It documents the required `mail-auth` 0.15 feature set, the
  `mail-send/dkim` version conflict, the DKIM independent-verifier gap, the
  `allow_invalid_certs` connector trap in `mail-send` 0.7, and the missing
  DSL benchmark and fuzz baseline. No issue was closed by this review.

### Installer integrity trust-anchor hook (issue #141)

Added:

- `install.sh` and `install.ps1` verify their own SHA-256 against an externally
  supplied manifest when `AG_INSTALLER_CHECKSUMS` is set, and abort before any
  build or install action on a missing file, a missing entry or a mismatch.
  Unset, they warn and keep the previous behavior; no in-repository file is
  treated as a trust anchor.
- `tools/check-installers.sh` honors the same variable and fails closed when the
  external manifest is unavailable, keeping the repository-local list as a
  documented development fallback. It now tests the fail-closed paths.

Known limitation: there is still no release key or out-of-band publication
channel, so this is a trust-boundary hook rather than a trust anchor. Issue #141
remains open; see `docs/security/INSTALLATION_INTEGRITY.md`.

### Verification packages for blocked gates (issues #156, #157)

Added:

- `docs/fuzz/24h-gate-verification.md`: exact commands, environment, artifact
  paths and empty results template for the 24-hour gate over all seven fuzz
  targets. The gate stays blocked; no 24-hour result is claimed.
- `docs/fuzz/results/2026-11-22-smoke.md`: real 30-second-per-target smoke of
  the seven harnesses (6,659,636 executions, exit 0, no crashes) on informative
  hardware, explicitly not extrapolated to the 24-hour gate.
- `docs/benchmarks/verification-package.md`: reproducible Stage 6 procedure,
  metadata commands, artifact paths and rule-17 results template.
- `docs/benchmarks/smoke-2026-11-22.md`: real reduced-sample criterion output
  for `ag-core` and `ag-workers`, labelled as harness smoke, not Stage 6
  evidence.

Changed:

- `fuzz/Cargo.lock` regenerated to match `fuzz/Cargo.toml` (the checked-in lock
  omitted current dependencies such as `ag-workers`), so fuzz runs start from
  a clean tree.
- Release gate `Fuzz 24h` and `Benchmarks` rows now point to the verification
  packages; both remain `pending`.

### OTLP trace exporter for ag-observe (issue #149)

Added:

- New opt-in Cargo feature `ag-observe/otlp` (off by default) exports traces
  over OTLP/HTTP with binary protobuf. When enabled and
  `OTEL_EXPORTER_OTLP_ENDPOINT` is set, `init()` installs a
  `tracing-opentelemetry` layer backed by a thread-based batch exporter.
- `SERVICE_NAME` now sets the `service.name` resource attribute for OTLP
  (default `ag-service`). The native logging and Prometheus exporters remain
  the default and require no collector.
- `ADR-0019` records why the HTTP/protobuf transport is used: the pinned
  `opentelemetry-otlp`/`opentelemetry-proto` versions ship pre-generated code,
  so no `protoc` is needed in multiplatform CI, and the blocking client needs
  no Tokio runtime. The stale `TECH-DEBT` markers in the workspace and
  `ag-observe` manifests were removed.

### reqwest 0.13 with the ring provider (issue #177)

Changed:

- `reqwest` upgraded from 0.12 to 0.13 in the workspace and in the native
  benchmark. The removed `rustls-tls` feature was replaced by
  `rustls-no-provider`, plus `form` and `query` for the existing call sites.
  The workspace keeps `ring` as its single crypto provider (ADR-0018).
- Every composition point that builds a `reqwest` client now installs `ring`
  as the process-default rustls provider with an idempotent pattern: ag-auth
  OAuth, the ag-domains DNS adapters, the examples, the benchmark and the
  affected tests. This preserves TLS verification and avoids the two-provider
  failure already fixed for the native MTA (issue #181).
- Added a test that performs a verified TLS handshake through `reqwest`
  against a local rustls server, so a missing or wrong provider fails CI.
- The standalone native benchmark manifest now declares its `rand` dependency
  explicitly instead of trying to inherit a workspace that does not exist, so
  its upgraded `reqwest` can actually be compiled and its lockfile refreshed.

### Mail protocol stack upgrade (issue #178, partial)

Changed:

- `ag-mail/mta` now uses `mail-send` 0.7, `mail-auth` 0.15 and
  `mail-parser` 1.0. `mail-auth` 0.15 removed the `ring` feature, so the
  crate now selects the pure-Rust `rust-crypto` backend plus `dns-hickory`;
  the workspace stays on ring for TLS and the process-level provider
  installation from issue #181 is unchanged. `mail-send/dkim` stays off.
- DKIM signing pins relaxed/relaxed canonicalization explicitly and the
  mail-parser 1.0 MIME API is used for DSN/ARF intake. The loopback
  ESMTP/STARTTLS/DKIM sink tests and a cross-version known-answer test (a
  signature from mail-auth 0.9 verified by the 0.15 verifier) pass.

Open (issue #178 stays open with `Refs`):

- Independent verifier: `mailrs-dkim` 3.2 hashes signed headers in `h=`
  order while mail-auth and deployed MTAs hash in message order, so it
  rejects signatures from both the old and the new stack. No dkimpy or
  OpenDKIM is available locally, and the GPL-licensed alternative would
  fail `cargo deny`.
- Real mailbox / Mail A gate: outbound port 25 is blocked, so the live MX
  test and Authentication-Results check remain manual.

### Design proposal: oauth2 replacement, unblock for chrono removal (issue #185)

Added (documentation only; no code, no manifest change, no phase change):

- `RFC-0029` evaluates concrete replacements for `oauth2 5.0.0`, whose
  non-optional chrono dependency blocks issue #185 and RFC-0026. Evidence:
  crates.io still reports 5.0.0 as newest; upstream 5.1.0 on `main` also
  depends on chrono; PR #355 is open and unmerged because chrono is in the
  public API.
- The RFC recommends a small native Authorization Code + PKCE client in
  `ag-auth` using dependencies already present (base64ct, sha2, getrandom,
  url, reqwest). It records the token endpoint, PKCE S256, CSRF state and
  refresh compatibility contract, plus golden tests and rollback.
- The RFC index gains the missing RFC-0026 row and the new RFC-0029 row.
- Issue #185 remains open and is referenced with `Refs #185`; the deny ignore
  is unchanged until every chrono edge is gone.

### Independent DKIM verifier for ag-mail (issue #178)

Added:

- `tools/dkim_verify.py` and an ignored `ag-mail` test that verify both the
  recorded 0.9 signature vector and a freshly produced 0.15 signature with
  dkimpy, an implementation outside the `mail-auth` lineage. No Cargo
  dependency was added; dkimpy (BSD-like) and PyNaCl (Apache-2.0) are
  on-demand test tooling.
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` with the exact install and
  verification commands, recorded `dkim=pass` output, the vector locations
  and the reason the cross-version test alone is not independent evidence.
  The manual real-mailbox gate of #178 remains blocked (port 25), so the
  issue stays open.

### Date/time migration: chrono to time (issue #185, RFC-0026)

Changed:

- Workspace `sqlx` now enables the `time` feature instead of `chrono`.
  `ag-workers`, `ag-domains` and `examples/workers-scheduled` use
  `time::OffsetDateTime` instead of `chrono::DateTime<Utc>`. Arithmetic uses
  `time::SignedDuration` (the name in time 0.3.55 for the RFC's `TimeDelta`).
  ACME ARI and `notAfter` parsing, Route 53 SigV4 formatting and Cloud DNS JWT
  timestamps are covered. JWT claim types are unchanged: they are `u64` seconds.
- The DSL maps `Timestamp` to `time::OffsetDateTime` and emits
  `#[serde(with = "time::serde::rfc3339")]` (or the `option` module for nullable
  fields). Generated projects no longer require chrono. The ecommerce generated
  fixture and the DSL reference were updated.
- Golden serde tests cover whole seconds, nine-digit fractions, trailing-zero
  fractions, `Z` spelling, non-UTC normalization and nullable fields. An
  `#[ignore]` PostgreSQL test documents the TIMESTAMPTZ microsecond boundary.

Breaking changes (documented, not claimed compatible):

- Fractional seconds with trailing zeros now serialize without them:
  `2025-01-02T04:00:00.100Z` becomes `2025-01-02T04:00:00.1Z`.
- Explicit UTC formatting now spells `Z` instead of chrono's `+00:00`.

Status:

- `deny.toml` deliberately KEEPS the RUSTSEC-2026-0543 ignore in this branch:
  oauth2 5.0.0 still reaches chrono through `ag-auth` and is being replaced
  separately. The coordinator must remove the ignore after both branches land
  and verify `cargo tree -i chrono` is empty. Refs #185; the issue stays open.

### Issue board reconciliation (wave 4)

Added (documentation and issue-board only; no code, no phase change):

- `docs/audits/issue-board-reconciliation.md` records the matrix of the 13
  issues that remained open after wave 3: seven blocked by an external gate
  (#87, #141, #156, #157, #178, #179, #185), four deferred by an accepted
  decision (#112, #152, #154, #180) and two tracking parents (#76, #142).
  Each row gives the governing decision, the exact unblock condition and the
  `Closes #NNN` keyword the coordinator must use only after verification.
- Short status comments posted on all 13 issues; `blocked` added to #141,
  the only externally gated issue missing it. Parent bodies #76 and #142
  updated with verified child counts (16 children / 15 CLOSED for #76;
  31 children / 26 CLOSED for #142; #149 is now CLOSED).
- No issue was closed: every gate is still real and unresolved.

### Declared environment and `Secret<T>` in ag-core (RFC-0020 stage B)

Added:

- `ag_core::secret::Secret<T>` (also re-exported at the crate root). `Debug`
  and `Display` print `[redacted]`, there is no `Serialize` implementation
  (enforced by a `compile_fail` doctest), the inner value is overwritten on
  drop through `zeroize::Zeroize`, and the value is readable only via an
  explicit `expose() -> &T`.
- `ag_core::env` with `EnvDeclaration`, `EnvLoader`, `EnvType`, `LoadedEnv`,
  `EnvValue` and `EnvValidationError`. The loader reads only declared keys
  from the process environment (non-UTF-8 values are reported, not panics),
  falls back to a declared default, and validates
  `string | int | bool | url | duration | path` before returning an immutable
  `LoadedEnv`. All problems are collected into one error; messages contain
  the name, expected type and sources checked but never the value. Secret
  values are delivered as `Secret<String>` with their exact source bytes.
  `EnvValidationError` converts into `AgError::Config` so a composition root
  can fail before a listener opens.
- Explicit workspace dependencies on `zeroize` (secret clearing) and `url`
  (URL validation). Both were already resolved in `Cargo.lock`, so no new
  transitive graph is introduced.

Changed:

- `ag-core` crate documentation now describes the secret and declared
  environment API, its startup contract and the additive migration policy.

Tests:

- Redaction of `Debug`/`Display` (including alternate and container
  formatting), compile-fail serialization, explicit `expose`, and a probe
  that observes `Zeroize::zeroize` being called on drop.
- Declared-key isolation, all six type parsers and duration suffixes,
  multi-problem fail-fast reporting with no values in the message, optional
  and default handling, redaction of loaded secrets, process-environment
  loading and error conversion.

Scope: stage B only. The `[env]` declaration in `ag.toml`, the encrypted
secrets file, the `ag env` commands and module migration off direct
`std::env::var` calls remain later stages of RFC-0020. No existing call site
was changed.

### Alpha installer channel (wave 5)

Added:

- `install.sh` and `install.ps1` accept `AG_INSTALLER_CHANNEL=alpha`: download a
  prebuilt `ag` binary plus `checksums.txt` from a configurable base URL or
  GitHub release tag, verify the SHA-256 checksum before installing, and fail
  closed on a missing entry, a download error or a mismatch. No install to a
  privileged location; the default destination is `~/.cargo/bin`.
- The default `source` channel is unchanged; the development-only local
  fallback is now labelled explicitly when `AG_INSTALLER_CHECKSUMS` is unset.
- `AG_INSTALLER_BINARY_CHECKSUMS` supplies a local, out-of-band binary manifest;
  when set, `checksums.txt` is not downloaded from the release channel.
- `tools/test-alpha-installer.sh` exercises the real `file://` download and
  verify path plus the fail-closed cases and the out-of-band manifest path;
  `tools/check-installers.sh` runs it.
- `docs/security/INSTALLATION_INTEGRITY.md` documents the alpha channel inputs,
  verification order and its limits.

The alpha channel verifies checksums only. No signing key exists and no
signature is claimed; the supplied release location is the trust boundary.
Issue #141 stays open; RFC-0030 remains the signed-manifest design.

### Evidencia de aceptacion alpha (smoke offline)

Anadido:

- `scripts/release-smoke.sh`: prueba end-to-end reproducible y sin red que
  compila el workspace con `--locked --offline`, genera un proyecto con
  `ag new ... --no-interactive`, compila un esquema DSL con `ag generate`,
  resuelve el lockfile y ejecuta `cargo check`/`cargo build` del proyecto
  generado, inicia `ag dev` en loopback con timeout y comprueba `GET /` con
  curl. Imprime PASS/FAIL con codigo de salida por paso, usa un directorio
  temporal y limpia el proceso y los archivos al terminar.
- `docs/audits/alpha-smoke-evidence.md`: salida real de la ejecucion con SO,
  version de Rust, commit, fecha y comando, ademas del inventario de criterios
  de aceptacion cubiertos y de los que requieren infraestructura externa.

Verificado:

- Ejecucion real: 10 pasos PASS, 0 FAIL, codigo de salida 0. Sin servicios
  externos y con el trafico de red de cargo desactivado.

Notas:

- No se inventa ningun resultado. Un intento con directorio de compilacion
  nuevo fallo por cuota de disco (`os error 122`); el script informo FAIL con
  codigo 101 y ese intento no se usa como evidencia. El defecto de bind
  (`0.0.0.0` ignorando `BIND`) se registra en la revision de seguridad de esta
  ola; no se abre un issue duplicado.

### Alpha release security review (wave 5)

Added:

- `docs/audits/alpha-release-security-review.md`: read-only audit of the alpha
  release path and generated prototypes. Installer downloads are verified
  before execution, no signing is claimed without a key, `--output json` keeps
  the secret canary clean (12/12 tests), and workflows SHA-pin actions with
  least privilege. Two defects are recorded: generated templates bind
  `0.0.0.0` despite the documented loopback default (#191), and the release
  checksum manifest format does not match what the alpha installer parses
  (#192). Four observations cover same-channel manifests, tag interpolation,
  docs drift and the scaffold's pinned git dependency.

### Alpha prototype walkthrough and secure scaffold binds (issue #191)

Added:

- `docs/examples/alpha-prototype.md`: a copy-paste guide from zero to a running
  local API and a generated TypeScript SDK, with no PostgreSQL, NATS, Redis or
  SMTP. Lists which templates and examples need external infrastructure, known
  alpha limitations, and the exact verification evidence. Linked from the
  examples catalog and `docs/INDEX.md`.
- `crates/ag-cli/tests/alpha_prototype.rs`: four offline-path regression tests
  covering the REST and layered scaffolds, the fullstack database boundary, and
  the ten-artifact `ag generate` output.

Fixed:

- Generated `rest`, `realtime`, `fullstack` and `layered` projects now bind
  `127.0.0.1:8080` by default and honour the `BIND` environment variable that
  `ag dev` sets. Their generated `config.toml` also defaults to
  `host = "127.0.0.1"`. Nothing generated hardcodes `0.0.0.0` any more, so a
  tester following the quickstart does not expose a development server on every
  interface (issue #191). Explicit `ag dev --bind 0.0.0.0:8080` still opts in
  to all interfaces.

Note: the generated `config.toml` `[bind]` section is still informational in
the alpha - the running listener is controlled by `BIND` or the loopback
default - but its value now matches the safe runtime default.
