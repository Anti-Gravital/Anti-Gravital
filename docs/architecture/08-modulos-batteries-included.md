# Capitulo 8. Modulos batteries-included

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 8
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [07-anti-dsl.md](./07-anti-dsl.md)
> Siguiente: [09-plugins-wasi.md](./09-plugins-wasi.md)

## 8. Batteries-included modules

This section specifies each of the standard modules of the ecosystem. Each subsection documents the purpose, the technical stack, the design decisions, and the extension points.

### 8.1 `ag-auth` — Authentication and authorization

The authentication module implements the modern identity schemes. The central architectural decision is to support Passkeys/WebAuthn as first-class, not as an afterthought; passwords are a legacy mechanism that is supported but not recommended.

The technical stack implemented is custom WebAuthn with `ciborium` (CBOR) and COSE verification (`p256` for ES256, `ed25519-dalek` for EdDSA) instead of `webauthn-rs`, for Apache-2.0 license compatibility (see `ADR-0006`); `jsonwebtoken` for JWT, `argon2` for password hashing (when used), and `oauth2` as the OAuth2 client (Google, GitHub) with the PKCE flow.

The supported flows are: registration and authentication with passkey, authentication with email + password (legacy), OAuth2 with preconfigured providers (Google, GitHub, Microsoft, Gravital ID), API keys for server-to-server integrations, and refresh tokens with rotation.

JWTs are signed with Ed25519 by default (Edwards25519 curve, faster than RSA and more secure than ECDSA-P256 against side-channel attacks). The private key lives in an external secret manager (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager) or in environment variables with documented rotation.

The RBAC is declared in the schema and compiled to evaluable expressions. The policy is evaluated once per request in the Shield, before reaching the handler. Policies can reference JWT claims, path parameters, and query the database if explicitly declared (with caching to avoid the N+1).

### 8.2 `ag-data` — Data access and migrations

The data module is built on sqlx, with compile-time verification of SQL queries. This means that when `cargo build` runs, sqlx connects to a development database (configurable by environment variable) and verifies that each query is syntactically valid and that the types of the returned columns match the Rust structs that receive them. A SQL error ceases to be a runtime error; it becomes a compile error.

The supported backends are PostgreSQL (recommended for production), SQLite (for development, tests, and edge applications), and MySQL (for legacy environments).

Migrations are embedded in the binary with `sqlx::migrate!`. This means that the binary itself contains the complete history of migrations, and at startup it can automatically apply the pending ones. For environments where this is not desirable (blue-green deployments with migration as a separate step), the `ag migrate apply` command runs the migrations without bringing up the server.

For multi-tenant architectures, `ag-data` natively supports schema-per-tenant in PostgreSQL: each tenant has its own schema with the same tables, and the connection router selects the schema based on the JWT claim. It also supports Row-Level Security (RLS) for cases where schema isolation is excessive.

Read replicas are configured declaratively; the module routes read-only queries to the nearest replica and write queries to the primary.

### 8.3 `ag-realtime` — Events and real-time communication

`ag-realtime` offers three modalities of bidirectional communication: binary WebSocket, Server-Sent Events for unidirectional streams, and a pub/sub event bus.

The event bus uses NATS as the broker. For small cases, NATS runs embedded in the same Anti-Gravital binary (edge mode). For cases at scale, the binary connects to an external NATS cluster. This duality allows starting simple and scaling without rewriting.

For WebSocket, the internal binary protocol (based on msgpack) reduces the overhead compared to JSON. The WebSocket handlers are declared in the schema and receive messages already deserialized to Rust structs.

For SSE, it is used as an automatic fallback in browsers that do not support WebSocket or are behind proxies that block it. The negotiation is transparent.

Event persistence uses JetStream (a component of NATS) when available, which allows event replay for new consumers and durability against broker crashes.

### 8.4 `ag-cache` — Multi-level cache

The cache module offers two levels. The L1 level, already implemented, is an in-memory cache with `moka`, a concurrent implementation without contended locks based on TinyLFU, with tag-based invalidation. The L2 level (distributed cache between instances) is not yet implemented: `RFC-0005` proposes a native L2 compatible with the RESP2 protocol, without a dependency on Redis as an external service, and remains pending approval and implementation.

Invalidation is done by events. When an endpoint emits an event (`user.updated`), `ag-cache` automatically invalidates the related entries at both levels. The invalidation policy is declared in the schema.

The SQL query cache is automatic: queries marked with `@cache(ttl: 5m)` in the schema are cached transparently, and invalidation is triggered when an event touches any of the involved tables.

### 8.5 `ag-storage` — Object storage

`ag-storage` offers an abstraction over three backends: S3 (AWS and compatibles), MinIO (self-hosted), and local filesystem (for development). The backend is selected by configuration; the application code does not notice.

Signed URLs for download and direct upload are generated with a single call: `storage.signed_url(key, Duration::from_mins(15), Permission::Write)`.

Image processing (resize, compress, format conversion) is done with the `image` crate, supporting JPEG, PNG, WebP, and AVIF. Thumbnails are generated automatically on upload if the policy is declared in the schema.

### 8.6 `ag-observe` — Traceability, metrics, and logging

Observability is a first-level concern and not an optional module for production. Its stack is `tracing` for structured spans, `opentelemetry-rust` for export to compatible backends (Jaeger, Tempo, Datadog, Honeycomb), `metrics` for metrics with a Prometheus backend, and pre-configured Grafana dashboards that are included as JSON in the repository.

Each request traverses the whole system with a unique correlation ID that appears in all structured logs, all tracing spans, and all errors returned to the client. This solves the problem of debugging in production: given a support ticket with a correlation ID, the operator can reconstruct the complete path of the request.

`tokio-console` is integrated in development mode for live inspection of the Tokio tasks.

### 8.7 `ag-ui` — Optional Server-Side Rendering

The SSR module exists for cases where a SPA frontend is excessive: internal dashboards, marketing pages, simple forms, and administrative interfaces. It is based on `askama` (build-time compiled templating, with verified types) and native integration with HTMX for interactivity without heavy JavaScript frameworks.

This module is explicitly *not* a competitor of React, Vue, Svelte, or Next.js. For SPA or rich SSR applications, the recommended pattern is Anti-Gravital as backend with a Next.js (or other) frontend that consumes the generated TypeScript client.

### 8.8 `ag-mail` — Transactional communication (deferred standard)

Introduced by `ADR-0007` in Phase 4.5. `ag-mail` is a **deferred** standard module: it has the maturity and scope of a standard, but is NOT installed by default in the official templates. It is incorporated when the project requires outbound transactional mail (account verification, magic links, password recovery, alerts, notifications).

The v1 scope is **exclusively outbound**. `ag-mail` is NOT an MTA, it does NOT receive mail (no IMAP/POP), it does NOT offer persistent mailboxes, it does NOT implement antispam, filtering, or IP reputation management. This restriction is deliberate and is fixed in the ADR: the inbound and complete mail server capabilities are the work of a different project, not of Anti-Gravital.

> **Scope update (`ADR-0010`, 2026-06-03).** The "NOT an MTA / inbound never" restriction above is **superseded**: `ag-mail` is being expanded into a native outbound MTA (MX resolution, ESMTP+STARTTLS delivery, DKIM signing, bounce classification) so a project can send authenticated mail with no third party in the sending path. The expansion is phased and opt-in behind Cargo features (`mta`, `api`, `queue-jetstream`); the outbound-relay baseline described here remains the default and is unchanged. Mailbox hosting, IMAP/POP/JMAP and general inbound stay out of scope; inbound is admitted only as DSN/ARF parsing for bounce processing. Technical plan: `RFC-0009`. The native MTA is forward work and is not claimed as implemented here.

The technical stack is `lettre` with async Tokio transport and `rustls` for the native SMTP sender (coherent with The Shield). External providers are reached with the native SMTP relay; the no-third-party path is the native MTA (`mta` feature). The `MailSender` trait abstracts both:

```rust
#[async_trait::async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
    fn provider_name(&self) -> &'static str;
    fn dns_requirements(&self, domain: &str) -> Vec<DnsRecordSpec>;
}

pub enum AgMail {
    Native(SmtpSender),                // lettre + rustls
    Adapter(Box<dyn MailSender>),      // external providers via SMTP
}
```

The Native | Adapter pattern is identical to the one used by `ag-storage` (`Native | S3`) and to the one planned for the L2 of `ag-cache` (L1 `moka` native today; L2 RESP2 native proposed in `RFC-0005`), reinforcing the project's interoperability rule: integrate dominant providers, do not replace them.

The **templates** are modeled with the `MailTemplate` trait and a `StringTemplate` implementation of `{{var}}` substitution; any external engine (askama, minijinja) can be plugged in by implementing the trait. Variable validation is done against the `schema.ag`: the DSL compiler emits a warning when the `from` of a `mail` block does not reference a declared `domain`, and verifies that the typed `vars` of the template match the markers used. A malformed email ceases to be a runtime bug and approaches a build-detectable error. This is the **real differentiator** (build-time correctness), not deliverability: deliverability is the provider's job; the correctness of the contract is the framework's job.

The **async queue** accepts jobs with retries and exponential backoff. Default backend in memory (Tokio task + channel). Optional persistent backend via `ag-data` (jobs table) to survive restarts. Optional integration with `ag-realtime` for event fan-out. Each job emits metrics towards `ag-observe`: `ag_mail_sent_total`, `ag_mail_failed_total`, `ag_mail_retry_total`, latency histogram.

The **integration with `ag-auth`** is strictly unidirectional: `ag-auth` consumes `ag-mail` by invoking a small trait that `ag-auth` defines. `ag-mail` does NOT know about `ag-auth`. The sixth rule of section 5.3 documents this directionality.

`mail` block of DSL v0.7 (example):

```ag
mail WelcomeEmail {
    from "hello@plenty.market"      # debe referenciar un bloque domain
    subject "Welcome to Plenty"
    template "emails/welcome.html"  # debe existir
    vars {
        name String
        activation_url String        # debe usarse en el HTML
    }
}
```

### 8.9 `ag-domains` — Domain and TLS management (optional infra)

Introduced by `ADR-0007` in Phase 4.5. `ag-domains` is an **infrastructure optional** module: not every backend administers DNS (many deploy behind a proxy or PaaS that already resolves it), but when a project wants `ag deploy` to deliver a URL `https://miapi.example.com` with a valid certificate in a single command, `ag-domains` is the responsible module.

The module is **NOT a domain registrar**: the domain is bought externally (Namecheap, Cloudflare Registrar, etc.) and delegated via nameservers to the configured provider. `ag-domains` also does not replace Terraform or Pulumi: for complex multi-cloud infrastructure or centralized management of arbitrary DNS zones, the project should use the dominant tools. The boundary is fixed in the ADR.

The core of the module is the `DnsProvider` trait:

```rust
#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    async fn list_records(&self, zone: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn upsert_record(&self, zone: &str, record: &DnsRecord) -> Result<(), AgDomainsError>;
    async fn delete_record(&self, zone: &str, id: &str) -> Result<(), AgDomainsError>;
    fn provider_name(&self) -> &'static str;
}
```

Small, versioned, with **contract tests** that every adapter must pass. The initial adapter is Cloudflare (authentication by API token). The trait is designed to add Route53, Namecheap, DigitalOcean, etc. in later iterations without touching the public surface.

The **ACME client** (`instant-acme`) issues and renews Let's Encrypt certificates. It supports the DNS-01 challenge (preferred, uses the `DnsProvider` itself to create the required TXT) and HTTP-01 (alternative). The renewal runs as a background Tokio task, watching expiration and renewing before the configured threshold. Certificate storage is filesystem by default, or optional `ag-storage`.

The cooperation with `ag-mail` materializes in `generate_mail_records`: `ag-mail` declares its requirements via `MailSender::dns_requirements` and `ag-domains` materializes them as records (SPF, DKIM, DMARC). This is a **cooperation** relationship, not a control one: `ag-mail` does not depend on `ag-domains`; a project can use `ag-mail` with a external provider (via SMTP) without `ag-domains` participating.

The **propagation verification** uses `hickory-resolver` to query multiple public resolvers and confirm that the records propagated before marking an operation as successful. This blocks `ag deploy` until the domain responds, avoiding delivering URLs that the operator promised but that do not resolve yet.

`domain` block of DSL v0.7 (example):

```ag
domain plenty.market {
    provider "cloudflare"
    tls { mode auto  acme true }
    dns {
        CNAME "api"     -> "ag-cloud-target"
        TXT   "_dmarc"  -> "v=DMARC1; p=quarantine"
    }
    mail { spf auto  dkim auto  dmarc quarantine }
}
```


---

