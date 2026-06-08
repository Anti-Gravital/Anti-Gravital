# Anti-Gravital installer for Windows PowerShell.
#
# Mirrors install.sh: installs the Rust toolchain (if missing), builds the
# workspace in release mode, and installs the `ag` binary.
#
# Security (ADR-0009 rule 4): when fetching a remote copy, verify the
# SHA-256 checksum published with the release artifact before running.
#
# Usage: .\install.ps1

$ErrorActionPreference = "Stop"
$RequiredRust = [version]"1.79.0"

function Log($msg) { Write-Host "[ag-install] $msg" }

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Log "Rust toolchain not found."
    Log "Install rustup from https://rustup.rs and re-run this script."
    exit 1
}

$rawVersion = (rustc --version).Split(" ")[1]
$installedRust = [version]$rawVersion
Log "Detected Rust $rawVersion (minimum required: $RequiredRust)."

if ($installedRust -lt $RequiredRust) {
    Log "Rust version too old. Run 'rustup update stable' and retry."
    exit 1
}

Log "Building the workspace in release mode..."
cargo build --workspace --release

Log "Installing the ag CLI into %USERPROFILE%\.cargo\bin ..."
cargo install --path crates/ag-cli --locked

Log "Installation complete."
Log "Ensure %USERPROFILE%\.cargo\bin is on your PATH, then run: ag --help"
