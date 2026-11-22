# Installer user journey - wave 10 real terminal test

Date: 2026-11-22. Role: `install` (wave 10 terminal QA). No code was changed;
this document records what actually happened when the documented installer was
executed as a normal user.

## Scope and method

Documentation read before running anything:

- `install.sh` (source and alpha channels),
- `docs/manual/04-instalacion-y-onboarding.md`,
- `docs/security/INSTALLATION_INTEGRITY.md`,
- `README.md` install/try sections.

All runs used real shell commands in an isolated copy of the repository under
`/home/angel-nereira/tmp-install-journey/work/Anti-Gravital` (the copy stands in
for the documented `git clone`). Stdout and stderr were captured verbatim with
`time`; exit codes and the resulting files were inspected afterwards.

Environment (recorded so timings can be interpreted):

| Item | Value |
|------|-------|
| OS / arch | Linux x86_64 |
| rustc / cargo | 1.99.1 (58cf8f9c3 2026-10-13) / 1.99.1 |
| Workspace version | 0.1.0-alpha.2 (`Cargo.toml`) |
| SHA-256 of `install.sh` | `1ebc3ad035d3f00f9a46149d57c910c77dffacc3bacaa5bee13a22548e92d08a` |
| Destination isolation | source run used `CARGO_INSTALL_ROOT` to avoid writing over other agents' `ag`; alpha runs used clean `HOME` values |
| Caches | shared warm Cargo registry; source run used the shared release target dir, but a different source path invalidated most fingerprints, so it was effectively a cold release build |
| Contention | several other wave-10 agents compiled Rust at the same time, inflating wall-clock times |

No command in this journey required elevated privileges and no system package
manager was touched.

## Documented commands under test

From `docs/manual/04-instalacion-y-onboarding.md` and `README.md`:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
ag --version
```

The alpha variant from `docs/security/INSTALLATION_INTEGRITY.md`:

```bash
AG_INSTALLER_CHANNEL=alpha \
AG_INSTALLER_BASE_URL=<release-assets-url> \
bash install.sh
```

## Test 1 - source channel, clean HOME

Exact command (the only deviation from the documented command is
`CARGO_INSTALL_ROOT`, added to isolate the installed binary from other agents;
`RUSTUP_HOME` was set so the installed toolchain remained visible under the
clean `HOME`):

```bash
env HOME=/home/angel-nereira/tmp-install-journey/home-src \
    RUSTUP_HOME=/home/angel-nereira/.rustup \
    CARGO_HOME=/home/angel-nereira/.cargo \
    CARGO_INSTALL_ROOT=/home/angel-nereira/tmp-install-journey/home-src/.cargo \
    CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
    bash -c 'TIMEFORMAT="ELAPSED real=%3R user=%3U sys=%3S"; { time bash install.sh ; } 2>&1'
```

Exit code: 0. Elapsed (bash `time`): `real=1282.132 user=2604.446 sys=100.624`
(21 min 22 s wall clock). The Cargo phase lines in the output report
`17m 10s` for `cargo build --workspace --release` and `4m 10s` for the separate
`cargo install`, which accounts for the total.

Relevant verbatim lines (cargo compile lines between them are omitted for
brevity; the full capture is 514 lines):

```
[ag-install] WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md.
[ag-install] Detected Rust 1.99.1 (minimum required: 1.99.1).
[ag-install] Building the workspace in release mode...
    Finished `release` profile [optimized] target(s) in 17m 10s
[ag-install] Installing the ag CLI into ~/.cargo/bin ...
  Installing ag-cli v0.1.0-alpha.2 (/home/angel-nereira/tmp-install-journey/work/Anti-Gravital/crates/ag-cli)
warning: default toolchain implicitly overridden with `1.99.1-x86_64-unknown-linux-gnu` by rustup toolchain file
    Updating crates.io index
    Finished `release` profile [optimized] target(s) in 4m 10s
  Installing /home/angel-nereira/tmp-install-journey/home-src/.cargo/bin/ag
   Installed package `ag-cli v0.1.0-alpha.2 (/home/angel-nereira/tmp-install-journey/work/Anti-Gravital/crates/ag-cli)` (executable `ag`)
warning: be sure to add `/home/angel-nereira/tmp-install-journey/home-src/.cargo/bin` to your PATH to be able to run the installed binaries
[ag-install] Installation complete.
[ag-install] Ensure ~/.cargo/bin is on your PATH, then run: ag --help
ELAPSED real=1282.132 user=2604.446 sys=100.624
```

Installed file: `home-src/.cargo/bin/ag`, mode 0755, 9,696,976 bytes, stripped
ELF 64-bit x86-64 executable. No other file was written to that bin directory.

`ag --version` from the isolated bin directory:

```
ag 0.1.0-alpha.2
```

`ag --help` ran successfully and listed the available commands. The install is
functional. Note how the installer's own line says `~/.cargo/bin` while Cargo
prints the real destination in the same output; that is issue #204. Also note
that the workspace build compiled examples and test fixtures (for example
`todo-api`, `ai-backend`, `ag-generated-rust-fixture`) before the CLI was
installed, and that `cargo install` recompiled the core crates in a separate
target directory, so this single install performed two full release builds.

## Test 2 - alpha channel with a local file:// release, valid checksum

A fake release directory was created with a 62-byte shell script as the
`ag-x86_64-unknown-linux-gnu` asset and a real SHA-256 `checksums.txt`:

```bash
printf '#!/bin/sh\necho "ag 0.1.0-alpha.2 (fake alpha-channel binary)"\n' \
  > ag-x86_64-unknown-linux-gnu
chmod 0755 ag-x86_64-unknown-linux-gnu
sha256sum ag-x86_64-unknown-linux-gnu > checksums.txt
```

Exact command:

```bash
env HOME=/home/angel-nereira/tmp-install-journey/home-alpha-valid \
    AG_INSTALLER_CHANNEL=alpha \
    AG_INSTALLER_BASE_URL=file:///home/angel-nereira/tmp-install-journey/releases/v0.1.0-alpha.2 \
    bash -c 'TIMEFORMAT="ELAPSED real=%3R user=%3U sys=%3S"; { time bash install.sh ; } 2>&1'
```

Exit code: 0. Elapsed: `real=0.410 user=0.027 sys=0.043`.

Verbatim output:

```
[ag-install] WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md.
[ag-install] Alpha channel: checksum verification only (no signing key, no signature).
[ag-install] Alpha channel: downloading checksum manifest from file:///home/angel-nereira/tmp-install-journey/releases/v0.1.0-alpha.2/checksums.txt
[ag-install] Alpha channel: downloading binary from file:///home/angel-nereira/tmp-install-journey/releases/v0.1.0-alpha.2/ag-x86_64-unknown-linux-gnu
[ag-install] Checksum verified for ag-x86_64-unknown-linux-gnu.
[ag-install] Installed ag to /home/angel-nereira/tmp-install-journey/home-alpha-valid/.cargo/bin/ag.
[ag-install] Installation complete.
[ag-install] Ensure /home/angel-nereira/tmp-install-journey/home-alpha-valid/.cargo/bin is on your PATH, then run: ag --version
ELAPSED real=0.410 user=0.027 sys=0.043
```

Installed file: `home-alpha-valid/.cargo/bin/ag`, mode 0755, 62 bytes.

`ag --version` output:

```
ag 0.1.0-alpha.2 (fake alpha-channel binary)
```

The binary that passed verification is the one that was installed and executed.
No build and no Cargo action occurred on this channel.

## Test 3 - alpha channel with an invalid checksum (fail closed)

`checksums.txt` was replaced with an all-zero hash for the same binary name.

Exact command: same as test 2 with
`HOME=/home/angel-nereira/tmp-install-journey/home-alpha-bad` and
`AG_INSTALLER_BASE_URL=file:///home/angel-nereira/tmp-install-journey/releases/bad-checksum`.

Exit code: 1. Elapsed: `real=0.298 user=0.015 sys=0.041`.

Verbatim output (excerpt):

```
[ag-install] WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback (no out-of-band trust anchor). See docs/security/INSTALLATION_INTEGRITY.md.
[ag-install] Alpha channel: checksum verification only (no signing key, no signature).
[ag-install] Alpha channel: downloading checksum manifest from file:///home/angel-nereira/tmp-install-journey/releases/bad-checksum/checksums.txt
[ag-install] Alpha channel: downloading binary from file:///home/angel-nereira/tmp-install-journey/releases/bad-checksum/ag-x86_64-unknown-linux-gnu
[ag-install] Checksum mismatch for ag-x86_64-unknown-linux-gnu: expected 0000000000000000000000000000000000000000000000000000000000000000, got 24faeca31cbcddb3fa714d6b14ad3c10b386301a75e0b47c6127860997ffb5ec.
ELAPSED real=0.298 user=0.015 sys=0.041
```

After the failure, `home-alpha-bad/.cargo/bin` did not exist: no binary was
left behind. This is the documented fail-closed behavior.

## Test 4 - alpha channel failure modes

All runs exited 1 and wrote nothing to the install directory.

| Case | Exact observable message (after the standard WARNING line) | Exit |
|------|------------------------------------------------------------|------|
| No `AG_INSTALLER_BASE_URL` / `AG_INSTALLER_RELEASE_TAG` | `AG_INSTALLER_CHANNEL=alpha requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG.` | 1 |
| `AG_INSTALLER_CHANNEL=beta` | `Unknown AG_INSTALLER_CHANNEL 'beta' (expected 'source' or 'alpha').` | 1 |
| `checksums.txt` without an entry for the target | `No checksum entry for ag-x86_64-unknown-linux-gnu in checksums.txt.` | 1 |
| Binary missing from the release directory | raw `cp: cannot stat '.../ag-x86_64-unknown-linux-gnu': No such file or directory` followed by `Could not download the binary from file://...` | 1 |
| `AG_INSTALLER_BINARY_CHECKSUMS` pointing at a tampered local manifest | `Checksum mismatch for ag-x86_64-unknown-linux-gnu: expected 0000..., got 24fa...` | 1 |

The out-of-band manifest path was also tested positively: with
`AG_INSTALLER_BINARY_CHECKSUMS` set to a correct manifest and a release
directory that had **no** `checksums.txt`, the install succeeded (exit 0) and
the log stated `using local binary checksum manifest ... (out-of-band)`. This
matches `docs/security/INSTALLATION_INTEGRITY.md`.

Two documented overrides were checked as well:

- `AG_INSTALLER_MANIFEST=SHA256SUMS.txt` (with a release directory containing
  that file instead of `checksums.txt`) exited 0; the log showed the custom
  manifest URL and the install succeeded.
- `AG_INSTALLER_TARGET=aarch64-unknown-linux-gnu` exited 1 with `Could not
  download the binary from .../ag-aarch64-unknown-linux-gnu` because that asset
  was absent from the fake release, confirming the override changes the asset
  name as documented.

## Test 5 - installer self-integrity (AG_INSTALLER_CHECKSUMS)

| Case | Observable result | Exit |
|------|-------------------|------|
| Manifest with the correct `install.sh` hash | `Integrity verified for install.sh against <manifest>.` then the alpha install proceeded | 0 |
| Manifest with an all-zero hash | `Integrity check failed for install.sh: expected 0000..., got 1ebc...` | 1 |
| Manifest path that does not exist | `Integrity manifest not found: <path>.` | 1 |
| Variable unset (all other tests) | `WARNING: AG_INSTALLER_CHECKSUMS is unset; using the development-only local fallback ...` | n/a |

The fail-closed behavior is real: a mismatch or missing manifest aborts before
any download or install action. The warning is honest about the lack of a trust
anchor and matches the documented "known limitations".

## Test 6 - Rust version gate

With a fake `rustc` that reported `1.90.0` earlier on `PATH`, `bash install.sh`
exited 1 with:

```
[ag-install] Detected Rust 1.90.0 (minimum required: 1.99.1).
[ag-install] Rust 1.99.1 or newer is required. Run 'rustup update stable' and retry.
```

The minimum-version check works as documented.

## What worked

- Both channels are selectable and the alpha channel rejects unknown values.
- `file://` mirrors work for offline/local testing, as documented.
- The SHA-256 verification is genuinely fail-closed: wrong hash, missing entry,
  missing binary, missing manifest and tampered out-of-band manifest all abort
  with exit 1 before any file is written to the install directory.
- The verified binary is the one installed; `ag --version` ran successfully.
- `AG_INSTALLER_BINARY_CHECKSUMS` correctly suppresses the download of
  `checksums.txt`.
- The installer never used privileges and never touched a package manager.
- The Rust version gate behaves as documented.

## What was confusing or costly

1. The source channel performs two independent compilations: first
   `cargo build --workspace --release` for the entire workspace (including
   examples and test fixtures), then `cargo install --path crates/ag-cli --locked`
   which rebuilds `ag-cli` and its dependencies in a separate temporary target
   directory. The release profile uses `lto = "fat"` and
   `codegen-units = 1` in `[profile.release]`, while
   several other agents were compiling concurrently. A user who only wants the
   `ag` binary waits for all of it (21 min 22 s wall clock here, of which
   17 min 10 s was the workspace build and 4 min 10 s the second build). This
   is documented behavior, not a broken behavior, but it is a real cost worth
   reporting.
2. The source channel prints `Installing the ag CLI into ~/.cargo/bin ...` and
   `Ensure ~/.cargo/bin is on your PATH, then run: ag --help` with the path
   hardcoded, even though `cargo install` honors `CARGO_HOME` /
   `CARGO_INSTALL_ROOT`. Reproduced independently with a stub `cargo`/`rustc`
   and `CARGO_INSTALL_ROOT=/home/angel-nereira/tmp-install-journey/home-stubsrc/.cargo`:
   the script exited 0 but still printed the literal `~/.cargo/bin` lines. The
   alpha channel prints the resolved `BIN_DIR` correctly, so the two channels
   are inconsistent. See issue #204.
3. When the binary is missing from a `file://` release, the raw `cp: cannot
   stat` error leaks to the user before the installer's own message. The
   installer still fails closed; the extra line is only cosmetic.
4. The warning about `AG_INSTALLER_CHECKSUMS` being unset is printed on every
   run. It is accurate but easy to overlook; users skimming output may miss the
   fact that no trust anchor is present.
5. `docs/manual/04-instalacion-y-onboarding.md` still says the released alpha is
   `v0.1.0-alpha.1` and that `ag --version` reports `0.1.0-alpha.1`, while both
   `README.md` and the workspace `Cargo.toml` say `0.1.0-alpha.2`. A reader
   following the manual would think the install produced the wrong binary.
   See issue #203.

## Issues opened

- #203 - `docs/manual/04 still states alpha.1 and expected ag --version
  0.1.0-alpha.1` (documentation, p3, good first issue). Opened from the doc/code
  contradiction above; nothing was closed.
- #204 - `Source installer reports ~/.cargo/bin even with a custom Cargo
  install root` (bug, workflow, p3, good first issue). Opened from the
  reproduced misleading message above; nothing was closed.
- #206 - `Source installer runs two full release builds with no time
  expectation` (enhancement, workflow, p3). Opened from the measured 21 min 22 s
  source run and the two distinct Cargo builds; nothing was closed.

All three issues were opened with verifiable "Done when" sections. This branch
does not close any issue; the coordinator decides closure after a fix is
verified.

## Verification of this worktree

- `git diff --check` was run before committing.
- Content scans for prohibited AI-tool evidence, prohibited brand names and
  emojis were run over the changed files.
- No shared code was modified; only this evidence document, the CHANGELOG and
  (optionally) a descriptor were added.
