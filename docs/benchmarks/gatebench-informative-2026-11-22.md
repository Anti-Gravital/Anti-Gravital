# Gatebench informative run - 2026-11-22

Status: **harness verification only. NOT Stage 6 evidence for issue #157.**
Sample size, warm-up and measurement time were deliberately reduced, the host
is not reference hardware, and no load generator is installed. No number below
may be published as an official Anti-Gravital metric, used to update the release
gate, or used to close #157.

## What was verified

- `bash -n scripts/run-benchmarks.sh` exits 0.
- `--help` and argument validation behave as documented.
- `--reference` refuses this host with exit 3 and writes no artifacts.
- A real short non-reference run completes with exit 0 and writes every
  rule-17 context field automatically, including the standard deviation read
  from `estimates.json`.

## Reference-mode refusal (real output, 2026-11-22)

```text
PASS  arch                   x86_64
FAIL  cpu_allowlist          12th Gen Intel(R) Core(TM) i7-1255U
FAIL  physical_cores         10
FAIL  ram_type               DDR4
PASS  kernel                 7.0.0-44-generic
FAIL  cpu_governor           powersave
FAIL  load_tool              none unknown
PASS  tooling                python3 + /usr/bin/time
PASS  rustc_pinned           rustc 1.99.1 (58cf8f9c3 2026-10-13) (pin 1.99.1)
FAIL  clean_tree             1 dirty files
PASS  quiet_attestation      yes
ERROR: 6 reference-profile check(s) failed; refusing to label this run as reference.
No numbers were produced. See docs/benchmarks/reference-host.md.
exit=3
```

The `clean_tree` failure above was from an earlier invocation before the
implementation was committed; the final informative run below started from a
clean tree.

## Real short run

Command (from the repository root, clean tree):

```sh
SAMPLE_SIZE=10 WARMUP_TIME=1 MEASUREMENT_TIME=1 \
  OUT_DIR=docs/benchmarks/raw/2026-11-22-informative \
  REPORTER=gatebench bash scripts/run-benchmarks.sh
```

Exit code: 0. Commit under test:
`8ef5792bc5ac6dfc7da737e7cd3c946f41f933c5`, branch `community11/gatebench`,
dirty files at start: 0. Date/time (UTC): 2026-11-22T15:06:16Z.

Rule-17 context fields (verbatim from
`docs/benchmarks/raw/2026-11-22-informative/rule17-summary.txt`):

- Hardware: x86_64, 12th Gen Intel Core i7-1255U, 1 socket, 10 physical cores,
  12 logical CPUs, max 4.70 GHz, 30 Gi DDR4, 512 GB PCIe SSD.
- OS: Ubuntu 26.04.1 LTS, kernel 7.0.0-44-generic, `ulimit -n` 524288,
  `somaxconn` 4096, governor `powersave` (not a quiet host).
- Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1, pinned by
  `rust-toolchain.toml` channel 1.99.1.
- Build profile: release, codegen-units 1, LTO fat, panic abort, strip symbols,
  opt-level 3.
- Configuration: 10 samples, 1 s warm-up, 1 s measurement, `--noplot`,
  release/bench profile, `--locked`.
- Number of runs: 10 criterion samples per benchmark; 0 end-to-end runs
  (informative mode never runs the load phase).
- Methodology: criterion microbenchmarks at reduced samples for harness sanity
  only. Standard deviation is `std_dev.point_estimate` from
  `target/criterion/<bench>/new/estimates.json`; the printed `time:` triplet is
  a 95% confidence interval of the mean, not a standard deviation.
- Load tool: none (`oha`, `wrk`, `bombardier`, `hey` not installed).

### Real results (informative, not design-target results)

| Benchmark | Printed `time:` (95% CI of mean) | Mean (ns) | Std dev (ns) | Samples |
| --- | --- | --- | --- | --- |
| `bare_axum_hello` | `[4.7582 us 5.0248 us 5.4705 us]` | 5208.69 | 376.25 | 10 |
| `shield_default_hello` | `[5.7183 us 5.8678 us 6.1209 us]` | 6136.58 | 468.47 | 10 |
| `shield_full_default_hello` | `[6.2691 us 6.6002 us 6.8621 us]` | 6372.62 | 513.72 | 10 |
| `payload_encode` | `[119.77 ns 124.72 ns 129.59 ns]` | 123.39 | 12.36 | 10 |
| `payload_decode` | `[83.591 ns 84.614 ns 86.115 ns]` | 85.33 | 2.12 | 10 |
| `enqueue_memory` | `[482.39 ns 488.80 ns 497.64 ns]` | 489.16 | 12.93 | 10 |
| `lease_memory_batch_100` | `[648.13 us 725.96 us 781.83 us]` | 668221.00 | 82914.27 | 10 |

Criterion compared against the previous reduced-sample baseline and printed
change/regression lines. Those comparisons are between different reduced
configurations and are not regression signals; they are excluded here.

### Artifacts committed

- `docs/benchmarks/raw/2026-11-22-informative/metadata.txt`
- `docs/benchmarks/raw/2026-11-22-informative/rule17-summary.txt`
- `docs/benchmarks/raw/2026-11-22-informative/no-run.log`
- `docs/benchmarks/raw/2026-11-22-informative/ag-core-criterion.log`
- `docs/benchmarks/raw/2026-11-22-informative/ag-workers-criterion.log`
- `docs/benchmarks/raw/2026-11-22-informative/estimates-summary.csv`

`target/criterion/` HTML/JSON is generated and not committed.

## Not measured

Throughput, p99, idle RSS, cold start, binary size, CRUD/DB and WebSocket
scenarios are not measured here: no load generator, no reference hardware, no
live database scenario and no quiet host. Their fields stay empty.

## Why #157 stays open

The release-gate Benchmarks row remains `pending`, the final curated report
`docs/benchmarks/pre-fase5-performance.md` has not been written, and README's
performance claim has not been reconciled. The exact conditions and evidence
needed to close the gate are listed in `docs/benchmarks/reference-host.md` and
`docs/benchmarks/wave9-host-results.md`. This branch keeps `Refs #157`.
