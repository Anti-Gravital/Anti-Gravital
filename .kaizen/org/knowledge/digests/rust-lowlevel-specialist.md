# Digest de conocimiento: rust-lowlevel-specialist

- **Load and saturation protocol** — `docs/rfc/RFC-0039-load-saturation-and-install-measurement.md` (sha256 b411e8db5c43)
- **Stage 6 verification package** — `docs/benchmarks/verification-package.md` (sha256 c2851c8a85c3)

## RFC-0039-load-saturation-and-install-measurement.md
# RFC-0039 - Load, saturation and installation measurement protocol

- Status: draft (pending BDFL approval)
- Draft date: 2026-11-22
- Target phase: additive, no phase change
- Affected modules/crates: none (measurement procedure and tooling only)
- Related: issue #157 (blocked), `docs/benchmarks/verification-package.md`
- Comment period: open

## 1. Motivation

The project must be able to state, honestly and reproducibly, how it behaves under load: how long it takes to install, how many requests per second and minute it serves, what the latency distribution is, where it saturates and what fails first. Claims must be evidence, not extrapolation (rule 17, rule 10).

## 2. Problem (verified on 2026-11-22)

1. Only two Criterion microbenchmark suites exist: `crates/ag-core/benches/shield_hello_world.rs` (Tower-level latency) and `crates/ag-workers/benches/queue_throughput.rs` (in-memory hot paths). Reproduced by `scripts/run-benchmarks.sh:50-80`.
2. No sustained network throughput, p50/p95/p99, error rate, RSS, cold start, binary size, install time or saturation figure is currently measurable.
3. No load generator is installed (`oha`, `wrk`, `bombardier`, `hey` all absent). `docs/benchmarks/smoke-2026-11-22.md:19-21` and `wave9-host-results.md:38-39` state this explicitly.
4. The host is informative mobile-class, not the reference class required by `verification-package.md:19-23` (Ryzen 9 7950X or equivalent, 16 physical cores, 32 GB DDR5, NVMe).
5. Stage 6 evidence is required at the final consolidation commit; `docs/audits/PRE_FASE5_RELEASE_GATE.md` keeps the `Benchmarks` row pending and issue #157 blocked (`verification-package.md:3-15,196-206`).
6. Historical numbers exist but on different stacks or commits (for example CRUD 14,478 req/s on a Ryzen 5 2500U in `README.md:496`, Neon saturation peak 67 req/s in `measurement-2026-05-22-neon-saturacion.md:21-46`, 50k subscribers in a debug profile in `ag-realtime-load.md:19-20`). None settles Stage 6 and none may be reused as current evidence.

## 3. Pending human decisions

- **Load target definition.** "200 / thousands of requests" mixes three quantities: total request count, concurrent connections (VUs) and target RPS. The user selected a concurrency ladder of 200 -> 1k -> 5k, with RPS as a measured result, not a target. This RFC adopts that decision.
- **Tool installation.** `oha` (Rust, open source, already the canonical tool in `verification-package.md:33`) must be installed with human approval. `wrk` is the fallback.
- **Reference hardware.** Official figures require a Ryzen 9 7950X-class host, fixed CPU governor and quiet environment. Until then all results are labelled "hardware informativo" and are not publishable.

## 4. Alternatives considered

- **A. Extrapolate microbenchmark latency into req/s claims.** Rejected: rule 10 forbids it; it would be an anti-hype violation.
- **B. Publish the reduced-sample smoke numbers.** Rejected: the smoke document explicitly forbids it (`smoke-2026-11-22.md:3-6`).
- **C. Run a documented concurrency ladder with full metadata, repeated runs and honest hardware labelling (chosen).**

## 5. Proposed protocol

### 5.1 Load curve

Warm-up at fixed duration, then stepped concurrency: 200, 1k, 5k and any final user-chosen value. Each step is held long enough to reach steady state. At least three independent runs with the server restarted between them; report median and sample standard deviation (`verification-package.md:115-121`).


## verification-package.md
# Stage 6 benchmark verification package (issue #157)

Status: **BLOCKED**. Final performance evidence on stabilized code has not been
produced. This document is the reproducible procedure and the empty results
template. It does not report a pass, and it does not replace the final report
at `docs/benchmarks/pre-fase5-performance.md`.

The single-command runner for the reference procedure is
`./scripts/run-benchmarks.sh --reference`. Its fail-closed host profile,
required tooling, artifact layout and empty evidence template are defined in
`docs/benchmarks/reference-host.md`. The command sequence below remains the
authoritative description of what is measured; the runner automates the metadata
capture, the criterion suites, the standard-deviation extraction and the
end-to-end load phase.

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

