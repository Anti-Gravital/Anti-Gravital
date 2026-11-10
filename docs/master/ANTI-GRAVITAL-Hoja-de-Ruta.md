# Anti-Gravital — Hoja de Ruta y Puertas de Verificación

**[English](#english) | [Espanol](#espanol)**

**Versión:** 4.2 — Noviembre 2026 (enmiendas `RFC-0018` a `RFC-0023`; base 4.0 de mayo 2026)
**Organización:** Gravital Labs — Nereira Technology and Business Solutions
**Origen:** República de Panamá
**Estado:** Documento vivo. Se actualiza con cada release.

---

## English

## How to read this document

This document defines the sequence of phases that the Anti-Gravital project must pass through from its inception until it becomes a stable, market-ready version 1.0, with the promise fulfilled.

Each phase contains four blocks:

1. **Entry criteria**: conditions that must be met before the phase can begin. These come from the previous phase.
2. **Deliverables**: concrete artifacts that the phase must produce.
3. **Exit criteria (gate)**: conditions that must be met before moving to the next phase. They function as blocking gates: if they are not met, there is no advancement. This is non-negotiable.
4. **Phase-specific risks and mitigations**.

The deliverables and exit criteria are expressed as checkable boxes. This document is kept in the repository and is updated by crossing off what has been accomplished. It serves as the project's public dashboard.

The main rule is: **a phase is not considered concluded until all of its exit criteria boxes are checked**. The project may be temporarily paused between phases, but it cannot skip steps due to external pressure or urgency.

---

## Phase summary

Status uses explicit evidence-based labels, not a binary "complete". A phase is
not called complete while a blocking technical criterion is still open; external
adoption criteria (stars, contributors, blog posts) are tracked separately and do
not block technical advancement. The per-phase remaining gate work is in the
README "Evidence-based roadmap" table and the formal `PRE_FASE5_RELEASE_GATE.md`.

| Phase | Name                                       | Estimated duration | Status    |
|-------|--------------------------------------------|--------------------|-----------|
| 0     | Foundations and governance                 | 1–2 months         | In progress (external deliverables pending) |
| 1     | The Shield MVP                             | 2–3 months         | Implemented and tested; reference performance/coverage gate open |
| 2     | The Core MVP + roundtrip                   | 2 months           | Implemented; published benchmarks below the 40K req/s and p99 targets |
| 3     | Anti-DSL alpha (v0.1–v0.4)                 | 3 months           | Implemented; 24h fuzz and generated-vs-manual benchmark gates open (issue #70) |
| 4     | Standard modules (auth, data, realtime)    | 3 months           | Implemented and tested; scale evidence and documented debt open |
| 4.5   | `ag-mail` + `ag-domains`: communication and domains | 1–2 months | Implemented; `ag-domains` active, release/doc evidence open |
| 4.6   | Additive pre-Phase-5 hardening (`ag-mail` MTA, `ag-workers`) | — | Implemented and CI-verified; live-DB parity and producer wiring open (issues #108/#109/#103/#112) |
| 4.7   | Interactive terminal experience (`ag` TUI) | 1–2 months         | Proposed (`RFC-0018`); no code yet |
| 4.8   | Project foundations: layers, environment, installation, remote operation | 2–3 months | Proposed (`RFC-0019`, `RFC-0020`, `RFC-0021`); no code yet |
| 5     | `ag-cloud` — simplified deployment         | 2 months           | Next |
| 5.5   | Serverless functions                       | 2 months           | Proposed (`RFC-0023`); no code yet |
| 6     | `ag-ai` and Knowledge Graph                | 2 months           | Pending |
| 7     | `ag-migrate` — importers                   | 2 months           | Pending |
| 8     | `ag-mobile` — Flutter bridge               | 2 months           | Pending |
| 9     | WASI plugin system                         | 2 months           | Pending |
| 10    | Hardening and 1.0 milestone                | 3 months           | Pending |

**Total estimated duration:** 25–30 months from the start.
**Public beta version milestone (0.5):** end of phase 5 (~15 months).
**Stable version 1.0 milestone:** end of phase 10 (~30 months).

Phases 4.5, 4.6, 4.7 and 4.8 are additive: they do not move the two
milestones above, although Phase 4.8 stages A and B change what Phase 5 builds
on and are entry criteria of its stage 5-A. Phase 4.7 runs in parallel with the closing of the pre-Phase-5 gate and
with Phase 5; its stages A and B are exit criteria of Phase 5 (section 5.3) and
its stage C is an entry criterion of Phase 10 (section 10.1).

**How later phases are detailed (amendment of 2026-11-10).** From Phase 5
onward each phase carries, in addition to the four blocks, a **Stages** block
that orders its deliverables into independently mergeable steps and a list of
**RFC prerequisites** (rule 28: new commands, new DSL constructs and new
dependencies need an approved RFC before code). Stages add no scope: every
stage item is a deliverable already listed in the phase.

**Status at the close of Phase 4.5 (2026-05-24), updated for the pre-Phase-5
gate (2026-06-11).** Phases 1 through 4.5 are technically implemented and merged
to `main` (code, tests, fmt, clippy, audit and deny deliverables fulfilled), but
none is declared formally complete while the pre-Phase-5 release gate
(`docs/audits/PRE_FASE5_RELEASE_GATE.md`) is OPEN: the 24-hour fuzz, stabilized
benchmark and open-debt rows remain pending. The granular detail of each box
lives in `docs/roadmap/STATUS.md`, the operational dashboard. Phase 0 remains in
progress due to external deliverables (Discord, landing, domain). Phase 5
(`ag-cloud`) is next and may not start until every blocking gate row passes.

**Note on Phase 4.5.** Phase 4.5 is an **additive** phase introduced by
`ADR-0007` after closing Phase 4. It does not modify the scope nor the
deliverables of the already-completed Phase 4. It does not advance the v0.5 BETA milestone, which
remains at the end of Phase 5. The ecosystem count goes from 15 to 17
crates with the incorporation of `ag-mail` and `ag-domains`, and grows
additively to 20 with `ag-lsp` (Phase 3 DSL tooling), `ag-edge`
(`ADR-0012`) and `ag-workers` (`ADR-0013`, the second deferred standard).

> Phases 0-4.5 are technically implemented; formal completion is gated by the
> pre-Phase-5 release gate. Outstanding technical debt that must be closed before
> Phase 5 is tracked as GitHub Issues (label `tech-debt`, CLAUDE.md rule 29);
> `docs/DEBT.md` is a frozen historical record.

---

## Phase 0 — Foundations and governance

**Objective.** Create the project's foundations: repository, license, governance documentation, CI, contributors, communication with the community. No code yet. The product of this phase is an open source project fit to receive collaborators.

### 0.1 Entry criteria

- [ ] Final decision to begin Anti-Gravital as a formal Gravital Labs project.
- [ ] Approval of Apache 2.0 license without restrictions.
- [ ] Public commitment from Ángel Nereira as initial maintainer.

### 0.2 Deliverables

- [ ] Repository `github.com/anti-gravital/anti-gravital` created and public.
- [ ] `LICENSE` file with complete Apache 2.0 text.
- [ ] Bilingual `README.md` file (Spanish + English) with value proposition.
- [ ] `CONTRIBUTING.md` file with contribution guide, code conventions, pull request process.
- [ ] `CODE_OF_CONDUCT.md` file adopting Contributor Covenant 2.1.
- [ ] `SECURITY.md` file with responsible disclosure policy and address `anti@gravitalcloud.com` (backup: `angelnereira@gravitalcloud.com`).
- [ ] `GOVERNANCE.md` file describing initial BDFL model and transition plan.
- [ ] CI configuration with GitHub Actions: build on Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Issue templates (bug report, feature request, RFC) and pull request template.
- [ ] Basic branding: logo, color palette, typography. Applied to the README.
- [ ] Official project Discord with channels `#español`, `#english`, `#announcements`, `#help`.
- [ ] Project account on X/Bluesky for announcements.
- [ ] Domain `antigravital.dev` registered and pointing to a minimal landing page.
- [ ] Institutional email `anti@gravitalcloud.com` operational (project root email).
- [ ] Public release calendar published.

### 0.3 Exit criteria (gate before Phase 1)

- [ ] The repository receives its first unsolicited external star.
- [ ] At least five external people have joined the Discord.
- [ ] The monorepo's folder structure is defined and committed (although without functional content).
- [ ] The Cargo workspace is initialized with the empty crates: `ag-core`, `ag-dsl`, `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- [ ] The CI successfully builds the empty workspace on the four target platforms.
- [ ] The landing page describes in one paragraph what the project is, what it is not, and where it is on the roadmap.

### 0.4 Phase risks

The main risk is procrastination due to perfectionism. Phase 0 does not produce code that runs, which tempts to postpone it. The mitigation is a strict timebox: 8 weeks maximum. If by the end not all deliverables are in place, it concludes with whatever exists and the pending items are documented as phase 0 technical debt to be resolved during phase 1.

---

## Phase 1 — The Shield MVP

**Status: Technical implementation complete.** Detail of boxes in `docs/roadmap/STATUS.md`.

**Objective.** Implement the core's Shield layer: a Tower middleware pipeline that validates, performs basic authentication, applies rate limiting and delivers requests to a placeholder handler. No complete Core yet. No DSL yet. The product is a binary that responds over HTTP with basic security and a publishable benchmark.

### 1.1 Entry criteria

- [ ] Phase 0 completed with all of its exit criteria checked.
- [ ] At least one contributor in addition to the main maintainer is active in the repository.

### 1.2 Deliverables

- [ ] `ag-core` crate with operational `shield` module.
- [ ] HTTP/1.1 and HTTP/2 support via Axum + Tokio.
- [ ] TLS 1.3 termination with rustls.
- [ ] Basic payload validation middleware (deserialization with serde and simple constraints).
- [ ] JWT authentication middleware with Ed25519 verification.
- [ ] Rate limiting middleware with governor (token bucket per IP).
- [ ] CORS and CSRF middleware with secure defaults.
- [ ] Structured logging middleware with `tracing`.
- [ ] Minimal configuration from a TOML file.
- [ ] Unit tests with coverage ≥ 80% of the `ag-core` crate.
- [ ] End-to-end integration tests of the Shield pipeline.
- [ ] Executable Hello World benchmark: `cargo bench` produces reproducible figures.
- [ ] Crate API documentation generated with `cargo doc`, published on `docs.rs`.
- [ ] User manual chapter explaining how to use the Shield directly as a library.

### 1.3 Exit criteria (gate before Phase 2)

- [ ] Hello World benchmark reaches ≥ 300 K req/s on documented reference hardware.
- [ ] Shield pipeline p99 latency ≤ 1 ms at 100 K req/s.
- [ ] Idle process memory ≤ 15 MB.
- [ ] Startup time ≤ 100 ms.
- [ ] CI passes on the four target platforms.
- [ ] Static analysis with `clippy` without warnings.
- [ ] Dependency analysis with `cargo-audit` without known vulnerabilities.
- [ ] Zero undocumented `unsafe` blocks.
- [ ] At least one technical blog post published about the Shield architecture.
- [ ] At least ten stars on the repository.

### 1.4 Phase risks

The main risk is underestimating the complexity of TLS and rate limiting in production. The mitigation is to use exclusively proven crates (rustls, governor) and not to roll our own implementations. The secondary risk is that the benchmark figures do not reach the target; the mitigation is to publish what is measured with honesty and document the shortfall.

---

## Phase 2 — The Core MVP and complete roundtrip

**Objective.** Complete the core with the Core layer: Axum router, typed extractors, error system, shared state. Implement the complete roundtrip Request → Shield → Core → Handler → Response. Connect to real PostgreSQL for a minimal CRUD. The product is a binary that serves a real API, although written manually without DSL.

### 2.1 Entry criteria

- [ ] Phase 1 completed with all of its exit criteria checked.
- [ ] The `ag-data` crate has been started with sqlx as a dependency.

### 2.2 Deliverables

- [ ] `ag-core` crate with operational `core` module.
- [ ] Axum router integrated with the Shield.
- [ ] Extractors: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`, `Query<T>`.
- [ ] `AgError` error system with automatic conversion to HTTP response.
- [ ] Response system: JSON, plaintext, streams.
- [ ] `ag-data` crate with PostgreSQL connection pool via sqlx.
- [ ] Embedded migrations system with `sqlx::migrate!`.
- [ ] Example app `todo-api` in `examples/` with complete CRUD.
- [ ] Executable CRUD + DB benchmark.
- [ ] `ag-cli` crate with commands `new` (creates project from template), `dev` (starts server with hot reload via `cargo-watch`), `build` (compiles release).
- [ ] Three templates: `rest`, `realtime`, `fullstack`.

### 2.3 Exit criteria (gate before Phase 3)

- [ ] CRUD + PostgreSQL benchmark reaches ≥ 40 K req/s on reference hardware.
- [ ] CRUD p99 latency ≤ 5 ms.
- [ ] The `todo-api` app runs successfully with `ag new` + `ag dev`.
- [ ] The `todo-api` app deploys as a single binary (`FROM scratch` Docker).
- [ ] The release binary of `todo-api` occupies ≤ 20 MB.
- [ ] Documentation: "Your first API with Anti-Gravital" published.
- [ ] At least 50 stars on the repository.
- [ ] At least three external contributors with merged PRs.

### 2.4 Phase risks

The main risk is scope drift: wanting to add features not strictly necessary for the Core MVP. The mitigation is an explicit scope declaration in the phase ticket: the Core of this phase does not include complex RBAC authorization, does not include events, does not include cache, does not include complete observability. Those arrive in later phases.

---

## Phase 3 — Anti-DSL alpha (versions 0.1 to 0.4 of the DSL)

**Objective.** Build the DSL compiler with a deliverable subset of the grammar. This phase delivers the first functional codegen: models, basic endpoints, validations and relations. No declarative auth yet, no events yet. The product is the first version of the "define → generate → implement" flow.

### 3.1 Entry criteria

- [ ] Phase 2 completed with all of its exit criteria checked.
- [ ] `ag-dsl` crate started.
- [ ] Final decision on the compiler's base libraries (logos for lexer, chumsky for parser, askama and quote for codegen). Documented in RFC.

### 3.2 Deliverables

- [ ] DSL version 0.1: basic models with primitive annotations (`@primary`, `@unique`, `@auto`).
- [ ] DSL version 0.2: endpoints (method, path, body, response).
- [ ] DSL version 0.3: validations (`@min`, `@max`, `@email`, `@regex`, `@length`).
- [ ] DSL version 0.4: relations between models (`1:1`, `1:N`, `N:M`).
- [ ] Rust generator: structs with serde, validators, sqlx query builders.
- [ ] SQL generator: idempotent migrations.
- [ ] TypeScript generator: types and HTTP client.
- [ ] OpenAPI 3.1 generator.
- [ ] `ag generate` command that reads `schema.ag` and produces all the artifacts.
- [ ] `ag schema lint` command that reports best-practices warnings.
- [ ] `ag schema diff <ref>` command that reports breaking vs non-breaking changes.
- [ ] Readable diagnostics for common DSL errors (model not found, unknown type, invalid annotation).
- [ ] Basic LSP server (`ag-lsp`) with autocompletion and diagnostics.
- [ ] VS Code plugin published on the marketplace.
- [ ] Compiler test suite with coverage ≥ 85%.
- [ ] Parser fuzzing with `cargo-fuzz`: 24 hours without crashes.
- [ ] DSL reference documentation version by version.

### 3.3 Exit criteria (gate before Phase 4)

- [ ] A complete project can be created, defined in `schema.ag`, generated, and executed using only the CLI.
- [ ] The `ecommerce-api` example is completely rewritten with DSL and works.
- [ ] The benchmarks are maintained: DSL-generated CRUD is not slower than hand-written CRUD.
- [ ] The VS Code plugin has ≥ 100 installations.
- [ ] At least one external collaborator has contributed to the compiler.
- [ ] The DSL documentation is complete and reviewed by at least two people.
- [ ] At least 200 stars on the repository.

### 3.4 Phase risks

The DSL compiler is the technically most complex component of the project. The main risk is underestimating the effort and exceeding the schedule. The mitigation is incremental implementation by subversions: if the phase runs long, subversion 0.4 (relations) can be postponed to phase 4 without blocking advancement.

The secondary risk is the compiler's error messages. A compiler with incomprehensible messages ruins the experience. The mitigation is to prioritize readable diagnostics from day one, with specific tests that verify that the messages are useful.

---

## Phase 4 — Standard modules

**Objective.** Complete the batteries-included modules: auth, realtime, cache, storage, observe. Each as an independent crate, with tests, documentation and examples.

### 4.1 Entry criteria

- [ ] Phase 3 completed.
- [ ] DSL version 0.5 (auth and policies) started.

### 4.2 Deliverables

- [ ] DSL version 0.5: auth declaration and RBAC policies.
- [ ] DSL version 0.6: events declaration.
- [ ] Complete `ag-auth` crate: WebAuthn, JWT Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens with rotation.
- [ ] Complete `ag-realtime` crate: binary WebSocket, SSE fallback, embedded NATS for small cases, external NATS client for production.
- [ ] Complete `ag-cache` crate: moka L1 + Redis L2 with fred, event-based invalidation.
- [ ] Complete `ag-storage` crate: S3, MinIO, local filesystem adapters. Signed URLs. Image processing.
- [ ] Complete `ag-observe` crate: tracing, OpenTelemetry exporter, Prometheus metrics, Grafana JSON dashboards included.
- [ ] tokio-console integration in dev mode.
- [ ] `realtime-chat` example in `examples/`.
- [ ] `ai-backend` example in `examples/` that demonstrates SSE streaming.
- [ ] Cross-module integration tests.

### 4.3 Exit criteria (gate before Phase 5)

- [ ] The five modules published on crates.io with their respective independent releases.
- [ ] Test coverage ≥ 80% in each module.
- [ ] Documentation for each module: README, usage guide, API reference.
- [ ] Performance: the `ag-realtime` module sustains 50 K WebSocket connections on a 2 vCPU instance without degradation.
- [ ] Performance: the `ag-cache` module shows ≥ 1 M ops/second in L1.
- [ ] At least five bug report issues closed by the community.
- [ ] At least 500 stars on the repository.

### 4.4 Phase risks

The main risk is the fragmentation of effort among five parallel modules. The mitigation is to sequence the implementation: first auth (blocks many use cases), then advanced data, then realtime, then cache, then storage, then observe.

---

## Phase 4.5 — `ag-mail` + `ag-domains`: communication and domains

**Status: Technical implementation complete (2026-05-24).**

**Objective.** Add operational capabilities for transactional communication, DNS,
TLS and domains without overloading Phase 4 nor delaying the standard modules.
It prepares the ground so that `ag-cloud` (Phase 5) deploys applications with
domain, certificate and transactional email using an integrated
experience. The introduction of this phase is made official in `ADR-0007`.

**Duration:** 1–2 months.

### 4.5.1 Entry criteria

- [x] Phase 4 completed with all of its exit criteria checked.
- [x] `ag-auth` exposes hooks/events for email verification, password
  recovery and magic links.
- [x] `ag-observe` records metrics and traces of asynchronous jobs.
- [x] RFC approved for the initial scope of `ag-mail`. See RFC-0006.
- [x] RFC approved for the initial scope of `ag-domains`. See RFC-0007.

### 4.5.2 Deliverables

- [x] `ag-mail` crate (deferred standard): `MailSender` trait + `SmtpSender`
  (`lettre` + `rustls`). 38 tests.
- [x] HTML/plaintext templates: `MailTemplate` trait + `StringTemplate` with
  `{{var}}` substitution. External engines (askama, minijinja) integrable via
  trait. Compile-time var validation via `template::validate`.
- [x] Email declaration in `schema.ag` (`mail` block). DSL v0.7.
- [x] `ag-auth` → `ag-mail` integration for verification, recovery and
  magic links. `AuthMailer` with `"mail"` feature.
- [x] Asynchronous queue with retries and exponential backoff. `InMemoryQueue`
  backend. Persistent backend via `ag-data` deferred (TECH-DEBT documented).
- [x] Metrics towards `ag-observe`: `ag_mail_sent_total`, `ag_mail_retry_total`,
  `ag_mail_send_latency_seconds` (feature `"metrics"`).
- [x] `ag-domains` crate (optional infra): `DnsProvider` trait with Cloudflare
  adapter; declarative A/AAAA/CNAME/TXT/MX model. 28 tests.
- [x] ACME support (Let's Encrypt) via `instant-acme`: `issue()` +
  `issue_with_credentials()` + `spawn_renewal_task()`. DNS-01 challenge.
  TECH-DEBT: `notAfter` parsing for exact renewal.
- [x] Generation of SPF/DKIM/DMARC required by `ag-mail`. `apply_mail_records`
  idempotent (`ag-mail` ↔ `ag-domains` cooperation without dependency cycle).
- [x] Propagation verification against multiple public resolvers
  (`hickory-resolver`). `PropagationChecker` + `DEFAULT_RESOLVERS`.
- [x] DSL v0.7: `mail`, `domain`, `template` blocks; compiler validates that
  `from` references a declared `domain`, provider is valid, vars exist in
  templates, and DMARC policy is valid.
- [x] Update of the `ag-lsp` LSP for the new blocks: hover and completions for
  `mail`/`domain`/`template` and their 7 properties.
- [x] CLI commands: `ag domains check`, `ag domains sync`, `ag mail test`.
- [x] `auth-mail-demo` example in `examples/`: three flows with `NullSender`.
- [x] Documentation: "Configure domain, TLS and transactional email with
  Anti-Gravital". See `docs/manual/03-dominio-tls-correo.md`.

### 4.5.3 Exit criteria (gate before Phase 5)

- [x] `ag-mail` sends transactional HTML and plaintext email from an
  Anti-Gravital project via the native sender **and** via at least one adapter.
- [x] `ag-auth` uses `ag-mail` for email verification and password
  recovery in the `auth-mail-demo` example.
- [x] `ag-domains` implements functional `CloudflareProvider` with contract tests.
- [x] `ag-domains` issues and renews TLS certificates via ACME (Let's Encrypt
  staging/production).
- [x] `ag-domains` generates SPF/DKIM/DMARC required by `ag-mail`.
- [x] `ag domains check`, `ag domains sync` and `ag mail test` compile and pass CI.
- [x] 14 cross-module E2E tests in `tests/integration` (7 Phase 4 + 7 Phase 4.5).
- [x] Zero circular dependencies (green CI job).
- [x] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` and
  `cargo deny check` green.

### 4.5.4 Phase risks

The main risk is **confusing `ag-mail` with a complete MTA**. The
mitigation is the explicit restriction of the v1 scope to outbound + adapters;
inbound, IMAP/POP, persistent mailboxes and antispam remain documented as
out of scope, not as "deferred to v2".

The second risk is the **dependency on young upstreams** (`instant-acme`,
`hickory-resolver`) in domains where bugs are paid for dearly: a certificate
that does not renew brings down the site. The mitigation is a small
and versioned `DnsProvider` trait with contract tests, explicit pinning in the
workspace, and active monitoring of the evolution of the crates.

The third risk is **turning Anti-Gravital into a hosting panel** by
accumulation of capabilities. The mitigation is the project's interoperability
rule: both crates are abstractions with adapters, not replacements for
providers. The boundary is fixed in `ADR-0007` and does not move without a new
ADR.

### 4.5.5 Forward note — Phase 4.6 native MTA (`ADR-0010`)

That `ADR-0007` boundary has now moved, via the new ADR it required.
`ADR-0010` (2026-06-03) supersedes the v1 "NOT an MTA / inbound never"
restriction and expands `ag-mail` into a native outbound MTA, phased and
opt-in behind Cargo features, preserving the Native | Adapter pattern and the
implemented Phase 4.5 baseline. The work is phased Phase 4.6 (`RFC-0009`
section 5: stages A-D) plus continuous deliverability hardening in Phase 5+.
Phase 4.6-A (the native MTA core: MX resolution, ESMTP+STARTTLS delivery,
Ed25519 DKIM signing and bounce classification) is implemented behind the
opt-in `mta` Cargo feature; stages 4.6-B..D remain forward work. Phase 4.5
stays complete for its original outbound-relay scope. The provider adapters
remain a supported production path until native deliverability is proven.

---

## Phase 4.6 — Additive pre-Phase-5 hardening: native MTA and `ag-workers`

**Status: in progress.** Additive phase authorized with the pre-Phase-5 gate
open (`ADR-0010`, `ADR-0013`). It makes no production claim until the gate
closes. Live box-level state: `docs/roadmap/STATUS.md`.

**Objective.** Remove two structural dependencies before `ag-cloud` exists: the
need for a third-party relay to send email, and the absence of a shared
background execution engine. Both are opt-in behind Cargo features and keep the
native default path (`ADR-0009` rule 2).

The phase has two independent tracks. The label "4.6-D" was historically used
for two different things (the marketing stage of the mail track in `RFC-0009`
and the whole workers track in `RFC-0012`); this document names them explicitly
to remove the ambiguity.

### 4.6.1 Mail track — native outbound MTA in `ag-mail` (`RFC-0009`)

| Stage | Scope | State |
|-------|-------|-------|
| Mail A | MX resolution, ESMTP+STARTTLS direct delivery, DKIM signing, bounce classification (`mta` feature) | Implemented; protocol path exercised in CI against a loopback sink |
| Mail B | Two-tier queue, suppression list, per-destination traffic shaping, durable PostgreSQL spool (`queue-postgres` feature) | Implemented; live PostgreSQL test is `#[ignore]` |
| Mail C | Managed REST API, API keys, idempotency, signed webhooks, multi-tenancy (`api` feature) | Partial: HMAC-SHA256 webhook signing only; REST surface open (issue #152) |
| Mail D | Audiences, broadcasts, one-click unsubscribe, template components | Not started (issue #152) |

### 4.6.2 Workers track — `ag-workers` execution engine (`RFC-0012`)

| Stage | Scope | State |
|-------|-------|-------|
| S1–S4 | Job model, in-memory runtime, PostgreSQL backend, scheduling | Implemented |
| S5 | DSL v0.8 `worker` declarations, `ag workers` CLI | Implemented |
| S6 | Producer-only enqueue from `ag-edge` | Partial; waits for a concrete consumer (issue #112) |
| S7 | Migration of the `ag-mail` queue onto `ag-workers` | Implemented; parity verified against a live PostgreSQL |

### 4.6.3 Exit criteria

- [x] Mail A and Mail B merged behind opt-in features with the relay sender
  still the default.
- [x] Workers S1–S5 and S7 merged; the duplicated generic mail queue removed.
- [ ] Mail A gate of `RFC-0009`: a message delivered by the native MTA to a
  real external mailbox passes aligned SPF, DKIM and DMARC. Manual evidence
  recorded with date, commit and headers.
- [ ] Mail C and Mail D either delivered or formally moved out of Phase 4.6 by
  an RFC amendment (issue #152). They do not block Phase 5.
- [ ] Workers S6 delivered or formally deferred with its consumer named
  (issue #112). It does not block Phase 5.

### 4.6.4 Phase risks

The main risk is that an additive phase never closes because its optional
stages keep it open. The mitigation is the rule in 4.6.3: a stage that has no
consumer is moved out by an RFC amendment instead of staying "in progress".
The second risk is deliverability: a native MTA is only as good as the
reputation of the addresses it sends from, which no code can provide. The
mitigation is that the relay sender remains the default and the documented
production path until the Mail A gate has real evidence.

---

## Phase 4.7 — Interactive terminal experience (`ag` TUI)

**Status: proposed (`RFC-0018`). No code exists.** Additive phase.

**Objective.** Make creating and configuring a project discoverable and guided
from the terminal, through a full-screen interactive interface in the `ag`
binary, without weakening scripting: every interactive action has an exact
non-interactive equivalent. The product is the experience promised by rule 32:
a new user reaches a running project without reading the manual first.

**Duration:** 1–2 months. Runs in parallel with the gate closing and Phase 5.

### 4.7.1 Entry criteria

- [ ] `RFC-0018` accepted, including its three open questions (interface
  language, gate position, behavior of bare `ag`).
- [ ] `ag-cli` split into modules with no behavior change (typed request per
  command and a single executor), verified by the existing command-tree test.

### 4.7.2 Deliverables

Stage A — foundation and project creation:

- [ ] `tui` Cargo feature in `ag-cli` (default on); the binary builds and all
  commands work with `--no-default-features`.
- [ ] Terminal guard that restores the terminal on exit, error, `Ctrl-C` and
  panic.
- [ ] `ag new` wizard: name with live validation, template with description,
  review screen, result screen. `NAME` becomes optional.
- [ ] Global `--no-interactive` flag and `AG_NO_TUI`; automatic non-interactive
  mode when not a terminal, when `CI` is set or when `TERM=dumb`.
- [ ] Equivalent non-interactive command printed at the end of every flow.
- [ ] Monochrome theme honoring `NO_COLOR`; ASCII and box-drawing glyphs only.
- [ ] Degraded line-prompt mode below 80x24.
- [ ] A single interaction library in `ag-cli` (the previous prompt library is
  removed).

Stage B — project configuration:

- [ ] `ag config get`, `ag config set`, `ag config validate` (non-interactive,
  usable in CI).
- [ ] `ag config` form generated from the configuration types of `ag-core`,
  with inline validation, defaults and descriptions.
- [ ] Format-preserving, atomic save with a diff on the review screen.
- [ ] Secret fields accept only a file path or an environment variable name.
- [ ] `ag new --with MODULE` for the first standard module with tested scaffold
  composition, then the matching wizard step.

Stage C — home screen:

- [ ] Bare `ag` in a terminal opens a context-aware launcher (outside a project
  / inside a project). Outside a terminal it prints help as before.
- [ ] Every launcher entry runs an existing command and displays it.

Cross-stage:

- [ ] State-transition unit tests for every screen.
- [ ] Render snapshot tests at 80x24 and 120x40, color and monochrome.
- [ ] Parity tests: the request built by a wizard equals the request parsed
  from its printed equivalent command.
- [ ] Non-interactive regression tests: no command blocks or emits escape
  sequences with stdin closed.
- [ ] Interaction flow diagram in `docs/diagrams/`.
- [ ] `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`, root
  `README.md` and the first-API manual chapter updated in the same pull
  request as each stage.

### 4.7.3 Exit criteria

- [ ] A person who has never used Anti-Gravital creates and starts a project
  using only `ag new` in a terminal, with no flags and no documentation.
  Verified with at least three people external to the project; the session
  notes are published.
- [ ] Every command passes the non-interactive regression suite on the four CI
  platforms.
- [ ] The wizard works over SSH and inside `tmux`, and on the Windows console.
- [ ] The size increase of the release `ag` binary caused by the `tui` feature
  is measured and published.
- [ ] No interactive flow has a capability missing from the flags (parity
  suite green).

### 4.7.4 Phase risks

The main risk is **two diverging products**: an interactive tool and a
scripting tool that slowly stop behaving the same. The mitigation is
architectural, not procedural: screens produce the same typed request as the
flag parser and one executor performs the work, with parity tests in CI.

The second risk is **scope creep toward a terminal IDE** (log viewers,
editors, dashboards). The mitigation is the rule fixed in `RFC-0018`: a screen
is a front-end to an existing command; anything else requires a new RFC.

The third risk is **terminal diversity**. The mitigation is a conservative
glyph set, a monochrome theme, four-platform CI and the degraded line mode.

---

## Phase 4.8 — Project foundations: layers, environment, installation and remote operation

**Status: proposed (`RFC-0019`, `RFC-0020`, `RFC-0021`). No code exists.**
Additive phase. The maintainability rules that accompany it are proposed
separately in `RFC-0022` (governance, not a phase).

**Objective.** Give every generated project a defined, enforced structure;
make configuration and secrets declared, validated and safe by default; and
make the tool installable on a server with one verified command and operable
from any other device through the terminal interface of Phase 4.7, by people
and by automated agents alike. Deployment (Phase 5) builds on all three.

**Scope statement fixed by this phase (`RFC-0019`).** Anti-Gravital is a
backend tool. It integrates with Docker, Kubernetes and Terraform and does not
replace them; it offers gRPC as an optional transport and generated clients
for Go, WebAssembly and Flutter consumers; it does not render user interfaces.

**Duration:** 2–3 months.

### 4.8.1 Entry criteria

- [ ] `RFC-0019`, `RFC-0020` and `RFC-0021` accepted with their open questions
  resolved (default layout, fate of `ag-ui`, gRPC and Go in scope, default
  secret storage, remote transport, scope of `ag setup`, release key custody).
- [ ] Phase 4.7 stage A complete (the interactive foundation these flows use).

### 4.8.2 Deliverables

Stage A — project structure (`RFC-0019`):

- [ ] `ag.toml` project manifest read by every command.
- [ ] Layered workspace layout (domain, application, infrastructure,
  interface, composition root) for the `fullstack` and `realtime` templates,
  with the dependency rule enforced by Cargo.
- [ ] Compact layout with the same four layer names for the `rest` template.
- [ ] Infrastructure layer divided by concern, with the database adapter as
  its own bounded module that owns the migrations.
- [ ] `ag generate` writes each artifact into the layer it belongs to;
  `ag generate --check` for CI.
- [ ] Architecture chapter on project structure and updated first-API manual.

Stage B — environment and secrets (`RFC-0020`):

- [ ] `[env]` declaration in `ag.toml`; typed loader with fail-fast validation
  that reports every problem at once.
- [ ] `Secret<T>` type: redacted formatting, zeroed on drop, explicit access.
- [ ] Encrypted per-environment secrets file with recipient management.
- [ ] `ag env list/check/set/unset/recipients/keygen/export`.
- [ ] Modules receive typed configuration instead of reading the process
  environment directly (one crate per pull request).
- [ ] Interactive environment form with masked secret fields.

Stage C — installation (`RFC-0021`):

- [ ] Release workflow that publishes static binaries for the four targets, a
  checksum manifest and its detached signature.
- [ ] Installer that detects system and architecture, verifies signature and
  checksum, and needs no compiler. Closes issue #141.
- [ ] `ag setup --check` and `ag setup` (detect, plan, confirm, apply, verify)
  using the system package manager only.

Stage D — remote console (`RFC-0021`):

- [ ] `ag servers add/list/remove` (inventory without secrets).
- [ ] `ag connect NAME` opening the server-side interface over SSH, and
  `ag connect NAME -- COMMAND` for a single remote command.

Stage E — machine interface (`RFC-0021`):

- [ ] `--output json` on every command, with versioned schemas.
- [ ] Documented exit-code table.
- [ ] `ag describe` emitting the command tree, argument types and output
  schemas.
- [ ] `--plan` / `--apply PLAN_ID` on every command that changes state outside
  the working directory; local audit log of mutating commands.

### 4.8.3 Exit criteria

- [ ] A project created from each template builds and runs in CI from a fresh
  `ag new`, and a domain crate that imports a database or HTTP crate fails to
  compile.
- [ ] Starting an application with three invalid variables reports all three
  and opens no listener; no test can make a secret value appear in logs or
  command output.
- [ ] On a clean server of each supported distribution, the verified installer
  followed by `ag setup` produces a working installation with no manual step,
  and a tampered binary or manifest is rejected.
- [ ] From a second machine, `ag connect` reaches the server interface; the
  same session works from a phone SSH client.
- [ ] An automated client completes a create-configure-validate cycle using
  only `ag describe` and JSON output, and cannot change server state without
  an explicit plan identifier.
- [ ] Stages A and B complete before Phase 5 stage 5-A starts; stages C to E
  complete before the 0.5 release.

### 4.8.4 Phase risks

The main risk is **size**: this phase touches the scaffolder, every module's
configuration and the release process. The mitigation is that the five stages
are independent and each is delivered as several small pull requests; none
requires the others except B on A.

The second risk is **security regressions introduced in the name of
convenience**, especially in installation and remote access. The mitigation is
in the design: signature verification before anything is written, package
manager only, SSH as the only transport with no new daemon or port, and plan
before apply for every state change.

The third risk is **breaking existing users** when generator paths and module
constructors change. The mitigation is that no release has been published;
the changes land before 0.5 and are recorded as pre-release breaking changes.

---

## Phase 5 — `ag-cloud` simplified deployment

**Objective.** Build the deployment subsystem in the style of Railway/Fly.io. Support for the four targets: docker-compose, fly, railway, k8s. This is the **public beta version (0.5)** milestone.

### 5.1 Entry criteria

- [ ] Phases 4, 4.5 and 4.6 implemented and the pre-Phase-5 release gate
  (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) closed: every blocking row `pass`,
  including the 24-hour fuzz run (issue #156) and the final benchmark evidence
  (issue #157).
- [ ] RFC decision on the deployment targets supported in 1.0.
- [ ] Phase 4.8 stages A and B complete (project layout and declared
  environment), because deployment artifacts are generated from them.

RFC prerequisites (rule 28), each approved before its stage starts:

- [ ] RFC for the `deploy.ag` specification (new DSL surface).
- [ ] RFC for the `ag deploy` and `ag rollback` commands, including the
  secrets model and what is stored locally.

### 5.2 Deliverables

- [ ] `ag-cloud` crate with modules for each target.
- [ ] Specification of the `deploy.ag` file.
- [ ] Multi-stage Dockerfile generator optimized for minimal image.
- [ ] docker-compose target: complete stack generation with Caddy as reverse proxy and automatic TLS.
- [ ] fly target: integration with flyctl.
- [ ] railway target: integration with its API.
- [ ] k8s target: generation of standard manifests.
- [ ] Terraform integration: generated variable declarations and outputs for
  the resources a project needs; the user's Terraform owns the resources
  (`RFC-0019` section 4.6).
- [ ] `ag deploy` command.
- [ ] `ag rollback` command.
- [ ] Database migrations pipeline integrated into the deployment.
- [ ] Interactive deploy flow per `RFC-0018`: target selection, review screen
  listing every resource that will be created or changed, explicit
  confirmation; `ag rollback` shows what it will revert before acting. Flag
  parity is mandatory.
- [ ] Documentation: "From zero to production in 15 minutes" with each target.

Stages:

1. **5-A, specification.** `deploy.ag` grammar and validation; `ag-cloud`
   crate layout with one decoupled module per target behind its own Cargo
   feature; contract-test harness shared by all targets.
2. **5-B, image.** Multi-stage Dockerfile generator; reproducible image build;
   image size recorded.
3. **5-C, self-hosted target first.** docker-compose target with reverse proxy
   and automatic TLS; migrations pipeline; `ag deploy` and `ag rollback` for
   this target; optional `ag-domains` integration. This is the native,
   self-hostable path required by `ADR-0009` rule 5 and is delivered before
   any hosted target.
4. **5-D, k8s target.** Standard manifests generated from the same
   specification.
5. **5-E, hosted targets.** fly and railway, each as an adapter module.
6. **5-F, interactive flow and documentation.** Deploy wizard, per-target
   guides, release of 0.5.

### 5.3 Exit criteria (gate before Phase 6 and version 0.5)

- [ ] The `todo-api` example deploys successfully to Fly.io with `ag deploy`.
- [ ] The `ecommerce-api` example deploys successfully with docker-compose to a VPS and is accessed via domain with TLS.
- [ ] The `realtime-chat` example deploys successfully to Railway.
- [ ] Phase 4.7 stages A and B complete (interactive project creation and
  configuration).
- [ ] A failed deployment leaves the previous version serving, and
  `ag rollback` restores it; demonstrated on the docker-compose target.
- [ ] Version 0.5 (public beta) released on GitHub Releases.
- [ ] Public announcement on Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] At least ten external projects report that they have deployed Anti-Gravital in production or staging.
- [ ] At least 1 500 stars on the repository.

### 5.4 Phase risks

The main risk is the dependency on external APIs (Fly, Railway) that can change. The mitigation is to structure each target as a decoupled module with contract tests.

The second risk is that deployment is the first place where the tool handles
secrets and performs irreversible actions on infrastructure the user pays for.
The mitigation is the order of the stages (self-hosted first), the mandatory
review screen before any change, and a secrets model fixed by RFC before code.

---

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

## Phase 6 — `ag-ai` and Knowledge Graph

**Objective.** Build the AI module with the knowledge graph and the assisted capabilities.

### 6.1 Entry criteria

- [ ] Version 0.5 (public beta) released.
- [ ] Feedback from the first users incorporated into the backlog.

RFC prerequisites (rule 28):

- [ ] RFC for the knowledge-graph schema (`.ag/knowledge-graph.json`) and its
  versioning.
- [ ] RFC for the `ag ai` commands and the provider trait, including what
  project data may leave the machine and the consent model.

### 6.2 Deliverables

- [ ] Knowledge graph generator from the DSL AST.
- [ ] Persistence of the graph in `.ag/knowledge-graph.json`.
- [ ] Markdown architectural documentation generator from the graph.
- [ ] C4 diagram generator (Context, Container, Component) in Mermaid.
- [ ] Interactive graph dashboard in the dev server (`ag dev`).
- [ ] `ag ai suggest-schema` command with integration to a configurable provider.
- [ ] `ag ai review-migration` command.
- [ ] `ag ai analyze-architecture` command.
- [ ] Support for providers: Anthropic Claude, OpenAI, local Ollama, local vLLM.
- [ ] Offline mode where the AI functions are disabled but the framework works.
- [ ] Documentation: "Anti-Gravital + AI agents: the schema-first flow" with complete examples.

Stages:

1. **6-A, graph (no AI involved).** Generator from the DSL AST, persistence,
   regeneration on `ag generate`, deterministic output.
2. **6-B, derived documents.** Markdown architecture documentation and C4
   diagrams produced from the graph.
3. **6-C, dashboard.** Interactive graph view in the dev server.
4. **6-D, provider abstraction.** Provider trait, one local provider and one
   remote adapter, offline mode as the default.
5. **6-E, assisted commands.** `suggest-schema`, `review-migration`,
   `analyze-architecture`, each producing a reviewable proposal and never
   writing files without confirmation.

Stages 6-A to 6-C are useful without any AI provider and are delivered first.

### 6.3 Exit criteria (gate before Phase 7)

- [ ] The knowledge graph regenerates correctly with each `ag generate`.
- [ ] The generated architectural documentation is readable and useful (reviewed by three people external to the team).
- [ ] At least one user organization reports that it has integrated `ag ai` into its workflow.
- [ ] At least 2 500 stars on the repository.

### 6.4 Phase risks

The main risk is the dependency on external AI providers. The mitigation is provider abstraction and offline mode.

The second risk is data exposure: schema and code sent to a remote provider.
The mitigation is that remote providers are opt-in adapters behind Cargo
features (`ADR-0009` rule 2, `ADR-0011`), a local provider exists, and each
command states what it will send before sending it.

---

## Phase 7 — `ag-migrate` importers

**Objective.** Build the migration importers from legacy frameworks. It is probably the phase with the greatest impact on real adoption.

### 7.1 Entry criteria

- [ ] Phase 6 completed.
- [ ] Research of real samples: at least ten schemas/projects of each target framework collected as a testing corpus.

### 7.2 Deliverables

- [ ] `ag-migrate` crate with six importers:
  - [ ] OpenAPI 3.0 and 3.1 importer.
  - [ ] Prisma importer.
  - [ ] Django importer.
  - [ ] FastAPI importer.
  - [ ] Sequelize importer.
  - [ ] GraphQL SDL importer.
- [ ] `ag migrate from <framework> <path>` command.
- [ ] Official migration guides per framework with complete examples.
- [ ] Documented case study: real migration of a medium-sized FastAPI application.
- [ ] Import report for every run: what was translated, what was approximated
  and what was skipped, with source locations. Interactive review of the
  report per `RFC-0018` before any file is written.

RFC prerequisites (rule 28):

- [ ] RFC for the `ag migrate from` command and the import report format.

Stages:

1. **7-A, contract formats.** OpenAPI 3.0/3.1 and GraphQL SDL (machine-readable
   specifications; no source-code analysis).
2. **7-B, schema formats.** Prisma.
3. **7-C, framework sources.** Django, FastAPI, Sequelize, in the order set by
   the size of the collected corpus.
4. **7-D, guides and case study.**

### 7.3 Exit criteria (gate before Phase 8)

- [ ] Each importer has test coverage ≥ 80% over the corpus of real projects.
- [ ] The FastAPI migration guide has been validated by at least one external team that migrated its application.
- [ ] At least 3 500 stars on the repository.

### 7.4 Phase risks

The importers cover the translation of the contract, not the business logic. The risk is generating exaggerated expectations. The mitigation is honest documentation about what is imported and what is not.

---

## Phase 8 — `ag-mobile` Flutter bridge

**Objective.** Build the integration with Flutter as the priority mobile target. Generation of complete Dart SDK, native auth, realtime.

### 8.1 Entry criteria

- [ ] Phase 7 completed.
- [ ] At least one collaborator with significant Flutter experience has joined the project.

### 8.2 Deliverables

- [ ] `ag-mobile` crate with Dart generator.
- [ ] `anti_gravital` pub package published on pub.dev:
  - [ ] Types generated with freezed.
  - [ ] HTTP client with dio + interceptors.
  - [ ] WebSocket client.
  - [ ] SSE client.
  - [ ] Mocks for tests.
- [ ] Authentication widgets: registration and login with native WebAuthn (Android Credential Manager, iOS Passkeys), OAuth2.
- [ ] `flutter-fullstack` example in `examples/`: complete Flutter app with Anti-Gravital backend.
- [ ] Documentation: Flutter user guide.

Stages:

1. **8-A, types and HTTP.** Dart generator, generated types, HTTP client,
   mocks; the CRUD case end to end.
2. **8-B, realtime.** WebSocket and SSE clients.
3. **8-C, authentication.** Native WebAuthn and OAuth2 widgets.
4. **8-D, example, guide and pub.dev publication.**

### 8.3 Exit criteria (gate before Phase 9)

- [ ] The `anti_gravital` package on pub.dev has at least 50 likes.
- [ ] The `flutter-fullstack` example runs on Android, iOS and web.
- [ ] At least one external Flutter application uses Anti-Gravital in staging or production.
- [ ] At least 4 500 stars on the repository.

### 8.4 Phase risks

The main risk is that the Rust → Dart context switch has unforeseen frictions. The mitigation is to start with the simplest case (CRUD) and build incrementally.

---

## Phase 9 — WASI plugin system

**Objective.** Build the WASI plugin system with wasmtime, define the stable ABI, publish the official plugins, and start the public registry.

### 9.1 Entry criteria

- [ ] Phase 8 completed.
- [ ] RFC decision on the scope of the 1.0 plugin ABI. Approved by the technical committee (formed in phase 4 or earlier).

### 9.2 Deliverables

- [ ] `ag-wasm-host` crate operational over wasmtime.
- [ ] Definition of WIT interfaces (WebAssembly Interface Types) for the host.
- [ ] Specification of `plugin.toml`.
- [ ] Implementation of the plugin life cycle (discovery, validation, loading, activation, unloading).
- [ ] Sandbox with memory, fuel and timeout limits.
- [ ] Official plugins: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`.
- [ ] `ag plugin add/remove/list` command.
- [ ] Public registry at `plugins.antigravital.dev`.
- [ ] Guide: "How to write a plugin for Anti-Gravital" with examples in Rust, Go (TinyGo) and AssemblyScript.
- [ ] Plugin installation shows the capabilities a plugin requests and asks
  for confirmation (interactive per `RFC-0018`, with flag parity).

RFC prerequisites (rule 28), in addition to the ABI RFC of 9.1:

- [ ] RFC for the `ag plugin` commands and the registry trust model
  (signing, integrity verification, revocation).

Stages:

1. **9-A, host.** WIT interfaces, `plugin.toml`, life cycle, sandbox limits.
2. **9-B, CLI.** `ag plugin add/remove/list` against local plugins.
3. **9-C, official plugins.**
4. **9-D, registry.** Public registry with integrity verification.
5. **9-E, guide and overhead benchmark.**

### 9.3 Exit criteria (gate before Phase 10)

- [ ] The registry publishes at least the six official plugins.
- [ ] At least three third-party external plugins published in the registry.
- [ ] The benchmark shows plugin overhead ≤ 1% over an equivalent native handler.
- [ ] At least 6 000 stars on the repository.

### 9.4 Phase risks

The main risk is the complexity of the WebAssembly component model, which keeps evolving. The mitigation is conservative pinning of the supported version and early commitment with the wasmtime community.

---

## Phase 10 — Hardening and 1.0 milestone

**Objective.** Bring the project to stable version 1.0. It is the phase of audits, hardening, final optimization, and public declaration of stability.

### 10.1 Entry criteria

- [ ] Phase 9 completed.
- [ ] DSL version 1.0 (stable grammar) ready for freeze.
- [ ] The technical committee is active and operational.
- [ ] Phase 4.7 stage C complete, and every multi-decision command introduced
  in Phases 5 to 9 has its interactive flow with flag parity.

### 10.2 Deliverables

- [ ] DSL version 1.0 (stable grammar, frozen).
- [ ] Test coverage ≥ 85% in all workspace crates.
- [ ] 72-hour fuzzing over the DSL parser without crashes.
- [ ] 72-hour fuzzing over the HTTP parser without crashes.
- [ ] External security audit of the Shield component, contracted with a specialized company (Trail of Bits, NCC Group or equivalent). Public report.
- [ ] Resolution of all critical and high findings of the audit.
- [ ] Load test: 500 K req/s sustained for 30 minutes with degradation ≤ 5%.
- [ ] Memory leak test: 24 hours of continuous load without detectable memory growth.
- [ ] Compilation verified on: Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Compilation to `wasm32-wasip1` (the target formerly named `wasm32-wasi`) to serve Anti-Gravital in edge functions.
- [ ] Official manual published: "The Anti-Gravital Book" in Spanish and English.
- [ ] Framework introduction course on YouTube (minimum six videos).
- [ ] Position in TechEmpower Framework Benchmarks: top 10 in Plaintext and JSON Serialization categories.

Stages:

1. **10-A, freeze.** DSL 1.0 grammar freeze; public API review of every crate;
   deprecations removed.
2. **10-B, verification.** Coverage, 72-hour fuzzing, load and leak tests.
3. **10-C, external audit and remediation.**
4. **10-D, platforms.** Four native targets and `wasm32-wasip1`.
5. **10-E, publication.** Book, course, benchmark submission, 1.0 release.

### 10.3 Exit criteria (version 1.0)

- [ ] At least three external projects using Anti-Gravital in production for at least 30 days without critical incidents.
- [ ] At least one internal Gravital Cloud service using Anti-Gravital in production for 30 days without critical incidents.
- [ ] Public announcement of version 1.0 with complete changelog.
- [ ] Commitment to strict semver from 1.0.
- [ ] Announcement of the LTS version calendar.
- [ ] Talk at at least one international conference (RustConf, EuroRust, RustNation or equivalent).
- [ ] At least 10 000 stars on the repository.
- [ ] The technical committee ratifies the promotion to version 1.0 unanimously.

### 10.4 Phase risks

The main risk is the pressure to release 1.0 before time. The mitigation is the project's strictest rule: the exit criteria are non-negotiable. If they are not met, 1.0 is not released. 0.9.5, 0.9.6 are released, until they are met.

---

## Beyond 1.0: future roadmaps

Once 1.0 is released, the project enters stable maintenance mode with minor releases every 3 months. The candidate topics for future versions include:

- Version 1.x: additional performance optimizations, support for additional protocols (HTTP/3 via QUIC).
- Version 2.x: refactoring of the plugin ABI if the WebAssembly community makes major changes. Support for new deployment targets.
- Swift generator for native iOS.
- Kotlin Multiplatform generator for native Android and cross-platform cases.
- More sophisticated multi-tenant support with instance federation.

This extended roadmap is not a commitment. It is documented to signal direction, but it will be reserved for specific RFCs when the time comes.

---

## Golden rules of the process

By way of closing, the five rules that govern this end-to-end process:

**First rule.** A phase is not considered concluded until all of its exit criteria boxes are checked. No exceptions.

**Second rule.** If a phase requires more time than estimated, it is extended. If the original scope is not attainable, it is reduced with a public RFC; the quality criteria are not relaxed.

**Third rule.** Every significant architectural decision requires an RFC. The iteration speed does not justify skipping the process.

**Fourth rule.** The project is released when it is ready, not when an external date demands it. Technical credibility is the project's most valuable asset.

**Fifth rule.** Every public promise (benchmark, feature, date) is documented with evidence. If there is no evidence, it is not promised.

These rules exist for a reason. Anti-Gravital sets out to compete with frameworks that have matured over decades. The only way to be taken seriously is to build with the same seriousness.

---

**End of the Roadmap document.**
Complementary document: *Technical Architecture and Implementation.*
Unified PDF version: *Anti-Gravital Blueprint v4.0 — Master Document.*

---

## Espanol

## Cómo leer este documento

Este documento define la secuencia de fases por las que debe pasar el proyecto Anti-Gravital desde su inicio hasta convertirse en una versión 1.0 estable, lista para mercado, con la promesa cumplida.

Cada fase contiene cuatro bloques:

1. **Criterios de entrada**: condiciones que deben cumplirse antes de que la fase pueda comenzar. Estos vienen de la fase anterior.
2. **Entregables**: artefactos concretos que la fase debe producir.
3. **Criterios de salida (puerta)**: condiciones que deben cumplirse antes de pasar a la siguiente fase. Funcionan como puertas bloqueantes: si no se cumplen, no se avanza. Esto es no negociable.
4. **Riesgos específicos de la fase y mitigaciones**.

Los entregables y criterios de salida se expresan como casillas marcables. Este documento se mantiene en el repositorio y se actualiza tachando lo cumplido. Sirve como tablero de mando público del proyecto.

La regla principal es: **una fase no se da por concluida hasta que todas sus casillas de criterio de salida están marcadas**. El proyecto puede pausarse temporalmente entre fases, pero no puede saltarse pasos por presión externa o por urgencia.

---

## Resumen de fases

El estado usa etiquetas explícitas basadas en evidencia, no un binario
"completa". Una fase no se declara completa mientras un criterio técnico
bloqueante siga abierto; los criterios de adopción externa (stars, contribuidores,
blog posts) se rastrean aparte y no bloquean el avance técnico. El trabajo de
gate pendiente por fase está en la tabla "Evidence-based roadmap" del README y en
el gate formal `PRE_FASE5_RELEASE_GATE.md`.

| Fase | Nombre                                     | Duración estimada | Estado    |
|------|--------------------------------------------|-------------------|-----------|
| 0    | Fundaciones y gobernanza                   | 1–2 meses         | En curso (entregables externos pendientes) |
| 1    | The Shield MVP                             | 2–3 meses         | Implementada y probada; gate de rendimiento/cobertura de referencia abierto |
| 2    | The Core MVP + roundtrip                   | 2 meses           | Implementada; benchmarks publicados por debajo de los objetivos 40K req/s y p99 |
| 3    | Anti-DSL alpha (v0.1–v0.4)                 | 3 meses           | Implementada; gates de fuzz 24h y benchmark generado-vs-manual abiertos (issue #70) |
| 4    | Módulos estándar (auth, data, realtime)    | 3 meses           | Implementada y probada; evidencia de escala y deuda documentada abiertas |
| 4.5  | `ag-mail` + `ag-domains`: comunicación y dominios | 1–2 meses  | Implementada; `ag-domains` activo, evidencia de release/doc abierta |
| 4.6  | Endurecimiento aditivo pre-Fase 5 (`ag-mail` MTA, `ag-workers`) | — | Implementada y verificada en CI; paridad live-DB y wiring de producer abiertos (issues #108/#109/#103/#112) |
| 4.7  | Experiencia interactiva de terminal (TUI de `ag`) | 1–2 meses  | Propuesta (`RFC-0018`); sin código aún |
| 4.8  | Fundaciones del proyecto: capas, entorno, instalación, operación remota | 2–3 meses | Propuesta (`RFC-0019`, `RFC-0020`, `RFC-0021`); sin código aún |
| 5    | `ag-cloud` — despliegue simplificado       | 2 meses           | Próxima |
| 5.5  | Funciones serverless                       | 2 meses           | Propuesta (`RFC-0023`); sin código aún |
| 6    | `ag-ai` y Knowledge Graph                  | 2 meses           | Pendiente |
| 7    | `ag-migrate` — importadores                | 2 meses           | Pendiente |
| 8    | `ag-mobile` — Flutter bridge               | 2 meses           | Pendiente |
| 9    | Sistema de plugins WASI                    | 2 meses           | Pendiente |
| 10   | Endurecimiento y hito 1.0                  | 3 meses           | Pendiente |

**Duración total estimada:** 25–30 meses desde el inicio.
**Hito de versión beta pública (0.5):** final de fase 5 (~15 meses).
**Hito de versión 1.0 estable:** final de fase 10 (~30 meses).

Las Fases 4.5, 4.6, 4.7 y 4.8 son aditivas: no mueven los dos hitos
anteriores, aunque las etapas A y B de la Fase 4.8 cambian aquello sobre lo
que construye la Fase 5 y son criterio de entrada de su etapa 5-A. La
Fase 4.7 corre en paralelo con el cierre del gate pre-Fase 5 y con la Fase 5;
sus etapas A y B son criterios de salida de la Fase 5 (sección 5.3) y su etapa
C es criterio de entrada de la Fase 10 (sección 10.1).

**Cómo se detallan las fases posteriores (enmienda del 2026-11-10).** Desde la
Fase 5 en adelante cada fase lleva, además de los cuatro bloques, un bloque de
**Etapas** que ordena sus entregables en pasos fusionables de forma
independiente y una lista de **RFC previas** (regla 28: comandos nuevos,
construcciones nuevas del DSL y dependencias nuevas requieren RFC aprobada
antes del código). Las etapas no añaden alcance: cada elemento de una etapa es
un entregable ya listado en la fase.

**Estado al cierre de la Fase 4.5 (2026-05-24), actualizado para el gate
pre-Fase 5 (2026-06-11).** Las fases 1 a 4.5 están técnicamente implementadas y
mergeadas a `main` (entregables de código, tests, fmt, clippy, audit y deny
cumplidos), pero ninguna se declara formalmente completa mientras el gate de
release pre-Fase 5 (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) esté ABIERTO: las
filas de fuzz 24 horas, benchmark estabilizado y deuda abierta siguen pendientes.
El detalle granular de cada casilla vive en `docs/roadmap/STATUS.md`, el tablero
de mando operativo. La Fase 0 permanece en curso por entregables externos
(Discord, landing, dominio). La Fase 5 (`ag-cloud`) es la próxima y no puede
iniciar hasta que cada fila bloqueante del gate pase.

**Nota sobre la Fase 4.5.** La Fase 4.5 es una fase **aditiva** introducida por
`ADR-0007` después de cerrar la Fase 4. No modifica el alcance ni los
entregables de la Fase 4 ya completada. No adelanta el hito v0.5 BETA, que
permanece al final de la Fase 5. La cuenta del ecosistema pasa de 15 a 17
crates con la incorporación de `ag-mail` y `ag-domains`, y crece de forma
aditiva a 20 con `ag-lsp` (tooling DSL de la Fase 3), `ag-edge`
(`ADR-0012`) y `ag-workers` (`ADR-0013`, el segundo estándar diferido).

> Las fases 0-4.5 están técnicamente implementadas; la completitud formal está
> sujeta al gate de release pre-Fase 5. La deuda técnica pendiente que debe
> cerrarse antes de la Fase 5 se rastrea como GitHub Issues (etiqueta
> `tech-debt`, regla 29 de CLAUDE.md); `docs/DEBT.md` es un registro histórico
> congelado.

---

## Fase 0 — Fundaciones y gobernanza

**Objetivo.** Crear las bases del proyecto: repositorio, licencia, documentación de gobernanza, CI, contribuyentes, comunicación con la comunidad. Sin código todavía. El producto de esta fase es un proyecto open source apto para recibir colaboradores.

### 0.1 Criterios de entrada

- [ ] Decisión final de comenzar Anti-Gravital como proyecto formal de Gravital Labs.
- [ ] Aprobación de licencia Apache 2.0 sin restricciones.
- [ ] Compromiso público de Ángel Nereira como mantenedor inicial.

### 0.2 Entregables

- [ ] Repositorio `github.com/anti-gravital/anti-gravital` creado y público.
- [ ] Archivo `LICENSE` con texto completo Apache 2.0.
- [ ] Archivo `README.md` bilingüe (español + inglés) con propuesta de valor.
- [ ] Archivo `CONTRIBUTING.md` con guía de contribución, convenciones de código, proceso de pull request.
- [ ] Archivo `CODE_OF_CONDUCT.md` adoptando Contributor Covenant 2.1.
- [ ] Archivo `SECURITY.md` con política de divulgación responsable y dirección `anti@gravitalcloud.com` (respaldo: `angelnereira@gravitalcloud.com`).
- [ ] Archivo `GOVERNANCE.md` describiendo modelo BDFL inicial y plan de transición.
- [ ] Configuración de CI con GitHub Actions: build en Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Plantillas de issue (bug report, feature request, RFC) y plantilla de pull request.
- [ ] Branding básico: logo, paleta de colores, tipografía. Aplicado al README.
- [ ] Discord oficial del proyecto con canales `#español`, `#english`, `#announcements`, `#help`.
- [ ] Cuenta del proyecto en X/Bluesky para anuncios.
- [ ] Dominio `antigravital.dev` registrado y apuntando a una landing page mínima.
- [ ] Email institucional `anti@gravitalcloud.com` operativo (correo raíz del proyecto).
- [ ] Calendario público de releases publicado.

### 0.3 Criterios de salida (puerta antes de Fase 1)

- [ ] El repositorio recibe su primer star externo no solicitado.
- [ ] Al menos cinco personas externas se han unido al Discord.
- [ ] La estructura de carpetas del monorepo está definida y commitada (aunque sin contenido funcional).
- [ ] El workspace Cargo está inicializado con los crates vacíos: `ag-core`, `ag-dsl`, `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- [ ] El CI construye exitosamente el workspace vacío en las cuatro plataformas objetivo.
- [ ] La landing page describe en un párrafo qué es el proyecto, qué no es, y dónde está en el roadmap.

### 0.4 Riesgos de la fase

El principal riesgo es la procrastinación por perfeccionismo. La fase 0 no produce código que se ejecute, lo que tienta a postergarla. La mitigación es un timebox estricto: 8 semanas máximo. Si al término no están todos los entregables, se concluye con lo que haya y se documenta lo pendiente como deuda técnica de fase 0 a resolver durante la fase 1.

---

## Fase 1 — The Shield MVP

**Estado: Implementación técnica completa.** Detalle de casillas en `docs/roadmap/STATUS.md`.

**Objetivo.** Implementar la capa Shield del núcleo: una pipeline de middleware Tower que valida, autentica básicamente, aplica rate limiting y entrega requests a un handler placeholder. Sin Core completo todavía. Sin DSL todavía. El producto es un binario que responde HTTP con seguridad básica y benchmark publicable.

### 1.1 Criterios de entrada

- [ ] Fase 0 completada con todos sus criterios de salida marcados.
- [ ] Al menos un contribuidor adicional al mantenedor principal está activo en el repositorio.

### 1.2 Entregables

- [ ] Crate `ag-core` con módulo `shield` operativo.
- [ ] Soporte de HTTP/1.1 y HTTP/2 vía Axum + Tokio.
- [ ] Terminación TLS 1.3 con rustls.
- [ ] Middleware de validación de payload básico (deserialización con serde y restricciones simples).
- [ ] Middleware de autenticación JWT con verificación Ed25519.
- [ ] Middleware de rate limiting con governor (token bucket por IP).
- [ ] Middleware CORS y CSRF con defaults seguros.
- [ ] Middleware de logging estructurado con `tracing`.
- [ ] Configuración mínima desde archivo TOML.
- [ ] Tests unitarios con cobertura ≥ 80% del crate `ag-core`.
- [ ] Tests de integración end-to-end del pipeline Shield.
- [ ] Benchmark Hello World ejecutable: `cargo bench` produce cifras reproducibles.
- [ ] Documentación API del crate generada con `cargo doc`, publicada en `docs.rs`.
- [ ] Capítulo del manual de usuario explicando cómo usar la Shield directamente como librería.

### 1.3 Criterios de salida (puerta antes de Fase 2)

- [ ] Benchmark Hello World alcanza ≥ 300 K req/s en hardware de referencia documentado.
- [ ] Latencia p99 del pipeline Shield ≤ 1 ms a 100 K req/s.
- [ ] Memoria del proceso idle ≤ 15 MB.
- [ ] Tiempo de arranque ≤ 100 ms.
- [ ] CI pasa en las cuatro plataformas objetivo.
- [ ] Análisis estático con `clippy` sin warnings.
- [ ] Análisis de dependencias con `cargo-audit` sin vulnerabilidades conocidas.
- [ ] Cero bloques `unsafe` no documentados.
- [ ] Al menos un blog post técnico publicado sobre la arquitectura de la Shield.
- [ ] Al menos diez stars en el repositorio.

### 1.4 Riesgos de la fase

El riesgo principal es underestimar la complejidad de TLS y rate limiting en producción. La mitigación es usar exclusivamente crates probados (rustls, governor) y no rodar implementaciones propias. El riesgo secundario es que las cifras de benchmark no alcancen el objetivo; la mitigación es publicar lo que se mide con honestidad y documentar el déficit.

---

## Fase 2 — The Core MVP y roundtrip completo

**Objetivo.** Completar el núcleo con la capa Core: router Axum, extractores tipados, sistema de errores, estado compartido. Implementar el roundtrip completo Request → Shield → Core → Handler → Respuesta. Conectar a PostgreSQL real para un CRUD mínimo. El producto es un binario que sirve una API real, aunque escrita manualmente sin DSL.

### 2.1 Criterios de entrada

- [ ] Fase 1 completada con todos sus criterios de salida marcados.
- [ ] El crate `ag-data` ha sido iniciado con sqlx como dependencia.

### 2.2 Entregables

- [ ] Crate `ag-core` con módulo `core` operativo.
- [ ] Router Axum integrado con la Shield.
- [ ] Extractores: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`, `Query<T>`.
- [ ] Sistema de errores `AgError` con conversión automática a respuesta HTTP.
- [ ] Sistema de respuestas: JSON, plaintext, streams.
- [ ] Crate `ag-data` con pool de conexiones PostgreSQL vía sqlx.
- [ ] Sistema de migraciones embebido con `sqlx::migrate!`.
- [ ] Example app `todo-api` en `examples/` con CRUD completo.
- [ ] Benchmark CRUD + DB ejecutable.
- [ ] Crate `ag-cli` con comandos `new` (crea proyecto desde template), `dev` (arranca servidor con hot reload vía `cargo-watch`), `build` (compila release).
- [ ] Tres templates: `rest`, `realtime`, `fullstack`.

### 2.3 Criterios de salida (puerta antes de Fase 3)

- [ ] Benchmark CRUD + PostgreSQL alcanza ≥ 40 K req/s en hardware de referencia.
- [ ] Latencia p99 del CRUD ≤ 5 ms.
- [ ] La app `todo-api` corre exitosamente con `ag new` + `ag dev`.
- [ ] La app `todo-api` se despliega como binario único (`FROM scratch` Docker).
- [ ] El binario release del `todo-api` ocupa ≤ 20 MB.
- [ ] Documentación: "Tu primera API con Anti-Gravital" publicada.
- [ ] Al menos 50 stars en el repositorio.
- [ ] Al menos tres contribuidores externos con PRs merged.

### 2.4 Riesgos de la fase

El riesgo principal es la deriva de scope: querer añadir features no estrictamente necesarias para el MVP del Core. La mitigación es una declaración explícita de scope en el ticket de la fase: el Core de esta fase no incluye autorización RBAC compleja, no incluye eventos, no incluye caché, no incluye observabilidad completa. Esos llegan en fases posteriores.

---

## Fase 3 — Anti-DSL alpha (versiones 0.1 a 0.4 del DSL)

**Objetivo.** Construir el compilador del DSL con un subconjunto entregable de la gramática. Esta fase entrega el primer codegen funcional: modelos, endpoints básicos, validaciones y relaciones. Sin auth declarativa todavía, sin eventos todavía. El producto es la primera versión del flujo "definir → generar → implementar".

### 3.1 Criterios de entrada

- [ ] Fase 2 completada con todos sus criterios de salida marcados.
- [ ] Crate `ag-dsl` iniciado.
- [ ] Decisión final sobre librerías base del compilador (logos para lexer, chumsky para parser, askama y quote para codegen). Documentada en RFC.

### 3.2 Entregables

- [ ] DSL versión 0.1: modelos básicos con anotaciones primitivas (`@primary`, `@unique`, `@auto`).
- [ ] DSL versión 0.2: endpoints (método, path, body, response).
- [ ] DSL versión 0.3: validaciones (`@min`, `@max`, `@email`, `@regex`, `@length`).
- [ ] DSL versión 0.4: relaciones entre modelos (`1:1`, `1:N`, `N:M`).
- [ ] Generador Rust: structs con serde, validators, query builders sqlx.
- [ ] Generador SQL: migraciones idempotentes.
- [ ] Generador TypeScript: tipos y cliente HTTP.
- [ ] Generador OpenAPI 3.1.
- [ ] Comando `ag generate` que lee `schema.ag` y produce todos los artefactos.
- [ ] Comando `ag schema lint` que reporta warnings de mejores prácticas.
- [ ] Comando `ag schema diff <ref>` que reporta cambios breaking vs no-breaking.
- [ ] Diagnostics legibles para errores comunes del DSL (modelo no encontrado, tipo desconocido, anotación inválida).
- [ ] Servidor LSP básico (`ag-lsp`) con autocompletado y diagnostics.
- [ ] Plugin VS Code publicado en el marketplace.
- [ ] Suite de tests del compilador con cobertura ≥ 85%.
- [ ] Fuzzing del parser con `cargo-fuzz`: 24 horas sin crashes.
- [ ] Documentación de referencia del DSL versión por versión.

### 3.3 Criterios de salida (puerta antes de Fase 4)

- [ ] Un proyecto completo se puede crear, definir en `schema.ag`, generar, y ejecutar usando solo la CLI.
- [ ] El example `ecommerce-api` se reescribe completamente con DSL y funciona.
- [ ] Los benchmarks se mantienen: CRUD generado por DSL no es más lento que CRUD escrito a mano.
- [ ] El plugin VS Code tiene ≥ 100 instalaciones.
- [ ] Al menos un colaborador externo ha contribuido al compilador.
- [ ] La documentación del DSL es completa y revisada por al menos dos personas.
- [ ] Al menos 200 stars en el repositorio.

### 3.4 Riesgos de la fase

El compilador del DSL es el componente técnicamente más complejo del proyecto. El riesgo principal es subestimar el esfuerzo y exceder el cronograma. La mitigación es la implementación incremental por subversiones: si la fase corre largo, la subversión 0.4 (relaciones) puede postergarse a la fase 4 sin bloquear el avance.

El riesgo secundario son los mensajes de error del compilador. Un compilador con mensajes incomprensibles arruina la experiencia. La mitigación es priorizar diagnostics legibles desde el día uno, con tests específicos que verifiquen que los mensajes son útiles.

---

## Fase 4 — Módulos estándar

**Objetivo.** Completar los módulos batteries-included: auth, realtime, cache, storage, observe. Cada uno como crate independiente, con tests, documentación y ejemplos.

### 4.1 Criterios de entrada

- [ ] Fase 3 completada.
- [ ] DSL versión 0.5 (auth y políticas) iniciada.

### 4.2 Entregables

- [ ] DSL versión 0.5: declaración de auth y políticas RBAC.
- [ ] DSL versión 0.6: declaración de eventos.
- [ ] Crate `ag-auth` completo: WebAuthn, JWT Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens con rotación.
- [ ] Crate `ag-realtime` completo: WebSocket binario, SSE fallback, NATS embebido para casos pequeños, cliente NATS externo para producción.
- [ ] Crate `ag-cache` completo: moka L1 + Redis L2 con fred, invalidación por evento.
- [ ] Crate `ag-storage` completo: adaptadores S3, MinIO, filesystem local. URLs firmadas. Procesamiento de imágenes.
- [ ] Crate `ag-observe` completo: tracing, OpenTelemetry exporter, métricas Prometheus, dashboards Grafana JSON incluidos.
- [ ] Integración de tokio-console en modo dev.
- [ ] Example `realtime-chat` en `examples/`.
- [ ] Example `ai-backend` en `examples/` que demuestra streaming SSE.
- [ ] Tests de integración cross-module.

### 4.3 Criterios de salida (puerta antes de Fase 5)

- [ ] Los cinco módulos publicados en crates.io con sus respectivos releases independientes.
- [ ] Cobertura de tests ≥ 80% en cada módulo.
- [ ] Documentación cada módulo: README, guía de uso, referencia de API.
- [ ] Performance: el módulo `ag-realtime` sostiene 50 K conexiones WebSocket en una instancia 2 vCPU sin degradación.
- [ ] Performance: el módulo `ag-cache` muestra ≥ 1 M ops/segundo en L1.
- [ ] Al menos cinco issues bug reports cerrados por la comunidad.
- [ ] Al menos 500 stars en el repositorio.

### 4.4 Riesgos de la fase

El riesgo principal es la fragmentación del esfuerzo entre cinco módulos paralelos. La mitigación es secuenciar la implementación: primero auth (bloquea muchos casos de uso), luego data avanzado, luego realtime, luego cache, luego storage, luego observe.

---

## Fase 4.5 — `ag-mail` + `ag-domains`: comunicación y dominios

**Objetivo.** Añadir capacidades operativas de comunicación transaccional, DNS,
TLS y dominios sin sobrecargar la Fase 4 ni retrasar los módulos estándar.
Prepara el terreno para que `ag-cloud` (Fase 5) despliegue aplicaciones con
dominio, certificado y correo transaccional usando una experiencia integrada.
La introducción de esta fase está oficializada en `ADR-0007`.

**Duración:** 1–2 meses.

### 4.5.1 Criterios de entrada

- [ ] Fase 4 completada con todos sus criterios de salida marcados.
- [ ] `ag-auth` expone hooks/eventos para verificación de correo, recuperación
  de contraseña y magic links.
- [ ] `ag-observe` registra métricas y trazas de jobs asíncronos.
- [ ] RFC aprobado para el alcance inicial de `ag-mail`.
- [ ] RFC aprobado para el alcance inicial de `ag-domains`.

### 4.5.2 Entregables

- [ ] Crate `ag-mail` (estándar diferido): sender SMTP outbound nativo
  (`lettre` + `rustls`) más trait `MailSender`; los proveedores externos se usan vía el relay SMTP nativo.
- [ ] Templates HTML/plaintext con `askama` tipados, validados en compile-time
  contra `schema.ag`.
- [ ] Declaración de correos en `schema.ag` (bloque `mail`).
- [ ] Integración `ag-auth` → `ag-mail` para verificación, recuperación y
  magic links, vía trait pequeño definido en `ag-auth`.
- [ ] Cola asíncrona con reintentos y backoff exponencial; backend en memoria
  por defecto, persistente vía `ag-data` opcional.
- [ ] Métricas hacia `ag-observe`: `ag_mail_sent_total`, `ag_mail_failed_total`,
  `ag_mail_retry_total`, histograma de latencia.
- [ ] Crate `ag-domains` (opcional infra): trait `DnsProvider` con adapter
  Cloudflare; modelo declarativo A/AAAA/CNAME/TXT/MX.
- [ ] Soporte ACME (Let's Encrypt) vía `instant-acme`: emisión y renovación
  automática, challenge DNS-01 preferido, HTTP-01 alternativo.
- [ ] Generación de SPF/DKIM/DMARC requeridos por `ag-mail` (cooperación
  `ag-mail` ↔ `ag-domains` sin ciclo de dependencia).
- [ ] Verificación de propagación contra múltiples resolvers públicos
  (`hickory-resolver`).
- [ ] DSL v0.7: bloques `mail`, `domain`, `dns`, `tls`; el compilador valida
  que el `from` referencia un `domain` declarado, que el template existe y
  que las variables del HTML coinciden con las `vars` tipadas.
- [ ] Actualización del LSP `ag-lsp` para los bloques nuevos.
- [ ] Comandos CLI: `ag domains check`, `ag domains sync`, `ag mail test`.
- [ ] Example `auth-mail-demo` en `examples/`: registro + verificación por
  correo + magic link.
- [ ] Documentación: "Configurar dominio, TLS y correo transaccional con
  Anti-Gravital".

### 4.5.3 Criterios de salida (puerta antes de Fase 5)

- [ ] `ag-mail` envía correo transaccional HTML y plaintext desde un proyecto
  Anti-Gravital vía sender nativo **y** vía al menos un adapter.
- [ ] `ag-auth` usa `ag-mail` para verificación de correo y recuperación de
  contraseña en el example `auth-mail-demo`.
- [ ] `ag-domains` crea y verifica registros DNS en al menos un proveedor
  real.
- [ ] `ag-domains` emite y renueva certificados TLS vía ACME en entorno de
  prueba (Let's Encrypt staging).
- [ ] `ag-domains` genera SPF/DKIM/DMARC requeridos por `ag-mail`.
- [ ] `ag domains check`, `ag domains sync` y `ag mail test` funcionan en CI
  reproducible.
- [ ] Cobertura de tests unitarios e integración ≥ 75 % en ambos crates.
- [ ] Cero dependencias circulares con `ag-core`, `ag-dsl`, `ag-auth` o
  `ag-cloud` (job de CI verde).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` y
  `cargo deny check` verdes.

### 4.5.4 Riesgos de la fase

El riesgo principal es **confundir `ag-mail` con un MTA completo**. La
mitigación es la restricción explícita del alcance v1 a outbound + adapters;
inbound, IMAP/POP, buzones persistentes y antispam quedan documentados como
fuera de alcance, no como "diferidos a v2".

El segundo riesgo es la **dependencia de upstreams jóvenes** (`instant-acme`,
`hickory-resolver`) en dominios donde los bugs se pagan caro: un certificado
que no renueva tumba el sitio. La mitigación es un trait `DnsProvider`
pequeño y versionado con tests de contrato, pinning explícito en el
workspace, y vigilancia activa de la evolución de los crates.

El tercer riesgo es **convertir Anti-Gravital en un panel de hosting** por
acumulación de capacidades. La mitigación es la regla de interoperabilidad
del proyecto: ambos crates son abstracciones con adapters, no reemplazos de
proveedores. La frontera está fijada en `ADR-0007` y no se mueve sin un nuevo
ADR.

### 4.5.5 Nota futura — Fase 4.6 MTA nativo (`ADR-0010`)

Esa frontera de `ADR-0007` ya se movió, mediante el nuevo ADR que ella misma
exigía. `ADR-0010` (2026-06-03) supersede la restricción v1 "NO es un MTA /
inbound nunca" y expande `ag-mail` a un MTA outbound nativo, por fases y
opt-in tras features de Cargo, conservando el patrón Native | Adapter y el
baseline implementado de la Fase 4.5. El trabajo es la Fase 4.6 por fases
(`RFC-0009` sección 5: etapas A-D) más endurecimiento continuo de
entregabilidad en la Fase 5+. La Fase 4.6-A (núcleo del MTA: resolución MX,
entrega ESMTP+STARTTLS, firma DKIM Ed25519 y clasificación de bounces) está
implementada tras la feature opt-in `mta`; las etapas 4.6-B..D siguen siendo
trabajo futuro. La Fase 4.5 sigue completa para su alcance de relay outbound
original. Los adapters de proveedor siguen siendo una ruta de producción
soportada hasta demostrar la entregabilidad nativa.

---

## Fase 4.6 — Endurecimiento aditivo pre-Fase 5: MTA nativo y `ag-workers`

**Estado: en curso.** Fase aditiva autorizada con el gate pre-Fase 5 abierto
(`ADR-0010`, `ADR-0013`). No hace ninguna afirmación de producción hasta que el
gate cierre. Estado vivo a nivel de casilla: `docs/roadmap/STATUS.md`.

**Objetivo.** Eliminar dos dependencias estructurales antes de que exista
`ag-cloud`: la necesidad de un relay de terceros para enviar correo y la
ausencia de un motor compartido de ejecución en segundo plano. Ambos son opt-in
tras features de Cargo y conservan el camino nativo por defecto (`ADR-0009`
regla 2).

La fase tiene dos pistas independientes. La etiqueta "4.6-D" se usó
históricamente para dos cosas distintas (la etapa de marketing de la pista de
correo en `RFC-0009` y toda la pista de workers en `RFC-0012`); este documento
las nombra de forma explícita para eliminar la ambigüedad.

### 4.6.1 Pista de correo — MTA outbound nativo en `ag-mail` (`RFC-0009`)

| Etapa | Alcance | Estado |
|-------|---------|--------|
| Correo A | Resolución MX, entrega directa ESMTP+STARTTLS, firma DKIM, clasificación de bounces (feature `mta`) | Implementada; camino de protocolo ejercitado en CI contra un sink en loopback |
| Correo B | Cola de dos niveles, lista de supresión, traffic shaping por destino, spool durable PostgreSQL (feature `queue-postgres`) | Implementada; el test con PostgreSQL vivo es `#[ignore]` |
| Correo C | API REST gestionada, API keys, idempotencia, webhooks firmados, multi-tenancy (feature `api`) | Parcial: solo firma de webhooks HMAC-SHA256; superficie REST abierta (issue #152) |
| Correo D | Audiencias, broadcasts, baja con un clic, componentes de plantilla | No iniciada (issue #152) |

### 4.6.2 Pista de workers — motor de ejecución `ag-workers` (`RFC-0012`)

| Etapa | Alcance | Estado |
|-------|---------|--------|
| S1–S4 | Modelo de jobs, runtime en memoria, backend PostgreSQL, scheduling | Implementadas |
| S5 | Declaraciones `worker` del DSL v0.8, CLI `ag workers` | Implementada |
| S6 | Encolado solo-productor desde `ag-edge` | Parcial; espera un consumidor concreto (issue #112) |
| S7 | Migración de la cola de `ag-mail` a `ag-workers` | Implementada; paridad verificada contra un PostgreSQL vivo |

### 4.6.3 Criterios de salida

- [x] Correo A y Correo B fusionadas tras features opt-in, con el sender de
  relay como valor por defecto.
- [x] Workers S1–S5 y S7 fusionadas; eliminada la cola genérica duplicada de
  correo.
- [ ] Gate de Correo A según `RFC-0009`: un mensaje entregado por el MTA nativo
  a un buzón externo real pasa SPF, DKIM y DMARC alineados. Evidencia manual
  registrada con fecha, commit y cabeceras.
- [ ] Correo C y Correo D entregadas o movidas formalmente fuera de la Fase 4.6
  mediante enmienda de RFC (issue #152). No bloquean la Fase 5.
- [ ] Workers S6 entregada o diferida formalmente con su consumidor nombrado
  (issue #112). No bloquea la Fase 5.

### 4.6.4 Riesgos de la fase

El riesgo principal es que una fase aditiva nunca cierre porque sus etapas
opcionales la mantienen abierta. La mitigación es la regla de 4.6.3: una etapa
sin consumidor se mueve fuera mediante enmienda de RFC en lugar de quedar "en
curso". El segundo riesgo es la entregabilidad: un MTA nativo vale lo que vale
la reputación de las direcciones desde las que envía, y eso no lo aporta
ningún código. La mitigación es que el sender de relay sigue siendo el valor
por defecto y la ruta de producción documentada hasta que el gate de Correo A
tenga evidencia real.

---

## Fase 4.7 — Experiencia interactiva de terminal (TUI de `ag`)

**Estado: propuesta (`RFC-0018`). No existe código.** Fase aditiva.

**Objetivo.** Hacer que crear y configurar un proyecto sea descubrible y guiado
desde la terminal, mediante una interfaz interactiva a pantalla completa en el
binario `ag`, sin debilitar el uso en scripts: toda acción interactiva tiene un
equivalente no interactivo exacto. El producto es la experiencia prometida por
la regla 32: un usuario nuevo llega a un proyecto en ejecución sin leer antes
el manual.

**Duración:** 1–2 meses. Corre en paralelo con el cierre del gate y la Fase 5.

### 4.7.1 Criterios de entrada

- [ ] `RFC-0018` aceptada, incluidas sus tres preguntas abiertas (idioma de la
  interfaz, posición en las puertas, comportamiento de `ag` sin argumentos).
- [ ] `ag-cli` dividido en módulos sin cambio de comportamiento (una petición
  tipada por comando y un único ejecutor), verificado por el test existente
  del árbol de comandos.

### 4.7.2 Entregables

Etapa A — fundación y creación de proyecto:

- [ ] Feature de Cargo `tui` en `ag-cli` (activa por defecto); el binario
  compila y todos los comandos funcionan con `--no-default-features`.
- [ ] Guarda de terminal que restaura la terminal al salir, ante error, ante
  `Ctrl-C` y ante panic.
- [ ] Asistente de `ag new`: nombre con validación en vivo, template con
  descripción, pantalla de revisión, pantalla de resultado. `NAME` pasa a ser
  opcional.
- [ ] Flag global `--no-interactive` y variable `AG_NO_TUI`; modo no
  interactivo automático cuando no hay terminal, cuando `CI` está definida o
  cuando `TERM=dumb`.
- [ ] Comando no interactivo equivalente impreso al final de cada flujo.
- [ ] Tema monocromo que respeta `NO_COLOR`; solo glifos ASCII y de trazado de
  cajas.
- [ ] Modo degradado de prompts por línea por debajo de 80x24.
- [ ] Una única librería de interacción en `ag-cli` (se retira la librería de
  prompts anterior).

Etapa B — configuración del proyecto:

- [ ] `ag config get`, `ag config set`, `ag config validate` (no interactivos,
  utilizables en CI).
- [ ] Formulario `ag config` generado a partir de los tipos de configuración de
  `ag-core`, con validación en línea, valores por defecto y descripciones.
- [ ] Guardado atómico que preserva el formato, con diff en la pantalla de
  revisión.
- [ ] Los campos secretos solo aceptan una ruta de archivo o el nombre de una
  variable de entorno.
- [ ] `ag new --with MODULE` para el primer módulo estándar con composición de
  scaffold probada, y después el paso correspondiente en el asistente.

Etapa C — pantalla de inicio:

- [ ] `ag` sin argumentos en una terminal abre un lanzador sensible al contexto
  (fuera de un proyecto / dentro de un proyecto). Fuera de una terminal
  imprime la ayuda como hasta ahora.
- [ ] Cada entrada del lanzador ejecuta un comando existente y lo muestra.

Transversales:

- [ ] Tests unitarios de transición de estado para cada pantalla.
- [ ] Tests de snapshot de render a 80x24 y 120x40, en color y monocromo.
- [ ] Tests de paridad: la petición construida por un asistente es igual a la
  petición parseada desde su comando equivalente impreso.
- [ ] Tests de regresión no interactiva: ningún comando se bloquea ni emite
  secuencias de escape con stdin cerrado.
- [ ] Diagrama del flujo de interacción en `docs/diagrams/`.
- [ ] `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`, el
  `README.md` raíz y el capítulo del manual de la primera API actualizados en
  la misma pull request de cada etapa.

### 4.7.3 Criterios de salida

- [ ] Una persona que nunca ha usado Anti-Gravital crea y arranca un proyecto
  usando solo `ag new` en una terminal, sin flags y sin documentación.
  Verificado con al menos tres personas externas al proyecto; las notas de las
  sesiones se publican.
- [ ] Todos los comandos pasan la suite de regresión no interactiva en las
  cuatro plataformas de CI.
- [ ] El asistente funciona sobre SSH, dentro de `tmux` y en la consola de
  Windows.
- [ ] El aumento de tamaño del binario release de `ag` causado por la feature
  `tui` está medido y publicado.
- [ ] Ningún flujo interactivo tiene una capacidad ausente en los flags (suite
  de paridad en verde).

### 4.7.4 Riesgos de la fase

El riesgo principal son **dos productos divergentes**: una herramienta
interactiva y una herramienta de scripting que poco a poco dejan de comportarse
igual. La mitigación es arquitectónica, no de proceso: las pantallas producen
la misma petición tipada que el parser de flags y un único ejecutor realiza el
trabajo, con tests de paridad en CI.

El segundo riesgo es el **scope creep hacia un IDE de terminal** (visores de
logs, editores, dashboards). La mitigación es la regla fijada en `RFC-0018`:
una pantalla es un front-end de un comando existente; cualquier otra cosa
requiere una RFC nueva.

El tercer riesgo es la **diversidad de terminales**. La mitigación es un
conjunto conservador de glifos, un tema monocromo, CI en cuatro plataformas y
el modo degradado por línea.

---

## Fase 4.8 — Fundaciones del proyecto: capas, entorno, instalación y operación remota

**Estado: propuesta (`RFC-0019`, `RFC-0020`, `RFC-0021`). No existe código.**
Fase aditiva. Las reglas de mantenibilidad que la acompañan se proponen aparte
en `RFC-0022` (gobernanza, no una fase).

**Objetivo.** Dar a cada proyecto generado una estructura definida y forzada;
hacer que la configuración y los secretos sean declarados, validados y seguros
por defecto; y hacer que la herramienta se instale en un servidor con un único
comando verificado y se opere desde cualquier otro dispositivo mediante la
interfaz de terminal de la Fase 4.7, tanto por personas como por agentes
automatizados. El despliegue (Fase 5) construye sobre las tres cosas.

**Declaración de alcance fijada por esta fase (`RFC-0019`).** Anti-Gravital es
una herramienta de backend. Se integra con Docker, Kubernetes y Terraform y no
los reemplaza; ofrece gRPC como transporte opcional y clientes generados para
consumidores Go, WebAssembly y Flutter; no renderiza interfaces de usuario.

**Duración:** 2–3 meses.

### 4.8.1 Criterios de entrada

- [ ] `RFC-0019`, `RFC-0020` y `RFC-0021` aceptadas con sus preguntas abiertas
  resueltas (layout por defecto, destino de `ag-ui`, gRPC y Go en el alcance,
  almacenamiento de secretos por defecto, transporte remoto, alcance de
  `ag setup`, custodia de la llave de release).
- [ ] Etapa A de la Fase 4.7 completa (la fundación interactiva que usan estos
  flujos).

### 4.8.2 Entregables

Etapa A — estructura del proyecto (`RFC-0019`):

- [ ] Manifiesto de proyecto `ag.toml` leído por todos los comandos.
- [ ] Layout de workspace por capas (dominio, aplicación, infraestructura,
  interfaz, raíz de composición) para los templates `fullstack` y `realtime`,
  con la regla de dependencias forzada por Cargo.
- [ ] Layout compacto con los mismos cuatro nombres de capa para el template
  `rest`.
- [ ] Capa de infraestructura dividida por responsabilidad, con el adaptador
  de base de datos como módulo acotado propio que posee las migraciones.
- [ ] `ag generate` escribe cada artefacto en la capa que le corresponde;
  `ag generate --check` para CI.
- [ ] Capítulo de arquitectura sobre la estructura del proyecto y manual de la
  primera API actualizado.

Etapa B — entorno y secretos (`RFC-0020`):

- [ ] Declaración `[env]` en `ag.toml`; cargador tipado con validación
  fail-fast que reporta todos los problemas a la vez.
- [ ] Tipo `Secret<T>`: formato redactado, borrado al liberar, acceso
  explícito.
- [ ] Archivo de secretos cifrado por entorno con gestión de destinatarios.
- [ ] `ag env list/check/set/unset/recipients/keygen/export`.
- [ ] Los módulos reciben configuración tipada en lugar de leer directamente
  el entorno del proceso (un crate por pull request).
- [ ] Formulario interactivo de entorno con campos secretos enmascarados.

Etapa C — instalación (`RFC-0021`):

- [ ] Workflow de release que publica binarios estáticos para los cuatro
  targets, un manifiesto de checksums y su firma separada.
- [ ] Instalador que detecta sistema y arquitectura, verifica firma y
  checksum, y no necesita compilador. Cierra el issue #141.
- [ ] `ag setup --check` y `ag setup` (detectar, planificar, confirmar,
  aplicar, verificar) usando solo el gestor de paquetes del sistema.

Etapa D — consola remota (`RFC-0021`):

- [ ] `ag servers add/list/remove` (inventario sin secretos).
- [ ] `ag connect NOMBRE` que abre la interfaz del lado servidor sobre SSH, y
  `ag connect NOMBRE -- COMANDO` para un único comando remoto.

Etapa E — interfaz de máquina (`RFC-0021`):

- [ ] `--output json` en todos los comandos, con esquemas versionados.
- [ ] Tabla documentada de códigos de salida.
- [ ] `ag describe` que emite el árbol de comandos, tipos de argumentos y
  esquemas de salida.
- [ ] `--plan` / `--apply PLAN_ID` en todo comando que cambie estado fuera del
  directorio de trabajo; log de auditoría local de los comandos que mutan.

### 4.8.3 Criterios de salida

- [ ] Un proyecto creado desde cada template compila y corre en CI a partir de
  un `ag new` limpio, y un crate de dominio que importa un crate de base de
  datos o de HTTP no compila.
- [ ] Arrancar una aplicación con tres variables inválidas reporta las tres y
  no abre ningún listener; ningún test logra que el valor de un secreto
  aparezca en logs ni en la salida de un comando.
- [ ] En un servidor limpio de cada distribución soportada, el instalador
  verificado seguido de `ag setup` produce una instalación funcional sin
  ningún paso manual, y un binario o manifiesto alterado es rechazado.
- [ ] Desde una segunda máquina, `ag connect` alcanza la interfaz del
  servidor; la misma sesión funciona desde un cliente SSH de teléfono.
- [ ] Un cliente automatizado completa un ciclo crear-configurar-validar
  usando solo `ag describe` y la salida JSON, y no puede cambiar el estado del
  servidor sin un identificador de plan explícito.
- [ ] Etapas A y B completas antes de iniciar la etapa 5-A de la Fase 5;
  etapas C a E completas antes de la liberación de la 0.5.

### 4.8.4 Riesgos de la fase

El riesgo principal es el **tamaño**: esta fase toca el scaffolder, la
configuración de todos los módulos y el proceso de release. La mitigación es
que las cinco etapas son independientes y cada una se entrega como varias pull
requests pequeñas; ninguna requiere a las otras salvo B sobre A.

El segundo riesgo son las **regresiones de seguridad introducidas en nombre de
la comodidad**, sobre todo en la instalación y el acceso remoto. La mitigación
está en el diseño: verificación de firma antes de escribir nada, solo gestor
de paquetes, SSH como único transporte sin daemon ni puerto nuevos, y plan
antes de aplicar para todo cambio de estado.

El tercer riesgo es **romper a usuarios existentes** cuando cambien las rutas
del generador y los constructores de los módulos. La mitigación es que no se
ha publicado ninguna versión; los cambios entran antes de la 0.5 y se
registran como cambios incompatibles pre-release.

---

## Fase 5 — `ag-cloud` despliegue simplificado

**Objetivo.** Construir el subsistema de despliegue al estilo Railway/Fly.io. Soporte para los cuatro targets: docker-compose, fly, railway, k8s. Este es el hito de **versión beta pública (0.5)**.

### 5.1 Criterios de entrada

- [ ] Fases 4, 4.5 y 4.6 implementadas y el gate de release pre-Fase 5
  (`docs/audits/PRE_FASE5_RELEASE_GATE.md`) cerrado: todas las filas
  bloqueantes en `pass`, incluida la corrida de fuzz de 24 horas (issue #156)
  y la evidencia final de benchmarks (issue #157).
- [ ] Decisión RFC sobre los targets de despliegue soportados en la 1.0.
- [ ] Etapas A y B de la Fase 4.8 completas (layout del proyecto y entorno
  declarado), porque los artefactos de despliegue se generan a partir de
  ellas.

RFC previas (regla 28), cada una aprobada antes de iniciar su etapa:

- [ ] RFC de la especificación de `deploy.ag` (superficie nueva del DSL).
- [ ] RFC de los comandos `ag deploy` y `ag rollback`, incluido el modelo de
  secretos y qué se almacena localmente.

### 5.2 Entregables

- [ ] Crate `ag-cloud` con módulos para cada target.
- [ ] Especificación del archivo `deploy.ag`.
- [ ] Generador de Dockerfile multi-stage optimizado para imagen mínima.
- [ ] Target docker-compose: generación completa de stack con Caddy como reverse proxy y TLS automático.
- [ ] Target fly: integración con flyctl.
- [ ] Target railway: integración con su API.
- [ ] Target k8s: generación de manifests estándar.
- [ ] Integración con Terraform: declaraciones de variables y outputs
  generados para los recursos que necesita un proyecto; el Terraform del
  usuario es dueño de los recursos (`RFC-0019` sección 4.6).
- [ ] Comando `ag deploy`.
- [ ] Comando `ag rollback`.
- [ ] Pipeline de migraciones de base de datos integrado al despliegue.
- [ ] Flujo interactivo de despliegue según `RFC-0018`: selección de target,
  pantalla de revisión que lista cada recurso que se creará o cambiará,
  confirmación explícita; `ag rollback` muestra qué va a revertir antes de
  actuar. La paridad con flags es obligatoria.
- [ ] Documentación: "Desde cero a producción en 15 minutos" con cada target.

Etapas:

1. **5-A, especificación.** Gramática y validación de `deploy.ag`; estructura
   del crate `ag-cloud` con un módulo desacoplado por target tras su propia
   feature de Cargo; arnés de tests de contrato compartido por todos los
   targets.
2. **5-B, imagen.** Generador de Dockerfile multi-stage; build de imagen
   reproducible; tamaño de imagen registrado.
3. **5-C, primero el target auto-alojado.** Target docker-compose con reverse
   proxy y TLS automático; pipeline de migraciones; `ag deploy` y
   `ag rollback` para este target; integración opcional con `ag-domains`. Es
   el camino nativo y auto-alojable exigido por la regla 5 de `ADR-0009` y se
   entrega antes que cualquier target alojado.
4. **5-D, target k8s.** Manifiestos estándar generados desde la misma
   especificación.
5. **5-E, targets alojados.** fly y railway, cada uno como módulo adaptador.
6. **5-F, flujo interactivo y documentación.** Asistente de despliegue, guías
   por target, liberación de la 0.5.

### 5.3 Criterios de salida (puerta antes de Fase 6 y versión 0.5)

- [ ] El example `todo-api` se despliega exitosamente a Fly.io con `ag deploy`.
- [ ] El example `ecommerce-api` se despliega exitosamente con docker-compose a un VPS y se accede vía dominio con TLS.
- [ ] El example `realtime-chat` se despliega exitosamente a Railway.
- [ ] Etapas A y B de la Fase 4.7 completas (creación y configuración
  interactiva de proyectos).
- [ ] Un despliegue fallido deja sirviendo la versión anterior y `ag rollback`
  la restaura; demostrado en el target docker-compose.
- [ ] Versión 0.5 (beta pública) liberada en GitHub Releases.
- [ ] Anuncio público en Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] Al menos diez proyectos externos reportan que han desplegado Anti-Gravital en producción o staging.
- [ ] Al menos 1 500 stars en el repositorio.

### 5.4 Riesgos de la fase

El riesgo principal es la dependencia de APIs externas (Fly, Railway) que pueden cambiar. La mitigación es estructurar cada target como un módulo desacoplado con tests de contrato.

El segundo riesgo es que el despliegue es el primer lugar donde la herramienta
maneja secretos y ejecuta acciones irreversibles sobre infraestructura que el
usuario paga. La mitigación es el orden de las etapas (primero auto-alojado),
la pantalla de revisión obligatoria antes de cualquier cambio y un modelo de
secretos fijado por RFC antes del código.

---

## Fase 5.5 — Funciones serverless

**Estado: propuesta (`RFC-0023`). No existe código.** Fase aditiva, posterior
a la beta pública.

**Objetivo.** Dar al código de aplicación una unidad más pequeña que el
servicio: una función que se despliega de forma independiente, se arranca bajo
demanda, se aísla en un sandbox WebAssembly con capacidades explícitas y
límites estrictos, y no cuesta nada mientras está inactiva. Las funciones
corren de forma nativa en los servidores del propio usuario; las plataformas
externas de funciones y los contenedores son adaptadores de empaquetado.

**Duración:** 2 meses.

### 5.5.1 Criterios de entrada

- [ ] `RFC-0023` aceptada con sus preguntas abiertas resueltas (runtime nativo
  o solo adaptadores, posición en la hoja de ruta, segundo lenguaje).
- [ ] Versión 0.5 (beta pública) liberada.
- [ ] Etapas B y E de la Fase 4.8 completas (secretos declarados; plan y
  aplicación).

### 5.5.2 Entregables

- [ ] Núcleo del host en `ag-wasm-host`: motor, límites de memoria, fuel y
  timeout por invocación, caché de módulos compilados. Es el trabajo listado
  como etapa 9-A; la Fase 9 construye los plugins encima.
- [ ] Interfaz WIT versionada para funciones y scaffold de función en Rust.
- [ ] Declaraciones `function` en el DSL con validación en compilación de
  triggers, límites y capacidades.
- [ ] Triggers: HTTP a través de la pipeline del Shield; schedule y cola a
  través de `ag-workers`; eventos.
- [ ] Capacidades, denegadas por defecto: datos, caché, almacenamiento,
  correo, HTTP saliente a hosts listados, variables de entorno declaradas,
  encolado de jobs.
- [ ] Modo embebido (dentro del proceso del servicio) y modo dedicado.
- [ ] `ag functions new/list/build/invoke/deploy/rollback/logs`, con salida
  JSON y flujos interactivos.
- [ ] Métricas y trazas por función a través de `ag-observe`.
- [ ] Empaquetado como contenedor y un adaptador de plataforma externa en
  `ag-cloud`.
- [ ] Guía para un segundo lenguaje.
- [ ] Documentación: cuándo usar una función, un worker o el servicio.

Etapas:

1. **5.5-A, núcleo del host e invocación local.**
2. **5.5-B, declaraciones y trigger HTTP.**
3. **5.5-C, triggers asíncronos sobre `ag-workers`.**
4. **5.5-D, capacidades**, una a la vez, cada una con su test de denegación.
5. **5.5-E, despliegue, rollback y observabilidad.**
6. **5.5-F, adaptadores de empaquetado y segundo lenguaje.**

### 5.5.3 Criterios de salida

- [ ] Una función sin una capacidad no puede leer un archivo, abrir una
  conexión ni leer una variable de entorno; cada denegación está cubierta por
  un test.
- [ ] Una función que excede memoria, fuel o timeout se termina sin afectar a
  otras invocaciones ni a la API, demostrado bajo carga.
- [ ] Un proyecto de ejemplo ejecuta una función por cada tipo de trigger.
- [ ] Desplegar o revertir una función no reinicia el servicio.
- [ ] Los tiempos de invocación en frío y en caliente están medidos y
  publicados con la metodología de la regla 17. No se afirma ninguna cifra
  antes de eso.

### 5.5.4 Riesgos de la fase

El riesgo principal es **un fallo del sandbox**, porque las funciones ejecutan
código con menos derechos que el servicio dentro del mismo proceso. La
mitigación son las capacidades denegadas por defecto, un test de denegación
por capacidad, un motor fijado y cubierto por la auditoría semanal, y la
inclusión del host de funciones en la auditoría externa de la Fase 10.

El segundo riesgo es la **expectativa de paridad con las grandes plataformas
de funciones**. La mitigación es declarar los límites en la documentación (sin
conexiones persistentes, sin trabajo de larga duración, sin estado entre
invocaciones) y ofrecer adaptadores para quien necesite una plataforma así.

---

## Fase 6 — `ag-ai` y Knowledge Graph

**Objetivo.** Construir el módulo de IA con el knowledge graph y las capacidades asistidas.

### 6.1 Criterios de entrada

- [ ] Versión 0.5 (beta pública) liberada.
- [ ] Retroalimentación de los primeros usuarios incorporada en backlog.

RFC previas (regla 28):

- [ ] RFC del esquema del knowledge graph (`.ag/knowledge-graph.json`) y su
  versionado.
- [ ] RFC de los comandos `ag ai` y del trait de proveedor, incluido qué datos
  del proyecto pueden salir de la máquina y el modelo de consentimiento.

### 6.2 Entregables

- [ ] Generador del knowledge graph desde el AST del DSL.
- [ ] Persistencia del graph en `.ag/knowledge-graph.json`.
- [ ] Generador de documentación arquitectónica Markdown desde el graph.
- [ ] Generador de diagramas C4 (Context, Container, Component) en Mermaid.
- [ ] Dashboard interactivo del graph en el dev server (`ag dev`).
- [ ] Comando `ag ai suggest-schema` con integración a proveedor configurable.
- [ ] Comando `ag ai review-migration`.
- [ ] Comando `ag ai analyze-architecture`.
- [ ] Soporte para proveedores: Anthropic Claude, OpenAI, Ollama local, vLLM local.
- [ ] Modo offline donde las funciones AI están deshabilitadas pero el framework funciona.
- [ ] Documentación: "Anti-Gravital + agentes IA: el flujo schema-first" con ejemplos completos.

Etapas:

1. **6-A, grafo (sin IA).** Generador desde el AST del DSL, persistencia,
   regeneración en `ag generate`, salida determinista.
2. **6-B, documentos derivados.** Documentación arquitectónica en Markdown y
   diagramas C4 producidos desde el grafo.
3. **6-C, dashboard.** Vista interactiva del grafo en el dev server.
4. **6-D, abstracción de proveedor.** Trait de proveedor, un proveedor local y
   un adaptador remoto, modo offline como valor por defecto.
5. **6-E, comandos asistidos.** `suggest-schema`, `review-migration`,
   `analyze-architecture`; cada uno produce una propuesta revisable y nunca
   escribe archivos sin confirmación.

Las etapas 6-A a 6-C son útiles sin ningún proveedor de IA y se entregan
primero.

### 6.3 Criterios de salida (puerta antes de Fase 7)

- [ ] El knowledge graph se regenera correctamente con cada `ag generate`.
- [ ] La documentación arquitectónica generada es legible y útil (revisada por tres personas externas al equipo).
- [ ] Al menos una organización usuaria reporta que ha integrado `ag ai` en su flujo de trabajo.
- [ ] Al menos 2 500 stars en el repositorio.

### 6.4 Riesgos de la fase

El riesgo principal es la dependencia de proveedores externos de IA. La mitigación es la abstracción del proveedor y el modo offline.

El segundo riesgo es la exposición de datos: schema y código enviados a un
proveedor remoto. La mitigación es que los proveedores remotos son adaptadores
opt-in tras features de Cargo (`ADR-0009` regla 2, `ADR-0011`), existe un
proveedor local y cada comando indica qué va a enviar antes de enviarlo.

---

## Fase 7 — `ag-migrate` importadores

**Objetivo.** Construir los importadores de migración desde frameworks legacy. Es probablemente la fase con mayor impacto en adopción real.

### 7.1 Criterios de entrada

- [ ] Fase 6 completada.
- [ ] Investigación de muestras reales: al menos diez schemas/proyectos de cada framework objetivo recolectados como corpus de testing.

### 7.2 Entregables

- [ ] Crate `ag-migrate` con seis importadores:
  - [ ] Importador OpenAPI 3.0 y 3.1.
  - [ ] Importador Prisma.
  - [ ] Importador Django.
  - [ ] Importador FastAPI.
  - [ ] Importador Sequelize.
  - [ ] Importador GraphQL SDL.
- [ ] Comando `ag migrate from <framework> <ruta>`.
- [ ] Guías oficiales de migración por framework con ejemplos completos.
- [ ] Estudio de caso documentado: migración real de una aplicación FastAPI mediana.
- [ ] Reporte de importación en cada ejecución: qué se tradujo, qué se aproximó
  y qué se omitió, con ubicaciones en el origen. Revisión interactiva del
  reporte según `RFC-0018` antes de escribir ningún archivo.

RFC previas (regla 28):

- [ ] RFC del comando `ag migrate from` y del formato del reporte de
  importación.

Etapas:

1. **7-A, formatos de contrato.** OpenAPI 3.0/3.1 y GraphQL SDL
   (especificaciones legibles por máquina; sin análisis de código fuente).
2. **7-B, formatos de schema.** Prisma.
3. **7-C, fuentes de framework.** Django, FastAPI, Sequelize, en el orden que
   fije el tamaño del corpus recolectado.
4. **7-D, guías y estudio de caso.**

### 7.3 Criterios de salida (puerta antes de Fase 8)

- [ ] Cada importador tiene cobertura de tests ≥ 80% sobre el corpus de proyectos reales.
- [ ] La guía de migración FastAPI ha sido validada por al menos un equipo externo que migró su aplicación.
- [ ] Al menos 3 500 stars en el repositorio.

### 7.4 Riesgos de la fase

Los importadores cubren la traducción del contrato, no la lógica de negocio. El riesgo es generar expectativas exageradas. La mitigación es documentación honesta sobre lo que se importa y lo que no.

---

## Fase 8 — `ag-mobile` Flutter bridge

**Objetivo.** Construir la integración con Flutter como objetivo prioritario móvil. Generación de SDK Dart completo, auth nativo, realtime.

### 8.1 Criterios de entrada

- [ ] Fase 7 completada.
- [ ] Al menos un colaborador con experiencia significativa en Flutter se ha unido al proyecto.

### 8.2 Entregables

- [ ] Crate `ag-mobile` con generador Dart.
- [ ] Paquete pub `anti_gravital` publicado en pub.dev:
  - [ ] Tipos generados con freezed.
  - [ ] Cliente HTTP con dio + interceptores.
  - [ ] Cliente WebSocket.
  - [ ] Cliente SSE.
  - [ ] Mocks para tests.
- [ ] Widgets de autenticación: registro y login con WebAuthn nativo (Android Credential Manager, iOS Passkeys), OAuth2.
- [ ] Example `flutter-fullstack` en `examples/`: app Flutter completa con backend Anti-Gravital.
- [ ] Documentación: guía de usuario Flutter.

Etapas:

1. **8-A, tipos y HTTP.** Generador Dart, tipos generados, cliente HTTP,
   mocks; el caso CRUD de extremo a extremo.
2. **8-B, realtime.** Clientes WebSocket y SSE.
3. **8-C, autenticación.** Widgets de WebAuthn nativo y OAuth2.
4. **8-D, ejemplo, guía y publicación en pub.dev.**

### 8.3 Criterios de salida (puerta antes de Fase 9)

- [ ] El paquete `anti_gravital` en pub.dev tiene al menos 50 likes.
- [ ] El example `flutter-fullstack` corre en Android, iOS y web.
- [ ] Al menos una aplicación Flutter externa usa Anti-Gravital en staging o producción.
- [ ] Al menos 4 500 stars en el repositorio.

### 8.4 Riesgos de la fase

El riesgo principal es que el cambio de contexto Rust → Dart tenga fricciones imprevistas. La mitigación es comenzar con el caso más simple (CRUD) y construir incrementalmente.

---

## Fase 9 — Sistema de plugins WASI

**Objetivo.** Construir el sistema de plugins WASI con wasmtime, definir la ABI estable, publicar los plugins oficiales, y arrancar el registro público.

### 9.1 Criterios de entrada

- [ ] Fase 8 completada.
- [ ] Decisión RFC sobre el alcance de la ABI 1.0 de plugins. Aprobada por el comité técnico (formado en fase 4 o anterior).

### 9.2 Entregables

- [ ] Crate `ag-wasm-host` operativo sobre wasmtime.
- [ ] Definición de interfaces WIT (WebAssembly Interface Types) para el host.
- [ ] Especificación de `plugin.toml`.
- [ ] Implementación del ciclo de vida de plugin (descubrimiento, validación, carga, activación, descarga).
- [ ] Sandbox con límites de memoria, fuel y timeout.
- [ ] Plugins oficiales: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`.
- [ ] Comando `ag plugin add/remove/list`.
- [ ] Registro público en `plugins.antigravital.dev`.
- [ ] Guía: "Cómo escribir un plugin para Anti-Gravital" con ejemplos en Rust, Go (TinyGo) y AssemblyScript.
- [ ] La instalación de un plugin muestra las capacidades que solicita y pide
  confirmación (interactiva según `RFC-0018`, con paridad de flags).

RFC previas (regla 28), además de la RFC de la ABI de 9.1:

- [ ] RFC de los comandos `ag plugin` y del modelo de confianza del registry
  (firma, verificación de integridad, revocación).

Etapas:

1. **9-A, host.** Interfaces WIT, `plugin.toml`, ciclo de vida, límites del
   sandbox.
2. **9-B, CLI.** `ag plugin add/remove/list` contra plugins locales.
3. **9-C, plugins oficiales.**
4. **9-D, registry.** Registry público con verificación de integridad.
5. **9-E, guía y benchmark de overhead.**

### 9.3 Criterios de salida (puerta antes de Fase 10)

- [ ] El registro publica al menos los seis plugins oficiales.
- [ ] Al menos tres plugins externos de terceros publicados en el registro.
- [ ] El benchmark muestra overhead de plugin ≤ 1% sobre handler nativo equivalente.
- [ ] Al menos 6 000 stars en el repositorio.

### 9.4 Riesgos de la fase

El riesgo principal es la complejidad del component model de WebAssembly, que sigue evolucionando. La mitigación es pinneo conservador de la versión soportada y compromiso temprano con la comunidad wasmtime.

---

## Fase 10 — Endurecimiento y hito 1.0

**Objetivo.** Llevar el proyecto a versión 1.0 estable. Es la fase de auditorías, hardening, optimización final, y declaración pública de estabilidad.

### 10.1 Criterios de entrada

- [ ] Fase 9 completada.
- [ ] DSL versión 1.0 (gramática estable) lista para freeze.
- [ ] El comité técnico está activo y operativo.
- [ ] Etapa C de la Fase 4.7 completa, y cada comando de múltiples decisiones
  introducido en las Fases 5 a 9 tiene su flujo interactivo con paridad de
  flags.

### 10.2 Entregables

- [ ] DSL versión 1.0 (gramática estable, congelada).
- [ ] Cobertura de tests ≥ 85% en todos los crates del workspace.
- [ ] Fuzzing de 72 horas sobre el parser DSL sin crashes.
- [ ] Fuzzing de 72 horas sobre el parser HTTP sin crashes.
- [ ] Auditoría externa de seguridad del componente Shield, contratada con empresa especializada (Trail of Bits, NCC Group o equivalente). Reporte público.
- [ ] Resolución de todos los findings críticos y altos de la auditoría.
- [ ] Load test: 500 K req/s sostenidos por 30 minutos con degradación ≤ 5%.
- [ ] Memory leak test: 24 horas de carga continua sin crecimiento de memoria detectable.
- [ ] Compilación verificada en: Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Compilación a `wasm32-wasip1` (el target antes llamado `wasm32-wasi`) para servir Anti-Gravital en edge functions.
- [ ] Manual oficial publicado: "The Anti-Gravital Book" en español e inglés.
- [ ] Curso de introducción al framework en YouTube (mínimo seis videos).
- [ ] Posición en TechEmpower Framework Benchmarks: top 10 en categorías Plaintext y JSON Serialization.

Etapas:

1. **10-A, freeze.** Congelación de la gramática DSL 1.0; revisión de la API
   pública de cada crate; eliminación de deprecaciones.
2. **10-B, verificación.** Cobertura, fuzzing de 72 horas, pruebas de carga y
   de fugas.
3. **10-C, auditoría externa y remediación.**
4. **10-D, plataformas.** Cuatro targets nativos y `wasm32-wasip1`.
5. **10-E, publicación.** Libro, curso, envío a benchmarks, liberación de la
   1.0.

### 10.3 Criterios de salida (versión 1.0)

- [ ] Al menos tres proyectos externos usando Anti-Gravital en producción por al menos 30 días sin incidentes críticos.
- [ ] Al menos un servicio interno de Gravital Cloud usando Anti-Gravital en producción por 30 días sin incidentes críticos.
- [ ] Anuncio público de versión 1.0 con changelog completo.
- [ ] Compromiso de semver estricto desde la 1.0.
- [ ] Anuncio del calendario de versiones LTS.
- [ ] Charla en al menos una conferencia internacional (RustConf, EuroRust, RustNation o equivalente).
- [ ] Al menos 10 000 stars en el repositorio.
- [ ] El comité técnico ratifica la promoción a versión 1.0 por unanimidad.

### 10.4 Riesgos de la fase

El riesgo principal es la presión por liberar 1.0 antes de tiempo. La mitigación es la regla más estricta del proyecto: los criterios de salida son no negociables. Si no se cumplen, no se libera 1.0. Se libera 0.9.5, 0.9.6, hasta que se cumplen.

---

## Más allá de la 1.0: hojas de ruta futuras

Una vez liberada la 1.0, el proyecto entra en modo de mantenimiento estable con releases minor cada 3 meses. Los temas candidatos para versiones futuras incluyen:

- Versión 1.x: optimizaciones de rendimiento adicionales, soporte de protocolos adicionales (HTTP/3 vía QUIC).
- Versión 2.x: refactorización de la ABI de plugins si la comunidad WebAssembly hace cambios mayores. Soporte de nuevos targets de despliegue.
- Generador Swift para iOS nativo.
- Generador Kotlin Multiplatform para Android nativo y casos cross-platform.
- Soporte multi-tenant más sofisticado con federación de instancias.

Esta hoja de ruta extendida no es un compromiso. Se documenta para señalar dirección, pero se reservará a RFCs específicos cuando llegue el momento.

---

## Reglas de oro del proceso

A modo de cierre, las cinco reglas que rigen este proceso de extremo a extremo:

**Primera regla.** Una fase no se considera concluida hasta que todas sus casillas de criterio de salida están marcadas. Sin excepciones.

**Segunda regla.** Si una fase requiere más tiempo del estimado, se extiende. Si el alcance original no es alcanzable, se reduce con un RFC público, no se relajan los criterios de calidad.

**Tercera regla.** Toda decisión arquitectónica significativa requiere un RFC. La velocidad de iteración no justifica saltar el proceso.

**Cuarta regla.** El proyecto se libera cuando está listo, no cuando lo exige una fecha externa. La credibilidad técnica es el activo más valioso del proyecto.

**Quinta regla.** Toda promesa pública (benchmark, feature, fecha) se documenta con evidencia. Si no hay evidencia, no se promete.

Estas reglas existen por una razón. Anti-Gravital se propone competir con frameworks que han madurado durante décadas. La única manera de ser tomado en serio es construir con la misma seriedad.

---

**Fin del documento de Hoja de Ruta.**
Documento complementario: *Arquitectura Técnica e Implementación.*
Versión PDF unificada: *Anti-Gravital Blueprint v4.0 — Documento Maestro.*
