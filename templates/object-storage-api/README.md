# object-storage-api

An object storage API scaffold built on
[ag-storage](../../../crates/ag-storage). It stores objects through the native
filesystem backend and issues short-lived HMAC signed URLs with
`AgStorage::signed_url`.

## Files

```
Cargo.toml      package manifest (ag-storage pinned by rev)
config.toml     storage and signing settings
src/main.rs     composition root: object routes and signed URLs
template.toml   catalog metadata
```

## Run

```sh
ag new mystore --template object-storage-api
cd mystore
ag dev
curl http://127.0.0.1:8080/health
printf 'hello' | curl -X PUT --data-binary @- http://127.0.0.1:8080/objects/notes/hello.txt
curl http://127.0.0.1:8080/objects/notes/hello.txt
export STORAGE_SIGN_SECRET='replace-with-a-deployment-secret'
curl 'http://127.0.0.1:8080/sign/notes/hello.txt?ttl=300'
```

Without the CLI, replace `{{name}}` in the source files and run `cargo run`.
`BIND` overrides the listen address (default `127.0.0.1:8080`).

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness probe. |
| `PUT` | `/objects/*key` | Store the request body under `key`. |
| `GET` | `/objects/*key` | Read an object. |
| `GET` | `/sign/*key?ttl=300` | Issue a signed-URL token (1 to 86400 seconds). |
| `POST` | `/verify` | Verify `{"key":..., "token":...}`. |

## External requirements

- `S3_BUCKET` (catalog requirement): the production bucket when ag-storage's
  `s3` feature is enabled and the backend is switched to S3/MinIO. It is not
  needed to build or run this scaffold.
- `STORAGE_SIGN_SECRET`: HMAC secret for signed URLs. When unset, `/sign`
  returns 503 and no signing key is bundled. Never commit a real secret.
- `STORAGE_ROOT` (optional): local store directory, default `./ag-store-data`.

## Limitations

- The default backend is local filesystem, not durable shared storage. S3
  support requires the `s3` feature and deployment credentials.
- The embedded ag-storage HTTP server is disabled; this service owns its own
  axum listener.
- Signed URL tokens grant read access until expiry; protect the signing secret.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
