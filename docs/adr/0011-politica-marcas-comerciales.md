# ADR-0011 - Third-party commercial brand name policy

- Status: accepted
- Date: 2026-06-04
- Source RFC: RFC-0010
- Author: Angel Nereira (BDFL), Gravital Labs
- Affected crates: `ag-mail` (removes provider-brand adapters); policy applies
  repo-wide. `ag-domains`/`ag-storage` adapters are reviewed and retained.

## Context

Anti-Gravital exposed third-party **commercial** brand names in its own public
surface. The clearest case is `ag-mail`: public types `ResendSender` /
`ResendConfig`, empty skeleton modules `ses.rs` / `postmark.rs`, and Cargo
features `resend` / `ses` / `postmark`, plus brand mentions in comments and
documentation. Presenting a third party's trademark as part of the product
surface is a branding and trademark hazard, and in several places the brand was
used merely as a convenient label where a neutral name would do.

Not every brand mention is equivalent, though. A brand name can also serve a
legitimate, purely descriptive purpose: labelling an **adapter that integrates
that specific third party**, so a contributor immediately knows "this is the
connector you plug that service into". `ag-domains`' `CloudflareProvider` and
`ag-storage`'s `S3Store` are of this kind: there is no generic native
substitute, and the brand is the accurate name of the socket.

The distinction matters for `ag-mail` specifically because a **generic native
path already exists**: the native `SmtpSender` relay (built on the open-source
`lettre`, no brand) can send through any external provider by pointing at that
provider's SMTP endpoint, and the native `MtaSender` delivers without any third
party. So the brand-named mail adapters were unnecessary — a convenience name,
not a required socket.

## Decision

Adopt a third-party commercial brand name policy and apply it:

1. **Rule.** A third-party commercial brand name may appear in the repository
   (code, identifiers, Cargo features, comments, documentation) **only** as the
   explicit label of an adapter that integrates that specific third-party
   service, confined to that adapter's module behind its own Cargo feature, and
   **only when no generic native path covers the same need**. A brand name must
   never name a generic or core component, must never be presented as part of
   Anti-Gravital's own surface, and must never be used out of convenience when a
   neutral name fits.

2. **Exemption.** Names of open-source technologies that Anti-Gravital is built
   on or uses are not commercial brand violations and are kept (for example
   `lettre`, `mail-send`, `mail-auth`, `hickory-resolver`, `rustls`, `tokio`,
   `axum`, `sqlx`, `moka`).

3. **Destinations.** Names of external mailbox/destination providers cited as
   interoperability or deliverability requirements (the providers whose
   SPF/DKIM/DMARC and bulk-sender rules a sender must satisfy) are not our
   components and are kept where technically necessary.

4. **Application to `ag-mail`.** Remove the provider-brand adapters
   (`ResendSender`/`ResendConfig` and the `ses`/`postmark` skeletons) and their
   Cargo features. To use an external provider, point the native `SmtpSender`
   at the provider's SMTP endpoint; the no-third-party path is the native
   `MtaSender`. The DSL `mail` block `provider` accepts only the native modes
   `"smtp"` and `"mta"`. Mechanics in RFC-0010.

5. **Retained adapters.** `ag-domains::CloudflareProvider` (feature
   `cloudflare`) and `ag-storage::S3Store` (feature `s3`) are retained: they are
   legitimate adapter labels for a specific third party with no generic native
   substitute. They must stay confined to their adapter module behind their
   feature and be documented as adapters, never as our own components.

The policy is added to `CLAUDE.md` and enforced by a `prohibited-content` CI
scan for the removed mail brands.

## Consequences

Positive:
- No third-party commercial brand is presented as part of `ag-mail`'s surface.
- The capability to use an external provider is preserved unbranded via the
  native SMTP relay; the no-third-party path is the native MTA.
- A clear, auditable rule prevents regressions and guides future adapters.
- Removing the `ses.rs`/`postmark.rs` skeletons also fixes a documentation
  inaccuracy (they were declared implemented but were empty), satisfying rule 26.

Negative:
- Breaking change to `ag-mail`'s internal API (removed public types/features).
  Acceptable: `ag-mail` is `publish = false` with no releases, and `ag-auth`
  consumes only the `MailSender` trait, not the concrete adapters.
- Schemas that declared `provider resend|ses|postmark` no longer validate; they
  migrate to `provider smtp` (or `mta`).

Neutral:
- `ag-domains`/`ag-storage` keep their brand-labelled adapters; the policy
  documents why this is allowed rather than requiring a rename.

## Alternatives considered

A. **Keep the brand adapters.** Rejected: presents third-party trademarks as
   product surface; the SMTP relay already covers the need unbranded.

B. **Rename the brand adapters to neutral names but keep three.** Rejected:
   neutral names for provider-specific HTTP shapes are arbitrary and still imply
   specific providers; the native SMTP relay is the honest generic path.

C. **Add one generic unbranded HTTP relay adapter.** Rejected for now: the
   native SMTP relay already reaches any provider; an HTTP relay adds surface
   without need.

D. **Apply a blanket "no brand anywhere" rule.** Rejected: it would force
   renaming legitimate adapter sockets (`CloudflareProvider`, `S3Store`) and
   strip unavoidable destination/interop references, reducing clarity.

## Notes

- Mechanics and migration: `docs/rfc/RFC-0010-ag-mail-superficie-sin-marcas.md`.
- Enforcement: `CLAUDE.md` policy section and the `prohibited-content` job in
  `.github/workflows/docs.yml`.
- Related: `ADR-0009` (native default + feature-gated externals), `ADR-0010`
  and `RFC-0009` (native MTA), `RFC-0006`/`ADR-0007` (original adapters, now
  superseded for the brand adapters).
- `CLAUDE.md` rules 12 (interoperability), 21 (anti-complexity), 30 (prefer
  deletion), 31 (public APIs), 39 (no hype).
