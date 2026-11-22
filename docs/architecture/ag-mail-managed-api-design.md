# ag-mail managed REST API: architecture and resource model (design only)

> Status: design proposal, NOT a capability claim. No routes, data model, CLI
> command or Cargo feature is implemented by this document. Issue #152 stays
> open with `Refs #152`.
>
> Relation to other documents: this is an **annex** under `docs/architecture/`,
> not a verbatim chapter of the master architecture. The owning plan is
> `docs/rfc/RFC-0009-ag-mail-native-mta.md` (sections 4.5, 4.6, 5, stages
> 4.6-C/D) and the roadmap is `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`
> (section 4.6.3). The HTTP contract authority for shapes, error codes,
> idempotency and capabilities is
> `docs/modules/ag-mail/managed-api-dx-design.md`; this annex does not restate
> or override it. Governing rules: `CLAUDE.md` 0, 1, 7, 8, 15, 22, 26, 28;
> `ADR-0008` (English canonical); `ADR-0009` (native default); `ADR-0010`
> (outbound-only scope); `ADR-0011` (generic names); `RFC-0024`
> (capability-scoped agents).

## 1. Purpose and decision

RFC-0009 sections 4.5/4.6 name the managed surface in one paragraph; the DX
design fixes its HTTP contract. What was still missing is the **technical
resource model**, the **outbound boundary**, the **composition** with the two
crates that already touch this path (`ag-auth`, `ag-workers`), and a single
place that lists the gates that must pass before code is written. This annex
supplies those four things.

Decision: **design only**. No implementation, no phase advance, no closure of
#152. The BDFL decision of 2026-11-22 formally deferred Mail C and Mail D under
Hoja de Ruta 4.6.3; the prerequisite Mail A gate has no real-mailbox evidence,
and starting C/D first would be "adelantarse de fase" (`CLAUDE.md` rule 8).

Document boundaries:

- HTTP paths, JSON shapes, RFC 9457 problem codes, the 24h idempotency rule and
  the six proposed capabilities are defined in the DX design. Endpoint
  sketches below are **shaped** after that contract; where a field is omitted,
  the DX design remains authoritative.
- The independent DKIM verification kit produced for #178 is **referenced,
  not duplicated**. See section 6, gate G1.
- No new DSL version: the v0.7 `mail`/`domain`/`template` blocks already cover
  declaration (RFC-0009 section 4.7). No new CLI command: any `ag mail api
  ...` needs its own RFC (rule 28).

## 2. Conceptual resource model

The mission vocabulary (messages, templates, suppression lists, campaigns,
consent) maps to the RFC-0009 / DX-design resources as follows. The HTTP layer
uses the RFC names; the conceptual names are kept here to make the model
explicit.

| Concept | v1 resource | Storage | Lifecycle owner |
| --- | --- | --- | --- |
| Message | `emails` (`POST /v1/emails`, `GET /v1/emails/{id}`) | `emails` + append-only `events` | `mail.delivery` job on `ag-workers` |
| Template | typed `mail` template selected by name, plus optional `minijinja` engine | no new table in v1; declaration lives in `schema.ag` | build-time validation (`template::validate`) |
| Suppression list | `suppressions` | `suppressions`; in-memory `SuppressionList` is the native default | DSN/ARF intake and one-click unsubscribe |
| Campaign | `broadcasts` with `audiences`/`segments`/`topics` | marketing tables (4.6-D) | fan-out into `mail.delivery` jobs |
| Consent | topic `subscriptions` + RFC 8058 one-click target | `subscriptions`, mirrored in `suppressions` | public signed-token endpoint, then event |

### 2.1 Messages

A message is the unit of delivery. Its status advances through
`queued | scheduled | sending | sent | failed | canceled | suppressed`. Each
transition appends an immutable row to `events` with stable type strings
(`email.queued`, `email.delivered`, `email.bounced`, `email.complained`,
`email.unsubscribed`). The same vocabulary is emitted by signed webhooks, so
the REST read model and the webhook stream cannot drift. A message references
either a typed template plus a `variables` map, or explicit `text_body`/
`html_body`; supplying both is a validation error in the DX contract.

### 2.2 Templates

Templates are not a new managed object in v1: a broadcast or a single email
names an already declared typed template. Validation remains compile-time and
the optional external engine remains behind `minijinja`. "Native bulletproof
template components" in RFC-0009 stage 4.6-D are a *later* addition to this
same resource; they do not introduce a second template model.

### 2.3 Suppression lists

The implemented native default is `sender::mta::suppress::SuppressionList`: a
thread-safe, case-insensitive set with reasons `HardBounce`, `Complaint`,
`Unsubscribe` and `SoftBounceLimit`, where stronger reasons never downgrade.
The durable `suppressions` table is a **projection** of that model behind the
`api` feature, not a different policy: a send is checked against suppression
before a job is enqueued, and a suppression event from DSN/ARF intake or from
one-click unsubscribe writes through to both the in-memory list and the table.
The blocking rule (a suppressed recipient is never delivered) is identical in
both tiers.

### 2.4 Campaigns

A campaign is a `broadcast` draft that references a typed template and an
audience or segment. `POST /v1/broadcasts/{id}/send` fans out into one
`mail.delivery` job per eligible contact; the whole fan-out is covered by a
single idempotency key, and partial results are returned in the response body
(idempotency semantics are defined in the DX design). Statuses are
`draft | scheduled | running | canceled | completed`. Contacts, audiences,
segments and topics are tenant-scoped and soft-deletable.

### 2.5 Consent

Consent is per `(tenant, contact, topic)` subscription state. It is distinct
from suppression: unsubscribing from one topic removes that subscription,
while an explicit `Unsubscribe` (or a hard bounce or complaint) suppresses the
address globally and blocks every future send. The RFC 8058 one-click target
is a public, signed-token endpoint; it never takes an API key. A one-click
request appends `email.unsubscribed`, updates the subscription, writes the
suppression, and is observable through webhooks using the existing signing
scheme.

### 2.6 Tenancy and persistence

A tenant is resolved from the API key; there is no tenant id in any path, so
one key cannot address another tenant's objects. Every row in the schema
carries `tenant_id`. RFC-0009 section 4.5 lists the schema: `tenants`,
`api_keys`, `domains`, `dkim_keys`, `emails`, `events`, `suppressions`,
`webhooks`, `webhook_deliveries`, `ip_pools`/`ip_addresses`, `delivery_queue`,
plus the marketing tables above. Migrations are embedded following the
existing project pattern. The native default mode requires none of this
schema (`ADR-0009` rule 2).

## 3. Outbound-only boundary in v1

The managed surface is outbound by design. `ag-mail` has never offered
mailbox storage, IMAP or POP, and `ADR-0010` explicitly limits inbound to
DSN/ARF parsing.

| Operation | v1 status | Rationale |
| --- | --- | --- |
| Queue single / batch email, schedule, cancel | In scope (4.6-C) | Core sending API, RFC-0009 section 4.6 |
| Read message status and append-only events | In scope (4.6-C) | Observability of outbound delivery |
| Domains, DKIM keys, egress pools, API keys, webhooks | In scope (4.6-C) | Identity, tenancy and delivery control |
| Bounce and complaint intake (DSN/ARF) | In scope as **signals**, not mailbox access | Feeds the existing suppression list |
| RFC 8058 one-click unsubscribe | In scope (4.6-D) | Signed target; no inbox involved |
| Broadcasts, contacts, audiences, segments, topics | In scope (4.6-D) | Marketing objects of the issue |
| IMAP/POP, general inbound receive, mailboxes, threads, replies | Out of scope | `ADR-0010`; would change the product |
| Hosted service, mail server, UI | Out of scope | Backend-only, generated project or `ag dev` runtime |
| Provider-brand adapters | Out of scope | `ADR-0011`; generic `SmtpSender` endpoint is the path |

This boundary is a direct consequence of the gate in section 6: even the
outbound surface stays experimental until real external delivery is proven.

## 4. OpenAPI-shaped endpoint sketches

Illustrative fragments, not a generated or published OpenAPI file. All routes
are versioned under `/v1`, JSON only, and require an API key except the
signed one-click target. Pagination, the error model and idempotency details
are in the DX design.

```yaml
openapi: 3.1.0
info:
  title: Anti-Gravital managed mail API (design, not implemented)
  version: "0.1-design"
servers:
  - url: /v1
security:
  - ApiKeyAuth: []   # tenant resolved from the key; no tenant id in path
paths:
  /emails:
    post:
      operationId: email.send
      x-capability: mail.send
      parameters:
        - $ref: "#/components/parameters/IdempotencyKey"
      requestBody:
        content:
          application/json:
            schema: { $ref: "#/components/schemas/EmailCreate" }
      responses:
        "201":
          description: Queued; replay carries idempotent_replay=true
          content:
            application/json:
              schema: { $ref: "#/components/schemas/Email" }
        "400": { $ref: "#/components/responses/Problem" }
        "409": { $ref: "#/components/responses/Problem" }   # idempotency_conflict
        "429": { $ref: "#/components/responses/Problem" }   # Retry-After
  /emails/{id}:
    get:
      operationId: email.read
      x-capability: mail.read
      responses:
        "200": { description: Status plus append-only events array }
        "404": { $ref: "#/components/responses/Problem" }
  /suppressions:
    get:
      operationId: suppression.list
      x-capability: mail.read
      responses:
        "200": { description: Paginated entries with reason and created_at }
    post:
      operationId: suppression.add
      x-capability: mail.manage
      responses:
        "201": { description: Recipient suppressed with a typed reason }
  /templates:
    get:
      operationId: template.list
      x-capability: mail.read
      responses:
        "200": { description: Declared typed templates (schema.ag v0.7) }
  /broadcasts:
    post:
      operationId: broadcast.create
      x-capability: marketing.write
      responses:
        "201": { description: Broadcast draft referencing template and audience }
  /broadcasts/{id}/send:
    post:
      operationId: broadcast.send
      x-capability: mail.send.broadcast
      parameters:
        - $ref: "#/components/parameters/IdempotencyKey"
      responses:
        "202": { description: Fan-out queued; per-contact partial results }
  /unsubscribe/one-click:
    post:
      operationId: consent.unsubscribeOneClick
      security: []        # public, RFC 8058 signed token
      requestBody:
        content:
          application/x-www-form-urlencoded:
            schema: { $ref: "#/components/schemas/OneClickPost" } # List-Unsubscribe=One-Click
      responses:
        "200": { description: Consent updated; suppression written }
        "400": { $ref: "#/components/responses/Problem" }   # invalid/expired token
components:
  parameters:
    IdempotencyKey:
      name: Idempotency-Key
      in: header
      required: true
      schema: { type: string, minLength: 1, maxLength: 256 }
  responses:
    Problem:
      description: application/problem+json (RFC 9457) with stable code
  schemas:
    EmailCreate:
      type: object
      required: [from, to, subject]
      properties:
        from: { type: string }
        to: { type: array, items: { type: string } }
        subject: { type: string }
        template: { type: string, description: "Typed template name" }
        variables: { type: object }
        text_body: { type: string }
        html_body: { type: string }
        metadata: { type: object }
        send_at: { type: string, format: date-time }
    OneClickPost:
      type: object
      properties:
        List-Unsubscribe: { const: One-Click }
```

Third-party brand names never appear in paths, capabilities, features or
identifiers (`ADR-0011`); external destinations are reached by pointing the
generic `SmtpSender` at an SMTP endpoint.

## 5. Composition and dependency direction

### 5.1 With `ag-auth`: consumer only, no reverse dependency

The Technical Architecture sixth rule (`docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
section 5.3) is strict: `ag-auth` consumes `ag-mail` through a small trait that
`ag-auth` defines; `ag-mail` does **not** depend on `ag-auth`. This is already
true in the code (`ag-auth` has `src/mailer.rs` behind its `mail` feature;
`ag-mail/Cargo.toml` has no `ag-auth` dependency) and must remain true when the
API lands.

Consequences for the managed surface:

- Verification, password recovery and magic-link mail keep flowing through
  `AuthMailer` -> `MailSender`. If a generated project enables the managed
  API, the host may inject a queue-backed sender so those messages also become
  tenant-visible `emails` rows; the choice is made by the host, not inside
  `ag-mail`.
- API key hashing is **BLAKE3-shaped**, matching `ag-auth::api_keys`, but it
  must not be imported from `ag-auth`. `ag-mail` defines a narrow port (for
  example `ApiKeyHasher`) and the host wires a hasher; a default implementation
  can use a direct primitive dependency, whose addition must be justified in
  the implementation PR (`CLAUDE.md` rules 15 and 22). The DX design's mention
  of the "existing ag-auth hashing primitive" describes the algorithm shape,
  not a crate edge; this annex removes that ambiguity.
- No capability, user or account object is duplicated in `ag-mail`. The API
  key carries explicit capabilities and a tenant reference; identity remains
  with `ag-auth`/the host.

### 5.2 With `ag-workers`: producer only, no reverse dependency

`ag-mail` already depends on `ag-workers` behind its `workers` /
`workers-postgres` features (RFC-0012 S7/M2-M4); `ag-workers` never depends on
`ag-mail`. The managed API reuses that edge rather than creating a second
generic queue, which was retired after parity was verified.

```text
POST /v1/emails (api feature, capability mail.send)
  -> validate shape + template/variables contract
  -> suppression pre-check (in-memory list + durable projection)
  -> write idempotency record + emails row + events row transactionally
  -> MailQueue::enqueue -> WorkersMailQueue -> ag-workers "mail.delivery" job
  -> worker pool + MailDeliveryHandler -> MailSender
       default: SmtpSender  (native relay)
       opt-in:  MtaSender   (mta feature, direct MX + DKIM)
  -> delivery result -> events -> signed webhook (api::webhook)
```

Properties that follow from the existing adapter:

- The job payload is the typed `Email` with `PAYLOAD_VERSION = 1` and stable
  kind `mail.delivery` on the shared `ag_worker_jobs` table.
- Retry classification is already fixed in `workers.rs`: transport/DNS/queue
  faults are retriable; invalid address, template, config, no mail host, DKIM
  and exhausted sends are permanent and dead-lettered.
- Durability comes from `workers-postgres` and the optional `queue-postgres`
  spool; the in-memory backend stays the default.
- Broadcast send is a fan-out producer over the same adapter; the campaign
  object stores aggregate status, not a second queue implementation.
- The `api` route layer consumes the `MailQueue` trait, so it does not require
  `workers` to compile. Both are opt-in and composable.

### 5.3 Native default and DX guarantee

None of the default features change: `SmtpSender`, `InMemoryQueue`, typed
templates and metrics stay as they are. `api`, `workers`, `workers-postgres`,
`mta` and `queue-postgres` are separate opt-ins. `ag new my-api --template rest`
followed by `ag dev` must not require the API feature, a database or a mailbox;
the test path uses local/ephemeral PostgreSQL 16 and the generic relay sender,
contacting no third party. `ag mail test --to ADDRESS` remains the only setup
check required for the first-API experience.

## 6. Gates that must pass before implementation

No code is written while any gate is open. Status is recorded honestly; an
offline check, a design document or a deferral decision is never presented as
a passed delivery gate.

| Gate | Requirement | Status (2026-11-22) |
| --- | --- | --- |
| G1 Mail A real-mailbox evidence | A message delivered by the native MTA to a real external mailbox passing aligned SPF, DKIM and DMARC, with date, commit, recipient domain and full headers recorded (RFC-0009 section 5 stage 1; Hoja de Ruta 4.6.3; #153/#178) | **NOT PASSED**. Outbound port 25 is blocked. |
| G2 Phase authorization | Mail A evidence OR a formal RFC-0009 amendment under Hoja de Ruta 4.6.3 that moves C/D out or reorders them, plus the current BDFL deferral being lifted for C/D | **NOT PASSED**. Deferred by BDFL decision 2026-11-22. |
| G3 Contract accepted | The DX design (sections 3-5) and this annex accepted; RFC-0024 amended with the mail/marketing tool groups and six capabilities; CLI decision taken (defer recommended); any `ag mail api` command covered by its own RFC | **OPEN**. RFC-0024 still defines only the six existing groups. |
| G4 Test plan fixed | Acceptance tests against local/ephemeral PostgreSQL 16 or a CI service container; `cargo test -p ag-mail --features api` and the `mail-*` CI jobs green; native default and relay sender unchanged | **OPEN**. No routes exist yet. |
| G5 Dependency justification | New dependencies (HTTP layer, hashing primitive, SQL layer) justified per rule 15 and declared in the implementing RFC/PR; no new crate without an RFC | **OPEN**. No dependency decision taken. |

### 6.1 DKIM evidence is referenced, not duplicated

The independent verifier for #178 already has its own kit:

- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` (method, exact command,
  recorded dkimpy output and scope);
- `scripts/check-dkim-independent.sh` (explicit PASS / SKIP / FAIL; full test
  path with `--exact` prevents a zero-test false PASS; SKIP is never PASS);
- `.github/workflows/dkim-independent.yml` (manual-only, opt-in) and
  `docs/audits/dkim-independent-ci.md` (real run and the remaining condition).

That kit verifies **offline embedded vectors** with a second implementation.
It does not contact a receiver and does not observe an
`Authentication-Results` header. It therefore does **not** satisfy G1, and
this annex neither copies nor extends it. G1 needs a maintainer with port 25
egress to deliver an ag-mail-signed message to a real mailbox and record the
receiver header with date and commit. Until then #178 and #152 both stay open
with `Refs`.

### 6.2 Exit condition

When G1-G5 hold, the issue's Done-when becomes executable: REST routes and the
PostgreSQL model land behind `api` with tenant-scoped idempotency, marketing
objects land behind `api`, tests cover them against local PostgreSQL, and
DEBT-021 (which already references #152) is resolved. If instead the amendment
path is chosen, this annex and the DX design are the input to that RFC-0009
amendment, #152 is retargeted to a later phase, and no Phase 4.6 code is
written.

## 7. References

- `docs/rfc/RFC-0009-ag-mail-native-mta.md`, sections 4.5, 4.6, 5.
- `docs/modules/ag-mail/managed-api-dx-design.md` (HTTP contract authority).
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, section 4.6.3.
- `docs/DEBT.md` DEBT-021 (points to #152).
- `crates/ag-mail/src/workers.rs`, `crates/ag-mail/src/sender/mta/suppress.rs`,
  `crates/ag-mail/src/api/webhook.rs` (implemented building blocks).
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` and
  `docs/audits/dkim-independent-ci.md` (#178 evidence; G1 is still open).
- `docs/rfc/RFC-0024-mcp-shared-machine-interface.md` (capability contract).
