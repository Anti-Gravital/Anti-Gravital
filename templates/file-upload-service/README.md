# {{name}} - file upload service

A scoped upload API built on `ag-storage`. Instead of exposing raw object
paths, it issues short-lived signed capabilities bound to a key prefix and a
scope. Object requests must present that capability and stay inside its
prefix.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP and security | `ag-core` (`Shield`), axum 0.7 |
| Object store | `ag-storage` native backend (filesystem, `cap-std` confined) |
| Capability token | `ag_storage::signed::sign_url` / `verify_signed_url` (HMAC-SHA256) |
| Confinement | prefix allow-list plus key validation inside ag-storage |

## Two layers of confinement

1. **Store layer.** `AgStorage` roots every key in `STORAGE_ROOT`. Keys are
   validated against traversal and symlink escapes, and writes use a
   capability-relative directory handle (`cap-std`). The process cannot read
   outside the root even if a key contains `../`.
2. **Application layer.** A capability signs `prefix|scope` with an expiry.
   Handlers verify the signature and reject keys that do not start with the
   granted prefix, or scopes that do not include the needed operation.

A leaked capability therefore grants access only to one subtree, only for the
operations it names, and only until it expires.

## Run

```bash
export STORAGE_SIGN_SECRET='a-long-random-value'
cargo run
```

Issue a capability, then use it:

```bash
CAP=$(curl -s -X POST http://127.0.0.1:8080/capabilities \
  -H 'content-type: application/json' \
  -d '{"prefix":"tenants/acme/","scope":"rw","ttl_secs":3600}')
TOKEN=$(printf '%s' "$CAP" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')

curl -i -X PUT http://127.0.0.1:8080/objects/tenants/acme/hello.txt \
  -H 'X-Capability-Prefix: tenants/acme/' \
  -H 'X-Capability-Scope: rw' \
  -H "X-Capability-Token: $TOKEN" \
  --data-binary 'hello world'

curl http://127.0.0.1:8080/objects/tenants/acme/hello.txt \
  -H 'X-Capability-Prefix: tenants/acme/' \
  -H 'X-Capability-Scope: rw' \
  -H "X-Capability-Token: $TOKEN"
```

Trying `tenants/other/...` with the same capability returns 401.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| POST | `/capabilities` | Mint a signed prefix/scope token with expiry |
| PUT | `/objects/*key` | Store raw bytes under an authorized key |
| GET | `/objects/*key` | Read an authorized object |
| GET | `/health` | Liveness and backend identifier |

## Verification tests

```bash
cargo test --offline
```

Tests cover prefix and scope validation, including traversal attempts.

## Limitations

- No authentication: whoever holds a capability can use it. Bind capabilities
  to a verified identity (`ag-auth`) before offering them to untrusted clients.
- Capabilities are bearer tokens with no revocation short of rotating
  `STORAGE_SIGN_SECRET`.
- No multipart parsing: uploads are raw bodies. Use `Content-Type` and a
  separate metadata endpoint for filenames and MIME types.
- The in-process `AgStorage` is not shared across replicas unless all replicas
  mount the same `STORAGE_ROOT`; use the S3 adapter behind its Cargo feature
  for shared storage.
- Maximum TTL is capped at 24 hours.
