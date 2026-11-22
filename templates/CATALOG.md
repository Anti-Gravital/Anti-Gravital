# Template catalog

This file is the authoritative, human-readable registry of the starting
templates offered by `ag new`. The machine-readable source of truth is
`crates/ag-cli/src/catalog.rs`; the clap value parser, the interactive selector
and the scaffolder all read from it, so a name accepted on the command line is
always a name the selector can show.

The default template is `rest`. It is the first entry, the value used when
`--template` is omitted in a non-interactive session, and the empty-answer
default of the interactive selectors. Any name not listed here is a usage
error (exit code 2), never a silent fallback.

## The `template.toml` contract

Every template directory contains a `template.toml` at its root. The file is
metadata only: it is never copied into a generated project. The `ag` binary
discovers template directories at build time (`crates/ag-cli/build.rs`), so a
new directory becomes usable after the next build without a CLI code change.

Minimal contract (all seven wave-6 template roles conform):

```toml
name = "static-site"
description = "One-line purpose shown in the selector."
category = "web"
external_requirements = []   # e.g. ["DATABASE_URL", "AI_API_KEY"]
```

| Field | Type | Required | Allowed values / rule |
| --- | --- | --- | --- |
| `name` | string | yes | Must equal the directory name and the catalog name. Lowercase ASCII letters, digits and hyphens only. |
| `description` | string | yes | One line, ASCII, no trailing period required. Shown next to the name in the selector. |
| `category` | string | yes | One of `web`, `api`, `realtime`, `infrastructure`, `ai`, `testing`. |
| `external_requirements` | array of strings | yes | Environment variables or services needed for the full runtime path. Use `[]` when the template runs fully offline. |

Extensions used by this catalog (optional, ignored by the minimal validator):

| Field | Type | Meaning |
| --- | --- | --- |
| `offline_check` | string | Command a validator can run to confirm the scaffold builds offline, e.g. `cargo check --offline`. |
| `limitations` | array of strings | Known boundaries to print in the template `README.md`. |

File and substitution conventions:

- Files ending in `.tmpl` are written without that suffix (`src/main.rs.tmpl`
  becomes `src/main.rs`). Files without the suffix are copied verbatim.
- `{{name}}` is replaced with the project name in every file. It is the only
  substitution; no template engine is involved.
- Subdirectories are preserved. `template.toml` is skipped. `README.md`, when
  present, is copied into the generated project and must contain run commands
  and limitations.
- A template must scaffold and `cargo check` offline using the shared target
  directory before it is declared usable.

## Catalog

External requirements are the variables or services needed for the full
runtime path; templates with `[]` run offline. The final per-template values
are those declared in each directory's `template.toml` when it is merged.

### Core (owner: catalog)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `rest` | api | Minimal REST API on ag-core (Shield + axum) with one example route. | none |
| `realtime` | realtime | WebSocket server on ag-core with a `/ws` echo route. | none |
| `fullstack` | api | REST API with PostgreSQL persistence through ag-data and SQL migrations. | `DATABASE_URL` |
| `layered` | api | Opt-in layered workspace (domain/app/infra/api) with an `ag.toml` manifest. | none |

### Web (owner: web)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `static-site` | web | Static HTML/CSS/JS site served by a minimal axum handler. | none |
| `marketing-site` | web | Marketing landing page with hero, features and contact sections. | none |
| `blog` | web | Server-rendered blog with a post index and article view. | none |
| `docs-site` | web | Documentation site with navigation and search-ready structure. | none |
| `dashboard` | web | Analytics dashboard shell with layout and chart placeholders. | none |
| `admin-panel` | web | Admin panel with an auth guard placeholder and CRUD table layout. | none |

### API (owner: api)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `database-api` | api | REST API with a database-backed repository and migrations. | `DATABASE_URL` |
| `auth-service` | api | Authentication service with registration, login and JWT sessions. | `DATABASE_URL` |
| `multi-tenant-saas` | api | Multi-tenant SaaS API with tenant isolation scaffolding. | `DATABASE_URL` |
| `webhook-gateway` | api | Inbound webhook receiver with signature verification and retry queue. | none |
| `file-upload-service` | api | Multipart file upload API with a local storage adapter. | none |
| `ecommerce-lite` | api | Lightweight e-commerce API: catalog, cart and checkout stubs. | `DATABASE_URL` |

### Realtime (owner: realtime)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `realtime-chat-v2` | realtime | Chat server with rooms and WebSocket/SSE transport. | none |
| `websocket-service` | realtime | WebSocket service with typed messages and broadcast. | none |
| `worker-service` | realtime | Background worker host with typed jobs and an in-memory queue. | none |
| `scheduled-jobs` | realtime | Cron-like scheduled job runner built on ag-workers. | none |
| `iot-ingest` | realtime | Telemetry ingestion endpoint with batch buffering. | `NATS_URL` |
| `edge-router` | realtime | Edge routing policy scaffold using ag-edge hostname rules. | none |

### Infrastructure (owner: infra)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `mail-service` | infrastructure | Transactional mail service with native SMTP/MTA senders. | `AG_SMTP_HOST` |
| `dns-manager` | infrastructure | DNS record manager with a provider adapter and SPF/DKIM/DMARC output. | `AG_CLOUDFLARE_TOKEN` |
| `object-storage-api` | infrastructure | Object storage API with local and pluggable bucket adapters. | `S3_BUCKET` |
| `monitoring-service` | infrastructure | Metrics and health endpoint scaffold with OpenTelemetry wiring. | none |
| `microservices-starter` | infrastructure | Starter for a multi-service workspace with shared contracts. | none |
| `cli-tool` | infrastructure | Standalone CLI binary scaffold using clap and ag-core. | none |

### AI (owner: ai)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `ai-backend-template` | ai | AI backend with `/health` and a provider-agnostic `/chat` stub. | `AI_API_KEY` |
| `vector-search` | ai | Vector search API with embeddings and a similarity query stub. | `DATABASE_URL`, `AI_API_KEY` |
| `rag-pipeline` | ai | Retrieval-augmented generation pipeline scaffold. | `DATABASE_URL`, `AI_API_KEY` |
| `wasi-plugin` | ai | WASI plugin scaffold for ag-wasm-host extensions. | none |

### Testing (owner: qa)

| Name | Category | Purpose | External requirements |
| --- | --- | --- | --- |
| `testing-suite` | testing | Integration, fuzz and benchmark test harness scaffold. | none |

Total: 33 cataloged templates (24+ required by the wave-6 board).

## CLI behavior

- `ag new demo --template <name>` accepts every name in this file.
- Flags and interactive selection build the same `NewProjectRequest`; the
  command echoed after an interactive flow re-parses through the same parser.
- `ag new --no-interactive` is unchanged: it never opens a selector and a
  missing template resolves to `rest`.
- The full-screen selector draws a six-entry window that follows the cursor,
  with `Template N of 33` and `more above`/`more below` hints. Selection is
  clamped at both ends and never wraps. The line-mode selector lists all names
  and also accepts a number or a name.
- Unknown names are rejected by clap with `InvalidValue` before any work runs.

## Adding a template

1. Create `templates/<name>/` with `template.toml`, the template files and a
   `README.md`. Do not edit `crates/ag-cli/src/cli.rs` or
   `crates/ag-cli/src/commands/project.rs`.
2. Add one row to `crates/ag-cli/src/catalog.rs` (the `define_catalog!` list)
   and one row to the matching table above. Keep `rest` first.
3. Run `scripts/validate-templates.sh` (qa role) to scaffold and build every
   template offline with the shared target directory.
4. Update `CHANGELOG.md` and the gallery in `docs/templates/GALLERY.md`.
