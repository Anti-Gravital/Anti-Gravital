# How-to — serve HTTPS for an attached domain and run the REST API

This covers RFC-0009 phases B and C: running the edge listeners (`ag-edge`) and
the control-plane REST API (`ag-domains`). Both default to native operation; no
external service is required.

## Run the edge HTTP listener (ACME HTTP-01 + routing)

Enable the `server` feature of `ag-edge`.

```rust,ignore
use ag_edge::challenge::Http01ChallengeStore;
use ag_edge::router::{BindingCache, RouteBinding};
use ag_edge::server::{serve_http, EdgeState};

# async fn run() -> std::io::Result<()> {
let mut bindings = BindingCache::new();
bindings.insert(RouteBinding {
    hostname_ascii: "api.example.com".into(),
    project: "site".into(),
    environment: "production".into(),
    target_ref: "svc_api".into(),
});

let challenges = Http01ChallengeStore::new();
// The ACME issuance flow registers (token, key_authorization) here.
// challenges.set(token, key_authorization);

let state = EdgeState::new(bindings, challenges);
let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await?;
serve_http(listener, state).await
# }
```

The listener serves `/.well-known/acme-challenge/<token>` from the challenge
store, applies any canonical/redirect policy, and routes by `Host`/`:authority`.
Unknown custom hostnames return 404 (fail closed).

## Serve HTTPS with SNI certificate selection

Enable the `tls` feature. Feed the PEM produced by the `ag-domains` ACME flow
(`IssuedCert { cert_chain_pem, private_key_pem }`) into the certificate store.

```rust,ignore
use std::sync::Arc;
use ag_edge::cert::{server_config, CertStore};
use ag_edge::server::serve_https;

# async fn run(state: ag_edge::server::EdgeState, cert_chain_pem: &str, key_pem: &str)
#   -> std::io::Result<()> {
let mut store = CertStore::new();
store.insert_pem("api.example.com", cert_chain_pem, key_pem).unwrap();
let tls = server_config(Arc::new(store)).unwrap();

let listener = tokio::net::TcpListener::bind("0.0.0.0:443").await?;
serve_https(listener, state, tls).await
# }
```

The server presents the certificate whose hostname matches the TLS SNI (exact,
then single-label wildcard).

## Run the control-plane REST API

Enable the `api` feature of `ag-domains`.

```rust,ignore
use std::sync::{Arc, Mutex};
use ag_domains::api::{serve, ApiState};
use ag_domains::instructions::EdgeTargets;
use ag_domains::store::{AttachmentStore, JsonFileStore};

# async fn run() -> std::io::Result<()> {
let store: Arc<Mutex<dyn AttachmentStore + Send>> =
    Arc::new(Mutex::new(JsonFileStore::open(".ag/domains.json").unwrap()));
let edge = EdgeTargets::new("edge.example-cloud.net");
let state = ApiState::new(store, edge);

let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
serve(listener, state).await
# }
```

Then:

```bash
curl -X POST localhost:8080/v1/domains/attachments \
  -H 'content-type: application/json' \
  -d '{"hostname":"api.example.com","project_id":"site","target_ref":"svc_api"}'

curl localhost:8080/v1/domains/attachments/<id>/instructions
```

The full contract is in `openapi/ag-domains.v1.yaml`.
