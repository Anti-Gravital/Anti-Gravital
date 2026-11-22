# RFC-0030 - Signed release manifest and trust anchor

- Status: draft (proposal; not accepted, not implemented)
- Author: Anti-Gravital maintainers
- Draft date: 2026-11-21
- Target phase: release infrastructure (pre-1.0; prerequisite for RFC-0021 stage C)
- Affected modules or crates: none in `crates/`; `install.sh`, `install.ps1`,
  `tools/check-installers.sh`, `SECURITY.md`,
  `docs/security/INSTALLATION_INTEGRITY.md`, release workflow
- Predecessor RFC: RFC-0021 (stage C leaves release key custody open)
- Comment period: minimum seven calendar days
- Tracked issue: #141 (this RFC does not close it)

**State declaration.** No release signing key exists today. No signed manifest
or signature artifact is published. This RFC defines a design; it does not
create a key, publish a key, sign anything, or claim that issue #141 is closed.

## 1. Motivation

`ADR-0009` rule 4 requires auditable installers with integrity verification.
The current installers fail closed when `AG_INSTALLER_CHECKSUMS` points to an
externally supplied SHA-256 manifest, but they only prove that the installer
bytes match a manifest the user supplied. The trust anchor is the *origin* of
that manifest, and no origin exists yet: `checksums/installers.sha256` lives in
the same repository as the scripts it attests, and GitHub Release assets would
be a different delivery path in the same trust domain.

The infrastructure review at `docs/audits/infrastructure-review-112-149-141.md`
and the maintainer comment on #141 agree: the fail-closed interface is the
correct boundary, but the issue stays open until a signed artifact and an
independently published key exist and the full flow is demonstrated end to end.
`RFC-0021` section 9 also lists release key custody as an open question for
stage C.

## 2. Problem

- A tampered checkout can carry tampered checksums; the repository-local list
  is not a trust anchor and is already labelled as a development fallback.
- A checksum file published as a release asset without a signature is created
  by the same party that controls the repository, so a repository compromise
  defeats it. The asset location is not a trust boundary.
- An online-only or keyless verification scheme would make an external service
  part of verification, violating `ADR-0009` rule 2 and preventing offline
  bootstrap.
- There is no documented decision for key generation, custody, publication,
  rotation or revocation. Implementing a release workflow first would force an
  improvised, unreviewable key decision (`CLAUDE.md` rules 5, 22 and 26).

## 3. Alternatives considered

| Alternative | Advantages | Why it is not chosen |
| --- | --- | --- |
| Do nothing; keep the development fallback | No work | Leaves #141 unclosed and ships no real trust anchor |
| Publish an unsigned checksum manifest as a release asset | Simple; CI-friendly | Same trust domain as the repository; a compromise substitutes both |
| Keyless/online transparency verification | No long-lived key to protect | Requires a mandatory external service at verification time; no offline bootstrap; violates `ADR-0009` rule 2 |
| Download over TLS from the project's own release page only | No new key material | TLS authenticates the transport, not the repository owner; still one trust domain |
| GPG signing key with full WOT | Mature; long-lived identity | Heavyweight ceremony and tooling for a small signed data file; poor Windows parity |
| Embed the public key in the installer only | Zero user setup | An attacker who controls the repository can replace both key and script; not an anchor |
| Offline Ed25519 key (minisign) with the public key published twice | Small artifacts, local offline verification, no service dependency | Chosen; requires a disciplined ceremony and two publication channels |

`minisign` is an open-source tool, not a third-party commercial brand used as a
component name; this is consistent with the `ADR-0011` exemption and with the
infrastructure review.

## 4. Proposed design

### 4.1 Scope

This RFC covers the **current build-from-source installers** and the release
artifacts that let a user verify them. It does not implement RFC-0021 stage C
(prebuilt static binaries downloaded by the installer). A later stage C
installer may embed the public key and verify the signature internally; that
is a separate decision and does not change the interface defined here.

The `AG_INSTALLER_CHECKSUMS` interface stays unchanged: a local path, no
network fetch, fail closed on missing file, missing entry or mismatch. This RFC
adds authenticity to the manifest; it does not change how the installers hash
their own bytes.

### 4.2 Signing key: generation and custody

- One active release key at a time, Ed25519, generated with `minisign -G` on an
  offline machine that has no repository credentials and no network access.
- Generation is a recorded ceremony: two named maintainers present, date,
  purpose and resulting key ID/fingerprint recorded in a signed audit note
  under `docs/audits/`. The private key itself is never recorded, committed,
  pasted into a PR, or placed in CI.
- Private key custody: encrypted offline backup in two physically separate
  locations controlled by named maintainers; access requires both maintainers.
  A hardware token may be used if available, but is not required by this RFC.
- The private key is used only to sign release manifests. It is never used for
  code signing, email, or day-to-day operations.
- CI never holds the private key. The release workflow generates the manifest
  and the signature is produced by the offline ceremony step described in
  4.4. A design that injects a private key into CI as a secret is explicitly
  rejected; changing that requires a successor RFC.
- No key exists yet. The ceremony is a prerequisite for any signed artifact.

### 4.3 Public key publication outside the repository

`SECURITY.md` publishes the full minisign public key and its key
ID/fingerprint for human convenience. That copy is a convenience, not the
anchor: an attacker who controls the repository can edit it.

The same fingerprint must be published through at least **two independent
channels outside the repository**, established before any signed release is
trusted:

1. Primary: a DNS TXT record at `_ag-release-key.gravitalcloud.com` containing
   the key ID/fingerprint and a pointer to the full key. The record should be
   protected by DNSSEC and read over an authenticated channel when published
   or checked. It is a *publication* channel; verification remains offline.
2. Secondary: a statement signed with a maintainer's existing personal key,
   published on a maintainer-controlled site or key server outside the project
   repository and outside the project DNS zone.

A user pins a local copy of the public key (for example
`~/.config/ag/ag-release.pub`) after comparing its fingerprint against both
channels. If the two channels disagree, the user stops; no silent choice of
one over the other. DNS or a maintainer site being unavailable at verification
time is not a failure: the pinned local copy is sufficient.

### 4.4 Manifest and signature format

For a release tagged `<tag>`:

- `installers-<tag>.sha256`: canonical `sha256sum` format, LF line endings,
  one entry per installer with the bare file name as the second field:

  ```
  <64 hex digits>  install.sh
  <64 hex digits>  install.ps1
  ```

  No comment or metadata lines: the Windows and macOS parsers used by the
  installers and by the guide only accept this shape. The file is regenerated
  from the exact tagged tree by a release workflow; it is never hand-edited.
- `installers-<tag>.sha256.minisig`: detached minisign signature over the exact
  bytes of the manifest. Both files are published as release assets on the
  same tag.
- The signature covers the manifest bytes, so any line-ending conversion or
  whitespace edit invalidates it. `.gitattributes` keeps the installers and
  `checksums/installers.sha256` at LF, and `tools/check-installers.sh`
  asserts this. Users on Windows must not re-normalize the scripts to CRLF.
- The manifest attests only the two installer scripts in this RFC. Prebuilt
  binary checksums belong to RFC-0021 stage C and are out of scope here.
- Downgrade risk is bounded: a manifest from an older tag carry the checksums
  of that tag's scripts. If the scripts changed, an old manifest fails the
  hash check. If they did not change, the scripts are byte-identical and the
  downgrade has no security consequence. The guide still requires the user to
  compare the release tag with the reviewed checkout.

### 4.5 Signature verification order

The documented procedure, in this order, is the trust chain:

1. **Bootstrap (once, before any release):** obtain the public key from the two
   out-of-band channels, compare fingerprints, and pin a local copy.
2. **Obtain the script** from the reviewed tag (Git checkout), never by piping
   a downloaded script into a shell.
3. **Obtain the manifest and signature** from the release assets, a separate
   delivery path from the checkout. Their location is not what makes them
   trustworthy.
4. **Verify the signature** locally with the pinned public key, before the
   manifest is used for any decision:
   `minisign -V -p ~/.config/ag/ag-release.pub -m installers-<tag>.sha256 -x installers-<tag>.sha256.minisig`.
   Failure stops the procedure before the installer is run.
5. **Verify the checksums:** `sha256sum --check installers-<tag>.sha256` on
   Linux/macOS, or `Get-FileHash -Algorithm SHA256` on Windows. The installers
   perform the equivalent self-check when `AG_INSTALLER_CHECKSUMS` is exported.
6. **Run the installer** only after steps 4 and 5 succeed.

The installer itself does **not** verify the minisign signature in this design:
signature verification is part of the documented, two-channel procedure that
precedes the installer (`ADR-0009` rule 4). The installer enforces step 5 and
fails closed. Adding optional signature verification inside the installers
(e.g. `AG_INSTALLER_SIGNATURE` when a local `minisign` binary is present) is a
future enhancement, not a requirement and not a mandatory dependency. No step
in this flow fetches anything from the network while verifying.

### 4.6 Rotation and revocation

- Scheduled rotation: at least annually, or immediately on suspicion of
  compromise, loss of custody, or departure of a Custodian.
- Rotation ceremony: generate the new key offline (4.2), then publish a
  transition statement signed by **both** old and new keys containing the new
  fingerprint, in `SECURITY.md` and both out-of-band channels.
- Overlap: old signatures remain verifiable for 30 calendar days after the
  transition statement, so existing releases can be checked. After the window,
  the old key is marked revoked and new releases are signed only by the new
  key. Users re-pin the new key.
- Revocation on compromise: publish a revocation statement signed by the old
  key (if still controlled) in all channels, remove the key from `SECURITY.md`,
  invalidate every signature made with it, re-sign the current manifest with a
  new key, and follow the coordinated-disclosure SLA in `SECURITY.md`. If the
  old key is not controlled, state that explicitly; the statement is then
  published unsigned and its authenticity rests on the channels themselves.
- The offline signing hand-off must refuse to use a revoked key ID, and the
  release workflow checks the recorded key ID before publishing assets.
  Revoked key IDs are listed in `SECURITY.md`.

### 4.7 Offline bootstrap and the development path

A clean machine can verify with three local files: pinned public key,
manifest, signature. No external service, transparency log, keyless issuer or
network fetch is needed at verification time. This satisfies `ADR-0009` rule 2
and keeps the design auditable for contributors without network access.

If the user cannot reach both out-of-band channels, they cannot establish the
anchor. In that case they must not describe the result as a verified install;
they may use the explicitly labelled development fallback (unset
`AG_INSTALLER_CHECKSUMS`, repository-local checksum list) with its documented
limitations.

### 4.8 Windows and macOS parity

- Linux/macOS: `install.sh` already supports `sha256sum` and `shasum -a 256`;
  signature verification uses the same `minisign` command with a locally
  installed binary.
- Windows: `install.ps1` already uses `Get-FileHash -Algorithm SHA256`; the
  operator obtains `minisign.exe` through their normal trusted software
  channel and verifies that tool's own provenance before use. No third-party
  brand becomes a project component.
- Windows CI must actually execute `install.ps1` with an external manifest set,
  including the missing and mismatched cases. A static parse is not evidence.
- All three platforms must preserve LF bytes for the attested files. A CRLF
  checkout breaks both the hash and the signature; the guide states this
  explicitly and `tools/check-installers.sh` enforces it on the repository
  side.

### 4.9 Failure behavior

- Signature invalid, key unavailable, manifest missing, entry missing, hash
  mismatch, unreadable manifest, or no SHA-256 tool available: stop with a
  clear message and a non-zero exit before any Cargo build, install, or write
  outside a temporary directory. No partial installation.
- When an external anchor was supplied, there is no silent fallback to the
  development path. Unset `AG_INSTALLER_CHECKSUMS` is the only development
  mode and always prints the warning it prints today.
- The installers never fetch the manifest or signature. A parsed-but-unknown
  manifest entry is not an excuse to skip verification of the running script;
  the current fail-closed parser is kept.
- A signature failure ends the documented procedure before the installer is
  invoked, so the installer's own fail-closed hash check is the last gate, not
  the first.

### 4.10 Explicit non-claims

- No key has been generated.
- No public key has been published outside the repository.
- No signed manifest or signature artifact exists.
- No release workflow publishes these assets.
- This RFC provides a design only. Issue #141 remains open.

## 5. Plan of implementation

Each step is a separate PR after this RFC is accepted; none is implied by
merging this document.

1. **PR 1 (this branch):** RFC-0030, RFC index, security guide roadmap,
   `SECURITY.md` roadmap and CHANGELOG. Documentation only.
2. **PR 2:** key generation ceremony; publish public key and fingerprint in
   `SECURITY.md` and the two out-of-band channels; record the ceremony without
   secret material. Requires BDFL/committee acceptance of RFC-0030 first.
3. **PR 3:** release workflow that regenerates the manifest from the exact
   tagged tree, performs the offline signing hand-off, and verifies the
   signature with the public key before publishing assets. CI holds no private
   key.
4. **PR 4:** update `docs/security/INSTALLATION_INTEGRITY.md` with the concrete
   two-channel commands and the platform matrix; add Windows CI execution of
   `install.ps1` with the external manifest.
5. **PR 5:** a pre-release or release containing both assets; clean-machine
   end-to-end verification on Linux, macOS and Windows; only then propose
   closing #141 with the evidence listed in section 11.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Private key lost | Medium | New releases cannot be signed | Two offline backups under joint custody; rotation procedure |
| Private key compromised | Low | Attacker can sign malicious manifests | Offline generation, no CI secret, revocation and re-sign procedure |
| Repository compromise edits `SECURITY.md` key | Medium | Users pinned to wrong key | Two independent out-of-band channels; fingerprint comparison at bootstrap |
| DNS record tampered | Low | Wrong fingerprint published | DNSSEC and a second channel; disagreement stops the user |
| CRLF conversion invalidates hash/signature | Medium on Windows | Verification fails | LF attributes, CI check, explicit guide note |
| Old signed manifest replayed | Low | Downgrade attempt | Hash binds manifest to script bytes; tag comparison in guide |
| `minisign` unavailable on a platform | Low | User cannot verify | Local binary from trusted channel; no mandatory service; doc per platform |
| Ceremony done informally | Medium | Custody ambiguity | Two maintainers, signed audit note, recorded fingerprint |
| Users skip signature step | Medium | False sense of security | Guide orders signature before hash; docs state hash alone is not an anchor |

## 7. Impact

- Product scope: no crate, DSL or public API changes; release infrastructure
  and documentation only. Product scope is unchanged.
- Roadmap: enables the open custody item of RFC-0021 stage C; does not start
  stage C itself.
- Operational complexity: one offline key ceremony, two publication channels,
  one additional release asset and one manual signing hand-off per release.
- APIs: `AG_INSTALLER_CHECKSUMS` keeps its current meaning and failure modes.
- Documentation: `SECURITY.md`, `docs/security/INSTALLATION_INTEGRITY.md`,
  `docs/rfc/README.md` and CHANGELOG are updated in this PR.

## 8. Rollback

If the design proves unworkable or the key is compromised: mark this RFC
`superseded` or `rejected` by a successor RFC, revoke any published key in all
channels, remove the signed assets from releases, revert the guide and
`SECURITY.md` sections, and keep the existing fail-closed checksum interface
as the development fallback. No code rollback is required because this RFC
changes no installer behavior. Evidence that triggers rollback: inability to
keep the private key offline, disagreement between publication channels that
cannot be resolved, or a successful signature forgery.

## 9. Decision

Pending. Comment period open; no decisor, date or result yet. This RFC is not
accepted and must not be implemented as if it were.

## 10. References

- Issue #141: installer integrity depends on a checksum file in the same
  repository.
- Infrastructure review: `docs/audits/infrastructure-review-112-149-141.md`,
  section 4.
- `ADR-0009` rules 2, 4 and 5; `ADR-0011`; `ADR-0008` (English canonical for
  `docs/rfc/`).
- `RFC-0021` sections 4.1 and 9 (stage C and open release key custody).
- `docs/security/INSTALLATION_INTEGRITY.md`; `SECURITY.md`.
- `CLAUDE.md` rules 5, 16, 22, 26, 28 and 36.

## 11. Condition to close issue #141

The issue may be closed only when all of the following are true and the
evidence is recorded in the PR or the issue:

1. RFC-0030 is accepted by the decisor defined in `docs/rfc/README.md`.
2. An offline-generated key exists; its public half and fingerprint are in
   `SECURITY.md` and in two independent out-of-band channels. No private key
   material is in the repository or CI.
3. A tagged release or pre-release publishes both `installers-<tag>.sha256` and
   its detached signature, generated from the exact tagged tree.
4. A clean-machine run of `docs/security/INSTALLATION_INTEGRITY.md` succeeds:
   script from the reviewed tag, manifest plus signature from the release,
   signature verified before the hash, then the installer run.
5. CI checksum/MSRV parity stays green; the external manifest path still fails
   closed when absent or mismatched; Windows CI actually executes
   `install.ps1` with the external manifest.
6. `SECURITY.md` states the published fingerprint and the stated limits, and
   the guide describes the trust anchor and both channels explicitly.

Until every item is verified, #141 stays open with evidence. A design RFC such
as this one advances the issue but does not resolve it.
