# ADR-0010 - ag-mail native outbound MTA pivot

- Status: accepted
- Date: 2026-06-03
- Source RFC: RFC-0009
- Author: Angel Nereira (BDFL), Gravital Labs
- Supersedes: the v1 scope restriction of `ag-mail` fixed in `ADR-0007`
  (section "Lo que `ag-mail` NO hace en v1"). `ADR-0007` otherwise stands:
  its `ag-domains` decision, the deferred-standard classification, and the
  `ag-auth -> ag-mail` dependency direction are unchanged.
- Affected crates: `ag-mail` (scope expansion); consumers `ag-auth`,
  `ag-cloud`; cooperators `ag-domains`, `ag-realtime`, `ag-data`,
  `ag-observe`. Core (`ag-core`, `ag-dsl`, `ag-cli`, `ag-wasm-host`) unchanged.
- Master documents touched: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  (section 8.8), `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (forward note).

## Context

`ADR-0007` introduced `ag-mail` (Phase 4.5) with a deliberately narrow v1
scope: outbound transactional email only, native `lettre` SMTP relay plus
optional provider HTTP adapters. That ADR fixed, as "out of scope,
not deferrable", the following: complete MTA, inbound reception (IMAP/POP),
persistent mailboxes, antispam/IP reputation, and bounce handling beyond
logging. RFC-0006 turned that scope into the implemented crate that exists
today.

> Note (`ADR-0011`, 2026-06-04): the provider-brand adapters mentioned in this
> ADR were removed by `ADR-0011`. The external-provider path is now the native
> `SmtpSender` relay pointed at the provider's SMTP endpoint, so no capability
> is lost. Read "provider adapters" below as that native relay path.

The implemented baseline is real and works: `MailSender` trait, native
`SmtpSender` (default relay), an in-memory retry queue with an optional
persistent backend (`queue-persistent` via `ag-data`), typed templates
validated at build time, and `ag-observe` metrics. `ag-auth` consumes it for
verification,
recovery, and magic links.

A new product requirement has been raised by the BDFL: a native, self-hosted,
**provider-independent** outbound email path. Today `ag-mail` can only relay
to an SMTP host or call an external email API; in both cases delivery depends
on a third-party sending service. The requirement is to send authenticated
mail **directly** to recipient mail servers with no third party in the
sending path. To offer that, `ag-mail` must grow a native outbound MTA engine
(MX resolution, delivery queues, traffic shaping, DKIM signing, bounce
classification) plus a managed email-sending API surface for multi-tenant use.

External email platforms are used only as an engineering reference for *which*
capabilities a mature email sender provides; none is a dependency or a target
to integrate with. The existing optional provider adapters remain available
for projects that choose them, but they are off by default and never required.

This requirement directly contradicts the v1 scope restriction of `ADR-0007`.
Per `CLAUDE.md` rules 5, 22, and 28, a scope reversal of this magnitude
cannot start from code: it requires a governing decision (this ADR) and a
technical plan (RFC-0009) before any implementation. This ADR records that
decision.

## Decision

`ag-mail` is expanded from an outbound transactional relay into a native,
self-contained outbound MTA, delivered across phases, **additively**: the
expansion only adds capability behind opt-in Cargo features and never removes,
demotes, or changes the behavior of anything the crate already ships.

This additive-only constraint is binding and overrides the framing of the
research blueprint that motivated this ADR. The blueprint proposed to
*overwrite* the crate — make the native MTA the default sender, relegate the
provider relay/third-party SMTP to a "not for production" feature, and
*migrate* the in-memory/`ag-data` queue to JetStream. That degradation is
**rejected**. Concretely, the following are preserved unchanged and remain
the defaults:

- the default Cargo features (`smtp`, `templates`, `metrics`) and the default
  `SmtpSender` (lettre relay) as the out-of-the-box sender;
- the `MailSender` trait, the `AgMail` enum, `NullSender`, and the native
  `SmtpSender` relay (the path to any external provider) as first-class,
  production-supported senders (no demotion);
- the in-memory retry queue as the default backend and the optional
  `ag-data` persistent queue (`queue-persistent`); JetStream is an
  **additional** optional backend, not a replacement;
- the typed compile-time templates, the `ag-auth -> ag-mail` integration, and
  the `ag mail test` CLI.

Concretely, the expansion:

1. The v1 "NOT an MTA / inbound never" restriction of `ADR-0007` is
   **superseded**. `ag-mail` may now resolve destination MX records, open
   ESMTP+STARTTLS sessions directly to recipient mail servers, sign DKIM on
   the outbound path, classify bounces (synchronous SMTP codes and
   asynchronous DSNs), and maintain delivery and suppression state.

2. The expansion follows the existing **Native | Adapter** pattern. The new
   MTA engine is added as a **new** opt-in sender (a `MtaSender` behind the
   `mta` feature), alongside — not in place of — the existing default
   `SmtpSender` relay (the path to any external provider via its SMTP
   endpoint), which remains first-class and production-supported. This keeps
   Blueprint section 3.3 (interoperability) satisfied: a project that prefers
   an external provider keeps using one with no change; a project that wants
   zero third
   parties opts into the native MTA. Which sender is the default is the
   integrator's choice; the crate's own default does not change.

3. The expansion is **opt-in and self-sufficient** per `ADR-0009`. Every
   external integration (durable queue over NATS/JetStream, PostgreSQL state
   mirror, S3 for attachments, FBL/ARF intake) sits behind a Cargo feature,
   and the crate keeps a native default mode that does not require any
   external service to send a message.

4. The dependency direction and interop rules are **unchanged**: `ag-mail`
   still does NOT depend on `ag-auth`; DKIM/SPF/DMARC DNS materialization
   stays a cooperation with `ag-domains` (no cycle); the DKIM private-key and
   signing responsibility is shared with `ag-domains` per RFC-0009 section 4.

5. Inbound is admitted **only** to the extent required for outbound
   correctness and deliverability: parsing asynchronous DSNs (RFC 3464) and
   feedback-loop ARF reports for bounce/complaint processing and suppression.
   Full mailbox hosting, IMAP/POP, JMAP, and a general inbound mail server
   remain out of scope and are still the job of Postfix/Stalwart.

The managed email-sending product surface (multi-tenant REST API, webhooks,
suppression lists, broadcasts/contacts/segments, IP pools) is admitted as the
target but is phased; see RFC-0009 for the phase plan and exit gates. No new
crate is created: `ag-mail` absorbs the work. The ecosystem stays at 17-18
crates.

## Consequences

Positive:

- A project can send authenticated transactional mail (SPF+DKIM+DMARC
  aligned) directly to Gmail/Outlook/Yahoo with no third party in the
  sending path, closing the "self-contained Anti-Gravital layer" narrative.
- The Native | Adapter pattern is preserved, so existing users of the ESP
  adapters and of `ag-auth -> ag-mail` are unaffected.
- DKIM signing and DNS materialization reuse the existing `ag-domains`
  cooperation; durable queues and the event bus reuse `ag-realtime`/`ag-data`;
  no new architectural concept is invented.

Negative:

- `ag-mail` grows from a library into something closer to a service:
  operating a native MTA means owning IP/domain reputation, warm-up, and
  deliverability — the most expensive and highest-risk part of running an
  email sender, normally delegated to a hosted service. This is acknowledged
  as the dominant risk; mitigation is to keep the optional provider adapters
  available as a production path until native deliverability is proven, and to
  phase the MTA behind opt-in features.
- The crate's surface and test matrix expand substantially (REST API,
  webhooks, suppression, IP pools). Complexity must be contained by the
  feature-gating discipline of `ADR-0009`.
- New external crates (`mail-send`, `mail-builder`, `mail-auth`,
  `mail-parser`, `hickory-resolver`) enter the dependency set; each is
  pinned and justified in RFC-0009 section 4.
- The estimated schedule for the full managed email surface is large
  (RFC-0009 estimates five phases); the roadmap reflects this as forward work,
  not as a completed Phase 4.5 deliverable.

Neutral:

- Phase 4.5 stays "technically complete" for its original outbound-relay
  scope. The MTA work is new, forward roadmap, tracked separately so the
  documented status never claims a capability that is not yet implemented
  (`CLAUDE.md` rule 26).
- `ADR-0007` remains the governing decision for `ag-domains` and for the
  deferred-standard classification.

## Alternatives considered

A. **Keep `ADR-0007` scope, stay an ESP relay only.** Cheapest and safest for
   deliverability, but does not satisfy the requirement of provider-independent
   sending. Rejected by the BDFL for the stated product goal; the ESP path is
   retained as the recommended production default, not removed.

B. **Build the MTA as a new separate crate (e.g. `ag-mta`).** Cleaner blast
   radius, but splits the `MailSender` abstraction, the queue, and the
   `ag-domains`/`ag-auth` cooperation across two crates and breaks the single
   Native | Adapter surface. Rejected: `ag-mail` already owns sender, queue,
   templates, and DNS cooperation; the MTA is the native implementation of the
   sender it already exposes.

C. **Wrap an external open-source MTA (KumoMTA/Postfix) as a subprocess.**
   Fast, but reintroduces an external operational dependency and contradicts
   the "self-contained, auditable Rust" goal; deployment and observability
   stop being native. Rejected. KumoMTA remains the architectural reference
   model, not a runtime dependency.

D (chosen). **Expand `ag-mail` in place, phased, behind opt-in features,
   preserving the Native | Adapter pattern and backward compatibility.**
   Keeps one abstraction, one crate, one dependency direction; honors
   `ADR-0009` (native default, external optional) and Blueprint 3.3 (adapters
   remain). Records honestly that this turns `ag-mail` into a heavier module
   with real operational cost.

## Notes

- Technical plan, dependencies, data model, REST surface, phase plan, risks,
  and rollback: `docs/rfc/RFC-0009-ag-mail-native-mta.md`.
- Governing precedent for the scope being superseded:
  `docs/adr/0007-ag-mail-ag-domains.md`, `docs/rfc/RFC-0006-ag-mail-alcance.md`.
- `ADR-0009` (corrective governance): native default + external-behind-feature
  is mandatory for every new integration in this expansion.
- Architecture specification updated: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
  section 8.8 and its verbatim derivative
  `docs/architecture/08-modulos-batteries-included.md`.
- `CLAUDE.md` rules 5 (architectural lock), 12 (interoperability), 22 (RFC),
  26 (code <-> documentation sync), 28 (RFC before changing module boundaries).
