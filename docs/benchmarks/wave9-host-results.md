# Wave 9 host benchmark results - 2026-11-22

Status: **informative hardware only. This is NOT Stage 6 evidence for issue
#157 and does not move the release gate `Benchmarks` row to `pass`.**

This document records a real criterion run performed on the wave-9 host. It
exists so the gate has an honest, reproducible dataset to point at while the
reference-hardware procedure in `docs/benchmarks/verification-package.md`
remains unexecuted. No number below is published as an official Anti-Gravital
design-target result.

## Rule 17 metadata

- Reporter: qabench (wave 9 benchmark owner).
- Date/time (UTC): 2026-11-22T13:02:07Z (metadata capture after both suites).
- Commit under test: `7cbc28a35a5b0ceac033fbab4aefb5736d00fd99`.
- Branch / dirty tree: `community9/qabench` / clean (0 modified files at
  measurement time). The documentation work that produced this report was added
  afterwards; no source crate was changed.
- Hardware: 12th Gen Intel Core i7-1255U, 1 socket, 10 physical cores, 12
  logical CPUs, max 4.70 GHz, 30 Gi RAM, NVMe storage. This is a mobile
  notebook class machine, **not** the reference class (AMD Ryzen 9 7950X or
  equivalent, 16 physical cores, 32 GB DDR5, NVMe, kernel >= 6.0).
- OS: Ubuntu 26.04.1 LTS, kernel `7.0.0-44-generic`.
  `uname -a`: `Linux angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC
  7.0.0-44-generic #44-Ubuntu SMP PREEMPT_DYNAMIC Fri Nov  6 02:38:04 UTC 2026
  x86_64 GNU/Linux`.
- Limits: `ulimit -n` = 524288, `net.core.somaxconn` = 4096.
- CPU governor: `powersave` (not a fixed-performance/quiet host; see
  Deviations).
- Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478
  2026-08-27)`, pinned by `rust-toolchain.toml` channel `1.99.1`.
- Build profile: release/bench, `codegen-units = 1`, `lto = "fat"` from the
  root `Cargo.toml` `[profile.release]`.
- Criterion: 0.8.2. Flags: `--sample-size 30 --warm-up-time 2
  --measurement-time 3 --noplot`. The official defaults (100 samples, 3 s
  warm-up, 5 s measurement) were **not** used.
- Load tool: none. `oha`, `wrk`, `bombardier` and `hey` are not installed in
  this environment.
- Cargo env: `CARGO_TARGET_DIR` set to the shared scratch directory,
  `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`.

Source of the environment fields: `docs/benchmarks/raw/wave9-2026-11-22/metadata.txt`.

## Commands

Run from the repository root (reproducible via `scripts/run-benchmarks.sh`,
which accepts `SAMPLE_SIZE`, `WARMUP_TIME`, `MEASUREMENT_TIME` and `OUT_DIR`):

```sh
cargo bench -p ag-core -p ag-workers --no-run
cargo bench -p ag-core --bench shield_hello_world -- \
  --sample-size 30 --warm-up-time 2 --measurement-time 3 --noplot
cargo bench -p ag-workers --bench queue_throughput -- \
  --sample-size 30 --warm-up-time 2 --measurement-time 3 --noplot
```

Both commands exited 0. Full logs were kept outside the repository as
`w9-qabench-ag-core.log` and `w9-qabench-ag-workers.log`; the curated numbers
are below.

## Criterion results

Criterion prints `time: [lower, estimate, upper]` as the 95% confidence
interval of the mean, **not** a standard deviation (rule 17). The `Std dev`
column is `std_dev.point_estimate` read from
`target/criterion/<bench>/new/estimates.json`; values are nanoseconds. The
`Mean` column is `mean.point_estimate` from the same file and can differ
slightly from the printed middle of the CI (criterion uses a bootstrap).

| Suite | Benchmark | Printed `time:` (95% CI of mean) | Mean (ns) | Std dev (ns) | Samples | Outliers |
| --- | --- | --- | --- | --- | --- | --- |
| ag-core | `bare_axum_hello` | `[5.8451 us 5.9672 us 6.0726 us]` | 5931.28 | 229.67 | 30 | 0 reported |
| ag-core | `shield_default_hello` | `[8.0734 us 9.5773 us 11.426 us]` | 8180.93 | 4101.65 | 30 | 3 (1 high mild, 2 high severe) |
| ag-core | `shield_full_default_hello` | `[9.0063 us 9.6583 us 10.657 us]` | 10685.96 | 3778.26 | 30 | 5 (3 high mild, 2 high severe) |
| ag-workers | `payload_encode` | `[270.49 ns 315.21 ns 357.93 ns]` | 283.77 | 107.74 | 30 | 0 reported |
| ag-workers | `payload_decode` | `[93.952 ns 96.985 ns 101.05 ns]` | 120.65 | 39.58 | 30 | 0 reported |
| ag-workers | `enqueue_memory` | `[462.50 ns 469.03 ns 476.62 ns]` | 477.51 | 17.06 | 30 | 0 reported |
| ag-workers | `lease_memory_batch_100` | `[566.83 us 571.90 us 575.54 us]` | 554829.66 | 23826.59 | 30 | 0 reported |

`lease_memory_batch_100` collected its 30 samples in 3.7332 s against a 3 s
target and produced no "unable to complete samples" warning at this
configuration. The standard deviations for `shield_default_hello`,
`shield_full_default_hello` and `lease_memory_batch_100` are large relative to
their means, consistent with the high-severity outliers in the first two and
with the shared, non-quiet host.

### On criterion's "Performance has improved" lines

Criterion compared these runs against the earlier reduced-sample smoke baseline
(`docs/benchmarks/smoke-2026-11-22.md`, 10 samples / 1 s / 1 s). The two
configurations are not comparable, so the printed `change: [-80% .. -60%]` and
`Performance has improved` lines are a measurement artifact of the different
sample count and timing, **not** a real code improvement and not a valid
regression signal. They are deliberately excluded from the results above.

## Not measured (skipped, with reason)

The following rule-17 design targets are **not measured** on this host. They are
left blank rather than extrapolated from any older figure:

- Hello World throughput, p99 at 100 K req/s, idle RSS, cold start and release
  binary size: require the `hello_world` release example plus a load generator
  (`oha`/`wrk`); neither a load generator nor a quiet host is available here.
  Criterion microbenchmarks measure Tower-level latency, not network throughput.
- CRUD throughput (PostgreSQL), p99 with DB query: require a live database
  scenario; none is configured in this environment (see the `ag-workers`
  bench README and issue #108).
- Concurrent WebSocket connections (2 vCPU): requires the dedicated load
  scenario; not reproduced here.
- Reference-hardware figures: this host is mobile-class with a `powersave`
  governor, so none of these results may settle the design-target table.

## Deviations and anomalies

- Shared `CARGO_TARGET_DIR` caused file-lock contention; the two suites spent
  about 5 m 34 s and 1 m 56 s in an LTO-fat rebuild before measurement. Build
  time is not part of any reported metric.
- CPU governor `powersave` and other concurrent workspace activity mean this is
  not a quiet host. Elevated variance (see the std dev column) is expected.
- Reduced sample count (30 vs the official 100) and shorter warm-up/measurement
  times: this dataset verifies that the suites execute and gives real,
  self-consistent numbers, but it is not the official configuration.

## Why issue #157 is still open, and the exact condition to close it

This run produces an informative dataset only. The gate row stays `pending`
and issue #157 stays open with `Refs #157`. To close #157 and move the
`Benchmarks` row to `pass` (or `fail` with the measured gap), all of the
following must hold:

1. Run the full procedure in `docs/benchmarks/verification-package.md` at the
   final stabilization commit, on a clean tree, on reference-class hardware
   (Ryzen 9 7950X or equivalent x86-64, 16 physical cores, 32 GB DDR5, NVMe,
   kernel >= 6.0, fixed CPU governor, quiet host).
2. Use the criterion defaults (100 samples, 3 s warm-up, 5 s measurement) and
   record every rule-17 field, taking standard deviation from `estimates.json`.
3. Install a load generator and collect at least three independent `oha`/`wrk`
   runs with the server restarted each time for throughput and p99, plus idle
   RSS, cold start and binary size. Mark any scenario that cannot be reproduced
   as `not measured`.
4. Write the final report to `docs/benchmarks/pre-fase5-performance.md`, update
   the `Benchmarks` row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` with date,
   commit and a reproducible command, and reconcile the README performance
   claim in both the English and Spanish sections.
5. Close #157 with `Closes #157` only after the gate row is genuinely `pass`.

## Reproducibility

A third party with commit `7cbc28a` and this same host can reproduce the
numbers within the stated standard deviations by running
`SAMPLE_SIZE=30 WARMUP_TIME=2 MEASUREMENT_TIME=3 ./scripts/run-benchmarks.sh`.
The numbers are expected to differ on the reference hardware, which is why
they are labelled informative.
