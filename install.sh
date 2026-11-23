#!/usr/bin/env bash
# Anti-Gravital installer (Linux / macOS).
#
# Default channel: release (prebuilt `ag` binary + SHA-256 checksum verification).
#   - `alpha` is accepted as a deprecated alias of `release` (migration notice).
#   - AG_INSTALLER_CHANNEL=source keeps the original Rust build path intact.
#   - There is NO automatic fallback to `source`: a failed download or checksum
#     check exits non-zero and explicitly tells the caller how to opt into source.
#
# Security (ADR-0009 rule 4): do not pipe this script from the network. Follow
# docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# This installer verifies SHA-256 CHECKSUMS only. No release signing key exists
# and no signature artifact is published; it never claims signature or
# authenticity verification. The base URL / release tag you supply is the trust
# boundary. See docs/security/INSTALLATION_INTEGRITY.md.
#
# Out-of-band integrity for this script: set AG_INSTALLER_CHECKSUMS to a SHA-256
# manifest obtained independently of this repository. When set, this script
# verifies its own bytes against that manifest and aborts before any build,
# download or install action on a missing file, entry or mismatch. When unset a
# warning is printed (never silenced); there is no trust anchor in that case.
#
# Progress display:
#   AG_INSTALLER_PROGRESS=auto|tui|headless (default: auto).
#   `auto` selects headless when CI is set, stdout is not a TTY, or TERM is
#   dumb/unset. NO_COLOR only disables colour in TUI; it does not force headless.
#   Headless emits deterministic ASCII lines (no timestamps, no ANSI, no local
#   paths):
#     ag-install <seq> <stage> <state> bytes=<n> total=<n|-> [code=<E_...>]
#   Progress bytes are emitted only when observed by the download tool; no
#   invented progress and no ETA when the total size is unknown.
#
# Usage:
#   bash install.sh                                  # release (prebuilt, default)
#   AG_INSTALLER_CHANNEL=release bash install.sh     # explicit release
#   AG_INSTALLER_CHANNEL=alpha bash install.sh       # deprecated alias of release
#   AG_INSTALLER_CHANNEL=source bash install.sh      # build from source (explicit)
#
# Configuration (all optional):
#   AG_INSTALLER_BASE_URL=<url>            assets directory (file:// or http(s))
#   AG_INSTALLER_RELEASE_TAG=<tag>         alternative: GitHub release tag
#   AG_INSTALLER_TARGET=<triple>           override auto-detected target triple
#   AG_INSTALLER_MANIFEST=<name>           manifest filename (default checksums.txt)
#   AG_INSTALLER_BIN_DIR=<dir>             destination (default ~/.cargo/bin)
#   AG_INSTALLER_BINARY_CHECKSUMS=<path>   local out-of-band binary manifest
#   AG_INSTALLER_CHECKSUMS=<path>          local out-of-band installer manifest
#   AG_INSTALLER_PROGRESS=<mode>           auto|tui|headless
#   NO_COLOR=1                             disable colour in TUI

set -euo pipefail

# Business constant (S-3): canonical GitHub release-assets base URL. Overridable
# through AG_INSTALLER_BASE_URL / AG_INSTALLER_RELEASE_TAG; contains no secret.
DEFAULT_RELEASE_BASE_URL="https://github.com/Anti-Gravital/Anti-Gravital/releases/download"
REQUIRED_RUST="1.99.1"

CHANNEL="${AG_INSTALLER_CHANNEL:-release}"
BASE_URL="${AG_INSTALLER_BASE_URL:-}"
RELEASE_TAG="${AG_INSTALLER_RELEASE_TAG:-}"
TARGET="${AG_INSTALLER_TARGET:-}"
MANIFEST_NAME="${AG_INSTALLER_MANIFEST:-checksums.txt}"
BIN_DIR="${AG_INSTALLER_BIN_DIR:-${CARGO_HOME:-$HOME/.cargo}/bin}"
BINARY_CHECKSUMS="${AG_INSTALLER_BINARY_CHECKSUMS:-}"
SELF_CHECKSUMS="${AG_INSTALLER_CHECKSUMS:-}"
PROGRESS_REQUEST="${AG_INSTALLER_PROGRESS:-auto}"
REDUCE_MOTION="${AG_INSTALLER_REDUCE_MOTION:-}"

# Progress / presentation state.
PROGRESS_MODE="headless"
TUI_STARTED=0
FRAME_HEIGHT=9
SEQ=0
CURRENT_STAGE=0
STATUS_TEXT=""
NOTICE_TEXT=""
SPINNER=('|' '/' '-' '\')
SPINNER_PID=""
SPIN_STOP=""
C_OK="" C_ERR="" C_DIM="" C_BOLD="" C_RESET=""
VERSION_RESOLVED=""
TAG_RESOLVED=""
VERSION_MISMATCH="false"
TARGET_ERROR=""
ALPHA_TMP=""

STAGE_NAMES=("" "Resolver versión" "Detectar plataforma" "Descargar" "Verificar SHA-256" "Instalar" "Listo")
STAGE_STATE=("" "pending" "pending" "pending" "pending" "pending" "pending")
STAGE_DETAIL=("" "PENDIENTE" "PENDIENTE" "PENDIENTE" "PENDIENTE" "PENDIENTE" "PENDIENTE")

# Human-readable messages keep the historical [ag-install] prefix.
log() { printf '[ag-install] %s\n' "$1"; }

cleanup() {
    if [[ "$PROGRESS_MODE" == "tui" && "$TUI_STARTED" == "1" ]]; then
        printf '\033[?25h'
    fi
    if [[ -n "$ALPHA_TMP" && -d "$ALPHA_TMP" ]]; then
        rm -rf "$ALPHA_TMP"
    fi
}

on_signal() {
    local code
    case "$1" in
        INT) code=130 ;;
        TERM) code=143 ;;
        *) code=1 ;;
    esac
    if [[ -n "$SPIN_STOP" ]]; then
        : > "$SPIN_STOP"
    fi
    if [[ -n "$SPINNER_PID" ]]; then
        if wait "$SPINNER_PID"; then :; fi
        SPINNER_PID=""
    fi
    cleanup
    exit "$code"
}

trap 'on_signal INT' INT
trap 'on_signal TERM' TERM
trap cleanup EXIT

sha256_of() {
    local file=$1
    if [[ -n "$(type -P sha256sum)" ]]; then
        sha256sum "$file" | awk '{print $1}'
    elif [[ -n "$(type -P shasum)" ]]; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        return 1
    fi
}

# Percent-encode a value as a single URL path component (S-4). All values we
# construct are additionally validated against closed sets before encoding.
urlencode() {
    local s="$1" out="" i c hex
    for (( i = 0; i < ${#s}; i++ )); do
        c="${s:i:1}"
        case "$c" in
            [A-Za-z0-9._~-]) out+="$c" ;;
            *)
                printf -v hex '%%%02X' "'$c"
                out+="$hex"
                ;;
        esac
    done
    printf '%s' "$out"
}

resolve_progress_mode() {
    local m="$PROGRESS_REQUEST"
    case "$m" in
        headless|tui) printf '%s\n' "$m"; return 0 ;;
        auto|"") ;;
        *) printf 'headless\n'; return 0 ;;
    esac
    if [[ -n "${CI:-}" && "$CI" != "false" && "$CI" != "0" ]]; then printf 'headless\n'; return 0; fi
    if [[ ! -t 1 ]]; then printf 'headless\n'; return 0; fi
    if [[ -z "${TERM:-}" || "$TERM" == "dumb" ]]; then printf 'headless\n'; return 0; fi
    printf 'tui\n'
}

stage_id() {
    case "$1" in
        1) printf 'resolve_version' ;;
        2) printf 'detect_platform' ;;
        3) printf 'download' ;;
        4) printf 'verify' ;;
        5) printf 'install' ;;
        6) printf 'ready' ;;
        *) printf 'unknown' ;;
    esac
}

sym_for_state() {
    case "$1" in
        pending) printf '○' ;;
        started|progress) printf '◉' ;;
        succeeded) printf 'OK' ;;
        failed) printf 'ERR' ;;
        skipped) printf '-' ;;
        *) printf '?' ;;
    esac
}

tui_init() {
    local i
    printf '\033[?25l'
    printf 'Anti-Gravital - instalacion rapida\n'
    for i in 1 2 3 4 5 6; do
        printf '  %d ○ %s - PENDIENTE\n' "$i" "${STAGE_NAMES[$i]}"
    done
    printf 'Estado: iniciando\n'
    printf 'Aviso: %s\n' "$NOTICE_TEXT"
    TUI_STARTED=1
}

tui_render() {
    local active="$1" spin="$2"
    local i sym detail line
    printf '\033[%dA' "$FRAME_HEIGHT"
    printf 'Anti-Gravital - instalacion rapida\r\033[K\n'
    for i in 1 2 3 4 5 6; do
        sym="$(sym_for_state "${STAGE_STATE[$i]}")"
        if [[ "$i" == "$active" && -n "$spin" ]]; then sym="$spin"; fi
        line="  $i ${C_BOLD}${sym}${C_RESET} ${STAGE_NAMES[$i]} - ${STAGE_DETAIL[$i]}"
        printf '%s\r\033[K\n' "${line:0:120}"
    done
    printf 'Estado: %s\r\033[K\n' "${STATUS_TEXT:0:70}"
    printf 'Aviso: %s\r\033[K\n' "${NOTICE_TEXT:0:70}"
}

# Emit a progress event. Headless output is a stable, deterministic contract.
# `detail` is presentation-only and is never printed in headless mode.
emit() {
    local stage_num="$1" state="$2" bytes="${3:-0}" total="${4:--}" code="${5:-}" detail="${6:-}"
    local line
    SEQ=$((SEQ + 1))
    CURRENT_STAGE="$stage_num"
    if [[ "$PROGRESS_MODE" == "headless" ]]; then
        line="ag-install $SEQ $(stage_id "$stage_num") $state bytes=$bytes total=$total"
        if [[ -n "$code" ]]; then line="$line code=$code"; fi
        printf '%s\n' "$line"
        return 0
    fi
    STAGE_STATE[$stage_num]="$state"
    STAGE_DETAIL[$stage_num]="$detail"
    STATUS_TEXT="$detail"
    tui_render "$stage_num" ""
}

spin_start() {
    [[ "$PROGRESS_MODE" == "tui" && "$TUI_STARTED" == "1" && "$REDUCE_MOTION" != "1" ]] || return 0
    SPIN_STOP="$ALPHA_TMP/.spin-stop"
    rm -f "$SPIN_STOP"
    (
        local i=0
        while [[ ! -e "$SPIN_STOP" ]]; do
            tui_render "$CURRENT_STAGE" "${SPINNER[$i]}"
            i=$(( (i + 1) % ${#SPINNER[@]} ))
            sleep 0.1
        done
        exit 0
    ) &
    SPINNER_PID=$!
}

spin_stop() {
    if [[ -n "$SPINNER_PID" ]]; then
        if [[ -n "$SPIN_STOP" ]]; then : > "$SPIN_STOP"; fi
        wait "$SPINNER_PID"
        SPINNER_PID=""
    fi
}

fail() {
    local code="$1" msg="$2" i
    if [[ "$PROGRESS_MODE" == "tui" && "$TUI_STARTED" == "1" ]]; then
        for (( i = CURRENT_STAGE + 1; i <= 6; i++ )); do
            STAGE_STATE[$i]="skipped"
            STAGE_DETAIL[$i]="OMITIDO"
        done
        STATUS_TEXT="$msg"
        tui_render 0 ""
    fi
    log "$msg"
    case "$code" in
        E_DOWNLOAD_FAILED|E_CHECKSUM_MISMATCH|E_MANIFEST_MISSING|E_VERSION_MISMATCH|E_UNSUPPORTED_TARGET|E_INSTALL_WRITE_FAILED)
            log "Si necesita compilar desde fuente: AG_INSTALLER_CHANNEL=source bash install.sh (coste ~21 min, dos builds)."
            ;;
    esac
    exit 1
}

verify_self_integrity() {
    local manifest="$SELF_CHECKSUMS"
    local script_path=${BASH_SOURCE[0]}
    local script_name expected actual

    if [[ -z "$manifest" ]]; then
        log "WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md."
        return 0
    fi
    if [[ ! -f "$manifest" ]]; then
        log "Integrity manifest not found: <configurado>."
        exit 1
    fi

    script_name="$(basename "$script_path")"
    expected="$(awk -v name="$script_name" '$2 == name { print $1; exit }' "$manifest")"
    if [[ -z "$expected" ]]; then
        log "No checksum entry for ${script_name} in the configured manifest."
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
    log "Integrity verified for ${script_name} against the configured manifest."
}

# Resolve the workspace version with the SAME awk expression as release.yml.
# VERSION (roadmap/framework label) is informational only and never builds a tag.
resolve_version() {
    local version candidate
    if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
        return 1
    fi
    version="$(awk '/^\[workspace\.package\]/{f=1;next} f && /^version = /{gsub(/"/,"",$3);print $3;exit}' "$SCRIPT_DIR/Cargo.toml")"
    if [[ -z "$version" ]] || ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$'; then
        return 1
    fi
    VERSION_RESOLVED="$version"
    TAG_RESOLVED="v${version}"
    VERSION_MISMATCH="false"
    if [[ -f "$SCRIPT_DIR/VERSION" ]]; then
        candidate="$(tr -d '[:space:]' < "$SCRIPT_DIR/VERSION")"
        if [[ -n "$candidate" && "$candidate" != "$version" ]]; then
            VERSION_MISMATCH="true"
        fi
    fi
}

resolve_base_url() {
    if [[ -n "$BASE_URL" ]]; then
        printf '%s\n' "${BASE_URL%/}"
        return 0
    fi
    if [[ -n "$RELEASE_TAG" ]]; then
        printf '%s/%s\n' "$DEFAULT_RELEASE_BASE_URL" "$(urlencode "$RELEASE_TAG")"
        return 0
    fi
    printf '%s/%s\n' "$DEFAULT_RELEASE_BASE_URL" "$(urlencode "$TAG_RESOLVED")"
}

# Closed target map: only the four triples published by release.yml.
validate_target() {
    local t="$1" os
    if [[ "$t" == "x86_64-apple-darwin" ]]; then
        TARGET_ERROR="x86_64-apple-darwin no esta publicado; el unico target macOS es aarch64-apple-darwin."
        return 1
    fi
    case "$t" in
        x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|aarch64-apple-darwin|x86_64-pc-windows-msvc) ;;
        *) TARGET_ERROR="Target '${t}' no esta en el mapa publicado."; return 1 ;;
    esac
    os="$(uname -s)"
    case "$os" in
        Linux)
            case "$t" in
                *-unknown-linux-gnu) return 0 ;;
                x86_64-pc-windows-msvc) TARGET_ERROR="x86_64-pc-windows-msvc no es valido en Linux; este instalador es para Linux/macOS."; return 1 ;;
                x86_64-apple-darwin) TARGET_ERROR="x86_64-apple-darwin no esta publicado; el target macOS es aarch64-apple-darwin."; return 1 ;;
                *) TARGET_ERROR="Target '${t}' no compatible con ${os}."; return 1 ;;
            esac
            ;;
        Darwin)
            case "$t" in
                aarch64-apple-darwin) return 0 ;;
                x86_64-apple-darwin) TARGET_ERROR="x86_64-apple-darwin no esta publicado; el target macOS es aarch64-apple-darwin."; return 1 ;;
                *) TARGET_ERROR="Target '${t}' no compatible con macOS."; return 1 ;;
            esac
            ;;
        *) TARGET_ERROR="Sistema operativo no soportado: ${os}."; return 1 ;;
    esac
}

detect_target() {
    local os arch triple=""
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) TARGET_ERROR="Arquitectura no soportada: ${arch}."; return 1 ;;
    esac
    case "$os" in
        Linux)
            case "$arch" in
                x86_64) triple="x86_64-unknown-linux-gnu" ;;
                aarch64) triple="aarch64-unknown-linux-gnu" ;;
                *) TARGET_ERROR="Arquitectura no soportada: ${arch}."; return 1 ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                aarch64) triple="aarch64-apple-darwin" ;;
                x86_64) TARGET_ERROR="x86_64-apple-darwin no esta publicado; el target macOS es aarch64-apple-darwin."; return 1 ;;
                *) TARGET_ERROR="Arquitectura no soportada: ${arch}."; return 1 ;;
            esac
            ;;
        *) TARGET_ERROR="Sistema operativo no soportado: ${os}."; return 1 ;;
    esac
    printf '%s\n' "$triple"
}

do_download() {
    local url="$1" dest="$2" src
    # file:// is supported for local/offline testing and air-gapped mirrors.
    # Check existence first so a raw `cp` error cannot echo a local user path.
    if [[ "$url" == file://* ]]; then
        src="${url#file://}"
        if [[ ! -f "$src" ]]; then
            return 1
        fi
        cp "$src" "$dest"
        return $?
    fi
    if [[ -n "$(type -P curl)" ]]; then
        curl -fsSL -o "$dest" "$url"
        return $?
    fi
    if [[ -n "$(type -P wget)" ]]; then
        wget -q -O "$dest" "$url"
        return $?
    fi
    log "No download tool available (curl or wget required)."
    return 1
}

install_release() {
    local base binary_name binary manifest manifest_url binary_url total="-" bytes expected actual tmp_install

    # Stage 1 - resolve version from Cargo.toml (never from VERSION).
    emit 1 started 0 - "" "Resolviendo version desde Cargo.toml..."
    if ! resolve_version; then
        emit 1 failed 0 - E_VERSION_MISMATCH "Version de workspace no valida"
        fail E_VERSION_MISMATCH "No se pudo resolver una version de workspace valida desde Cargo.toml."
    fi
    emit 1 succeeded 0 - "" "OK (v${VERSION_RESOLVED})"
    if [[ "$VERSION_MISMATCH" == "true" ]]; then
        STAGE_DETAIL[1]="OK (v${VERSION_RESOLVED}, version_source_mismatch)"
    fi

    # Stage 2 - detect and validate the target against the closed map.
    emit 2 started 0 - "" "Detectando plataforma..."
    if [[ -z "$TARGET" ]]; then
        if ! TARGET="$(detect_target)"; then
            emit 2 failed 0 - E_UNSUPPORTED_TARGET "Plataforma no soportada"
            fail E_UNSUPPORTED_TARGET "$TARGET_ERROR"
        fi
    fi
    if ! validate_target "$TARGET"; then
        emit 2 failed 0 - E_UNSUPPORTED_TARGET "Target no soportado"
        fail E_UNSUPPORTED_TARGET "$TARGET_ERROR"
    fi
    emit 2 succeeded 0 - "" "OK (${TARGET})"

    base="$(resolve_base_url)"
    binary_name="ag-${TARGET}"
    ALPHA_TMP="$(mktemp -d)"
    binary="$ALPHA_TMP/$binary_name"
    manifest_url="$base/$(urlencode "$MANIFEST_NAME")"
    binary_url="$base/$(urlencode "$binary_name")"

    # Stage 3 - fail-closed order: manifest FIRST, then binary.
    emit 3 started 0 - "" "Descargando manifiesto y ${binary_name}..."
    if [[ -n "$BINARY_CHECKSUMS" ]]; then
        manifest="$BINARY_CHECKSUMS"
        if [[ ! -f "$manifest" ]]; then
            emit 3 failed 0 - E_MANIFEST_MISSING "Manifiesto local ausente"
            fail E_MANIFEST_MISSING "Binary checksum manifest not found: <configurado>."
        fi
    else
        manifest="$ALPHA_TMP/$MANIFEST_NAME"
        if ! do_download "$manifest_url" "$manifest"; then
            emit 3 failed 0 - E_MANIFEST_MISSING "No se pudo descargar el manifiesto"
            fail E_MANIFEST_MISSING "Could not download the checksum manifest from the configured release location."
        fi
    fi

    # Only file:// lets us observe a real total without an extra request. Over
    # HTTP(S) the total stays unknown: no bar, no ETA, no invented progress.
    if [[ "$binary_url" == file://* && -f "${binary_url#file://}" ]]; then
        total="$(wc -c < "${binary_url#file://}" | tr -d '[:space:]')"
    fi

    spin_start
    if ! do_download "$binary_url" "$binary"; then
        spin_stop
        emit 3 failed 0 - E_DOWNLOAD_FAILED "No se pudo descargar ${binary_name}"
        fail E_DOWNLOAD_FAILED "Could not download the binary from the configured release location."
    fi
    spin_stop
    bytes="$(wc -c < "$binary" | tr -d '[:space:]')"
    emit 3 succeeded "$bytes" "$total" "" "OK (${bytes} bytes)"

    # Stage 4 - SHA-256 of the exact base name from the manifest.
    emit 4 started 0 - "" "Verificando SHA-256 (fail-closed)..."
    expected="$(awk -v name="$binary_name" '$2 == name { print $1; exit }' "$manifest")"
    if [[ -z "$expected" ]]; then
        emit 4 failed 0 - E_MANIFEST_MISSING "Entrada ausente en el manifiesto"
        fail E_MANIFEST_MISSING "No checksum entry for ${binary_name} in the configured manifest."
    fi
    if ! printf '%s\n' "$expected" | grep -Eq '^[0-9a-fA-F]{64}$'; then
        emit 4 failed 0 - E_MANIFEST_MISSING "Entrada de checksum no valida"
        fail E_MANIFEST_MISSING "Invalid checksum entry for ${binary_name} in the configured manifest."
    fi
    if ! actual="$(sha256_of "$binary")"; then
        emit 4 failed 0 - E_CHECKSUM_MISMATCH "Sin herramienta SHA-256"
        fail E_CHECKSUM_MISMATCH "No SHA-256 tool available (sha256sum or shasum required)."
    fi
    if [[ "$actual" != "$expected" ]]; then
        emit 4 failed 0 - E_CHECKSUM_MISMATCH "Digest no coincide"
        fail E_CHECKSUM_MISMATCH "Checksum mismatch for ${binary_name}: expected ${expected}, got ${actual}."
    fi
    emit 4 succeeded 0 - "" "OK (digest coincide, fail-closed)"
    log "Checksum verified for ${binary_name}."

    # Stage 5 - atomic copy into BIN_DIR with mode 0755. Never execute the binary.
    emit 5 started 0 - "" "Instalando ag..."
    if ! mkdir -p "$BIN_DIR"; then
        emit 5 failed 0 - E_INSTALL_WRITE_FAILED "No se pudo crear el destino"
        fail E_INSTALL_WRITE_FAILED "Could not create the install destination."
    fi
    tmp_install="$BIN_DIR/.ag.install.$$"
    if ! cp "$binary" "$tmp_install"; then
        emit 5 failed 0 - E_INSTALL_WRITE_FAILED "No se pudo copiar el binario"
        fail E_INSTALL_WRITE_FAILED "Could not write the binary to the install destination."
    fi
    if ! chmod 0755 "$tmp_install"; then
        emit 5 failed 0 - E_INSTALL_WRITE_FAILED "No se pudo fijar permisos"
        fail E_INSTALL_WRITE_FAILED "Could not set permissions on the installed binary."
    fi
    if ! mv -f "$tmp_install" "$BIN_DIR/ag"; then
        emit 5 failed 0 - E_INSTALL_WRITE_FAILED "No se pudo instalar el binario"
        fail E_INSTALL_WRITE_FAILED "Could not finalize the binary in the install destination."
    fi
    emit 5 succeeded 0 - "" "OK (ag instalado)"

    # Stage 6 - ready. No `ag` execution and no local destination path printed.
    emit 6 succeeded 0 - "" "OK (ag ${VERSION_RESOLVED}, 0 compilaciones)"
    log "Installation complete."
    log "Ensure the configured bin directory is on your PATH, then run: ag --version"
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

# ---- Initialization --------------------------------------------------------

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROGRESS_MODE="$(resolve_progress_mode)"
if [[ "$PROGRESS_MODE" == "tui" && -z "${NO_COLOR:-}" ]]; then
    C_OK=$'\033[32m' C_ERR=$'\033[31m' C_DIM=$'\033[2m' C_BOLD=$'\033[1m' C_RESET=$'\033[0m'
fi
if [[ -n "${AG_INSTALLER_CHECKSUMS:-}" ]]; then
    NOTICE_TEXT="Verificacion de integridad activa (fuera de banda)."
else
    NOTICE_TEXT="AG_INSTALLER_CHECKSUMS no definido: fallback local de desarrollo."
fi

# ---- Main ------------------------------------------------------------------

# Fail-closed order step 1: self-integrity runs before any download or install.
verify_self_integrity

if [[ "$CHANNEL" == "alpha" ]]; then
    log "El canal 'alpha' es un alias de 'release'; use AG_INSTALLER_CHANNEL=release."
    CHANNEL="release"
fi

if [[ "$CHANNEL" == "release" ]]; then
    log "Checksum verification only (no signing key, no signature)."
    if [[ "$PROGRESS_MODE" == "tui" ]]; then
        tui_init
    fi
    install_release
    exit 0
fi

if [[ "$CHANNEL" != "source" ]]; then
    log "Unknown AG_INSTALLER_CHANNEL '${CHANNEL}' (expected 'release', 'alpha' or 'source')."
    exit 1
fi

# Explicit source channel: original build path, unchanged.
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
