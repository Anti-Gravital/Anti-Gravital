## ADR-0013: ag-workers execution model and authorization

**Status:** Aprobado
**Date:** 2026-06-08
**Author:** Angel Nereira (BDFL)
**Affected crates:** `ag-workers` (new, estándar diferido), `ag-dsl` (new `worker`
declaration), `ag-cli` (new `ag workers` subcommands), `ag-data` (consumed, unchanged),
`ag-observe` (consumed), `ag-mail` (later queue migration), `ag-edge` (producer-only)
**Master documents touched:** `ANTI-GRAVITAL-Hoja-de-Ruta.md` (Phase 4.6-D note);
`ANTI-GRAVITAL-Arquitectura-Tecnica.md` updated as later stages land.
**Governing RFC:** RFC-0012 (ag-workers).

---

## Context

The ecosystem repeats one pattern: deferred work behind HTTP handlers. `ag-mail`
already implements a queue with retry, exponential backoff, a persistent backend over
`ag-data` and worker execution (`crates/ag-mail/src/queue/mod.rs`,
`crates/ag-mail/src/queue/store.rs`, `FOR UPDATE SKIP LOCKED`). Phase 4.6 expands
`ag-mail` further (4.6-A `mta`, 4.6-C `api`), and future consumers (webhook delivery,
ACME renewal in `ag-domains`/`ag-edge`, post-upload processing in `ag-storage`,
notification fanout, reports) will each grow their own queue/retry/backoff/scheduler
unless a shared substrate exists.

RFC-0012 proposes extracting that proven pattern into a first-class crate
`ag-workers` (estándar diferido, ADR-0007 precedent) without breaking `ag-mail`'s
queue. This ADR records the execution-model decisions, the consequences, and the
explicit owner authorization to implement, so the codebase and the documentation stay
consistent (CLAUDE.md sections 0 and 26).

Two facts constrain the work. First, the workspace `[profile.release]` sets
`panic = "abort"`, so the engine cannot claim full panic isolation in a production
binary; a panicking job aborts the process, and the durable backend plus lease expiry
must make it recoverable — and a job that panics every time must not crash-loop the
process forever. Second, the pre-Phase-5 release gate
(`docs/audits/PRE_FASE5_RELEASE_GATE.md`) is open, which would normally defer new
feature work. The project owner (BDFL, ADR-0003) authorizes this work to proceed now,
additive and feature-gated, with no production-readiness/GA claim until the gate
closes.

## Decision

1. **Introduce `ag-workers` as an estándar diferido crate** beside the standard
   modules, never inside `ag-core`, with a strict acyclic dependency direction
   (`ag-workers -> { ag-data?, ag-observe?, ag-cache?, ag-realtime? }`, all
   feature-gated). It must not depend on `ag-mail`, `ag-auth`, `ag-cloud`,
   `ag-domains`, `ag-edge`, `ag-ui` or `ag-cli`. Roadmap slot: Phase 4.6-D.

2. **One job model.** A typed `JobHandler` trait with a closed registry; an explicit
   job state machine whose illegal transitions the persistence layer rejects;
   `rmp-serde`-encoded versioned payloads (`PAYLOAD_VERSION`) where a decode/version
   failure routes to the DLQ as `InvalidPayload` (never panic, never drop).

3. **Two first-class backends.** In-memory (default, native, pure Rust) and
   PostgreSQL (durable) via `ag-data`, behind the `postgres` feature, reusing
   `ag-data`'s pool and embedded-migration machinery (not a parallel `sqlx` surface).
   Leasing uses `FOR UPDATE SKIP LOCKED` with heartbeat and a reaper for abandoned
   leases. NATS/JetStream and Redis backends are deferred and feature-gated.

4. **Poison-job circuit breaker correct under `panic = "abort"`.** The attempt
   counter increments at lease time; a job exceeding `panic_guard_attempts` is routed
   directly to the DLQ without executing. This converts an infinite crash loop into a
   bounded, observable DLQ entry, and is the engine's most important production-safety
   property. `catch_unwind` worker isolation may exist only under a non-default
   feature where `panic = "unwind"` is explicitly enabled, never as the default
   reliability claim.

5. **Honest semantics.** At-least-once execution with idempotency support
   (`dedup_key`, exposed `job_id`/`attempt`), not exactly-once. CPU-bound jobs run on
   `spawn_blocking` gated by a fixed `Semaphore` (not `rayon`) so they never starve
   HTTP latency. Interval scheduling uses a `FOR UPDATE SKIP LOCKED` singleton claim
   on `ag_worker_schedules` so a job fires once under horizontal scaling. Backpressure
   and admission control are mandatory on every enqueue path. Observability metric
   labels are bounded and never include `tenant_id`.

6. **Three locked implementation decisions** (also recorded in RFC-0012 §9):
   - **Delivery:** documentation lands first; code lands in sequenced stages S1-S7,
     each green (`fmt`/`clippy`/`test`), per `CONTRIBUTING.md`'s small-PR rule. The
     scope (RFC-0012 §3.1) is fixed and does not grow; only the landing order is
     sequenced.
   - **Cancellation:** add `tokio-util` (0.7, `default-features = false`) to the
     workspace and use `CancellationToken`. It is the only genuinely new dependency;
     everything else reuses existing workspace deps.
   - **`enqueue_in_tx`:** accepts `&mut sqlx::Transaction<'_, Postgres>` behind the
     `postgres` feature. `ag-data` exposes no canonical transaction handle (`AgTx`)
     today; this matches `ag-mail`'s existing raw-sqlx usage and does not modify
     `ag-data`. The gap is tracked as a GitHub Issue (label `tech-debt`, per
     CLAUDE.md rule 29; `docs/DEBT.md` is frozen); a canonical `AgTx` is a future RFC.

7. **Authorization to implement before the pre-Phase-5 gate closes.** The owner
   authorizes the work now because it is additive, feature-gated, native-by-default
   and reversible. No production-readiness or GA claim is made for `ag-workers` until
   `docs/audits/PRE_FASE5_RELEASE_GATE.md` permits it. README and `//!` headers must
   state the real, in-progress status (ADR-0009 rule 1).

## Consequences

Positive:

- One boring, durable, observable substrate for all background work; future consumers
  stop re-implementing queues, retry, backoff, scheduling and metrics.
- The crash-loop hazard of `panic = "abort"` plus durable leases is closed by design.
- `enqueue_in_tx` gives the transactional-outbox property without a separate outbox
  table (the job is in the same database, in the same transaction).
- Additive, feature-gated and native-by-default, so adoption is opt-in and rollback is
  a member deletion plus doc reversion.

Negative / costs:

- A new crate and a new DSL declaration to maintain, plus one new workspace dependency
  (`tokio-util`).
- Two persistent Postgres queues (`ag-mail` and `ag-workers`) coexist until the
  staged migration (RFC-0012 §5 S7) retires the duplication; tracked as debt.
- `enqueue_in_tx` leaks `sqlx::Transaction` into call sites until a canonical `AgTx`
  is designed; tracked as debt.
- Implementing with the pre-Phase-5 gate open adds surface to stabilize before the
  gate can close; mitigated by feature gating, native defaults and the no-GA-claim
  rule.

## Alternatives

- **Per-module queues (status quo).** Rejected: guarantees duplication and divergent
  reliability semantics across `ag-mail`, `ag-domains`/`ag-edge`, `ag-storage`, etc.
- **Adopt an external broker (require Redis/NATS).** Rejected for v1: violates the
  native-first invariant (ADR-0009, CLAUDE.md section 12); offered later as optional
  feature-gated backends.
- **Build inside `ag-mail` and let others depend on it.** Rejected: wrong layering;
  `ag-mail` is a leaf consumer, not a substrate. A substrate that everything depends on
  cannot live in a leaf.
- **`rayon` for CPU-bound work.** Rejected for v1: an unjustified new dependency;
  `spawn_blocking` + a bounded semaphore suffices because Anti-Gravital CPU jobs are
  independent units. Revisit by ADR only if a real workload needs intra-job
  parallelism.
- **Add a canonical `AgTx` to `ag-data` now.** Deferred: it expands scope into a
  standard crate; raw `sqlx::Transaction` behind the `postgres` feature is sufficient
  for v1 and consistent with `ag-mail`.

## Supersession note

This ADR does not contradict any prior ADR. It builds on ADR-0007 (estándar diferido
classification), ADR-0009 (native-first / real-state) and ADR-0008 (language policy),
and is governed by RFC-0012.
