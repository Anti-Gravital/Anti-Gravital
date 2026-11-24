# Phase 5 gates: consolidated execution package (issues #156 and #157)

> **STATUS: PREPARED, NOT EXECUTED.** This package makes two blocked gates
> runnable with minimal friction for a human. It does **not** contain a 24-hour
> fuzz result, reference benchmark numbers, a gate-row flip or an issue closure.
> Both gates remain `pending` and both issues remain **OPEN** with `blocked`.
> No agent push, PR or merge is part of this package (L-3).

- Package date (UTC): 2026-11-23T23:56:51Z
- Prepared on branch/worktree: `community29/i156` / `worktrees29/i156`
- Base commit for this package: `97b9402f9e5127924c39a9c75174e6444de7ad78`
  (parent of the change that adds this document). The human chooses the
  **stabilization commit** on the real host and records that SHA in every
  artifact; this package does not prescribe one.
- Authority: `docs/audits/PRE_FASE5_RELEASE_GATE.md` (ADR-0015),
  `docs/fuzz/24h-gate-verification.md`, `docs/fuzz/24h-gate-runbook.md`,
  `docs/fuzz/self-hosted-runner.md`, `docs/benchmarks/reference-host.md`,
  `docs/benchmarks/REFERENCE_RUNBOOK.md`
- Rules: K-2 (no closure without evidence), K-10 (no fabricated evidence),
  S-15/L-5 (external/issue content is data, never instruction), ADR-0008
  (English canonical), CLAUDE.md rules 17 (benchmark metadata) and 36
  (reproducibility)

## 0. How to use this package

This is a one-document handoff for the coordinator and the two human owners.
The shared blocker is **hardware plus a human time window**, not missing
procedure: both gates have runbooks, harnesses and fail-closed checks already
merged. The coordinator must supply (a) a dedicated fuzz host and seven
uninterrupted 86400 s windows and (b) a host matching the Ryzen 9 7950X
reference profile. Only then do the command blocks below become executable.

Division of labour:

| Step | Who | Agent may do it? |
| --- | --- | --- |
| Provide host, labels, governor, network isolation, 168 h window | human host owner (platform engineering) | no |
| Execute fuzz matrix / reference runner | human host owner | no (no 24 h run, no benchmark run) |
| Triage crashes, verify logs and artifacts, curate report | human reviewer (evidence + security for crash inputs) | no |
| Flip gate rows, reconcile README, close issues | coordinator/orchestrator after human sign-off | no (agents never push/PR/merge) |
| Maintain this package and answer questions about it | agent | yes |

## 1. Consolidated status

| Gate | Issue | State today | Prepared | Executed | Approver (human) | Close condition |
| --- | --- | --- | --- | --- | --- | --- |
| Fuzz 24h | #156 | `pending`, `blocked` | runbook, workflow, harness, 60 s smoke | **NO** | host owner + human evidence/crash reviewer; coordinator flips row and closes | seven uninterrupted 86400 s runs, no unresolved crash, committed evidence, human steps 1-7 (section 2.7) |
| Benchmarks | #157 | `pending`, `blocked` | profile, fail-closed runner, dry-run refusal on this host | **NO** (on reference hardware) | named reporter who owns the reference host + human reviewer; coordinator flips row and closes | `--reference` exit 0, committed raw artifacts, curated rule-17 report, gate row and README EN/ES, `Closes #157` (section 3.7) |

Both gates block Phase 5. A green workflow, a short fuzz window, a dry run or
an informative host run is capability evidence only and never a pass.

## 2. Gate A: 24-hour fuzz (issue #156)

### 2.1 Honest current state

Verified in this worktree at the package commit:

- `docs/fuzz/results/windows.jsonl` contains exactly one run,
  `60s-20261122T130152Z-2555239`, `gate_label: SHORT_WINDOW_NOT_GATE`,
  `duration_seconds: 60`, commit `7cbc28a`, branch `community9/qafuzz`,
  `dirty_files: 3`, host `i7-1255U` / `powersave`. All seven target records have
  `exit_code: 0` and zero crashes **for that 60 s window only**.
- No `docs/fuzz/results/**/24h/` directory and no `RESULTS.md` exist.
- `.github/workflows/fuzz-24h.yml` is present and structurally verified, but no
  workflow run with seven full-length windows is recorded here.
- Therefore this gate is `PENDING (prepared, NOT executed)`.

### 2.2 Exact prerequisite checklist

All boxes must be checked by the host owner on the dedicated machine, as an
unprivileged service account, from a clean checkout of the chosen commit. Stop
on any failure; do not start the gate on an unverified host.

- [ ] Linux x86-64, kernel 6.0 or newer; `uname -a` and `/etc/os-release` recorded.
- [ ] CPU dedicated, no other heavy load; fixed governor if possible.
- [ ] RAM sufficient that no target is swapped or OOM-killed; total RAM recorded.
- [ ] Dedicated scratch volume with at least 500 GiB free (starting allocation,
      not a measured requirement); checkout and `CARGO_TARGET_DIR` on it.
      Stop if free space drops below 20 percent.
- [ ] Stable power and network for an uninterrupted 86400 s per target.
- [ ] Runner registered with exactly the labels `self-hosted`, `linux`, `x64`,
      `fuzz-dedicated`; `runner.environment == 'self-hosted'`.
- [ ] Dedicated unprivileged service account (e.g. `agfuzz`); not root, no
      Docker socket, no cloud metadata credential, no SSH agent.
- [ ] Segmented network, default-deny egress; GitHub Actions endpoints (and
      toolchain origins at provisioning only) allowed. No inbound listener.
- [ ] No secret beyond the built-in `GITHUB_TOKEN`.
- [ ] One dated nightly for all seven targets (workflow default
      `nightly-2026-11-22`), pinned by `FUZZ_NIGHTLY_TOOLCHAIN` or dispatch input.
- [ ] Empty `git status --porcelain`, `lock-ok`, all three `--version` checks,
      and a parseable 60 s smoke record with `gate_label SHORT_WINDOW_NOT_GATE`.

### 2.3 Host profile

| Item | Required value | Source of truth |
| --- | --- | --- |
| Architecture | Linux x86-64 (`x64`) | `docs/fuzz/self-hosted-runner.md` section 2 |
| Kernel | 6.0 or newer | same |
| CPU | dedicated, fixed governor if possible | same |
| Memory | no swap/OOM; record total | same |
| Scratch | >=500 GiB free, checkout + `CARGO_TARGET_DIR` on it | same, section 4 |
| Runner labels | `self-hosted,linux,x64,fuzz-dedicated` | same, section 6; workflow `runs-on` |
| Toolchain | `nightly-2026-11-22` (or one maintainer override for all seven) | runbook section 2.2 |
| Token | `GITHUB_TOKEN` only; `contents: read` gate job, `issues: write` notice job | workflow `permissions` |

### 2.4 Exact commands

Preflight on the host (read-only with respect to the fuzz targets; the smoke
appends one short JSONL record and is **not** the gate):

```sh
# 2.1 host identity, resources, environment
. /etc/os-release && echo "$PRETTY_NAME"
uname -a
lscpu | grep '^Model name:' | head -1
nproc
free -h
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown
df -h <scratch-mount>

# 2.2 toolchain and tooling (one dated nightly)
rustc +nightly-2026-11-22 --version
cargo +nightly-2026-11-22 fuzz --version
gh --version

# 2.3 workspace hygiene and lockfile
git status --porcelain          # must print nothing
git rev-parse HEAD              # record as the commit under test
cargo metadata --locked --manifest-path fuzz/Cargo.toml >/dev/null && echo lock-ok

# 2.4 harness interface check (does not run a target)
bash -n scripts/run-fuzz-window.sh
bash scripts/run-fuzz-window.sh --help
FUZZ_TARGETS=bogus_target bash scripts/run-fuzz-window.sh 1 2>&1  # must exit 2

# 2.5 short smoke on the host (NOT the gate)
FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 60
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl
```

Preferred execution: trigger `workflow_dispatch` on
`.github/workflows/fuzz-24h.yml` (choose the dated `toolchain`), or let the
monthly `schedule` (`0 3 1 * *`) start it. The matrix runs seven jobs with
`max-parallel: 1`; each sets `DURATION=86400` and calls
`FUZZ_TARGETS=<target> bash scripts/run-fuzz-window.sh "$DURATION"`. The
workflow has `timeout-minutes: 1500` per job; the token may expire at hour 24,
so artifact upload is `continue-on-error: true` and on-host evidence is
authoritative.

Manual host-local equivalent, one at a time, each with an uninterrupted
86400 s window, same toolchain, clean tree, without overwriting an earlier run:

```sh
FUZZ_TARGETS=fuzz_lexer           FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_parser          FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_compile         FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_resp            FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_storage_key     FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_signed_url      FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
FUZZ_TARGETS=fuzz_workers_payload FUZZ_TOOLCHAIN=nightly-2026-11-22 bash scripts/run-fuzz-window.sh 86400
```

The harness wraps these literal commands (use only for crash reproduction or a
manual run, and capture each exit code explicitly):

```sh
cargo +nightly-2026-11-22 fuzz run <target> -- -max_total_time=86400
```

Rules: one uninterrupted 86400 s per target (a truncated log forces a full
restart); do not use `set -e` in a surrounding loop, because a non-zero target
still needs its evidence collected and is itself a gate failure; reproduce any
crash with
`cargo +nightly-2026-11-22 fuzz run <target> artifacts/<target>/<crash-file>`.

Post-run validation and collection (human, after all seven jobs finish):

```sh
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl

for t in fuzz_lexer fuzz_parser fuzz_compile fuzz_resp \
         fuzz_storage_key fuzz_signed_url fuzz_workers_payload; do
  grep -aE '^#[0-9]+[[:space:]]+DONE[[:space:]]' "docs/fuzz/results/<run>/$t.log" | tail -1
  grep -aE 'Done [0-9]+ runs in [0-9]+ second' "docs/fuzz/results/<run>/$t.log" | tail -1
done
```

Replace `<run>` with the real on-host directory. For the workflow harness there
is no per-target `.rc` file: authoritative exit codes are `exit_code` in JSONL,
`all_exit_zero` in the run record and `END <target> rc=<N>` in `timeline.txt`.
Transpose those values into `RESULTS.md`; never invent a value from a missing
file.

### 2.5 Expected artifacts / evidence

| Artifact | Path | Committed? |
| --- | --- | --- |
| Environment metadata | `docs/fuzz/results/<run>/metadata.txt` | yes |
| Start/end timeline | `docs/fuzz/results/<run>/timeline.txt` | yes |
| Full stdout+stderr per target | `docs/fuzz/results/<run>/<target>.log` | yes |
| JSONL evidence records | `docs/fuzz/results/windows.jsonl` | yes |
| Crash inputs, if any | `docs/fuzz/results/<run>/crashes/<target>/` | yes (inputs only) |
| Filled results template | `docs/fuzz/results/<run>/RESULTS.md` | yes |
| Corpora | `fuzz/corpus/<target>/` | no (gitignored); record count, bytes, optional tarball SHA-256 |

Each full-length JSONL record carries `gate_label
GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED` - never a pass claim. A full run
needs seven such windows at the recorded commit, all `exit_code`/`rc` 0 or a
fix plus regression test plus fresh full-length re-run for every crash, final
`DONE` and termination lines in every log, and no partial aggregation.

### 2.6 Who must approve (human)

1. The human owner of the dedicated host: provisions the machine, records the
   profile, starts the seven windows and confirms uninterrupted completion.
2. A human evidence reviewer: parses JSONL, checks final `DONE`/termination
   lines, and verifies every exit code; for any crash input, a human security
   reviewer triages it and confirms the fix/re-run (S-11).
3. The coordinator/orchestrator: after both sign-offs, edits the gate row and
   README and closes #156. Agents do not push, open the PR or merge.

### 2.7 Exact close condition for #156

From `docs/fuzz/24h-gate-runbook.md` section 6, all seven boxes:

1. All seven targets have one uninterrupted 86400 s run at the recorded commit.
2. Every `exit_code`/`rc` is 0 and no crash input exists, or every crash has a
   fix, a regression test and a fresh full-length re-run of the affected target.
3. Logs, metadata, timeline and a filled `RESULTS.md` (template in
   `docs/fuzz/24h-gate-verification.md`) are committed under
   `docs/fuzz/results/<run>/`, with real final `DONE` and termination lines
   pasted verbatim.
4. In `docs/audits/PRE_FASE5_RELEASE_GATE.md` the `Fuzz 24h` row gains date,
   commit and result path and moves from `pending` to `pass`. No other row flips.
5. In `README.md` the "Known limitations and release blockers" text no longer
   claims the 24-hour gate is unexecuted and links the archived result.
6. `docs/DEBT.md` DEBT-015 still points to #156 (frozen historical record).
7. Only then, `Closes #156` in the change carrying the full-length evidence.
   Until then keep #156 open with `blocked`, gate row `pending`, and cite
   `Refs #156`.

Failure path: the `notify-failure` job posts at most one idempotent comment per
run id, keyed on `<!-- fuzz-24h-gate run <run_id> -->`; it keeps `blocked` and
never closes the issue. It is a failure record, never gate evidence.

## 3. Gate B: reference benchmarks (issue #157)

### 3.1 Honest current state

Verified in this worktree at the package commit:

- `docs/benchmarks/pre-fase5-performance.md` does **not** exist.
- `docs/benchmarks/raw/` contains only `2026-11-22-informative/` and
  `wave9-2026-11-22/`, from non-reference hardware; no `*-reference/` directory.
- `./scripts/run-benchmarks.sh --reference` cannot pass on this host: the
  profile gate refuses non-reference CPUs, core counts, RAM type and governor
  (recorded real refusal in `docs/benchmarks/REFERENCE_RUNBOOK.md` section 1.4).
- Therefore this gate is `PENDING (prepared, NOT executed on reference
  hardware)`. No number in this document is a benchmark result.

### 3.2 Exact prerequisite checklist

The named human reporter fills every box before the real run:

- [ ] Stabilization commit chosen; `git rev-parse HEAD` recorded in every
      artifact; `git status --porcelain` prints nothing. A later
      performance-relevant commit requires a full repeat.
- [ ] Private `CARGO_TARGET_DIR` on the reference host.
- [ ] `arch = x86_64`, CPU model `AMD Ryzen 9 7950X` (exact allowlist),
      physical cores exactly 16 (`Socket(s) * Core(s) per socket`).
- [ ] RAM >= 32212254720 bytes and type `DDR5` (`dmidecode`).
- [ ] Kernel major version >= 6; governor fixed to `performance` as root.
- [ ] Exactly one load generator, `oha` or `wrk`, version recorded.
- [ ] `python3` and executable `/usr/bin/time` present.
- [ ] `rustc --version` equals the pin in `rust-toolchain.toml`.
- [ ] Quiet-host attestation is a real human observation: no other heavy load,
      no thermal throttling, stable power. Conditions go into the report's
      `Deviations and anomalies`.
- [ ] `REPORTER` is a human name, never `unspecified` or an agent/tool name.
- [ ] Dry run exits 0 with all checks `PASS` and writes no artifact.

"Or equivalent" is narrow: a different CPU becomes a reference host only after
its exact `lscpu` model string is added to the allowlist in
`scripts/run-benchmarks.sh` **and** to the table in
`docs/benchmarks/reference-host.md`, with rationale in the same commit. That is
a hardware addition, not a performance-target change; changing a target needs
an RFC (rule 22).

### 3.3 Host profile

| Runner label | Required value | Exact check |
| --- | --- | --- |
| `arch` | `x86_64` | `lscpu \| grep '^Architecture'` |
| `cpu_allowlist` | `AMD Ryzen 9 7950X` | `lscpu \| grep '^Model name'` |
| `physical_cores` | exactly 16 | `S=$(lscpu \| awk -F: '/Socket\(s\)/{print $2}'); C=$(lscpu \| awk -F: '/Core\(s\) per socket/{print $2}'); echo $((S*C))` |
| `ram_capacity` | >= 32212254720 bytes | `free -b \| awk '/^Mem:/{print $2}'` |
| `ram_type` | `DDR5` | `sudo -n dmidecode -t 17 \| grep 'Type:'` |
| `kernel` | major >= 6 | `uname -r` |
| `cpu_governor` | `performance` | `cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` |
| `load_tool` | `oha` or `wrk`, version recorded | `command -v oha \|\| command -v wrk`, then `<tool> --version` |
| `tooling` | `python3` + `/usr/bin/time` | `command -v python3`; `test -x /usr/bin/time` |
| `rustc_pinned` | equals `rust-toolchain.toml` pin | `rustc --version`; `grep channel rust-toolchain.toml` |
| `clean_tree` | 0 dirty files | `git status --porcelain \| wc -l` |
| `quiet_attestation` | `AG_QUIET_HOST_ATTESTED=yes` | human decision |
| `reporter` | human name, not `unspecified` | `REPORTER` set for the real run |

### 3.4 Exact commands

Fix the commit and prepare the host:

```sh
git rev-parse HEAD                 # record this exact SHA in the report
git branch --show-current
git status --porcelain             # must print nothing

# As root on the reference host:
cpupower frequency-set -g performance
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
sudo dmidecode -t 17 | grep -E 'Size|Type:'
oha --version    # or: wrk --version
python3 --version
/usr/bin/time --version | head -1
rustc --version
```

Fail-closed rehearsal (no benchmark runs, no directory/log/CSV written):

```sh
bash -n scripts/run-benchmarks.sh
./scripts/run-benchmarks.sh --help
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  ./scripts/run-benchmarks.sh --dry-run
echo "exit=$?"

git status --porcelain             # unchanged from the commit check
test ! -e docs/benchmarks/raw/$(date -u +%Y-%m-%d)-reference && echo "no artifacts"
```

Exit codes: `0` = every check `PASS` plus printed plan (proceed); `3` = one or
more `FAIL` (fix the host or stop; do not relax the profile); `2` = bad arguments
or not run from the repository root. A bare `--dry-run` without the attestation
honestly reports `FAIL quiet_attestation no` and exits 3.

Real run, from the repository root, on a clean tree at the stabilization commit,
host otherwise idle; do not interrupt it:

```sh
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  CARGO_TARGET_DIR="$PWD/target" \
  ./scripts/run-benchmarks.sh --reference
```

Optional knobs, all recorded in `metadata.txt`:

```sh
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  OHA_DURATION=30s OHA_CONCURRENCY=100 \
  ./scripts/run-benchmarks.sh --reference --runs 5
```

`--reference` refuses `SAMPLE_SIZE`, `WARMUP_TIME` or `MEASUREMENT_TIME` other
than `100`, `3`, `5` (exit 2). Default output:
`docs/benchmarks/raw/<YYYY-MM-DD>-reference` (`OUT_DIR` overrides). If the run
fails after host validation, do not hand-edit partial artifacts and do not call
it a pass; record the failure, keep #157 open and re-run from a clean tree.

Artifact verification before writing the report:

```sh
OUT=docs/benchmarks/raw/<YYYY-MM-DD>-reference
find "$OUT" -maxdepth 1 -type f | sort
grep -E 'unavailable|error:' "$OUT/estimates-summary.csv" && echo "INCOMPLETE"
grep -c 'example listening' "$OUT"/server-*.log
wc -l "$OUT/e2e-runs.csv"                      # header + RUNS data rows
grep -E 'reporter=|commit=|mode=reference' "$OUT/rule17-summary.txt"
```

The criterion `time: [a b c]` triplet is a 95% CI of the mean, **not** a
standard deviation. Rule 17's standard deviation is `std_dev.point_estimate`
from `target/criterion/<bench>/new/estimates.json`, copied into
`estimates-summary.csv` by the runner.

### 3.5 Expected artifacts / evidence

Under `docs/benchmarks/raw/<YYYY-MM-DD>-reference/`:

```text
metadata.txt                 # every rule-17 context field
rule17-summary.txt           # context, configuration, methodology, runs, std-dev source
no-run.log                   # cargo bench --no-run compile check
ag-core-criterion.log        # three ag-core benchmarks, full criterion output
ag-workers-criterion.log     # four ag-workers benchmarks, full criterion output
estimates-summary.csv        # benchmark, mean_ns, std_dev_ns, sample_size, source
build-hello-world.log        # cargo build --release -p ag-core --example hello_world
binary-size.txt              # stat -c %s of the release example
e2e-runs.csv                 # run, cold_start_ms, idle_rss_kb, load_output path
server-N.log                 # one per independent run
<oha|wrk>-run-N.txt          # one per independent run
```

All seven criterion rows need numeric mean and std dev; `e2e-runs.csv` needs one
row per independent run with numeric cold start and idle RSS. Commit the raw
directory together with the curated report. Generated `target/criterion/`
HTML/JSON is not committed. A scenario that was not executed stays `not
measured` with an explicit reason; never fill it from an informative run or a
design-target table.

### 3.6 Who must approve (human)

1. The named reporter, who owns the reference host: attests the quiet host,
   runs the real command, signs the rule-17 report with their human name.
2. A human reviewer: verifies artifacts, standard deviations, `not measured`
   decisions and the gate-row wording (no `pass` if any design target is unmet).
3. The coordinator/orchestrator: reconciles README EN/ES and closes #157 after
   sign-off. Agents do not push, open the PR or merge.

### 3.7 Exact close condition for #157

All five conditions in `docs/benchmarks/reference-host.md`, plus the checklist
in `docs/benchmarks/REFERENCE_RUNBOOK.md` section 7:

1. `./scripts/run-benchmarks.sh --reference` exited 0 on the stabilization
   commit, host validation printed all `PASS`, command and commit recorded.
2. Every artifact in section 3.5 exists in the committed raw directory, with
   numeric std dev for all seven criterion benchmarks and complete e2e rows.
3. `docs/benchmarks/pre-fase5-performance.md` is filled from those artifacts
   with all rule-17 fields, standard deviations and explicit `not measured`
   decisions.
4. The `Benchmarks` row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` says `pass`
   or `fail` with the measured gap and cites report, date, commit and command;
   no other row is touched.
5. README EN and ES blocker bullets are reconciled in the same commit.
6. Content scans clean (`bash -n`, `git diff --check`, AI evidence, descriptor
   prefixes, emojis, retired brands). Only then `Closes #157`, remove
   `blocked`, and remove the descriptor when the PR merges.

Until every box is checked, #157 stays open with `blocked`, gate row `pending`,
and the branch keeps `Refs #157`.

## 4. Prepared but NOT executed - explicit register

| Item | Prepared (exists and checked here) | Missing (gate evidence) | State |
| --- | --- | --- | --- |
| 24h fuzz | runbook, workflow, harness, 60 s smoke JSONL | seven 86400 s logs, `24h/` run dir, `RESULTS.md`, triage | prepared, NOT executed |
| Reference benchmarks | profile, fail-closed runner, dry-run refusal, empty template | `*-reference/` raw dir, curated report, gate row, README reconciliation | prepared, NOT executed |
| This package | exact commands, host profiles, approvers, close conditions | human execution and sign-off | prepared, NOT executed |

Verification performed **while preparing this package** (read-only; no gate was
run, no number invented):

| Check | Command | Real result |
| --- | --- | --- |
| Workflow YAML parses | `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/fuzz-24h.yml"))'` | exit 0; top-level keys present. Note: PyYAML 1.1 maps the key `on` to boolean `True`; this is a parser quirk, not a file defect (GitHub Actions uses YAML 1.2). |
| Shell syntax | `bash -n` on `scripts/run-benchmarks.sh`, `scripts/run-fuzz-window.sh`, `scripts/ag-check`, `scripts/kaizen` | all exit 0 |
| Fuzz evidence inventory | `find docs/fuzz/results ...`, `cat windows.jsonl` | one 60 s smoke only; no 24 h directory or `RESULTS.md` |
| Benchmark evidence inventory | `find docs/benchmarks/raw ...`, test for report | only informative/wave9 dirs; no reference dir or report |
| Diff hygiene | `git diff --check` | exit 0, clean |
| Local gate script | `bash scripts/ag-check` | exit 0; `PASS=8 FAIL=0 SKIP=0` (descriptors, fmt, clippy, test, audit, deny, masters, prohibited) |
| Kaizen gate | `bash scripts/kaizen check` | exit 0; `ok: kaizen check: todo en verde` (debt 0 <= 0) |
| Content scans | AI attribution, session URLs, emojis, retired brands, descriptor prefixes | none found in the new files or tracked descriptors |

Source hashes at the package commit (recomputed locally, unchanged by this
document-only change):

| File | SHA-256 |
| --- | --- |
| `.github/workflows/fuzz-24h.yml` | `925f9f0fb8b3d219161577e98acbffd7877e4780946ba8052f4590492da7ebe7` |
| `scripts/run-benchmarks.sh` | `c4b56a6030fe2282bf5a4458c8a0d00613e17f066d87df60f6963d906c7af2bf` |
| `docs/fuzz/results/windows.jsonl` | `a5d7e59af57a513e9eb0da52fdef969ffcf12329e8818bdf9456cfe4ec089145` |
| `docs/audits/PRE_FASE5_RELEASE_GATE.md` | `c330de046d9fdb53704b7c07e560d38bf9667d92402842d57a575fbe77f43eb2` |

## 5. Decision requested from the coordinator

The shared blocker is one hardware/human window, so decide both gates together:

1. Name a human owner for the fuzz host and authorize 7 x 86400 s (at least
   168 h sequential) plus a clean 500 GiB scratch volume and a dedicated
   service account. Without this, #156 stays `blocked`.
2. Name the human reporter who owns an `AMD Ryzen 9 7950X` / DDR5 / 16-core /
   `performance`-governor host and a quiet time slot for one reference run.
   Without this, #157 stays `blocked`.
3. Confirm the approvers in sections 2.6 and 3.6. Only after their evidence
   sign-off may the gate rows flip and the issues close.
4. Do not treat this package, the green workflow structure, a 60 s smoke or an
   informative host run as gate evidence (K-2, K-10, rule 10).

This package changes no gate row, closes no issue and executes no gate.

## 6. Verification log for this package

Real results from the package worktree (base commit
`97b9402f9e5127924c39a9c75174e6444de7ad78`):

- `git diff --check`: exit 0, clean (files staged with `git add -N` so the
  check covers the new, otherwise untracked files).
- `bash scripts/ag-check`: exit 0, `PASS: descriptors, fmt, clippy, test,
  audit, deny, masters, prohibited`; `Summary: PASS=8 FAIL=0 SKIP=0`;
  `ag-check: all selected checks passed.`
- `bash scripts/kaizen check`: exit 0; `ok: descriptores validados`,
  `ok: politicas de contenido`, `ok: formato`, `ok: lint`, `ok: tests`,
  `ok: deuda 0 <= 0`, `ok: kaizen check: todo en verde`.
- Content scans reproducing `.github/workflows/docs.yml`: no AI attribution,
  no session URLs, no emojis, no retired third-party brands and no prohibited
  descriptor prefixes; no suppression added.
- No 24-hour fuzz run and no reference benchmark run were executed. No gate
  row was flipped and no issue was closed.

## 7. References

- Issues #156, #157; RFC-0036; ADR-0015; ADR-0008; CLAUDE.md rules 17 and 36.
- `docs/fuzz/24h-gate-verification.md`, `docs/fuzz/24h-gate-runbook.md`,
  `docs/fuzz/self-hosted-runner.md`, `docs/fuzz/windows-jsonl-schema.md`,
  `docs/audits/fuzz-24h-workflow-structural-verification.md`.
- `docs/benchmarks/reference-host.md`, `docs/benchmarks/REFERENCE_RUNBOOK.md`,
  `docs/benchmarks/verification-package.md`,
  `docs/benchmarks/wave9-host-results.md` (informative only).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md`,
  `docs/audits/estado-honesto-puertas.md`, `docs/audits/issue-board-reconciliation.md`.
