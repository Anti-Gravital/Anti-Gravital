# Installation Integrity

The supported installation path starts with a Git checkout. Do not pipe
`install.sh` or `install.ps1` from the network into a shell.

## Verify the checkout

1. Open the commit or release tag on
   <https://github.com/Anti-Gravital/Anti-Gravital> and note its Git identifier.
2. Clone that exact revision and inspect the installer before executing it.
3. Verify the installer checksums as described below.

The installers do not use elevated privileges, modify system package managers,
or download and execute another script. They require an existing Rust toolchain,
build the checked-out workspace, and install `ag` into Cargo's user bin directory.
Integrity verification runs before any build or install action.

## Trust anchor: `AG_INSTALLER_CHECKSUMS`

`checksums/installers.sha256` lives in the same repository as the scripts it
attests. A tampered checkout can carry tampered checksums, so that file is **not
a trust anchor**. It only proves that a local script matches the reviewed commit.

For a verification that an attacker who controls the repository cannot defeat,
supply a SHA-256 manifest obtained **independently of that repository**:

```bash
export AG_INSTALLER_CHECKSUMS=/path/to/installers.sha256
```

The manifest uses the `sha256sum` line format, one entry per installer, with the
bare file name as the second field:

```
<64 hex digits>  install.sh
<64 hex digits>  install.ps1
```

Obtain the manifest from a source whose integrity does not depend on the checkout
you are verifying (for example a release artifact published through a separate
channel once that infrastructure exists, or values you recorded earlier from a
revision you independently reviewed). `AG_INSTALLER_CHECKSUMS` accepts an
absolute or relative path to a local manifest file; the installers never fetch
it from the network.

Behavior when the variable is set (fail closed):

- the manifest file is missing or unreadable -> abort before any build/install;
- the manifest has no entry for the running installer -> abort;
- the running installer's SHA-256 does not match its entry -> abort;
- every entry relevant to the running installer matches -> the installer logs
  `Integrity verified for <name>` and proceeds.

This same variable is honored by `tools/check-installers.sh`. When it is set,
that tool verifies against the external manifest and fails closed if it is
unavailable; the repository-local list is ignored in that mode.

Linux / macOS:

```bash
sha256sum --check "$AG_INSTALLER_CHECKSUMS"
bash install.sh
```

Windows PowerShell:

```powershell
$expected = Get-Content $env:AG_INSTALLER_CHECKSUMS
$expected | ForEach-Object {
    $hash, $path = $_ -split '\s+', 2
    if ((Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash) {
        throw "Checksum mismatch: $path"
    }
}
.\install.ps1
```

The installers perform the equivalent self-check themselves, so the explicit
`sha256sum --check` / `Get-FileHash` step above is redundant when the variable is
exported. It is shown to make the trust boundary auditable.

## Development fallback (no trust anchor)

When `AG_INSTALLER_CHECKSUMS` is unset, both installers print a warning and run
without self-verification. You can still check the repository-local list for
consistency with the reviewed commit:

```bash
sha256sum --check checksums/installers.sha256
```

This is a development convenience, not a security control. It does not detect a
tampered repository and must not be presented as proof of integrity.

## Known limitations

This mechanism defines and enforces the *interface* for an out-of-band trust
anchor; it does not create one. The project currently has:

- no release signing key and no signed installer or signature artifact;
- no release workflow that publishes checksums through a channel independent of
  the repository.

Consequently a user must still obtain the manifest through a trusted channel out
of band. Until that channel and a published key exist, a verified install rests
on how well the user protected that channel. This is tracked in issue #141; the
issue stays open until a real out-of-band publication or signature path exists.
