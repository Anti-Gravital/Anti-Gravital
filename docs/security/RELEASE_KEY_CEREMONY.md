# Release key ceremony

This document describes the **human** procedure for creating, publishing,
custodying, rotating and revoking the Anti-Gravital release signing key. It is
the procedural companion to
`docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md` (the signed
manifest design) and `docs/security/INSTALLATION_INTEGRITY.md` (the user-facing
verification guide).

## State declaration

**No production release key exists. No public key has been published outside
the repository. No signed manifest or detached signature exists. No release
workflow publishes signing assets.** This document defines a procedure to be
performed by named humans; it does not perform it.

Issue #141 stays **open** until every closure condition in RFC-0030 section 11
is met with recorded evidence. Writing this document, rehearsing with a local
test key (`tools/release-signing/`) or adding a signing step to CI does not
close it. The local rehearsal uses an OpenSSL Ed25519 test key and produces no
artifact with trust value.

## 1. Preconditions

None of these steps may start until all are true:

1. RFC-0030 is accepted by the decisor defined in `docs/rfc/README.md`. It is a
   draft at the time of writing, so this ceremony is **not authorized yet**.
2. Two named maintainers are available in the same room or on a recorded call:
   the **Generator** (operates the offline machine) and the **Witness**
   (observes every step and countersigns the ceremony note). A third person,
   the **Decisor**, accepts the RFC and any successor decisions.
3. The Generator has an offline machine with no repository credentials, no SSH
   keys for the project, no CI tokens and no network access. The signing tool
   (`minisign`, per RFC-0030 section 4.2) is already installed from a verified
   package; nothing is downloaded during the ceremony.
4. A review of `docs/security/RELEASE_KEY_CEREMONY.md` and RFC-0030 sections
   4.2, 4.3, 4.6 and 4.9 has been read by both maintainers.

If any precondition is false, stop. Do not generate a key "ahead of time".

## 2. Roles and separation of duties

| Role | Responsibility | Must not do |
| --- | --- | --- |
| Generator | Generate the key offline, sign release manifests during a release, restore the key from custody for signing | Publish the key alone, store the key in a repository, cloud drive or password manager shared with one person |
| Witness | Observe generation and signing, verify recorded fingerprint, countersign the ceremony note | Hold a copy of the private key alone |
| Decisor | Accept RFC-0030 and successor RFCs, accept the ceremony note | Generate or sign with the key |
| Release operator | Run the release workflow, collect the generated manifest, hand it to the Generator, publish verified assets | Possess the private key or weaken the workflow's checks |

Two-person control means the private key is never used, restored or transported
by a single person acting alone.

## 3. Offline key generation

On the offline machine, with the Witness present:

1. Create a dedicated, encrypted directory for the key, for example on a
   hardware-encrypted volume. Confirm the machine has no network route.
2. Generate one Ed25519 release key with the established tool selected by
   RFC-0030:

   ```text
   minisign -G -p ag-release.pub -s ag-release.key -c "Anti-Gravital release manifest key"
   ```

   The tool prompts for a passphrase. Use a randomly generated passphrase
   recorded only in the custody envelope (section 5), never in a repository,
   issue, PR, chat message or CI variable.

3. Record the public key and its key ID/fingerprint as printed by
   `minisign -R` (or the tool's equivalent fingerprint command). The
   fingerprint is the value later compared across the two publication
   channels.
4. The Witness independently reads the fingerprint back from the public key
   file and confirms it matches the recorded value.
5. Destroy any intermediate copies of the passphrase input from the terminal
   history or scratch files before leaving the offline machine.

The private key never enters the repository, a PR, an issue, a chat message, a
container image, a CI runner or an online password manager. There is no
exception for "temporary" testing with the production key: use
`tools/release-signing/` for rehearsals instead.

## 4. Ceremony record

Create a signed audit note under `docs/audits/` in the same PR or commit that
announces the published key. It contains:

- date and time (with time zone) of the ceremony;
- names of the Generator and Witness, and the Decisor decision reference;
- the exact tool and version used, and how its package provenance was checked;
- the purpose of the key (signing `installers-<tag>.sha256` manifests only);
- the public key and its fingerprint;
- the two custody locations (by label, not by precise address);
- a statement that the private key and passphrase were not recorded, committed
  or transmitted.

The note is signed by both maintainers using their existing personal keys. It
is a record of the ceremony, not a trust anchor for users; the anchor is the
two out-of-band channels in section 6.

## 5. Custody

- The private key and its passphrase are stored **encrypted offline in two
  physically separate locations** controlled by different named maintainers
  (for example two sealed envelopes in two safes). Access requires both
  Custodians together; neither can open an envelope alone.
- Location labels and Custodian names go into the ceremony note. Physical
  addresses do not.
- The key is used only to sign release manifests. It is never used for code
  signing, email, package signing or routine operations.
- Any use of the key is logged: date, tag, manifest hash, fingerprint, and the
  names of the two maintainers present. The log lives with the custody
  material, not in the repository.
- A hardware token may hold the key if the maintainers already operate one;
  it is optional and does not change the two-person rule.

## 6. Publishing the public key outside the repository

The repository copy is a convenience, not an anchor: an attacker who controls
the repository can edit it. Publish the full public key and fingerprint in
`SECURITY.md`, then publish the **same fingerprint** through two independent
channels that are outside the repository and outside each other's trust
domain. Both channels must be established before any signed release is
trusted.

1. **Primary - DNS TXT record.** Publish the fingerprint and a pointer to the
   full key at `_ag-release-key.gravitalcloud.com`. Protect the zone with
   DNSSEC and read the record over an authenticated channel when publishing or
   checking it. DNS is a *publication* channel; verification of a release
   stays local and offline.
2. **Secondary - maintainer-controlled statement.** Publish a statement
   signed with a maintainer's existing personal key on a site or key server
   outside the project repository and outside the project DNS zone. The
   statement contains the same fingerprint and the ceremony note reference.

A user pins a local copy of the public key (for example
`~/.config/ag/ag-release.pub`) only after comparing the fingerprint from both
channels. If the channels disagree, the user stops; no silent preference for
one channel over the other. If a channel is unreachable at verification time,
that is not a failure: the pinned local copy is sufficient. Full user steps
are in `docs/security/INSTALLATION_INTEGRITY.md`.

## 7. Signing a release

1. The release workflow (`.github/workflows/release.yml`) builds the artifacts,
   generates `installers-<tag>.sha256` from the exact tagged tree and verifies
   its own manifest with `sha256sum --check`. It creates a draft prerelease and
   publishes nothing by itself.
2. The Release operator hands the exact manifest bytes to the Generator
   through the agreed offline hand-off. The tag and SHA-256 of the manifest are
   recorded on both sides.
3. With the Witness present, the Generator restores the key from custody,
   confirms the recorded key ID is active and not revoked, and creates the
   detached signature:

   ```text
   minisign -S -s ag-release.key -m installers-<tag>.sha256 \
     -x installers-<tag>.sha256.minisig
   ```

   The signature covers the exact manifest bytes. Any line-ending or
   whitespace change invalidates it; attested files and manifests must stay LF.

4. The Generator or Release operator verifies the signature locally with the
   pinned public key before upload:

   ```text
   minisign -V -p ~/.config/ag/ag-release.pub \
     -m installers-<tag>.sha256 -x installers-<tag>.sha256.minisig
   ```

5. Only after verification succeeds are both files attached to the same draft
   release and the manifest checked with `sha256sum --check`. The human
   publisher then publishes the draft.
6. The key returns to two-location custody immediately after signing.

The signature authenticates the manifest, not the binaries listed in
`checksums.txt` of RFC-0032. Prebuilt binary signing belongs to a later stage
C decision.

## 8. Private key in CI: current decision and future option

**Current design (RFC-0030 section 4.2): CI never holds the private key.**
The workflow generates and verifies the manifest; a human performs the offline
signing hand-off in section 7. `.github/workflows/release.yml` does not read
any signing secret and does not produce a `.minisig` asset. Do not add a secret
to CI and call the result an RFC-0030 signing implementation.

A future design could let a release workflow consume the private key from CI
secrets. That changes the trust model (an online service would hold key
material) and therefore **requires a successor RFC that supersedes RFC-0030**
before any implementation, per `CLAUDE.md` rules 5, 22 and 26. If such an RFC
is accepted, the minimum controls would be:

- the private key stored as a masked GitHub Actions secret (for example
  `AG_RELEASE_SIGNING_KEY`) in a protected `release` environment with required
  reviewers, exposed only to the signing job;
- the job materializes it into a `0600` temporary file outside the checkout,
  never echoes it, never passes it on a command line that could be logged, and
  deletes it in an `EXIT` trap;
- the workflow verifies the resulting signature with the independently pinned
  public key before publishing any asset, and fails closed on a mismatch;
- the secret is rotated at the same cadence as the key and immediately on
  Custodian departure or suspected exposure;
- the ceremony note and `SECURITY.md` record that the key is CI-held, so users
  can judge the different trust model.

Until that RFC exists, this paragraph is a description of a possible future
option, not an instruction to implement it.

## 9. Rotation and revocation

- **Scheduled rotation:** at least annually, or immediately on suspicion of
  compromise, loss of custody, loss of a Custodian, or their departure.
- **Rotation ceremony:** generate the new key offline as in section 3, then
  publish a transition statement signed by **both** the old and new keys that
  contains the new fingerprint, in `SECURITY.md` and in both out-of-band
  channels. The offline signing hand-off must refuse a revoked key ID.
- **Overlap:** old signatures remain verifiable for 30 calendar days after the
  transition statement so existing releases can still be checked. After the
  window the old key is marked revoked, new releases are signed only by the
  new key, and users re-pin the new key.
- **Revocation on compromise:** publish a revocation statement signed by the
  old key if it is still controlled, in all channels; if it is not controlled,
  state that explicitly and publish the unsigned statement, with authenticity
  resting on the channels themselves. Remove the key from `SECURITY.md`, add
  its ID to the revoked list, invalidate every signature made with it, re-sign
  the current manifest with a new key, and follow the coordinated-disclosure
  SLA in `SECURITY.md`.

## 10. Failure behavior

Any invalid signature, unavailable key, missing manifest, missing entry, hash
mismatch, unreadable manifest or missing SHA-256 tool stops the procedure with
a clear message and a non-zero exit before any install action. There is no
silent fallback to the development path: unset `AG_INSTALLER_CHECKSUMS` is the
only development mode and always prints its warning. The installers never
fetch a signature or manifest from the network.

## 11. Open issue

This document and `tools/release-signing/` prepare the ceremony but do not
perform it. No production key has been generated, no fingerprint has been
published in two independent channels, no tag publishes a signed manifest and
no clean-machine end-to-end run has been demonstrated. Consequently issue #141
remains open with the `blocked` label. Close it only with the six items of
evidence listed in RFC-0030 section 11.
