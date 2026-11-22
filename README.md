# Anti-Gravital

Rust-native, modular backend framework for building secure, high-performance
services with a schema-first workflow: declare your models, endpoints, auth,
events, mail, domains and workers in a single `.ag` schema, generate the
Rust/SQL/TypeScript/OpenAPI surface, and ship a small static binary.

[![CI](https://github.com/Anti-Gravital/Anti-Gravital/actions/workflows/ci.yml/badge.svg)](https://github.com/Anti-Gravital/Anti-Gravital/actions/workflows/ci.yml)
[![Quality](https://github.com/Anti-Gravital/Anti-Gravital/actions/workflows/quality.yml/badge.svg)](https://github.com/Anti-Gravital/Anti-Gravital/actions/workflows/quality.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust: 1.99+](https://img.shields.io/badge/rust-1.99%2B-orange.svg)](rust-toolchain.toml)
[![Status: alpha v0.1.0-alpha.2](https://img.shields.io/badge/status-alpha-orange.svg)](docs/releases/0.1.0-alpha.2.md)

[English](#english) | [Espanol](#espanol) — English is the canonical project
language (ADR-0008). A full Spanish version follows the English one.

---

## English

### Table of contents

1. [Project at a glance](#project-at-a-glance)
2. [What is Anti-Gravital](#what-is-anti-gravital)
3. [Try it](#try-it)
4. [Alpha: what works and what does not yet work](#alpha-what-works-and-what-does-not-yet-work)
5. [What you can do today](#what-you-can-do-today)
6. [What you cannot do yet](#what-you-cannot-do-yet)
7. [What Anti-Gravital is not](#what-anti-gravital-is-not)
8. [Quick start](#quick-start)
9. [The Anti-DSL](#the-anti-dsl)
10. [CLI reference](#cli-reference)
11. [Architecture](#architecture)
12. [Repository layout](#repository-layout)
13. [Vision and engineering principles](#vision-and-engineering-principles)
14. [Roadmap and phase model](#roadmap-and-phase-model)
15. [Known limitations and release blockers](#known-limitations-and-release-blockers)
16. [Documentation map](#documentation-map)
17. [Contributing and security](#contributing-and-security)

### Project at a glance

| | |
| --- | --- |
| Language and runtime | Rust (MSRV 1.99.1) on Tokio, Axum, Tower and rustls |
| License | Apache-2.0 |
| Released version | Alpha `v0.1.0-alpha.2`, available for testing from source ([release notes](docs/releases/0.1.0-alpha.2.md)); no crates.io package, source install is the verified path |
| Workspace | 19 `ag-*` crates plus the `ag` developer CLI (5 of them are reserved placeholders for later phases) |
| Current position | Phases 0-4.5 implemented with their exit gates still open; additive Phase 4.6 in progress |
| Production-ready | No. This is a testing alpha; the formal pre-Phase-5 release gate is OPEN ([release gate](docs/audits/PRE_FASE5_RELEASE_GATE.md)) |
| Live status | [docs/roadmap/STATUS.md](docs/roadmap/STATUS.md), checkbox-level, updated with every roadmap PR |

Status statements in this README were verified on 2026-11-22 against
`docs/roadmap/STATUS.md` (last updated 2026-11-10). "Implemented" means the
code and tests exist in this repository and pass CI; it is never a
production-readiness certification.

### What is Anti-Gravital

Anti-Gravital gives Rust backend teams a coherent framework experience without
hiding the underlying Rust ecosystem. Instead of wiring Axum, sqlx, tracing,
JWT, rate limiting and code generation by hand on every project, you get:

- **The Shield**: a security-first HTTP pipeline in `ag-core` (TLS 1.3,
  HTTP/1.1 and HTTP/2, JWT auth, rate limiting, CORS, CSRF, payload
  validation, structured logging) with secure defaults.
- **The Anti-DSL**: a declarative `.ag` schema language (versions v0.1-v0.8
  implemented) for models, relations, validations, endpoints, auth policies,
  events, mail, domains, templates and background workers.
- **Code generation**: one schema produces Rust handlers and types, SQL
  migrations, TypeScript types and client, OpenAPI 3.1 and AsyncAPI 2.6.
- **The `ag` CLI**: scaffolding, development loop, release builds, schema
  workflows, mail checks, domain operations and worker management.
- **Modular crates**: auth, data, cache, realtime, storage, observability,
  transactional mail, background jobs, DNS/TLS domain management and edge
  routing — every crate independently selectable, nothing mandatory beyond
  `ag-core`.

The result targets small, auditable deployments: the reference `todo-api`
example builds to a 5.3 MB static MUSL binary and a 2.49 MB `FROM scratch`
Docker image (measured 2026-05-21).

### Try it

`v0.1.0-alpha.2` is an alpha release for testing, not for production. Install
from source, scaffold a project, run it locally and generate code from a
schema:

```bash
# 1. Install (Git and Rust 1.99.1+ required)
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh

# 2. Scaffold a project (Windows: install with .\install.ps1, then use the same ag commands)
ag new my-api --template rest

# 3. Run locally (see the bind limitation below before exposing the server)
cd my-api && ag dev

# 4. Lint a schema and generate artifacts (from the project directory)
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

Windows users install with `.\install.ps1` (PowerShell) from the cloned repo;
the `ag` commands are identical. Verify the checkout before running an
installer with the [installation integrity procedure](docs/security/INSTALLATION_INTEGRITY.md).
Full walkthrough: [Your first API](docs/manual/02-primera-api.md). Runnable
prototypes: [`examples/`](examples/). Release notes:
[`v0.1.0-alpha.2`](docs/releases/0.1.0-alpha.2.md).

### Alpha: what works and what does not yet work

Works in `v0.1.0-alpha.2`:

- Install from source with `install.sh`, `install.ps1` or `cargo install --path`.
- Scaffold `rest`, `realtime` and `fullstack` projects with `ag new`.
- Run a local development server with `ag dev` (CLI default bind is
  `127.0.0.1:8080`; current scaffolds bind all interfaces, issue #191).
- Lint a `.ag` schema and generate Rust, SQL, TypeScript and OpenAPI with
  `ag schema lint` and `ag generate`.
- Build a release binary with `ag build`; inspect and validate config with
  `ag config`.
- Render authored Docker/Kubernetes artifacts locally with
  `ag deploy plan --target docker|k8s`, review the plan id and write the files
  with `ag deploy apply --plan ID` (alpha, local only: no cluster or runtime).

Does not yet work in the alpha:

- No crates.io package. Prebuilt binaries are produced for four targets and an
  opt-in alpha channel (`AG_INSTALLER_CHANNEL=alpha`) downloads one with a
  SHA-256 check only; it makes no signature or authenticity claim and its
  end-to-end verification is still pending (#141 remains open). Source install
  is the verified path.
- Do not expose the `ag dev` server on untrusted networks: the CLI default
  bind is loopback, but generated scaffolds hardcode `0.0.0.0:8080` and ignore
  the `BIND` variable (issue #191).
- `ag deploy` only renders local artifacts; cluster deployment with
  `ag-cloud`, plus `ag migrate` and `ag plugin`, belongs to future phases.
- No MCP server or shared machine interface: the RFC-0024 prerequisites are
  not met.
- No production-readiness claim: the pre-Phase-5 gate is open and the
  performance and 24-hour fuzz gates are pending.
- Generated Rust handlers are stubs; your application supplies the logic.
- Public APIs may change before 1.0; there is no SemVer stability promise yet.

### What you can do today

Everything below exists in this repository, compiles, and is covered by tests
in CI:

- Serve HTTPS APIs through the `ag-core` Shield pipeline with all security
  layers active (Phase 1).
- Use typed extractors and responses, PostgreSQL pools and embedded
  migrations through `ag-data`, and scaffold projects with three templates:
  `rest`, `realtime`, `fullstack` (Phase 2).
- Write `.ag` schemas and generate Rust, SQL, TypeScript, OpenAPI and
  AsyncAPI artifacts, with readable diagnostics, an LSP server (`ag-lsp`)
  and a packaged VS Code extension (Phase 3).
- Use the standard modules (Phase 4):
  - `ag-auth`: WebAuthn/FIDO2, OAuth2 PKCE, JWT Ed25519, API keys, refresh
    tokens.
  - `ag-cache`: in-process L1 cache plus a native RESP2 L2 server, with an
    optional external Redis L2 for multi-instance setups (feature `redis-l2`).
  - `ag-realtime`: in-process event bus, WebSocket/SSE helpers, optional
    external NATS client.
  - `ag-storage`: filesystem store, image processing, signed URLs, optional
    S3-compatible backend.
  - `ag-observe`: structured tracing, Prometheus metrics, OTLP groundwork.
- Send transactional email (`ag-mail`: SMTP relay, templates, auth flows for
  verification/recovery/magic links) and manage DNS records, ACME/Let's
  Encrypt certificates and SPF/DKIM/DMARC (`ag-domains`) (Phase 4.5).
- Run the opt-in native outbound MTA and signed webhooks in `ag-mail`
  (features `mta`/`api`, Phase 4.6-A/B/C).
- Run background jobs with `ag-workers` (Phase 4.6-D): typed jobs, retries
  with backoff, dead-letter queue, interval scheduling and worker pools. The
  in-memory backend is the default; the durable PostgreSQL backend is opt-in
  and was verified against a live PostgreSQL 16 (its integration tests stay
  `#[ignore]` because default CI runs without a database).

Ten runnable examples live in [`examples/`](examples/), from `todo-api`
(CRUD against PostgreSQL) to `workers-postgres` and `auth-mail-demo`.
Later roadmap phases are additive: none of them is required to use the
capabilities above.

### What you cannot do yet

Be aware of these boundaries before adopting the project:

- **No cluster deployment.** `ag deploy` (RFC-0034) only copies authored
  Docker/Kubernetes artifacts into `deploy/generated/`; it runs no container
  and makes no API call. Secrets, live deployment, logs and rollback arrive
  with `ag-cloud` in Phase 5. `ag migrate` (Phase 7) and `ag plugin`
  (Phase 9) are also future commands.
- **No package-manager install.** Nothing is published to crates.io yet;
  you build from source with the audited installer scripts.
- **No stable-API promise.** Public APIs can still change until Phase 10
  (1.0). SemVer discipline starts with the first tagged release.
- **No production certification.** The pre-Phase-5 release gate is open;
  performance targets and the 24-hour fuzz gate are still pending (see
  [Known limitations](#known-limitations-and-release-blockers)).
- **Placeholder crates.** `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`
  and `ag-wasm-host` exist only to reserve names and module boundaries;
  their implementation begins in Phases 5-9. `ag-ui` was removed from the
  roster by `RFC-0019`.

### What Anti-Gravital is not

Anti-Gravital integrates with the dominant tools instead of replacing them.
It does not reimplement PostgreSQL, Redis, NATS, object storage, Docker,
Kubernetes, Terraform, Flutter or frontend frameworks. `ag-mail` sends
transactional email; it is not an IMAP/POP mailbox host. `ag-domains`
manages DNS/TLS workflows; it is not a domain registrar. It does not render
user interfaces either (`RFC-0019`): UI consumers get generated client SDKs.
Generated Rust handlers are deliberate stubs owned by your application — the
framework never hides your business logic.

### Quick start

Prerequisites: Git and Rust 1.99.1 or newer.

Linux or macOS:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

The installer verifies the Rust version, builds the workspace in release mode
and installs `ag` into the Cargo bin directory. It performs no privileged
system changes. Before running it, verify the checkout with the canonical
[installation integrity procedure](docs/security/INSTALLATION_INTEGRITY.md).

Then create and run a project. In a terminal, `ag new` with no arguments
opens a guided wizard that asks for the name and the template and shows what
it will write before writing it:

```bash
ag new
```

The same thing without any prompt, for scripts and CI:

```bash
ag new my-api --template rest
cd my-api
ag dev
```

Walkthrough: [Your first API with Anti-Gravital](docs/manual/02-primera-api.md).

### Templates

Use `ag new <name> --template <name> --no-interactive` to scaffold from a
catalog template. The [template gallery](docs/templates/GALLERY.md) is the
decision table from use case to template, external requirements and example
command; [docs/templates/README.md](docs/templates/README.md) is its index.
The current alpha binary accepts `rest`, `realtime`, `fullstack` and `layered`;
the remaining wave-6 names arrive with the catalog registry and template
directories.

To inspect or change configuration, run `ag config` for the interactive form
or use its scriptable equivalents:

```bash
ag config get bind
ag config set rate_limit.enabled true
ag config validate
```

Every write is validated with the same `ShieldConfig` loader the runtime uses,
and the file is replaced atomically. Bare `ag` opens a home screen in a
terminal that launches these commands without performing work itself.

### The Anti-DSL

The `.ag` schema is the source of truth of a project. Abridged excerpt from
[`examples/ecommerce-api/schema.ag`](examples/ecommerce-api/schema.ag):

```text
config {
    project_name "ecommerce-api"
    database "postgres"
}

model Product {
    id          UUID      @primary @auto
    name        String    @min(2) @max(200)
    price       Decimal   @min(0)
    category_id UUID      @references(Category.id)
    category    Category  @relation(category_id)
    created_at  Timestamp @auto
}

endpoint GetProduct {
    method   GET
    path     /products/{id}
    response ProductResponse
    errors   [NotFound]
}
```

Lint and generate:

```bash
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

The generator writes a Rust module, SQL migration, TypeScript types and
client, OpenAPI and optional AsyncAPI artifacts. Generated handler bodies are
deliberate stubs that your application implements. Rust-side `@regex` request
validation is executable and caches each compiled pattern; generated projects
using it must declare `regex = "1"`. Implemented DSL versions: v0.1 (models)
through v0.8 (`worker` declarations). Full reference:
[docs/dsl/](docs/dsl/).

### CLI reference

| Command | Purpose |
| --- | --- |
| `ag` (no arguments) | In an interactive terminal, opens a home screen that launches existing commands; otherwise prints help |
| `ag new [NAME] [--template TYPE]` | Scaffold a project (`rest`, `realtime`, `fullstack`). Missing input opens a guided wizard in an interactive terminal; `--no-interactive`, `CI` or a non-terminal session never prompt |
| `ag config` | In a terminal, opens a full-screen form for `config.toml`. Non-interactive equivalents: `ag config get KEY`, `ag config set KEY VALUE` and `ag config validate` (the last exits non-zero on an invalid file, usable in CI). Secrets are never shown; key paths or environment-variable names only |
| `ag dev [--bind ADDR]` | Run in development mode; the CLI default bind is `127.0.0.1:8080`. Caveat: generated templates currently hardcode `0.0.0.0:8080` and ignore `BIND` (issue #191), so the server listens on all interfaces until fixed. Uses cargo-watch when installed |
| `ag build [--target TRIPLE]` | Build a release binary |
| `ag generate --schema schema.ag --output generated` | Generate DSL artifacts |
| `ag schema lint` | Validate a DSL schema and report diagnostics |
| `ag schema diff REFERENCE` | Classify schema changes |
| `ag mail test --to ADDRESS` | Verify SMTP configuration |
| `ag domains check --domain HOST` | Check DNS propagation |
| `ag domains sync --zone-id ID` | Apply schema DNS records through the configured provider |
| `ag domains attach\|instructions\|export-zone\|status\|list\|verify\|detach\|diagnose` | Operate the local domain attachment workflow |
| `ag workers list` | List the background workers declared in a schema |
| `ag workers run` | Run a standalone worker process against the configured backend |
| `ag workers enqueue KIND --payload FILE` | Enqueue a job onto the durable backend (needs `DATABASE_URL`) |
| `ag workers queues` | Show queue depths on the durable backend |
| `ag workers dlq list\|inspect\|retry\|purge` | Inspect and manage the dead-letter queue |
| `ag workers doctor` | Check workers config and durable-backend connectivity |
| `ag deploy plan --target docker\|k8s` | Print a deterministic, read-only plan (file list, sizes, fingerprints and plan id) from authored artifacts (alpha, local only) |
| `ag deploy apply --plan ID` | Write that exact plan under `deploy/generated/`; refuses malformed or stale plan ids |
| `ag deploy status` | Report generated files and whether the applied plan is still current |

The global `--output json` flag produces versioned machine-readable output for
the read-only commands `ag schema lint`, `ag schema diff`, `ag workers list`,
`ag deploy plan` and `ag deploy status` (contract `0.2.0`, documented in
[docs/mcp/](docs/mcp/)). Other commands, including `ag deploy apply`, refuse it
before doing any work; prose output is unchanged when the flag is absent.

Run `ag COMMAND --help` for authoritative flags and environment variables.
`ag deploy` renders local artifacts only (RFC-0034); `ag migrate` and
`ag plugin` are future commands and are not in the current binary. Detailed
workflows: [`ag-cli` guide](crates/ag-cli/README.md) and
[domain CLI reference](docs/ag-domains/reference/cli.md).

### Architecture

The workspace is a strict, layered Cargo workspace. `ag-core` depends on no
other Anti-Gravital crate; every other crate is opt-in; circular dependencies
are forbidden and checked in CI.

```mermaid
flowchart TB
    subgraph workflow["Schema-first workflow"]
        SCHEMA["schema.ag<br>(Anti-DSL)"]
        CLI["ag CLI"]
        GEN["Generated artifacts:<br>Rust, SQL migrations, TypeScript,<br>OpenAPI, AsyncAPI"]
        CLI -- "ag generate / lint / diff" --> SCHEMA
        SCHEMA --> GEN
    end

    subgraph app["Your application (single static binary)"]
        HANDLERS["Handlers you implement<br>(generated stubs)"]
    end
    GEN --> HANDLERS

    subgraph core["Core tier"]
        AGCORE["ag-core<br>Shield security pipeline + HTTP runtime"]
    end

    subgraph standard["Standard tier (opt-in)"]
        AUTH["ag-auth"]
        DATA["ag-data"]
        CACHE["ag-cache"]
        RT["ag-realtime"]
        STORE["ag-storage"]
        OBS["ag-observe"]
    end

    subgraph deferred["Deferred standard (opt-in)"]
        MAIL["ag-mail"]
        WORKERS["ag-workers"]
    end

    subgraph infra["Optional infrastructure"]
        DOMAINS["ag-domains"]
        EDGE["ag-edge"]
    end

    HANDLERS --> AGCORE
    HANDLERS -.-> standard
    HANDLERS -.-> deferred
    HANDLERS -.-> infra

    DATA --> PG[("PostgreSQL")]
    AGCORE --> NET(("HTTPS / HTTP"))
```

Crate tiers follow the canonical classification in [`CLAUDE.md`](CLAUDE.md)
§14 and chapter 5 of the architecture master. `*` marks a placeholder crate
whose implementation begins in a later phase.

| Tier | Crates | Role |
| --- | --- | --- |
| Core | `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`* | Shield/runtime, DSL compiler, developer CLI, editor LSP, WASI plugin host |
| Standard | `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe` | Auth, PostgreSQL data layer, events, cache, files/images, telemetry |
| Deferred standard | `ag-mail`, `ag-workers` | Transactional mail and native MTA; background-job execution engine. Standard-tier maturity, not installed by default in templates |
| Optional | `ag-cloud`*, `ag-ai`*, `ag-mobile`*, `ag-migrate`* | Cloud deploy, AI/knowledge graph, mobile bridge, importers |
| Optional infra | `ag-domains`, `ag-edge` | DNS/domain/TLS control plane and the request-time edge data plane |

Every external service integration (NATS, S3, Cloudflare, third-party SMTP)
lives behind a Cargo feature with a native default mode, so no external
dependency is ever required to use a module (ADR-0009).

### Repository layout

```text
anti-gravital/
|-- crates/        19 ag-* crates (Cargo workspace; see architecture map)
|-- examples/      10 runnable example projects
|-- templates/     project templates used by `ag new`
|-- benchmarks/    benchmark harnesses and methodology
|-- fuzz/          cargo-fuzz targets (lexer, parser, compile, workers payload)
|-- tests/         cross-module integration tests
|-- tools/         repository tooling
|-- docs/          all documentation (see the documentation map below)
|-- install.sh     source installer (Linux/macOS); install.ps1 for Windows
`-- CLAUDE.md      the technical constitution that governs all work
```

### Vision and engineering principles

Anti-Gravital is built as real, sustainable, technically defensible
infrastructure — never as a hype-driven demo. The non-negotiable principles
(codified in [`CLAUDE.md`](CLAUDE.md) and the master documents):

- **Documentation first.** The architecture is documented before it is
  implemented; if code contradicts the documentation, the code is wrong.
  Large decisions require an RFC, and accepted decisions persist as ADRs.
- **Integration over reinvention.** Where a dominant tool exists
  (PostgreSQL, Kubernetes, Terraform, ...), Anti-Gravital integrates with it.
- **Security by construction.** `unsafe` is denied workspace-wide, defaults
  are secure, and `cargo audit`/`cargo deny`/`clippy -D warnings` gate CI.
- **Honest evidence.** Benchmarks publish hardware, methodology and standard
  deviation; targets that were not met are recorded as not met (see Phase 2
  in the roadmap below for a real example).
- **Operational simplicity.** Static binaries, explicit configuration,
  deterministic builds, native observability, minimal runtime.
- **Phase discipline.** Nothing is built ahead of its phase; no speculative
  abstractions.

### Roadmap and phase model

How to read the roadmap:

- **Blocking phases (0-10)** run in order. Each phase document under
  [`docs/roadmap/`](docs/roadmap/) defines entry criteria, deliverables and
  exit criteria (the "gate"). A phase counts as **implemented** when its
  deliverables exist with tests in this repository, and as **closed** only
  when every exit criterion — including performance evidence and external
  community criteria — is verified. Implementation of Phases 1+ proceeds in
  parallel with the pending external criteria of Phase 0 under the
  documented exception RFC-0001.
- **Additive phases (4.5, 4.6)** extend the ecosystem between blocking
  phases. They are authorized by ADR, never required by earlier
  capabilities, and never skipped ahead of their own RFCs.
- **Future phases** follow the same mechanism: the next phase opens only
  when the previous gate closes, every new crate or scope change requires an
  RFC first, and no production/GA claim is made while a gate is open.
  Durations in the phase documents are planning estimates, not release
  promises.

Current state, phase by phase (evidence in
[docs/roadmap/STATUS.md](docs/roadmap/STATUS.md)):

| Phase | Scope | Status | What keeps the gate open |
| --- | --- | --- | --- |
| 0 | Foundations: governance, Apache-2.0, monorepo, CI on four platforms, technical constitution | In progress | External deliverables: branding, community channels, landing page, public release calendar |
| 1 | Shield MVP: HTTP/1.1+2, TLS 1.3, JWT, rate limiting, CORS, CSRF, validation, structured logging | Implemented; gate open | Reference-hardware targets (>= 300K req/s hello-world, p99 <= 1 ms), official coverage measurement, external criteria |
| 2 | Core MVP: typed extractors/responses, PostgreSQL layer, migrations, scaffolds, CRUD example | Implemented; gate open | CRUD targets not met on recorded hardware: measured 14 478 req/s vs the 40K target and p99 14.6 ms vs 5 ms on a Ryzen 5 2500U; full analysis published in [docs/benchmarks/](docs/benchmarks/) |
| 3 | Anti-DSL v0.1-v0.4: parser, diagnostics, Rust/SQL/TypeScript/OpenAPI generators, LSP, VS Code extension | Implemented; gate open | 24-hour fuzz gate, direct generated-vs-manual benchmark, generator consolidation (issue #70), adoption criteria |
| 4 | Standard modules: `ag-auth`, `ag-cache`, `ag-realtime`, `ag-storage`, `ag-observe`; DSL v0.5-v0.6 | Implemented; gate open | crates.io releases, scale benchmarks (50K WebSocket connections, 1M cache ops/s), community criteria |
| 4.5 (additive) | `ag-mail` transactional email; `ag-domains` DNS/ACME/SPF-DKIM-DMARC; DSL v0.7 | Implemented; gate open | Gate re-run on the final consolidation commit; `ag-domains` has active ongoing work |
| 4.6 (additive) | Pre-Phase-5 hardening: native outbound MTA + signed webhooks in `ag-mail` (A/B/C); `ag-workers` engine (D); DSL v0.8 | In progress | `ag-edge` producer wiring (issue #112); MTA live-delivery evidence (issue #153). The MTA durable spool now ships behind the `queue-postgres` feature (issue #151; its live PostgreSQL test is `#[ignore]`). Stages S1-S5 and the S7 `ag-mail` migration are done; the `ag-workers` PostgreSQL backend was verified against a live database (issues #108/#109/#103) |
| 4.7 (additive) | Interactive terminal experience: full-screen TUI in the `ag` binary to create and configure a project, with exact non-interactive equivalents | In progress | `RFC-0018` accepted. Stages A-B done: `ag new` wizard, `--no-interactive`, `ag config` get/set/validate plus the full-screen form with validation before write. Stage C done: bare `ag` home screen with keyboard navigation that only launches existing commands. Stages A-B are required for v0.5, stage C for 1.0 |
| 4.8 (additive) | Project foundations: layered project structure, declared environment and encrypted secrets, verified one-command installation, remote console over SSH, machine-readable interface and a shared capability-scoped MCP interface | Accepted, not started | `RFC-0019`, `RFC-0020` and `RFC-0021` accepted on 2026-11-10; `RFC-0024` accepted on 2026-11-21; no code yet |
| 5 | `ag-cloud`: simplified build/deploy, secrets, logs, rollback, domains, TLS | Pending | Opens when the pre-Phase-5 gate closes. Milestone: public beta v0.5 |
| 5.5 (additive) | Serverless functions: independently deployed, sandboxed WebAssembly functions with explicit capabilities, triggered by HTTP, schedule, queue or events | Accepted, not started | `RFC-0023` accepted on 2026-11-10; no code yet |
| 6 | `ag-ai` and Knowledge Graph: providers, retrieval, graph-assisted workflows | Pending | Phase 5 completion and beta feedback |
| 7 | `ag-migrate`: importers and assisted migration from other backend frameworks | Pending | Phase 6 completion and importer acceptance tests |
| 8 | `ag-mobile`: Flutter/Dart bridge, generated clients, offline contracts | Pending | Phase 7 completion and mobile compatibility gates |
| 9 | WASI plugins: sandboxed extensions, lifecycle hooks, permissions, registry | Pending | Phase 8 completion and security review |
| 10 | Hardening and 1.0: stable API/DSL, security audit, performance, docs, LTS | Pending | Phase 9 completion and the 1.0 release gates. Milestone: stable 1.0 |

Estimated milestones from the original plan: public beta v0.5 at the end of
Phase 5 (around month 15) and stable 1.0 at the end of Phase 10 (around
month 30). These are estimates, not commitments. The authoritative release
decision lives in
[docs/audits/PRE_FASE5_RELEASE_GATE.md](docs/audits/PRE_FASE5_RELEASE_GATE.md);
the complete phase documents and calendar live in
[docs/roadmap/](docs/roadmap/README.md).

### Known limitations and release blockers

Stated plainly, because hiding them would violate the project's own rules:

- The 24-hour fuzz gate (Phase 3) has not been executed yet; the fuzz
  harness runs a 60-second smoke test in CI.
- Final performance evidence on stabilized code is pending; published Phase 1
  and Phase 2 targets have not been met on the recorded hardware.
- Generated Rust handlers are intentional stubs; the framework does not
  write your business logic.
- `ag-domains` has active, ongoing development.
- The durable PostgreSQL paths of `ag-workers` and `ag-mail`'s workers-backed
  delivery were verified manually against a live PostgreSQL 16; their
  integration tests remain `#[ignore]` because default CI provisions no
  database. The native MTA scheduled-queue spool (feature `queue-postgres`,
  issue #151) adds its own `#[ignore]` PostgreSQL test, pending a live run; its
  in-memory durability mechanism is covered by passing tests.
- Open technical debt is tracked as GitHub Issues (label `tech-debt`);
  [docs/DEBT.md](docs/DEBT.md) is a frozen historical record.
- The release gate must be re-evaluated on the final consolidation commit
  before any production-readiness claim.

### Documentation map

Reading order for newcomers — human or automated. An AI agent working on
this repository must read [`CLAUDE.md`](CLAUDE.md) first: it is the binding
technical constitution, and documentation takes precedence over code.

| Resource | What it gives you |
| --- | --- |
| [`CLAUDE.md`](CLAUDE.md) | The technical constitution: governance rules, crate boundaries, workflow, quality gates |
| [`docs/INDEX.md`](docs/INDEX.md) | Master index of all documentation |
| [`docs/master/`](docs/master/) | The three master documents (Blueprint, Technical Architecture, Roadmap) — the source of truth |
| [`docs/releases/0.1.0-alpha.2.md`](docs/releases/0.1.0-alpha.2.md) | Scope, known limitations and verification evidence for the alpha |
| [`docs/roadmap/STATUS.md`](docs/roadmap/STATUS.md) | Live checkbox-level status of every phase |
| [`docs/architecture/`](docs/architecture/) | Architecture chapters derived from the masters |
| [`docs/rfc/`](docs/rfc/) and [`docs/adr/`](docs/adr/) | Design proposals and accepted decisions |
| [`docs/manual/`](docs/manual/) | User manual chapters (Shield as library, first API, domains/TLS/mail) |
| [`docs/dns/`](docs/dns/README.md) | Bilingual domain-to-deploy guide: attach, verify, environments, public/private, troubleshooting |
| [`docs/modules/`](docs/modules/) | Per-crate module documentation |
| [`docs/benchmarks/`](docs/benchmarks/) | Measured benchmarks with hardware and methodology |
| [`docs/graph/`](docs/graph/) | Structured knowledge graph of modules, commands and relations |
| [`examples/`](examples/) | Ten runnable example projects |

### Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md), [CLAUDE.md](CLAUDE.md),
[SECURITY.md](SECURITY.md) and [GOVERNANCE.md](GOVERNANCE.md) before opening
a change. Every contributor should be able to `git clone`, `cargo build` and
`cargo test` without depending on the maintainer. Vulnerabilities are
reported through the process in [SECURITY.md](SECURITY.md), never through
public issues.

---

## Espanol

### Indice

1. [El proyecto de un vistazo](#el-proyecto-de-un-vistazo)
2. [Que es Anti-Gravital](#que-es-anti-gravital)
3. [Probarlo](#probarlo)
4. [Alpha: que funciona y que aun no funciona](#alpha-que-funciona-y-que-aun-no-funciona)
5. [Que se puede hacer hoy](#que-se-puede-hacer-hoy)
6. [Que no se puede hacer todavia](#que-no-se-puede-hacer-todavia)
7. [Que NO es Anti-Gravital](#que-no-es-anti-gravital)
8. [Inicio rapido](#inicio-rapido)
9. [El Anti-DSL](#el-anti-dsl)
10. [Referencia de la CLI](#referencia-de-la-cli)
11. [Arquitectura](#arquitectura)
12. [Estructura del repositorio](#estructura-del-repositorio)
13. [Vision y principios de ingenieria](#vision-y-principios-de-ingenieria)
14. [Hoja de ruta y modelo de fases](#hoja-de-ruta-y-modelo-de-fases)
15. [Limitaciones conocidas y bloqueadores de release](#limitaciones-conocidas-y-bloqueadores-de-release)
16. [Mapa de documentacion](#mapa-de-documentacion)
17. [Contribuir y seguridad](#contribuir-y-seguridad)

### El proyecto de un vistazo

| | |
| --- | --- |
| Lenguaje y runtime | Rust (MSRV 1.99.1) sobre Tokio, Axum, Tower y rustls |
| Licencia | Apache-2.0 |
| Version publicada | Alpha `v0.1.0-alpha.2` disponible para pruebas desde el codigo fuente ([notas de release](docs/releases/0.1.0-alpha.2.md)); sin paquete en crates.io; la instalacion desde el fuente es la ruta verificada |
| Workspace | 19 crates `ag-*` mas la CLI de desarrollo `ag` (5 de ellos son placeholders reservados para fases posteriores) |
| Posicion actual | Fases 0-4.5 implementadas con sus puertas de salida aun abiertas; Fase aditiva 4.6 en curso |
| Listo para produccion | No. Es un alpha de prueba; la puerta formal de release pre-Fase 5 esta ABIERTA ([release gate](docs/audits/PRE_FASE5_RELEASE_GATE.md)) |
| Estado vivo | [docs/roadmap/STATUS.md](docs/roadmap/STATUS.md), a nivel de casilla, actualizado con cada PR de la hoja de ruta |

Las afirmaciones de estado de este README se verificaron el 2026-11-22 contra
`docs/roadmap/STATUS.md` (ultima actualizacion 2026-11-10). "Implementado"
significa que el codigo y sus tests existen en este repositorio y pasan CI;
nunca es una certificacion de produccion.

### Que es Anti-Gravital

Anti-Gravital ofrece a los equipos backend de Rust una experiencia de
framework coherente sin ocultar el ecosistema Rust subyacente. En lugar de
cablear a mano Axum, sqlx, tracing, JWT, rate limiting y generacion de codigo
en cada proyecto, se obtiene:

- **La Shield**: pipeline HTTP con seguridad primero en `ag-core` (TLS 1.3,
  HTTP/1.1 y HTTP/2, auth JWT, rate limiting, CORS, CSRF, validacion de
  payloads, logging estructurado) con defaults seguros.
- **El Anti-DSL**: lenguaje declarativo `.ag` (versiones v0.1-v0.8
  implementadas) para modelos, relaciones, validaciones, endpoints, politicas
  de auth, eventos, correo, dominios, templates y workers en segundo plano.
- **Generacion de codigo**: un schema produce handlers y tipos Rust,
  migraciones SQL, tipos y cliente TypeScript, OpenAPI 3.1 y AsyncAPI 2.6.
- **La CLI `ag`**: scaffolding, ciclo de desarrollo, builds de release,
  flujos de schema, verificacion de correo, operaciones de dominios y gestion
  de workers.
- **Crates modulares**: auth, datos, cache, realtime, storage,
  observabilidad, correo transaccional, jobs en segundo plano, gestion
  DNS/TLS de dominios y edge routing — cada crate se selecciona de forma
  independiente; nada es obligatorio mas alla de `ag-core`.

El resultado apunta a despliegues pequenos y auditables: el ejemplo de
referencia `todo-api` compila a un binario estatico MUSL de 5.3 MB y a una
imagen Docker `FROM scratch` de 2.49 MB (medido el 2026-05-21).

### Probarlo

`v0.1.0-alpha.2` es un alpha de prueba, no para produccion. Instala desde el
codigo fuente, crea un proyecto, ejecutalo localmente y genera codigo desde un
schema:

```bash
# 1. Instalar (se requieren Git y Rust 1.99.1+)
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh

# 2. Crear un proyecto
ag new mi-api --template rest

# 3. Ejecutar localmente (vease la limitacion de bind mas abajo)
cd mi-api && ag dev

# 4. Validar un schema y generar artefactos (dentro del proyecto)
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

En Windows se instala con `.\install.ps1` (PowerShell) desde el repo clonado;
los comandos `ag` son identicos. Antes de ejecutar un instalador, verifica el
checkout con el [procedimiento canonico de integridad](docs/security/INSTALLATION_INTEGRITY.md).
Guia completa: [Tu primera API](docs/manual/02-primera-api.md). Prototipos
ejecutables: [`examples/`](examples/). Notas de release:
[`v0.1.0-alpha.2`](docs/releases/0.1.0-alpha.2.md).

### Alpha: que funciona y que aun no funciona

Funciona en `v0.1.0-alpha.2`:

- Instalar desde el fuente con `install.sh`, `install.ps1` o `cargo install --path`.
- Crear proyectos `rest`, `realtime` y `fullstack` con `ag new`.
- Ejecutar un servidor de desarrollo local con `ag dev` (el default de la CLI
  es `127.0.0.1:8080`; los templates actuales enlazan todas las interfaces,
  issue #191).
- Validar un schema `.ag` y generar Rust, SQL, TypeScript y OpenAPI con
  `ag schema lint` y `ag generate`.
- Compilar un binario de release con `ag build`; inspeccionar y validar la
  configuracion con `ag config`.
- Renderizar artefactos Docker/Kubernetes autorizados localmente con
  `ag deploy plan --target docker|k8s`, revisar el id del plan y escribir los
  archivos con `ag deploy apply --plan ID` (alpha, solo local: sin cluster ni
  runtime).

Aun no funciona en el alpha:

- No hay paquete en crates.io. Se producen binarios para cuatro targets y un
  canal opt-in (`AG_INSTALLER_CHANNEL=alpha`) descarga uno con verificacion
  SHA-256 unicamente; no afirma firma ni autenticidad y su verificacion
  integral aun esta pendiente (#141 sigue abierto). La ruta verificada es la
  instalacion desde el fuente.
- No expongas el servidor de `ag dev` a redes no confiables: el default de la
  CLI es loopback, pero los templates generados fijan `0.0.0.0:8080` e ignoran
  la variable `BIND` (issue #191).
- `ag deploy` solo renderiza artefactos locales; el despliegue en cluster con
  `ag-cloud`, ademas de `ag migrate` y `ag plugin`, pertenece a fases futuras.
- No hay servidor MCP ni interfaz de maquina compartida: no se cumplen los
  prerequisitos de RFC-0024.
- No hay afirmacion de listo para produccion: la puerta pre-Fase 5 esta
  abierta y las puertas de rendimiento y de fuzzing de 24 horas estan pendientes.
- Los handlers Rust generados son stubs; la aplicacion aporta la logica.
- Las APIs publicas pueden cambiar antes de 1.0; aun no hay promesa de
  estabilidad SemVer.

### Que se puede hacer hoy

Todo lo siguiente existe en este repositorio, compila y esta cubierto por
tests en CI:

- Servir APIs HTTPS a traves del pipeline Shield de `ag-core` con todas las
  capas de seguridad activas (Fase 1).
- Usar extractores y respuestas tipadas, pools PostgreSQL y migraciones
  embebidas con `ag-data`, y crear proyectos con tres templates: `rest`,
  `realtime`, `fullstack` (Fase 2).
- Escribir schemas `.ag` y generar artefactos Rust, SQL, TypeScript, OpenAPI
  y AsyncAPI, con diagnostics legibles, un servidor LSP (`ag-lsp`) y una
  extension de VS Code empaquetada (Fase 3).
- Usar los modulos estandar (Fase 4):
  - `ag-auth`: WebAuthn/FIDO2, OAuth2 PKCE, JWT Ed25519, API keys, refresh
    tokens.
  - `ag-cache`: cache L1 en proceso mas un servidor L2 RESP2 nativo, con un L2
    Redis externo opcional para despliegues multi-instancia (feature `redis-l2`).
  - `ag-realtime`: bus de eventos en proceso, helpers WebSocket/SSE, cliente
    NATS externo opcional.
  - `ag-storage`: store de filesystem, procesamiento de imagenes, URLs
    firmadas, backend compatible con S3 opcional.
  - `ag-observe`: tracing estructurado, metricas Prometheus, base OTLP.
- Enviar correo transaccional (`ag-mail`: relay SMTP, templates, flujos de
  auth para verificacion/recuperacion/magic links) y gestionar registros DNS,
  certificados ACME/Let's Encrypt y SPF/DKIM/DMARC (`ag-domains`) (Fase 4.5).
- Ejecutar el MTA outbound nativo opt-in y los webhooks firmados de `ag-mail`
  (features `mta`/`api`, Fase 4.6-A/B/C). La cola de entrega del MTA admite un
  spool durable opt-in en PostgreSQL (feature `queue-postgres`) para que los
  jobs programados sobrevivan a un reinicio; el nivel en memoria sigue siendo el
  default.
- Ejecutar jobs en segundo plano con `ag-workers` (Fase 4.6-D): jobs tipados,
  reintentos con backoff, dead-letter queue, scheduling por intervalo y
  worker pools. El backend en memoria es el default; el backend PostgreSQL
  durable es opt-in y se verifico contra una PostgreSQL 16 viva (sus tests de
  integracion permanecen `#[ignore]` porque el CI por defecto no levanta base
  de datos).

Hay diez ejemplos ejecutables en [`examples/`](examples/), desde `todo-api`
(CRUD contra PostgreSQL) hasta `workers-postgres` y `auth-mail-demo`. Las
fases posteriores de la hoja de ruta son aditivas: ninguna es requisito para
usar las capacidades anteriores.

### Que no se puede hacer todavia

Limites a tener en cuenta antes de adoptar el proyecto:

- **Sin despliegue en cluster.** `ag deploy` (RFC-0034) solo copia artefactos
  Docker/Kubernetes autorizados a `deploy/generated/`; no ejecuta contenedores
  ni hace llamadas API. Secretos, despliegue real, logs y rollback llegan con
  `ag-cloud` en la Fase 5. `ag migrate` (Fase 7) y `ag plugin` (Fase 9)
  tambien son comandos futuros.
- **Sin instalacion via gestor de paquetes.** Nada esta publicado en
  crates.io todavia; se compila desde el fuente con los instaladores
  auditables.
- **Sin promesa de API estable.** Las APIs publicas pueden cambiar hasta la
  Fase 10 (1.0). La disciplina SemVer comienza con el primer release
  etiquetado.
- **Sin certificacion de produccion.** La puerta pre-Fase 5 esta abierta;
  los targets de rendimiento y la puerta de fuzzing de 24 horas siguen
  pendientes (vease [Limitaciones conocidas](#limitaciones-conocidas-y-bloqueadores-de-release)).
- **Crates placeholder.** `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate` y
  `ag-wasm-host` existen solo para reservar nombres y limites de modulo;
  su implementacion comienza en las Fases 5-9. `ag-ui` fue retirado del
  roster por `RFC-0019`.

### Que NO es Anti-Gravital

Anti-Gravital se integra con las herramientas dominantes en lugar de
reemplazarlas. No reimplementa PostgreSQL, Redis, NATS, object storage,
Docker, Kubernetes, Terraform, Flutter ni frameworks frontend. `ag-mail`
envia correo transaccional; no aloja buzones IMAP/POP. `ag-domains` gestiona
flujos DNS/TLS; no es un registrador de dominios. Tampoco renderiza interfaces
de usuario (`RFC-0019`): los consumidores de UI reciben SDKs cliente generados.
Los handlers Rust generados son stubs deliberados propiedad de cada aplicacion
— el framework nunca oculta la logica de negocio.

### Inicio rapido

Requisitos: Git y Rust 1.99.1 o superior.

Linux o macOS:

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git && cd Anti-Gravital && bash install.sh
```

Windows PowerShell:

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git; Set-Location Anti-Gravital; .\install.ps1
```

El instalador verifica la version de Rust, compila el workspace en release e
instala `ag` en el directorio binario de Cargo. No realiza cambios
privilegiados del sistema. Antes de ejecutarlo, verifica el checkout con el
[procedimiento canonico de integridad](docs/security/INSTALLATION_INTEGRITY.md).

Despues, crear y ejecutar un proyecto. En una terminal, `ag new` sin
argumentos abre un asistente guiado que pide el nombre y el template y muestra
lo que va a escribir antes de escribirlo:

```bash
ag new
```

Lo mismo sin ninguna pregunta, para scripts y CI:

```bash
ag new mi-api --template rest
cd mi-api
ag dev
```

Guia completa: [Tu primera API con Anti-Gravital](docs/manual/02-primera-api.md).

### Templates (plantillas)

Usa `ag new <nombre> --template <nombre> --no-interactive` para crear un
proyecto desde una plantilla del catalogo. La
[galeria de plantillas](docs/templates/GALLERY.md) es la tabla de decision de
caso de uso a plantilla, requisitos externos y comando de ejemplo;
[docs/templates/README.md](docs/templates/README.md) es su indice. El binario
alpha actual acepta `rest`, `realtime`, `fullstack` y `layered`; los demas
nombres de la ola 6 llegan con el registro del catalogo y los directorios de
plantillas.

Para inspeccionar o cambiar la configuracion, ejecuta `ag config` para abrir
el formulario interactivo o usa sus equivalentes por script:

```bash
ag config get bind
ag config set rate_limit.enabled true
ag config validate
```

Cada escritura se valida con el mismo loader `ShieldConfig` que usa el runtime
y el archivo se reemplaza de forma atomica. `ag` sin argumentos abre una
pantalla de inicio que lanza estos comandos sin realizar trabajo por si misma.

### El Anti-DSL

El schema `.ag` es la fuente de verdad de un proyecto. Extracto abreviado de
[`examples/ecommerce-api/schema.ag`](examples/ecommerce-api/schema.ag):

```text
config {
    project_name "ecommerce-api"
    database "postgres"
}

model Product {
    id          UUID      @primary @auto
    name        String    @min(2) @max(200)
    price       Decimal   @min(0)
    category_id UUID      @references(Category.id)
    category    Category  @relation(category_id)
    created_at  Timestamp @auto
}

endpoint GetProduct {
    method   GET
    path     /products/{id}
    response ProductResponse
    errors   [NotFound]
}
```

Validar y generar:

```bash
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

El generador escribe un modulo Rust, una migracion SQL, tipos y cliente
TypeScript, OpenAPI y artefactos AsyncAPI opcionales. Los cuerpos de los
handlers generados son stubs deliberados que implementa la aplicacion. La
validacion Rust de `@regex` es ejecutable y cachea cada patron compilado; los
proyectos generados que la usen deben declarar `regex = "1"`. Versiones del
DSL implementadas: v0.1 (modelos) hasta v0.8 (declaraciones `worker`).
Referencia completa: [docs/dsl/](docs/dsl/).

### Referencia de la CLI

| Comando | Proposito |
| --- | --- |
| `ag` (sin argumentos) | En una terminal interactiva abre una pantalla de inicio que lanza comandos existentes; en cualquier otro caso imprime la ayuda |
| `ag new [NOMBRE] [--template TIPO]` | Crear un proyecto (`rest`, `realtime`, `fullstack`). Si falta un dato y la terminal es interactiva se abre un asistente guiado; con `--no-interactive`, `CI` o sin terminal nunca pregunta |
| `ag config` | En una terminal abre un formulario a pantalla completa para `config.toml`. Equivalentes no interactivos: `ag config get CLAVE`, `ag config set CLAVE VALOR` y `ag config validate` (este ultimo sale con codigo distinto de cero si el archivo es invalido, util en CI). Los secretos nunca se muestran; solo rutas de clave o nombres de variable de entorno |
| `ag dev [--bind DIR]` | Modo desarrollo; el default de la CLI es `127.0.0.1:8080`. Advertencia: los templates generados fijan `0.0.0.0:8080` e ignoran `BIND` (issue #191), por lo que el servidor escucha en todas las interfaces hasta corregirlo. Usa cargo-watch si esta instalado |
| `ag build [--target TRIPLE]` | Compilar un binario release |
| `ag generate --schema schema.ag --output generated` | Generar artefactos del DSL |
| `ag schema lint` | Validar un schema y reportar diagnostics |
| `ag schema diff REFERENCIA` | Clasificar cambios de schema |
| `ag mail test --to DIRECCION` | Verificar la configuracion SMTP |
| `ag domains check --domain HOST` | Verificar propagacion DNS |
| `ag domains sync --zone-id ID` | Aplicar los registros DNS del schema via el proveedor configurado |
| `ag domains attach\|instructions\|export-zone\|status\|list\|verify\|detach\|diagnose` | Operar el flujo local de adjuntar dominios |
| `ag workers list` | Listar los workers declarados en un schema |
| `ag workers run` | Ejecutar un proceso worker independiente contra el backend configurado |
| `ag workers enqueue KIND --payload FILE` | Encolar un job en el backend durable (requiere `DATABASE_URL`) |
| `ag workers queues` | Mostrar profundidad de colas en el backend durable |
| `ag workers dlq list\|inspect\|retry\|purge` | Inspeccionar y gestionar la dead-letter queue |
| `ag workers doctor` | Verificar configuracion de workers y conectividad del backend durable |
| `ag deploy plan --target docker\|k8s` | Imprimir un plan determinista de solo lectura (archivos, tamanos, huellas e id) a partir de los artefactos autorizados (alpha, solo local) |
| `ag deploy apply --plan ID` | Escribir ese plan exacto en `deploy/generated/`; rechaza ids mal formados o obsoletos |
| `ag deploy status` | Informar de los archivos generados y de si el plan aplicado sigue vigente |

El flag global `--output json` produce salida versionada legible por maquinas
para los comandos de solo lectura `ag schema lint`, `ag schema diff`,
`ag workers list`, `ag deploy plan` y `ag deploy status` (contrato `0.2.0`,
documentado en [docs/mcp/](docs/mcp/)). Los demas comandos, incluido
`ag deploy apply`, lo rechazan antes de actuar; la salida en prosa no cambia
cuando el flag esta ausente.

Ejecuta `ag COMANDO --help` para los flags y variables de entorno
autoritativos. `ag deploy` solo renderiza artefactos locales (RFC-0034);
`ag migrate` y `ag plugin` son comandos futuros y no estan en el binario
actual. Flujos detallados:
[guia de `ag-cli`](crates/ag-cli/README.md) y
[referencia CLI de dominios](docs/ag-domains/reference/cli.md).

### Arquitectura

El workspace es un workspace Cargo estricto y por capas. `ag-core` no depende
de ningun otro crate Anti-Gravital; el resto de crates es opt-in; las
dependencias circulares estan prohibidas y se verifican en CI. El diagrama de
la seccion inglesa ([Architecture](#architecture)) es el canonico y aplica
igualmente aqui.

Los niveles siguen la clasificacion canonica de [`CLAUDE.md`](CLAUDE.md) §14
y el capitulo 5 del maestro de arquitectura. `*` marca un crate placeholder
cuya implementacion comienza en una fase posterior.

| Nivel | Crates | Rol |
| --- | --- | --- |
| Nucleo | `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host`* | Shield/runtime, compilador DSL, CLI de desarrollo, LSP de editor, host de plugins WASI |
| Estandar | `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe` | Auth, capa de datos PostgreSQL, eventos, cache, archivos/imagenes, telemetria |
| Estandar diferido | `ag-mail`, `ag-workers` | Correo transaccional y MTA nativo; motor de ejecucion de jobs. Madurez de estandar, no instalados por defecto en los templates |
| Opcional | `ag-cloud`*, `ag-ai`*, `ag-mobile`*, `ag-migrate`* | Deploy cloud, IA/knowledge graph, bridge mobile, importadores |
| Opcional infra | `ag-domains`, `ag-edge` | Plano de control DNS/dominios/TLS y plano de datos edge en tiempo de request |

Toda integracion con un servicio externo (NATS, S3, Cloudflare, SMTP de
terceros) vive detras de una feature de Cargo con un modo nativo por defecto,
de modo que ninguna dependencia externa es requisito para usar un modulo
(ADR-0009).

### Estructura del repositorio

```text
anti-gravital/
|-- crates/        19 crates ag-* (workspace Cargo; ver mapa de arquitectura)
|-- examples/      10 proyectos de ejemplo ejecutables
|-- templates/     templates de proyecto usados por `ag new`
|-- benchmarks/    harnesses de benchmark y metodologia
|-- fuzz/          targets de cargo-fuzz (lexer, parser, compile, payload de workers)
|-- tests/         tests de integracion cross-module
|-- tools/         tooling del repositorio
|-- docs/          toda la documentacion (ver mapa de documentacion)
|-- install.sh     instalador desde fuente (Linux/macOS); install.ps1 para Windows
`-- CLAUDE.md      la constitucion tecnica que gobierna todo el trabajo
```

### Vision y principios de ingenieria

Anti-Gravital se construye como infraestructura real, sostenible y
tecnicamente defendible — nunca como una demo inflada por hype. Los
principios no negociables (codificados en [`CLAUDE.md`](CLAUDE.md) y los
documentos maestros):

- **Documentacion primero.** La arquitectura se documenta antes de
  implementarse; si el codigo contradice la documentacion, el codigo esta
  mal. Las decisiones grandes requieren RFC y las aceptadas persisten como
  ADR.
- **Integracion sobre reinvencion.** Donde existe una herramienta dominante
  (PostgreSQL, Kubernetes, Terraform, ...), Anti-Gravital se integra con
  ella.
- **Seguridad por construccion.** `unsafe` esta denegado en todo el
  workspace, los defaults son seguros y `cargo audit`/`cargo deny`/`clippy
  -D warnings` bloquean el CI.
- **Evidencia honesta.** Los benchmarks publican hardware, metodologia y
  desviacion estandar; los targets no alcanzados se registran como no
  alcanzados (vease la Fase 2 en la hoja de ruta como ejemplo real).
- **Simplicidad operacional.** Binarios estaticos, configuracion explicita,
  builds deterministas, observabilidad nativa, runtime minimo.
- **Disciplina de fases.** Nada se construye antes de su fase; sin
  abstracciones especulativas.

### Hoja de ruta y modelo de fases

Como leer la hoja de ruta:

- **Fases bloqueantes (0-10)** se ejecutan en orden. Cada documento de fase
  bajo [`docs/roadmap/`](docs/roadmap/) define criterios de entrada,
  entregables y criterios de salida (la "puerta" o gate). Una fase cuenta
  como **implementada** cuando sus entregables existen con tests en este
  repositorio, y como **cerrada** solo cuando todos los criterios de salida
  — incluida la evidencia de rendimiento y los criterios externos de
  comunidad — estan verificados. La implementacion de las Fases 1+ avanza en
  paralelo con los criterios externos pendientes de la Fase 0 bajo la
  excepcion documentada RFC-0001.
- **Fases aditivas (4.5, 4.6)** amplian el ecosistema entre fases
  bloqueantes. Se autorizan por ADR, nunca son requisito de las capacidades
  anteriores y nunca se adelantan a sus propias RFC.
- **Fases futuras** siguen el mismo mecanismo: la siguiente fase abre solo
  cuando cierra la puerta anterior, todo crate nuevo o cambio de alcance
  requiere primero una RFC, y no se hace ninguna afirmacion de produccion/GA
  mientras una puerta este abierta. Las duraciones de los documentos de fase
  son estimaciones de planificacion, no promesas de fecha.

Estado actual, fase por fase (evidencia en
[docs/roadmap/STATUS.md](docs/roadmap/STATUS.md)):

| Fase | Alcance | Estado | Que mantiene la puerta abierta |
| --- | --- | --- | --- |
| 0 | Fundaciones: gobernanza, Apache-2.0, monorepo, CI en cuatro plataformas, constitucion tecnica | En curso | Entregables externos: branding, canales de comunidad, landing page, calendario publico de releases |
| 1 | Shield MVP: HTTP/1.1+2, TLS 1.3, JWT, rate limiting, CORS, CSRF, validacion, logging estructurado | Implementada; puerta abierta | Targets en hardware de referencia (>= 300K req/s hello-world, p99 <= 1 ms), medicion oficial de cobertura, criterios externos |
| 2 | Core MVP: extractores/respuestas tipadas, capa PostgreSQL, migraciones, scaffolds, ejemplo CRUD | Implementada; puerta abierta | Targets CRUD no alcanzados en el hardware registrado: medidos 14 478 req/s frente al objetivo de 40K y p99 de 14.6 ms frente a 5 ms en un Ryzen 5 2500U; analisis completo publicado en [docs/benchmarks/](docs/benchmarks/) |
| 3 | Anti-DSL v0.1-v0.4: parser, diagnostics, generadores Rust/SQL/TypeScript/OpenAPI, LSP, extension VS Code | Implementada; puerta abierta | Puerta de fuzzing de 24 horas, benchmark directo generado-vs-manual, consolidacion del generador (issue #70), criterios de adopcion |
| 4 | Modulos estandar: `ag-auth`, `ag-cache`, `ag-realtime`, `ag-storage`, `ag-observe`; DSL v0.5-v0.6 | Implementada; puerta abierta | Releases en crates.io, benchmarks de escala (50K conexiones WebSocket, 1M ops/s de cache), criterios de comunidad |
| 4.5 (aditiva) | Correo transaccional `ag-mail`; `ag-domains` DNS/ACME/SPF-DKIM-DMARC; DSL v0.7 | Implementada; puerta abierta | Reejecucion de la puerta sobre el commit final de consolidacion; `ag-domains` tiene trabajo activo |
| 4.6 (aditiva) | Endurecimiento pre-Fase 5: MTA outbound nativo + webhooks firmados en `ag-mail` (A/B/C); motor `ag-workers` (D); DSL v0.8 | En curso | Wiring del modo producer en `ag-edge` (issue #112); evidencia de entrega en vivo del MTA (issue #153). El spool durable del MTA ya se entrega tras la feature `queue-postgres` (issue #151; su test con PostgreSQL vivo es `#[ignore]`). Las etapas S1-S5 y la migracion S7 de `ag-mail` estan hechas; el backend PostgreSQL se verifico contra una base viva (issues #108/#109/#103) |
| 4.7 (aditiva) | Experiencia interactiva de terminal: TUI a pantalla completa en el binario `ag` para crear y configurar un proyecto, con equivalentes no interactivos exactos | En curso | `RFC-0018` aceptada. Etapas A-B hechas: asistente de `ag new`, `--no-interactive`, `ag config` get/set/validate mas el formulario a pantalla completa con validacion antes de escribir. Etapa C hecha: pantalla de inicio de `ag` sin argumentos, con navegacion por teclado que solo lanza comandos existentes. Las etapas A-B son requisito de la v0.5 y la etapa C de la 1.0 |
| 4.8 (aditiva) | Fundaciones del proyecto: estructura por capas, entorno declarado y secretos cifrados, instalacion verificada con un comando, consola remota sobre SSH, interfaz legible por maquina e interfaz MCP compartida por capacidades | Aceptada, sin iniciar | `RFC-0019`, `RFC-0020` y `RFC-0021` aceptadas el 2026-11-10; `RFC-0024` aceptada el 2026-11-21; sin codigo aun |
| 5 | `ag-cloud`: build/deploy simplificados, secretos, logs, rollback, dominios, TLS | Pendiente | Abre cuando cierre la puerta pre-Fase 5. Hito: beta publica v0.5 |
| 5.5 (aditiva) | Funciones serverless: funciones WebAssembly aisladas, desplegadas de forma independiente, con capacidades explicitas, disparadas por HTTP, schedule, cola o eventos | Aceptada, sin iniciar | `RFC-0023` aceptada el 2026-11-10; sin codigo aun |
| 6 | `ag-ai` y Knowledge Graph: providers, retrieval, flujos asistidos por grafo | Pendiente | Cierre de la Fase 5 y feedback de la beta |
| 7 | `ag-migrate`: importadores y migracion asistida desde otros frameworks backend | Pendiente | Cierre de la Fase 6 y tests de aceptacion de importadores |
| 8 | `ag-mobile`: bridge Flutter/Dart, clientes generados, contratos offline | Pendiente | Cierre de la Fase 7 y puertas de compatibilidad mobile |
| 9 | Plugins WASI: extensiones aisladas, hooks de ciclo de vida, permisos, registry | Pendiente | Cierre de la Fase 8 y revision de seguridad |
| 10 | Endurecimiento y 1.0: API/DSL estable, auditoria de seguridad, rendimiento, docs, LTS | Pendiente | Cierre de la Fase 9 y puertas de release 1.0. Hito: 1.0 estable |

Hitos estimados del plan original: beta publica v0.5 al cierre de la Fase 5
(alrededor del mes 15) y 1.0 estable al cierre de la Fase 10 (alrededor del
mes 30). Son estimaciones, no compromisos. La decision autoritativa de
release vive en
[docs/audits/PRE_FASE5_RELEASE_GATE.md](docs/audits/PRE_FASE5_RELEASE_GATE.md);
los documentos completos de fase y el calendario viven en
[docs/roadmap/](docs/roadmap/README.md).

### Limitaciones conocidas y bloqueadores de release

Dichas sin rodeos, porque ocultarlas violaria las propias reglas del
proyecto:

- La puerta de fuzzing de 24 horas (Fase 3) aun no se ha ejecutado; el
  harness de fuzzing corre un smoke test de 60 segundos en CI.
- Falta la evidencia final de rendimiento sobre codigo estabilizado; los
  targets publicados de las Fases 1 y 2 no se han alcanzado en el hardware
  registrado.
- Los handlers Rust generados son stubs intencionales; el framework no
  escribe la logica de negocio.
- `ag-domains` tiene desarrollo activo en curso.
- Los caminos durables PostgreSQL de `ag-workers` y de la entrega de `ag-mail`
  respaldada por workers se verificaron manualmente contra una PostgreSQL 16
  viva; sus tests de integracion siguen `#[ignore]` porque el CI por defecto no
  provisiona base de datos. El spool de la cola programada del MTA nativo
  (feature `queue-postgres`, issue #151) anade su propio test PostgreSQL
  `#[ignore]`, pendiente de una ejecucion en vivo; su mecanismo de durabilidad
  en memoria esta cubierto por tests que pasan.
- La deuda tecnica abierta se rastrea como GitHub Issues (etiqueta
  `tech-debt`); [docs/DEBT.md](docs/DEBT.md) es un registro historico
  congelado.
- La puerta de release debe reevaluarse sobre el commit final de
  consolidacion antes de cualquier afirmacion de produccion.

### Mapa de documentacion

Orden de lectura para recien llegados — humanos o automatizados. Un agente de
IA que trabaje en este repositorio debe leer primero [`CLAUDE.md`](CLAUDE.md):
es la constitucion tecnica vinculante, y la documentacion tiene precedencia
sobre el codigo.

| Recurso | Que aporta |
| --- | --- |
| [`CLAUDE.md`](CLAUDE.md) | La constitucion tecnica: reglas de gobernanza, limites entre crates, flujo de trabajo, puertas de calidad |
| [`docs/INDEX.md`](docs/INDEX.md) | Indice maestro de toda la documentacion |
| [`docs/master/`](docs/master/) | Los tres documentos maestros (Blueprint, Arquitectura Tecnica, Hoja de Ruta) — la fuente de verdad |
| [`docs/releases/0.1.0-alpha.2.md`](docs/releases/0.1.0-alpha.2.md) | Alcance, limitaciones conocidas y evidencia de verificacion del alpha |
| [`docs/roadmap/STATUS.md`](docs/roadmap/STATUS.md) | Estado vivo a nivel de casilla de cada fase |
| [`docs/architecture/`](docs/architecture/) | Capitulos de arquitectura derivados de los maestros |
| [`docs/rfc/`](docs/rfc/) y [`docs/adr/`](docs/adr/) | Propuestas de diseno y decisiones aceptadas |
| [`docs/manual/`](docs/manual/) | Capitulos del manual de usuario (Shield como libreria, primera API, dominios/TLS/correo) |
| [`docs/dns/`](docs/dns/README.md) | Guia bilingue del circulo dominio-despliegue: adjunto, verificacion, entornos, publico/privado, resolucion de problemas |
| [`docs/modules/`](docs/modules/) | Documentacion por crate |
| [`docs/benchmarks/`](docs/benchmarks/) | Benchmarks medidos con hardware y metodologia |
| [`docs/graph/`](docs/graph/) | Knowledge graph estructurado de modulos, comandos y relaciones |
| [`examples/`](examples/) | Diez proyectos de ejemplo ejecutables |

### Contribuir y seguridad

Lee [CONTRIBUTING.md](CONTRIBUTING.md), [CLAUDE.md](CLAUDE.md),
[SECURITY.md](SECURITY.md) y [GOVERNANCE.md](GOVERNANCE.md) antes de abrir un
cambio. Todo contribuidor debe poder hacer `git clone`, `cargo build` y
`cargo test` sin depender del mantenedor. Las vulnerabilidades se reportan
por el proceso de [SECURITY.md](SECURITY.md), nunca por issues publicos.

---

## License / Licencia

Apache License 2.0. See [LICENSE](LICENSE).

Project initiated by Angel Nereira under Gravital Labs / Nereira Technology and Business Solutions.
- No `ag deploy`, `ag migrate` or `ag plugin` (future phases).
- **No deployment story.** `ag deploy`, secrets, logs and rollback arrive
  (Phase 9) are also future commands; the current binary does not include
  them.
| `ag dns plan [--manifest PATH] [--env ENV]` | Produce a deterministic, offline record plan from the `[domain]` table of `ag.toml` |
| `ag dns apply --plan ID [--env ENV] [--provider NAME]` | Apply a reviewed plan to local state; a provider adapter runs only when explicitly named |
| `ag dns status [--env ENV]` | Read-only report of the last applied plan and whether it is still in sync |
| `ag domains attach [--manifest PATH] [--allow-wildcard]` | Attach a domain to a project from the manifest or flags, printing exact records (no credentials) |
`ag dns plan` and `ag dns status` (contract `0.2.0`, documented in
[docs/mcp/](docs/mcp/)). Other commands refuse it before doing any work; prose
output is unchanged when the flag is absent.
`ag deploy`, `ag migrate` and `ag plugin` are future commands and are not in
the current binary. Detailed workflows: [`ag-cli` guide](crates/ag-cli/README.md)
and [domain CLI reference](docs/ag-domains/reference/cli.md).
- No existen `ag deploy`, `ag migrate` ni `ag plugin` (fases futuras).
- **Sin historia de despliegue.** `ag deploy`, secretos, logs y rollback
  llegan con `ag-cloud` en la Fase 5. `ag migrate` (Fase 7) y `ag plugin`
  (Fase 9) tambien son comandos futuros; el binario actual no los incluye.
| `ag dns plan [--manifest RUTA] [--env ENV]` | Produce un plan de registros determinista y sin red desde la tabla `[domain]` de `ag.toml` |
| `ag dns apply --plan ID [--env ENV] [--provider NOMBRE]` | Aplica un plan revisado al estado local; el adaptador solo se ejecuta si se nombra explicitamente |
| `ag dns status [--env ENV]` | Informe de solo lectura del ultimo plan aplicado y de si sigue sincronizado |
| `ag domains attach [--manifest RUTA] [--allow-wildcard]` | Adjunta un dominio al proyecto desde el manifest o flags e imprime los registros exactos (sin credenciales) |
`ag workers list`, `ag dns plan` y `ag dns status` (contrato `0.2.0`,
documentado en [docs/mcp/](docs/mcp/)). Los demas comandos lo rechazan antes
de actuar; la salida en prosa no cambia cuando el flag esta ausente.
autoritativos. `ag deploy`, `ag migrate` y `ag plugin` son comandos futuros y
no estan en el binario actual. Flujos detallados:
