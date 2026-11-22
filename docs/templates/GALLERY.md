# Template gallery

English canonical reference for the wave-6 project template catalog. This is a
discovery document, not a contract: the authoritative metadata for a template is
its own `template.toml` (`name`, `description`, `category`,
`external_requirements`).

Back to the [template index](README.md) and the
[documentation index](../INDEX.md).

## How to read the table

- **Use case**: the situation that should make you pick this template.
- **Template**: the value passed as `--template <name>` to `ag new`.
- **External requirements**: the minimum service or credential the full path
  needs. `None` means the template runs locally with no service, no credential
  and no broker. An entry such as `None (S3 optional)` means the default path
  is native and the external backend is opt-in.
- **Example command**: a non-interactive scaffold you can copy. Replace
  `<project>` with your project name.

Requirement vocabulary used below:

| Value | Meaning |
| --- | --- |
| `None` | No external service or credential for the full path |
| `PostgreSQL (DATABASE_URL)` | A reachable PostgreSQL instance is required |
| `SMTP endpoint` | An SMTP relay endpoint, with a native in-process MTA as an alternative |
| `DNS provider + ACME CA` | DNS provider credentials plus a certificate authority |
| `AI provider key` | A provider API key needed for live inference; optional when a local provider is the default |
| `Vector store` | An embeddings store, typically PostgreSQL with pgvector; optional when an in-memory repository is the default |

Entries marked `confirm` are inferred from the wave-6 catalog plan and the
documented crate architecture, not from a merged `template.toml`. Check them
against the manifest after integration.

### Availability caveat

The current alpha `ag` binary accepts only `rest`, `realtime`, `fullstack` and
`layered`. The remaining names arrive with the wave-6 catalog registry and the
template directories themselves; once those changes are merged,
`ag new --template <name>` accepts them. Until then, the commands below are the
documented target form, not a claim about the installed binary.
The four available templates are documented with verification evidence in
[Build an offline prototype with the alpha](../examples/alpha-prototype.md).

## Core

The four templates already consumed by `ag new`. These are the safe starting
points in the alpha.

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Minimal HTTP/HTTPS API with the Shield security pipeline | `rest` | None (native `ag-core`, loopback by default) | `ag new my-api --template rest --no-interactive` |
| Small service with WebSocket streaming | `realtime` | None (in-process event bus, `/ws` endpoint) | `ag new my-socket --template realtime --no-interactive` |
| API that persists to PostgreSQL with embedded migrations | `fullstack` | PostgreSQL (`DATABASE_URL`, runs migrations) | `ag new my-data-api --template fullstack --no-interactive` |
| Clean four-crate layered workspace with an `ag.toml` manifest | `layered` | None (health route, in-memory port in stage A) | `ag new my-layered --template layered --no-interactive` |

## Web

Static or SDK-consuming front ends served without a bundled backend service.
Anti-Gravital does not render UI components (`RFC-0019`); these scaffolds
deliver assets and generated client types.

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Serve a single static site or landing page | `static-site` | None (static assets served by `ag-core`) | `ag new my-site --template static-site --no-interactive` |
| Multi-page marketing site | `marketing-site` | None (static assets) | `ag new my-marketing --template marketing-site --no-interactive` |
| Blog with repository-backed content | `blog` | None (content in repo; confirm `template.toml`) | `ag new my-blog --template blog --no-interactive` |
| Documentation site built from markdown | `docs-site` | None (static assets) | `ag new my-docs --template docs-site --no-interactive` |
| Dashboard consuming a generated SDK | `dashboard` | None (points at your API origin; no bundled backend) | `ag new my-dashboard --template dashboard --no-interactive` |
| Admin panel with CRUD views against your API | `admin-panel` | None (points at your API origin; no bundled backend) | `ag new my-admin --template admin-panel --no-interactive` |

## API

Service-shaped starting points. The database-backed ones declare
`DATABASE_URL` and keep an in-memory fallback so the scaffold still builds
offline.

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Typed REST API backed by PostgreSQL | `database-api` | PostgreSQL (`DATABASE_URL`; in-memory fallback offline) | `ag new my-db-api --template database-api --no-interactive` |
| Authentication service with JWT, keys and recovery flows | `auth-service` | None (local key material; confirm store in `template.toml`) | `ag new my-auth --template auth-service --no-interactive` |
| Multi-tenant SaaS API with tenant isolation | `multi-tenant-saas` | PostgreSQL (`DATABASE_URL`) | `ag new my-saas --template multi-tenant-saas --no-interactive` |
| Receive and verify signed webhooks | `webhook-gateway` | None (in-process signature verification) | `ag new my-hooks --template webhook-gateway --no-interactive` |
| Accept, store and serve uploaded files | `file-upload-service` | None (filesystem store; S3-compatible backend optional) | `ag new my-uploads --template file-upload-service --no-interactive` |
| Small catalog and order API | `ecommerce-lite` | PostgreSQL (`DATABASE_URL`) | `ag new my-shop --template ecommerce-lite --no-interactive` |

## Realtime

Event-driven and background workloads. Native in-process modes are the
default; broker and durable backends are opt-in behind Cargo features
(`ADR-0009`).

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Multi-user chat over WebSocket or SSE | `realtime-chat-v2` | None (in-process bus; NATS optional) | `ag new my-chat --template realtime-chat-v2 --no-interactive` |
| Dedicated WebSocket endpoint service | `websocket-service` | None (WebSocket transport, in-process bus) | `ag new my-ws --template websocket-service --no-interactive` |
| Typed background jobs with retries and a dead-letter queue | `worker-service` | None (in-memory backend; PostgreSQL optional for durable queue) | `ag new my-jobs --template worker-service --no-interactive` |
| Periodic scheduled jobs | `scheduled-jobs` | None (in-memory scheduler; PostgreSQL optional when durable) | `ag new my-cron --template scheduled-jobs --no-interactive` |
| Ingest and fan out device events | `iot-ingest` | None (in-process bus; NATS or PostgreSQL optional; confirm `template.toml`) | `ag new my-iot --template iot-ingest --no-interactive` |
| Hostname routing and canonical redirects at request time | `edge-router` | None (local routing config; DNS/TLS via `ag-domains` optional) | `ag new my-edge --template edge-router --no-interactive` |

## Infrastructure

Operational services built on the standard and infrastructure crates.
`mail-service` and `dns-manager` are the two categories that genuinely need
an external counterpart for their full path.

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Send transactional email from a service | `mail-service` | SMTP endpoint, or native in-process MTA (no external service) | `ag new my-mail --template mail-service --no-interactive` |
| Manage DNS records and ACME certificates | `dns-manager` | DNS provider credentials + ACME CA (Let's Encrypt) | `ag new my-dns --template dns-manager --no-interactive` |
| Expose a filesystem-backed object storage API | `object-storage-api` | None (filesystem store; S3-compatible backend optional) | `ag new my-storage --template object-storage-api --no-interactive` |
| Collect structured traces and metrics | `monitoring-service` | None (local tracing and Prometheus scrape; OTLP collector optional) | `ag new my-metrics --template monitoring-service --no-interactive` |
| Boot a multi-service Rust workspace | `microservices-starter` | None (native defaults per service; confirm `template.toml`) | `ag new my-services --template microservices-starter --no-interactive` |
| Build a standalone command-line tool | `cli-tool` | None (`ag-core` without a server) | `ag new my-cli --template cli-tool --no-interactive` |

## AI

AI-shaped scaffolds. `ag-ai`, `ag-wasm-host` and the knowledge graph are
placeholder or future-phase modules, so these templates implement their
concepts natively in-process and mark the integration seams. The default path
needs no key and no model service; a live provider or durable vector store is
opt-in.

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Serve an AI backend with a local provider and offline health checks | `ai-backend-template` | None (local echo provider by default; provider API key needed only for a live provider) | `ag new my-ai --template ai-backend-template --no-interactive` |
| Similarity search over embeddings | `vector-search` | None (in-memory repository with local embeddings; external vector store optional; confirm `template.toml`) | `ag new my-search --template vector-search --no-interactive` |
| Retrieval-augmented generation pipeline | `rag-pipeline` | None (in-memory retrieval and extractive answers; AI key and durable vector store optional; confirm `template.toml`) | `ag new my-rag --template rag-pipeline --no-interactive` |
| Package a sandboxed WebAssembly module | `wasi-plugin` | None (host skeleton and capability policy; execution reserved for the later-phase `ag-wasm-host`) | `ag new my-plugin --template wasi-plugin --no-interactive` |

## Testing

| Use case | Template | External requirements | Example command |
| --- | --- | --- | --- |
| Start a project with unit, integration and property tests wired | `testing-suite` | None (offline test harness, no service) | `ag new my-tests --template testing-suite --no-interactive` |

## Choosing quickly

- Need a server that runs with no setup at all: `rest`, `realtime`, `layered`.
- Need PostgreSQL on day one: `fullstack`, `database-api`, `multi-tenant-saas`,
  `ecommerce-lite`.
- Need events or background work without a broker or database: `realtime`,
  `websocket-service`, `worker-service`, `scheduled-jobs`.
- Need outbound email or DNS/TLS automation: `mail-service`, `dns-manager`.
- Need a build-time-only test scaffold: `testing-suite`.

## Status

This gallery covers every name in the wave-6 board: 4 core, 6 web, 6 api,
6 realtime, 6 infrastructure, 4 ai and 1 testing template (33 total). It
documents selection only. Per-template run commands, limitations and the
authoritative `external_requirements` live in each `templates/<name>/README.md`
and `templates/<name>/template.toml`.
