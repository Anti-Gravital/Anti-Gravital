# Anti-Gravital installer for Windows PowerShell.
#
# Mirrors install.sh: verifies the Rust toolchain, builds the workspace in
# release mode, and installs the `ag` binary.
#
# Security (ADR-0009 rule 4): do not invoke this script from the network.
# Follow docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# Usage: .\install.ps1

$ErrorActionPreference = "Stop"
$RequiredRust = [version]"1.95.0"

function Log($msg) { Write-Host "[ag-install] $msg" }

if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -or
    -not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Log "Rust toolchain not found."
    Log "Install Rust with rustup from https://rustup.rs, restart PowerShell, and retry."
    exit 1
}

$rawVersion = (rustc --version).Split(" ")[1]
$installedRust = [version]$rawVersion
Log "Detected Rust $rawVersion (minimum required: $RequiredRust)."

if ($installedRust -lt $RequiredRust) {
    Log "Rust $RequiredRust or newer is required. Run 'rustup update stable' and retry."
    exit 1
}

Log "Building the workspace in release mode..."
cargo build --workspace --release

Log "Installing the ag CLI into $env:USERPROFILE\.cargo\bin ..."
cargo install --path crates/ag-cli --locked

Log "Installation complete."
Log "Ensure $env:USERPROFILE\.cargo\bin is on your PATH, then run: ag --help"
