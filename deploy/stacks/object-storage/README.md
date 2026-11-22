# Local S3-compatible object storage

An S3-compatible endpoint for `ag-storage` S3Backend and the object-storage
template family. The server is S3Mock, an Apache-2.0 testing tool, not
production storage. Objects persist in a named volume.

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 9090 | s3mock | S3 HTTP API |

Override with `S3_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/object-storage
cp .env.example .env
docker compose up -d
```

`S3_BUCKET` (default `ag-local`) is created at startup. Verify with:

```sh
curl -X PUT http://127.0.0.1:9090/ag-local/probe --upload-file /etc/hostname
curl http://127.0.0.1:9090/ag-local/probe
```

## Point a generated project at the stack

`ag-storage` reads these variables for the S3 backend:

```sh
export STORAGE_BACKEND='s3'
export STORAGE_ENDPOINT='http://127.0.0.1:9090'
export STORAGE_BUCKET='ag-local'
export STORAGE_REGION='us-east-1'
export AWS_ACCESS_KEY_ID='local'         # placeholder, not validated
export AWS_SECRET_ACCESS_KEY='local-only'
```

`ag-storage` needs its `s3` Cargo feature enabled for `STORAGE_BACKEND=s3`;
without it the crate warns and uses the Native backend. The examples also
accept `MINIO_URL` and `MINIO_BUCKET` as endpoint aliases; point `MINIO_URL` at
the same `http://127.0.0.1:9090` value.

## Health and persistence

The healthcheck probes `/favicon.ico`. The named volume `s3data` is mounted at
`/s3mockroot`, the writable store root for the image's non-root user.

## Limits and security

S3Mock supports path-style access only (`/bucket/key`, not virtual-hosted
`bucket.host`), accepts but does not validate presigned URLs, and is explicitly
not for production. Keep port 9090 on localhost. No real cloud credentials are
committed; the values in `.env.example` are placeholders.
