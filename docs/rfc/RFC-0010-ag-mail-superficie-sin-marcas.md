# RFC-0010: ag-mail - brand-neutral sender surface

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-04
- Target phase: Phase 4.6 (corrective, alongside the native MTA work)
- Affected crates: `ag-mail` (remove brand adapters), `ag-dsl` (provider
  values), `ag-cli` and `ag-auth` (comments/features only)
- Predecessor RFC: RFC-0006 (superseded for the brand adapters)
- Governing ADR: `docs/adr/0011-politica-marcas-comerciales.md`
- Comment period: waived by the BDFL; formalizes the mechanics of an
  already-approved decision (ADR-0011), as `CLAUDE.md` rule 28 requires an RFC
  before changing module boundaries.

## 1. Motivation

`ADR-0011` decided to remove the third-party commercial brand names from
`ag-mail`'s surface. This RFC specifies the concrete mechanics: what is removed,
what replaces it, the DSL value change, and the migration for consumers, so the
change is reviewable and reversible.

## 2. Problem

`ag-mail` shipped provider-brand adapters (`ResendSender`/`ResendConfig`
implemented; `ses.rs`/`postmark.rs` empty skeletons) behind Cargo features
`resend`/`ses`/`postmark`, plus the DSL `provider` accepting those brand
values. The native `SmtpSender` relay already reaches any external provider via
its SMTP endpoint, and the native `MtaSender` delivers without third parties, so
the brand adapters add surface and a trademark hazard without unique value.

## 3. Alternatives considered

See `ADR-0011` (keep adapters; rename to neutral; add one generic HTTP relay;
blanket no-brand rule). The chosen option is removal, with the native SMTP relay
as the unbranded path to external providers.

## 4. Proposed design

### 4.1 Removed from `ag-mail`
- Files `src/sender/{resend.rs,ses.rs,postmark.rs}`.
- Cargo features `resend`, `ses`, `postmark`; the now-unused optional
  dependency `reqwest`; the `wiremock` dev-dependency (only used by the removed
  adapter tests).
- Brand mentions in `src/lib.rs`, `src/sender/mod.rs`, `src/metrics.rs`, and the
  package `description`.

### 4.2 Replacement / migration
- To send through an external email provider: construct `SmtpSender` with that
  provider's SMTP host, port and credentials (`SmtpConfig`). No code in
  `ag-mail` names the provider.
- The no-third-party path: the native `MtaSender` (feature `mta`).

### 4.3 DSL change (`ag-dsl`)
- `mail` block `provider` accepted values change from
  `["smtp","resend","ses","postmark"]` to `["smtp","mta"]`
  (`semantic.rs` `VALID_PROVIDERS`). The grammar is unchanged; only the accepted
  value set changes, so no new DSL version is required. Schemas using a removed
  value get a semantic error pointing to the valid set.

### 4.4 Consumers
- `ag-auth` uses only the `MailSender` trait (`Arc<dyn MailSender>`); no code
  change, only a comment.
- `ag-cli` `ag mail test` already uses `SmtpSender`; its dependency drops the
  `resend` feature.

### 4.5 Governance / CI
- `CLAUDE.md` gains the brand-name policy section.
- `.github/workflows/docs.yml` `prohibited-content` job gains a scan for the
  removed mail brands (`Resend`, `Postmark`, `SES`, `Svix`, and the
  `*Sender` type names), excluding the files that define the policy.

## 5. Implementation plan

Single corrective PR on the active branch: remove adapters and wiring; update
the DSL value set and tests; scrub `ag-mail`-related documentation of brand
names (keeping destination providers and the retained `Cloudflare`/`S3`
adapters); add ADR-0011, this RFC, the `CLAUDE.md` policy and the CI scan;
recompute master hashes. Each push must keep `cargo fmt`, `clippy -D warnings`,
`cargo test`, and the content scans green.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| A consumer relied on a removed adapter type | Low | Low | No releases; `ag-auth` is trait-based; migration is `SmtpSender`. |
| A schema used `provider resend` | Low | Low | Clear semantic error with the valid set; documented migration. |
| CI brand scan false positives | Medium | Low | Scope the scan to the removed mail brands; exclude policy-defining files and destination/retained-adapter names. |

## 7. Impact

- **Product scope:** unchanged capability (external providers via SMTP; native
  MTA unchanged); only the brand surface is removed.
- **Roadmap:** corrective within Phase 4.6; no schedule change.
- **Public APIs:** removes `ResendSender`/`ResendConfig` and the `resend`/`ses`/
  `postmark` features from `ag-mail`. Acceptable (`publish = false`, no releases).
- **Documentation:** brand scrub across `ag-mail` docs; ADR-0011 + this RFC.

## 8. Rollback

The change is removal. To restore an external-provider HTTP adapter later, add
it under the policy of `ADR-0011` (only if a generic native path does not cover
the need) with a neutral, non-brand type name. Reverting this RFC means
re-adding the removed modules, which is mechanically simple but reintroduces the
trademark hazard, so it is not recommended.

## 9. Decision

- Decider: Angel Nereira (BDFL)
- Decision date: 2026-06-04
- Outcome: accepted
- Justification: implements `ADR-0011` without reducing capability; the native
  SMTP relay and MTA cover all sending needs unbranded.

## 10. References

- `docs/adr/0011-politica-marcas-comerciales.md` - governing decision.
- `docs/rfc/RFC-0006-ag-mail-alcance.md` - original adapters (superseded here).
- `docs/rfc/RFC-0009-ag-mail-native-mta.md`, `docs/adr/0010-...` - native MTA.
- `CLAUDE.md` rules 12, 21, 28, 30, 31.
