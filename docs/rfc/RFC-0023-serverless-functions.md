# RFC-0023 - Serverless functions

- Status: accepted
- Decision date: 2026-11-10
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: new additive Phase 5.5
- Affected modules/crates: `ag-wasm-host` (function runtime), `ag-dsl`
  (`function` declarations), `ag-core` (HTTP trigger), `ag-workers` (schedule
  and queue triggers), `ag-cli` (`ag functions` commands), `ag-cloud`
  (packaging), `ag-observe`
- Predecessor RFC: RFC-0019 (project layout), RFC-0020 (secrets), RFC-0021
  (plan and apply, machine interface)
- Comment period: waived by BDFL decision.

## 1. Motivation

A backend is not only a long-running API. A large share of real work is small,
event-shaped code: handle a webhook, resize an uploaded image, run a nightly
reconciliation, react to a record being created. Today an Anti-Gravital user
has two options for that code: put it inside the main service, where a defect
or a slow dependency affects every endpoint, or stand up a second system.

Serverless functions give that code its own unit: deployed independently,
started on demand, isolated from the service and from each other, limited in
what it may consume and touch, and costing nothing while idle.

Naming: the feature is called **functions**. Brand names of third-party
function services are not used for it (`ADR-0011`).

## 2. Problem

1. There is no unit smaller than the service. All code shares one process, one
   set of permissions and one failure domain.
2. `ag-workers` runs background jobs, but a job handler is compiled into the
   service binary: it cannot be deployed or rolled back on its own, and it runs
   with the full privileges of the service.
3. Users who want on-demand execution must adopt an external function
   platform, with its own packaging, permissions and observability, outside
   the schema and outside the tool.
4. The WebAssembly host planned for plugins (Phase 9) is the right isolation
   mechanism for this, but the roadmap only uses it to extend the framework,
   not to run user code.

## 3. Alternatives considered

**A. Do nothing; use `ag-workers` for everything.** Rejected: workers solve
scheduling and retries, not isolation or independent deployment (problem 2).

**B. Functions as separate native processes, one per invocation.** Simple
model, any language. Rejected as the default: process start and teardown cost
is high for short invocations, isolation depends on operating-system
sandboxing that differs per platform, and limiting resources portably is hard.

**C. Functions as containers.** Strong isolation and the industry default.
Rejected as the native path: it makes a container runtime a requirement
(`ADR-0009` rule 2) and start-up is slow. Kept as a packaging target (4.8).

**D. Only an adapter to external function platforms.** Rejected as the only
path: an external service would be required to use the feature (`ADR-0009`
rule 2). Kept as an adapter (4.8).

**E. Functions as WebAssembly components executed by the Anti-Gravital host,
with external platforms as adapters (chosen).** Sandboxed by construction, no
ambient authority, portable across the four supported platforms, start-up
measured in milliseconds, and it reuses the host the roadmap already plans.

## 4. Proposed design

### 4.1 What a function is

A function is a WebAssembly component that exports one handler and is invoked
once per event. It has:

- no access to anything it was not granted: no file system, no network, no
  environment, no clock beyond what the host provides;
- hard limits per invocation: memory, CPU budget (fuel), wall-clock timeout;
- no state between invocations. State lives in the modules the function is
  granted (database, cache, storage).

A function is not a plugin. A plugin (Phase 9) extends the framework through
lifecycle hooks and is installed by an operator. A function is application
code invoked by an event. Both run on the same host and sandbox and differ in
the interface they implement.

### 4.2 Declaration

Functions are declared in `schema.ag`, so the schema stays the single source
of truth (rule 20) and every generator sees them:

```
function ResizeAvatar {
  trigger storage.uploaded(bucket: "avatars")
  input   UploadEvent
  memory  64mb
  timeout 10s
  allow   storage.read("avatars"), storage.write("avatars-thumbs")
}

function NightlyReconcile {
  trigger schedule("0 3 * * *")
  timeout 5m
  allow   data.query, env(PAYMENTS_API_KEY), http("api.payments.example")
}
```

The compiler validates that referenced models, buckets, declared environment
variables (`RFC-0020`) and hosts exist, and that limits are within the
project-wide maximums. Exact grammar is fixed by a DSL version of its own in
the implementation plan.

### 4.3 Triggers

| Trigger | Source | Delivery |
| --- | --- | --- |
| HTTP | A route in the Shield pipeline; authentication, rate limiting and validation apply before the function runs | Synchronous; response returned to the caller |
| Schedule | `ag-workers` scheduler (distributed-singleton claim already implemented) | At least once |
| Queue | An `ag-workers` job whose handler is the function | At least once, with the existing retry, backoff and DLQ |
| Event | `ag-realtime` subject, storage events, DSL-declared domain events | At least once |

Asynchronous triggers reuse `ag-workers` for durability and retries. The
function runtime adds no second queue.

### 4.4 Capabilities

Each `allow` entry is a capability the host hands to the function for one
invocation. Nothing is available by default.

| Capability | Grants |
| --- | --- |
| `data.query` / `data.write(Model)` | Typed access through `ag-data` to the listed models |
| `cache.read` / `cache.write` | `ag-cache` under a per-function key prefix |
| `storage.read(bucket)` / `storage.write(bucket)` | The named buckets only |
| `mail.send` | Sending through `ag-mail` |
| `http(host)` | Outbound HTTPS to the exact hosts listed |
| `env(NAME)` | The named declared variable; secrets are delivered as secrets and never logged |
| `jobs.enqueue(queue)` | Enqueue to the named queue |

The capability list is part of the deployed artifact and is shown on the
review screen before deployment. Widening it is a visible diff.

### 4.5 Authoring and languages

- Rust first: `ag functions new NAME` scaffolds a small crate in
  `functions/NAME/` with a generated, typed interface for its input, output
  and granted capabilities. It builds to a WebAssembly component.
- Any language that can produce a WebAssembly component for the same
  interface can be used. Go is the second documented language, consistent
  with the interoperability scope of `RFC-0019`.
- The interface is defined in WIT and versioned. A function built against
  interface version N keeps running on hosts that support N.

### 4.6 Runtime

- Hosted in `ag-wasm-host`, on the engine already chosen for plugins.
- **Embedded mode (default):** the function runtime runs inside the service
  process started by `ag dev` and in production. No extra process, no extra
  port.
- **Dedicated mode:** the same runtime as its own process (`ag functions
  serve`) for deployments that want function load separated from the API.
- Compiled modules are cached; instances are created per invocation and
  discarded. Concurrency is capped per function and globally; excess
  asynchronous work stays queued, excess synchronous requests are rejected
  with a retryable status.
- Exceeding memory, fuel or timeout terminates the invocation, records the
  cause and, for asynchronous triggers, follows the retry policy.

### 4.7 Commands

| Command | Effect |
| --- | --- |
| `ag functions new NAME` | Scaffold a function and its declaration |
| `ag functions list` | Declared functions, triggers, limits, deployed version |
| `ag functions build [NAME]` | Build to components; reports artifact size |
| `ag functions invoke NAME --input FILE` | Run locally with a given event |
| `ag functions deploy [NAME]` | Deploy one or all, with plan and apply (`RFC-0021`) |
| `ag functions rollback NAME` | Return to the previous version |
| `ag functions logs NAME` | Invocation log: time, duration, outcome, limit hit |

All have JSON output and are reachable from the terminal interface
(`RFC-0018`). `ag dev` rebuilds and reloads a function when its source changes.

### 4.8 External platforms and containers (adapters)

- A function can be packaged as a container image that embeds the runtime,
  for platforms that run containers on demand.
- Adapters for external function platforms live in `ag-cloud`, each behind its
  own Cargo feature and named after the platform it integrates, as adapter
  labels only (`ADR-0011`). They translate the platform's event into the
  function's input and run the same component.
- The declaration in `schema.ag` does not change with the target.

### 4.9 Observability

Per function, through `ag-observe`: invocation count, duration histogram,
cold versus warm instantiation time, outcome (ok, error, timeout, memory,
fuel), concurrency, queue delay for asynchronous triggers. One trace span per
invocation, linked to the triggering request or job.

### 4.10 Limits stated up front

Functions are not suited to, and will not be extended toward: persistent
connections (WebSocket, long streams), work longer than the maximum timeout,
in-memory state across invocations, or arbitrary native libraries that cannot
be compiled to WebAssembly. Those belong in the service or in `ag-workers`.

No start-up or throughput figure is promised by this RFC. Rule 17 applies:
numbers are published only after being measured on documented hardware.

### 4.11 Dependencies

The WebAssembly engine and component tooling already planned for
`ag-wasm-host` in Phase 9. Their versions are pinned conservatively, as the
Phase 9 risk section already requires. No other new dependency.

### 4.12 Testing

Sandbox escape tests (file system, network, environment, time) asserting
denial without the capability; limit tests for memory, fuel and timeout;
trigger tests per source; at-least-once and retry tests through `ag-workers`;
fuzzing of the event decoding path; a benchmark of cold and warm invocation;
interface compatibility tests across versions.

## 5. Implementation plan (only after approval)

1. PR 1 - host core in `ag-wasm-host`: engine, limits, module cache
   (the work scheduled as stage 9-A, pulled forward; see section 7).
2. PR 2 - WIT interface v1 and the Rust function scaffold;
   `ag functions new/build/invoke`.
3. PR 3 - DSL `function` declarations, validation and generators.
4. PR 4 - HTTP trigger through the Shield pipeline.
5. PR 5 - schedule and queue triggers through `ag-workers`.
6. PR 6 - capabilities, one per pull request, each with its denial test.
7. PR 7 - `ag functions deploy/rollback/logs`; observability.
8. PR 8 - container packaging; first external adapter.
9. PR 9 - second language guide; benchmark report.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Sandbox flaw lets a function exceed its capabilities | Low | Critical | Deny-by-default design; denial tests per capability; engine pinned and covered by the weekly audit; external review before 1.0. |
| WebAssembly component tooling still changing | High | Medium | Conservative pinning; versioned WIT interface; Rust first. |
| Users expect parity with large function platforms | Medium | Medium | Section 4.10 stated in the documentation; adapters for those who need a platform. |
| A noisy function starves the API in embedded mode | Medium | High | Per-function and global concurrency caps; fuel and time limits; dedicated mode. |
| Overlap with `ag-workers` confuses users | Medium | Low | One rule in the docs: workers schedule and retry, functions isolate and deploy independently; asynchronous functions run on workers. |
| Pulling the host forward delays other phases | Medium | Medium | The host core is needed by Phase 9 regardless; Phase 9 shrinks by the same amount. |

## 7. Impact

- **Scope:** a new product capability, inside the backend scope statement.
- **Roadmap:** new additive Phase 5.5 after the public beta. The host core
  moves from Phase 9 stage 9-A to Phase 5.5; Phase 9 keeps plugins, the
  registry and the official plugins.
- **DSL:** a new `function` declaration (its own DSL minor version).
- **CLI:** the `ag functions` command group.
- **Crate roster:** unchanged. The runtime lives in `ag-wasm-host`, whose
  description widens from "plugin host" to "sandboxed host for plugins and
  functions".
- **Documentation:** architecture chapter 9, a manual chapter, the module
  pages of `ag-wasm-host` and `ag-workers`.

## 8. Rollback

Functions are opt-in per project. If the design proves wrong, the `function`
declaration is deprecated and the trigger wiring removed; the host core
remains because Phase 9 needs it.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-10.
- Result: accepted. The comment period was waived by BDFL decision.

Resolution of the open questions:

1. **Meaning.** Native functions that run on the user's own servers. External
   function platforms and containers are packaging adapters.
2. **Position.** Additive Phase 5.5, after the v0.5 beta and before `ag-ai`.
   The host core moves from Phase 9 stage 9-A to Phase 5.5.
3. **Second language.** Go.

## 10. References

- `CLAUDE.md` rules 12, 15, 16, 17, 20, 28.
- `ADR-0009` rules 2 and 5; `ADR-0011`; `ADR-0013`.
- `RFC-0012` (`ag-workers`), `RFC-0019`, `RFC-0020`, `RFC-0021`.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`, chapter 9.
