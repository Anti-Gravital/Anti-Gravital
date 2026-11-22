# RFC-0026 - Chrono replacement: assessment and blocked migration plan

- Status: proposed (blocked, not accepted; no implementation authorized)
- Author: Gravital Labs engineering
- Draft date: 2026-11-21
- Target phase: maintenance / dependency hygiene (post Phase 4.6)
- Affected crates: ag-auth, ag-workers, ag-domains, ag-dsl, ag-cli,
  examples/workers-*, workspace manifests
- Predecessor RFC: none (tracked by issue #185)
- Comment period: minimum seven calendar days
- Language: English (ADR-0008)

## 1. Motivation

`chrono` 0.4.45 is flagged by RUSTSEC-2026-0543 as unmaintained; no safe
upgrade exists. The workspace currently carries a dated, justified ignore in
`deny.toml` so the advisory gate stays green. Issue #185 asks for the ignore
to be removed once chrono is gone.

This RFC records the migration assessment required by #185 step 1, the exact
call sites, the chosen replacement, and the concrete blocker that prevents
closing the issue in this iteration. It does **not** authorize code changes.

## 2. Problem

Evidence collected on the current tree:

- `cargo tree -i chrono` reaches chrono through four independent paths:
  `ag-domains`, `ag-workers`, `sqlx-core 0.9.1` (sqlx `chrono` feature), and
  `oauth2 5.0.0` (via `ag-auth`). Examples and dev-dependencies add further
  edges (`workers-scheduled`, integration tests).
- With the current `deny.toml`, `cargo deny check advisories` passes. With a
  config that removes only the RUSTSEC-2026-0543 ignore, it fails with
  `error[unmaintained]: chrono is unmaintained`, listing the same four paths.
- The advisory itself recommends `time` or `jiff` as alternatives.

Two findings contradict the issue text and matter for planning:

1. **JWT claims do not use chrono.** `crates/ag-auth/src/jwt.rs` stores `exp`
   and `iat` as `u64` filled from
   `SystemTime::now().duration_since(UNIX_EPOCH)`. Only the stale example in
   `crates/ag-auth/README.md:23` still shows a chrono expression. No JWT code
   migration is required.
2. **chrono cannot be removed by migrating our own crates.** `oauth2 5.0.0`
   declares a non-optional, non-feature-gated dependency on
   `chrono = "0.4.31"` with features `clock`, `serde`, `std`, `wasmbind`
   (`[dependencies.chrono]` in the published crate manifest). crates.io
   reports 5.0.0 as the newest stable oauth2 release, so no lockfile update
   removes it. Consequently `cargo tree -i chrono` cannot become empty and the
   deny ignore cannot be removed while `ag-auth` depends on oauth2 5.

A third constraint: the timestamp types appear in public signatures
(`seconds_until_renewal`, `RenewalWindow`, `RetryDecision`, `DeadLetter`,
`JobEnvelope` fields, generated DSL models). Replacing them changes public
APIs and generated SDK types; a compatibility shim would itself keep chrono
in the tree. This is a SemVer/API decision, not a mechanical swap.

## 3. Alternatives considered

### 3.1 `time` 0.3.x

- Advantages: sqlx 0.9 has a native `time` feature for `OffsetDateTime` <->
  `TIMESTAMPTZ`; the API maps closely to `DateTime<Utc>` (`now_utc`,
  `from_unix_timestamp`, subtraction to `Duration`, `whole_seconds`,
  `Rfc3339` parse/format); `time 0.3.55` is already a transitive dependency
  (x509-parser/asn1-rs, rcgen, testcontainers), so no new crate enters the
  graph; maintained and explicitly recommended by the advisory.
- Disadvantages: serde fractional-second rendering and `to_rfc3339()` output
  (`Z` vs `+00:00`) differ from chrono in edge cases, needing golden tests;
  public signatures change.
- Assessment: **preferred replacement**, contingent on 3.4.

### 3.2 `jiff` 0.2.x

- Advantages: maintained, advisory-recommended, ergonomic civil-time API.
- Disadvantages: sqlx 0.9 has no `jiff` feature, so every SQL boundary needs a
  custom adapter or a manual `Timestamp` conversion; the split
  `Timestamp`/`Zoned`/civil model is a larger rewrite than
  `DateTime<Utc>` -> `OffsetDateTime`; `jiff 0.2.42` is in `Cargo.lock` but
  `cargo tree -i jiff` prints "nothing to print", so it would add a second
  date/time library.
- Assessment: rejected for this codebase.

### 3.3 Do nothing (keep the dated ignore)

- Advantages: zero risk.
- Disadvantages: leaves an unmaintained crate in shipped binaries forever;
  the ignore has no expiry; a future advisory escalation becomes urgent.
- Assessment: not acceptable as a final state; acceptable only while 3.4 is
  unresolved (current state).

### 3.4 Resolve the oauth2 dependency (prerequisite, separate RFC)

- Option A: replace oauth2 with a maintained client that does not depend on
  chrono. Correct long-term fix; requires a dependency/stack RFC and code
  changes in `ag-auth`, coordination with issue #177 (reqwest/TLS) and
  review by seguridad.
- Option B: patch/vendor oauth2 to drop chrono. Removes the advisory edge but
  adds a local fork and supply-chain maintenance burden; needs an RFC.
- Option C: remove oauth2 usage from `ag-auth` and implement the two flows
  directly. Largest change; only justified if A is unavailable.
- Assessment: **blocking prerequisite**. None of these options belongs in
  #185's stated plan.

Follow-up (2026-11-22, native OAuth implementation): the prerequisite above
was resolved by RFC-0029 option 3.5. `ag-auth` now uses a native Authorization
Code + PKCE client and no longer depends on `oauth2`; the `oauth2 5.0.0 ->
ag-auth -> chrono` edge is gone from `cargo tree -i chrono`. The remaining
chrono paths are those already listed in section 2: `ag-domains`, `ag-workers`,
sqlx (via its `chrono` feature) and `workers-scheduled`. This does not change
the selection of `time` 0.3.x in 3.1, nor the requirement to keep the dated
`RUSTSEC-2026-0543` ignore until every remaining edge is gone.

## 4. Proposed design (after the blocker is resolved)

### 4.1 Dependency changes

- Add `time = { version = "0.3", features = ["serde", "macros", "formatting",
  "parsing"] }` to `[workspace.dependencies]`.
- Replace `chrono` with `time` in the sqlx feature list in the workspace
  manifest: `"uuid", "json", "chrono"` -> `"uuid", "json", "time"`.
- Replace `chrono = { workspace = true }` with `time = { workspace = true }`
  in `crates/ag-workers/Cargo.toml` (dependencies and dev-dependencies) and
  in `examples/workers-scheduled/Cargo.toml`.
- In `crates/ag-domains/Cargo.toml`, swap `dep:chrono` for `dep:time` in the
  `acme`, `route53` and `google-cloud-dns` features and in the optional
  dependency table.
- The workspace `chrono` entry may be dropped once oauth2 no longer forces
  chrono into the graph. Until then, oauth2 supplies its own independent copy.

### 4.2 Type and call-site mapping

| chrono | time | Notes |
| --- | --- | --- |
| `DateTime<Utc>` | `OffsetDateTime` | UTC-normalized at construction boundaries |
| `Utc::now()` | `OffsetDateTime::now_utc()` | |
| `DateTime::from_timestamp(s, 0)` | `OffsetDateTime::from_unix_timestamp(s)` | |
| `signed_duration_since(x).num_seconds()` | `(a - b).whole_seconds()` | Clamp to 0 as today |
| `Duration::from_std(d).unwrap_or_else(...)` | `TimeDelta::try_from(d).unwrap_or(...)` | Preserve current fallbacks |
| `DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc)` | `OffsetDateTime::parse(s, &Rfc3339)` | Reject non-UTC offsets explicitly if required |
| `Datelike::year()` | `OffsetDateTime::year()` | |
| `Utc.with_ymd_and_hms(...)` | `macros::datetime!(...).assume_utc()` | Route 53 SigV4 test fixture |
| `.timestamp()` | `.unix_timestamp()` | GCP service-account JWT `iat`/`exp` |

### 4.3 Interfaces and formats

- SQL: `sqlx` `time` feature encodes `OffsetDateTime` to `TIMESTAMPTZ` using
  the same binary representation; no migration of stored rows is expected.
  This must be verified by a round-trip test, not assumed.
- Wire (ARI RFC 9773): continue parsing RFC 3339 strings; `Rfc3339` is the
  equivalent formatter. Current tests assert chrono's `+00:00` spelling;
  those assertions must be updated with an explicit decision on whether the
  public textual form is part of the contract.
- AWS SigV4: Route 53 `X-Amz-Date` must stay `%Y%m%dT%H%M%SZ` (UTC). Use
  `time::format_description::parse` once, not per request.
- JWT: no change (see section 2).
- DSL codegen: `FieldType::Timestamp` currently emits
  `chrono::DateTime<chrono::Utc>`. Switching to `time` changes generated
  public SDK types and requires regenerating fixtures and updating the DSL
  reference. Keeping it emits chrono into generated user projects even after
  the workspace is clean. This needs its own explicit decision in the oauth2
  RFC or a follow-up RFC.

### 4.4 Documentation and tooling

- Update `docs/modules/ag-workers/README.md`, `docs/dsl/referencia-v01-v04.md`,
  `docs/rfc/RFC-0012-ag-workers.md` (dependency list), `CHANGELOG.md` and the
  stale `crates/ag-auth/README.md` example in the same PR that changes code.
- `deny.toml`: remove the RUSTSEC-2026-0543 ignore only after
  `cargo tree -i chrono` is empty. Not in this RFC's implementation scope.

## 5. Exact call sites

### 5.1 Manifests

- `Cargo.toml:68` (sqlx features), `Cargo.toml:119` (workspace chrono).
- `crates/ag-workers/Cargo.toml:46,61`.
- `crates/ag-domains/Cargo.toml:23,32,34,87`.
- `examples/workers-scheduled/Cargo.toml:20`.
- `benchmarks/neon-real/Cargo.toml:25` (not a workspace member; separate).

### 5.2 ag-workers (about 78 matching lines)

- `src/job.rs`: import (line 11), `JobEnvelope` timestamp fields
  (`scheduled_at`, `available_at`, `created_at`, `updated_at`, `lock_until`,
  lines 185-195), `into_envelope` / delay arithmetic (line 350).
- `src/queue/mod.rs:15,70` (`RetryDecision.available_at`).
- `src/queue/memory.rs:13,115,262-263`.
- `src/queue/dlq.rs:8,79` (`DeadLetter.dead_lettered_at`).
- `src/queue/postgres.rs:11,84,346,528-560` (bindings and `try_get`
  `DateTime<Utc>` columns).
- `src/scheduler.rs:13,61,79-81,310,326,336,347`.
- `src/backoff.rs:10,68,76,132`; `src/runtime.rs:12,194`;
  `src/registry.rs:232,249,269`.
- `tests/scheduler_dynamic.rs:26`; `examples/workers-*`, `crates/ag-cli`
  (transitive callers to be updated after the type change).

### 5.3 ag-domains (about 41 matching lines)

- `src/acme/ari.rs:16,40,55-59,82-94,104` (RFC 3339 parse, window bounds,
  duration arithmetic, tests).
- `src/acme/renewal.rs:16,215-224,231-239,431` (notAfter parsing and
  threshold arithmetic).
- `src/provider/route53.rs:24,94,391-403,465,478` (SigV4 amz_date/date_stamp
  and fixture).
- `src/provider/google_cloud_dns.rs:20,103` (JWT `iat`/`exp`).
- `src/attachment.rs:144` (documentation only).

### 5.4 DSL and documentation

- `crates/ag-dsl/src/codegen/rust_gen.rs:45,58-59`, `src/ast.rs:177`,
  `src/lexer.rs:49`.
- `examples/ecommerce-api/generated/src/models.rs:8,17-18,68,101-102`
  (not a workspace member).
- `docs/dsl/referencia-v01-v04.md:79,215-216`,
  `docs/modules/ag-workers/README.md:56`, `docs/rfc/RFC-0012-ag-workers.md:250`,
  `CHANGELOG.md:70`, `crates/ag-auth/README.md:23`.

## 6. Implementation order (once section 3.4 is accepted)

1. Land the oauth2 decision (blocking).
2. Workspace dependency swap and sqlx feature swap; add golden serde/RFC 3339
   tests before changing production types.
3. ag-workers: records, queue backends, scheduler, runtime, backoff, tests,
   examples. Verify TIMESTAMPTZ round trips with a `#[ignore]` PostgreSQL test.
4. ag-domains: ARI, renewal, Route 53 SigV4, Google Cloud DNS JWT.
5. JWT: no code change; fix the stale README example only.
6. DSL codegen decision and regeneration of examples/fixtures.
7. Remove the RUSTSEC-2026-0543 ignore from `deny.toml`; update CHANGELOG,
   README and module docs in the same commit.

Each step is a separate PR with `cargo fmt`, `cargo clippy`,
`cargo test --workspace --all-features`, `cargo audit` and
`cargo deny --all-features check` in green.

## 7. Risks

- **Blocked prerequisite (probability certain, impact high):** oauth2 5 keeps
  chrono in the graph. Mitigation: separate RFC before any code change; keep
  the dated ignore with its removal condition unchanged in the meantime.
- **Serialization drift (medium, medium):** fractional-second rendering and
  RFC 3339 `Z` vs `+00:00`. Mitigation: golden tests asserting exact strings
  for representative values before and after each crate migration.
- **Public API break (high, medium):** timestamp types in public signatures
  and generated models. Mitigation: treat as an explicit API decision,
  document it, and avoid partial migrations that leave two type systems
  mixed.
- **SQL type regression (low, high):** wrong sqlx feature or encoding.
  Mitigation: round-trip test against PostgreSQL with real TIMESTAMPTZ rows.
- **Scope creep (medium, medium):** DSL codegen and non-workspace benchmark
  paths. Mitigation: keep them as separate, explicitly accepted steps.

## 8. Impact

- Scope: no product feature changes; dependency hygiene only.
- Roadmap: no phase change; this is maintenance work.
- Operations: one fewer unmaintained crate once the blocker is resolved; no
  configuration or deployment changes.
- Public APIs: changed if timestamp types are replaced; must be announced.
- Documentation: several module, DSL and changelog updates are required.

## 9. Rollback

Revert the dependency swap and restore the archived chrono-based types from
the preceding commit. Because TIMESTAMPTZ storage is unchanged in principle,
no data migration is expected; if golden round-trip tests show any storage
difference, stop before merging and re-evaluate. Rollback signal: any failing
serde/RFC 3339 golden test or PostgreSQL round-trip test.

## 10. Decision

Deferred. The replacement analysis selects `time` 0.3.x, but implementation is
blocked on the oauth2 dependency decision (section 3.4). No code was changed
for this RFC. Issue #185 remains open; `Refs #185` is used, not `Closes #185`.

- Decisor: BDFL or technical committee.
- Decision date: pending.
- Result: deferred pending the oauth2 RFC.

## 11. References

- Issue #185: https://github.com/Anti-Gravital/Anti-Gravital/issues/185
- RUSTSEC-2026-0543: https://rustsec.org/advisories/RUSTSEC-2026-0543
- Issue #177 (reqwest / TLS dependency surface)
- RFC-0012 (ag-workers), ADR-0008 (language policy)
