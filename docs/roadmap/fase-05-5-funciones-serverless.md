# Fase 5.5 - Funciones serverless

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)
> Siguiente: [fase-06-ag-ai-knowledge-graph.md](./fase-06-ag-ai-knowledge-graph.md)

## Phase 5.5 — Serverless functions

**Status: proposed (`RFC-0023`). No code exists.** Additive phase, after the
public beta.

**Objective.** Give application code a unit smaller than the service: a
function that is deployed independently, started on demand, isolated in a
WebAssembly sandbox with explicit capabilities and hard limits, and idle at no
cost. Functions run natively on the user's own servers; external function
platforms and containers are packaging adapters.

**Duration:** 2 months.

### 5.5.1 Entry criteria

- [ ] `RFC-0023` accepted with its open questions resolved (native runtime or
  adapters only, position in the roadmap, second language).
- [ ] Version 0.5 (public beta) released.
- [ ] Phase 4.8 stages B and E complete (declared secrets; plan and apply).

### 5.5.2 Deliverables

- [ ] Host core in `ag-wasm-host`: engine, per-invocation memory, fuel and
  timeout limits, compiled-module cache. This is the work listed as stage 9-A;
  Phase 9 builds plugins on top of it.
- [ ] Versioned WIT interface for functions and a Rust function scaffold.
- [ ] `function` declarations in the DSL with compile-time validation of
  triggers, limits and capabilities.
- [ ] Triggers: HTTP through the Shield pipeline; schedule and queue through
  `ag-workers`; events.
- [ ] Capabilities, deny by default: data, cache, storage, mail, outbound
  HTTP to listed hosts, declared environment variables, job enqueue.
- [ ] Embedded mode (inside the service process) and dedicated mode.
- [ ] `ag functions new/list/build/invoke/deploy/rollback/logs`, with JSON
  output and interactive flows.
- [ ] Per-function metrics and traces through `ag-observe`.
- [ ] Container packaging and one external-platform adapter in `ag-cloud`.
- [ ] Guide for a second language.
- [ ] Documentation: when to use a function, a worker or the service.

Stages:

1. **5.5-A, host core and local invocation.**
2. **5.5-B, declarations and HTTP trigger.**
3. **5.5-C, asynchronous triggers on `ag-workers`.**
4. **5.5-D, capabilities**, one at a time, each with its denial test.
5. **5.5-E, deployment, rollback and observability.**
6. **5.5-F, packaging adapters and second language.**

### 5.5.3 Exit criteria

- [ ] A function without a capability cannot read a file, open a connection
  or read an environment variable; each denial is covered by a test.
- [ ] A function that exceeds memory, fuel or timeout is terminated without
  affecting other invocations or the API, demonstrated under load.
- [ ] An example project runs one function per trigger type.
- [ ] Deploying or rolling back one function does not restart the service.
- [ ] Cold and warm invocation times are measured and published with the
  methodology of rule 17. No figure is claimed before that.

### 5.5.4 Phase risks

The main risk is **a sandbox flaw**, because functions run code with fewer
rights than the service in the same process. The mitigation is deny-by-default
capabilities, a denial test per capability, a pinned engine under the weekly
audit, and inclusion of the function host in the external audit of Phase 10.

The second risk is **expectation of parity with large function platforms**.
The mitigation is to state the limits in the documentation (no persistent
connections, no long-running work, no state between invocations) and to offer
adapters for those who need such a platform.

---

