# RFC-0032 - Alpha release process

- Status: accepted
- Decision date: 2026-11-22
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-22
- Target phase: release infrastructure; alpha milestone (pre-0.5)
- Affected modules or crates: workspace manifests (`Cargo.toml`,
  `Cargo.lock`), `.github/workflows/release.yml`, `CHANGELOG.md`,
  `docs/rfc/README.md`
- Predecessor RFC: RFC-0030 (signed manifest design, still a draft),
  RFC-0022 (`--locked` gate)
- Comment period: waived by the recorded BDFL decision in section 9
- Tracked issue: #141 (referenced, not closed)
- Fits the scope statement: yes (release infrastructure, not a product feature)

**State declaration.** This RFC authorizes an unsigned, build-from-source
alpha prerelease. It does not create a signing key, publish signed assets,
or close issue #141.

## 1. Motivation

The repository has nine phases of implemented code, ten runnable examples,
two binaries and CI on four target platforms, but no tagged release and no
repeatable way to obtain versioned binaries. Users can only build from a
checkout. The alpha milestone needs three concrete properties: an
executable artifact per supported platform, an installable path with a
known version, and a reproducible integrity check that a tester can run.

At the same time, release infrastructure must not invent a trust anchor.
RFC-0030 documents a future signed-manifest design; no key exists, no key
ceremony has happened, and #141 stays open. This RFC therefore defines an
honest unsigned alpha and explicitly separates checksum integrity from
cryptographic authenticity.

## 2. Problem

- `Cargo.toml` pins the whole workspace at `0.0.0`, so nothing can be
  identified by a version string or a release tag.
- There is no release workflow. The `ci` workflow builds and tests every
  commit but produces no downloadable artifact.
- The installers (`install.sh`, `install.ps1`) build from source and keep
  their fail-closed `AG_INSTALLER_CHECKSUMS` interface, but no release-side
  manifest exists to feed that interface in an alpha channel.
- Without a written process, the first release risks being an improvised
  local tag with hand-made assets, which would violate `CLAUDE.md` rules 5,
  22, 26 and 36.

## 3. Alternatives considered

| Alternative | Advantages | Why it is not chosen |
| --- | --- | --- |
| Publish nothing until 1.0 | No release risk | Users cannot test or give feedback on a real alpha; the milestone requires an executable, installable, testable platform |
| Publish crates to crates.io | Standard Rust install path | `publish = false` is deliberate: APIs are unstable, the pre-Phase-5 gate is open, and no downstream consumer has asked for packages; publishing would create a SemVer contract we cannot honor |
| Signed release assets now | Real trust anchor | Requires the RFC-0030 key ceremony, two out-of-band channels and an end-to-end demonstration; none exists (#141 is blocked) |
| Unsigned checksum manifest attached to a draft prerelease | Reproducible integrity check, no invented key, easy rollback | Chosen; integrity only, explicitly not authenticity |
| Manual local build and upload | Zero workflow code | Not reproducible, not auditable, fails rule 36 |

## 4. Proposed design

### 4.1 Version scheme and tag

- Single source of truth: `[workspace.package] version` in `Cargo.toml`.
  Every crate uses `version.workspace = true`, so all published-facing
  packages share one version. Internal path-dependency version requirements
  are kept in sync with it.
- Scheme: SemVer with a numeric prerelease counter,
  `<major>.<minor>.<patch>-alpha.<n>`. The first alpha is
  `0.1.0-alpha.1`; the matching Git tag is `v0.1.0-alpha.1`.
- Later alphas increment `alpha.<n>` without changing the core triple.
  A beta or a stable `0.1.0` needs its own decision and the closure of the
  relevant roadmap gate; this RFC does not authorize either.
- The `0.x` major and the `alpha` suffix state the stability promise:
  public APIs, generated code and the DSL may still change. The alpha tag
  is not a compatibility guarantee.
- `publish = false` is retained in `[workspace.package]` and in the
  crates. No crates.io publication is part of this release.
- The workflow refuses a tag whose `v`-stripped value differs from the
  workspace version, so a tag cannot silently describe different bytes.

### 4.2 Build matrix and binaries

Four target platforms, the same set as `.github/workflows/ci.yml`:

| Label | Runner | Target triple |
| --- | --- | --- |
| `linux-x86-64` | `ubuntu-latest` | `x86_64-unknown-linux-gnu` |
| `linux-arm64` | `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` |
| `macos-arm64` | `macos-15` | `aarch64-apple-darwin` |
| `windows-x64` | `windows-latest` | `x86_64-pc-windows-msvc` |

Two binaries are built per target, from the only two `[[bin]]` targets in
`crates/`:

- `ag` from `ag-cli` (the unified developer CLI).
- `ag-lsp` from `ag-lsp` (the Anti-DSL language server).

That is eight binary artifacts per release. Release assets are named
`<binary>-<target>` with a `.exe` suffix on Windows, for example
`ag-x86_64-unknown-linux-gnu` and `ag-lsp-aarch64-apple-darwin`; the matrix
label is only an internal CI artifact name. This is the naming the opt-in
alpha channel of the installers expects. All builds use
`cargo build --locked --release` with the pinned toolchain, so the release
consumes exactly the committed lockfile (RFC-0022 section 4.2). Examples
and benchmark binaries are not release artifacts.

### 4.3 Checksum manifest generation

- The release job downloads the eight artifacts from its own workflow run,
  then generates one canonical manifest on a Linux runner with
  `sha256sum`, sorted by relative path.
- Format: plain `sha256sum` output, LF endings, one entry per binary, no
  comment or metadata lines. The second field is the bare asset name,
  which is unique because the target triple is part of it:

  ```
  <64 hex digits>  ag-x86_64-unknown-linux-gnu
  <64 hex digits>  ag-lsp-x86_64-unknown-linux-gnu
  ```

- File name: `checksums.txt`, one manifest for the whole release. The
  installer alpha channel downloads this file and the matching
  `ag-<target>` asset from the same release; its parsers look up the bare
  name in the second field, so no path prefix may be added.
- The job verifies its own manifest with `sha256sum --check` before any
  release asset is created. A mismatch fails the job.
- This manifest proves that a downloaded binary matches the one built by
  this workflow. It is NOT a trust anchor: the release page and the
  repository are the same trust domain. Users must not describe a
  checksum-only check as an authenticity check.
- `checksums/installers.sha256` (the installer self-check fallback) is a
  separate artifact with a separate purpose and is not regenerated here.
  Its signed successor is RFC-0030.

### 4.4 Draft prerelease and publication boundary

- `.github/workflows/release.yml` runs on `workflow_dispatch` (explicit
  tag input) and on pushed tags matching `v*`.
- The workflow builds, checks sums, and creates a GitHub release that is
  both `draft` and `prerelease`. It never publishes it.
- The tag must already exist; the workflow does not create, push or move
  tags, does not push commits, and does not merge branches.
- Release notes state the alpha status, the four targets, the two
  binaries, the source-install path, the opt-in checksum-only alpha channel
  and the unsigned-manifest limitation.
- The coordinator reviews the draft, the artifact list and the manifest,
  then publishes it manually. Publication is a human action outside this
  RFC's automation.

### 4.5 Why signing is NOT claimed

- No release signing key exists. RFC-0030 is still a draft; its section 4
  requires an offline key ceremony, a public key in `SECURITY.md` and two
  independent out-of-band publication channels. None of those steps has
  happened.
- Issue #141 is open and labelled `blocked`. Its condition is a real trust
  anchor, not merely a checksum file on a release page.
- Consequently the workflow creates no `.minisig` file, does not modify
  `SECURITY.md`, and makes no signed or verified-install claim. The
  CHANGELOG and release notes repeat this limitation in plain language.
- The installers keep their current fail-closed behavior: an externally
  supplied `AG_INSTALLER_CHECKSUMS` must match or the install aborts. The
  opt-in alpha channel consumes this release's `checksums.txt` and the
  matching `ag-<target>` asset with the same fail-closed comparison; it is
  checksum-only and makes no signature claim. Key custody and signing stay
  outside this RFC.

### 4.6 Permissions and pinned actions

- Top-level permissions are `contents: read`. Only the release job gets
  `contents: write` (to create the draft) and `actions: read` (to download
  this run's artifacts), and only for that job.
- All third-party actions are pinned to full commit SHAs with a version
  comment, following the existing workflows: `actions/checkout`,
  `dtolnay/rust-toolchain` and `actions/upload-artifact`. Artifact download
  uses the preinstalled `gh` CLI, so no additional action pin is needed.
- No new dependency, crate, DSL change or public API is introduced.

## 5. Plan of implementation

1. This PR: RFC-0032, RFC index entry, version bump with lockfile refresh,
   release workflow, CHANGELOG alpha section and PR descriptor.
2. Follow-up by the installer role: an alpha installer channel that uses
   the published manifest without inventing a signing key.
3. Only after RFC-0030 is accepted: key ceremony, out-of-band key
   publication and a signed-manifest workflow. That work proposes closing
   #141 with the evidence listed in RFC-0030 section 11.
4. A stable release needs the pre-Phase-5 gate and the Phase 10 rules; it
   is outside this RFC.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Users read a checksum manifest as a trust anchor | Medium | False security confidence | Explicit non-claim in RFC, CHANGELOG and release notes; wording "integrity only, not authenticity" |
| Tag does not match built sources | Low | Misleading artifact | Workflow compares tag with the workspace version and requires an existing tag |
| One platform fails to build | Medium | Incomplete release | Matrix with `fail-fast: false`; the release job needs all four builds, so a single failure blocks the draft |
| `--locked` drift after the version bump | Medium | Workflow fails late | Refresh `Cargo.lock` with `cargo check` in the implementing PR and keep every Cargo invocation `--locked` |
| Accidental publication | Low | Unsupported binary in the wild | `--draft` and `--prerelease` are hard-coded; manual publish step left to the coordinator |
| Unpinned or over-privileged CI action | Low | Supply-chain or token misuse | Full SHA pins and job-scoped minimal permissions |
| Checksum file normalized to CRLF | Low | Verification fails | Manifest generated on Linux with LF bytes and verified in the same job |

## 7. Impact

- Product scope: unchanged. No crate, feature, DSL or API is added.
- Roadmap: adds the alpha release milestone; it does not close a phase
  exit gate or the pre-Phase-5 release gate.
- Operations: one new manually dispatched workflow and one draft release
  per alpha tag.
- Public APIs: unchanged. The version string becomes observable, but
  `publish = false` means no package-registry SemVer contract is created.
- Documentation: this RFC, `docs/rfc/README.md` and a CHANGELOG section.
  The README is owned by the docs role and is not edited here.

## 8. Rollback

Before a draft is published, rollback is a plain revert of the version
bump, the workflow and the documentation entries; no external state
exists. If a draft was already created, the coordinator first deletes the
draft release and its assets, then the commits are reverted. Triggering
evidence: a build that fails on any of the four targets, a checksum
self-check failure, a tag/version mismatch, or an asset published without
the draft flag. A signed-release decision requires a successor RFC that
supersedes this one; this document then becomes historical.

## 9. Decision

- Decisor: BDFL (Angel Nereira).
- Decision date: 2026-11-22.
- Result: accepted.
- Recorded waiver: the seven-calendar-day comment period of
  `docs/rfc/README.md` and the phase-4 committee decision are waived for
  this release-only decision. Basis: `GOVERNANCE.md` reserves version
  release authority up to and including 0.5 to the BDFL, no technical
  committee has been constituted yet (`docs/roadmap/STATUS.md`), and the
  alpha milestone has a fixed date. The waiver is recorded here for audit,
  not hidden; any change to the version scheme or to the signing claim
  needs a successor RFC.

## 10. References

- Issue #141: installer integrity depends on a checksum file inside the
  same repository (open, blocked; this RFC does not close it).
- RFC-0030: signed release manifest and trust anchor (draft; no key).
- RFC-0022 section 4.2: every Cargo invocation in CI uses `--locked`.
- `GOVERNANCE.md`: BDFL release authority through 0.5.
- `docs/rfc/README.md`: RFC process and decision rules.
- `CLAUDE.md` rules 5, 16, 17, 22, 26, 28, 29 and 36.
