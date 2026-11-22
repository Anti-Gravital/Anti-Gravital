# RFC-0037 - Diagnostic error contract

- Status: accepted
- Draft date: 2026-11-22
- Target phase: additive, no phase change
- Affected modules/crates: `ag-core` (contract), `ag-observe` (telemetry), all crates with public error enums
- Predecessor: ADR-0014 (non-exhaustive public error enums)
- Comment period: open

## 1. Motivation

An error is only useful if it answers four questions: when, how, why and where it happened, and what to do next. Today the workspace has a uniform `#[non_exhaustive]` policy (ADR-0014) but no uniform diagnostic contract.

## 2. Problem (verified on 2026-11-22)

1. 29 public error enums exist across `crates/`; all carry `#[non_exhaustive]` per ADR-0014 (for example `crates/ag-core/src/error.rs:18`, `crates/ag-mail/src/error.rs:10`, `crates/ag-domains/src/error.rs:10`).
2. Only two expose a stable code: `AgError::code()` with 14 snake_case codes and HTTP mapping (`crates/ag-core/src/error.rs:74-109`, serialized at `:118-137`) and the CLI `ErrorBody.code` (`crates/ag-cli/src/output.rs:55-60`). The other roughly 27 enums have Display messages but no stable code.
3. `ag-observe` records `ag_requests_total{method,endpoint,status}` and a duration histogram (`crates/ag-observe/src/metrics.rs:31-46`) but no error count by code.
4. Error logs carry fields such as `code`, `error`, `queue` and `job_id` in scattered sites (`crates/ag-workers/src/supervisor.rs:188-246`), but the central request log emits only `method`, `path`, `status` and `latency_ms` (`crates/ag-core/src/shield/logging.rs:87-99`): no `trace_id`/`span_id`.
5. `WorkerErrorKind` already models retriable/permanent/cancelled/timeout with stable `as_str()` labels (`crates/ag-workers/src/error.rs:13-53`). This is the precedent to generalize, not replace.
6. The manual code list is stale: 9 codes documented (`docs/manual/01-shield-as-library.md:302-305`) versus 14 returned by `code()` (`error.rs:79-91`). `docs/architecture/14-observabilidad-ag-observe.md:16,20` claims error rate per HTTP code and correlation IDs on every log that the code does not deliver.

## 3. Alternatives considered

- **A. Keep per-crate Display strings and fix only the docs.** Rejected: diagnostic quality stays inconsistent and code cannot be grouped without parsing human text.
- **B. A shared error trait return type replacing all enums.** Rejected: a public API break across 29 enums with no incremental path; conflicts with ADR-0014 stability goals.
- **C. Uniform stable code plus optional shared trait, added additively, with telemetry and a generated catalog (chosen).** Errors keep their concrete types and `#[non_exhaustive]`; a code is added where missing, with a bounded label set.

## 4. Proposed design

### 4.1 Stable code

Each public error enum exposes a `code(&self) -> &'static str` (shared trait or uniform per-crate method; the choice is recorded in an ADR). Codes are `snake_case`, stable and documented. Existing `AgError::code()` values are preserved. New variants require a new code, never an overload of an existing one.

### 4.2 Correlation

Every error event emitted through `ag-observe` carries `code`, `error`, `trace_id` and `span_id` when an OTLP/tracing context is active. The central request log in `ag-core` is extended with the same fields. 5xx detail redaction at `crates/ag-core/src/error.rs:121-131` is preserved: no payload, secret or internal trace reaches a user-facing response.

### 4.3 Metric

`ag_errors_total{code,scope,status}` with a documented, bounded label set. Cardinality rules from `crates/ag-observe/src/metrics.rs:24-30` apply: no unbounded values such as free text or identifiers.

### 4.4 Catalog

A generated reference document (code, crate, status, retriable, cause, remediation) produced from the code declarations, with a CI freshness check. Per CLAUDE.md rule 29 this is a system-as-is reference, not a debt file. The generated doc lives under `docs/` and is updated in the same PR as any code change (rule 26).

### 4.5 Placement

No new crate (rules 5, 14, 28): `ag-core` owns the contract, `ag-observe` owns telemetry, each crate owns its own codes.

## 5. KPI and verifiable acceptance criteria

- K1: 29/29 public error enums expose a stable code. Verified by a workspace test or lint that enumerates public error variants.
- K2: every error log event carries `code`, `error`, `trace_id`/`span_id` when a context exists. Verified by a test asserting field presence.
- K3: `ag_errors_total{code,scope,status}` is rendered by `/metrics`, with a documented bounded cardinality. Verified by a test asserting presence and label bounds.
- K4: generated catalog exists and CI fails on drift. Verified by running the generator and checking a clean diff.
- K5: manual section 1.10 code list reconciled with `code()`; architecture ch.14 corrected to match implementation or implementation added.
- K6: `cargo fmt`, `cargo clippy`, `cargo test --workspace`, `cargo audit`, `cargo deny` green.

## 6. Risks

- Public API change: adding a required method/trait affects SemVer and downstream implementors (rules 22, 31; ADR-0014:22-24). Requires human approval.
- Security: deciding which context fields may be logged must not leak payloads or secrets (rule 16); preserve existing redaction.
- Cardinality: unbounded metric labels can degrade the Prometheus recorder.
- A shared trait versus per-crate methods is an architectural decision needing an ADR.

## 7. Impact

- Scope: cross-cutting error surface, telemetry and docs.
- Public APIs: additive method on error enums; SemVer decision pending.
- Documentation: manual 1.10, architecture ch.14, crate READMEs, CHANGELOG.

## 8. Rollback

Codes and telemetry are additive. If a change proves breaking, keep the concrete error enums and gate the new fields behind an `ag-observe` feature; the catalog generator can be reverted independently. No data migration.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-22.
- Result: accepted. Comment period waived by BDFL decision.
- The SemVer decision for the shared-trait-versus-per-crate-code choice remains an ADR to be written before implementation.
