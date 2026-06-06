# ag-mail

Correo transaccional outbound para Anti-Gravital.

Status: **Phase 4.5 — implemented**, plus the opt-in native MTA core of
**Phase 4.6-A**. Native SMTP relay, an opt-in native MTA, in-memory
and persistent retry queues, custom SMTP headers, string templating and
`ag-observe` metrics are functional. The `mta` feature (off by default) adds a
native outbound MTA: MX resolution, ESMTP+STARTTLS direct delivery, Ed25519 and
RSA-SHA256 DKIM signing, a bounce classifier and MTA metrics. Remaining tech
debt (external template engines; native MTA durable queue/shaping/DSN-FBL/REST
in later phases) is tracked in `docs/DEBT.md`. Decisions:
`docs/adr/0007-ag-mail-ag-domains.md`, `docs/adr/0010-ag-mail-native-mta-pivot.md`.
Technical plans: `docs/rfc/RFC-0006-ag-mail-alcance.md`,
`docs/rfc/RFC-0009-ag-mail-native-mta.md`. Module sheet:
`docs/modules/ag-mail/README.md`.

## Alcance v1

- Relay SMTP nativo (lettre + rustls) y MTA outbound nativo (opt-in).
- HTML/plaintext templates via the built-in `StringTemplate` engine. Compile-time
  validation against `schema.ag` and external engines (askama/minijinja) are tracked
  as tech debt in `docs/DEBT.md`.
- Cola asincrona con reintentos y backoff exponencial.
- Metricas hacia ag-observe.

## Native MTA (feature `mta`, opt-in)

Additive native outbound MTA (Phases 4.6-A/B, `ADR-0010` / `RFC-0009`):

- `MtaSender` — direct delivery to the destination MX over ESMTP+STARTTLS,
  with SMTP-reply classification; also a `queue::DeliveryBackend`.
- MX resolution with preference ordering, `site_name` rollup and implicit-MX
  fallback (`hickory-resolver`).
- Egress sources/pools (source IP + EHLO) with weighted round-robin warm-up.
- Ed25519 and RSA-SHA256 DKIM signing applied last (`mail-auth`).
- Pure SMTP bounce classifier (transient vs permanent, RFC 3463).
- Per-`site_name` traffic shaping (token-bucket rate + connection cap).
- Two-tier scheduled/ready delivery queue with retry/backoff, max-age,
  `max_ready` and an automatic suppression list; a `run` worker.
- `ag-observe` metrics on the send path and queue depth; `mail-mta` CI job.

Off by default; the default sender is unchanged. A durable queue spool
(JetStream/PostgreSQL), the REST API/webhooks, marketing and asynchronous
DSN/FBL intake are later phases.

## Fuera de alcance

- Servidor de correo completo / buzones / IMAP / POP / JMAP.
- Recepcion de correo general (inbound mas alla de DSN/FBL para bounces).
- Antispam como servicio.

## Direccion de dependencia

`ag-auth` consume `ag-mail` para verificacion de correo, recuperacion de
contrasena y magic links. `ag-mail` NO depende de `ag-auth`. Esta regla
esta documentada como sexta regla de dependencias en
`docs/architecture/05-ecosistema-modulos.md` y proviene de `ADR-0007`.

## Features de Cargo

- `smtp` (default): relay SMTP nativo (apuntable a cualquier proveedor externo).
- `templates` (default): built-in StringTemplate rendering.
- `minijinja` (opt-in): external Jinja2-style template engine
  (`template::jinja::MinijinjaTemplate`, loops/conditionals/filters) via the
  `MailTemplate` trait. Pulls `minijinja`; StringTemplate stays the default.
- `metrics` (default): metricas hacia ag-observe.
- `queue-persistent`: PostgreSQL-backed persistent queue via ag-data (`PersistentQueue`,
  migration in `migrations/0001_mail_queue.sql`).
- `mta` (opt-in): native outbound MTA (MX resolution, ESMTP+STARTTLS delivery,
  Ed25519/RSA DKIM, egress pools, traffic shaping, two-tier queue, suppression,
  DSN/ARF intake, metrics). Pulls `mail-send`, `mail-auth`, `mail-parser`,
  `hickory-resolver`, `rustls-pki-types`; not in the default build.
- `api` (opt-in): managed API surface; currently HMAC-SHA256 signed webhooks
  (`api::webhook`). Pulls `hmac`/`sha2`/`base64`; not in the default build.
