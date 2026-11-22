# {{name}} - multi-tenant SaaS API

A starter for tenant-scoped HTTP services. Tenant identity is resolved once in
middleware, validated, and propagated to every handler through request
extensions. Storage is keyed by `(tenant_id, id)` so cross-tenant reads are
structurally impossible in the default build.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP and security | `ag-core` (`Shield`), axum 0.7 |
| Tenant context | `X-Tenant-Id` header, validated and inserted into extensions |
| Isolation | composite key `(tenant_id, resource_id)` in memory |
| Ids | `uuid` v4 |

## Run

```bash
cargo run

curl -H 'X-Tenant-Id: acme' http://127.0.0.1:8080/resources
curl -H 'X-Tenant-Id: acme' -X POST http://127.0.0.1:8080/resources \
  -H 'content-type: application/json' -d '{"name":"invoice-1"}'
curl -H 'X-Tenant-Id: globex' http://127.0.0.1:8080/resources
```

The two tenants see disjoint sets. A request without `X-Tenant-Id` (or with
an invalid value) receives 400 before reaching any handler. `/health` is
tenant-agnostic.

Valid tenant ids are 1 to 64 ASCII alphanumeric characters, `-` or `_`.

## Tenant propagation

```
request
  -> tenant_context_middleware (validate X-Tenant-Id)
  -> Request::extensions_mut().insert(TenantId)
  -> handlers read Extension<TenantId>
  -> storage key = (tenant_id, resource_id)
```

The header is used here for clarity. In production, derive the tenant from a
**verified** JWT claim (`ag_auth::Claims` can be extended with `tenant_id`)
after Shield's `auth-jwt` layer has validated the token. Never trust a raw
client header for authorization.

## PostgreSQL row-level security (documented, not implemented)

`external_requirements` declares `DATABASE_URL` because a real SaaS workload
needs durable storage. This template intentionally ships **no RLS code**: the
default build stays offline and the memory store demonstrates the isolation
contract. A production migration would add `tenant_id` to every tenant-owned
table and enforce access in the database, for example:

```sql
ALTER TABLE resources ENABLE ROW LEVEL SECURITY;

CREATE POLICY resources_tenant_isolation ON resources
  USING (tenant_id = current_setting('app.tenant_id', true));
```

Each request-scoped transaction then sets the context before any query:

```sql
BEGIN;
SET LOCAL app.tenant_id = 'acme';
-- application queries here
COMMIT;
```

With `ag-data`, the pool from `ag_data::connect` and the `AgTx` transaction
helper are the sanctioned place to apply this pattern. Keep the application
layer's tenant key as defense in depth; do not rely on a single mechanism.

## Limitations

- No authentication: the tenant header identifies a tenant, not a user.
- No tenant provisioning, quota or plan enforcement.
- The in-memory store is per process and resets on restart.
- The RLS snippets above are documentation, not compiled or tested SQL.
- `list` has no pagination.
