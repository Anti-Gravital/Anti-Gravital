# RFC-0025 - DSL-generated typed ORM, row-level security and multi-tenancy for ag-data

- Status: ACCEPTED AS DESIGN; IMPLEMENTATION DEFERRED. Decision recorded in
  ADR-0020 (2026-11-22, BDFL). No code may be written under this RFC until
  Phase 5 opens and the pre-Phase-5 release gate closes.
- Author: Architecture working group, Anti-Gravital community
- Draft date: 2026-11-21
- Target phase: Phase 5 and later (Phase 5+). The first ORM increment may start
  earlier only if the pre-Phase-5 release gate is closed and a separate
  implementation PR records that decision.
- Affected modules/crates: `ag-dsl` (schema annotations and codegen), `ag-data`
  (tenant key binding, transaction-local scope), `ag-core` (`TenantId`,
  `TenantContext`), `ag-cli` (`ag generate --check`), `ag-observe` (admin-scope
  audit events), `ag-workers`, `ag-mail`, `ag-domains` (tenant key alignment as
  a follow-up)
- Predecessor RFC: RFC-0019 (layered generated projects), RFC-0020 (declared
  environment variables and secrets), ADR-0013 (the `AgTx` executor boundary)
- Related work: issue #154, the data-layer design note at
  `docs/modules/ag-data/data-layer-rls-tenancy.md` (review by the data-model
  role, incorporated here), issue #185 / RFC-0026 (date/time decision)
- Comment period: minimum seven calendar days from publication.

## 1. Motivation

Issue #154 (migrated from DEBT-009) records a forward-looking capability, not a
defect: typed queries, row-level security (RLS) and multi-tenancy are generated
from the Anti-DSL on top of the `ag-data` pool, and none of them exist yet.
`ag-data` currently offers `DataConfig`, `DbPool`, `connect()`,
`run_migrations()`, `DataError` and the `AgTx` transaction handle
(`AgTx::as_executor()` per ADR-0013). The DSL generates model structs and SQL,
but no repository implementations and no tenant-aware policy.

A backend toolkit that claims to serve production services needs a safe default
for the most common multi-tenant failure: a handler that forgets one `WHERE`
clause and reads another customer's rows. Hand-written discipline is not an
enforceable boundary. The generated code must make omission a compile error and
the database must provide a second, independent barrier.

This RFC is **design only**. Its target is Phase 5+; it does not change any
crate, public API, phase or release gate.

## 2. Problem

1. No typed ORM. `ag generate` emits model structs and migrations, but every
   query is still hand-written SQL against `sqlx`, with no compile-time link to
   the schema and no drift gate for generated repository code.
2. No tenant key contract. `ag_workers::ids::TenantId` is a `String` newtype on
   every `JobEnvelope`, `ag-mail` carries `tenant: String`, and `ag-domains`
   uses its own project identifier. There is no shared type and no guarantee
   that an asynchronous job still knows which tenant owns the row it touches.
3. No database-enforced isolation. Nothing prevents a pooled connection from
   inheriting a previous request's context, and nothing stops a query with a
   forgotten predicate from returning every tenant's rows.
4. No migration story for enabling RLS on a populated table, including the
   `CREATE INDEX CONCURRENTLY` restriction inside `sqlx::migrate!`.
5. Unresolved type mapping. The DSL currently emits
   `chrono::DateTime<chrono::Utc>` for `Timestamp` (`crates/ag-dsl/src/ast.rs`
   and `codegen/rust_gen.rs`); issue #185 / RFC-0026 may change that mapping.
   A data-layer design that hard-codes the current type would diverge from the
   generated projects as soon as that decision lands.

The data-model review recorded in
`docs/modules/ag-data/data-layer-rls-tenancy.md` verified all five points in
this tree and is the technical input to this RFC.

## 3. Alternatives considered

**A. No change (status quo).** Zero cost now. Rejected: every generated project
reimplements tenant scoping, and a single mistake is a cross-tenant data leak.
It also leaves #154 permanently open.

**B. Hand-written repositories with a lint that checks for a tenant predicate.**
Cheap, but a custom import/SQL analyzer is exactly the weak enforcement that
RFC-0019 section 3 rejected. A linter can be bypassed; the compiler and the
database cannot.

**C. A third-party ORM that owns tenancy.** Rejected: rule 12 (integrate, do
not reinvent) applies to infrastructure, not to the core data contract of the
product. It would add an unplanned stack dependency, a new RFC per version
change and a runtime abstraction over `ag-data`. The sanctioned sqlx boundary
(ADR-0013) must remain single.

**D. RLS only, no generated typed ORM.** Database policies are strong, but
application code still has stringly typed queries, no drift gate and no
compile-time context argument. Rejected as the default.

**E. Generated typed ORM with mandatory context plus generated RLS (chosen).**
The compiler enforces that a context was supplied; the database enforces that
the context matches the rows; both generate from one schema. Cost is generator
complexity and a migration sequence, both bounded and testable.

## 4. Proposed design

### 4.1 Scope and increments

Three increments, each independently reviewable. None is authorised to start by
this document.

| Increment | Deliverable | Gate |
| --- | --- | --- |
| I1 - typed ORM | DSL emits ports and repository adapters with a mandatory `TenantContext`, explicit tenant predicates and `tenant_id` columns | Generated fixture builds; two-tenant tests pass against PostgreSQL |
| I2 - RLS | DSL emits fail-closed policies, the transaction-local GUC call and the role/scoping rules | Two-tenant tests pass with RLS forced; unscoped queries return zero rows |
| I3 - asynchronous propagation and admin scope | Jobs and support tools carry tenant context; audited `TenantScope::All` | Envelope round-trip test; audit event asserted |

RLS is not postponed indefinitely: I1 emits the `tenant_id` column and reserves
the policy name so I2 is additive. If the release schedule forces a split, I1
ships with the explicit predicate as the only barrier and the documentation
states that fact; it is never described as RLS.

### 4.2 Tenant key and context

`ag-core` gains an opaque `TenantId` newtype over `uuid::Uuid`, with `serde` as
a string, `Display` and validation on construction. `ag-core` remains free of
crate dependencies (rule 15). `ag-data` implements `sqlx::Type<Postgres>`,
`Encode` and `Decode` once, transparent over a `UUID` column, so generated
repositories bind the newtype and never a bare string.

`TenantContext { tenant_id: TenantId, trace_id: TraceId }` is a plain value with
no I/O and no pool. It is passed explicitly to every generated repository
method; there is no ambient or thread-local "current tenant". Omitting it is a
compile error, which is the enforcement mechanism RFC-0019 prefers.

Tenant identity enters at the interface layer from a verified JWT claim named
`tenant_id`, or from a header that the edge is configured to trust and to strip
from inbound traffic. It is never read from a request body, query parameter or
configuration. A missing claim rejects the request; there is no default tenant.
RFC-0020 keeps `DATABASE_URL` as `secret = true` in `ag.toml`; the composition
root exposes it once into `DataConfig`, and a tenant id stays audit-loggable
because it is not a secret.

Existing ad-hoc keys (`ag-workers`, `ag-mail`, `ag-domains`) keep their current
representations in I1/I2. Explicit conversion adapters at each boundary map
them to `TenantId`; `ag-workers` already carries the tenant in `JobEnvelope`,
which is the correct propagation shape. Unification of those crates on the
shared type is a follow-up RFC, not silent churn inside this one.

### 4.3 Transaction-local scope (the central correctness constraint)

Before any tenant-owned query, the generated repository opens an `AgTx` and
runs, through the existing `as_executor()`:

```sql
SELECT set_config('app.tenant_id', $1, true);
```

The third argument must be `true`. A session-level `SET` survives on a reused
`sqlx::PgPool` connection and leaks one tenant's context into the next request
that receives that connection. `AgTx::as_executor()` already supplies the
`&mut sqlx::PgConnection` required; no new executor surface and no `unsafe` is
added. A use case needing several repositories in one transaction sets the GUC
once on the shared `AgTx`, and the generated repositories accept `&mut AgTx`
instead of `&DbPool`.

### 4.4 RLS policy model

Each model is classified by a DSL marker; domain models are tenant-owned by
default and global/reference tables must opt out explicitly, so a missing
marker cannot silently create a cross-tenant read.

| Class | `tenant_id` | Policy |
| --- | --- | --- |
| Tenant-owned | required, `NOT NULL` | `USING` and `WITH CHECK` compare it to the context |
| Global / reference | absent or NULL means global | no tenant policy; writes restricted to admin scope |
| System | absent | no policy; migrations and operators only |

Generated migration for every tenant-owned table:

```sql
ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <table> FORCE ROW LEVEL SECURITY;

CREATE POLICY ag_tenant_isolation ON <table>
  FOR ALL
  USING (
    tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::uuid
  )
  WITH CHECK (
    tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::uuid
  );
```

Fail-closed behaviour: `current_setting(..., true)` returns NULL when the GUC
was never set, `NULLIF` also handles an empty string, and comparing `uuid` to
NULL selects no rows. `FORCE ROW LEVEL SECURITY` applies the policy to the table
owner too; the runtime role must not have `BYPASSRLS`, while the migration role
is a separate privileged role. `WITH CHECK` blocks inserting or moving a row to
another tenant. Generated SQL also keeps an explicit `WHERE tenant_id = $1`
and an explicit `tenant_id` insert column: RLS is the second barrier, not the
only one.

`TenantScope::All` is a named escape hatch obtained through a separate
constructor that requires an explicit `AdminAuditHandle`. Every use emits an
`ag-observe` event with actor, table, predicate and reason. Generated
application code never uses it; it exists for migrations, support and
reporting. No ambient admin mode is introduced.

### 4.5 Generated ORM shape and output paths

RFC-0019 paths are mandatory: domain structs in `crates/domain/src/generated/`,
repository ports in `crates/app/src/generated/`, implementations and migrations
in `crates/infra/src/db/generated/` and `crates/infra/migrations/`, routes and
request/response types in `crates/api/src/generated/`. The compact layout uses
the same four module names. `ag generate --check` is the drift gate.

```rust
#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn find_by_id(&self, ctx: &TenantContext, id: Uuid)
        -> Result<Option<Todo>, DataError>;
    async fn list(&self, ctx: &TenantContext, page: Page)
        -> Result<Vec<Todo>, DataError>;
    async fn insert(&self, ctx: &TenantContext, value: &NewTodo)
        -> Result<Todo, DataError>;
}
```

The generated adapter holds only a cheaply cloneable `DbPool` and no tenant
state, so one instance is safe to share across requests. Default query mode is
runtime-checked `sqlx::query_as` with derived `FromRow`: generated projects
build without a live database, satisfying reproducibility (rule 36) and the
`git clone; cargo build; cargo test` promise (rule 37). `query_as!` with
committed `.sqlx` offline metadata is opt-in and requires a CI preparation
step. Generated code never reaches `sqlx::PgPool` outside `ag-data`'s
abstractions.

The `Timestamp` row of the generated type map is explicitly provisional and
depends on the issue #185 / RFC-0026 decision. Whichever type is chosen, the
DSL type mapping, the emitted `use` line, the sqlx features and the golden
serialization tests change in the same PR. If `time::OffsetDateTime` is chosen,
sqlx's `time` feature must be enabled and every generated timestamp field must
carry `#[serde(with = "time::serde::rfc3339")]` (or the `option` module for
optional fields), because the default serde output is a numeric array, not an
RFC 3339 string. Golden tests must cover trailing-zero fractions and the
`+00:00` spelling used by explicit `to_rfc3339()` formatting in `ag-domains`;
a round-trip test alone does not prove the wire format is unchanged. PostgreSQL
`TIMESTAMPTZ` is microsecond precision, so tests do not assert nanoseconds.

### 4.6 Migration path

Migrations stay append-only in the consumer's infrastructure crate, applied via
`ag_data::run_migrations`. Enabling RLS on a populated table is expand/contract:

1. Add nullable `tenant_id`, backfill existing rows, add a temporary default.
2. Set `NOT NULL`, create the `(tenant_id, id)` index, enable and force RLS,
   create the policy, then drop the temporary default.
3. Only when every writer includes `tenant_id`, make it required in the
   generated model.

`CREATE INDEX CONCURRENTLY` cannot run inside the transaction used by
`sqlx::migrate!`. For large tables the implementation PR must choose and
document either a maintenance window or a separate non-transactional step;
generated SQL must not hide it. Rollback is a new compensating migration (drop
policy, disable RLS, drop column), never an edit of an applied file; down
migrations are not assumed.

### 4.7 Tests and documentation

Live tests follow the existing `ag-data` pattern: `#[ignore]` gated on
`DATABASE_URL`, with required variables documented in the module README. Each
test uses two tenants and asserts that a scoped transaction cannot read, update
or insert the other tenant's rows, that an unscoped query returns zero rows,
and that a reused pool connection cannot inherit a previous tenant. Tests run
with a runtime role without `BYPASSRLS`; a separate check covers the privileged
migration role. Generated fixtures and `ag generate --check` run in CI.

Documentation delivered with each increment: the DSL annotation reference, the
crate README for `ag-data`, the generated-code contract in `docs/modules/`, a
diagram under `docs/diagrams/` for the tenant context flow, and an update to the
architecture master (chapter 5 for the type, chapter 8.2 for the data module)
through the master-hash procedure. No master changes are made by this RFC.

## 5. Implementation plan (only after acceptance)

1. PR 1 - DSL marker for tenant-owned/global models, `TenantId` and
   `TenantContext` in `ag-core`, `TenantId` SQL binding in `ag-data`, type map
   aligned with the #185 decision.
2. PR 2 - I1 codegen for ports and adapters with mandatory context and explicit
   predicates; generated fixture and drift gate.
3. PR 3 - I2 RLS migration generation, GUC call, roles and two-tenant live tests.
4. PR 4 - I3 job-envelope propagation, `TenantScope::All` and audit events.
5. PR 5 - master documentation and the first multi-tenant manual chapter.

Each PR states which phase it belongs to; none may be merged while the
pre-Phase-5 gate is open unless the roadmap is amended by a separate decision.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Tenant GUC set at session level on a pooled connection | Medium if hand-written | Cross-tenant data leak | Mandatory `set_config(..., true)` inside `AgTx`; two-tenant reused-connection test |
| RLS disabled or owner bypass active in production | Low | Cross-tenant data leak | `FORCE ROW LEVEL SECURITY`, runtime role without `BYPASSRLS`, explicit predicate as second barrier |
| Generated repository omits the tenant filter | Medium | Cross-tenant data leak | Mandatory context argument, `WITH CHECK`, `ag generate --check`, live tests |
| Date/time decision changes the generated JSON shape | High until #185 lands | Silent API contract break | Provisional mapping, explicit serde adapter, golden-string tests in the same PR as the type change |
| Backfill locks writers | Medium | Operational outage | Expand/contract migrations, documented concurrency choice, rehearsal on a production-sized table |
| Ambiguous admin scope becomes a general bypass | Medium | Audit gap | Named constructor, explicit audit handle, event on each use, no ambient mode |
| Generator complexity slows builds | Medium | Medium | Runtime-checked `query_as` by default; measure `ag generate` before and after |

## 7. Impact

- **Scope:** no new product capability is claimed today. The design fixes how a
  Phase 5+ capability will be built, inside the existing backend-only scope.
- **Roadmap:** belongs to Phase 5+; no phase is opened or advanced by this RFC.
- **Public APIs:** future additions (`TenantId`, `TenantContext`, repository
  traits) are additive to pre-1.0 crates; the existing `AgTx` executor surface
  is reused, not changed.
- **Dependencies:** no new crate. sqlx features may change only as a
  consequence of the accepted #185 decision, in the same PR as the DSL mapping.
- **Operations:** requires a runtime role without `BYPASSRLS`, a separate
  migration role and a documented concurrency decision for large backfills.
- **Documentation:** new DSL reference, module docs, diagram and master update
  at implementation time; the data-layer design note remains the detailed
  engineering input.

## 8. Rollback

This RFC is a document: rollback is to mark it `rejected` or `superseded`, with
no code to revert. Once increments merge, each is revertible independently:
reverting generated code is a regeneration plus `ag generate --check`; reverting
RLS is a new compensating migration (never an edit of an applied file). Signs
that trigger rollback of an increment: two-tenant isolation test failure, a
cross-tenant audit event, a measurable build or migration-time regression
outside the target, or a change in serialized timestamp strings not covered by
golden tests. Rollback of I1 after I2 is not supported until the compensating
migration is applied.

## 9. Acceptance criteria (for future implementation, not met today)

- [ ] `TenantId` and `TenantContext` exist and `ag-core` still depends on no
      Anti-Gravital crate.
- [ ] Generated repository methods require `TenantContext`; a method without it
      does not compile.
- [ ] Generated SQL contains an explicit tenant predicate and insert column.
- [ ] Tenant-owned tables have `USING` and `WITH CHECK` policies, RLS enabled
      and forced, and the runtime role lacks `BYPASSRLS`.
- [ ] The GUC is set with `set_config('app.tenant_id', $1, true)` inside `AgTx`.
- [ ] Two-tenant `#[ignore]` tests pass against live PostgreSQL with documented
      env vars, including the reused-connection case.
- [ ] `ag generate --check` passes on generated fixtures in CI.
- [ ] Timestamp serialization is covered by golden strings, including the
      #185-decided type, trailing-zero fractions and explicit `to_rfc3339()`.
- [ ] `TenantScope::All` is reachable only through its audited constructor.
- [ ] Master documents, module docs and the dependency/context diagram are
      updated in the same PR as the code that changes them.

Current status: **none of the boxes above is met**. Accepted as design on
2026-11-22 with implementation deferred (ADR-0020). Issue #154 stays open.

## 10. Decision

- Decisor: BDFL (Angel Nereira, project maintainer); recorded by the
  architecture committee chair.
- Decision date: 2026-11-22 (`CLAUDE.md` rule 28).
- Result: ACCEPTED AS DESIGN. Implementation is DEFERRED until Phase 5 opens
  and every blocking row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` is `pass`
  (ADR-0020). No increment I1-I3 may start before then; the gate is still open
  on the decision date (Audit and Deny `fail`, Fuzz 24h, Benchmarks and Open
  debt `pending`). The #185 timestamp decision must land before the DSL type
  map is frozen in generated code. Issue #154 stays open.

## 11. References

- Issue #154, issue #185, issue #142 (tracking).
- `docs/modules/ag-data/data-layer-rls-tenancy.md` - data-model review
  incorporated into sections 2, 4.2, 4.3, 4.4, 4.5 and 4.6.
- RFC-0019 sections 4.1, 4.2, 4.4; RFC-0020 sections 4.1-4.3; ADR-0013.
- `crates/ag-data/src/lib.rs` (`DbPool`, `AgTx`, `DataConfig`).
- `crates/ag-dsl/src/ast.rs` and `crates/ag-dsl/src/codegen/rust_gen.rs`
  (current `Timestamp` mapping).
- `crates/ag-workers/src/ids.rs` and `crates/ag-workers/src/job.rs`
  (existing tenant propagation through job envelopes).
- `CLAUDE.md` rules 5, 15, 16, 22, 26, 36, 37.
