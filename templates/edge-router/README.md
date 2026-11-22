# edge-router

An HTTP edge data plane built on `ag-edge`. It resolves incoming hostnames to
project targets, applies canonical-host redirects, and serves ACME HTTP-01
challenge paths. Configuration lives in `config.toml`.

## What it demonstrates

- Hostname routing: exact bindings first, then wildcard (`*.example.com`), then
  legacy/fallback. Unknown custom hostnames fail closed with 404, so one tenant
  can never resolve to another tenant's binding.
- Canonical redirects: `www.example.com` redirects to `example.com` with status
  308, preserving path and query.
- ACME HTTP-01: challenge paths are served from the challenge store. The example
  starts with an empty store; the control plane would populate tokens at runtime.
- Declarative config: bindings and redirect policies are parsed from `config.toml`
  with `toml` and `serde`.

## Run

```sh
cargo run
curl -H 'Host: api.example.com' localhost:8080/       # resolves to svc_api_prod
curl -H 'Host: foo.staging.example.com' localhost:8080/  # wildcard match
curl -i -H 'Host: www.example.com' localhost:8080/path?q=1  # 308 redirect
```

`BIND` is not used here; the listen address comes from `config.toml`. Set `CONFIG`
to load a different config file. `RUST_LOG` controls log verbosity.

## External services

- Required to build or run: none. All routing logic is native and in-process.
- Optional, production only: an `ag-domains` control plane (and a DNS provider
  adapter such as Cloudflare) to issue certificates and publish bindings. The
  data plane itself has no third-party runtime dependency.

## Limitations

- No TLS listener: this template serves plain HTTP. Enable the `tls` feature on
  `ag-edge` and provide certificates for SNI-based HTTPS.
- Upstream proxying is the deployment's responsibility. On a successful custom
  match `ag-edge` returns the target in `x-ag-*` headers rather than proxying.
- The challenge store starts empty; ACME issuance requires populating it from
  the control plane.
- Bindings are read once at startup; there is no hot reload.
