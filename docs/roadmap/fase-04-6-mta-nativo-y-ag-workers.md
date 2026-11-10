# Fase 4.6 - Endurecimiento aditivo: MTA nativo y ag-workers

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-5-ag-mail-y-ag-domains.md](./fase-04-5-ag-mail-y-ag-domains.md)
> Siguiente: [fase-04-7-experiencia-interactiva-de-terminal.md](./fase-04-7-experiencia-interactiva-de-terminal.md)

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

