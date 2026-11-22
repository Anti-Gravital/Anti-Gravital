# Release signing test-key evidence

This file records a real local rehearsal of the flow designed in RFC-0030
sections 4.4 and 4.5. It uses the test-only tooling under
`tools/release-signing/` and an Ed25519 key generated on this machine. No
production key exists, no fingerprint is published as a trust anchor, and no
signature is a release asset. Issue #141 remains open.

All output below is verbatim from the commands shown; exit codes are real.
The private key bytes are not reproduced here. The public key fingerprint
shown is a SHA-256 of the DER-encoded SubjectPublicKeyInfo of a throwaway test
key and has no trust value.

## Host and tooling

```text
=== metadata ===
UTC 2026-11-22T14:56:15Z
9a95e795dc5e15eff9946d5c438df76905f13913
Linux angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC 7.0.0-44-generic #44-Ubuntu SMP PREEMPT_DYNAMIC Fri Nov  6 02:38:04 UTC 2026 x86_64 GNU/Linux
OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
```

`9a95e795dc5e15eff9946d5c438df76905f13913` is the commit the commands ran
against. The OpenSSL CLI is `/usr/bin/openssl`; its provenance is recorded in
`tools/release-signing/README.md`. No crypto code was written for this
rehearsal: key generation, signing and verification are OpenSSL operations.

## Check 1: key material cannot be written inside the repository

Command (from the worktree root):

```bash
AG_SIGN_STATE_DIR="$PWD/.ag-sign-should-not-exist" \
  bash tools/release-signing/release-signing.sh genkey
```

Output:

```text
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] ERROR: state directory must be outside the repository: /home/angel-nereira/Github-local-angelnereira/worktrees11/signing/.ag-sign-should-not-exist
EXIT=1
test -e .ag-sign-should-not-exist -> ABSENT
```

The guard fired before any file was created.

## Check 2: explicit generate, sign, verify, tamper flow

The input manifest is a copy of the repository's `checksums/installers.sha256`
placed at `/tmp/opencode/ag-sign-installers-test.sha256`. The state directory
is `/tmp/opencode/ag-sign-test-state`, outside the worktree.

```bash
export AG_SIGN_STATE_DIR=/tmp/opencode/ag-sign-test-state
cp checksums/installers.sha256 /tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh genkey
bash tools/release-signing/release-signing.sh sign /tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh verify /tmp/opencode/ag-sign-installers-test.sha256
sed -i '1s/^./0/' /tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh verify /tmp/opencode/ag-sign-installers-test.sha256
```

Output:

```text
=== explicit flow: genkey / sign / verify / tamper ===
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] generated test-only Ed25519 keypair in /tmp/opencode/ag-sign-test-state
[signing] test public key fingerprint (SHA-256 of DER SPKI): 8a74823ae72418af43572b8bece9cbd484e9e9cf8842c8bff5e89c43d4786186
[signing] private key is never printed and must never leave this temp directory
EXIT=0
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] signed /tmp/opencode/ag-sign-installers-test.sha256 -> /tmp/opencode/ag-sign-installers-test.sha256.sig (raw Ed25519 signature, test key only)
EXIT=0
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] VERIFY OK: Signature Verified Successfully
EXIT=0
--- tamper: modify one byte in the manifest copy ---
04411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624  install.sh
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] VERIFY FAILED: 40B7274ADD760000:error:030000EA:digital envelope routines:EVP_DigestVerify:provider signature failure:../crypto/evp/m_sigver.c:779:ED25519 digest_verify:
Signature Verification Failure
EXIT=1
```

A one-byte change in the manifest is rejected with a non-zero exit. The
signature covers exact manifest bytes, so LF/CRLF normalization would also be
rejected.

## Check 3: key file permissions and worktree cleanliness

```text
=== key material location check ===
total 8
drwx------  2 angel-nereira angel-nereira   80 Nov 22 09:56 .
drwxrwxr-x 39 angel-nereira angel-nereira 6960 Nov 22 09:56 ..
-rw-------  1 angel-nereira angel-nereira  119 Nov 22 09:56 test-release.key
-rw-r--r--  1 angel-nereira angel-nereira  113 Nov 22 09:56 test-release.pub
=== no key material in worktree (git status) ===
?? tools/release-signing/
```

The private key is mode `0600` in a `0700` directory under `/tmp/opencode`,
never in a tracked path.

## Check 4: one-command selftest

Command and exit code:

```bash
bash tools/release-signing/release-signing.sh selftest
echo "EXIT=$?"
```

Output:

```text
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] ============ release-signing selftest (TEST ONLY) ============
[signing] OpenSSL: OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
[signing] state dir: /tmp/ag-release-signing-selftest.zAYzkA/state (outside repository, removed at exit)
[signing] [1/6] generate local test Ed25519 keypair
[signing] generated test-only Ed25519 keypair in /tmp/ag-release-signing-selftest.zAYzkA/state
[signing] test public key fingerprint (SHA-256 of DER SPKI): be81fa2262638607e0e772c5b9cf86b1641b9b67f0f5904cc758955ac7b58cf1
[signing] private key is never printed and must never leave this temp directory
[signing] [2/6] build a sample sha256sum-format manifest
[signing]   0000000000000000000000000000000000000000000000000000000000000000  install.sh
[signing]   0000000000000000000000000000000000000000000000000000000000000000  install.ps1
[signing] [3/6] sign the manifest
[signing] signed /tmp/ag-release-signing-selftest.zAYzkA/installers-test.sha256 -> /tmp/ag-release-signing-selftest.zAYzkA/installers-test.sha256.sig (raw Ed25519 signature, test key only)
[signing] [4/6] verify the unmodified manifest (must succeed)
[signing] VERIFY OK: Signature Verified Successfully
[signing] [5/6] tamper with the manifest bytes
[signing] [6/6] verify the tampered manifest (must be rejected)
[signing]   [signing] VERIFY FAILED: 40A7CBE6BB790000:error:030000EA:digital envelope routines:EVP_DigestVerify:provider signature failure:../crypto/evp/m_sigver.c:779:ED25519 digest_verify:
[signing]   Signature Verification Failure
[signing] VERIFY FAILED as expected for the tampered manifest
[signing] ============ RESULT: PASS (test key only; no production key exists) ============
EXIT=0
```

## Check 5: opt-in signature verifier (`scripts/check-release-signature.sh`)

This records a real run of the opt-in verifier added under issue #141. It uses
the same throwaway test key as the signing tooling; it is **not** in CI, not
called by the installers and not a production trust decision. The verifier
always prints `TEST KEY ONLY - NOT A PRODUCTION SIGNATURE` before a verdict.

Metadata for this run:

```text
=== metadata ===
UTC 2026-11-22T17:01:34Z
72b97adbdccf8cab3fbd5f411f54343beab40672
Linux 7.0.0-44-generic x86_64 GNU/Linux
OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
```

Commands (from the worktree root; state dir and manifest under `/tmp/opencode`,
outside the worktree):

```bash
export AG_SIGN_STATE_DIR=/tmp/opencode/ag-verify-i141-state
export AG_TEST_RELEASE_PUBKEY=/tmp/opencode/ag-verify-i141-state/test-release.pub
export AG_SIGNED_MANIFEST=/tmp/opencode/ag-verify-i141-installers.sha256
bash tools/release-signing/release-signing.sh genkey
cp checksums/installers.sha256 "$AG_SIGNED_MANIFEST"
bash tools/release-signing/release-signing.sh sign "$AG_SIGNED_MANIFEST"
bash scripts/check-release-signature.sh
sed -i '1s/^./0/' "$AG_SIGNED_MANIFEST"
bash scripts/check-release-signature.sh
```

Output (verbatim; exit codes real):

```text
=== 1/6 genkey ===
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] generated test-only Ed25519 keypair in /tmp/opencode/ag-verify-i141-state
[signing] test public key fingerprint (SHA-256 of DER SPKI): 2d9bd9d73ad91a0c324d350ced8f6d633866dcba1c4b62d9464e49ff9369c896
[signing] private key is never printed and must never leave this temp directory
EXIT=0
=== 2/6 prepare manifest copy ===
74411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624  install.sh
9f6022f7186bbed86b93ef4942edb170510ad709d5a42b348793401c3fc9274c  install.ps1
EXIT=0
=== 3/6 sign ===
[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL
[signing] signed /tmp/opencode/ag-verify-i141-installers.sha256 -> /tmp/opencode/ag-verify-i141-installers.sha256.sig (raw Ed25519 signature, test key only)
EXIT=0
=== 4/6 verify via opt-in checker (expect PASS) ===
[verify] TEST KEY ONLY - NOT A PRODUCTION SIGNATURE - RFC-0030 REHEARSAL
repo.root=/home/angel-nereira/Github-local-angelnereira/worktrees12/i141
input.pubkey=/tmp/opencode/ag-verify-i141-state/test-release.pub
input.manifest=/tmp/opencode/ag-verify-i141-installers.sha256
input.signature=/tmp/opencode/ag-verify-i141-installers.sha256.sig
key.test_fingerprint_sha256=2d9bd9d73ad91a0c324d350ced8f6d633866dcba1c4b62d9464e49ff9369c896
tool.openssl=OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
signature.verified=yes
[verify] TEST KEY ONLY - NOT A PRODUCTION SIGNATURE - RFC-0030 REHEARSAL
RESULT: PASS reason=test-signature-verified detail=TEST KEY ONLY - NOT A PRODUCTION SIGNATURE; rehearsal pass only, not release authenticity
EXIT=0
=== 5/6 tamper one byte ===
04411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624  install.sh
9f6022f7186bbed86b93ef4942edb170510ad709d5a42b348793401c3fc9274c  install.ps1
EXIT=0
=== 6/6 verify tampered via opt-in checker (expect FAIL) ===
[verify] TEST KEY ONLY - NOT A PRODUCTION SIGNATURE - RFC-0030 REHEARSAL
repo.root=/home/angel-nereira/Github-local-angelnereira/worktrees12/i141
input.pubkey=/tmp/opencode/ag-verify-i141-state/test-release.pub
input.manifest=/tmp/opencode/ag-verify-i141-installers.sha256
input.signature=/tmp/opencode/ag-verify-i141-installers.sha256.sig
key.test_fingerprint_sha256=2d9bd9d73ad91a0c324d350ced8f6d633866dcba1c4b62d9464e49ff9369c896
tool.openssl=OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)
RESULT: FAIL reason=signature-mismatch detail=signature does not verify against these exact manifest bytes (tampering or wrong key/signature)
--- openssl output (start) ---
40372C45A8740000:error:030000EA:digital envelope routines:EVP_DigestVerify:provider signature failure:../crypto/evp/m_sigver.c:779:ED25519 digest_verify:
Signature Verification Failure
--- openssl output (end) ---
EXIT=1
```

The valid signature is accepted and the one-byte tamper is rejected with a
non-zero exit. Both verdicts carry the test-only banner. The fingerprint
`2d9bd9d73ad91a0c324d350ced8f6d633866dcba1c4b62d9464e49ff9369c896` is a local
throwaway key and has no trust value.

### Absence is SKIP, never PASS

With no inputs supplied, the verifier exits 0 but reports `SKIP`, not `PASS`.
Since exit 0 can mean either PASS or SKIP, a caller must read the `RESULT:`
line as the verdict:

```bash
env -u AG_TEST_RELEASE_PUBKEY -u AG_SIGNED_MANIFEST -u AG_MANIFEST_SIGNATURE \
  bash scripts/check-release-signature.sh
echo "EXIT=$?"
```

```text
[verify] TEST KEY ONLY - NOT A PRODUCTION SIGNATURE - RFC-0030 REHEARSAL
repo.root=/home/angel-nereira/Github-local-angelnereira/worktrees12/i141
RESULT: SKIP reason=no-inputs-supplied detail=supply AG_TEST_RELEASE_PUBKEY and AG_SIGNED_MANIFEST to verify; absence is never PASS
EXIT=0
```

A partial input set (public key only) also returns SKIP with the missing input
named; a supplied-but-missing signature file returns FAIL. Both were exercised
locally. None of these paths produces a PASS on absent material.

## What this evidence does not show

- No production key was generated and no production fingerprint was published.
- No out-of-band DNS TXT record or maintainer statement was created.
- `RELEASE_KEY_CEREMONY.md` describes those human steps; they have not happened.
- `.github/workflows/release.yml` was not modified and claims no signature.
- The opt-in `scripts/check-release-signature.sh` is not called by CI or the
  installers; its PASS is a rehearsal result, not release authenticity.
- No release asset, tag or clean-machine install was produced.

This rehearsal only proves that the local tooling can generate a test key,
sign a manifest, verify it and reject tampering. The six closure conditions in
RFC-0030 section 11 are still unmet, so issue #141 stays open.
