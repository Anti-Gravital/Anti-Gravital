# Domain-link example (offline)

This example shows the complete project-to-URL link with no external
services: a canonical domain manifest for dev, staging and prod, a
dependency-free local service, and a mock DNS/hosts check that runs entirely
offline. It is a demonstration of `deploy/link/`, not a new template and not
a live deployment. The manifest follows
`docs/dns/ENVIRONMENT_RULES.md` (ER-01..ER-11).

## Files

| File | Purpose |
| --- | --- |
| `domain.yaml` | Reference manifest, `schema_version: 1`, three deployable environments. |
| `Cargo.toml` | Standalone zero-dependency crate (own workspace root). |
| `src/main.rs` | Tiny HTTP service that echoes the request Host header and `APP_ENV`. |
| `mock-hosts.sh` | Renders all three overlays, checks custom hosts offline and verifies ER-08. |

Hosts use `*.invalid` (ER-10c), so they are guaranteed not to resolve
publicly. The `port` field is a Service port; the mock maps each environment
to a separate loopback port (8080, 8081, 8082) inside `mock-hosts.sh`, not in
the manifest, because ER-01 rejects unknown keys such as `localPort`.

The `api` service demonstrates the policy ladder: `vpn-only` in dev/staging,
`public` in prod. A second `metrics` service is `private` and must never get
an Ingress rule (ER-08).

## Run everything offline

From the repository root:

```sh
bash examples/domain-link/mock-hosts.sh
```

Expected summary (host echoes include the port from the Host header):

```
PASS manifest exists
PASS manifest parse (3 deployable environments)
PASS cargo build --offline
PASS render overlay dev (vpn-only) -- api.dev.example.invalid
PASS curl --resolve dev -- ok host=api.dev.example.invalid:8080 path=/health env=dev
PASS render overlay staging (vpn-only) -- api.staging.example.invalid
PASS curl --resolve staging -- ok host=api.staging.example.invalid:8081 path=/health env=staging
PASS render overlay prod (public) -- api.example.invalid
PASS curl --resolve prod -- ok host=api.example.invalid:8082 path=/health env=prod
PASS private service has no Ingress (ER-08) -- prod/metrics
PASS rendered private overlay has no Ingress (ER-08)
PASS=11 FAIL=0 SKIP=0
```

The script renders each environment into `/tmp/ag-domain-link-example/<env>`
(override with `RENDER_OUT`), prints the `/etc/hosts` line you would add on a
real dev machine, and starts one mock server per environment. `curl --resolve`
is a per-request DNS override: it proves the custom hostname reaches the
right service without editing `/etc/hosts` and without root privileges.

TLS in the manifest (all three deployable environments) is cluster
configuration: the mock serves plain HTTP locally on purpose. The rendered
`rendered.yaml` from `deploy/link/link.sh` is where `tls:` and `secretName:`
appear.

## Run one environment manually

```sh
cd examples/domain-link
BIND=127.0.0.1:8080 APP_ENV=dev cargo run --offline --quiet
```

In another shell:

```sh
curl --resolve api.dev.example.invalid:8080:127.0.0.1 \
  http://api.dev.example.invalid:8080/health
# ok host=api.dev.example.invalid:8080 path=/health env=dev
```

For a persistent hostname on your own machine (requires admin):

```
127.0.0.1 api.dev.example.invalid
127.0.0.1 api.staging.example.invalid
127.0.0.1 api.example.invalid
```

This is shown, never applied: the example does not modify `/etc/hosts`.

## Kubernetes side of the same manifest

```sh
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env prod --service api \
  --out /tmp/ag-domain-link/prod
grep -E 'host:|tls:|secretName:' /tmp/ag-domain-link/prod/rendered.yaml

# A private service produces no Ingress rule:
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env prod --service metrics \
  --out /tmp/ag-domain-link/prod-metrics
```

Rendering and parsing are offline. Applying to a real cluster, provisioning a
real DNS record and creating the TLS Secret are separate explicit steps that
this example does not perform. See `deploy/link/README.md` for the full
local/dev/staging/prod commands and `docs/audits/domain-link-validation.md`
for the recorded evidence.
