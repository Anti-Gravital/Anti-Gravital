# RFC-0009: ag-mail native outbound MTA - scope, stack and phased plan

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-03
- Target phase: Phase 4.6 (additive, post Phase 4.5; precedes Phase 5 gates as
  far as the ESP path; native MTA hardening continues into Phase 5 and beyond)
- Affected modules or crates: `ag-mail` (scope expansion), `ag-domains`
  (DKIM cooperation), `ag-realtime` (durable queue + event bus, optional),
  `ag-data` (PostgreSQL state, optional), `ag-storage` (attachments, optional),
  `ag-auth` (unchanged consumer), `ag-dsl` (template/domain blocks, already v0.7)
- Predecessor RFC: RFC-0006 (superseded for the `ag-mail` scope it fixed)
- Governing ADR: `docs/adr/0010-ag-mail-native-mta-pivot.md`
- Comment period: waived by the BDFL; this RFC formalizes the technical detail
  of `ADR-0010`, an already-approved decision, as `CLAUDE.md` rule 28 requires
  an RFC before changing module boundaries.

## 1. Motivation

`ADR-0010` decided to expand `ag-mail` from an outbound transactional relay
into a native, provider-independent outbound MTA, while keeping the ESP
adapters as the "integrate, do not replace" path. This RFC turns that decision
into a verifiable plan: which libraries are added, which subsystems compose the
expanded crate, what the data model and REST surface look like, how the work is
phased, and how backward compatibility is preserved.

Without this RFC the implementation would start with ad-hoc technical choices
that contradict `CLAUDE.md` rule 28.

## 2. Problem

`ADR-0010` defines the **what** (a native MTA, phased, opt-in, behind the
existing Native | Adapter pattern). It does not define the **how**: the
concrete Stalwart `mail-*` libraries, the two-tier queue layout, the native
default spool vs the optional JetStream/PostgreSQL backends, the DKIM
private-key ownership split with `ag-domains`, the multi-tenant REST surface,
the webhook signing scheme, and the order of pull requests. Without agreement
on these, the implementer improvises and PR review becomes expensive.

The research baseline behind this RFC (Resend internals, the KumoMTA queueing
model, bulk-sender requirements from Gmail/Yahoo/Microsoft, the Rust mail
ecosystem) is recorded in the engineering blueprint that accompanies this RFC.
That blueprint is research input, not a specification; this RFC is the
specification and supersedes any factual drift in it (notably: the current
crate already ships `SesSender`, `PostmarkSender`, and a persistent queue).

## 3. Alternatives considered

### A. Do nothing (stay an ESP relay)

Advantage: zero deliverability risk, smallest surface.

Disadvantage: does not meet the provider-independent goal of `ADR-0010`.
Rejected at the ADR level; the ESP path is retained as the recommended
production default, not as the only option.

### B. Wrap an external MTA (KumoMTA/Postfix) as a subprocess

Advantage: proven deliverability engine, fast.

Disadvantage: reintroduces an external operational dependency, breaks the
self-contained Rust goal, and makes observability/deployment non-native.
Rejected. KumoMTA is the reference model for the queue design, not a runtime.

### C. New separate crate `ag-mta`

Advantage: contained blast radius.

Disadvantage: splits `MailSender`, the queue, and the `ag-domains`/`ag-auth`
cooperation across two crates; breaks the single Native | Adapter surface.
Rejected (see `ADR-0010` alternative B).

### D (chosen). Expand `ag-mail` in place, phased, behind opt-in features

The native MTA is added as a new opt-in `MtaSender` (`mta` feature), alongside
the existing default `SmtpSender` relay and the ESP adapters, which are kept
unchanged. Durable queue, PostgreSQL state, REST API, webhooks, and IP pools
are each behind features with a native default. One crate, one abstraction,
one dependency direction.

This RFC is **additive-only** per `ADR-0010`: it must not remove, demote, or
change the behavior of any existing public item. The default Cargo features,
the default `SmtpSender`, the `MailSender`/`AgMail`/`NullSender` API, the
adapters, the in-memory and `ag-data` queues, the typed templates, and the
`ag-auth` integration all stay exactly as shipped. Every item below is a new,
feature-gated addition. The blueprint's "make the MTA the default / demote
Resend / migrate the queue" framing is explicitly out of scope.

## 4. Proposed design

### 4.1 Subsystems

The expanded crate is organized in six subsystems, layered on the existing
module tree (`message`, `sender`, `queue`, `template`, `metrics`, `error`):

```
crates/ag-mail/src/
|- sender/
|  |- smtp.rs          # existing relay sender (kept)
|  |- mta/             # NEW native MTA engine               [feature "mta"]
|  |  |- resolve.rs    # MX lookup (hickory-resolver), site_name rollup
|  |  |- session.rs    # ESMTP + STARTTLS via mail-send, connection pooling
|  |  |- shaping.rs    # per site_name / egress source throttling
|  |  |- egress.rs     # egress sources, IP pools, weighted round-robin
|  |  |- bounce.rs     # SMTP code + DSN classification (IANA TOML table)
|  |- resend.rs / ses.rs / postmark.rs   # adapters (kept, feature-gated)
|- auth/               # NEW DKIM signing + SPF/DKIM/DMARC/ARC [feature "mta"]
|- queue/
|  |- mod.rs           # native in-memory two-tier queue (default)
|  |- store.rs         # existing ag-data persistence (kept)
|  |- jetstream.rs     # NEW durable scheduled queue          [feature "queue-jetstream"]
|- api/                # NEW multi-tenant REST surface (axum)  [feature "api"]
|- webhook/            # NEW Svix-compatible signed webhooks   [feature "api"]
|- template/           # existing typed templates (kept) + bulletproof components
```

The native default (`mta` off) is the current behavior: relay via `SmtpSender`
or an ESP adapter, in-memory or `ag-data` queue. With `mta` on, the native
engine resolves MX and delivers directly.

### 4.2 Queue model (KumoMTA-inspired)

- **Scheduled queue**, keyed `tenant:campaign:domain`: holds retry state
  (backoff, max age). Native default is the existing in-memory/`ag-data`
  queue; optional durable backend over NATS/JetStream behind
  `queue-jetstream`, mirrored to PostgreSQL.
- **Ready queue**, keyed `egress_source + site_name`, in memory, with
  `max_ready`; overflow re-schedules with a "Delayed" log.
- **site_name rollup**: destination MX hostnames are merged into one
  `site_name` so throttling applies per shared-MX destination set.

### 4.3 Authentication and identity

DKIM signing on the outbound path uses `mail-auth`. Key ownership is split:
`ag-domains` already generates and publishes SPF/DKIM/DMARC DNS records and
owns the DKIM key material lifecycle; `ag-mail` consumes the active private
key to sign. **DKIM is signed last, after any link rewriting**, to avoid
breaking the signature. SPF/DKIM/DMARC/ARC verification of inbound DSN/ARF is
also via `mail-auth`. No cycle: `ag-mail` still depends on `ag-domains`, never
the reverse.

### 4.4 New dependencies (feature-gated)

| Crate | Version | Feature | Justification |
|---|---|---|---|
| `hickory-resolver` | (workspace, already via `ag-domains`) | `mta` | Async MX resolution and site_name construction. |
| `mail-send` | pin in workspace | `mta` | ESMTP+STARTTLS client, DKIM-capable, async Tokio. |
| `mail-builder` | pin in workspace | `mta` | RFC 5322/MIME message construction. |
| `mail-auth` | 0.8.x | `mta` | DKIM/SPF/DMARC/ARC sign+verify, DMARC/TLS-RPT reports. |
| `mail-parser` | pin in workspace | `mta` | Parse inbound DSN and ARF for bounce/complaint processing. |
| `axum` | (workspace) | `api` | Multi-tenant REST API. |
| `hmac`/`sha2` | (workspace) | `api` | Svix-compatible webhook signatures. |

All Stalwart `mail-*` crates are Apache-2.0 OR MIT, pinned in the workspace,
with active upstream watch (precedent: `ADR-0006`). `lettre` stays for the
relay `SmtpSender`. No dependency is required to use `ag-mail` in its default
mode (`ADR-0009` rule 2 and 5).

### 4.5 Data model (PostgreSQL via sqlx, optional)

Behind `queue-persistent`/`api`, the schema mirrors Resend's objects:
`tenants`, `api_keys` (BLAKE3 via `ag-auth` hashing), `domains`, `dkim_keys`,
`emails`, `events` (append-only), `suppressions`, `webhooks`,
`webhook_deliveries`, `ip_pools`/`ip_addresses`, `delivery_queue` (scheduled
queue mirror); marketing objects (`contacts`, `audiences`/`segments`,
`topics`, `broadcasts`) land in the marketing phase. Migrations are embedded
(existing `ag-mail/migrations` pattern). The native default mode does not
require this schema.

### 4.6 REST surface (drop-in conceptual vs Resend)

`POST /emails`, `POST /emails/batch`, `GET /emails/{id}`,
`POST /emails/{id}/cancel`, `/domains` (+`/verify`), `/api-keys`,
`/audiences` + `/contacts`, `/broadcasts` (+`/send`), `/webhooks`
(returns `signing_secret` once). Idempotency via `Idempotency-Key`
(1-256 chars, 24h TTL). Webhooks are HMAC-SHA256 signed over
`{id}.{timestamp}.{payload}` with a `whsec_` secret and `svix-*` headers,
at-least-once with dedupe by id.

### 4.7 DSL

No new DSL version is required for the MTA engine itself; the v0.7 `mail` /
`domain` / `template` blocks already cover declaration. Traffic-shaping and
egress-pool configuration are TOML/runtime config, not DSL, to avoid a v0.8
DSL change inside this RFC (a DSL change would need its own RFC per rule 28).

### 4.8 CI and master documentation

- New CI jobs gated on the `mta` feature: MX-resolution unit tests against a
  fixture resolver, an end-to-end "deliver to a local sink MTA" smoke test,
  and a DKIM sign/verify round-trip. Third-party adapter tests stay `#[ignore]`
  behind secrets, as today.
- Master documents: `Arquitectura-Tecnica` section 8.8 records the pivot and
  points here; `Hoja-de-Ruta` adds the forward Phase 4.6 note. Neither claims
  the MTA exists until the corresponding phase ships (rule 26).

## 5. Implementation plan

Phased; each phase is a gate and ships its own PRs. Implementation is NOT part
of this RFC's merge - this RFC and `ADR-0010` are the documentation-first
deliverable.

1. **Phase 4.6-A - MTA core PoC** (`mta` feature): MX resolution + site_name,
   ESMTP+STARTTLS via `mail-send` to a real MX, DKIM signing via `mail-auth`,
   basic bounce classification (4xx retry / 5xx suppress). Gate: deliver to a
   test Gmail/Outlook mailbox passing aligned SPF+DKIM+DMARC.
2. **Phase 4.6-B - durable queues + data**: two-tier queue, optional
   JetStream backend + PostgreSQL mirror, automatic suppression lists, per
   site_name traffic shaping. Gate: retries survive restart; provider
   throttling verifiable.
3. **Phase 4.6-C - REST API + webhooks + multi-tenancy**: axum API with
   BLAKE3 API keys, idempotency, batch, scheduling; Svix-compatible signed
   webhooks; per-tenant egress IP pools with weighted round-robin warm-up.
   Gate: parity on `emails`, `domains`, `api-keys`, `webhooks`.
4. **Phase 4.6-D - marketing + templates**: audiences/contacts/segments/
   topics, broadcast API, RFC 8058 one-click unsubscribe, native bulletproof
   template components, per-tenant tracking domain (single redirect, HTTPS,
   disableable). Gate: a broadcast with working unsubscribe.
5. **Phase 5+ - deliverability hardening (continuous)**: MTA-STS, TLS-RPT,
   ARC, BIMI, Google Postmaster + Yahoo/Microsoft FBL intake, automated IP
   warm-up. Operating thresholds: complaints < 0.1%, hard bounce < 2%.

Each PR requires `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
`cargo audit`, `cargo deny check` green, and updates the affected
documentation in the same PR (`ADR-0009` rule 3).

## 6. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Poor deliverability from new native IPs | High | High (mail not delivered) | Keep ESP adapters as recommended production default; phased IP warm-up; native MTA opt-in until proven. |
| Scope creep into a full mail server | Medium | High (becomes a different project) | Inbound limited to DSN/ARF for bounce processing; mailboxes/IMAP/POP stay out per `ADR-0010`. |
| Stalwart `mail-*` API churn | Medium | Medium | Pin versions, contract tests around `mail-send`/`mail-auth`, active upstream watch. |
| `ag-mail` surface explosion | Medium | Medium | Strict Cargo feature gating (`mta`, `api`, `queue-jetstream`); native default stays small. |
| Reputation/abuse on multi-tenant IP pools | Medium | High | Per-tenant pools, suppression enforcement, rate limits via `ag-core`; complaints monitored < 0.1%. |

## 7. Impact

- **Product scope:** expands `ag-mail` per `ADR-0010`; the ESP relay scope of
  RFC-0006 is preserved as a subset.
- **Roadmap:** adds forward Phase 4.6 (A-D) plus continuous Phase 5+
  hardening. Phase 4.5 stays complete for its original scope.
- **Operational complexity:** significant for the native MTA; contained by
  feature gating and by keeping ESP adapters the default production path.
- **Public APIs:** `MailSender` trait, `AgMail` enum, `NullSender`,
  `SmtpSender`, and the existing adapters are preserved unchanged; new surface
  is additive and feature-gated.
- **Existing documentation:** RFC-0006 superseded for scope; `ADR-0007`
  partially superseded (ag-mail scope only); module README, architecture 8.8,
  roadmap, and root README updated in this same change set.

## 8. Rollback

The native MTA is behind the `mta` feature. If native deliverability proves
unviable, disabling the feature returns `ag-mail` to the RFC-0006 relay
behavior with zero API change; the ESP adapters remain the supported path. If
the whole expansion is abandoned, removing the `mta`/`api`/`queue-jetstream`
modules is safe because no other crate depends on them (the `ag-auth -> ag-mail`
integration uses only the `MailSender` trait). Rollback signal: native-path
complaint rate above 0.3% or hard-bounce above 2% that warm-up cannot recover.

## 9. Decision

- Decider: Angel Nereira (BDFL)
- Decision date: 2026-06-03
- Outcome: accepted
- Justification: formalizes the technical detail of `ADR-0010` without
  contradicting its constraints (Native | Adapter preserved, native default
  per `ADR-0009`, no dependency on `ag-auth`, DKIM cooperation with
  `ag-domains`, interoperability via retained adapters). Satisfies `CLAUDE.md`
  rule 28.

## 10. References

- `docs/adr/0010-ag-mail-native-mta-pivot.md` - governing decision.
- `docs/adr/0007-ag-mail-ag-domains.md`, `docs/rfc/RFC-0006-ag-mail-alcance.md`
  - superseded scope.
- `docs/adr/0009-gobernanza-correctiva.md` - native default + feature-gated
  external integrations.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` section 8.8 - specification.
- `docs/modules/ag-mail/README.md` - module sheet.
- `CLAUDE.md` rules 5, 12, 14, 15, 22, 26, 28.
