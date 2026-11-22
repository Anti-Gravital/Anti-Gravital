#!/usr/bin/env bash
# Anti-Gravital release-signing rehearsal script. TEST ONLY.
#
# This script orchestrates the OpenSSL CLI to exercise the release-manifest
# signing flow described in RFC-0030 sections 4.4 and 4.5 with a locally
# generated Ed25519 test key. It does NOT generate, publish or reference a
# production release key, and it does NOT modify the release workflow.
# No cryptographic primitive is implemented here: key generation, signing and
# verification are delegated to `openssl` (see README.md for provenance).
#
# Commands:
#   genkey               Generate a test-only Ed25519 keypair in a temp dir.
#   sign <manifest>      Sign a checksum manifest with the test private key.
#   verify <manifest>    Verify a manifest against the test public key.
#   selftest             Generate, sign, verify, tamper and confirm rejection.
#
# Safety invariants:
#   - key material is written only outside the repository working tree;
#   - private keys are created with umask 077 and never printed;
#   - no network access and no CI secrets are used;
#   - every invocation prints a test-only banner.
#
# Environment:
#   AG_SIGN_STATE_DIR   State directory (default ${TMPDIR:-/tmp}/ag-release-signing-test).

set -euo pipefail

umask 077

PROGRAM="release-signing.sh"
STATE_DIR="${AG_SIGN_STATE_DIR:-${TMPDIR:-/tmp}/ag-release-signing-test}"
KEY_FILE="$STATE_DIR/test-release.key"
PUB_FILE="$STATE_DIR/test-release.pub"
REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel 2>/dev/null || true)"

log() { printf '[signing] %s\n' "$*"; }
die() { printf '[signing] ERROR: %s\n' "$*" >&2; exit 1; }

banner() {
    printf '[signing] TEST ONLY - NOT A PRODUCTION RELEASE KEY - RFC-0030 REHEARSAL\n'
}

# Refuse to place key material anywhere inside the repository working tree.
assert_state_dir_outside_repo() {
    local abs
    abs="$(realpath -m -- "$STATE_DIR" 2>/dev/null || printf '%s' "$STATE_DIR")"
    if [[ -n "$REPO_ROOT" ]]; then
        local root_abs
        root_abs="$(realpath -m -- "$REPO_ROOT" 2>/dev/null || printf '%s' "$REPO_ROOT")"
        if [[ "$abs" == "$root_abs" || "$abs" == "$root_abs"/* ]]; then
            die "state directory must be outside the repository: $abs"
        fi
    fi
}

assert_openssl() {
    command -v openssl >/dev/null 2>&1 || die "openssl CLI not found in PATH"
}

fingerprint_of() {
    openssl pkey -pubin -in "$1" -outform DER 2>/dev/null \
        | openssl dgst -sha256 2>/dev/null | awk '{print $NF}'
}

do_genkey() {
    assert_state_dir_outside_repo
    assert_openssl
    if [[ -e "$KEY_FILE" || -e "$PUB_FILE" ]]; then
        die "test key already exists in $STATE_DIR; choose a new AG_SIGN_STATE_DIR"
    fi
    mkdir -p "$STATE_DIR"
    chmod 700 "$STATE_DIR"
    openssl genpkey -algorithm ED25519 -out "$KEY_FILE" >/dev/null 2>&1
    openssl pkey -in "$KEY_FILE" -pubout -out "$PUB_FILE" >/dev/null 2>&1
    chmod 600 "$KEY_FILE"
    chmod 644 "$PUB_FILE"
    log "generated test-only Ed25519 keypair in $STATE_DIR"
    log "test public key fingerprint (SHA-256 of DER SPKI): $(fingerprint_of "$PUB_FILE")"
    log "private key is never printed and must never leave this temp directory"
}

do_sign() {
    local manifest="${1:-}" sig
    [[ -n "$manifest" ]] || die "usage: $PROGRAM sign <manifest>"
    [[ -f "$manifest" ]] || die "manifest not found: $manifest"
    assert_state_dir_outside_repo
    assert_openssl
    [[ -f "$KEY_FILE" ]] || die "no test key in $STATE_DIR; run 'genkey' first"
    sig="$manifest.sig"
    openssl pkeyutl -sign -inkey "$KEY_FILE" -rawin -in "$manifest" -out "$sig" >/dev/null 2>&1 \
        || die "signing failed"
    log "signed $manifest -> $sig (raw Ed25519 signature, test key only)"
}

do_verify() {
    local manifest="${1:-}" sig="${2:-}" out
    [[ -n "$manifest" ]] || die "usage: $PROGRAM verify <manifest> [signature]"
    [[ -f "$manifest" ]] || die "manifest not found: $manifest"
    [[ -n "$sig" ]] || sig="$manifest.sig"
    [[ -f "$sig" ]] || die "signature not found: $sig"
    assert_state_dir_outside_repo
    assert_openssl
    [[ -f "$PUB_FILE" ]] || die "no test public key in $STATE_DIR; run 'genkey' first"
    if out="$(openssl pkeyutl -verify -pubin -inkey "$PUB_FILE" -rawin -in "$manifest" -sigfile "$sig" 2>&1)"; then
        log "VERIFY OK: $out"
        return 0
    fi
    printf '[signing] VERIFY FAILED: %s\n' "$out" >&2
    return 1
}

do_selftest() {
    local tmp manifest sig
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/ag-release-signing-selftest.XXXXXX")"
    STATE_DIR="$tmp/state"
    KEY_FILE="$STATE_DIR/test-release.key"
    PUB_FILE="$STATE_DIR/test-release.pub"
    manifest="$tmp/installers-test.sha256"
    sig="$manifest.sig"

    printf '[signing] ============ release-signing selftest (TEST ONLY) ============\n'
    log "OpenSSL: $(openssl version)"
    log "state dir: $STATE_DIR (outside repository, removed at exit)"
    printf '[signing] [1/6] generate local test Ed25519 keypair\n'
    do_genkey
    printf '[signing] [2/6] build a sample sha256sum-format manifest\n'
    printf '%064d  %s\n' 0 install.sh > "$manifest"
    printf '%064d  %s\n' 0 install.ps1 >> "$manifest"
    cat "$manifest" | sed 's/^/[signing]   /'
    printf '[signing] [3/6] sign the manifest\n'
    do_sign "$manifest"
    printf '[signing] [4/6] verify the unmodified manifest (must succeed)\n'
    do_verify "$manifest" "$sig" || die "selftest failed: valid signature was rejected"
    printf '[signing] [5/6] tamper with the manifest bytes\n'
    printf '%064d  %s\n' 1 install.sh > "$manifest"
    printf '%064d  %s\n' 0 install.ps1 >> "$manifest"
    printf '[signing] [6/6] verify the tampered manifest (must be rejected)\n'
    if tamper_out="$(do_verify "$manifest" "$sig" 2>&1)"; then
        die "selftest failed: tampered manifest was accepted"
    else
        printf '%s\n' "$tamper_out" | sed 's/^/[signing]   /'
    fi
    log "VERIFY FAILED as expected for the tampered manifest"
    printf '[signing] ============ RESULT: PASS (test key only; no production key exists) ============\n'
    rm -rf "$tmp"
}

main() {
    banner
    local cmd="${1:-}"
    case "$cmd" in
        genkey)   do_genkey ;;
        sign)     shift; do_sign "$@" ;;
        verify)   shift; do_verify "$@" ;;
        selftest) do_selftest ;;
        *)
            printf 'Usage: %s {genkey|sign <manifest>|verify <manifest> [signature]|selftest}\n' "$PROGRAM"
            exit 2
            ;;
    esac
}

main "$@"
