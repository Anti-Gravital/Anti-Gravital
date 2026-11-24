# S6 producer mode - readiness audit (top5-roadmap, task S6)

- Date: 2026-11-23
- Scope: Issue #112 (RFC-0012 S6), `ag-edge` producer port and `ag-workers` producer feature
- Verdict: READINESS AUDIT ONLY. No production code changed. Issue #112 stays OPEN.
- Authority: proposal with evidence; the coordinator closes #112 only after a real emitter exists and full verification passes.

## 1. Question and verdict

The task asks whether S6 can advance honestly. Issue #112 says "when a concrete
consumer exists" and explicitly defers the work because `ag-edge` has no
background-work need today. The two candidate paths were:

- (a) find a real `ag-edge` event that emits through the port today, wire it and
  test it;
- (b) if none exists, document the exact gap rather than invent an emitter.

Verdict: (b). Exhaustive search of `crates/ag-edge/src/` found no emitter. The
only call to `emit()` in `ag-edge` is inside the port's own unit test; the only
`emit()` call in a runnable program is in the example's host process. There is
no `ag-edge -> ag-workers` dependency in either direction. Adding an emitter now
would require inventing an event with no producer and no consumer, which rules
5, 8, 21, 28 and 30 of `CLAUDE.md` forbid. No production code was changed.

## 2. What is already delivered (evidence)

| Item | Evidence |
| --- | --- |
| `ag-edge` producer port: `EnqueueRequest`, `DeferredJobProducer`, `ProducerError`, `emit` | `crates/ag-edge/src/producer.rs:36,64,96,105`; gated by `#[cfg(feature = "producer")]` at `crates/ag-edge/src/lib.rs:35-36` |
| Feature declared, dependency-free | `crates/ag-edge/Cargo.toml:30` - `producer = ["dep:async-trait"]`; no `ag-workers` entry in `[dependencies]` |
| Backpressure-shaped error contract | `crates/ag-edge/src/producer.rs:64-88` - `Backpressure { queue }` and `Backend(String)`, both with `Display`/`Error` |
| One behavioural unit test | `crates/ag-edge/src/producer.rs:113-140`, test `emit_forwards_request_to_adapter`; passes (see section 7) |
| `ag-workers` enqueue-only mode | `crates/ag-workers/src/config.rs:99` (`WorkerMode::Producer`) and `:110` (parser accepts `"producer"`); feature `producer = []` at `crates/ag-workers/Cargo.toml:31` |
| Reference adapter in an example | `examples/workers-producer-edge/src/bridge.rs:26-45` maps `QueueError::Full` to `ProducerError::Backpressure` and other errors to `ProducerError::Backend`; no lease, scheduler or spawn in the adapter |
| End-to-end pattern demonstration | `examples/workers-producer-edge/src/main.rs:90-101` emits four opaque payloads; a separate `WorkerPool` leases and runs them (`:118-160`); `WorkerMode::Producer` asserted at `:82` |
| Honest status already recorded | `docs/roadmap/STATUS.md:359-361` and `:402-413`; `crates/ag-workers/README.md` Status/Partial sections |

## 3. Exact gap

1. **No event source in `ag-edge`.** `crates/ag-edge/src/lib.rs:1-35` defines the
   crate as request-time data-plane logic: hostname routing, SNI certificate
   selection, ACME HTTP-01 challenge serving and canonical/redirect policy. The
   sole background task is the HTTP listener at
   `crates/ag-edge/src/server.rs:268` (`tokio::spawn`), which serves requests and
   does not enqueue work.
2. **The port is inert outside its test.** Content scan of
   `crates/ag-edge/src/*.rs` for `emit`, `EnqueueRequest`, `DeferredJobProducer`
   and `producer` returns only matches inside `producer.rs` and the feature gate
   in `lib.rs`. No production call site exists.
3. **Emission is performed by the host.** In
   `examples/workers-producer-edge/src/main.rs:90` the `emit()` call is written
   in the example binary, i.e. the host process. This matches
   `docs/roadmap/STATUS.md:411-413`: "hoy la emision la hace el proceso anfitrion".
4. **The plausible event is not an emitter.** `crates/ag-edge/src/tls.rs:68`
   `allow_on_demand` is a pure predicate over an SNI string and a
   `is_known_verified` callback. It returns a `bool`; it performs no enqueue and
   has no call site that defers work (`grep -rn on_demand` over `ag-edge`,
   `ag-domains` and `ag-cli` finds only `crates/ag-edge/README.md:17`). The
   "peticion de emision on-demand" named in `STATUS.md:410` is a hypothetical
   example, not an implemented event.
5. **The real use case lives in another crate.** Issue #112 states that ACME
   renewal (the use case the RFC cites) lives in the `ag-domains` control plane,
   not the `ag-edge` data plane.

## 4. Why an emitter cannot be added now

Adding, for example, an `on_demand_issuance` event inside `ag-edge` that calls
`emit()` would require a queue name, a handler kind, a payload schema, a version
and a worker that consumes them. None of those exists. This violates:

- **CLAUDE.md rule 5** (`CLAUDE.md:143-159`): nothing is implemented that is not
  in the documentation or that alters the phase/scope. A new event changes the
  module contract and needs an RFC first (rule 28, `CLAUDE.md:698-716`).
- **CLAUDE.md rule 8** (`CLAUDE.md:201-212`): no premature future systems, no
  code prepared for distant features, no speculative abstractions.
- **CLAUDE.md rule 21** (`CLAUDE.md:506-518`): rejects implicit coupling and
  premature abstraction; the system must stay auditable and explicit.
- **CLAUDE.md rule 28** (`CLAUDE.md:698-716`): altering module boundaries or
  crate dependencies requires an RFC before code.
- **CLAUDE.md rule 30** (`CLAUDE.md:820-827`): eliminate > simplify > refactor >
  add; unused functionality must be evaluated for removal, not extended.

It would also break the honest-status invariant (K-1, K-10): a test would pass
against a self-invented producer while the issue would be closed on a capability
nobody uses. Issue #112 itself names this exact reasoning in its
"Decision / why this is deferred (not done now)" section.

## 5. Contract a future emitter must satisfy

When a real event source is introduced through an RFC/ADR and appears in code,
the wiring must meet all of the following. This is a specification, not an
implementation done here.

1. **Real trigger.** A named `ag-edge` (or concrete serverless target) request
   path must require durable deferred work. It must be documented in the RFC
   and present in code as an event with a caller; ACME renewal routed through
   `ag-workers` is the example in issue #112, but the `ag-domains` owner must
   decide whether that belongs in `ag-edge` at all.
2. **Boundary decision.** Either (i) keep the current dependency-free port and
   place the `DeferredJobProducer` adapter in the concrete consuming crate (the
   pattern the example proves today), or (ii) add
   `ag-workers = { default-features = false, features = ["producer"] }` to
   `ag-edge` behind its `producer` feature, as issue #112 "Done when" states.
   Choice (ii) changes a crate boundary and therefore needs an RFC/ADR and the
   architect's review; this audit does not choose it. In both cases no worker
   runtime, leasing loop or scheduler may be pulled into the edge binary.
3. **Emit shape.** Use `ag_edge::producer::emit` with
   `EnqueueRequest::new(kind, queue, payload_bytes, payload_version)` from
   `crates/ag-edge/src/producer.rs:36,105`. Payload bytes are already encoded by
   the caller (MessagePack via `encode` plus `JobPayload::PAYLOAD_VERSION` in the
   example); `ag-edge` stays schema- and backend-agnostic.
4. **Adapter duties.** Implement `DeferredJobProducer::enqueue` only; map
   `QueueError::Full` to `ProducerError::Backpressure` and all other queue
   errors to `ProducerError::Backend`, as in
   `examples/workers-producer-edge/src/bridge.rs:26-45`. No lease, dispatch or
   spawn in the adapter.
5. **Error handling (S-7).** Propagate `ProducerError` to the caller; never swallow
   it and never fabricate a success identifier. Backpressure must reach the
   request path as an explicit admission outcome.
6. **Data and observability (S-1, S-8, L-12).** No secrets or personal data in
   payloads or logs; metric labels stay in the bounded set (`queue`, `kind`,
   `status`, `outcome`, `reason`) with no `tenant_id` (RFC-0012:164-165).
7. **Integration test.** Show `ag-edge` emitting a job that a *separate* worker
   process leases and runs. The one-process example is the pattern proof, not
   the deliverable; the issue requires the cross-process case.
8. **Dependency direction.** `ag-workers` must still not depend on `ag-edge`.
   `cargo tree -p ag-edge --features producer -e features` must show only the
   permitted producer surface, and the CI dependency-cycle check must stay green.
9. **No GA claim.** The pre-Phase-5 release gate is open; the status wording must
   keep saying "partial" until a real emitter and its verification land.

## 6. Exact condition to close #112

Issue #112 is closed only when ALL of these hold (quoted from its "Done when"
and "Verification" sections):

1. A concrete `ag-edge` (or serverless target) use case needs to enqueue durable
   work, with the event and its consumer named in an RFC/ADR and visible in code.
2. `ag-edge` depends on `ag-workers` with `default-features = false`,
   `features = ["producer"]` only, no worker runtime pulled in (or an approved
   equivalent adapter placement per section 5.2 above).
3. An integration test shows `ag-edge` enqueues a job that a separate worker
   process leases and runs.
4. The dependency direction is confirmed acyclic: `ag-workers` still must not
   depend on `ag-edge`.
5. `cargo tree -p ag-edge -e features` shows only the `producer` feature of
   `ag-workers` (where applicable), and the CI dependency-cycle check is green.
6. `docs/roadmap/STATUS.md:402-413`, the `ag-workers` README status section and
   this audit are updated in the same change that adds the emitter.

This change satisfies none of 1-4; therefore `Closes: none` and `Refs #112` in
the descriptor. The coordinator owns closure.

## 7. Verification evidence (commands actually run)

Environment: cargo 1.99.1, rustc 1.99.1;
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo test -p ag-edge --features producer
   Compiling ag-core v0.1.0-alpha.2
   Compiling ag-domains v0.1.0-alpha.2
   Compiling ag-edge v0.1.0-alpha.2
    Finished `test` profile [unoptimized] target(s) in 2.20s
running 24 tests
test producer::tests::emit_forwards_request_to_adapter ... ok
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   Doc-tests ag_edge: 0 passed; exit 0
TEST_RC=0
```

```
$ cargo tree -p ag-edge --features producer | grep -i ag-workers
NO ag-workers in ag-edge tree (expected)
$ cargo tree -p ag-edge --features producer -e features | grep -i ag-workers
NO ag-workers in feature tree (expected)
$ cargo tree -p ag-workers --features producer | grep -i ag-edge
NO ag-edge in ag-workers tree (expected)
```

The root of the tree is `ag-edge v0.1.0-alpha.2 -> ag-domains -> ag-core -> ...`;
no background-execution crate appears. Feature and reverse trees agree: the
dependency direction is acyclic and the port remains dependency-free.

Content scans (no emitter outside the port/test):

```
$ grep -rn "EnqueueRequest\|DeferredJobProducer\|emit(\|producer" crates/ag-edge/src crates/ag-edge/Cargo.toml
  matches only crates/ag-edge/src/producer.rs and the feature gate at
  crates/ag-edge/src/lib.rs:19,35-36 and crates/ag-edge/Cargo.toml:27,30,42
$ grep -rn "spawn\|tokio::spawn\|producer\|emit" crates/ag-edge/src/*.rs | grep -v src/producer.rs
  crates/ag-edge/src/lib.rs:19,20 ; crates/ag-edge/src/server.rs:268 (HTTP listener)
$ grep -rn "on_demand\|OnDemand" crates/ag-edge crates/ag-domains crates/ag-cli
  crates/ag-edge/README.md:17 ; crates/ag-edge/src/tls.rs (pure predicate only)
```

No production file was modified by this task, so these results describe the
baseline, unchanged state.

Repository gate on the branch with all audit artifacts present:

```
$ git diff --check
GIT_DIFF_CHECK_RC=0
$ bash scripts/ag-check
PASS: descriptors
PASS: fmt
PASS: clippy
PASS: test
PASS: audit
PASS: deny
PASS: masters
PASS: prohibited
Summary: PASS=8 FAIL=0 SKIP=0
ag-check: all selected checks passed.
AG_CHECK_RC=0
```

## 8. Finding: stale RFC section references

Issue #112 and `docs/roadmap/STATUS.md:404` cite "RFC-0012 section 17.4", and
issue #112 also cites "section 32" and "section 7". RFC-0012 has ten sections
(`docs/rfc/RFC-0012-ag-workers.md:27-275`, headings `## 1` to `## 10`); the S6
producer-mode text is at `:187-188` (section 5, phased plan) and the dependency
direction at `:131-145` (section 4). `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md:1142`
has a section 17.4, but it is "Compatibility", not producer mode. The references
are therefore stale, not a hidden specification. Reported via the S-11 outbox
as `s6-rfc-section-drift.md`. This audit cites the actual RFC locations.

## 9. Non-claims

- No emitter, event, dependency or integration test was added.
- `docs/roadmap/STATUS.md` S6 checkbox stays `[/]`; no issue was closed.
- No dependency on `ag-workers` was introduced in `ag-edge` in either direction.
- No production-readiness or GA claim is made; the pre-Phase-5 gate remains open.

## 10. Revalidation 2026-11-23 (wave 29, issue #112)

- Date: 2026-11-23T23:56Z
- Scope: re-check the single condition that could flip the verdict of section 1:
  whether any real `ag-edge` event now needs to defer a job through the port.
- Verdict: UNCHANGED. No real emitter exists, so no code was added. This is an
  additive note only. Issue #112 stays OPEN with `Refs #112`.

### What was re-checked

| Candidate emitter point | Result |
| --- | --- |
| Background tasks in `crates/ag-edge/src/` | The only `tokio::spawn` is at `server.rs:268`, the per-connection task inside the TLS accept loop. It serves one request and enqueues nothing. |
| On-demand TLS / ACME issuance trigger | `tls.rs:68` `allow_on_demand` is a pure `bool` predicate over an SNI string and an `is_known_verified` callback. No production caller defers work; only its own unit tests reference it. |
| Config reload | `grep -rni reload crates/ag-edge --include=*.rs` has no match. No reload path exists to attach an enqueue to. |
| Port call sites | `grep -rnE 'emit\(|\.enqueue\(|DeferredJobProducer' crates/ag-edge/src --include=*.rs` matches only `producer.rs` (definition and its own unit test). No runnable production path emits. |
| Dependency direction | `cargo tree -p ag-edge --features producer` and `-e features` show no `ag-workers`; `cargo tree -p ag-workers --features producer` shows no `ag-edge`. |

No real trigger, queue name, handler kind, payload schema or consumer exists, so
adding an emitter would still violate CLAUDE.md rules 5, 8, 21, 28 and 30 and
would close #112 on a self-invented producer. Section 5 (contract for a future
emitter) and section 6 (exact closure conditions) are unchanged.

### Verification evidence (commands actually run)

Environment: cargo 1.99.1, rustc 1.99.1;
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo test -p ag-edge --features producer
running 24 tests
test producer::tests::emit_forwards_request_to_adapter ... ok
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Doc-tests ag_edge: 0 passed
TEST_RC=0

$ cargo clippy -p ag-edge --features producer --all-targets -- -D warnings
Finished `dev` profile
CLIPPY_RC=0

$ cargo tree -p ag-edge --features producer | grep -i ag-workers
NO ag-workers (expected)
$ cargo tree -p ag-edge --features producer -e features | grep -i ag-workers
NO ag-workers (expected)
$ cargo tree -p ag-workers --features producer | grep -i ag-edge
NO ag-edge (expected)
```

Repository gates and the full command log on branch `community29/i112` are
recorded in the descriptor `docs/kaizen/pr-drafts/community29-i112.md` and its
mirror `docs/pr-drafts/community29-i112.md`.

### Non-claims

- No emitter, event, dependency, integration test or production code was added.
- `docs/roadmap/STATUS.md` S6 checkbox stays `[/]`; #112 is not closed.
- No GA claim is made; the pre-Phase-5 release gate remains open.
