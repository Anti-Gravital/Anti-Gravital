#!/usr/bin/env bash
# Opt-in local verifier for a TEST-KEY-only release-manifest signature (#141).
#
# It verifies a raw Ed25519 signature produced by
# tools/release-signing/release-signing.sh over a checksum manifest, using a
# throwaway TEST public key. It has deliberately NO production trust value:
# every run prints a test-only banner before any verdict.
#
# Outcomes (the RESULT: line is the machine-readable verdict):
#   PASS  a key, manifest and signature were supplied and OpenSSL verified the
#         exact manifest bytes. This is a REHEARSAL PASS, not a signed release.
#   SKIP  no key or no manifest was supplied. Absence is NEVER PASS.
#   FAIL  inputs were supplied but a file is missing, OpenSSL is unavailable, or
#         the signature does not match the manifest bytes (tampering/wrong key).
#
# Exit status is 0 for PASS and SKIP, 1 for FAIL. A SKIP is not a pass: callers
# must not read exit 0 as release-authenticity evidence.
#
# Inputs, by environment variable or positional argument (in this order):
#   AG_TEST_RELEASE_PUBKEY / $1  TEST public key (test-release.pub)
#   AG_SIGNED_MANIFEST     / $2  signed checksum manifest
#   AG_MANIFEST_SIGNATURE  / $3  raw signature (default: <manifest>.sig)
#
# No network, no CI secrets, no private key. See tools/release-signing/README.md
# and docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md. Issue #141 stays open.

set -euo pipefail

BANNER="[verify] TEST KEY ONLY - NOT A PRODUCTION SIGNATURE - RFC-0030 REHEARSAL"
report() { printf '%s\n' "$*"; }

report "$BANNER"

repo_root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel 2>/dev/null || printf 'unknown')"
report "repo.root=$repo_root"

pubkey="${AG_TEST_RELEASE_PUBKEY:-${1:-}}"
manifest="${AG_SIGNED_MANIFEST:-${2:-}}"
signature="${AG_MANIFEST_SIGNATURE:-${3:-}}"

# Absence is an explicit SKIP, never a PASS.
if [[ -z "$pubkey" && -z "$manifest" ]]; then
    report "RESULT: SKIP reason=no-inputs-supplied detail=supply AG_TEST_RELEASE_PUBKEY and AG_SIGNED_MANIFEST to verify; absence is never PASS"
    exit 0
fi
if [[ -z "$pubkey" || -z "$manifest" ]]; then
    missing=""
    if [[ -z "$pubkey" ]]; then missing="${missing}AG_TEST_RELEASE_PUBKEY "; fi
    if [[ -z "$manifest" ]]; then missing="${missing}AG_SIGNED_MANIFEST "; fi
    report "RESULT: SKIP reason=incomplete-inputs detail=missing: ${missing}- absent inputs are never PASS"
    exit 0
fi

[[ -n "$signature" ]] || signature="${manifest}.sig"
report "input.pubkey=$pubkey"
report "input.manifest=$manifest"
report "input.signature=$signature"

if ! command -v openssl >/dev/null 2>&1; then
    report "RESULT: FAIL reason=openssl-unavailable detail=inputs supplied but openssl CLI not found in PATH"
    exit 1
fi

for f in "$pubkey" "$manifest" "$signature"; do
    if [[ ! -f "$f" ]]; then
        report "RESULT: FAIL reason=input-file-missing detail=no such file: $f"
        exit 1
    fi
done

fingerprint="$(openssl pkey -pubin -in "$pubkey" -outform DER 2>/dev/null \
    | openssl dgst -sha256 2>/dev/null | awk '{print $NF}' || true)"
if [[ -n "$fingerprint" ]]; then
    report "key.test_fingerprint_sha256=$fingerprint"
fi
report "tool.openssl=$(openssl version 2>/dev/null)"

if verify_out="$(openssl pkeyutl -verify -pubin -inkey "$pubkey" -rawin \
    -in "$manifest" -sigfile "$signature" 2>&1)"; then
    report "signature.verified=yes"
    report "$BANNER"
    report "RESULT: PASS reason=test-signature-verified detail=TEST KEY ONLY - NOT A PRODUCTION SIGNATURE; rehearsal pass only, not release authenticity"
    exit 0
fi

report "RESULT: FAIL reason=signature-mismatch detail=signature does not verify against these exact manifest bytes (tampering or wrong key/signature)"
report "--- openssl output (start) ---"
printf '%s\n' "$verify_out"
report "--- openssl output (end) ---"
exit 1
