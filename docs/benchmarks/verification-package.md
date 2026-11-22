# Stage 6 benchmark verification package (issue #157)

Status: **BLOCKED**. Final performance evidence on stabilized code has not been
produced. This document is the reproducible procedure and the empty results
template. It does not report a pass, and it does not replace the final report
at `docs/benchmarks/pre-fase5-performance.md`.

## Why the gate is still pending

`docs/audits/PRE_FASE5_RELEASE_GATE.md` keeps the `Benchmarks` row `pending`
(Stage 6), and README lists final performance evidence as pending. Stage 6 is
scheduled last among the technical stages precisely because earlier fixes
invalidate earlier numbers (execution order
`0→1→2→3→4→5→7→8→9→6→10→11`). A benchmark run on any other commit than the
final consolidation commit is not valid evidence for this gate.

## Required environment

- Hardware (reference class, per `crates/ag-core/benches/README.md`): AMD
  Ryzen 9 7950X or equivalent x86-64 with 16 physical cores, 32 GB DDR5, NVMe
  storage, Linux kernel 6.0 or newer. Results on other hardware are published
  as "hardware informativo" and labelled as such; they cannot settle the
  design-target table below.
- OS: record `uname -a`, `/etc/os-release`, `ulimit -n` and
  `sysctl net.core.somaxconn`.
- Toolchain: the pinned stable in `rust-toolchain.toml` (1.99.1 at the time of
  writing). Record `rustc --version`, `cargo --version` and the toolchain file.
- Build: release profile of the workspace, LTO and codegen settings as declared
  in the root `Cargo.toml`. Record them; do not use dev/test profiles.
- Repository: clean tree at the consolidation commit. Record
  `git rev-parse HEAD`, `git branch --show-current` and
  `git status --porcelain`.
- Load generator: `oha` (or `wrk`); record `<tool> --version`. No load generator
  was installed in the QA environment, so the network figures could not be
  collected locally. This is part of the blocker, not an omitted step.
- Quiet host: no other heavy load, fixed CPU governor, no thermal throttling.
  Record governor and note anomalies.

## Exact commands

All commands run from the repository root. Replace `YYYY-MM-DD` with the UTC
date of the run.

### 1. Metadata

```sh
RUN=docs/benchmarks/raw/YYYY-MM-DD
mkdir -p "$RUN"
{
  echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "commit=$(git rev-parse HEAD)"
  echo "branch=$(git branch --show-current)"
  echo "dirty_files=$(git status --porcelain | wc -l)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "toolchain=$(cat rust-toolchain.toml)"
  echo "uname=$(uname -a)"
  echo "os=$(. /etc/os-release && echo "$PRETTY_NAME")"
  echo "cpu=$(lscpu | grep '^Model name:' | head -1)"
  echo "physical_cores=$(lscpu | grep '^Core(s) per socket:' | head -1)"
  echo "logical_cpus=$(nproc)"
  echo "ram=$(free -h | awk '/^Mem:/{print $2}')"
  echo "storage=$(lsblk -d -o MODEL,ROTA 2>/dev/null | head -5)"
  echo "ulimit_n=$(ulimit -n)"
  echo "somaxconn=$(sysctl -n net.core.somaxconn)"
  echo "governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
  echo "load_tool=$(oha --version 2>&1 || wrk --version 2>&1)"
} | tee "$RUN/metadata.txt"
```

### 2. Criterion suites (microbenchmarks)

Compile check and full runs. Defaults are 100 samples, 3 s warm-up, 5 s
measurement per benchmark; do not shorten them for official numbers.

```sh
cargo bench -p ag-core -p ag-workers --no-run
cargo bench -p ag-core --bench shield_hello_world 2>&1 | tee docs/benchmarks/raw/YYYY-MM-DD/ag-core-criterion.log
cargo bench -p ag-workers --bench queue_throughput 2>&1 | tee docs/benchmarks/raw/YYYY-MM-DD/ag-workers-criterion.log
```

Criterion writes HTML and JSON under `target/criterion/`. Commit curated
results, not that directory. The suites and what they measure are documented in
`crates/ag-core/benches/README.md` and `crates/ag-workers/benches/README.md`.

The `time: [a b c]` triplet printed by criterion is the 95% confidence
interval of the mean, not the standard deviation required by CLAUDE.md rule 17.
Read the standard deviation from the generated JSON for each benchmark:

```sh
for b in bare_axum_hello shield_default_hello shield_full_default_hello \
         payload_encode payload_decode enqueue_memory lease_memory_batch_100; do
  printf '%s ' "$b"
  python3 -c "import json,sys;d=json.load(open(sys.argv[1]));print(d['std_dev']['point_estimate'])" \
    "target/criterion/$b/new/estimates.json"
done
```

Values are in nanoseconds. Record the unit conversion in the report.

### 3. Rule-17 end-to-end figures

These are the figures the published Phase 1/Phase 2 targets refer to. There is
no `ag bench` subcommand in this tree; the reproducible commands are below.

```sh
cargo build --release -p ag-core --example hello_world
./target/release/examples/hello_world > docs/benchmarks/raw/YYYY-MM-DD/server.log 2>&1 &
SERVER_PID=$!
# wait for the "ag-core hello_world example listening" tracing event
oha -z 30s -c 100 http://127.0.0.1:8080/ | tee docs/benchmarks/raw/YYYY-MM-DD/oha-run-1.txt
# restart the server between independent runs
```

Repeat at least three independent runs with the server restarted each time;
report median and standard deviation. Idle RSS and cold start use
`ps -o rss= -p $SERVER_PID` (5 s after start, no traffic) and
`/usr/bin/time -v` from fork to the first `listening` event, as specified in
`docs/benchmarks/measurement-template.md`. Server bind comes from
`ShieldConfig::default()` (`0.0.0.0:8080`); note any TOML override in the
report.

Binary size: `stat -c %s target/release/examples/hello_world` (or the
workspace release binary being claimed), reported in MB. WebSocket connection
and CRUD/DB figures require the corresponding live scenario in
`docs/benchmarks/`; if a scenario cannot be reproduced in the target
environment, mark it `not measured` rather than reusing an older number.

## Artifacts

| Artifact | Path | Committed? |
| --- | --- | --- |
| Environment metadata | `docs/benchmarks/raw/YYYY-MM-DD/metadata.txt` | yes |
| Criterion full output | `docs/benchmarks/raw/YYYY-MM-DD/<suite>-criterion.log` | yes |
| Load-test output | `docs/benchmarks/raw/YYYY-MM-DD/oha-run-N.txt`, `server.log` | yes |
| Criterion HTML/JSON | `target/criterion/` | no (generated) |
| Final curated report | `docs/benchmarks/pre-fase5-performance.md` | yes |
| Updated gate row | `docs/audits/PRE_FASE5_RELEASE_GATE.md` | yes |

## Results template (copy to `docs/benchmarks/pre-fase5-performance.md`)

Every field of CLAUDE.md rule 17 must be filled. Paste real numbers from the
logs; do not reuse a pre-Stage-6 figure.

```markdown
# Pre-Phase 5 performance evidence — YYYY-MM-DD

- Reporter: <human name>
- Commit under test: <git rev-parse HEAD>
- Branch / dirty tree: <branch> / <no>
- Hardware: <CPU model, base and boost frequency, physical/logical cores, RAM type and size, storage>
- OS: <os-release, kernel, ulimit -n, somaxconn, governor>
- Rust: <rustc --version> with <rust-toolchain.toml>
- Build profile: release; LTO and codegen units: <values from Cargo.toml>
- Load tool: <tool and version>
- Commands: exact commands from docs/benchmarks/verification-package.md
- Methodology: <criterion defaults or explicit flags; oha duration/concurrency; restarts between runs>
- Runs: <count>; statistic: median and sample standard deviation

## Criterion results

| Suite | Benchmark | Mean | Std dev | Samples | CI (low..high) | Outliers |
| --- | --- | --- | --- | --- | --- | --- |
| ag-core | bare_axum_hello |  |  |  |  |  |
| ag-core | shield_default_hello |  |  |  |  |  |
| ag-core | shield_full_default_hello |  |  |  |  |  |
| ag-workers | payload_encode |  |  |  |  |  |
| ag-workers | payload_decode |  |  |  |  |  |
| ag-workers | enqueue_memory |  |  |  |  |  |
| ag-workers | lease_memory_batch_100 |  |  |  |  |  |

## Rule-17 design targets

| Metric | Target | Run 1 | Run 2 | Run 3 | Median | Std dev | Met? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Hello World throughput | >= 300 K req/s |  |  |  |  |  |  |
| p99 at 100 K req/s | <= 1 ms |  |  |  |  |  |  |
| Idle RSS | <= 15 MB |  |  |  |  |  |  |
| Cold start | <= 100 ms |  |  |  |  |  |  |
| Release binary size | <= 20 MB |  |  |  |  |  |  |
| CRUD throughput (PostgreSQL) | >= 40 K req/s |  |  |  |  |  |  |
| p99 with DB query | <= 5 ms |  |  |  |  |  |  |
| Concurrent WebSocket connections (2 vCPU) | >= 50 000 |  |  |  |  |  |  |

## Deviations and anomalies

<thermal throttling, CPU governor changes, discarded outliers with reason,
scenarios not measured and why>

## Reproducibility

Third party with this commit and hardware reproduces the numbers within the
stated standard deviation (CLAUDE.md rule 36).
```

## Acceptance criteria for issue #157

- [ ] `docs/benchmarks/pre-fase5-performance.md` exists with every rule-17
      field filled at the stabilization commit.
- [ ] The release gate `Benchmarks` row cites that file with a date and commit
      and is moved to `pass`, or to `fail` with the measured gap stated.
- [ ] README's performance claim is reconciled with the measured evidence
      (both the English and Spanish sections).
- [ ] Commands above reproduce the numbers within the stated deviation.

Until every box is checked, issue #157 stays open and labelled `blocked`.

## Smoke evidence collected in this worktree

Compilation of both suites plus a reduced-sample run (10 samples, 1 s warm-up,
1 s measurement) was executed locally on informative hardware on 2026-11-22.
It verifies that the suites run; it is **not** Stage 6 evidence and none of its
numbers may be published as a design-target result. See
`docs/benchmarks/smoke-2026-11-22.md`.
