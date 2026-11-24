# Release signing: human decision package and closure evidence (issue #141)

- Date: 2026-11-23 (UTC)
- Role: atender-issues subagent, issue-141 (decision package only; no push, no merge)
- Base commit reviewed: `97b9402f9e5127924c39a9c75174e6444de7ad78`
- Scope: RFC-0030 section 11 closure decision, the release key ceremony, and the
  evidence required to close issue #141.
- Type: additive decision memo. It changes no code, no workflow, no CI secret
  and no trust anchor.
- Disposition: issue #141 stays **OPEN** with the `blocked` label and `Refs #141`.

## 1. State declaration (explicit)

**No production release key exists.** The agent that wrote this memo did **not**
create, generate, publish, sign with, or simulate a production key. No public
key has been published outside the repository as a trust anchor, no signed
manifest or detached signature exists, no release asset carries a signature,
and no release workflow publishes signing assets.

The only key material touched during this work is a throwaway Ed25519 **test**
key produced by `tools/release-signing/release-signing.sh selftest` in a
`mktemp -d` directory under `/tmp`, removed at exit (section 6). Its public
fingerprint has no trust value. The private key bytes were never printed, never
committed and never copied into this memo.

This memo advances the decision package. It does **not** satisfy any of the six
closure conditions in RFC-0030 section 11 and does not authorize the ceremony.

## 2. Decision required from a human

RFC-0030 is still a **draft** (`Status: draft`, section 9 "Decision: Pending").
Per `docs/rfc/README.md` section 4, the decisor is the BDFL for phases 0-3 and
the technical committee from phase 4 onward. A human decisor must accept
RFC-0030 before any ceremony step starts: `RELEASE_KEY_CEREMONY.md` section 1
lists that acceptance as precondition 1 and states the ceremony is **not
authorized yet**. The agent has no authority to accept the RFC, name a decisor,
hold key material or close #141.

Two named maintainers (Generator and Witness) plus the Decisor are required by
the separation of duties in `RELEASE_KEY_CEREMONY.md` section 2. An agent cannot
substitute for any of them.

## 3. The six RFC-0030 section 11 conditions and their current status

The table reproduces the six conditions verbatim in substance and records the
honest status at the base commit. "Satisfied" means verified with evidence in
the repository today; every other row is human work.

| # | RFC-0030 section 11 condition | Status at base commit | Owner | Evidence needed to mark satisfied |
| --- | --- | --- | --- | --- |
| 1 | RFC-0030 is accepted by the decisor defined in `docs/rfc/README.md`. | **NOT MET** - RFC status is `draft`; section 9 says Pending. | Human decisor (BDFL for phases 0-3, technical committee from phase 4). | RFC front matter changed to `accepted`, `docs/rfc/README.md` row updated, and the decisor name, date and PR/commit recorded. |
| 2 | An offline-generated key exists; its public half and fingerprint are in `SECURITY.md` and in two independent out-of-band channels. No private key material is in the repository or CI. | **NOT MET** - no key, no published fingerprint, no channels. | Generator + Witness + Decisor. | Signed ceremony note under `docs/audits/`, public key and fingerprint in `SECURITY.md`, fingerprint in the DNS TXT record and in a maintainer-signed statement outside the repo and DNS zone, plus a check that no private key is in the tree or CI. |
| 3 | A tagged release or pre-release publishes both `installers-<tag>.sha256` and its detached signature, generated from the exact tagged tree. | **NOT MET** - no signed tag or assets. | Release operator + Generator. | Release/tag URLs for both assets, the tag name, the manifest bytes, the detached signature, workflow output showing the manifest regenerated from the exact tagged tree, and `sha256sum --check` passing. |
| 4 | A clean-machine run of `docs/security/INSTALLATION_INTEGRITY.md` succeeds: script from the reviewed tag, manifest plus signature from the release, signature verified before the hash, then the installer run. | **NOT MET** - no clean-machine run with a real key. | Human verifier. | Dated transcript on a clean machine for Linux, macOS and Windows showing signature verification before hash verification and the installer run only after both succeed. |
| 5 | CI checksum/MSRV parity stays green; the external manifest path still fails closed when absent or mismatched; Windows CI actually executes `install.ps1` with the external manifest. | **PARTIALLY MET** - the fail-closed `AG_INSTALLER_CHECKSUMS` interface and checksum parity exist today, but the signed-flow CI parity and the required Windows execution are not recorded for a signed release. | CI/release operator. | Green CI logs for checksum/MSRV parity, tests showing absent and mismatched external manifests fail closed, and a Windows CI log that executes `install.ps1` with the external manifest (not a static parse). |
| 6 | `SECURITY.md` states the published fingerprint and the stated limits, and the guide describes the trust anchor and both channels explicitly. | **NOT MET** - current text is roadmap language; no published fingerprint. | Maintainer + docs. | `SECURITY.md` section with the published fingerprint, key ID, revoked-key list and stated limits, and `docs/security/INSTALLATION_INTEGRITY.md` describing the two channels, fingerprint comparison and local pinning. |

Condition 5 is the only one with existing partial evidence, and even that
partial evidence must be re-run for the signed flow. Conditions 1, 2, 3, 4 and 6
require the human ceremony and a real release. Therefore #141 cannot be closed
by this memo or by any agent action.

## 4. Exact steps the human must take

These steps restate the existing human procedure so the decisor can act without
inventing a key decision. They are instructions for named humans, not actions
taken by the agent.

### 4.1 Accept the RFC (decision)

1. Complete the seven-day comment period recorded in RFC-0030.
2. The decisor defined in `docs/rfc/README.md` section 4 records `accepted`,
   the date and their identity in the RFC front matter and RFC index.
3. If any implementation detail departs from RFC-0030 (for example changing the
   production signing tool from `minisign`), open a successor RFC that
   supersedes it; do not improvise the change in CI.

### 4.2 Offline key generation (`RELEASE_KEY_CEREMONY.md` sections 1-4)

Preconditions: RFC accepted; Generator, Witness and Decisor available in the
same room or on a recorded call; an offline machine with no repository
credentials, no project SSH keys, no CI tokens and no network route; `minisign`
already installed from a verified package (nothing downloaded during the
ceremony); both maintainers have read RFC-0030 sections 4.2, 4.3, 4.6, 4.9 and
the ceremony document. If any precondition is false, stop; do not generate a
key "ahead of time".

1. On the offline machine, with the Witness present, create a dedicated
   encrypted directory for the key and confirm there is no network route.
2. Generate one Ed25519 release key:
   `minisign -G -p ag-release.pub -s ag-release.key -c "Anti-Gravital release manifest key"`.
   Use a randomly generated passphrase recorded only in the custody envelope,
   never in a repository, issue, PR, chat message or CI variable.
3. Record the public key and its key ID/fingerprint as printed by `minisign -R`.
   The Witness independently reads the fingerprint back and confirms the match.
4. Clear any passphrase input from terminal history or scratch files.
5. Create a signed audit note under `docs/audits/` (same PR/commit as the key
   announcement) containing: date/time with time zone; Generator, Witness and
   Decisor reference; exact tool and version and how package provenance was
   checked; the key purpose (only `installers-<tag>.sha256` manifests); the
   public key and fingerprint; the two custody location labels; and a statement
   that the private key and passphrase were not recorded, committed or
   transmitted. Both maintainers sign the note. It is a record, not the anchor.
6. The private key never enters a repository, PR, issue, chat message,
   container image, CI runner or online password manager. No "temporary" test
   with the production key; use `tools/release-signing/` for rehearsals.

### 4.3 Custody (`RELEASE_KEY_CEREMONY.md` section 5)

- Store the private key and passphrase encrypted offline in **two physically
  separate locations** controlled by different named Custodians (for example
  two sealed envelopes in two safes). Access requires both Custodians together.
- Record location labels and Custodian names in the ceremony note, not precise
  addresses.
- Use the key only for release manifests; never for code signing, email,
  package signing or routine operations.
- Log every use (date, tag, manifest hash, fingerprint, both maintainers
  present) with the custody material, not in the repository.
- A hardware token is optional and does not change the two-person rule.

### 4.4 Publication channels (`RELEASE_KEY_CEREMONY.md` section 6)

1. Put the full public key and fingerprint in `SECURITY.md` for convenience and
   state there that this copy is not the anchor.
2. Primary channel: publish the fingerprint and a pointer to the full key in a
   DNS TXT record at `_ag-release-key.gravitalcloud.com`, protected by DNSSEC
   and read over an authenticated channel when published or checked.
3. Secondary channel: publish a statement signed with a maintainer's existing
   personal key on a site or key server **outside the project repository and
   outside the project DNS zone**, containing the same fingerprint and the
   ceremony note reference.
4. Both channels must be established before any signed release is trusted. A
   user pins `~/.config/ag/ag-release.pub` only after comparing both
   fingerprints; disagreement stops the user, and an unreachable channel at
   verification time is not a failure.

### 4.5 Signing a release (`RELEASE_KEY_CEREMONY.md` section 7)

1. The release workflow builds artifacts, regenerates
   `installers-<tag>.sha256` from the exact tagged tree, runs
   `sha256sum --check`, and creates a draft prerelease without publishing.
2. The Release operator hands the exact manifest bytes to the Generator
   offline; record the tag and manifest SHA-256 on both sides.
3. With the Witness present, the Generator restores the key, confirms the key
   ID is active and not revoked, and creates the detached signature with
   `minisign -S -s ag-release.key -m installers-<tag>.sha256 -x installers-<tag>.sha256.minisig`.
4. Verify locally with the pinned public key before upload:
   `minisign -V -p ~/.config/ag/ag-release.pub -m installers-<tag>.sha256 -x installers-<tag>.sha256.minisig`.
5. Only after verification attach both files to the same draft release and
   recheck the manifest; a human publisher then publishes the draft.
6. Return the key to two-location custody immediately.
7. CI never holds the private key. A design that injects it as a CI secret
   requires a successor RFC before implementation (`RELEASE_KEY_CEREMONY.md`
   section 8 and RFC-0030 section 4.2).

### 4.6 Rotation and revocation (`RELEASE_KEY_CEREMONY.md` section 9)

Rotate at least annually or immediately on suspected compromise, loss of
custody or Custodian departure. Publish a transition statement signed by both
old and new keys, keep a 30-day overlap for old signatures, then mark the old
key revoked and update the revoked list in `SECURITY.md`. On compromise, publish
a revocation statement through all channels, remove the key from `SECURITY.md`,
invalidate its signatures, re-sign the current manifest with a new key and
follow the coordinated-disclosure SLA.

## 5. Exact evidence required to close #141

A closing PR must record all six items below with reproducible commands, real
output and no secret material, then use `Closes #141` (this decision memo uses
`Refs #141` only):

1. Acceptance: RFC-0030 front matter `accepted`, updated RFC index, decisor,
   date and PR/commit reference.
2. Key: signed ceremony note under `docs/audits/`; public key and fingerprint in
   `SECURITY.md`; DNS TXT record and maintainer-signed statement outside the
   repo and DNS zone; and a content scan showing no private key in the tree or
   CI configuration.
3. Assets: tag/pre-release URLs for `installers-<tag>.sha256` and
   `installers-<tag>.sha256.minisig`; workflow output proving the manifest came
   from the exact tagged tree; `sha256sum --check` result.
4. Clean machine: dated transcripts for Linux, macOS and Windows following
   `INSTALLATION_INTEGRITY.md` in order (script from reviewed tag, assets from
   release, signature verified before hash, installer run last).
5. CI: green checksum/MSRV parity logs; absent and mismatched external manifest
   cases shown to fail closed; a Windows CI log that actually executes
   `install.ps1` with the external manifest.
6. Documentation: `SECURITY.md` with fingerprint, key ID, revoked-key list and
   stated limits; `INSTALLATION_INTEGRITY.md` with the trust anchor, both
   channels and pinning steps; CHANGELOG updated (K-7).

Until every item is verified, #141 stays open. Writing a memo, rehearsing with a
test key or adding an unsigned step to CI never closes it.

## 6. Agent verification performed (TEST-ONLY, no production key)

All commands ran from the worktree root at the base commit above. Nothing below
is release-authenticity evidence; the fingerprint is a throwaway test value.

### 6.1 Test-key selftest (`tools/release-signing/release-signing.sh selftest`)

```text
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] ============ release-signing selftest (TEST ONLY) ============
[signing] OpenSSL: OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
[signing] state dir: /tmp/ag-release-signing-selftest.plNpEh/state (outside repository, removed at exit)
[signing] [1/6] generate local test Ed25519 keypair
[signing] generated test-only Ed25519 keypair in /tmp/ag-release-signing-selftest.plNpEh/state
[signing] test public key fingerprint (SHA-256 of DER SPKI): 793f1bb12bb0f13e94a0429fb7ad707719da55e36d9c9d9149eb008d721d8911
[signing] private key is never printed and must never leave this temp directory
[signing] [2/6] build a sample sha256sum-format manifest
[signing]   0000000000000000000000000000000000000000000000000000000000000000  install.sh
[signing]   0000000000000000000000000000000000000000000000000000000000000000  install.ps1
[signing] [3/6] sign the manifest
[signing] signed /tmp/ag-release-signing-selftest.plNpEh/installers-test.sha256 -> /tmp/ag-release-signing-selftest.plNpEh/installers-test.sha256.sig (raw Ed25519 signature, test key only)
[signing] [4/6] verify the unmodified manifest (must succeed)
[signing] VERIFY OK: Signature Verified Successfully
[signing] [5/6] tamper with the manifest bytes
[signing] [6/6] verify the tampered manifest (must be rejected)
[signing]   [signing] VERIFY FAILED: 40D7249770700000:error:030000EA:digital envelope routines:EVP_DigestVerify:provider signature failure:../crypto/evp/m_sigver.c:779:ED25519 digest_verify:
[signing]   Signature Verification Failure
[signing] VERIFY FAILED as expected for the tampered manifest
[signing] ============ RESULT: PASS (test key only; no production key exists) ============
EXIT=0
```

Result: PASS, exit 0. The valid test signature was accepted and the one-byte
tamper was rejected. The state directory was removed at exit and no
`/tmp/ag-release-signing-selftest.*` directory remained. No private key was
printed or written to a tracked path.

### 6.2 Reproducing the check

```bash
date -u +'%Y-%m-%dT%H:%M:%SZ'
git rev-parse HEAD
uname -srmo
openssl version
bash tools/release-signing/release-signing.sh selftest
echo "EXIT=$?"
ls -d /tmp/ag-release-signing-selftest.* 2>/dev/null || echo none
git status --short
```

Metadata captured: `2026-11-23T23:55:59Z`, commit `97b9402f9e5127924c39a9c75174e6444de7ad78`,
`Linux 7.0.0-44-generic x86_64 GNU/Linux`, `OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)`.
The fingerprint `793f1bb12bb0f13e94a0429fb7ad707719da55e36d9c9d9149eb008d721d8911`
belongs to that disposable key and has no trust value.

### 6.3 Other gates

Results for `git diff --check`, `scripts/ag-check` and the content/secret scans
are recorded in the matching decision descriptor
`docs/kaizen/pr-drafts/community29-i141.md` after this memo is added. They are
integrity checks of the change only; none of them validates a release signature
or moves a closure condition.

## 7. What this memo does not claim

- No production key was generated, published or used.
- No fingerprint was established as a trust anchor; the DNS TXT record and the
  maintainer-signed statement do not exist.
- No signed manifest, `.minisig` asset, tag or clean-machine install exists.
- `.github/workflows/release.yml` was not modified and claims no signature.
- `scripts/check-release-signature.sh` is not called by CI or the installers.
- RFC-0030 remains a draft and issue #141 remains open with `blocked`.

## 8. Disposition

Decision package advanced. Issue #141 is referenced with `Refs #141` and stays
**OPEN**; this change uses `Closes: none` because it closes no issue. No push,
PR or merge was performed by the agent. The human decisor and the named
maintainers own the ceremony and the six evidence items in section 5.
