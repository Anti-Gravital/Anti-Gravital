# ag-mail managed REST API - DX design (design only, not implemented)

> Status: design proposal, NOT a capability claim. No code ships with this
> document. Owning plan: `docs/rfc/RFC-0009-ag-mail-native-mta.md`
> (sections 4.5/4.6 and section 5, stages 4.6-C/D). Tracked by issue #152.
> Governing rules: `CLAUDE.md` 0, 1, 7, 8, 22, 26, 28, 32; `ADR-0008`
> (English canonical); `ADR-0009` (native default); `ADR-0011` (no third-party
> brand names in public surface); `RFC-0024` (capability-scoped agents).

## 1. Purpose and decision

This document closes the design gap for the "managed REST API and marketing
objects" debt (historical DEBT-021, issue #152) before any code is written.
It makes the surface precise enough to be implemented without improvisation,
and it records the exact gate condition that must be satisfied first.

**Decision: design only.** No REST routes, data model, CLI commands or
marketing objects are implemented here. The rationale:

1. Phase 4.6 is *in progress* and authorized with the pre-Phase-5 gate open
   (`docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` section 4.6). Exit criterion
   4.6.3 says Mail C and Mail D may be "either delivered or formally moved out
   of Phase 4.6 by an RFC amendment" and that they "do not block Phase 5".
   So starting them is *permitted*, and would not by itself break the release
   gate.
2. RFC-0009 section 5 nevertheless makes each stage a sequencing gate: C/D
   follow A/B. The **Mail A gate** - a message delivered by the native MTA to
   a real external mailbox passing aligned SPF, DKIM and DMARC, with date,
   commit and headers recorded - has **no evidence**. Real external delivery
   is impossible in this environment (outbound port 25 is blocked; DEBT-022 /
   issue #153). Implementing C/D ahead of that evidence would be
   "adelantarse de fase" (`CLAUDE.md` rule 8).
3. The contract does not exist yet. RFC-0009 sections 4.5/4.6 are a
   one-paragraph sketch: they name objects and endpoints but define no
   request/response shapes, error model or idempotency semantics. Coding now
   would be "codificación a ciegas" (`CLAUDE.md` rules 0, 1, 7).
4. RFC-0024 was accepted on 2026-11-21, after RFC-0009. It defines
   capability-scoped, deny-by-default agent access, plan/apply for mutations,
   one executor across surfaces and secret redaction by construction. None of
   that is reflected in RFC-0009. Designing the HTTP surface without it would
   create guaranteed rework.
5. Any new `ag mail api ...` CLI commands require an RFC (`CLAUDE.md` rule
   28); none exists.

The missing deliverable is therefore this design, not a partial
implementation.

## 2. Backend-only scope

The managed API is an HTTP surface exposed by a generated Anti-Gravital
project (or the `ag` dev runtime) when the `api` Cargo feature is enabled.
It is **not** a hosted service, a mail server, or a UI.

- It integrates with the native MTA (`MtaSender`) and the default relay
  (`SmtpSender`) through the existing `MailSender` abstraction. The native
  default and the ESP path stay unchanged (`ADR-0009`, RFC-0009 section 3D).
- The data model uses local/ephemeral PostgreSQL 16 or a CI service
  container. No third party is involved. The debt is the unimplemented
  surface, not a missing environment.
- Inbound remains limited to DSN/ARF for bounce and complaint processing,
  plus the RFC 8058 one-click unsubscribe endpoint. Mailboxes, IMAP, POP and
  general inbound reception stay out of scope (`ADR-0010`).
- Names are generic; no third-party brand appears in endpoints, capability
  names, feature names or identifiers (`ADR-0011`). The existing generic
  `SmtpSender` is the way to point at any external provider.

## 3. HTTP surface

All routes are versioned under `/v1`. JSON only. A tenant is resolved from
the API key; there is no tenant id in the path, so one key cannot address
another tenant's objects. Every list endpoint is paginated with
`?limit` (default 50, max 200) and `?cursor`, returning `next_cursor`.

### 3.1 Transactional and core objects

| Method | Path | Purpose | Capability |
| --- | --- | --- | --- |
| POST | `/v1/emails` | Queue one email | `mail.send` |
| POST | `/v1/emails/batch` | Queue up to 100 emails | `mail.send` |
| GET | `/v1/emails/{id}` | Read status and events | `mail.read` |
| POST | `/v1/emails/{id}/cancel` | Cancel a scheduled/queued email | `mail.send` |
| GET | `/v1/domains` | List sending domains | `mail.read` |
| POST | `/v1/domains` | Register a sending domain | `mail.manage` |
| POST | `/v1/domains/{id}/verify` | Trigger DNS verification | `mail.manage` |
| GET | `/v1/api-keys` | List key metadata (never secrets) | `mail.manage` |
| POST | `/v1/api-keys` | Create a key; plaintext returned once | `mail.manage` |
| DELETE | `/v1/api-keys/{id}` | Revoke a key | `mail.manage` |
| GET | `/v1/webhooks` | List endpoint metadata (never secrets) | `mail.manage` |
| POST | `/v1/webhooks` | Register endpoint; `signing_secret` once | `mail.manage` |
| DELETE | `/v1/webhooks/{id}` | Remove endpoint | `mail.manage` |
| GET | `/v1/events` | Append-only delivery events for the tenant | `mail.read` |

### 3.2 Marketing objects (stage 4.6-D)

| Method | Path | Purpose | Capability |
| --- | --- | --- | --- |
| GET/POST | `/v1/contacts` | List / create a contact | `marketing.read` / `marketing.write` |
| POST | `/v1/contacts/batch` | Upsert up to 1000 contacts | `marketing.write` |
| GET | `/v1/audiences` | List audiences | `marketing.read` |
| POST | `/v1/audiences` | Create an audience | `marketing.write` |
| GET/POST | `/v1/segments` | List / create a segment (filter expression) | `marketing.read` / `marketing.write` |
| GET/POST | `/v1/topics` | List / create a subscription topic | `marketing.read` / `marketing.write` |
| GET/POST | `/v1/broadcasts` | List / create a broadcast draft | `marketing.read` / `marketing.write` |
| POST | `/v1/broadcasts/{id}/send` | Queue a broadcast to its audience | `mail.send.broadcast` |
| POST | `/v1/broadcasts/{id}/cancel` | Stop a running broadcast | `mail.send.broadcast` |
| POST | `/v1/unsubscribe/one-click` | RFC 8058 `List-Unsubscribe-Post` target | public, signed token |

Contacts, audiences, segments, topics and broadcasts are tenant-scoped and
soft-deletable. A broadcast holds a reference to a typed template (DSL v0.7
`mail` block) and an audience/segment; no new DSL version is required
(RFC-0009 section 4.7).

### 3.3 Request and response shapes

`POST /v1/emails`:

```json
{
  "from": "hello@example.test",
  "to": ["reader@example.net"],
  "cc": [],
  "subject": "Welcome",
  "template": "WelcomeEmail",
  "variables": { "name": "Ada" },
  "text_body": "Welcome, Ada",
  "html_body": "<p>Welcome, Ada</p>",
  "metadata": { "source": "signup" },
  "send_at": "2026-11-22T10:00:00Z"
}
```

`template` plus `variables` uses the existing typed templates; `text_body` /
`html_body` are the explicit alternative. Supplying both is a validation
error. Successful response `201`:

```json
{
  "id": "em_01HZ...",
  "status": "queued",
  "tenant_id": "tn_01HZ...",
  "created_at": "2026-11-21T12:00:00Z",
  "idempotent_replay": false
}
```

`GET /v1/emails/{id}` returns the same object with `status` advanced through
`queued | scheduled | sending | sent | failed | canceled | suppressed` and
an `events` array (`type`, `timestamp`, `data`). Event types are stable
strings such as `email.queued`, `email.delivered`, `email.bounced`,
`email.complained`, `email.unsubscribed`; the same names are used in signed
webhooks, so one vocabulary covers both surfaces.

`POST /v1/webhooks` request `{ "url": "https://...", "events": ["email.delivered"] }`;
response `201` includes `id`, `url`, `events`, and `signing_secret` (`whsec_`
form) **only once**. Subsequent GET/POST responses omit it. The signing and
verification scheme is the already implemented `api::webhook` (HMAC-SHA256
over `{id}.{timestamp}.{payload}`, `v1,<base64>` headers, multi-signature and
replay-window verification, constant-time compare).

`POST /v1/api-keys` response includes the plaintext `key` **only once** and
its `capabilities` array; later reads return metadata only. Keys are hashed
with BLAKE3 via the existing `ag-auth` hashing primitive (the `ag-mail` crate
depends on the primitive, not on `ag-auth` itself; `ADR-0007` sixth rule).

## 4. Error model

All errors use `application/problem+json` (RFC 9457) with a stable machine
code, so scripts and agents never parse prose:

```json
{
  "type": "https://errors.antigravital.dev/validation",
  "title": "Invalid request",
  "status": 400,
  "detail": "provide either template+variables or text_body/html_body, not both",
  "instance": "/v1/emails",
  "code": "validation_conflicting_body",
  "request_id": "req_01HZ..."
}
```

| Condition | Status | `code` |
| --- | --- | --- |
| Missing/invalid API key | 401 | `auth_invalid_key` |
| Key lacks the route capability | 403 | `auth_capability_denied` |
| Malformed or contradictory body | 400 | `validation_*` (specific suffix) |
| Idempotency key reused with a different body | 409 | `idempotency_conflict` |
| Relay/MTA temporarily unavailable | 503 | `delivery_unavailable` (with `Retry-After`) |
| Rate limit exceeded via `ag-core` | 429 | `rate_limited` (with `Retry-After`) |
| Unknown object id | 404 | `not_found` |

Secrets, API key material and webhook signing secrets never appear in
`detail`, `request_id`, logs or debug resources (`RFC-0024` principle 5).

## 5. Idempotency

- Applies to `POST /v1/emails`, `/v1/emails/batch`, `/v1/contacts/batch`,
  `/v1/broadcasts/{id}/send` and `/v1/webhooks`.
- Caller supplies `Idempotency-Key`: 1-256 characters, 24h TTL (RFC-0009
  section 4.6).
- Scope is `(tenant_id, api_key_id, route, idempotency_key)`. A request is
  fingerprinted with BLAKE3 over the canonical JSON body, excluding volatile
  fields.
- Same key + same fingerprint within TTL replays the stored response with
  `idempotent_replay: true` and the original status code; no second delivery
  is queued.
- Same key + different fingerprint returns `409 idempotency_conflict`.
- The idempotency record is written transactionally with the queued job and
  the event row, so a crash cannot produce a delivery without a record.
- Batch sends are idempotent as a unit: one key covers the whole batch, and
  partial results are returned in the response body.

## 6. Capability-scoped access for agents (RFC-0024)

Two caller classes share one executor; they cannot drift (`RFC-0024`
principle 3):

1. **HTTP callers** present a BLAKE3 API key carrying an explicit
   `capabilities` array. Default set for a new key is `mail.read` only -
   deny by default.
2. **Agents** connect through `ag mcp serve` (stdio, opt-in, RFC-0024) with a
   local manifest granting capabilities. The default is `read`. A tool is
   unavailable if the capability is absent; it is not merely rejected after
   the fact.

Capability set proposed for this surface (to be added to the RFC-0024
contract by amendment, because RFC-0024 currently defines only the
`project`, `schema`, `config`, `workers`, `deploy` and `debug` groups):

| Capability | Grants |
| --- | --- |
| `mail.read` | email status/events, domains, webhook metadata |
| `mail.send` | `/emails`, `/emails/batch`, cancel |
| `mail.manage` | domains, api-keys, webhooks |
| `mail.send.broadcast` | broadcast send/cancel |
| `marketing.read` | contacts, audiences, segments, topics, broadcasts |
| `marketing.write` | create/upsert marketing objects |

Rules, inherited from RFC-0024:

- An MCP tool is a strict subset of the API key capabilities presented; the
  server holds no ambient authority. An agent cannot escalate through MCP.
- Mutating tools return a plan identifier first and require explicit apply
  (RFC-0021 plan/apply), and are recorded in the shared local audit log.
- `mail.read` does not imply `marketing.read`; scopes are additive and
  explicit.
- Webhook secrets and API key plaintext are returned once at creation, never
  in tool results, resources, errors or the `AG_MCP_DEBUG=1` debug group.
- Tool names use verbs and nouns (`email.send`, `audience.list`,
  `broadcast.send`), never product or brand names (`ADR-0011`).

## 7. Developer ergonomics (CLAUDE.md rule 32)

The design must preserve `ag new -> ag dev`:

- The basic scaffold and dev loop do not require the `api` feature, a
  database, or any mailbox. `ag new my-api --template rest` followed by
  `ag dev` keeps working exactly as today.
- `api` is opt-in. When enabled in a generated project, the managed API is
  served by the project binary on loopback next to the app; local dev uses an
  ephemeral PostgreSQL and the default relay sender, so no third party is
  contacted.
- The existing `ag mail test --to ADDRESS` remains the only command needed
  to validate SMTP setup. No new command is required for the first-API
  experience.
- Any future `ag mail api ...` commands (key issue/list/revoke, webhook
  register, capability grants for an agent) are a separate RFC per rule 28
  and are intentionally out of this design, so they cannot block the REST
  slice or the dev-loop guarantee.

## 8. Exact gate condition to start implementation

All of the following must hold before code is written:

1. **Sequencing evidence (RFC-0009 section 5, Mail A gate):** a message
   delivered by the native MTA to a real external mailbox passing aligned
   SPF, DKIM and DMARC, with date, commit and headers recorded. This is
   currently blocked by the lack of outbound port 25 (DEBT-022 / issue #153).
   *Alternative:* an RFC-0009 amendment formally reorders or defers C/D,
   which is the other sanctioned outcome of Hoja de Ruta section 4.6.3.
2. **Contract accepted:** this document (sections 3-5) accepted as the API
   contract, so endpoint names, shapes, problem codes and idempotency
   semantics are fixed before implementation.
3. **RFC-0024 amendment:** the mail/marketing tool groups and the six
   capabilities in section 6 are added to the shared MCP contract, so agent
   access is designed before code.
4. **CLI decision:** any `ag mail api ...` command is either deferred (the
   recommended v1 choice) or covered by its own RFC (rule 28).
5. **Test plan fixed:** acceptance tests run against a local/ephemeral
   PostgreSQL 16 or a CI service container, with no third party; `cargo test
   -p ag-mail --features api` and the `mail-*` CI jobs green; the native
   default and the relay sender unchanged (`ADR-0009`).

When these hold, issue #152 "Done when" is executable: REST routes and data
model land behind `api` with idempotency, marketing objects land behind
`api`, tests cover them against local PostgreSQL, and DEBT-021 already points
to #152.

If instead the BDFL chooses the amendment path, this document is the input to
that RFC-0009 amendment, #152 is re-targeted to a later phase, and no code is
written in Phase 4.6.
