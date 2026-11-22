# 24-hour fuzz gate runbook (issue #156)

English canonical (ADR-0008). This is an operational runbook for the scheduled
workflow `.github/workflows/fuzz-24h.yml` (RFC-0036). It is **not** gate evidence.
Preparing the runner, dispatching the workflow or reading this document does
**not** close issue #156, move the `Fuzz 24h` release-gate row to `pass`, or claim
that a 24-hour run happened. No such run was executed by the author of this
runbook. The human acceptance steps in section 6 remain the only way to close the
gate.

This runbook is the execution companion to:

- `docs/fuzz/24h-gate-verification.md` - canonical procedure, artifact contract
  and results template (the authority for acceptance).
- `docs/fuzz/self-hosted-runner.md` - host preparation, labels, network
  isolation and disk sizing.
- `docs/fuzz/windows-jsonl-schema.md` - evidence schema and `gate_label`.
- `docs/audits/fuzz-24h-workflow-structural-verification.md` - structural
  verification of the workflow on main.

## 1. Scope and non-goals

The gate is seven `cargo fuzz` targets, 86400 s (`-max_total_time=86400`) each,
on a quiet, dedicated Linux x86-64 host: `fuzz_lexer`, `fuzz_parser`,
`fuzz_compile`, `fuzz_resp`, `fuzz_storage_key`, `fuzz_signed_url`,
`fuzz_workers_payload`. Seven sequential windows need at least 168 h, so the
workflow runs one target per matrix job with `max-parallel: 1`.

Non-goals:

- Do not run the gate on a public GitHub-hosted runner or on a shared build host.
- Do not add any secret beyond the built-in `GITHUB_TOKEN`.
- Do not treat a short window, a green workflow or an automated comment as a
  gate pass. The JSONL `gate_label` for a full run is
  `GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED`, never a pass claim.

## 2. Exact preflight on the self-hosted runner

Run all commands as the dedicated unprivileged service account from a clean
checkout of the commit under test. Stop on any failure; do not start the gate on
an unverified host.

### 2.1 Host identity, resources and environment

```sh
. /etc/os-release && echo "$PRETTY_NAME"
uname -a
lscpu | grep '^Model name:' | head -1
nproc
free -h
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown
df -h <scratch-mount>
```

`<scratch-mount>` is the dedicated volume that holds the checkout, the
`CARGO_TARGET_DIR` and the corpora. Require at least 500 GiB free as a starting
allocation (not a measured requirement), and stop if free space drops below 20
percent. Confirm the runner is registered with exactly the labels `self-hosted`,
`linux`, `x64`, `fuzz-dedicated`; the workflow also refuses to run when
`runner.environment != 'self-hosted'`.

### 2.2 Toolchain and tooling

Use one dated nightly for all seven targets. The workflow default is
`nightly-2026-11-22`; a maintainer may override it via the repository variable
`FUZZ_NIGHTLY_TOOLCHAIN` or the `toolchain` input on manual dispatch.

```sh
rustc +nightly-2026-11-22 --version
cargo +nightly-2026-11-22 fuzz --version
gh --version
```

### 2.3 Workspace hygiene and lockfile

```sh
git status --porcelain          # must print nothing
git rev-parse HEAD              # record this commit
cargo metadata --locked --manifest-path fuzz/Cargo.toml >/dev/null && echo lock-ok
```

An empty `git status --porcelain` and `lock-ok` are mandatory. A dirty tree or a
stale `fuzz/Cargo.lock` invalidates the run.

### 2.4 Harness interface check (does not run a target)

```sh
bash -n scripts/run-fuzz-window.sh
bash scripts/run-fuzz-window.sh --help
FUZZ_TARGETS=bogus_target bash scripts/run-fuzz-window.sh 1 2>&1  # must exit 2
```

### 2.5 Short smoke on the host (NOT the gate)

```sh
FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 60
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl
```

Acceptance of the runner setup requires: empty `git status --porcelain`,
`lock-ok`, all three `--version` commands succeeding, and the smoke appending a
parseable JSONL record with `gate_label` `SHORT_WINDOW_NOT_GATE`. The smoke does
not satisfy issue #156.

## 3. The seven full-length runs

Preferred path: trigger `workflow_dispatch` on `.github/workflows/fuzz-24h.yml`
(choose the dated `toolchain`), or let the monthly `schedule` (`0 3 1 * *`)
start it. Each of the seven matrix jobs performs this step:

```sh
FUZZ_TARGETS=<matrix.target> bash scripts/run-fuzz-window.sh "$DURATION"
# DURATION=86400, exported at workflow level
```

### 3.1 The seven FUZZ_TARGETS commands

For a manual, host-local equivalent of what the matrix executes, run these one
at a time, each with an uninterrupted 86400 s window. Use the same dated
toolchain for every command and a clean tree at the recorded commit. Do not
overwrite an earlier run directory.

```sh
FUZZ_TARGETS=fuzz_lexer          FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_parser         FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_compile        FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_resp           FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_storage_key    FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_signed_url     FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_workers_payload FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
```

The harness validates every `FUZZ_TARGETS` name against the canonical seven and
exits 2 on an unknown name before any fuzzing starts. With `FUZZ_TARGETS` unset
it runs all seven in one invocation (the legacy short-window behaviour). Every
86400 s invocation produces exactly one full-length window.

### 3.2 Direct cargo-fuzz equivalent

The harness wraps the following commands; they are the literal targets the
verification package records. Use these only for crash reproduction or a manual
run, and capture the exit code of each explicitly.

```sh
cargo +nightly-2026-11-22 fuzz run fuzz_lexer           -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_parser          -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_compile         -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_resp            -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_storage_key     -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_signed_url      -- -max_total_time=86400
cargo +nightly-2026-11-22 fuzz run fuzz_workers_payload -- -max_total_time=86400
```

Rules that apply to both forms:

- One uninterrupted 86400 s per target. A log ending early (OOM, SIGKILL, host
  reboot, toolchain error) is not a completed run; restart that target in full.
- Do not use `set -e` in the surrounding loop: a non-zero target still needs its
  evidence collected, and a non-zero exit is a gate failure.
- Reproduce any crash with
  `cargo +nightly-2026-11-22 fuzz run <target> artifacts/<target>/<crash-file>`.

## 4. Artifact collection

The harness writes per-target logs, a start/end `timeline.txt`, `metadata.txt`,
crash copies and append-only JSONL records. The workflow uploads
`docs/fuzz/results/` and `fuzz/artifacts/` from every matrix job with
`if: always()` as artifact `fuzz-24h-<run_id>-<target>` (90-day retention,
`continue-on-error: true`, because the token may expire at hour 24). The
on-host evidence under `docs/fuzz/results/` is authoritative.

| Artifact | Path | Committed? |
| --- | --- | --- |
| Environment metadata | `docs/fuzz/results/<run>/metadata.txt` | yes |
| Start/end timeline | `docs/fuzz/results/<run>/timeline.txt` | yes |
| Full stdout+stderr per target | `docs/fuzz/results/<run>/<target>.log` | yes |
| JSONL evidence records | `docs/fuzz/results/windows.jsonl` | yes |
| Crash inputs, if any | `docs/fuzz/results/<run>/crashes/<target>/` | yes (crash inputs only) |
| Corpora | `fuzz/corpus/<target>/` (gitignored) | no; record file count, bytes, optional tarball SHA-256 |

For the workflow harness a per-target `.rc` file is not written: the real exit
code is recorded as `exit_code` in the JSONL target record, in the run record as
`all_exit_zero`, and as `END <target> rc=<N>` in `timeline.txt`. The manual loop
in `docs/fuzz/24h-gate-verification.md` writes `<target>.rc` files instead. When
filling the results template, transpose the authoritative `exit_code`/`rc`
values into the `Exit` column; never invent a value from a missing file.

Human collection steps after all seven jobs finish:

```sh
# Validate every JSONL line parses.
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl

# Confirm each target log has the final libFuzzer DONE and termination lines.
for t in fuzz_lexer fuzz_parser fuzz_compile fuzz_resp \
         fuzz_storage_key fuzz_signed_url fuzz_workers_payload; do
  grep -aE '^#[0-9]+[[:space:]]+DONE[[:space:]]' "docs/fuzz/results/<run>/$t.log" | tail -1
  grep -aE 'Done [0-9]+ runs in [0-9]+ second' "docs/fuzz/results/<run>/$t.log" | tail -1
done
```

Replace `<run>` with the actual on-host run directory. A missing `DONE` or
termination line, a non-zero `exit_code`, or any crash input is a failure; do
not aggregate a partial run into a pass.

## 5. Idempotent failure notice on issue #156

The `notify-failure` job runs only `if: failure()` of the fuzz matrix, on the
same self-hosted label set, with `contents: read` and `issues: write`. It uses
only `${{ github.token }}` (`GITHUB_TOKEN`) via `GH_TOKEN`. It posts at most one
comment per workflow run, keyed on the HTML marker
`<!-- fuzz-24h-gate run <run_id> -->`. Before commenting it reads existing
comments with `gh issue view 156 --json comments` and exits early if the marker
is already present, so a re-run or a retried job cannot duplicate the notice.
It then ensures the `blocked` label is present. The notice states that the
`Fuzz 24h` row stays `pending`, that #156 stays open, and gives the run URL and
artifact names; it never closes the issue and never edits the gate row.

A human can verify the check after a failed run:

```sh
gh issue view 156 --json comments --jq '.comments[].body' \
  | grep -F "<!-- fuzz-24h-gate run ${RUN_ID} -->"   # expect exactly one match
gh issue view 156 --json state,labels \
  --jq '{state:.state, labels:[.labels[].name]}'     # state OPEN, blocked present
```

Re-running the dispatch and repeating the check must not increase the match
count for the same `RUN_ID`. The automated notice is a failure record only; it
is never the evidence that closes the gate.

## 6. Human acceptance: gate row and README evidence steps

Only after all seven full-length runs are verified does a human perform the
acceptance steps. These are the closure criteria for #156:

1. All seven targets have one uninterrupted 86400 s run at the recorded commit.
2. Every `exit_code`/`rc` is 0 and no crash input exists, **or** every crash has
   a fix, a regression test and a fresh full-length re-run of the affected
   target.
3. Logs, metadata, timeline and a filled `RESULTS.md` (template in
   `docs/fuzz/24h-gate-verification.md`) are committed under
   `docs/fuzz/results/<run>/`, with real final `DONE` and termination lines
   pasted verbatim and rule 17 benchmark-style host metadata where applicable.
4. In `docs/audits/PRE_FASE5_RELEASE_GATE.md`, update the `Fuzz 24h` row with
   the date, commit and result path, then move its state from `pending` to
   `pass`. No other `pending` row is flipped by this gate.
5. In `README.md`, update the "Known limitations and release blockers" text so
   it no longer claims the 24-hour gate is unexecuted; link the archived result.
6. Confirm `docs/DEBT.md` DEBT-015 still points to #156 (frozen historical
   record; do not add new entries).
7. Close issue #156 with `Closes #156` in the change that carries the full-length
   evidence. Until every box above is checked, keep #156 open with `blocked` and
   the gate row `pending`, citing `Refs #156` in interim work.

## 7. Workflow invariants verified on main

The structural verification recorded in
`docs/audits/fuzz-24h-workflow-structural-verification.md` confirms that
`.github/workflows/fuzz-24h.yml` has: both `schedule` (`0 3 1 * *`) and
`workflow_dispatch` triggers; `runs-on: [self-hosted, linux, x64,
fuzz-dedicated]` plus a `runner.environment` guard in both jobs; top-level
`permissions: {}` with `contents: read` only for the gate job and `issues: write`
added only for the failure-notice job; two SHA-pinned actions; `DURATION=86400`;
a seven-target matrix with `max-parallel: 1`; and no secret beyond
`GITHUB_TOKEN`. This is a capability check, not a gate result.

## 8. References

- Issue #156 and RFC-0036 (`docs/rfc/RFC-0036-scheduled-fuzz-24h-gate.md`).
- `docs/fuzz/24h-gate-verification.md`, `docs/fuzz/windows-jsonl-schema.md`,
  `docs/fuzz/self-hosted-runner.md`.
- `docs/audits/fuzz-24h-workflow-structural-verification.md`.
- CLAUDE.md rules 16 (security), 17 (benchmark metadata), 22/28 (RFC for CI),
  26 (code-doc sync) and 36 (reproducibility).
