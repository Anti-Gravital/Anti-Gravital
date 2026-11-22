# Infrastructure review: issues #112, #149 and #141

- Role: infrastructure reviewer (read-only review; no code or dependency changes).
- Date: 2026-11-21.
- Scope: the edge-to-workers producer contract (#112), the OTLP exporter and
  protoc trade-off (#149), and the installer trust anchor (#141).
- Governing documents: `CLAUDE.md` rules 5, 8, 12, 13, 16, 19, 22, 26, 30 and 36;
  `ADR-0009` rules 2, 4 and 5; `ADR-0011`; `ADR-0012`; `RFC-0012` sections 3.3,
  4, 5 (S6) and 7.
- Status: review complete for all three issues. This document closes no issue and
  changes no product capability. It records findings for the primary owners and
  the coordinator.

## 1. Method and evidence

Static reads of the affected manifests, source trees, workflows and audit records,
plus the following commands executed in the reviewer worktree with the shared
Cargo target directory:

- `cargo tree -p ag-workers --no-default-features --features producer -e normal
  --offline`: no `ag-data`, `sqlx`, `metrics`, NATS or Redis edge.
- `cargo check --locked --offline -p ag-workers --no-default-features --features
  producer`: finished successfully.
- `cargo tree -p ag-edge -e features --offline`: `ag-edge` depends on
  `ag-domains` only; no `ag-workers` edge.
- `grep -R` over `.github/workflows/` and `tools/`: no dependency-cycle check
  exists; the only acyclicity evidence is the one-off static audit in
  `docs/audits/pre-fase5-architecture.md`.
- Registry source inspection of `opentelemetry-otlp` 0.33.1,
  `opentelemetry-proto` 0.33.1 and `opentelemetry-http` 0.33.1, plus a scratch
  crate that depends on `opentelemetry-otlp = "=0.33.1"` with
  `default-features = false, features = ["http-proto", "reqwest-client", "trace"]`
  and `PROTOC=/nonexistent/protoc`: `cargo check --offline` succeeded.
- Read-only review of the `community/seguridad` commit for #141 (installers,
  integrity guide, `SECURITY.md`, `tools/check-installers.sh`).

## 2. Edge-to-workers producer contract (issue #112)

### 2.1 Facts confirmed; deferral endorsed

- `ag-edge` has no dependency on `ag-workers` in its manifest or source. Its only
  `tokio::spawn` is the HTTP listener in `crates/ag-edge/src/server.rs:268`.
- `crates/ag-edge/README.md` states that certificate issuance and attachment
  state belong to `ag-domains`; the edge is a data-plane consumer of bindings and
  certificate references. ACME renewal is implemented in the control plane
  (`crates/ag-domains/src/acme/renewal.rs`, `spawn_renewal_task` at line 104).
- `crates/ag-workers/README.md:59-61` and `docs/roadmap/STATUS.md` already record
  the dedicated wiring as deferred until a concrete consumer exists.
- `RFC-0012` section 4 allows `ag-edge -> ag-workers` (producer-only) and
  forbids the reverse direction. That direction is acyclic and respects
  `ag-core` independence and the infrastructure crate boundaries.

There is no concrete consumer for durable enqueue from `ag-edge` today. Adding
the dependency now would be speculative coupling and would violate `CLAUDE.md`
rules 5, 8 and 30. The DESIGN_ONLY/BLOCKED decision by the primary owner is
correct; issue #112 must stay open until a consumer is filed in its own issue.

### 2.2 Corrections to the future wiring plan

The future plan is directionally correct, but five details need correction before
it is treated as a contract:

1. **`producer` is a marker feature, not a compile-time guard.** There is no
   `#[cfg(feature = "producer")]` anywhere in `crates/ag-workers/src/`; only
   `metrics` and `postgres` gate code. The producer-only tree compiles with no
   external-service dependency, but `WorkerPool`, the supervisor, the scheduler
   and the in-memory backend are still compiled into the linked crate. The
   accurate claim is behavioural: the adapter calls only the enqueue API and
   spawns no worker tasks. Do not write "no worker runtime pulled in" as a
   compile/link fact in the issue or the README.
2. **The example proves runtime mode, not minimal feature linking.**
   `examples/workers-producer-edge/Cargo.toml` links `ag-workers` with default
   features; the producer-only behaviour there comes from
   `WorkerMode::Producer`. Either tighten that manifest to
   `default-features = false, features = ["producer"]` (this review verified the
   crate compiles with exactly that set) or state the distinction explicitly in
   the example README.
3. **Feature-tree verification must cover both states.** If the future
   `ag-workers` dependency in `ag-edge` is optional (recommended, so the native
   default stays free of it), `cargo tree -p ag-edge -e features` with default
   features shows no `ag-workers` at all. Verify the default tree (absent) and
   the enabled tree (`--features <name>`, only `producer` plus its transitive
   dependencies). The single command in the issue's verification section is
   incomplete.
4. **There is no CI dependency-cycle check to keep green.** No workflow or tool
   implements one; acyclicity was validated once by the static audit in
   `docs/audits/pre-fase5-architecture.md`. When #112 is implemented, either add
   a small manifest cycle guard to CI or amend the issue verification to the
   manual `cargo tree` check plus that audit record. A verification criterion
   that names a non-existent check is misleading.
5. **Test shape.** Use an always-running in-memory test (enqueue on one side,
   separate lease and run on the other) and an `#[ignore]` PostgreSQL
   cross-process test gated by `DATABASE_URL`, following the existing
   `ag-workers` integration-test precedent. The default test path must need no
   external service (`ADR-0009` rule 2).

### 2.3 Verification criteria for a future implementation

- `cargo tree -p ag-edge -e features` on default features: no `ag-workers`.
- The same command with the producer feature enabled: `ag-workers` present with
  only the `producer` feature requested; no `postgres`, `metrics` or durable
  backend feature.
- `cargo check -p ag-edge --no-default-features --features <producer-feature>`
  and `cargo clippy -p ag-edge --all-features -- -D warnings` clean.
- Integration test: producer enqueues a typed job, a separate lease/run path
  consumes it; the producer path contains no worker loop or spawn.
- Dependency direction asserted: no `ag-edge` mention in any `ag-workers`
  manifest dependency; cycle check (CI guard or documented manual check) green.
- Docs: `crates/ag-edge/README.md`, `crates/ag-workers/README.md`, roadmap S6
  and `docs/roadmap/STATUS.md` updated in the same PR as the code.

### 2.4 ADR compliance

`ADR-0009` rules 2 and 5 pass: the native default needs no external service and
a durable backend remains opt-in through `ag-data`/`postgres`. `ADR-0011` passes:
`ag-edge`, `ag-workers` and `producer` are project-owned or neutral names; no
third-party commercial brand is used as a component name.

## 3. OTLP exporter and protoc (issue #149)

### 3.1 The protoc premise is stale at the pinned versions

- `Cargo.lock` pins `opentelemetry` 0.33.1, `opentelemetry_sdk` 0.33.1,
  `tracing-opentelemetry` 0.34 and `tonic` 0.14.9. No `protoc`, `prost-build`,
  `tonic-prost-build` or `protobuf-src` crate appears in the lockfile.
- `opentelemetry-proto` 0.33.1 ships generated Rust sources under
  `src/proto/tonic/*.rs` and has no build script. `tonic-prost-build` is only a
  dev-dependency used by that crate's own `grpc_build` test; it is not pulled
  into a dependent build.
- `opentelemetry-otlp` 0.33.1 default features are `http-proto`,
  `reqwest-blocking-client`, `trace`, `metrics`, `logs` and `internal-logs`;
  `grpc-tonic` is opt-in. Both HTTP feature sets enable
  `opentelemetry-proto/gen-tonic-messages`, not `gen-tonic`, so they do not
  trigger tonic code generation either.
- The scratch crate compiled with `PROTOC=/nonexistent/protoc`. Making system
  `protoc` mandatory on Windows (or anywhere else) is therefore unnecessary at
  the current versions. A future upgrade that reintroduces build-time codegen
  should be handled by pinning a release with pre-generated code or by keeping
  the HTTP path, not by adding a system prerequisite.

### 3.2 Recommended re-enable path (design only; dev5 owns implementation)

- Add an opt-in Cargo feature `otlp` to `ag-observe`, with
  `opentelemetry-otlp` optional in both the workspace manifest and the crate,
  declared with `default-features = false`.
- Prefer OTLP/HTTP (`http-proto`) over gRPC (`grpc-tonic`): it avoids protoc and
  tonic entirely and has the smaller dependency surface on every CI platform.
- Transport trade-off: `opentelemetry-otlp` 0.33.1 requires `reqwest` 0.13.1,
  while this workspace pins `reqwest` 0.12. Enabling either `reqwest-client` or
  `reqwest-blocking-client` adds a second major version of `reqwest` and must be
  coordinated with issue #177 (single `ring` crypto provider, no provider panic
  as fixed in #181). The `hyper-client` feature reuses the workspace `hyper` 1.4
  and avoids the second `reqwest`; HTTPS endpoints then need explicit TLS
  wiring. Decide this with the owner of #177 before implementing.
- Preserve the native default: with `otlp` off, `init()` behaves exactly as
  today. Keep the fail-loud `ObserveError::OtlpSetup` when
  `OTEL_EXPORTER_OTLP_ENDPOINT` is set but the feature is off, so configuration
  is never silently dropped. With the feature on, build the exporter inside the
  caller's Tokio runtime and document that requirement; provide a constructor
  that accepts a pre-built exporter so runtime-dependent tests can use
  `#[tokio::test]` without hidden runtime creation.
- Remove both `TECH-DEBT` markers (in `crates/ag-observe/Cargo.toml` and the
  workspace `Cargo.toml`) only in the PR that actually wires the exporter, and
  update `crates/ag-observe/README.md` and the environment-variable table in
  the crate docs in the same PR.

### 3.3 Verification criteria

- `cargo build --locked -p ag-observe --features otlp` and the same command with
  `--target x86_64-pc-windows-msvc` succeed on a runner without `protoc`.
- `cargo doc --no-deps -p ag-observe --features otlp` succeeds.
- `cargo clippy --locked -p ag-observe --all-features -- -D warnings` clean.
- Default-feature `cargo test -p ag-observe` unchanged; a feature-on test proves
  that a configured endpoint produces a live exporter and that an endpoint set
  with the feature off still returns `OtlpSetup`.
- Both `TECH-DEBT` markers absent; CI matrix includes at least Linux and Windows
  builds of the opt-in feature.

### 3.4 ADR compliance

`ADR-0009` rules 2 and 5 pass: the collector is opt-in via a Cargo feature and
nothing external is required in the default build. `ADR-0011` passes: `OTLP` and
`OpenTelemetry` are protocol and open-source names, acceptable as transport or
feature labels; do not name an Anti-Gravital component after a vendor.

## 4. Installer trust anchor (issue #141)

### 4.1 Review of the partial implementation (read-only)

The `community/seguridad` commit defining `AG_INSTALLER_CHECKSUMS` is a sound
interface. Both installers hash their own bytes and abort before any Cargo
action when the variable is set and the manifest is missing, incomplete or
mismatched; unset means an explicitly documented development fallback, and the
installers never fetch the manifest. This is the correct boundary: the script
never claims to be its own trust anchor, and the documented limitations are
honest. A self-hash proves only that the script matches a supplied manifest;
trust still lives in the manifest's origin. Fail-closed parsing (including the
`sha256sum -b` form with a `*` before the filename, which will simply not match
and abort) should be kept; do not relax it for convenience.

### 4.2 Design needed to actually close the issue

- Add a release workflow triggered by release tags. It should regenerate the
  SHA-256 manifest from the exact tagged tree, sign it with an offline key, and
  upload the manifest and its signature as release assets.
- Sign the manifest with `minisign` (an open-source tool name, covered by the
  `ADR-0011` exemption) and publish the public key and its fingerprint in
  `SECURITY.md`. Repeat the fingerprint in a channel not controlled by the same
  repository (for example a maintainer key or a project DNS record), so a
  repository compromise alone cannot substitute the key.
- A GitHub Release asset is a different delivery path but not a different trust
  domain. The signature plus an independently published key, not the asset
  location, is what closes the trust gap.
- Keep verification local and offline-capable: signature check plus
  `sha256sum`/`Get-FileHash`, with no network fetch in either installer. Do not
  make an online-only or keyless verification flow the sole path, because that
  would make an external service part of verification (`ADR-0009` rule 2).
- Key custody belongs in an RFC/ADR before adoption: private key offline,
  rotation and revocation documented, no key generated inside a pull request or
  stored in the repository. `RFC-0021` stage C still lists release key custody
  as an open question; do not claim it resolved here.
- Update `docs/security/INSTALLATION_INTEGRITY.md` with the concrete two-channel
  flow (script from the tag, manifest plus signature from the release) and its
  limits. Keep issue #141 open until a pre-release demonstrates the full flow on
  a clean machine and the Windows PowerShell path runs in CI.

### 4.3 Verification criteria

- A release or pre-release exposes both the checksum manifest and its signature.
- On a clean machine following `docs/security/INSTALLATION_INTEGRITY.md`: obtain
  the installer from the reviewed revision, obtain the manifest and signature
  from the release, verify the signature against the published key, verify the
  manifest with `sha256sum --check`, then run the installer.
- The existing CI checksum/MSRV parity check stays green, and the external
  manifest path fails closed when the manifest is absent or mismatched.
- Windows CI actually executes `install.ps1` with the external manifest set;
  a static parse alone is not sufficient evidence.
- `SECURITY.md` contains the public key fingerprint and the stated limits; no
  private key material appears anywhere in the repository.

### 4.4 ADR compliance

`ADR-0009` rule 4 passes at the interface level today and will pass fully once a
real out-of-band publication and signature exist. `ADR-0011` passes: `minisign`
remains an implementation detail and no vendor name becomes a product component.

## 5. Compliance summary and disagreements

| Check | #112 | #149 | #141 |
| --- | --- | --- | --- |
| Native default, no external service required (ADR-0009 r2/r5) | pass | pass when `otlp` is off | pass |
| External integration behind a Cargo feature | `postgres` only; producer is a marker | `otlp` feature proposed | manifest path is env-gated, no network |
| No third-party brand as component name (ADR-0011) | pass | pass | pass |
| Acyclic crate direction | pass now; no CI guard exists | not applicable | not applicable |
| Docs updated with behaviour in the same PR | required at implementation | required at implementation | guide updated; release flow pending |
| Verification criteria honest and reproducible | correction needed on cycle check | correction needed on protoc premise | partial; E2E pending |

No substantive disagreement with the primary owners. The three corrections that
matter most are: `producer` does not gate code, no CI cycle check exists, and the
protoc premise is not true at the pinned dependency versions. These are recorded
in the issue comments and in the blackboard note for the coordinator.

## 6. Reviewer actions and status

- No files outside this branch were changed. No other role's worktree was
  edited. No dependency, manifest or crate source was modified.
- #112: endorse DESIGN_ONLY/BLOCKED; issue stays open.
- #149: implementation premise corrected; protoc need not be mandatory; can be
  closed after the feature-on, Windows-without-protoc evidence exists.
- #141: endorse the fail-closed interface; issue stays open until a release
  publication and signature path is demonstrated end to end.
