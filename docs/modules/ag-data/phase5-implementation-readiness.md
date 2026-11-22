# Phase 5+ implementation readiness for typed ORM, RLS and multi-tenancy (issue #154)

- Status: additive design note. It is NOT a decision, NOT an RFC, and NOT an
  implementation authorisation.
- Contract: RFC-0025 (ACCEPTED AS DESIGN, implementation deferred) and ADR-0020
  (acceptance and deferral, 2026-11-22). This note supersedes nothing and amends
  neither document.
- Tracks: issue #154. Affected phase: Phase 5 and later only.
- Date: 2026-11-22. Language: English canonical (ADR-0008).
- Related: `docs/modules/ag-data/data-layer-rls-tenancy.md`, RFC-0019, RFC-0020,
  RFC-0026, ADR-0013, `docs/audits/PRE_FASE5_RELEASE_GATE.md`.

## Why this note exists, and why no new RFC was opened

RFC-0025 already carries the full design for this capability: motivation,
problem, alternatives (including the sqlx macro versus DSL-generated versus
hand-written choice), design (typed query builders, RLS expressed in the schema,
tenant context), risks, impact, rollback, and a phased rollout explicitly gated
behind Phase 5+. ADR-0020 accepted that design and deferred implementation.

Opening a second, competing RFC marked `proposed` would duplicate and contradict
an accepted architectural contract. That would breach CLAUDE.md rules 2, 4, 5, 6
and 23, and the RFC `Vigencia` rule, which permits a successor RFC only when
reality diverges from an accepted decision. Nothing has diverged here; the
deferral is intentional. The architecture folder is verbatim-from-master only
(`docs/architecture/README.md`, rule 2), so additive engineering detail belongs
in module docs, which RFC-0025 already cites as its technical input.

This note therefore stays strictly inside the accepted design. It fixes the
concrete seams an implementer will otherwise have to guess: the exact DSL
spelling of the tenant marker, the shape of the typed repository flow, the
identity-module hand-off, and how the drift gate covers generated RLS. If the
committee judges any refinement below to be a grammar or scope change beyond
RFC-0025, a successor RFC is required before implementation (CLAUDE.md rules 22
and 28).

## 1. Motivation

Issue #154 is a forward-looking capability, not a defect: typed queries, RLS
and multi-tenancy are generated from Anti-DSL on top of the `ag-data` pool and
none of them exist yet. `ag-data` currently exposes `DataConfig`, `DbPool`,
`connect()`, `run_migrations()`, `DataError` and the `AgTx` handle with
`AgTx::as_executor()` (ADR-0013). The DSL emits model structs and SQL, but no
repositories and no tenant policy.

Two things have moved since ADR-0020 was written:

1. The date/time dependency is resolved. The DSL maps `Timestamp` to
   `time::OffsetDateTime` (`crates/ag-dsl/src/ast.rs`, `rust_type`), the mapping
   is documented in `docs/dsl/referencia-v01-v04.md`, and the serde contract is
   fixed by RFC-0026. A data layer no longer has to treat the type map as
   provisional.
2. The release gate has partially advanced: Audit and Deny are now `pass`
   (2026-11-22). The gate is still open, so the deferral stands.

What is still missing is an implementer-level specification of three seams that
RFC-0025 leaves open. This note supplies them without changing the contract.

## 2. Problem (additive gaps only)

1. The marker exists as an idea but not as grammar. RFC-0025 section 4.4 gives
   `@tenant` and `tenant: true` as examples and fixes neither; the current DSL
   annotation grammar (`docs/dsl/referencia-v01-v04.md`, section 3) has no
   tenancy annotation.
2. The identity hand-off is unspecified. `ag_auth::Claims`
   (`crates/ag-auth/src/jwt.rs`) has `sub`, `exp`, `iat`, `jti` and `role` but no
   tenant field, while RFC-0025 section 4.2 requires a verified `tenant_id`
   claim. RFC-0025's affected-module list does not name `ag-auth`, so the crate
   boundary needs to be stated explicitly.
3. Drift-gate coverage is underspecified: RFC-0019's `ag generate --check` must
   be stated to cover the generated RLS migration as well as models, ports and
   adapters, otherwise RLS can be dropped from committed output silently.

The core problems (no typed ORM, no shared tenant key, no database-enforced
isolation, no RLS migration path) are already analysed in RFC-0025 section 2
and `docs/modules/ag-data/data-layer-rls-tenancy.md` section 1 and are not
repeated here.

## 3. Alternatives (reaffirmed, not reopened)

RFC-0025 section 3 already selects option E: generated typed ORM with a
mandatory `TenantContext` plus generated RLS from one schema. Options A (no
change), B (hand-written repositories plus a predicate lint), C (a third-party
ORM owning tenancy) and D (RLS only) were rejected there for the same reasons:
the compiler and the database are enforceable boundaries, a linter is not, and
the sqlx boundary (ADR-0013) must stay single. Those alternatives are not
relitigated by this note.

The query-verification choice is also settled: RFC-0025 section 4.5 selects
runtime-checked `sqlx::query_as` with derived `FromRow` as the default (so
generated projects build without a database, per CLAUDE.md rules 36 and 37),
with `query_as!` plus committed `.sqlx` metadata as an opt-in. This note does
not propose a third mode.

## 4. Concrete refinements (design)

All of this is Phase 5+ work under RFC-0025 section 5 and is not authorised by
this document.

### 4.1 Expressing tenancy and RLS in `schema.ag`

Extend the existing `@`-annotation grammar at model level:

| Annotation | Meaning | Generated effect |
| --- | --- | --- |
| `@tenant` | Marks a `model` tenant-owned. Explicit form of the default (RFC-0025 section 4.4). | Generated `tenant_id` column, RLS migration, context-aware port. |
| `@global` | Opt-out for reference/global tables. | No tenant policy; writes only through admin scope. |

Rules that make the accepted "a missing marker cannot silently create a
cross-tenant read" property concrete:

- Tenant-owned is the default for `model` blocks. `@tenant` is optional and
  exists for schemas that prefer explicitness; `@global` must be deliberate.
- System tables (`_sqlx_migrations`, the audit log) are not DSL models and need
  no marker.
- `@tenant` and `@global` on the same model is a lint error.
- The tenant key column `tenant_id UUID NOT NULL` is generated for every
  tenant-owned model. It is reserved: a schema that declares its own
  `tenant_id` field is a lint error. This mirrors `@auto`/`@auto_update`, whose
  fields are already excluded from `Create*`/`Update*` request types
  (`docs/dsl/referencia-v01-v04.md`, section 4). The repository binds
  `ctx.tenant_id` on INSERT; no database default is emitted, because a random
  `gen_random_uuid()` tenant would be a correctness bug.
- New tenant-owned tables get
  `CREATE INDEX IF NOT EXISTS "idx_<table>_tenant_id_id" ON "<table>" ("tenant_id", "<pk>");`.
  The populated-table path keeps the expand/contract sequence and composite
  index of RFC-0025 section 4.6.
- RLS is emitted into a separate append-only migration (for example
  `<n>_rls.sql`) so `ENABLE`/`FORCE` and the policy can be applied after
  backfill. The policy name `ag_tenant_isolation` is per table (PostgreSQL
  scopes policy names by table); `USING` and `WITH CHECK` are exactly those in
  RFC-0025 section 4.4, with `NULLIF(current_setting('app.tenant_id', true), '')::uuid`
  and fail-closed semantics.
- `ag schema lint` diagnostics to add: error on a hand-declared `tenant_id`;
  error on `@tenant` together with `@global`; warning when a `@global` model
  has a foreign key to a tenant-owned model (a possible cross-tenant join that
  deserves review).

This is a refinement of the marker already authorised by RFC-0025 section 4.4
and section 5 PR 1, not a new decision. If the committee treats the grammar as
a change beyond RFC-0025, a successor RFC is required first.

### 4.2 Typed query builders

For each tenant-owned model the generator emits one port trait under
`crates/app/src/generated/` and one adapter under
`crates/infra/src/db/generated/` (RFC-0019 paths; the compact layout mirrors the
same four module names):

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

Every generated method follows one fixed sequence: `AgTx::begin(&pool)`, then
`SELECT set_config('app.tenant_id', $1, true);` through `AgTx::as_executor()`
(third argument `true` is mandatory: a session-level `SET` leaks across pooled
connections), then a parameterised statement with an explicit
`WHERE tenant_id = $1` and an explicit `tenant_id` insert column, then
commit or rollback. Read-only methods also open a transaction because the GUC
is transaction-local. A use case spanning several repositories sets the GUC
once on a shared `&mut AgTx`.

The adapter holds only a cheaply cloneable `DbPool` and no tenant state, so one
instance is safe to share across requests. Generated code never reaches
`sqlx::PgPool` outside `ag-data`. `ag generate --check` covers domain structs,
ports, adapters, the initial migration **and** the generated RLS migration; a
drift in any of them fails CI.

### 4.3 Multi-tenant scope tied to the identity module

The scope enters from `ag-auth`, the verified-identity module, as follows:

1. RFC-0025 section 4.2 fixes the claim name `tenant_id`. In RFC-0025 PR 1,
   `ag_auth::Claims` gains that field and `JwtSigner::verify()` keeps Ed25519
   verification and returns it. No new token flow or format is invented; the
   claim name is already decided.
2. The generated interface layer (`<name>-api`) builds
   `TenantContext { tenant_id, trace_id }` from the verified claim. A token
   without `tenant_id` on a tenant-owned endpoint is rejected, never mapped to
   a default tenant. The trusted-edge-header path stays exactly as RFC-0025
   section 4.2 (the edge strips the inbound header before setting it).
3. `TenantContext` is passed explicitly into the app use case and the
   repository. There is no ambient or thread-local "current tenant"; an
   omitted argument is a compile error. `ag-data` does **not** depend on
   `ag-auth` (`docs/modules/ag-data/README.md` dependency rule): the claim is
   validated into the `ag_core::TenantId` newtype at the interface boundary,
   and `ag-data` only binds that newtype through its single sqlx `Type`/`Encode`/
   `Decode` implementation.
4. `trace_id` is carried into `ag-workers` job envelopes, which already have a
   tenant field and a trace field, so a background job reconstructs the scope
   before touching tenant-owned tables. A job without a tenant id either
   targets global tables only or is rejected.

Adding the claim to `ag-auth` adds that crate to the modules touched by RFC-0025
PR 1 (RFC-0025's affected-module list omits it). Because the claim name is
already decided, this is an implementation seam, not a policy change. Record it
in the implementation PR, or open a successor RFC if the committee considers it
a scope change.

### 4.4 Phased rollout (all gated behind Phase 5+)

| Increment | Content (from RFC-0025 section 4.1) | Concrete gate in this note |
| --- | --- | --- |
| I1 - typed ORM | DSL marker, `TenantId`/`TenantContext`, `ag-auth` claim, ports and adapters, drift gate | Generated fixture builds; two-tenant tests; `ag generate --check` includes RLS file |
| I2 - RLS | Separate RLS migration, GUC call, roles | Two-tenant tests with RLS forced; unscoped query returns zero rows; reused connection cannot inherit a tenant |
| I3 - async and admin scope | Job propagation, `TenantScope::All`, audit events | Envelope round trip; audited admin event asserted |

No increment starts while the pre-Phase-5 release gate is open. The five-PR
implementation plan is RFC-0025 section 5 and is not reopened here.

## 5. Risks (additive)

The full risk set is RFC-0025 section 6. Additional risks from the refinements:

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Marker syntax ambiguous between schemas | Inconsistent generated policy | Default tenant-owned plus the lint diagnostics in section 4.1; successor RFC if the grammar needs a formal change |
| Deployed service issues tokens without `tenant_id` | Every tenant-owned endpoint rejects legitimate traffic | Reject with a clear diagnostic at the interface layer, never default to a tenant; cover the missing-claim case in tests |
| RLS file excluded from drift gate | Policy dropped silently on regeneration | Include the generated RLS migration in `ag generate --check` (section 4.2) |
| `ag-auth` treated as out of scope | Claim seam implemented ad hoc outside the identity module | Record the crate in the implementation PR; successor RFC if the committee requires it |

## 6. Impact

Document only. No crate is added or changed, no phase is opened, no public API,
release gate, RFC status or ADR is altered. Issue #154 remains open. No README
status change is required because the observable phase state does not change;
the module README gains a pointer to this note for discoverability.

## 7. Rollback

This is a document: rollback is to mark it `superseded` or `rejected`, with no
code to revert. It creates no contract, so it cannot invalidate RFC-0025 or
ADR-0020.

## 8. From deferred to actionable

The deferral is lifted only when all of the following hold:

1. Every blocking row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` is `pass`.
   Pending at the date of this note: Fuzz 24h (#156), Benchmarks (#157) and
   Open debt. Audit and Deny are now `pass`; the `fail` text in ADR-0020 was an
   older snapshot and is superseded by the gate document.
2. Phase 5 opens, per the Hoja de Ruta.
3. A separate implementation PR records the gate closure and the phase before
   merging any increment (ADR-0020 point 3). Implementation then follows the
   RFC-0025 section 5 five-PR plan without reopening the design.
4. The timestamp decision is met: `time::OffsetDateTime` is already the DSL
   mapping and RFC-0026 fixes the serde contract, so this ADR-0020 dependency
   no longer blocks.
5. If the committee treats the section 4.1 grammar or the section 4.3 `ag-auth`
   addition as beyond RFC-0025, a successor RFC is accepted first.

## 9. References

- Issue #154; RFC-0025; ADR-0020.
- `docs/modules/ag-data/data-layer-rls-tenancy.md` (engineering input to RFC-0025).
- RFC-0019 sections 4.1, 4.2, 4.4; RFC-0020; RFC-0026; ADR-0013; ADR-0008.
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (current gate state).
- `crates/ag-data/src/lib.rs` (`DbPool`, `AgTx`, `run_migrations`).
- `crates/ag-dsl/src/ast.rs`, `docs/dsl/referencia-v01-v04.md` (grammar and type map).
- `crates/ag-auth/src/jwt.rs` (`Claims`, `JwtSigner::verify`).
- `CLAUDE.md` rules 2, 4, 5, 6, 8, 22, 23, 26, 28.
