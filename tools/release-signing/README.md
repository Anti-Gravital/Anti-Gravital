# Release signing rehearsal (test key only)

This directory contains a local rehearsal of the signed-manifest flow designed
in `docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md`. It uses a
throwaway Ed25519 key generated on this machine. It does **not** create,
publish or reference a production release key, and it does not change
`.github/workflows/release.yml`.

## Status

- Test-only tooling. No production key exists.
- Issue #141 remains open. This directory advances the design; it does not
  satisfy any of the closure conditions in RFC-0030 section 11.
- No signature produced here is a release asset and no `.minisig` file is
  published. The output is a raw 64-byte Ed25519 signature (`.sig`) for local
  rehearsal only.
- `scripts/check-release-signature.sh` is an opt-in verifier for that test
  signature. It is not called by CI or the installers and never claims release
  authenticity; see the section below.

## Tool provenance

The script `release-signing.sh` contains no cryptographic implementation. It
orchestrates the OpenSSL command-line tool that is already installed on the
build host:

| Property | Value recorded on 2026-11-22 |
| --- | --- |
| Tool | OpenSSL CLI (`openssl`) |
| Version | `OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)` |
| Path | `/usr/bin/openssl` |
| Upstream | The OpenSSL Project, <https://www.openssl.org/> |
| License | Apache License 2.0 (OpenSSL 3.x) |
| Primitive | Ed25519 (`genpkey -algorithm ED25519`, `pkey -pubout`, `pkeyutl -sign/-verify -rawin`) |

Why OpenSSL here: `minisign` is not installed on this host and `age` is not
available either. OpenSSL is installed, maintained and provides Ed25519 through
its documented CLI, so it is suitable for a local rehearsal. RFC-0030 still
selects `minisign` for the **production** ceremony; its detached `.minisig`
format is different from the raw signature produced here. Switching the
production tool is a successor-RFC decision, not a change to be made in this
tooling. `gpg` is also present but is not used: RFC-0030 rejected a GPG/WOT
ceremony as heavyweight for the small manifest involved.

The exact build host, commands, outputs and exit codes are recorded in
`docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md`.

## Usage

```bash
# One-command rehearsal: generate, sign, verify, tamper, expect rejection.
bash tools/release-signing/release-signing.sh selftest

# Explicit flow with a state directory outside the repository.
export AG_SIGN_STATE_DIR=/tmp/ag-release-signing-test
bash tools/release-signing/release-signing.sh genkey
bash tools/release-signing/release-signing.sh sign /path/to/installers.sha256
bash tools/release-signing/release-signing.sh verify /path/to/installers.sha256
```

`verify` exits 0 only when the signature matches the exact manifest bytes;
it exits 1 on a mismatch, a missing file or a missing key. The default state
directory is `${TMPDIR:-/tmp}/ag-release-signing-test`.

## Opt-in verifier (`scripts/check-release-signature.sh`)

This is a separate, **opt-in** checker that verifies a signature produced by
`release-signing.sh` against a TEST public key. It is deliberately not wired
into CI, not called by the installers and not a release-authenticity check.
Every run prints `TEST KEY ONLY - NOT A PRODUCTION SIGNATURE` before any
verdict. It never invents a trust anchor: a successful run proves only that a
throwaway local key signed those exact manifest bytes.

Inputs (environment variables, with positional arguments `$1`, `$2`, `$3` as
fallbacks):

- `AG_TEST_RELEASE_PUBKEY` (`$1`) - path to the TEST public key
  (`test-release.pub` from `genkey`).
- `AG_SIGNED_MANIFEST` (`$2`) - path to the signed checksum manifest.
- `AG_MANIFEST_SIGNATURE` (`$3`) - path to the raw signature; defaults to
  `<manifest>.sig`, which is what `release-signing.sh sign` produces.

Outcomes (the `RESULT:` line is the machine-readable verdict):

- `PASS` (exit 0): key, manifest and signature were supplied and OpenSSL
  verified the exact bytes. Labeled a rehearsal pass, not a signed release.
- `SKIP` (exit 0): no key or no manifest was supplied. Absence is **never**
  PASS. The message names the missing input.
- `FAIL` (exit 1): a required file is missing, `openssl` is unavailable, or
  the signature does not match the manifest (tampering or wrong key). The
  OpenSSL error is included.

Because exit 0 means PASS **or** SKIP, callers must read the `RESULT:` line,
not the exit code, as evidence. This mirrors the outcome discipline of
`scripts/check-dkim-independent.sh`.

```bash
export AG_SIGN_STATE_DIR=/tmp/opencode/ag-sign-test-state
export AG_TEST_RELEASE_PUBKEY="$AG_SIGN_STATE_DIR/test-release.pub"
export AG_SIGNED_MANIFEST=/tmp/opencode/ag-sign-installers-test.sha256
bash tools/release-signing/release-signing.sh genkey
bash tools/release-signing/release-signing.sh sign "$AG_SIGNED_MANIFEST"
bash scripts/check-release-signature.sh                 # PASS (test key only)
sed -i '1s/^./0/' "$AG_SIGNED_MANIFEST"
bash scripts/check-release-signature.sh                 # FAIL (tampered)
```

A real run of this exact chain is recorded in
`docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md`.

## Safety invariants

- The script refuses any `AG_SIGN_STATE_DIR` that resolves inside the
  repository working tree; no key material is ever written to a tracked path.
- `umask 077` is set before any file is created. Private key files are mode
  `0600`, the state directory is mode `0700`, and private key bytes are never
  printed.
- No network access, no CI secrets, no GitHub token and no production key.
- Every invocation prints a banner identifying the run as test-only.
- `selftest` uses a fresh `mktemp -d` directory and removes it on success.

## Limitations

- A test key has no trust value: anyone can generate one. Authenticity comes
  from the offline ceremony and two out-of-band publication channels described
  in `docs/security/RELEASE_KEY_CEREMONY.md`, not from this script.
- The signature covers the manifest bytes only. It does not verify the two
  installer scripts directly; that remains the fail-closed
  `AG_INSTALLER_CHECKSUMS` interface.
- Neither this script nor the opt-in `scripts/check-release-signature.sh`
  implements the installer-side verification from RFC-0030 section 4.5, and
  neither publishes assets. They rehearse the key operations so the human
  ceremony can be reviewed before a production key is generated.
