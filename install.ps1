# Anti-Gravital installer for Windows (PowerShell 5.1+).
#
# Channels (AG_INSTALLER_CHANNEL):
#   release (DEFAULT) - download the prebuilt `ag` binary for this platform,
#                       verify its SHA-256 against a checksum manifest BEFORE
#                       installing, and refuse on any mismatch. No compilation.
#   alpha             - alias of `release`, accepted with a migration notice.
#   source            - legacy path: require a Rust toolchain, build the
#                       workspace in release mode and install `ag` from source.
# There is NO automatic fallback to `source`: a failed fast channel exits
# non-zero with a stable code and a message that offers the explicit opt-in.
#
# Security (ADR-0009 rule 4): do not invoke this script from the network.
# Follow docs/security/INSTALLATION_INTEGRITY.md before running a downloaded copy.
#
# This installer uses SHA-256 CHECKSUM verification ONLY (integrity, not
# authenticity). No release signing key exists and no signature artifact is
# published; this installer NEVER claims a signature, authenticity or a trust
# anchor. The base URL / release tag you supply is the trust boundary. See
# docs/security/INSTALLATION_INTEGRITY.md.
#
# Out-of-band self-integrity: set AG_INSTALLER_CHECKSUMS to a SHA-256 manifest
# obtained independently of this repository. When set, this script verifies its
# own bytes and aborts before any build, download or install on a missing file,
# a missing entry or a mismatch. When unset, a development-only local fallback
# is used; the warning is printed on EVERY run and is never silenced.
#
# UI: native TUI on PowerShell 5.1+ ([Console]::SetCursorPosition /
# Write-Host -NoNewline; ANSI/VT only when the console supports it). No Add-Type,
# no NuGet modules, no new packages, no downloaded-binary execution. Deterministic
# headless mode uses AG_INSTALLER_PROGRESS=auto|tui|headless.
#
# Usage:
#   .\install.ps1                                        # release (default, prebuilt, 0 builds)
#   AG_INSTALLER_CHANNEL=source; .\install.ps1           # explicit source build
#   AG_INSTALLER_PROGRESS=headless; .\install.ps1        # deterministic CI output
#
# Environment variables (all optional except a reachable asset location):
#   AG_INSTALLER_CHANNEL=release|alpha|source          channel selection (default: release)
#   AG_INSTALLER_BASE_URL=<url>                        asset dir (file:// or http(s)); wins
#   AG_INSTALLER_RELEASE_TAG=<v-tag>                    expands to the default release URL
#   AG_INSTALLER_TARGET=<triple>                        override; validated against closed map
#   AG_INSTALLER_MANIFEST=<name>                        manifest filename (default: checksums.txt)
#   AG_INSTALLER_BIN_DIR=<dir>                          destination (default: ~/.cargo/bin)
#   AG_INSTALLER_BINARY_CHECKSUMS=<path>                local out-of-band binary manifest
#   AG_INSTALLER_CHECKSUMS=<path>                       local out-of-band installer manifest
#   AG_INSTALLER_PROGRESS=auto|tui|headless             UI mode (default: auto)
#   NO_COLOR=1                                          disable color, keep symbols and TUI
#   CI=1 / TERM=dumb / non-TTY                          force headless in `auto`
#
# Canonical tag source: [workspace.package].version in Cargo.toml (e.g.
# 0.1.0-alpha.2 -> v0.1.0-alpha.2), matching .github/workflows/release.yml.
# VERSION (framework/roadmap label) is NOT used to build the tag.

$ErrorActionPreference = "Stop"

# --- Named constants (S-3: no hardcoded operational values without a name) ---
$DefaultReleaseBaseTemplate = "https://github.com/Anti-Gravital/Anti-Gravital/releases/download"
$DefaultManifestName         = "checksums.txt"
$DefaultBinaryBaseName       = "ag"
$InstalledBinaryName         = "ag.exe"
$RequiredRust                = [version]"1.99.1"
$UiWidth                     = 78
$TickMs                      = 100
$SpinUnicode                 = @([char]0x280B, [char]0x2819, [char]0x2839, [char]0x2838, [char]0x283C, [char]0x2834, [char]0x2826, [char]0x2827, [char]0x2807, [char]0x280F)
$SpinAscii                   = @('|', '/', '-', '\')

# Closed target map: ONLY what release.yml publishes for Windows x64.
# aarch64-pc-windows-msvc is deliberately absent (not a published asset).
$SupportedTargets = @("x86_64-pc-windows-msvc")

# Stable exit codes. Ctrl-C terminates the pipeline and the `finally` block
# removes the temp dir; the exact 130 code cannot be forced without native
# interop (Add-Type/P-Invoke), which is prohibited here, so it is not claimed.
$EXIT_OK              = 0
$EXIT_CONFIG          = 1
$EXIT_NETWORK         = 2
$EXIT_CHECKSUM        = 3
$EXIT_PERMISSIONS     = 4

# Stable error codes (headless `code=` field).
$E_CONFIG                  = "E_CONFIG"
$E_VERSION_MISMATCH        = "E_VERSION_MISMATCH"
$E_UNSUPPORTED_TARGET      = "E_UNSUPPORTED_TARGET"
$E_SELF_INTEGRITY          = "E_SELF_INTEGRITY"
$E_MANIFEST_MISSING        = "E_MANIFEST_MISSING"
$E_MANIFEST_ENTRY_MISSING  = "E_MANIFEST_ENTRY_MISSING"
$E_DOWNLOAD_FAILED         = "E_DOWNLOAD_FAILED"
$E_CHECKSUM_MISMATCH       = "E_CHECKSUM_MISMATCH"
$E_INSTALL_WRITE_FAILED    = "E_INSTALL_WRITE_FAILED"

# --- UI state ---
$script:Seq        = 0
$script:Mode       = "headless"
$script:NoColor    = $false
$script:UseAnsi    = $false
$script:UseUnicode = $false
$script:RenderTop  = -1
$script:SpinIdx    = 0
$script:Notice     = ""
$script:BinaryName = ""
$script:DlBytes    = [int64]0
$script:DlTotal    = [int64](-1)
$script:DlStart    = [datetime]::MinValue

$StageNames = @("resolve_version", "detect_platform", "download", "verify", "install", "ready")
$StageLabels = @(
    @{ Tui = "Resolver version";    Headless = "Resolver version" },
    @{ Tui = "Detectar plataforma"; Headless = "Detectar plataforma" },
    @{ Tui = "Descargar";           Headless = "Descargar" },
    @{ Tui = "Verificar SHA-256";   Headless = "Verificar SHA-256" },
    @{ Tui = "Instalar";            Headless = "Instalar" },
    @{ Tui = "Listo";               Headless = "Listo" }
)
$Stages = @()
for ($i = 0; $i -lt $StageNames.Count; $i++) {
    $Stages += [PSCustomObject]@{ Name = $StageNames[$i]; State = "pending"; Detail = "PENDIENTE" }
}

function Get-ProgressMode {
    $requested = "auto"
    if ($env:AG_INSTALLER_PROGRESS) { $requested = $env:AG_INSTALLER_PROGRESS.Trim().ToLowerInvariant() }
    if ($requested -notin @("auto", "tui", "headless")) { $requested = "auto" }

    $forcedHeadless = $false
    if ($env:CI) { $forcedHeadless = $true }
    if ($env:TERM -eq "dumb") { $forcedHeadless = $true }
    if ([Console]::IsOutputRedirected) { $forcedHeadless = $true }

    if ($requested -eq "headless") { return "headless" }
    if ($forcedHeadless) { return "headless" }
    if ($requested -eq "tui") { return "tui" }
    return "tui"  # auto with a TTY
}

function Test-VtSupport {
    if ($env:WT_SESSION) { return $true }
    if ($env:TERM -match 'xterm|screen|tmux|256color|linux') { return $true }
    if ($PSVersionTable.PSVersion.Major -ge 7) { return $true }
    return $false
}

function Format-Bytes([int64]$n) {
    if ($n -lt 0) { return "-" }
    if ($n -lt 1KB) { return "{0} B" -f $n }
    if ($n -lt 1MB) { return "{0:N1} KiB" -f ($n / 1KB) }
    return "{0:N1} MiB" -f ($n / 1MB)
}

function Get-SpinFrame {
    if ($script:SpinIdx -ge ($script:SpinFrames.Count)) { $script:SpinIdx = 0 }
    $f = $script:SpinFrames[$script:SpinIdx]
    $script:SpinIdx++
    return $f
}

function Write-UiLine {
    param([string]$Text, [string]$Ansi, $ConsoleColor)
    $pad = $Text.PadRight($UiWidth)
    if ($script:UseAnsi) {
        if ($script:NoColor -or -not $Ansi) {
            Write-Host $pad -NoNewline
        } else {
            Write-Host "$Ansi$pad`e[0m" -NoNewline
        }
    } else {
        if ($script:NoColor -or -not $ConsoleColor) {
            Write-Host $pad -NoNewline
        } else {
            Write-Host $pad -NoNewline -ForegroundColor $ConsoleColor
        }
    }
    Write-Host ''
}

function Get-StageStyle($state) {
    # Every marker carries a symbol AND a word (accessibility: never color alone).
    # conhost clásico without VT gets the ASCII symbol set (diseno-tui.md 2.2).
    if ($script:UseUnicode) {
        switch ($state) {
            "active"  { return @{ Sym = "◉"; Word = "ACTIVO";  Ansi = "`e[33m"; Color = [ConsoleColor]::Yellow } }
            "ok"      { return @{ Sym = "OK"; Word = "OK";      Ansi = "`e[32m"; Color = [ConsoleColor]::Green } }
            "error"   { return @{ Sym = "ERR"; Word = "FALLO";  Ansi = "`e[31m"; Color = [ConsoleColor]::Red } }
            "skipped" { return @{ Sym = "–";  Word = "OMITIDO"; Ansi = "`e[90m"; Color = [ConsoleColor]::DarkGray } }
            default   { return @{ Sym = "○";  Word = "PENDIENTE"; Ansi = "`e[90m"; Color = [ConsoleColor]::DarkGray } }
        }
    }
    switch ($state) {
        "active"  { return @{ Sym = "[>]"; Word = "ACTIVO";   Ansi = "`e[33m"; Color = [ConsoleColor]::Yellow } }
        "ok"      { return @{ Sym = "[x]"; Word = "OK";       Ansi = "`e[32m"; Color = [ConsoleColor]::Green } }
        "error"   { return @{ Sym = "[!]"; Word = "FALLO";    Ansi = "`e[31m"; Color = [ConsoleColor]::Red } }
        "skipped" { return @{ Sym = "[-]"; Word = "OMITIDO";  Ansi = "`e[90m"; Color = [ConsoleColor]::DarkGray } }
        default   { return @{ Sym = "[ ]"; Word = "PENDIENTE"; Ansi = "`e[90m"; Color = [ConsoleColor]::DarkGray } }
    }
}

function Render-Tui {
    if ($script:Mode -ne "tui") { return }
    if ($script:RenderTop -lt 0) {
        $script:RenderTop = [Console]::CursorTop
    } else {
        [Console]::SetCursorPosition(0, $script:RenderTop)
    }
    $spin = Get-SpinFrame
    Write-UiLine ("Anti-Gravital · instalación rápida (release, 0 compilaciones)") "`e[1m" [ConsoleColor]::White
    Write-UiLine ("-" * 24) $null $null
    for ($i = 0; $i -lt $Stages.Count; $i++) {
        $s = $Stages[$i]
        $style = Get-StageStyle $s.State
        $detail = $s.Detail
        if ($s.State -eq "active" -and $script:SpinActive) { $detail = "$detail  $spin" }
        $line = (" {0,2} {1,-3} {2,-21} {3}" -f ($i + 1), $style.Sym, $StageLabels[$i].Tui, $detail)
        Write-UiLine $line $style.Ansi $style.Color
    }
    Write-UiLine ("-" * 24) $null $null
    Write-UiLine ("Estado   $($script:StatusLine)") $null [ConsoleColor]::Cyan
    if ($script:Notice) {
        Write-UiLine ("Aviso    $($script:Notice)") "`e[33m" [ConsoleColor]::Yellow
    }
    Write-UiLine ("Ctrl-C cancela y limpia   ? ayuda (teclas de overlay no enlazadas)") $null [ConsoleColor]::DarkGray
}

function Emit-Event {
    param(
        [string]$Stage,
        [string]$State,
        [int64]$Bytes = 0,
        $Total = $null,
        [string]$Code = "",
        [string]$Hint = ""
    )
    $script:Seq++
    $totalStr = "-"
    if ($null -ne $Total) { $totalStr = [string]$Total }
    $line = "ag-install $($script:Seq) $Stage $State bytes=$Bytes total=$totalStr"
    if ($Code) { $line += " code=$Code" }

    $idx = [array]::IndexOf($StageNames, $Stage)
    if ($idx -ge 0) {
        switch ($State) {
            "started"   { $Stages[$idx].State = "active";  $Stages[$idx].Detail = "INICIO" }
            "succeeded" { $Stages[$idx].State = "ok";      $Stages[$idx].Detail = "OK" }
            "failed"    { $Stages[$idx].State = "error";   $Stages[$idx].Detail = "FALLO" }
            "skipped"   { $Stages[$idx].State = "skipped"; $Stages[$idx].Detail = "OMITIDO" }
        }
        $script:StatusLine = "$($StageLabels[$idx].Tui): $State"
        $script:SpinActive = ($State -eq "started")
    }

    if ($script:Mode -eq "headless") {
        Write-Host $line
        if ($Hint) { Write-Host "ag-install hint $Code $Hint" }
    } else {
        Render-Tui
        if ($State -eq "failed" -and $Hint) {
            Write-Host $line
            Write-Host "ag-install hint $Code $Hint"
        }
    }
}

function Fail-Installer {
    param([string]$Code, [string]$Stage, [string]$Message, [int]$ExitCode)
    Emit-Event -Stage $Stage -State "failed" -Code $Code -Hint $Message
    exit $ExitCode
}

function Assert-SafeComponent {
    param([string]$Value, [string]$Name)
    if ([string]::IsNullOrWhiteSpace($Value)) {
        Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Empty component: $Name." -ExitCode $EXIT_CONFIG
    }
    if ($Value -match '[/\\]|\.\.|\s' -or $Value -match '[\x00-\x1F]') {
        Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Unsafe component: $Name." -ExitCode $EXIT_CONFIG
    }
}

function Assert-BaseUrl {
    param([string]$Url)
    try { $uri = [Uri]$Url } catch {
        Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Invalid base URL." -ExitCode $EXIT_CONFIG
    }
    if ($uri.Scheme -notin @("http", "https", "file")) {
        Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Unsupported URL scheme (use http(s) or file)." -ExitCode $EXIT_CONFIG
    }
    return $uri
}

function Get-WorkspaceVersion {
    # Equivalent to release.yml:111 (awk over [workspace.package] / version).
    $cargoToml = $null
    if ($PSCommandPath) { $cargoToml = Join-Path (Split-Path -Parent $PSCommandPath) "Cargo.toml" }
    if (-not $cargoToml -or -not (Test-Path -LiteralPath $cargoToml -PathType Leaf)) {
        Fail-Installer -Code $E_VERSION_MISMATCH -Stage "resolve_version" -Message "Cargo.toml not found next to the installer." -ExitCode $EXIT_CONFIG
    }
    $inSection = $false
    foreach ($line in [System.IO.File]::ReadAllLines($cargoToml)) {
        if ($line -match '^\s*\[workspace\.package\]\s*$') { $inSection = $true; continue }
        if ($inSection -and $line -match '^\s*version\s*=\s*"([^"]+)"') { return $Matches[1] }
        if ($inSection -and $line -match '^\s*\[') { break }
    }
    Fail-Installer -Code $E_VERSION_MISMATCH -Stage "resolve_version" -Message "No [workspace.package].version in Cargo.toml." -ExitCode $EXIT_CONFIG
}

function Resolve-ReleaseBase {
    # Precedence: AG_INSTALLER_BASE_URL > AG_INSTALLER_RELEASE_TAG > workspace version.
    if ($env:AG_INSTALLER_BASE_URL) {
        $base = $env:AG_INSTALLER_BASE_URL.Trim().TrimEnd('/')
        Assert-BaseUrl $base | Out-Null
        return $base
    }
    if ($env:AG_INSTALLER_RELEASE_TAG) {
        $tag = $env:AG_INSTALLER_RELEASE_TAG.Trim()
        Assert-SafeComponent $tag "AG_INSTALLER_RELEASE_TAG"
        if ($tag -notmatch '^v[0-9A-Za-z._-]+$') {
            Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Release tag must be v-prefixed and URL-safe." -ExitCode $EXIT_CONFIG
        }
        return "$DefaultReleaseBaseTemplate/$tag"
    }
    $version = Get-WorkspaceVersion
    if ($version -notmatch '^\d+\.\d+\.\d+-alpha\.\d+$') {
        Fail-Installer -Code $E_VERSION_MISMATCH -Stage "resolve_version" -Message "Workspace version is not an -alpha.N release version." -ExitCode $EXIT_CONFIG
    }
    Assert-SafeComponent $version "workspace version"
    return "$DefaultReleaseBaseTemplate/v$version"
}

function Resolve-Target {
    if ($env:AG_INSTALLER_TARGET) {
        $t = $env:AG_INSTALLER_TARGET.Trim()
        if ($SupportedTargets -notcontains $t) {
            Fail-Installer -Code $E_UNSUPPORTED_TARGET -Stage "detect_platform" -Message "Unsupported target. Only x86_64-pc-windows-msvc is published for Windows." -ExitCode $EXIT_CONFIG
        }
        return $t
    }
    switch -Regex ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { return "x86_64-pc-windows-msvc" }
        default {
            Fail-Installer -Code $E_UNSUPPORTED_TARGET -Stage "detect_platform" -Message "Unsupported architecture. This release publishes x86_64-pc-windows-msvc only (ARM64 is not published)." -ExitCode $EXIT_CONFIG
        }
    }
}

function Get-AssetUrl($base, $name) {
    if ($base -like 'file://*') {
        $local = ([Uri]$base).LocalPath
        return ([Uri](Join-Path $local $name)).AbsoluteUri
    }
    return "$base/$name"
}

function Download-To {
    param([string]$Url, [string]$Dest, [scriptblock]$OnTick)
    if ($Url -like 'file://*') {
        $src = ([Uri]$Url).LocalPath
        Copy-Item -LiteralPath $src -Destination $Dest -Force
        return
    }
    $wc = New-Object System.Net.WebClient
    $script:DlBytes = [int64]0
    $script:DlTotal = [int64](-1)
    $script:asyncError = $null
    try {
        $progressHandler = [System.Net.DownloadProgressChangedEventHandler]{
            param($s, $e)
            $script:DlBytes = [int64]$e.BytesReceived
            $script:DlTotal = [int64]$e.TotalBytesToReceive
        }
        $completeHandler = [System.Net.AsyncCompletedEventHandler]{
            param($s, $e)
            if ($e.Error) { $script:asyncError = $e.Error }
        }
        $wc.add_DownloadProgressChanged($progressHandler)
        $wc.add_DownloadFileCompleted($completeHandler)
        $wc.DownloadFileAsync([Uri]$Url, $Dest)
        while ($wc.IsBusy) {
            if ($OnTick) { & $OnTick $script:DlBytes $script:DlTotal }
            Start-Sleep -Milliseconds $TickMs
        }
        if ($script:asyncError) { throw $script:asyncError }
    } finally {
        $wc.Dispose()
    }
}

function Update-DownloadFrame {
    param([int64]$Bytes, [int64]$Total)
    if ($script:Mode -ne "tui") { return }
    $detail = "Descargando $($script:BinaryName)"
    if ($Total -gt 0 -and $Bytes -gt 0) {
        $pct = [int][math]::Floor((100.0 * $Bytes) / $Total)
        if ($pct -gt 100) { $pct = 100 }
        $filled = [int][math]::Floor(12.0 * $pct / 100.0)
        $bar = ("[" + ("#" * $filled) + ("." * (12 - $filled)) + "]")
        $detail = "$bar ${pct}% $(Format-Bytes $Bytes)/$(Format-Bytes $Total)"
        $elapsed = ((Get-Date) - $script:DlStart).TotalSeconds
        if ($elapsed -gt 0) {
            $speed = $Bytes / $elapsed
            if ($speed -gt 0) {
                $eta = ($Total - $Bytes) / $speed
                if ($eta -lt 0) { $eta = 0 }
                $detail += (" ETA {0:N0}s" -f $eta)
            }
        }
    } else {
        $detail = "$detail $(Format-Bytes $Bytes) recibidos (total desconocido)"
    }
    $Stages[2].Detail = $detail
    Render-Tui
}

function Verify-InstallerIntegrity {
    $manifest = $env:AG_INSTALLER_CHECKSUMS
    $scriptPath = $PSCommandPath
    $scriptName = Split-Path -Leaf $scriptPath

    if ([string]::IsNullOrWhiteSpace($manifest)) {
        $script:Notice = "AG_INSTALLER_CHECKSUMS unset: development-only fallback (see docs/security/INSTALLATION_INTEGRITY.md)"
        if ($script:Mode -eq "headless") {
            Write-Host "ag-install notice AG_INSTALLER_CHECKSUMS unset: development-only fallback; see docs/security/INSTALLATION_INTEGRITY.md"
        } else {
            Render-Tui
        }
        return
    }
    if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
        Fail-Installer -Code $E_SELF_INTEGRITY -Stage "resolve_version" -Message "Installer integrity manifest not found." -ExitCode $EXIT_CONFIG
    }
    $entries = @(Get-Content -LiteralPath $manifest | Where-Object {
        $parts = $_ -split '\s+', 2
        $parts.Length -ge 2 -and (Split-Path -Leaf $parts[1].Trim()) -eq $scriptName
    })
    if ($entries.Count -eq 0) {
        Fail-Installer -Code $E_SELF_INTEGRITY -Stage "resolve_version" -Message "No installer checksum entry in manifest." -ExitCode $EXIT_CONFIG
    }
    if ($entries.Count -gt 1) {
        Fail-Installer -Code $E_SELF_INTEGRITY -Stage "resolve_version" -Message "Duplicate installer checksum entry." -ExitCode $EXIT_CONFIG
    }
    $expected = ($entries[0] -split '\s+', 2)[0].Trim().ToLowerInvariant()
    if ($expected -notmatch '^[0-9a-f]{64}$') {
        Fail-Installer -Code $E_SELF_INTEGRITY -Stage "resolve_version" -Message "Installer checksum is not 64 hex digits." -ExitCode $EXIT_CONFIG
    }
    $actual = (Get-FileHash -LiteralPath $scriptPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        Fail-Installer -Code $E_SELF_INTEGRITY -Stage "resolve_version" -Message "Installer integrity mismatch: no action taken." -ExitCode $EXIT_CONFIG
    }
    $script:Notice = "Installer self-integrity verified against AG_INSTALLER_CHECKSUMS"
}

function Find-BinaryChecksum($manifestPath, $binaryName) {
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        Fail-Installer -Code $E_MANIFEST_MISSING -Stage "verify" -Message "Checksum manifest not found. Review AG_INSTALLER_BASE_URL / AG_INSTALLER_RELEASE_TAG." -ExitCode $EXIT_NETWORK
    }
    $entries = @(Get-Content -LiteralPath $manifestPath | Where-Object {
        $parts = $_ -split '\s+', 2
        $parts.Length -ge 2 -and (Split-Path -Leaf $parts[1].Trim()) -eq $binaryName
    })
    if ($entries.Count -eq 0) {
        Fail-Installer -Code $E_MANIFEST_ENTRY_MISSING -Stage "verify" -Message "No checksum entry for the downloaded asset. Integrity, not authenticity." -ExitCode $EXIT_CHECKSUM
    }
    if ($entries.Count -gt 1) {
        Fail-Installer -Code $E_MANIFEST_ENTRY_MISSING -Stage "verify" -Message "Duplicate checksum entry for the downloaded asset." -ExitCode $EXIT_CHECKSUM
    }
    $expected = ($entries[0] -split '\s+', 2)[0].Trim().ToLowerInvariant()
    if ($expected -notmatch '^[0-9a-f]{64}$') {
        Fail-Installer -Code $E_MANIFEST_ENTRY_MISSING -Stage "verify" -Message "Checksum entry is not 64 hex digits." -ExitCode $EXIT_CHECKSUM
    }
    return $expected
}

function Install-Release {
    Emit-Event -Stage "resolve_version" -State "started"
    $base = Resolve-ReleaseBase
    Emit-Event -Stage "resolve_version" -State "succeeded"

    Emit-Event -Stage "detect_platform" -State "started"
    $target = Resolve-Target
    $script:BinaryName = "$DefaultBinaryBaseName-$target.exe"
    Assert-SafeComponent $script:BinaryName "binary asset name"
    Emit-Event -Stage "detect_platform" -State "succeeded"

    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("ag-release-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        $manifestName = if ($env:AG_INSTALLER_MANIFEST) { $env:AG_INSTALLER_MANIFEST.Trim() } else { $DefaultManifestName }
        Assert-SafeComponent $manifestName "AG_INSTALLER_MANIFEST"
        $binaryPath = Join-Path $tmp $script:BinaryName

        # 1) manifest (out-of-band local, or from the same base URL)
        Emit-Event -Stage "verify" -State "started"
        $manifestPath = $null
        if ($env:AG_INSTALLER_BINARY_CHECKSUMS) {
            $manifestPath = $env:AG_INSTALLER_BINARY_CHECKSUMS
            if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
                Fail-Installer -Code $E_MANIFEST_MISSING -Stage "verify" -Message "Local binary checksum manifest not found." -ExitCode $EXIT_NETWORK
            }
        } else {
            $manifestPath = Join-Path $tmp $manifestName
            try {
                Download-To -Url (Get-AssetUrl $base $manifestName) -Dest $manifestPath
            } catch {
                Fail-Installer -Code $E_MANIFEST_MISSING -Stage "verify" -Message "Could not download the checksum manifest. Review AG_INSTALLER_BASE_URL / AG_INSTALLER_RELEASE_TAG." -ExitCode $EXIT_NETWORK
            }
        }

        # 2) binary (never executed, never copied to destination yet)
        Emit-Event -Stage "download" -State "started"
        $script:DlStart = Get-Date
        try {
            Download-To -Url (Get-AssetUrl $base $script:BinaryName) -Dest $binaryPath -OnTick {
                param($b, $t)
                Update-DownloadFrame -Bytes ([int64]$b) -Total ([int64]$t)
            }
        } catch {
            Fail-Installer -Code $E_DOWNLOAD_FAILED -Stage "download" -Message "Could not download ag-<target>. Review AG_INSTALLER_BASE_URL / AG_INSTALLER_RELEASE_TAG. To compile locally instead, rerun with AG_INSTALLER_CHANNEL=source (cost: full source build)." -ExitCode $EXIT_NETWORK
        }
        if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
            Fail-Installer -Code $E_DOWNLOAD_FAILED -Stage "download" -Message "Downloaded asset is missing. Review AG_INSTALLER_BASE_URL / AG_INSTALLER_RELEASE_TAG." -ExitCode $EXIT_NETWORK
        }
        $downloadedBytes = (Get-Item -LiteralPath $binaryPath).Length
        Emit-Event -Stage "download" -State "succeeded" -Bytes $downloadedBytes -Total $downloadedBytes

        # 3) verify: exact entry + SHA-256 (fail-closed)
        $expected = Find-BinaryChecksum $manifestPath $script:BinaryName
        $actual = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            Fail-Installer -Code $E_CHECKSUM_MISMATCH -Stage "verify" -Message "Checksum mismatch. No binary was written. Checksum verification only (integrity, not authenticity)." -ExitCode $EXIT_CHECKSUM
        }
        Emit-Event -Stage "verify" -State "succeeded"
        if ($script:Mode -eq "headless") {
            Write-Host "ag-install info checksum verification only (integrity, not authenticity); no signing key, no signature"
        }

        # 4) install: copy to <BIN_DIR>\ag.exe, no elevation, no execution
        Emit-Event -Stage "install" -State "started"
        try {
            if (-not (Test-Path -LiteralPath $BinDir -PathType Container)) {
                New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
            }
            Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $BinDir $InstalledBinaryName) -Force
        } catch {
            Fail-Installer -Code $E_INSTALL_WRITE_FAILED -Stage "install" -Message "Could not write ag.exe to the destination. Check permissions or close a running ag.exe." -ExitCode $EXIT_PERMISSIONS
        }
        Emit-Event -Stage "install" -State "succeeded"

        # 5) ready
        Emit-Event -Stage "ready" -State "succeeded"
        if ($script:Mode -eq "headless") {
            Write-Host "ag-install info installed <BIN_DIR>\ag.exe; ensure the destination is on PATH, then run: ag --version"
        }
    } finally {
        if (Test-Path -LiteralPath $tmp) {
            Remove-Item -LiteralPath $tmp -Recurse -Force
        }
    }
}

function Test-CommandExists([string]$name) {
    # Typed capability probe (S-7/S-10): catch ONLY the expected "not found"
    # exception; never swallow an unexpected error and never use a suppression.
    try {
        return [bool](Get-Command -Name $name -CommandType Application -ErrorAction Stop)
    } catch [System.Management.Automation.CommandNotFoundException] {
        return $false
    }
}

function Install-Source {
    Emit-Event -Stage "resolve_version" -State "started"
    if (-not (Test-CommandExists "cargo") -or -not (Test-CommandExists "rustc")) {
        Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Rust toolchain not found. Install Rust from https://rustup.rs, restart PowerShell, and retry." -ExitCode $EXIT_CONFIG
    }
    $rawVersion = (rustc --version).Split(" ")[1]
    $installedRust = [version]$rawVersion
    Emit-Event -Stage "resolve_version" -State "succeeded"
    if ($installedRust -lt $RequiredRust) {
        Fail-Installer -Code $E_VERSION_MISMATCH -Stage "resolve_version" -Message "Rust 1.99.1 or newer is required. Run 'rustup update stable' and retry." -ExitCode $EXIT_CONFIG
    }

    Emit-Event -Stage "detect_platform" -State "succeeded"
    Emit-Event -Stage "download" -State "skipped"
    Emit-Event -Stage "verify" -State "skipped"

    Emit-Event -Stage "install" -State "started"
    $Stages[4].Detail = "Compilando desde fuente (puede tardar)"
    cargo build --workspace --release
    if ($LASTEXITCODE -ne 0) {
        Fail-Installer -Code $E_CONFIG -Stage "install" -Message "cargo build failed." -ExitCode $EXIT_CONFIG
    }
    cargo install --path crates/ag-cli --locked
    if ($LASTEXITCODE -ne 0) {
        Fail-Installer -Code $E_CONFIG -Stage "install" -Message "cargo install failed." -ExitCode $EXIT_CONFIG
    }
    Emit-Event -Stage "install" -State "succeeded"
    Emit-Event -Stage "ready" -State "succeeded"
    if ($script:Mode -eq "headless") {
        Write-Host "ag-install info source channel; ensure Cargo's user bin is on PATH, then run: ag --help"
    }
}

# --- Bootstrap ---
$script:Mode = Get-ProgressMode
$script:NoColor = [bool]$env:NO_COLOR
$script:UseAnsi = (Test-VtSupport) -and ($script:Mode -eq "tui") -and (-not $script:NoColor)
$script:UseUnicode = $script:UseAnsi
$script:SpinFrames = if ($script:UseUnicode) { $SpinUnicode } else { $SpinAscii }
$script:SpinActive = $false
$script:StatusLine = "Iniciando"

$Channel = if ($env:AG_INSTALLER_CHANNEL) { $env:AG_INSTALLER_CHANNEL.Trim().ToLowerInvariant() } else { "release" }
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
$BinDir = if ($env:AG_INSTALLER_BIN_DIR) { $env:AG_INSTALLER_BIN_DIR } else { Join-Path $CargoHome "bin" }

# Step 1 of the fail-closed chain: self-integrity always runs first (S-10: warning never silenced).
Verify-InstallerIntegrity

if ($Channel -eq "source") {
    Install-Source
    exit $EXIT_OK
}
if ($Channel -eq "alpha") {
    if ($script:Mode -eq "headless") {
        Write-Host "ag-install notice AG_INSTALLER_CHANNEL=alpha is an alias of release; use release."
    } else {
        Write-Host "AG_INSTALLER_CHANNEL=alpha is an alias of release; use release."
    }
    Install-Release
    exit $EXIT_OK
}
if ($Channel -ne "release") {
    Fail-Installer -Code $E_CONFIG -Stage "resolve_version" -Message "Unknown AG_INSTALLER_CHANNEL (expected release, alpha or source)." -ExitCode $EXIT_CONFIG
}

Install-Release
exit $EXIT_OK
