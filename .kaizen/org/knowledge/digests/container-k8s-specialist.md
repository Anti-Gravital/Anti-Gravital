# Digest de conocimiento: container-k8s-specialist

- **Installation integrity rules** — `docs/security/INSTALLATION_INTEGRITY.md` (sha256 0be0a7e51ed1)

## INSTALLATION_INTEGRITY.md
# Installation Integrity

The supported installation path starts with a Git checkout. Do not pipe
`install.sh` or `install.ps1` from the network into a shell.

## Verify the checkout

1. Open the commit or release tag on
   <https://github.com/Anti-Gravital/Anti-Gravital> and note its Git identifier.
2. Clone that exact revision and inspect the installer before executing it.
3. Verify the installer checksums as described below.

The installers do not use elevated privileges, modify system package managers,
or download and execute another script. In the default `source` channel they
require an existing Rust toolchain, build the checked-out workspace, and install
`ag` into Cargo's user bin directory. The opt-in `alpha` channel installs a
prebuilt binary instead (see below). Integrity verification runs before any
build, download-based install or install action.

## Alpha channel: prebuilt binary with checksum verification

Wave 5 provides a testable alpha. The prebuilt-binary path is opt-in:

```bash
AG_INSTALLER_CHANNEL=alpha \
AG_INSTALLER_BASE_URL=https://github.com/Anti-Gravital/Anti-Gravital/releases/download/v0.1.0-alpha.1 \
bash install.sh
```

`AG_INSTALLER_RELEASE_TAG=v0.1.0-alpha.1` is an alternative to the full base URL;
it expands to the project's GitHub release assets URL. A `file://` base URL is
supported for local mirrors and offline testing. Replace the example tag with
the one you reviewed.

Inputs:

- `AG_INSTALLER_CHANNEL=alpha` selects the channel. Any value other than `alpha`
  or `source` is rejected before any action.
- `AG_INSTALLER_BASE_URL` (or `AG_INSTALLER_RELEASE_TAG`) locates the assets. If
  neither is set, the installer aborts; it never guesses a version.

