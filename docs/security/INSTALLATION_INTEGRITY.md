# Installation Integrity

The supported installation path starts with a Git checkout. Do not pipe
`install.sh` or `install.ps1` from the network into a shell.

## Verify the checkout

1. Open the commit or release tag on
   <https://github.com/Anti-Gravital/Anti-Gravital> and note its Git identifier.
2. Clone that exact revision and inspect the installer before executing it.
3. From the repository root, verify the installer checksums.

Linux / macOS:

```bash
sha256sum --check checksums/installers.sha256
bash install.sh
```

Windows PowerShell:

```powershell
$expected = Get-Content checksums/installers.sha256
$expected | ForEach-Object {
    $hash, $path = $_ -split '\s+', 2
    if ((Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash) {
        throw "Checksum mismatch: $path"
    }
}
.\install.ps1
```

`checksums/installers.sha256` is the repository's authoritative checksum list
for installer scripts. Git binds that list and the scripts to the reviewed
commit identifier. A checksum proves that the local script matches that
commit; it does not make an untrusted commit trustworthy.

The installers do not use elevated privileges, modify system package managers,
or download and execute another script. They require an existing Rust toolchain,
build the checked-out workspace, and install `ag` into Cargo's user bin directory.
