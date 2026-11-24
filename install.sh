#!/usr/bin/env bash
# Anti-Gravital installer (Linux / macOS).
#
# Two channels:
#   source (default) - verify the Rust toolchain, build the workspace in release
#                      mode and install the `ag` binary into Cargo's user bin.
#   alpha            - download a prebuilt `ag` binary and its checksum manifest
#                      from a configurable release location, verify the SHA-256
#                      checksum BEFORE installing, and refuse on any mismatch.
#
# Security (ADR-0009 rule 4): do not pipe this script from the network. Follow
# docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# The alpha channel uses SHA-256 CHECKSUM verification only. No release signing
# key exists and no signature artifact is published; this installer never claims
# signature or authenticity verification. The base URL / release tag you supply
# is the trust boundary. See docs/security/INSTALLATION_INTEGRITY.md.
#
# Out-of-band integrity for this script: set AG_INSTALLER_CHECKSUMS to a SHA-256
# manifest obtained independently of this repository. When set, this script
# verifies its own bytes against that manifest and aborts before any build,
# download or install action on a missing file, a missing entry or a mismatch.
# When unset, a development-only local fallback is used; it is never a trust
# anchor. See docs/security/INSTALLATION_INTEGRITY.md for the limits.
#
# Usage:
#   bash install.sh                                        # source channel
#   AG_INSTALLER_CHANNEL=source bash install.sh           # explicit source
#   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL=... bash install.sh
#
# Alpha configuration:
#   AG_INSTALLER_CHANNEL=alpha      select the prebuilt-binary channel
#   AG_INSTALLER_BASE_URL=<url>     release assets directory (file:// or http(s))
#   AG_INSTALLER_RELEASE_TAG=<tag>  alternative: GitHub release tag
#   AG_INSTALLER_TARGET=<triple>    override auto-detected target triple
#   AG_INSTALLER_MANIFEST=<name>    manifest filename (default: checksums.txt)
#   AG_INSTALLER_BIN_DIR=<dir>      destination (default: ~/.cargo/bin)
#   AG_INSTALLER_BINARY_CHECKSUMS=<path>  local out-of-band binary manifest;
#                                  when set, checksums.txt is NOT downloaded

set -euo pipefail

REQUIRED_RUST="1.99.1"

CHANNEL="${AG_INSTALLER_CHANNEL:-source}"
BASE_URL="${AG_INSTALLER_BASE_URL:-}"
RELEASE_TAG="${AG_INSTALLER_RELEASE_TAG:-}"
TARGET="${AG_INSTALLER_TARGET:-}"
MANIFEST_NAME="${AG_INSTALLER_MANIFEST:-checksums.txt}"
BIN_DIR="${AG_INSTALLER_BIN_DIR:-${CARGO_HOME:-$HOME/.cargo}/bin}"
ALPHA_TMP=""

log() { printf '[ag-install] %s\n' "$1"; }

cleanup() {
    if [[ -n "$ALPHA_TMP" && -d "$ALPHA_TMP" ]]; then
        rm -rf "$ALPHA_TMP"
    fi
}
trap cleanup EXIT

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
        log "WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md."
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

resolve_base_url() {
    if [[ -n "$BASE_URL" ]]; then
        printf '%s\n' "${BASE_URL%/}"
        return 0
    fi
    if [[ -n "$RELEASE_TAG" ]]; then
        printf 'https://github.com/Anti-Gravital/Anti-Gravital/releases/download/%s\n' "$RELEASE_TAG"
        return 0
    fi
    return 1
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            log "Unsupported architecture for alpha channel: ${arch}."
            return 1
            ;;
    esac
    case "$os" in
        Linux) printf '%s-unknown-linux-gnu\n' "$arch" ;;
        Darwin) printf '%s-apple-darwin\n' "$arch" ;;
        *)
            log "Unsupported operating system for alpha channel: ${os}."
            return 1
            ;;
    esac
}

download() {
    local url=$1 dest=$2
    # file:// is supported for local/offline testing and air-gapped mirrors.
    if [[ "$url" == file://* ]]; then
        cp "${url#file://}" "$dest"
        return $?
    fi
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$dest" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url"
    else
        log "No download tool available (curl or wget required)."
        return 1
    fi
}

install_alpha() {
    local base manifest_url binary_url binary_name manifest binary expected actual
    local binary_manifest=${AG_INSTALLER_BINARY_CHECKSUMS:-}

    if ! base="$(resolve_base_url)"; then
        log "AG_INSTALLER_CHANNEL=alpha requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG."
        exit 1
    fi
    if [[ -z "$TARGET" ]]; then
        if ! TARGET="$(detect_target)"; then
            log "Set AG_INSTALLER_TARGET to the target triple published by the release."
            exit 1
        fi
    fi

    binary_name="ag-${TARGET}"
    ALPHA_TMP="$(mktemp -d)"
    binary="$ALPHA_TMP/$binary_name"
    binary_url="$base/$binary_name"

    log "Alpha channel: checksum verification only (no signing key, no signature)."
    if [[ -n "$binary_manifest" ]]; then
        if [[ ! -f "$binary_manifest" ]]; then
            log "Binary checksum manifest not found: ${binary_manifest}."
            exit 1
        fi
        manifest="$binary_manifest"
        log "Alpha channel: using local binary checksum manifest ${manifest} (out-of-band)."
    else
        manifest="$ALPHA_TMP/$MANIFEST_NAME"
        manifest_url="$base/$MANIFEST_NAME"
        log "Alpha channel: downloading checksum manifest from ${manifest_url}"
        if ! download "$manifest_url" "$manifest"; then
            log "Could not download the checksum manifest from ${manifest_url}."
            exit 1
        fi
    fi
    log "Alpha channel: downloading binary from ${binary_url}"
    if ! download "$binary_url" "$binary"; then
        log "Could not download the binary from ${binary_url}."
        exit 1
    fi

    expected="$(awk -v name="$binary_name" '$2 == name { print $1; exit }' "$manifest")"
    if [[ -z "$expected" ]]; then
        log "No checksum entry for ${binary_name} in ${MANIFEST_NAME}."
        exit 1
    fi
    if ! actual="$(sha256_of "$binary")"; then
        log "No SHA-256 tool available (sha256sum or shasum required)."
        exit 1
    fi
    if [[ "$actual" != "$expected" ]]; then
        log "Checksum mismatch for ${binary_name}: expected ${expected}, got ${actual}."
        exit 1
    fi
    log "Checksum verified for ${binary_name}."

    mkdir -p "$BIN_DIR"
    cp "$binary" "$BIN_DIR/ag"
    chmod 0755 "$BIN_DIR/ag"
    log "Installed ag to ${BIN_DIR}/ag."
    log "Installation complete."
    log "Ensure ${BIN_DIR} is on your PATH, then run: ag --version"
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

if [[ "$CHANNEL" == "alpha" ]]; then
    install_alpha
    exit 0
fi
if [[ "$CHANNEL" != "source" ]]; then
    log "Unknown AG_INSTALLER_CHANNEL '${CHANNEL}' (expected 'source' or 'alpha')."
    exit 1
fi

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

log "Building the workspace in release mode (this can take ~15-20 min on a cold cache)..."
cargo build --locked --workspace --release

log "Installing the ag CLI into $BIN_DIR (a second release build may take several minutes) ..."
cargo install --path crates/ag-cli --locked

log "Installation complete."
log "Ensure $BIN_DIR is on your PATH, then run: ag --help"
