# Anti-Gravital — Hoja de Ruta y Puertas de Verificación

**[English](#english) | [Espanol](#espanol)**

**Versión:** 4.0 — Mayo 2026
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
| 5     | `ag-cloud` — simplified deployment         | 2 months           | Next |
| 6     | `ag-ai` and Knowledge Graph                | 2 months           | Pending |
| 7     | `ag-migrate` — importers                   | 2 months           | Pending |
| 8     | `ag-mobile` — Flutter bridge               | 2 months           | Pending |
| 9     | WASI plugin system                         | 2 months           | Pending |
| 10    | Hardening and 1.0 milestone                | 3 months           | Pending |

**Total estimated duration:** 25–30 months from the start.
**Public beta version milestone (0.5):** end of phase 5 (~15 months).
**Stable version 1.0 milestone:** end of phase 10 (~30 months).

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

- [ ] Repository `github.com/gravital-labs/anti-gravital` created and public.
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

## Phase 5 — `ag-cloud` simplified deployment

**Objective.** Build the deployment subsystem in the style of Railway/Fly.io. Support for the four targets: docker-compose, fly, railway, k8s. This is the **public beta version (0.5)** milestone.

### 5.1 Entry criteria

- [ ] Phase 4 completed.
- [ ] RFC decision on the deployment targets supported in 1.0.

### 5.2 Deliverables

- [ ] `ag-cloud` crate with modules for each target.
- [ ] Specification of the `deploy.ag` file.
- [ ] Multi-stage Dockerfile generator optimized for minimal image.
- [ ] docker-compose target: complete stack generation with Caddy as reverse proxy and automatic TLS.
- [ ] fly target: integration with flyctl.
- [ ] railway target: integration with its API.
- [ ] k8s target: generation of standard manifests.
- [ ] `ag deploy` command.
- [ ] `ag rollback` command.
- [ ] Database migrations pipeline integrated into the deployment.
- [ ] Documentation: "From zero to production in 15 minutes" with each target.

### 5.3 Exit criteria (gate before Phase 6 and version 0.5)

- [ ] The `todo-api` example deploys successfully to Fly.io with `ag deploy`.
- [ ] The `ecommerce-api` example deploys successfully with docker-compose to a VPS and is accessed via domain with TLS.
- [ ] The `realtime-chat` example deploys successfully to Railway.
- [ ] Version 0.5 (public beta) released on GitHub Releases.
- [ ] Public announcement on Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] At least ten external projects report that they have deployed Anti-Gravital in production or staging.
- [ ] At least 1 500 stars on the repository.

### 5.4 Phase risks

The main risk is the dependency on external APIs (Fly, Railway) that can change. The mitigation is to structure each target as a decoupled module with contract tests.

---

## Phase 6 — `ag-ai` and Knowledge Graph

**Objective.** Build the AI module with the knowledge graph and the assisted capabilities.

### 6.1 Entry criteria

- [ ] Version 0.5 (public beta) released.
- [ ] Feedback from the first users incorporated into the backlog.

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

### 6.3 Exit criteria (gate before Phase 7)

- [ ] The knowledge graph regenerates correctly with each `ag generate`.
- [ ] The generated architectural documentation is readable and useful (reviewed by three people external to the team).
- [ ] At least one user organization reports that it has integrated `ag ai` into its workflow.
- [ ] At least 2 500 stars on the repository.

### 6.4 Phase risks

The main risk is the dependency on external AI providers. The mitigation is provider abstraction and offline mode.

---

## Phase 7 — `ag-migrate` importers

**Objective.** Build the migration importers from legacy frameworks. It is probably the phase with the greatest impact on real adoption.

### 7.1 Entry criteria

- [ ] Phase 6 completed.
- [ ] Research of real samples: at least ten schemas/projects of each target framework collected as a testing corpus.

### 7.2 Deliverables

- [ ] `ag-migrate` crate with five importers:
  - [ ] OpenAPI 3.0 and 3.1 importer.
  - [ ] Prisma importer.
  - [ ] Django importer.
  - [ ] FastAPI importer.
  - [ ] Sequelize importer.
  - [ ] GraphQL SDL importer.
- [ ] `ag migrate from <framework> <path>` command.
- [ ] Official migration guides per framework with complete examples.
- [ ] Documented case study: real migration of a medium-sized FastAPI application.

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
- [ ] Compilation to `wasm32-wasi` to serve Anti-Gravital in edge functions.
- [ ] Official manual published: "The Anti-Gravital Book" in Spanish and English.
- [ ] Framework introduction course on YouTube (minimum six videos).
- [ ] Position in TechEmpower Framework Benchmarks: top 10 in Plaintext and JSON Serialization categories.

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
| 5    | `ag-cloud` — despliegue simplificado       | 2 meses           | Próxima |
| 6    | `ag-ai` y Knowledge Graph                  | 2 meses           | Pendiente |
| 7    | `ag-migrate` — importadores                | 2 meses           | Pendiente |
| 8    | `ag-mobile` — Flutter bridge               | 2 meses           | Pendiente |
| 9    | Sistema de plugins WASI                    | 2 meses           | Pendiente |
| 10   | Endurecimiento y hito 1.0                  | 3 meses           | Pendiente |

**Duración total estimada:** 25–30 meses desde el inicio.
**Hito de versión beta pública (0.5):** final de fase 5 (~15 meses).
**Hito de versión 1.0 estable:** final de fase 10 (~30 meses).

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

- [ ] Repositorio `github.com/gravital-labs/anti-gravital` creado y público.
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

## Fase 5 — `ag-cloud` despliegue simplificado

**Objetivo.** Construir el subsistema de despliegue al estilo Railway/Fly.io. Soporte para los cuatro targets: docker-compose, fly, railway, k8s. Este es el hito de **versión beta pública (0.5)**.

### 5.1 Criterios de entrada

- [ ] Fase 4 completada.
- [ ] Decisión RFC sobre los targets de despliegue soportados en la 1.0.

### 5.2 Entregables

- [ ] Crate `ag-cloud` con módulos para cada target.
- [ ] Especificación del archivo `deploy.ag`.
- [ ] Generador de Dockerfile multi-stage optimizado para imagen mínima.
- [ ] Target docker-compose: generación completa de stack con Caddy como reverse proxy y TLS automático.
- [ ] Target fly: integración con flyctl.
- [ ] Target railway: integración con su API.
- [ ] Target k8s: generación de manifests estándar.
- [ ] Comando `ag deploy`.
- [ ] Comando `ag rollback`.
- [ ] Pipeline de migraciones de base de datos integrado al despliegue.
- [ ] Documentación: "Desde cero a producción en 15 minutos" con cada target.

### 5.3 Criterios de salida (puerta antes de Fase 6 y versión 0.5)

- [ ] El example `todo-api` se despliega exitosamente a Fly.io con `ag deploy`.
- [ ] El example `ecommerce-api` se despliega exitosamente con docker-compose a un VPS y se accede vía dominio con TLS.
- [ ] El example `realtime-chat` se despliega exitosamente a Railway.
- [ ] Versión 0.5 (beta pública) liberada en GitHub Releases.
- [ ] Anuncio público en Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] Al menos diez proyectos externos reportan que han desplegado Anti-Gravital en producción o staging.
- [ ] Al menos 1 500 stars en el repositorio.

### 5.4 Riesgos de la fase

El riesgo principal es la dependencia de APIs externas (Fly, Railway) que pueden cambiar. La mitigación es estructurar cada target como un módulo desacoplado con tests de contrato.

---

## Fase 6 — `ag-ai` y Knowledge Graph

**Objetivo.** Construir el módulo de IA con el knowledge graph y las capacidades asistidas.

### 6.1 Criterios de entrada

- [ ] Versión 0.5 (beta pública) liberada.
- [ ] Retroalimentación de los primeros usuarios incorporada en backlog.

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

### 6.3 Criterios de salida (puerta antes de Fase 7)

- [ ] El knowledge graph se regenera correctamente con cada `ag generate`.
- [ ] La documentación arquitectónica generada es legible y útil (revisada por tres personas externas al equipo).
- [ ] Al menos una organización usuaria reporta que ha integrado `ag ai` en su flujo de trabajo.
- [ ] Al menos 2 500 stars en el repositorio.

### 6.4 Riesgos de la fase

El riesgo principal es la dependencia de proveedores externos de IA. La mitigación es la abstracción del proveedor y el modo offline.

---

## Fase 7 — `ag-migrate` importadores

**Objetivo.** Construir los importadores de migración desde frameworks legacy. Es probablemente la fase con mayor impacto en adopción real.

### 7.1 Criterios de entrada

- [ ] Fase 6 completada.
- [ ] Investigación de muestras reales: al menos diez schemas/proyectos de cada framework objetivo recolectados como corpus de testing.

### 7.2 Entregables

- [ ] Crate `ag-migrate` con cinco importadores:
  - [ ] Importador OpenAPI 3.0 y 3.1.
  - [ ] Importador Prisma.
  - [ ] Importador Django.
  - [ ] Importador FastAPI.
  - [ ] Importador Sequelize.
  - [ ] Importador GraphQL SDL.
- [ ] Comando `ag migrate from <framework> <ruta>`.
- [ ] Guías oficiales de migración por framework con ejemplos completos.
- [ ] Estudio de caso documentado: migración real de una aplicación FastAPI mediana.

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
- [ ] Compilación a `wasm32-wasi` para servir Anti-Gravital en edge functions.
- [ ] Manual oficial publicado: "The Anti-Gravital Book" en español e inglés.
- [ ] Curso de introducción al framework en YouTube (mínimo seis videos).
- [ ] Posición en TechEmpower Framework Benchmarks: top 10 en categorías Plaintext y JSON Serialization.

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
