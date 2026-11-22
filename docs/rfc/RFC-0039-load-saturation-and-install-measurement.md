# RFC-0039 - Load, saturation and installation measurement protocol

- Status: accepted
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

### 5.2 Metrics per load point

Achieved RPS and requests/minute; p50/p95/p99/p99.9; error rate; idle and loaded RSS; CPU percent; cold start (fork to first `listening` event); install time in a fresh environment with a defined command; release binary size.

### 5.3 Metadata

Every rule-17 field: hardware, OS, Rust version, commit, configuration, methodology, number of runs and standard deviation. Record `uname -a`, `/etc/os-release`, `ulimit -n`, `sysctl net.core.somaxconn`, CPU governor and tool version (`verification-package.md:44-68,140-158`). Read standard deviation from Criterion `estimates.json`, never from the printed 95% confidence interval (`:86-99`).

### 5.4 Hardware versus system separation

1. The existing bare Axum versus full Shield Tower benchmark isolates middleware cost (`shield_hello_world.rs:30-60`).
2. Run the same load curve against the no-Shield `hello_world` example and the Shield build. The delta is system overhead; the ceiling is hardware.
3. Classify saturation: if RPS plateaus while CPU stays below saturation, suspect OS or software limits (file descriptors, `somaxconn`, pool depth, as seen in the Neon finding at `measurement-2026-05-22-neon-saturacion.md:44-61`); if CPU saturates, it is a hardware limit.
4. Record governor and quiet-host status as validity conditions (`:36-37`).

### 5.5 Tooling

`oha` is a measurement tool, not a new product component, so no new crate is created. The protocol and target definitions are governed by rules 22 and 28; sub-issues under #157 follow rule 29 (one issue per problem, with acceptance criteria and labels).

## 6. KPI and verifiable acceptance criteria

- K1: at least three independent runs per load point with median and sample standard deviation reported.
- K2: every rule-17 field filled; all scenarios either measured or explicitly `not measured` (`verification-package.md:126-127`).
- K3: final report at the stabilization commit on reference hardware, with metadata file, raw logs and curated report committed (`verification-package.md:129-138`).
- K4: the release gate `Benchmarks` row and README performance claim reconciled only after verification (`:196-206`, rule 27).
- K5: issue #157 stays open and labelled `blocked` until every box in `verification-package.md:196-206` is checked; PR descriptors use `Refs #157`, not `Closes #157`, until then.
- K6: smoke and wave9 documents remain labelled non-publishable.

## 7. Risks

- Human approval is required to install the load generator, provision reference hardware and fund machine time.
- Publishable figures cannot be produced on informative hardware; do not relax this (rule 36 reproducibility, rule 17).
- Unbounded concurrency can distort results or crash the service; record the failure point as data, not as a pass.

## 8. Impact

- Scope: benchmark procedure, tooling installation, documentation and release gate.
- Code: none in this RFC. Public APIs: none.
- Documentation: `docs/benchmarks/`, gate row, README performance claims (both English and Spanish sections).

## 9. Rollback

No product change. The protocol can be suspended at any time; raw artifacts remain for audit.

## 10. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-22.
- Result: protocol accepted. Comment period waived by BDFL decision.
- Installation of `oha`, provisioning of reference hardware and machine-time cost remain separate human approvals; no load test or install-time measurement has been executed.
