# Data layer design: typed ORM, row-level security and multi-tenancy

- Status: design note, not implemented. Target: Phase 5+ (ORM track may start in
  an earlier phase, per the Hoja de Ruta).
- Tracks: issue #154 (ag-data ORM / RLS / tenancy).
- Related decisions: RFC-0019 (layered generated projects), RFC-0020 (declared
  environment variables and secrets), ADR-0013 (transaction handle `AgTx`),
  ADR-0009 rule 2 (external services behind adapters, native mode by default),
  ADR-0008 (English is canonical for technical documentation).
- Scope: design only. No code, no new crate, no change to the current public API
  is proposed here. The architect turns the decisions into an RFC before any
  implementation, as required by CLAUDE.md rules 5 and 22.

## 1. Current state (verified in this tree)

- `ag-data` exposes `DataConfig`, `DbPool` (`sqlx::PgPool`), `connect()`,
  `run_migrations()`, `DataError` and the `AgTx` transaction handle. `AgTx`
  exposes `as_executor() -> &mut sqlx::PgConnection`; this is the sanctioned
  single point where an executor leaves the crate (ADR-0013 section 6).
- `ag-data` ships no migrations of its own. Consuming crates own their SQL and
  embed it with `sqlx::migrate!`. The live test under `tests/migrations/` is a
  fixture, not a product migration.
- The DSL maps `Timestamp` to `chrono::DateTime<chrono::Utc>` in
  `crates/ag-dsl/src/ast.rs` and emits `use chrono::{DateTime, Utc};` from
  `codegen/rust_gen.rs` when a schema has a timestamp field. The generated SQL
  type is `TIMESTAMPTZ`. This mapping is a design input to issue #185.
- Tenant keys exist today only as ad-hoc, per-crate representations:
  `ag_workers::ids::TenantId` (a `String` newtype, used as a nullable column in
  the job queue and propagated into `JobEnvelope`), `ag_mail`'s `tenant: String`
  in the MTA spool, and `ag_domains`' project/tenant `String` used by the abuse
  limiter. There is no shared tenant type and no database-enforced isolation.
- Workspace `sqlx` 0.9.1 is configured with features
  `runtime-tokio`, `tls-rustls-ring-webpki`, `postgres`, `migrate`, `macros`,
  `uuid`, `json`, `chrono`. The `time` feature is not enabled and sqlx 0.9 has
  no `jiff` feature.

## 2. Tenant key propagation

### 2.1 One tenant key type

Introduce a single opaque `TenantId` in `ag-core` (no crate dependency on
`ag-data` or any I/O crate, per rule 15). It is a validated newtype over the
canonical representation chosen by the RFC (UUID is recommended: fixed size,
index-friendly, no ambiguity with slugs or email-like values). It implements
`serde` as a string, `Display` for logs and correlation, and, in `ag-data`,
`sqlx::Type<Postgres>` / `Encode` / `Decode` as a transparent wrapper over the
chosen SQL column type. Generated code binds the newtype directly, never a bare
`String`, so a tenant id cannot be confused with an arbitrary text parameter.

Alignment of the existing per-crate tenant types is a follow-up, not part of
this issue: `ag-workers` already carries `Option<TenantId>` into every job
envelope, which is the right shape for propagating a tenant across an
asynchronous boundary. The RFC must decide whether to unify the types now or
to add explicit conversion adapters at each boundary.

### 2.2 Where the key enters and how it travels

```
verified JWT claim (ag-core shield)  or  trusted edge header
        |  (RFC-0019 interface layer, <name>-api)
        v
TenantContext { tenant_id, trace_id }   (no I/O, no pool)
        |  explicit method argument
        v
application use case  (<name>-app, ports only)
        |  explicit method argument
        v
generated repository  (<name>-infra/src/db/generated)
        |  begins AgTx, sets the transaction-local GUC, then queries
        v
PostgreSQL RLS policy
```

Rules:

1. The tenant id is derived from an authenticated claim or from a header that
   the edge is configured to trust. It is never read from a request body, a
   query parameter or configuration. RFC-0020 keeps per-request values out of
   the configuration loader entirely.
2. `TenantContext` is passed explicitly to generated repository methods. There
   is deliberately no ambient/global context and no "current tenant" thread
   local: an omitted argument is a compile error, which is the enforcement
   mechanism RFC-0019 prefers over a custom lint.
3. `trace_id` in the context is carried into `ag-workers` jobs (the envelope
   already has that field) so an RLS decision can be correlated with the
   originating request.
4. Background workers and scheduled jobs reconstruct the context from the
   envelope before touching tenant-owned tables. A job that has no tenant id
   must not be able to read tenant-owned tables; it either targets global
   tables only or is rejected.
5. `DATABASE_URL` is declared `secret = true` in `ag.toml` (RFC-0020). The
   composition root exposes it once when building `DataConfig`; `ag-data`
   continues to redact it in logs through `sanitize_url`. A tenant id is not a
   secret in the RFC-0020 sense and is safe to log for audit purposes.

### 2.3 The transaction-local setting

Before any tenant-owned query, the generated repository begins an `AgTx` and
executes, through `as_executor()`:

```sql
SELECT set_config('app.tenant_id', $1, true);  -- third argument true = local to this transaction
```

`true` is mandatory. A session-level `SET app.tenant_id = ...` would survive on
a pooled connection and leak one tenant's context into the next request that
happens to receive that connection. With `sqlx::PgPool`, `SET LOCAL` (or
`set_config(..., true)`) inside an open transaction is the only correct option.
The generated repository owns the transaction: it sets the value, performs its
queries, and commits or rolls back. If a use case needs several repositories to
participate in one transaction, the context is set once on the shared `AgTx`
and every generated repository accepts `&mut AgTx` instead of `&DbPool`.

## 3. RLS policy model

### 3.1 Table classes

| Class | `tenant_id` column | RLS policy | Example |
| --- | --- | --- | --- |
| Tenant-owned | required, `NOT NULL` | `USING` and `WITH CHECK` compare it to the context | generated domain tables, job queue, mail spool |
| Global / reference | absent or nullable with NULL meaning global | no tenant policy; readable by all tenants, writable only by admin scope | country codes, feature flags |
| System | absent | no policy; reachable only by migrations and admin tooling | `_sqlx_migrations`, audit log |

The DSL needs a per-model marker (for example `@tenant` or `tenant: true`) to
say that a generated table is tenant-owned. Default is tenant-owned for domain
models; global tables must be opted in explicitly, so a missing marker cannot
silently create a cross-tenant read.

### 3.2 Canonical policy

Generated per tenant-owned table by a migration, with a stable policy name:

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

Properties:

- Fail closed: `current_setting(..., true)` returns NULL when the GUC was never
  set; `NULLIF(..., '')` also handles empty strings. Comparing a `uuid` to NULL
  yields no rows, so a query that forgets to open a scoped transaction sees
  nothing instead of everything.
- `WITH CHECK` prevents a tenant from inserting or moving a row into another
  tenant's partition; `FOR ALL` covers SELECT, INSERT, UPDATE and DELETE with
  one policy. If a table needs a different rule for a verb, the generator emits
  separate policies rather than a permissive combination.
- `FORCE ROW LEVEL SECURITY` makes the policy apply to the table owner too. The
  migration role therefore needs `BYPASSRLS` (or the policies must be created
  before FORCE is enabled); this is a deployment prerequisite to document, not
  an application role.
- RLS is defence in depth, not the only control. Generated SQL also contains an
  explicit `WHERE tenant_id = $1` predicate and an explicit `tenant_id` column
  in `INSERT`. If RLS is accidentally disabled, the query is still scoped.

### 3.3 Admin and cross-tenant access

A single named escape hatch, `TenantScope::All`, is available only through a
separate constructor on the repository that requires an explicit audit handle.
It must be impossible to obtain by forgetting a parameter. Every use emits an
`ag-observe` audit event with the actor, table, predicate and reason. Generated
application code never uses it; it exists for migrations, support tools and
reporting. PostgreSQL roles with `BYPASSRLS` are limited to the migration and
operator roles, never to the runtime role of a generated application.

## 4. Generated ORM shape

### 4.1 Output paths (RFC-0019)

| Artifact | Destination |
| --- | --- |
| Domain model structs and validation | `crates/domain/src/generated/` |
| Repository traits (ports) | `crates/app/src/generated/` |
| SQL migrations and `sqlx` repository implementations | `crates/infra/migrations/`, `crates/infra/src/db/generated/` |
| Route definitions and request/response types | `crates/api/src/generated/` |

Everything under `generated/` is fully rewritten by `ag generate`;
`ag generate --check` fails when committed output differs from `schema.ag`.
Compact layout mirrors the same four names as modules.

### 4.2 Illustrative API

```rust
// app/generated: port. The context is mandatory on every method.
#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn find_by_id(
        &self,
        ctx: &TenantContext,
        id: Uuid,
    ) -> Result<Option<Todo>, DataError>;
    async fn list(&self, ctx: &TenantContext, page: Page) -> Result<Vec<Todo>, DataError>;
    async fn insert(&self, ctx: &TenantContext, value: &NewTodo) -> Result<Todo, DataError>;
}

// infra/generated: adapter. Holds the pool; creates the scoped transaction.
pub struct PgTodoRepository { pool: DbPool }

impl PgTodoRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }
}
```

The generated implementation begins a transaction, calls `set_config`, runs
parameterised statements with an explicit tenant predicate, and commits. It
holds no tenant state of its own; a single repository instance is safe to share
across requests. That is important because `DbPool` is cheaply cloneable and is
normally injected once by the composition root.

### 4.3 Type mapping and query verification

| DSL type | Rust type (generated) | PostgreSQL type |
| --- | --- | --- |
| `Uuid` | `uuid::Uuid` | `UUID` |
| `String` | `String` | `TEXT` |
| `Int` | `i64` | `BIGINT` |
| `Float` | `f64` | `DOUBLE PRECISION` |
| `Bool` | `bool` | `BOOLEAN` |
| `Timestamp` | currently `chrono::DateTime<chrono::Utc>` | `TIMESTAMPTZ` |
| `Decimal` | `rust_decimal::Decimal` | `NUMERIC` |

The `Timestamp` row is explicitly provisional and belongs to issue #185. Of
the candidates under evaluation, `time::OffsetDateTime` is the only one with
native sqlx 0.9 support once the `time` feature is enabled; sqlx 0.9 has no
`jiff` feature, so `jiff::Timestamp` would require a conversion shim at every
SQL boundary. If the date/time type changes, the DSL type mapping, the
generated `use` line and the SQL feature set must change in the same PR.

Two query-verification modes are possible:

- Default: derive `sqlx::FromRow` on generated models and use the runtime-checked
  `sqlx::query_as::<_, T>` with bound parameters. Generated projects then build
  without a database, which matches the reproducibility rule (CLAUDE.md 36)
  and the `git clone; cargo build; cargo test` promise (rule 37).
- Opt-in: `query_as!` (compile-time checked) with committed `.sqlx` offline
  metadata produced by `cargo sqlx prepare`. Useful for large schemas, but it
  adds a preparation step that must run in CI and stay in sync with migrations.

Generated code must never reach `sqlx::PgPool` directly outside `ag-data`'s
abstractions; `ag-data` remains the single sanctioned sqlx boundary.

## 5. Migration strategy

1. Migrations are owned by the consumer's infrastructure crate under
   `crates/infra/migrations/` and applied at startup via `ag_data::run_migrations`.
   They are append-only: sqlx stores checksums, and an applied file is never
   edited.
2. Enabling RLS on an existing populated table is an expand/contract sequence in
   ordered migrations:
   1. Add `tenant_id UUID NULL` (or the chosen type), add the backfill default
      or a mapping table, and backfill existing rows.
   2. Set `NOT NULL`, create the `(tenant_id, id)` index, enable and force RLS,
      create the policy, then drop the temporary default.
   3. Only after every writer includes `tenant_id`, make the column part of the
      generated model as required. 
3. `CREATE INDEX CONCURRENTLY` cannot run inside the transaction that
   `sqlx::migrate!` uses by default. For large tables the RFC must choose
   either a documented maintenance window or a separate non-transactional
   migration step; this cannot be hidden in generated SQL.
4. Rollback is a new compensating migration (drop policy, disable RLS, drop
   column), never an edit of an applied file. Down migrations are not assumed.
5. Live tests follow the existing `ag-data` pattern: `#[ignore]` tests gated on
   `DATABASE_URL`, with the required env vars documented in the module README.
   Each test uses two tenant ids and asserts that a scoped transaction cannot
   read, update or insert the other tenant's row, and that an unscoped query
   returns zero rows. Tests must use a runtime role without `BYPASSRLS`, plus a
   separate migration check for the privileged role.

## 6. sqlx 0.9 compatibility checklist

- The workspace currently enables the sqlx `chrono` feature. Switching the
  generated model type to `time::OffsetDateTime` requires enabling sqlx's
  `time` feature and adding `time` as a workspace dependency; removing the
  `chrono` feature is only safe when no remaining crate binds `chrono` values.
- sqlx 0.9 has no `jiff` feature. A `jiff` choice implies manual conversion to a
  supported type at every binding site (notably the `ag-workers` PostgreSQL
  queue and `ag-domains` stores), which is a recurring source of mistakes.
- PostgreSQL `TIMESTAMPTZ` has microsecond resolution while Rust date/time
  types carry nanoseconds. Tests must not assert nanosecond precision after a
  database round trip.
- `set_config` with the third argument `true` must run inside the `AgTx`
  transaction; `AgTx::as_executor` already provides the required
  `&mut PgConnection`. No new executor surface is needed.
- `TenantId` binding is implemented once in `ag-data` via sqlx's `Type`,
  `Encode` and `Decode` traits (transparent over `TEXT` or `UUID`), so generated
  repositories have no per-crate conversions.
- `sqlx::migrate!` resolves its path relative to the calling crate's
  `Cargo.toml`; generator output must emit the path accordingly (RFC-0019 puts
  it in the infrastructure crate).

## 7. Open decisions for the architect's RFC

1. Canonical `TenantId` representation (UUID vs validated string) and the
   alignment path for the existing `ag-workers`, `ag-mail` and `ag-domains`
   tenant keys.
2. Exact claim name and trust rule at the interface layer; behaviour when the
   claim is absent (recommended: reject the request, never fall back to a
   default tenant).
3. `TenantScope::All` API shape, audit event fields and who may obtain it.
4. Whether RLS generation lands with the first ORM increment or in a later
   increment of issue #154, and which generated tests gate each increment.
5. The date/time type selected in issue #185, because it fixes the DSL
   `Timestamp` mapping and the sqlx feature set.
6. Whether the generated default is runtime-checked `query_as` or committed
   offline metadata for `query_as!`.

## 8. Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Tenant GUC set at session level on a pooled connection | Cross-tenant data leak | Mandatory transaction-local `set_config(..., true)`; test with two tenants and a reused pool connection. |
| RLS disabled or owner-bypass active in production | Cross-tenant data leak | `FORCE ROW LEVEL SECURITY`, runtime role without `BYPASSRLS`, explicit predicate in generated SQL as second layer. |
| A generated repository omits the tenant filter | Cross-tenant data leak | Mandatory `TenantContext` argument, `WITH CHECK` policy, `ag generate --check` gate, two-tenant live tests. |
| Generated type mapping drifts from the date/time decision | Generated projects do not compile | Single source of truth in `ag-dsl::ast::rust_type`; change mapping, imports and sqlx features together. |
| Backfill of an existing table locks writers | Operational outage | Expand/contract migrations and a documented concurrency decision (section 5.3). |
| Ambiguous admin scope becomes a general bypass | Audit gap and data exposure | Named constructor, explicit audit handle, audit event on every use; no ambient admin mode. |

## 9. References

- Issue #154: `ag-data`: DSL-generated typed ORM, row-level security and
  multi-tenancy (Phase 5+).
- Issue #185: chrono replacement; data-model review recorded in the community
  board notes.
- `docs/rfc/RFC-0019-project-layered-architecture.md`, sections 4.1, 4.2, 4.4.
- `docs/rfc/RFC-0020-environment-and-secrets.md`, sections 4.1, 4.2, 4.3.
- `crates/ag-data/src/lib.rs` (`DbPool`, `AgTx`, `DataConfig`, migrations).
- `crates/ag-dsl/src/ast.rs`, `crates/ag-dsl/src/codegen/rust_gen.rs`.
- `crates/ag-workers/src/ids.rs`, `crates/ag-workers/src/job.rs`,
  `crates/ag-workers/src/queue/postgres.rs` (existing tenant key and SQL
  binding precedent).
- `CLAUDE.md` rules 5, 15, 16, 22, 26, 36, 37.
