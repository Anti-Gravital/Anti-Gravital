# Reference host and one-command benchmark procedure (issue #157)

Status: **procedure only**. No official result has been produced. This document
defines the host on which Stage 6 numbers may be published, the exact commands
that produce them, and the empty template to fill afterwards. It does not report
a pass and it does not change the release gate.

Benchmarks must follow CLAUDE.md rule 17 (hardware, OS, Rust version, commit,
configuration, methodology, number of runs, standard deviation) and
`docs/benchmarks/measurement-template.md`. The curated report belongs in
`docs/benchmarks/pre-fase5-performance.md`; the surrounding procedure is
`docs/benchmarks/verification-package.md`. The one-session maintainer handoff,
including the gate-row and README edits and the close-out evidence checklist,
is `docs/benchmarks/REFERENCE_RUNBOOK.md`.

## Why this document exists

`docs/benchmarks/wave9-host-results.md` and `docs/benchmarks/smoke-2026-11-22.md`
are honest datasets from an informative mobile notebook. They prove the suites
run, but a mobile CPU with a `powersave` governor cannot settle the design-target
table. Without a machine-checkable profile, an informative run can be mislabelled
as reference evidence by accident. `scripts/run-benchmarks.sh --reference`
implements the profile below as a fail-closed gate: it validates the host first
and refuses to write any artifact when a check fails.

## Target profile

Every item is checked by the runner in `--reference` mode before any benchmark
starts. A failed check prints `FAIL` and exits 3 with no numbers and no output
directory.

| Item | Required value | How the runner verifies it |
| --- | --- | --- |
| Architecture | `x86_64` | `lscpu` `Architecture` |
| CPU model | `AMD Ryzen 9 7950X` (allowlist) | substring match on `lscpu` `Model name` |
| Physical cores | exactly 16 | `Socket(s)` times `Core(s) per socket` from `lscpu` |
| Memory capacity | at least 30 GiB (32 GB nominal) | `free -b` total memory |
| Memory type | `DDR5` | `dmidecode -t 17` (`Type:`), via `sudo -n` when needed |
| Kernel | major version 6 or newer | `uname -r` |
| CPU governor | `performance` (fixed) | `/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` |
| Load generator | `oha` or `wrk`, version recorded | `command -v` and `<tool> --version` |
| Extra tooling | `python3` and `/usr/bin/time` present | `command -v` and `-x` |
| Toolchain | `rustc` version equals the pin in `rust-toolchain.toml` | version comparison |
| Repository | clean tree at the stabilization commit | `git status --porcelain` count is 0 |
| Quiet host | attested by the reporter | `AG_QUIET_HOST_ATTESTED=yes` |
| Reporter | a human name, never an agent or tool name | `REPORTER` is set and not `unspecified` |

Equivalence rule: "or equivalent" in the benchmark READMEs is deliberately
narrow. A different CPU is a reference host only after its exact `lscpu` model
string is added to the allowlist in `scripts/run-benchmarks.sh` **and** to the
table above, with the rationale recorded in the same commit. Adding hardware is
not a performance-target change; changing a target itself requires an RFC
(CLAUDE.md rule 22).

The quiet-host attestation covers conditions a script cannot measure: no other
heavy load, no thermal throttling, stable power. The reporter must write the
observed conditions in the `Deviations and anomalies` section of the final
report. Attesting to a loaded or throttling host is a false claim even if every
automated check passes.

### Host preparation

```sh
# As root on the reference host: fix the governor and confirm no throttling.
cpupower frequency-set -g performance
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

# Confirm memory type and capacity (dmidecode output is recorded by the runner).
sudo dmidecode -t 17 | grep -E 'Size|Type:'

# Install exactly one load generator and record its version.
oha --version    # or: wrk --version

# Cross-check tooling used by the procedure.
python3 --version
/usr/bin/time --version | head -1
```

The automated cold-start figure is wall time from process fork to the first
`ag-core hello_world example listening` tracing event, measured with epoch
timestamps. `/usr/bin/time -v` is required so the reporter can cross-check the
same figure manually and attach a resource log; record that separate launch in
the report as a cross-check, not as an extra independent run.

## Exact commands

Rehearse first with the fail-closed preflight. It runs the same validation as
`--reference` and prints the commands and artifacts of the real run, but writes
nothing and executes no benchmark. Exit 0 means the host is ready; exit 3 means
the profile was not met and no artifact was produced:

```sh
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  ./scripts/run-benchmarks.sh --dry-run
```

A bare `--dry-run` without the attestation fails closed as well: the missing
attestation is reported as `FAIL quiet_attestation no`. Supply the human name
that will sign the final report so the rehearsal matches the real invocation.

Then run the real command below from the repository root, on a clean tree at the
final stabilization commit.
The default output directory is
`docs/benchmarks/raw/<YYYY-MM-DD>-reference`; set `OUT_DIR` to override it. Use
a private `CARGO_TARGET_DIR` on the reference host so the run is not affected by
other builds.

```sh
# One command: validates the host, writes rule-17 metadata, runs both criterion
# suites with official defaults, extracts std dev, then runs the load phase.
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  CARGO_TARGET_DIR="$PWD/target" \
  ./scripts/run-benchmarks.sh --reference
```

Optional knobs, all recorded in the metadata:

```sh
# More independent end-to-end runs (default 3), different load shape.
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  OHA_DURATION=30s OHA_CONCURRENCY=100 \
  ./scripts/run-benchmarks.sh --reference --runs 5
```

`--reference` refuses shortened criterion runs. Setting `SAMPLE_SIZE`,
`WARMUP_TIME` or `MEASUREMENT_TIME` to anything other than `100`, `3` or `5`
exits 2. No flag can make an informative host pass the reference gate.

A refusal on the development host looks like this (real output, i7-1255U
notebook, 2026-11-22):

```text
FAIL  cpu_allowlist          12th Gen Intel(R) Core(TM) i7-1255U
FAIL  physical_cores         10
FAIL  ram_type               DDR4
FAIL  cpu_governor           powersave
FAIL  load_tool              none unknown
ERROR: 6 reference-profile check(s) failed; refusing to label this run as reference.
No numbers were produced. See docs/benchmarks/reference-host.md.
```

The informative mode is still available for harness sanity checks and is always
labelled as such:

```sh
SAMPLE_SIZE=10 WARMUP_TIME=1 MEASUREMENT_TIME=1 \
  ./scripts/run-benchmarks.sh
```

It writes to `docs/benchmarks/raw/<YYYY-MM-DD>-informative` by default and never
runs the load phase. Its numbers must not be published as design-target results.

### After the runner

The runner collects metadata and raw outputs; it does not decide whether a
target was met. Curating the report is a human step:

```sh
# 1. Inspect the rule-17 fields and the std dev source.
cat docs/benchmarks/raw/<YYYY-MM-DD>-reference/rule17-summary.txt
cat docs/benchmarks/raw/<YYYY-MM-DD>-reference/estimates-summary.csv

# 2. Inspect each independent load run and the server logs.
for f in docs/benchmarks/raw/<YYYY-MM-DD>-reference/*-run-*.txt; do
  echo "== $f"; cat "$f"
done

# 3. Fill docs/benchmarks/pre-fase5-performance.md from those files using the
#    template below; compute median and sample standard deviation from the
#    independent runs. Mark any unreproducible scenario as "not measured".
```

The `time: [a b c]` triplet printed by criterion is the 95% confidence interval
of the mean, **not** a standard deviation. Rule 17's standard deviation is
`std_dev.point_estimate` from `target/criterion/<bench>/new/estimates.json`,
copied into `estimates-summary.csv` by the runner.

## Artifact layout

```text
docs/benchmarks/raw/<YYYY-MM-DD>-reference/
├── metadata.txt              # every rule-17 context field
├── rule17-summary.txt        # consolidated context, configuration, methodology, runs, std-dev source
├── no-run.log                # cargo bench --no-run compile check
├── ag-core-criterion.log     # three ag-core benchmarks, full criterion output
├── ag-workers-criterion.log  # four ag-workers benchmarks, full criterion output
├── estimates-summary.csv     # benchmark, mean_ns, std_dev_ns, sample_size, source
├── build-hello-world.log     # cargo build --release -p ag-core --example hello_world
├── binary-size.txt           # stat -c %s of the release example
├── e2e-runs.csv              # run, cold_start_ms, idle_rss_kb, load_output path
├── server-N.log              # one per independent run (listening event, tracing)
└── oha-run-N.txt             # one per independent run (or wrk-run-N.txt for wrk)
```

`target/criterion/` HTML and JSON are generated and are not committed. The raw
directory and the curated report are committed. The root `.gitignore` excludes
`*.log` globally but re-includes `docs/benchmarks/raw/**/*.log` so the criterion
and server logs above are not silently dropped.

## Evidence template (empty copy for `pre-fase5-performance.md`)

```markdown
# Pre-Phase 5 performance evidence - YYYY-MM-DD

- Reporter: <human name>
- Date/time (UTC): <date -u +%Y-%m-%dT%H:%M:%SZ>
- Reference profile: <all checks PASS; paste the validation block from the runner log>
- Commit under test: <git rev-parse HEAD>
- Branch / dirty tree at start: <branch> / <no>
- Hardware: <CPU model, base and boost frequency, physical/logical cores, RAM type and size, storage>
- OS: <os-release, kernel, ulimit -n, somaxconn>
- CPU governor: <performance>
- Rust: <rustc --version> with <rust-toolchain.toml>
- Build profile: release; codegen-units and LTO from the root Cargo.toml
- Load tool: <tool and version>
- Commands: <exact command from docs/benchmarks/reference-host.md>
- Configuration: <sample size, warm-up, measurement time, oha duration and concurrency>
- Methodology: <criterion defaults; independent runs with server restart; idle RSS at 5 s; cold start fork to listening; statistic>
- Number of runs: <criterion samples per benchmark; independent e2e runs>
- Standard deviation source: <std_dev.point_estimate from estimates.json for criterion; sample std dev across e2e runs>

## Criterion results

| Suite | Benchmark | Mean (ns) | Std dev (ns) | Samples | 95% CI | Outliers |
| --- | --- | --- | --- | --- | --- | --- |
| ag-core | bare_axum_hello |  |  |  |  |  |
| ag-core | shield_default_hello |  |  |  |  |  |
| ag-core | shield_full_default_hello |  |  |  |  |  |
| ag-workers | payload_encode |  |  |  |  |  |
| ag-workers | payload_decode |  |  |  |  |  |
| ag-workers | enqueue_memory |  |  |  |  |  |
| ag-workers | lease_memory_batch_100 |  |  |  |  |  |

## Rule-17 end-to-end results

| Metric | Run 1 | Run 2 | Run 3 | Median | Std dev | Met? |
| --- | --- | --- | --- | --- | --- | --- |
| Hello World throughput (req/s) |  |  |  |  |  |  |
| p99 at 100 K req/s (ms) |  |  |  |  |  |  |
| Idle RSS (MB) |  |  |  |  |  |  |
| Cold start (ms) |  |  |  |  |  |  |
| Release binary size (MB) |  |  |  |  |  |  |
| CRUD throughput, PostgreSQL (req/s) |  |  |  |  |  |  |
| p99 with DB query (ms) |  |  |  |  |  |  |
| Concurrent WebSocket connections (2 vCPU) |  |  |  |  |  |  |

## Deviations and anomalies

<thermal throttling, other load, discarded outliers with reason, scenarios marked
not measured and why, differences from the reference profile>

## Reproducibility

<exact command, commit and artifacts; a third party with the same profile must
reproduce the numbers within the stated standard deviation (rule 36)>
```

## Conditions to close issue #157

1. The runner exits 0 in `--reference` mode on the stabilization commit and all
   artifacts listed above exist in the committed raw directory.
2. `docs/benchmarks/pre-fase5-performance.md` is filled from those artifacts
   with every rule-17 field, including standard deviations.
3. The `Benchmarks` row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` cites the
   report, date, commit and command, and is `pass` or `fail` with the measured
   gap.
4. README's performance claim is reconciled in both the English and Spanish
   sections.
5. Only then is #157 closed with `Closes #157` and the `blocked` label removed.

Until all five hold, issue #157 stays open with `blocked`, and this branch keeps
`Refs #157`.
