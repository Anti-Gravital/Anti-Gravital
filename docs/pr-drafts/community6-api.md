# Descriptor de Pull Request

## Resumen

Add six API and service templates to the wave 6 catalog

## Fase afectada

Wave 6, template catalog. No roadmap phase changes; no RFC required. Templates
are additive and consume the existing 19-crate ecosystem plus the pinned
`ag-*` git revision already used by the four core templates.

## Tipo de cambio

- [x] Added: six template directories under `templates/`
- [x] Added: CHANGELOG entry under `[Unreleased]`
- [ ] Changed: none in shared CLI code or existing templates
- [ ] Fixed: none
- [ ] Removed: none
- [ ] Docs: template metadata, READMEs, schema files and migration files

## Documentos relacionados

- `templates/README.md` (existing catalog description, untouched by this branch)
- `CLAUDE.md` rules 14, 15, 19, 26 and the descriptor policy
- `ADR-0008` (English canonical for technical documentation and code comments)
- `examples/ecommerce-api/schema.ag` (source of the ecommerce-lite subset)

## Cambios concretos

| Template | Highlight | External requirement |
| --- | --- | --- |
| `database-api` | ag-data pool, migrations, in-memory fallback | `DATABASE_URL` |
| `auth-service` | ag-auth JWT and API keys, Shield | none |
| `multi-tenant-saas` | tenant context propagation, RLS documented | `DATABASE_URL` |
| `webhook-gateway` | constant-time HMAC-SHA256 verification | `WEBHOOK_SECRET` |
| `file-upload-service` | ag-storage signed prefix capabilities | `STORAGE_SIGN_SECRET` |
| `ecommerce-lite` | products/orders with transactional stock | `DATABASE_URL` |

All templates pin `ag-*` dependencies to
`91a95ff61ed6ef4c9d3f1c9069357e97389d1382`, matching the existing core
templates. Default builds need no live service.

## Plan de prueba

Scaffold each template to `/tmp`, replace `{{name}}`, and run offline checks
with the shared target directory:

```bash
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
for t in database-api auth-service multi-tenant-saas webhook-gateway file-upload-service ecommerce-lite; do
  bash /tmp/ag-api-scaffold.sh "templates/$t" "/tmp/ag-api-$t" "check-$t"
  (cd "/tmp/ag-api-$t" && cargo check --offline)
done
```

Additional checks executed:

- `cargo check --offline --features postgres` for `database-api` and
  `ecommerce-lite`.
- `cargo test --offline` for `webhook-gateway` (3 signature tests) and
  `file-upload-service` (2 confinement tests).
- `cargo fetch` once to warm the local cache with `passkey-types 0.5`, which
  the pinned `ag-auth` revision needs and which was absent from the shared
  cache. The final verification commands above ran fully offline.

## Resultado de la verificacion

- `database-api`: `cargo check --offline` OK; with `--features postgres` OK.
- `auth-service`: `cargo check --offline` OK.
- `multi-tenant-saas`: `cargo check --offline` OK.
- `webhook-gateway`: `cargo check --offline` OK; `cargo test --offline` 3 passed.
- `file-upload-service`: `cargo check --offline` OK; `cargo test --offline` 2 passed.
- `ecommerce-lite`: `cargo check --offline` OK; with `--features postgres` OK.

No live PostgreSQL, SMTP or object storage endpoint was required.

## Criterios de salida que avanza

- Adds six catalog-ready templates; the wave 6 goal is at least twenty usable
  prebuilt project templates across all roles.
- Keeps every default build offline-capable and free of mandatory external
  services, per the wave 6 contract.
- Preserves the dependency rules: templates depend on `ag-*` crates only, no
  shared CLI file and no other role's directory was modified.

## Cierre de issues

Closes: none

## Checklist

- [x] Resumen has 256 characters or fewer
- [x] Test plan contains reproducible commands
- [x] Verification results are recorded without invented claims
- [x] Documentation updated with the code (CHANGELOG and template READMEs)
- [x] No push or merge performed
