# Crate map (human-readable mirror of knowledge-graph.json)

Curated view of the 20-crate workspace: classification, inter-crate
dependencies and role. Machine-readable source: `knowledge-graph.json`.
Diagram: `docs/diagrams/workspace-dependencies.md`.

## Classification (CLAUDE.md section 14)

| Class | Crates |
| --- | --- |
| Nucleo | `ag-core`, `ag-dsl`, `ag-cli`, `ag-lsp`, `ag-wasm-host` |
| Estandar | `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe` |
| Estandar diferido | `ag-mail`, `ag-workers` |
| Opcionales | `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate` |
| Opcionales de infraestructura | `ag-domains`, `ag-edge` |

## Crates and direct dependencies

| Crate | Class | Depends on (ag-*) | Role |
| --- | --- | --- | --- |
| `ag-core` | nucleo | (none) | Shield middleware pipeline, core router/extractors, `AgError`. |
| `ag-dsl` | nucleo | (none) | Anti-DSL compiler: lexer, parser, AST, semantics, diagnostics, codegen. |
| `ag-cli` | nucleo | `ag-dsl`, `ag-domains`, `ag-mail`, `ag-workers` | The `ag` binary: scaffold, dev, build, codegen, domains/mail/workers ops. |
| `ag-lsp` | nucleo | `ag-dsl` | LSP tooling for the DSL (Phase 3). |
| `ag-wasm-host` | nucleo | (none) | WASM host/runtime surface. |
| `ag-auth` | estandar | `ag-data`, `ag-mail` | JWT Ed25519, WebAuthn, OAuth, API keys. Consumes `ag-mail` (sixth rule). |
| `ag-data` | estandar | `ag-core` | PostgreSQL pool, config, migrations, the sanctioned sqlx boundary. |
| `ag-realtime` | estandar | (none) | Event bus / realtime; optional persistence and NATS adapter. |
| `ag-cache` | estandar | `ag-core` | L1 (moka) + native RESP2 server. Redis L2 deferred (issue #144). |
| `ag-storage` | estandar | `ag-auth` | Object store: filesystem default, HTTP server, optional S3, signed URLs. |
| `ag-observe` | estandar | (none) | Tracing + Prometheus metrics. OTLP deferred (issue #149). |
| `ag-mail` | estandar diferido | `ag-core`, `ag-workers` | Outbound email + native MTA. Does NOT depend on `ag-auth`. |
| `ag-workers` | estandar diferido | `ag-data` | Background jobs: retries, DLQ, scheduling. In-memory default, PostgreSQL opt-in. |
| `ag-ui` | opcional | (none) | UI surface. |
| `ag-cloud` | opcional | (none) | Deployment/cloud (Phase 5). Consumes `ag-domains` at `ag deploy` (seventh rule). |
| `ag-ai` | opcional | (none) | AI integration (Phase 6); future knowledge-graph generation. |
| `ag-mobile` | opcional | (none) | Mobile client surface. |
| `ag-migrate` | opcional | (none) | Migration tooling. |
| `ag-domains` | opcional infra | `ag-core` | DNS (`DnsProvider` + adapters), ACME, SPF/DKIM/DMARC. Not a registrar. |
| `ag-edge` | opcional infra | `ag-domains` | Edge data plane: hostname routing, SNI, canonical/redirect (ADR-0012). |

## Dependency rules (CLAUDE.md section 15)

- `ag-core` depends on no other Anti-Gravital crate.
- No circular dependencies.
- `ag-mail` does not depend on `ag-auth`; the reverse holds (sixth rule).
- `ag-workers` does not depend on `ag-edge`; the allowed producer direction
  is `ag-edge -> ag-workers` behind the `producer` feature (issue #112).
- `ag-domains` is consumed by `ag-cloud` during `ag deploy` (seventh rule).

## CLI commands

| Command | Effect |
| --- | --- |
| `ag new <name> [--template rest\|realtime\|fullstack]` | Scaffold a new project. |
| `ag dev` | Development server with optional hot reload. |
| `ag build` | Production build. |
| `ag generate` | Run DSL codegen over `schema.ag`. |
| `ag schema lint` | Lint/validate the DSL schema. |
| `ag run` | Run the built application. |
| `ag domains <attach\|instructions\|export-zone\|status\|list\|verify\|detach\|diagnose\|check\|sync>` | Domain control plane and DNS ops. |
| `ag mail test` | Send a test email. |
| `ag workers <enqueue\|queues\|doctor>` | Enqueue jobs and inspect the durable backend. |

## DSL artifacts (CLAUDE.md section 20)

- Pipeline: `schema.ag` -> lexer -> parser -> AST -> semantic analysis ->
  diagnostics -> codegen.
- Codegen targets: Rust, SQL, OpenAPI, TypeScript, Dart, migrations, SDKs,
  knowledge graph.
