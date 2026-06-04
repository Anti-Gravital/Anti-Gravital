# ag-mail

> Architecture chapter: `docs/architecture/08-modulos-batteries-included.md`, section 8.8.
> Governing decisions: `docs/adr/0007-ag-mail-ag-domains.md` (introduction) and
> `docs/adr/0010-ag-mail-native-mta-pivot.md` (native MTA scope expansion).
> Technical plans: `docs/rfc/RFC-0006-ag-mail-alcance.md` (relay, superseded for
> scope) and `docs/rfc/RFC-0009-ag-mail-native-mta.md` (native MTA).
> Roadmap: `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md` (implemented baseline)
> and the forward Phase 4.6 note in the master roadmap.
> Crate README: `crates/ag-mail/README.md`.
> Criticality: deferred standard.

## Domain

Outbound transactional communication: account verification, magic links,
password recovery, alerts and notifications. Native SMTP sender and managed
provider adapters (Resend, SES, Postmark). Typed HTML/plaintext templates
validated at compile time against `schema.ag`. Async queue with retries and
metrics.

`ADR-0010` expands this domain: `ag-mail` is growing a native outbound MTA so
projects can send authenticated mail directly to recipient servers with no
third party in the sending path. The expansion is phased and opt-in; the
implemented baseline below is unchanged and remains the default.

## Implemented baseline (Phase 4.5)

This is what the crate ships today. The status here must match the code
(`CLAUDE.md` rule 26 / `ADR-0009`).

- `MailSender` trait and `AgMail` enum (`Native | Adapter`).
- Native `SmtpSender` (`lettre` + `rustls`), default sender.
- Provider adapters behind Cargo features: `ResendSender` (`resend`),
  `SesSender` (`ses`), `PostmarkSender` (`postmark`).
- `Email` / `EmailBuilder` message model; typed templates validated at build
  time (`template::validate`).
- Async retry queue with exponential backoff: in-memory by default, optional
  persistent backend via `ag-data` (`queue-persistent`, with the
  `migrations/0001_mail_queue.sql` schema).
- `ag-observe` metrics: `ag_mail_sent_total`, `ag_mail_failed_total`,
  retry/latency series.
- `NullSender` (`test-utils`) for downstream tests.
- Consumed by `ag-auth` (`mail` feature): `AuthMailer` with
  `send_verification` / `send_password_reset` / `send_magic_link`. `ag-mail`
  does NOT depend on `ag-auth`.
- CLI: `ag mail test`.

Outstanding debt for this baseline is tracked in `docs/DEBT.md` (persistent
queue worker, custom SMTP headers, external template engines).

## Planned: native outbound MTA (ADR-0010 / RFC-0009)

Phased, opt-in behind Cargo features, preserving the Native | Adapter pattern
and full backward compatibility. The expansion is **additive only**: it adds a
new opt-in `MtaSender` and new feature-gated subsystems without removing,
demoting, or changing the behavior of anything in the baseline above. The
default features, the default `SmtpSender`, the ESP adapters, and the existing
queues all stay as they are; the native MTA is never silently made the
default. Nothing below is implemented yet; this section is a forward plan, not
a capability claim.

- `mta` feature: MX resolution (`hickory-resolver`) with `site_name` rollup,
  ESMTP+STARTTLS sessions via `mail-send`, outbound DKIM signing via
  `mail-auth` (signed last, after any link rewriting), bounce classification
  from SMTP codes and DSNs (`mail-parser`).
- Two-tier queue (KumoMTA-inspired): scheduled queue keyed
  `tenant:campaign:domain`, ready queue keyed `egress_source + site_name`.
  Native in-memory/`ag-data` default; optional durable backend over
  NATS/JetStream (`queue-jetstream`) mirrored to PostgreSQL.
- `api` feature: multi-tenant REST surface (Axum) drop-in conceptual vs
  Resend (`/emails`, `/emails/batch`, `/domains`, `/api-keys`, `/webhooks`,
  marketing objects), BLAKE3 API keys via `ag-auth`, idempotency, batch,
  scheduling; Svix-compatible HMAC-SHA256 signed webhooks.
- Per-tenant egress IP pools with weighted round-robin warm-up; suppression
  lists (hard bounce / complaint / unsubscribe).
- Deliverability hardening (continuous, Phase 5+): MTA-STS, TLS-RPT, ARC,
  BIMI, Postmaster/FBL intake.

Every external integration keeps a native default and sits behind a feature
(`ADR-0009` rules 2 and 5). The ESP adapters remain the recommended
production path until native deliverability is proven (`RFC-0009` risk table).

## Scope boundaries

### In scope

- Outbound transactional mail (baseline) and native outbound MTA delivery
  (planned).
- Native SMTP sender, ESP adapters, native MTA engine — all under one
  `MailSender` abstraction.
- Inbound limited to DSN (RFC 3464) and FBL/ARF parsing for bounce and
  complaint processing only.

### Out of scope (still, after ADR-0010)

- Full mail server / persistent mailboxes / IMAP / POP / JMAP.
- General inbound mail reception beyond DSN/ARF.
- Antispam content filtering as a service.

For mailbox hosting or a complete mail server, use Postfix, Stalwart, or
another specialized project.

## Dependency rules (verified in CI)

- May depend on `ag-core`, `ag-data` (optional persistent queue / MTA state),
  `ag-realtime` (optional durable queue + event bus), `ag-observe` (metrics),
  `ag-storage` (optional attachments), and `ag-domains` (SPF/DKIM/DMARC
  cooperation and DKIM key material).
- **Must NOT depend on `ag-auth`.** `ag-auth` consumes `ag-mail` through a
  small trait it defines. Sixth rule of architecture chapter 5 (`ADR-0007`),
  verified in CI.
- No cycle with `ag-domains`: `ag-mail` consumes the active DKIM key and the
  DNS materialization, never the reverse.

## DSL v0.7 — `mail` block

```ag
mail WelcomeEmail {
    from "hello@plenty.market"     # must reference a declared domain block
    subject "Welcome to Plenty"
    template "emails/welcome.html" # must exist
    vars {
        name String
        activation_url String       # must be used in the HTML
    }
}
```

Build-time validations: the `from` references a declared `domain`; the
template file exists; the HTML variables match the declared `vars`; a
plaintext fallback or explicit policy exists. A malformed email becomes a
compile error instead of a runtime bug — the real differentiator versus
Resend. The MTA expansion does not change the DSL; traffic shaping and egress
pools are runtime/TOML config (a DSL change would require its own RFC).

## Cooperation with `ag-domains`

`ag-mail` declares DNS requirements via `MailSender::dns_requirements` and
`ag-domains` materializes them as SPF/DKIM/DMARC records (and, for the native
MTA, MTA-STS/TLS-RPT). `ag-domains` owns the DKIM key lifecycle; `ag-mail`
consumes the active private key to sign. The cooperation is optional: a
project using a managed adapter without self-administered DNS does not need
`ag-domains`. See `docs/modules/ag-domains/README.md`.

## Exit criteria

Baseline (Phase 4.5 gate) — met:

- [x] Sends transactional HTML and plaintext via native sender and at least
      one adapter.
- [x] `ag-auth` uses `ag-mail` for verification and recovery in the
      `auth-mail-demo` example.
- [x] Unit + integration coverage >= 75%.
- [x] Zero dependency cycles (CI green).
- [x] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit`,
      `cargo deny check` green.

Native MTA gates are defined per phase in `RFC-0009` section 5 (4.6-A through
4.6-D, plus continuous Phase 5+ hardening) and are not yet started.

## Watched risks

- **Native deliverability on new IPs.** Mitigation: ESP adapters stay the
  recommended production default; phased IP warm-up; MTA opt-in.
- **Confusion with a complete MTA / scope creep.** Mitigation: inbound limited
  to DSN/ARF; mailboxes and IMAP/POP stay out per `ADR-0010`.
- **Stalwart `mail-*` upstream churn.** Mitigation: pinned versions, contract
  tests, active upstream watch.
