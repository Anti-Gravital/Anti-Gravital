# Lockfile user journey and reproducibility audit (wave 10)

## Scope and environment

This audit answers the questions a developer hits when trusting the committed
lockfiles: is every `Cargo.lock` up to date, do the documented `--locked`
commands actually work, and can the local automation scripts silently change a
lockfile mid-run.

- Date (UTC): 2026-11-22
- Commit audited: `72fa9d836a6de2f4f96429c11ffcff13142b2ef4`
- Branch: `community10/locks` (worktree under `worktrees10/locks`)
- Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`
- Host: Ubuntu 26.04.1 LTS, Linux 7.0.0-44-generic x86-64,
  12th Gen Intel Core i7-1255U, 12 logical CPUs
- Target-dir policy: every check used a private `CARGO_TARGET_DIR` under
  `/tmp/opencode/locks-work/` so no other agent's binaries were overwritten.
- Source policy: checks ran from fresh copies created with
  `git archive HEAD | tar -x` so only committed content was tested.

## Definitive list of lockfiles

Four `Cargo.lock` files are tracked in the repository. There are no lockfiles
under `tools/` or under the individually generated `templates/` directories
(the templates generate one at scaffold time, by design).

| Path | Workspace root | SHA-256 (committed) | `cargo metadata --locked --offline` | Time |
| --- | --- | --- | --- | --- |
| `Cargo.lock` | repo root (19 crates + examples + tests) | `5db3d3dd04fb5c3b3226c62bb95a254796e5a0e9f89fa99577c3762d4801613d` | PASS rc=0 | 3.447 s |
| `fuzz/Cargo.lock` | `fuzz/` | `6426df12e74e14bc853ba1862cf7c0800b4561282dffca7ee5db6d6037b801ec` | PASS rc=0 | 1.769 s |
| `examples/domain-link/Cargo.lock` | `examples/domain-link/` | `b3466b2b272eb9189b22e73f45e6a3c0ab043824f8fef9daf7a6d0037e550118` | PASS rc=0 | 0.561 s |
| `benchmarks/neon-real/Cargo.lock` | `benchmarks/neon-real/` | `390cabc1fb09a01f344a09c500186b36abf2ae62766304fee92b798e42f40941` | PASS rc=0 | 1.126 s |

Enumeration command:

```
find . -name Cargo.lock -not -path '*/target/*' | sort
git ls-files '*Cargo.lock'
```

`grep -rln '^\[workspace\]' --include=Cargo.toml .` confirms exactly four
workspace roots (root, `fuzz`, `examples/domain-link`,
`benchmarks/neon-real`), matching the four lockfiles one-to-one.

**Result: no stale lockfile exists at this commit.** All four resolve with
`--locked`, so a manifest change without a matching lock refresh is not hiding
in the tree today.

Real `cargo check --locked --offline` builds were also run from the clean copy
with a private target dir (not just resolution):

```
cargo check --locked --offline --manifest-path examples/domain-link/Cargo.toml   # rc=0,  1.7 s
cargo check --locked --offline --manifest-path benchmarks/neon-real/Cargo.toml  # rc=0, 127.9 s
cargo check --locked --offline --manifest-path fuzz/Cargo.toml                  # rc=0,  89.6 s
```

The root workspace is covered by the real `cargo install --locked` build below
and by the repository's existing `cargo build --locked --workspace` CI gate.

## Offline cache caveat (not a lock defect)

The first offline pass failed for the two small workspaces with download
errors, not with the `--locked` staleness error:

```
cargo metadata --manifest-path fuzz/Cargo.toml --locked --offline --format-version 1
# rc=101: error: failed to download `js-sys v0.3.99`
#         caused by: attempting to make an HTTP request, but --offline was specified

cargo metadata --manifest-path benchmarks/neon-real/Cargo.toml --locked --offline --format-version 1
# rc=101: error: failed to download `android_system_properties v0.1.5`
```

Resolution itself succeeded in both cases; only crate archives were missing
from the local registry cache. Running the same commands online downloaded the
missing archives and returned rc=0 (fuzz 3.84 s, neon-real 2.91 s). Re-running
offline afterwards gave the PASS times in the table above.

This also clarifies the note in `docs/fuzz/wave9-window-evidence.md` that
`cargo metadata ... --locked --offline` "cannot complete in this environment":
that is a registry-cache limitation, not evidence of a stale lock. The wave-9
author's conclusion (seven Linux fuzz targets build without those wasm crates)
is consistent with this audit.

## Reproduction: plain cargo silently rewrites a stale lock

The wave-9 branch already found and fixed one real stale lock:
`docs/fuzz/wave9-window-evidence.md` records that the first `cargo fuzz run`
regenerated `fuzz/Cargo.lock` (299 insertions, 19 deletions) after the workspace
was bumped to `0.1.0-alpha.2`. The regenerated lock is the one committed today.

The underlying cause is still reachable because the local scripts do not pass
`--locked`. Reproduced in a clean copy with a warm cache:

Fuzz workspace, package version changed `0.0.0` -> `0.0.1` in
`fuzz/Cargo.toml`:

```
cargo metadata --manifest-path fuzz/Cargo.toml --locked --offline --format-version 1
# rc=101: error: cannot update the lock file .../fuzz/Cargo.lock because --locked was passed

cargo metadata --manifest-path fuzz/Cargo.toml --offline --format-version 1
# rc=0, no warning; fuzz/Cargo.lock changed:
# before 6426df12e74e14bc853ba1862cf7c0800b4561282dffca7ee5db6d6037b801ec
# after  22834425b1b6928697575eea70fea0fefe12bbb121edd426953eb9bfcd08de5b
# diff: -version = "0.0.0"  +version = "0.0.1" under [[package]] ag-dsl-fuzz
```

Root workspace, `crates/ag-core/Cargo.toml` version source changed from
`version.workspace = true` to `version = "0.1.0-alpha.3"`:

```
cargo metadata --manifest-path ./Cargo.toml --locked --offline --format-version 1
# rc=101: error: cannot update the lock file .../Cargo.lock because --locked was passed

cargo metadata --manifest-path ./Cargo.toml --offline --format-version 1
# rc=0, no warning; Cargo.lock changed:
# before 5db3d3dd04fb5c3b3226c62bb95a254796e5a0e9f89fa99577c3762d4801613d
# after  36379f830acdbb794bce0a5991084b71b8d094506fb51c1411a257940d4a47c2
```

The two plain commands above are the same resolution mechanism used by the
script invocations listed next. This is the exact failure mode wave 9 hit.

## Scripts that can mutate a lockfile

| Script / command | Invocation | `--locked`? | Effect |
| --- | --- | --- | --- |
| `scripts/run-fuzz-window.sh:144` | `cargo +"$FUZZ_TOOLCHAIN" fuzz run "$t" -- -max_total_time=...` | no | can rewrite `fuzz/Cargo.lock` on manifest drift |
| `scripts/run-benchmarks.sh:50,53,57` | `cargo bench -p ag-core -p ag-workers ...` | no | can rewrite root `Cargo.lock` on manifest drift |
| `install.sh:265` | `cargo build --workspace --release` | no | can rewrite root `Cargo.lock` before the locked install |
| `scripts/run-fuzz-window.sh:89` | `cargo fuzz --version` | n/a | read-only version probe |
| `scripts/release-smoke.sh:160` | `cargo build --workspace --locked --offline` | yes | safe |
| `scripts/release-smoke.sh:234-239` | `cargo generate-lockfile --offline`, then `cargo check --locked --offline`, `cargo build --locked --offline` | yes for check/build | safe; `generate-lockfile` intentionally creates the lock of a fresh scaffold |
| `scripts/validate-templates.sh:155-161` | `cargo generate-lockfile --offline` then `cargo check --offline --locked` | yes for check | safe; same intentional fresh-scaffold flow |
| `install.sh:266`, `install.ps1:203` | `cargo install --path crates/ag-cli --locked` | yes | safe |

Wave 9 did verify that a full seven-target run leaves the now-committed
`fuzz/Cargo.lock` unchanged (SHA `6426df1...` before and after). That confirms
the current in-sync state; it does not protect a future manifest change, which
is why the missing `--locked` guard is still a defect.

## Documented `--locked` user commands

### `cargo install --path crates/ag-cli --locked`

Run from a clean checkout at the audited HEAD with a private `CARGO_TARGET_DIR`
and a temporary `CARGO_INSTALL_ROOT` (no shared binaries overwritten):

```
cargo install --path crates/ag-cli --locked
```

- Start (UTC): `2026-11-22T13:35:15Z`
- End (UTC): `2026-11-22T13:39:39Z`
- Exit code: `0`
- Elapsed: `264 s` (cargo reported "Finished `release` profile ... in 4m 23s")
- Installed: `ag-cli v0.1.0-alpha.2`, executable `ag`
- `ag --version`: `ag 0.1.0-alpha.2`
- Root `Cargo.lock` SHA-256 after the run:
  `5db3d3dd04fb5c3b3226c62bb95a254796e5a0e9f89fa99577c3762d4801613d`,
  unchanged from the baseline, so `--locked` held during a real release build.
  The 3 unrelated modified files present in that checkout belong to other
  roles; neither of the two tracked lockfiles there was touched.

A first attempt hit a 900 s harness ceiling while linking the `ag` binary under
the release profile (`lto=fat`, `codegen-units=1`) during heavy host CPU
contention and was terminated without a result. The successful run reused the
warm private target dir, so 264 s is a warm-target time, not a cold-build time.
The command used the network for the crates.io index update; it was not an
offline install.

Note that `install.sh` runs `cargo build --workspace --release` without
`--locked` immediately before this locked install; see finding 2.

### `scripts/release-smoke.sh`

The script uses `cargo build --workspace --locked --offline` for the workspace
and `cargo check/build --locked --offline` for the generated scaffold (with an
explicit `cargo generate-lockfile --offline` first, which is correct for a
freshly generated project with no lockfile). No committed lockfile is mutated
by this script. The full script also starts `ag dev` and curls a local endpoint;
that is outside the lockfile scope of this audit.

### `scripts/validate-templates.sh`

Each generated template starts without a lockfile. The script runs
`cargo generate-lockfile --offline` and then `cargo check --offline --locked`
(and optionally `--all-targets`). This is a deliberate scaffold-lock generation
followed by a reproducibility assertion, not drift on a committed lock.

## Findings

1. **No stale committed lockfile.** All four locks resolve under `--locked`.
   No issue opened.
2. **Local automation scripts omit `--locked` and can silently rewrite
   committed locks.** Reproduced above; wave 9 has a real precedent. Tracked
   in issue #199 (labels `tech-debt`, `workflow`, `p2`). The unlocked
   `cargo build --workspace --release` in `install.sh:265` is documented as an
   additional instance in the same issue.
3. **The offline metadata note in `docs/fuzz/wave9-window-evidence.md` is a
   cache limitation, not a lock problem.** Recorded here for accuracy; no
   issue opened because the underlying claim about the fuzz targets is correct.

## Remediation

- Add `--locked` to every cargo invocation in `scripts/run-fuzz-window.sh`,
  `scripts/run-benchmarks.sh` and the release-build line of `install.sh`.
- On a `--locked` failure, fail fast with a message telling the contributor to
  commit the regenerated lockfile instead of letting cargo mutate the tree.
- Keep an explicit lock SHA-256 before/after assertion in the fuzz and
  benchmark runners so a gate can never publish evidence from a mutated tree.
- Add the four workspaces to a scheduled `cargo metadata --locked --offline`
  check so a future manifest change is caught even outside CI's workspace gate.

## Verification of this audit

- `git diff --check` on the change set.
- Content scans: no emoji, no AI-tool attribution, no prohibited third-party
  markers, all commit messages and the issue title <= 256 characters.
- No issue was closed and no branch was pushed or merged by this role.
