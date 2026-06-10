#!/usr/bin/env bash
# Anti-Gravital installer (Linux / macOS).
#
# Verifies the Rust toolchain, builds the workspace in release mode, and
# installs the `ag` binary into the user's Cargo bin directory.
#
# Security (ADR-0009 rule 4): do not pipe this script from the network. Follow
# docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# Usage: bash install.sh

set -euo pipefail

REQUIRED_RUST="1.95.0"

log() { printf '[ag-install] %s\n' "$1"; }

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
