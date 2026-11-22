# Anti-Gravital installer for Windows PowerShell.
#
# Mirrors install.sh: verifies the Rust toolchain, builds the workspace in
# release mode, and installs the `ag` binary.
#
# Security (ADR-0009 rule 4): do not invoke this script from the network.
# Follow docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# Out-of-band integrity: set AG_INSTALLER_CHECKSUMS to a SHA-256 manifest obtained
# independently of this repository. When set, this script verifies its own bytes
# against that manifest and aborts before any build or install action on a
# missing file, a missing entry or a mismatch. When unset, it warns and keeps the
# documented development fallback; no in-repository file is treated as a trust
# anchor. See docs/security/INSTALLATION_INTEGRITY.md for the limits.
#
# Usage: .\install.ps1

$ErrorActionPreference = "Stop"
$RequiredRust = [version]"1.99.1"

function Log($msg) { Write-Host "[ag-install] $msg" }

function Verify-InstallerIntegrity {
    $manifest = $env:AG_INSTALLER_CHECKSUMS
    $scriptPath = $PSCommandPath
    $scriptName = Split-Path -Leaf $scriptPath

    if ([string]::IsNullOrWhiteSpace($manifest)) {
        Log "WARNING: AG_INSTALLER_CHECKSUMS is unset; no out-of-band integrity anchor. See docs/security/INSTALLATION_INTEGRITY.md."
        return
    }
    if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
        Log "Integrity manifest not found: $manifest"
        exit 1
    }

    $entries = @(Get-Content -LiteralPath $manifest | Where-Object {
        $parts = $_ -split '\s+', 2
        $parts.Length -ge 2 -and (Split-Path -Leaf $parts[1].Trim()) -eq $scriptName
    })
    if ($entries.Count -eq 0) {
        Log "No checksum entry for $scriptName in $manifest."
        exit 1
    }

    $expected = ($entries[0] -split '\s+', 2)[0].Trim().ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $scriptPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        Log "Integrity check failed for ${scriptName}: expected $expected, got $actual."
        exit 1
    }
    Log "Integrity verified for $scriptName against $manifest."
}

Verify-InstallerIntegrity

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
