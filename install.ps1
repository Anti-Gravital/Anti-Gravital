# Anti-Gravital installer for Windows PowerShell.
#
# Two channels:
#   source (default) - verify the Rust toolchain, build the workspace in release
#                      mode and install the `ag` binary.
#   alpha            - download a prebuilt `ag` binary and its checksum manifest
#                      from a configurable release location, verify the SHA-256
#                      checksum BEFORE installing, and refuse on any mismatch.
#
# Security (ADR-0009 rule 4): do not invoke this script from the network.
# Follow docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
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
#   .\install.ps1                                      # source channel
#   AG_INSTALLER_CHANNEL=alpha; AG_INSTALLER_BASE_URL=...; .\install.ps1
#
# Alpha configuration (environment variables):
#   AG_INSTALLER_CHANNEL=alpha      select the prebuilt-binary channel
#   AG_INSTALLER_BASE_URL=<url>     release assets directory (file:// or http(s))
#   AG_INSTALLER_RELEASE_TAG=<tag>  alternative: GitHub release tag
#   AG_INSTALLER_TARGET=<triple>    override auto-detected target triple
#   AG_INSTALLER_MANIFEST=<name>    manifest filename (default: checksums.txt)
#   AG_INSTALLER_BIN_DIR=<dir>      destination (default: ~/.cargo/bin)
#   AG_INSTALLER_BINARY_CHECKSUMS=<path>  local out-of-band binary manifest;
#                                  when set, checksums.txt is NOT downloaded

$ErrorActionPreference = "Stop"
$RequiredRust = [version]"1.99.1"

$Channel = if ($env:AG_INSTALLER_CHANNEL) { $env:AG_INSTALLER_CHANNEL } else { "source" }
$ManifestName = if ($env:AG_INSTALLER_MANIFEST) { $env:AG_INSTALLER_MANIFEST } else { "checksums.txt" }
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
$BinDir = if ($env:AG_INSTALLER_BIN_DIR) { $env:AG_INSTALLER_BIN_DIR } else { Join-Path $CargoHome "bin" }

function Log($msg) { Write-Host "[ag-install] $msg" }

function Verify-InstallerIntegrity {
    $manifest = $env:AG_INSTALLER_CHECKSUMS
    $scriptPath = $PSCommandPath
    $scriptName = Split-Path -Leaf $scriptPath

    if ([string]::IsNullOrWhiteSpace($manifest)) {
        Log "WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md."
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

function Resolve-BaseUrl {
    if ($env:AG_INSTALLER_BASE_URL) { return ($env:AG_INSTALLER_BASE_URL).TrimEnd('/') }
    if ($env:AG_INSTALLER_RELEASE_TAG) {
        return "https://github.com/Anti-Gravital/Anti-Gravital/releases/download/$($env:AG_INSTALLER_RELEASE_TAG)"
    }
    return $null
}

function Get-Target {
    if ($env:AG_INSTALLER_TARGET) { return $env:AG_INSTALLER_TARGET }
    switch -Regex ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { return "x86_64-pc-windows-msvc" }
        'ARM64' { return "aarch64-pc-windows-msvc" }
        default {
            throw "Unsupported architecture: $($env:PROCESSOR_ARCHITECTURE). Set AG_INSTALLER_TARGET."
        }
    }
}

function Download-File($url, $dest) {
    # file:// is supported for local/offline testing and air-gapped mirrors.
    if ($url -like 'file://*') {
        $src = ([Uri]$url).LocalPath
        Copy-Item -LiteralPath $src -Destination $dest -Force
        return
    }
    Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
}

function Install-Alpha {
    $base = Resolve-BaseUrl
    if (-not $base) {
        Log "AG_INSTALLER_CHANNEL=alpha requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG."
        exit 1
    }
    $resolvedTarget = Get-Target
    $binaryName = "ag-$resolvedTarget.exe"

    $alphaTmp = Join-Path ([IO.Path]::GetTempPath()) ("ag-alpha-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $alphaTmp | Out-Null
    try {
        $binary = Join-Path $alphaTmp $binaryName
        $binaryUrl = "$base/$binaryName"

        Log "Alpha channel: checksum verification only (no signing key, no signature)."
        if ($env:AG_INSTALLER_BINARY_CHECKSUMS) {
            $binaryManifest = $env:AG_INSTALLER_BINARY_CHECKSUMS
            if (-not (Test-Path -LiteralPath $binaryManifest -PathType Leaf)) {
                Log "Binary checksum manifest not found: $binaryManifest."
                exit 1
            }
            $manifest = $binaryManifest
            Log "Alpha channel: using local binary checksum manifest $manifest (out-of-band)."
        } else {
            $manifest = Join-Path $alphaTmp $ManifestName
            $manifestUrl = "$base/$ManifestName"
            Log "Alpha channel: downloading checksum manifest from $manifestUrl"
            Download-File $manifestUrl $manifest
        }
        Log "Alpha channel: downloading binary from $binaryUrl"
        Download-File $binaryUrl $binary

        $entries = @(Get-Content -LiteralPath $manifest | Where-Object {
            $parts = $_ -split '\s+', 2
            $parts.Length -ge 2 -and (Split-Path -Leaf $parts[1].Trim()) -eq $binaryName
        })
        if ($entries.Count -eq 0) {
            Log "No checksum entry for $binaryName in $ManifestName."
            exit 1
        }

        $expected = ($entries[0] -split '\s+', 2)[0].Trim().ToLowerInvariant()
        $actual = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            Log "Checksum mismatch for ${binaryName}: expected $expected, got $actual."
            exit 1
        }
        Log "Checksum verified for $binaryName."

        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        Copy-Item -LiteralPath $binary -Destination (Join-Path $BinDir 'ag.exe') -Force
        Log "Installed ag to $BinDir\ag.exe."
        Log "Installation complete."
        Log "Ensure $BinDir is on your PATH, then run: ag --version"
    } finally {
        if (Test-Path -LiteralPath $alphaTmp) {
            Remove-Item -LiteralPath $alphaTmp -Recurse -Force
        }
    }
}

Verify-InstallerIntegrity

if ($Channel -eq "alpha") {
    Install-Alpha
    exit 0
}
if ($Channel -ne "source") {
    Log "Unknown AG_INSTALLER_CHANNEL '$Channel' (expected 'source' or 'alpha')."
    exit 1
}

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
