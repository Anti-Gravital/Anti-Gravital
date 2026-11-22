#!/usr/bin/env bash
# Anti-Gravital installer (Linux / macOS).
#
# Verifies the Rust toolchain, builds the workspace in release mode, and
# installs the `ag` binary into the user's Cargo bin directory.
#
# Security (ADR-0009 rule 4): do not pipe this script from the network. Follow
# docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# Out-of-band integrity: set AG_INSTALLER_CHECKSUMS to a SHA-256 manifest obtained
# independently of this repository. When set, this script verifies its own bytes
# against that manifest and aborts before any build or install action on a
# missing file, a missing entry or a mismatch. When unset, it warns and keeps the
# documented development fallback; no in-repository file is treated as a trust
# anchor. See docs/security/INSTALLATION_INTEGRITY.md for the limits.
#
# Usage: bash install.sh

set -euo pipefail

REQUIRED_RUST="1.99.1"

log() { printf '[ag-install] %s\n' "$1"; }

sha256_of() {
    local file=$1
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        return 1
    fi
}

verify_self_integrity() {
    local manifest=${AG_INSTALLER_CHECKSUMS:-}
    local script_path=${BASH_SOURCE[0]}
    local script_name expected actual

    if [[ -z "$manifest" ]]; then
        log "WARNING: AG_INSTALLER_CHECKSUMS is unset; no out-of-band integrity anchor. See docs/security/INSTALLATION_INTEGRITY.md."
        return 0
    fi
    if [[ ! -f "$manifest" ]]; then
        log "Integrity manifest not found: ${manifest}"
        exit 1
    fi

    script_name="$(basename "$script_path")"
    expected="$(awk -v name="$script_name" '$2 == name { print $1; exit }' "$manifest")"
    if [[ -z "$expected" ]]; then
        log "No checksum entry for ${script_name} in ${manifest}."
        exit 1
    fi

    if ! actual="$(sha256_of "$script_path")"; then
        log "No SHA-256 tool available (sha256sum or shasum required)."
        exit 1
    fi
    if [[ "$actual" != "$expected" ]]; then
        log "Integrity check failed for ${script_name}: expected ${expected}, got ${actual}."
        exit 1
    fi
    log "Integrity verified for ${script_name} against ${manifest}."
}

version_at_least() {
    local current=$1 required=$2
    local current_major current_minor current_patch
    local required_major required_minor required_patch

    IFS=. read -r current_major current_minor current_patch <<< "$current"
    IFS=. read -r required_major required_minor required_patch <<< "$required"
    current_patch=${current_patch%%-*}
    required_patch=${required_patch%%-*}

    (( current_major > required_major )) ||
        (( current_major == required_major && current_minor > required_minor )) ||
        (( current_major == required_major && current_minor == required_minor && current_patch >= required_patch ))
}

verify_self_integrity

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    log "Rust toolchain not found."
    log "Install Rust with rustup from https://rustup.rs, restart your shell, and retry."
    exit 1
fi

INSTALLED_RUST="$(rustc --version | awk '{print $2}')"
log "Detected Rust ${INSTALLED_RUST} (minimum required: ${REQUIRED_RUST})."

if ! version_at_least "$INSTALLED_RUST" "$REQUIRED_RUST"; then
    log "Rust ${REQUIRED_RUST} or newer is required. Run 'rustup update stable' and retry."
    exit 1
fi

log "Building the workspace in release mode..."
cargo build --workspace --release

log "Installing the ag CLI into ~/.cargo/bin ..."
cargo install --path crates/ag-cli --locked

log "Installation complete."
log "Ensure ~/.cargo/bin is on your PATH, then run: ag --help"
