# admin-panel

An admin panel on [ag-core](../../../crates/ag-core) with a stub, fail-closed
authorization gate. The expected token comes from `AG_ADMIN_TOKEN`; no secret
is stored in source or configuration.

## Files

```
Cargo.toml.tmpl        package manifest (ag-core pinned by rev)
config.toml.tmpl       loopback Shield bind configuration
src/main.rs.tmpl       state, gate and routes
static/index.html.tmpl public landing page
static/admin.html.tmpl gated admin content
static/styles.css.tmpl embedded stylesheet
template.toml          catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Routes

| Path | Description |
| --- | --- |
| `/` | Public landing page with a link to admin |
| `/admin` | Gated: 503 if no token configured, 401 if header missing or wrong, 200 otherwise |
| `/styles.css` | Embedded stylesheet |
| `/health` | Plain-text health probe |

## Run

```sh
ag new backoffice --template admin-panel
cd backoffice
AG_ADMIN_TOKEN=dev-only-example ag dev
curl -i http://127.0.0.1:8080/admin
curl -i -H 'Authorization: Bearer dev-only-example' http://127.0.0.1:8080/admin
```

Without `AG_ADMIN_TOKEN` the admin route returns 503 (fail closed). The value
above is an illustrative local example, not a production credential.

## Verification

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo check --offline
```

Recorded evidence: `/tmp/opencode/agents/w6-web.md`.

## Limitations and security notice

- This gate is a STUB. It uses plain string equality (not constant-time), has
  no rate limiting, no session handling and no audit log.
- Replace it with the Shield `auth-jwt` layer or another vetted authentication
  mechanism before exposing real administrative operations.
- The token is read from the environment, never committed. Do not place a real
  credential in `config.toml` or in source.
- The admin data shown is static placeholder content.
