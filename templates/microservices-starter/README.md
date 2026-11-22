# microservices-starter

A two-service scaffold: a gateway that accepts jobs and a worker that processes
them. Both are built from one binary and selected with `SERVICE_ROLE`, and both
compile the same typed wire contract from `src/contract.rs`.

## Files

```
Cargo.toml          package manifest
config.toml         role and bind settings
src/main.rs         role selection and the two HTTP surfaces
src/contract.rs     shared request/response types
template.toml       catalog metadata
```

## Run

Start each role in its own terminal (or container):

```sh
ag new myservices --template microservices-starter
cd myservices
SERVICE_ROLE=gateway cargo run   # listens on 127.0.0.1:8081
SERVICE_ROLE=worker cargo run    # listens on 127.0.0.1:8082
```

```sh
curl http://127.0.0.1:8081/health
curl -X POST http://127.0.0.1:8081/enqueue \
  -H 'content-type: application/json' \
  -d '{"job_type":"resize","payload":{"width":128}}'
curl http://127.0.0.1:8082/health
curl -X POST http://127.0.0.1:8082/process \
  -H 'content-type: application/json' \
  -d '{"job_type":"resize","payload":{"width":128}}'
```

Without the CLI, replace `{{name}}` in the source files and run Cargo directly.
`GATEWAY_BIND` and `WORKER_BIND` override the per-role listen addresses.

## Shared contract

`src/contract.rs` defines `EnqueueRequest`, `EnqueueResponse`,
`HealthResponse` and `new_job_id`. It depends only on `serde` and `serde_json`,
so it can be extracted into a standalone crate shared by independently deployed
services. `SCHEMA_VERSION` is included in responses to make drift visible.

## External requirements

None. Both services run against loopback with in-memory state, so the template
builds and starts with no broker or database.

## Limitations

- The gateway does not forward to the worker over the network; the two roles
  are separate addressable services that share the contract. Add a queue or
  HTTP client for real dispatch.
- Job state is in-memory and lost on restart.
- One binary with role selection is a scaffold; production deployments should
  build and ship the roles as separate artifacts.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
