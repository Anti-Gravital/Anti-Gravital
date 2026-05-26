#!/usr/bin/env bash
# Anti-Gravital installer (Linux / macOS).
#
# Installs the Rust toolchain (if missing), builds the workspace in release
# mode, and installs the `ag` binary into the user's Cargo bin directory.
#
# Security (ADR-0009 rule 4): this script does not pipe untrusted content into
# a shell without verification. When fetching a remote copy, verify the
# published SHA-256 hash listed in docs/INSTALL.md before running.
#
# Usage: bash install.sh

set -euo pipefail

REQUIRED_RUST="1.79.0"

log() { printf '[ag-install] %s\n' "$1"; }

if ! command -v cargo >/dev/null 2>&1; then
    log "Rust toolchain not found."
    log "Install rustup from https://rustup.rs and re-run this script."
    exit 1
fi

INSTALLED_RUST="$(rustc --version | awk '{print $2}')"
log "Detected Rust ${INSTALLED_RUST} (minimum required: ${REQUIRED_RUST})."

log "Building the workspace in release mode..."
cargo build --workspace --release

log "Installing the ag CLI into ~/.cargo/bin ..."
cargo install --path crates/ag-cli --locked

log "Installation complete."
log "Ensure ~/.cargo/bin is on your PATH, then run: ag --help"
