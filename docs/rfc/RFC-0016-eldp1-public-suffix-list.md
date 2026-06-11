# RFC-0016 - eTLD+1 via the Public Suffix List (ag-domains)

- Status: accepted
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-06-11
- Decision date: 2026-06-11
- Target phase: maintenance of Phase 4.5 ag-domains (correctness fix)
- Affected modules/crates: `ag-domains` (`hostname`, `issuance`, `registrable`);
  a new optional dependency (`psl`)
- Predecessor RFC: RFC-0011 (ag-domains control plane)
- Comment period: waived by BDFL decision. The dependency is pure-Rust, embedded
  and feature-gated OFF by default; the native build is unaffected.

## 1. Motivation

`ag-domains` derives the registrable domain (eTLD+1) with a two-label heuristic
in two places — `hostname::registrable_domain` and `issuance::counting_key`.
Multi-label public suffixes (`co.uk`, `com.br`, `gov.au`, ...) are misclassified:
`example.co.uk` is treated as registrable `co.uk`. This skews apex/subdomain
classification, the generated DNS instructions, and — most importantly — the
per-registered-domain issuance rate-limit key, which protects against hitting the
CA's certificates-per-registered-domain cap. This is issue #78 (DEBT-024).

This RFC is required because the fix introduces a new dependency, which CLAUDE.md
rule 15/28 gates behind an approved RFC.

## 2. Problem

- The two-label heuristic is correct only for single-label TLDs (`example.com`).
- For multi-label suffixes it both over-counts (collapsing distinct registrable
  domains under `co.uk`) and misclassifies apex vs subdomain.
- The Public Suffix List (PSL) is the authoritative source for this and is not
  expressible as a simple rule; it must be embedded or loaded.

## 3. Alternatives considered

1. **Do nothing.** Keep the heuristic. Pro: no dependency. Con: incorrect for a
   large, common class of domains; the issuance counter is security-relevant.
   Rejected.
2. **Hand-maintain a suffix table.** Rejected: the PSL changes frequently;
   hand-maintenance is error-prone and is exactly what the PSL exists to avoid.
3. **Adopt a PSL-backed crate, feature-gated (this RFC).** Candidates: `psl`
   (compile-time embedded list, no runtime fetch, `no_std`-friendly) or
   `publicsuffix` (loads a list, supports updates). Pro: correct, well-maintained.
   Con: a new dependency and a larger binary (embedded list). Chosen, with `psl`
   preferred for offline/native-first operation (ADR-0009).

## 4. Proposed design

- Add an optional Cargo feature `psl` (default OFF to keep the native build
  minimal and offline) that pulls the chosen PSL crate.
- Introduce a single internal function `registrable_domain(host) -> String` that
  is **the only** eTLD+1 source, shared by `hostname` and `issuance::counting_key`
  (DRY). With the `psl` feature it uses the PSL; without it, it falls back to the
  current two-label heuristic, so the default build keeps working offline exactly
  as today.
- No public API shape change: the function's signature and the existing fields
  (`registered_domain`, counting key) stay the same; only the derivation improves.
- Dependency justification (rule 15): the PSL crate is mature, widely used,
  permissively licensed, and pure-Rust; it adds no external service.

## 5. Implementation plan (only after approval)

1. PR 1: add the `psl` feature and the shared `registrable_domain` function with
   the heuristic fallback; route `hostname` and `issuance` through it; add tests
   for `co.uk`/`com.br`-style suffixes under both feature states.
2. PR 2 (optional): wire `cargo deny`/`audit` entries and document the feature in
   the capability/installation notes.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Binary-size increase from the embedded list | Medium | Low | Feature-gated, OFF by default |
| PSL crate staleness | Low | Medium | Pin + `cargo update` cadence; both candidate crates are actively maintained |
| Behaviour divergence feature on/off | Medium | Low | Single shared function; tests assert both paths; document that the heuristic is a best-effort offline fallback |

## 7. Impact

- Correctness: fixes apex/subdomain classification, generated instructions and
  the issuance counter for multi-label suffixes.
- Public APIs: unchanged.
- Operational complexity: none by default (feature OFF); opt-in correctness.
- Documentation: a note in the ag-domains reference and installation features.

## 8. Rollback

Disable the `psl` feature: the crate reverts to the heuristic fallback with no
API change. No data migration is involved.

## 9. Decision

- Decider: Angel Nereira (BDFL).
- Decision date: 2026-06-11.
- Outcome: accepted (comment period waived by BDFL).
- Rationale: the fix is a correctness improvement that shares a single eTLD+1
  source between `hostname` and `issuance`; the dependency is pure-Rust,
  embedded, permissively licensed and gated OFF by default, so the native build
  stays offline (ADR-0009) and rule 15 is satisfied.

## 10. References

- Issue #78 (DEBT-024), issue #76. RFC-0011, ADR-0009. CLAUDE.md rules 15, 21, 28.
- `crates/ag-domains/src/hostname.rs`, `crates/ag-domains/src/issuance.rs`.
