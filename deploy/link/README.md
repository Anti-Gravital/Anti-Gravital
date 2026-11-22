# Domain link for generated deploy artifacts

This directory makes the last step of project scaffolding concrete: a
generated service, a deploy artifact and a domain manifest produce a URL you
can reach. It is an **optional** path. The default way to develop is still
`ag new`, `cargo run` and `ag dev` on the host; Kubernetes, Docker and DNS
providers are integration targets, never requirements (CLAUDE.md rule 12,
ADR-0009 rule 2).

Nothing here contacts a provider, a cluster, a registry or a DNS server. The
linker writes local files and renders manifests offline. A live domain, a
TLS certificate and a cluster are only needed if you explicitly choose to
apply the rendered manifests.

The manifest vocabulary is not defined here. It is the normative contract in
`docs/dns/ENVIRONMENT_RULES.md` (rules ER-01..ER-11), shared with the `dnsapi`
and `envrules` roles. The machine-readable reference is
`docs/dns/examples/domain-manifest.yaml`; this document only describes how the
deploy artifacts consume that data.

## Layout

```
deploy/link/
  README.md                 this document
  render_ingress.py         manifest -> generated Kustomize overlay + JSON6902 patch
  link.sh                   one-command wrapper (render, build, print URL)
  compose/local-host.yml    optional Compose override for local custom-host access
```

The runnable offline example lives in `examples/domain-link/`.

## Manifest fields the linker reads

The linker accepts the strict `schema_version: 1` shape from
`docs/dns/ENVIRONMENT_RULES.md`. Unknown top-level or environment keys are
rejected (ER-01). Relevant fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Must be `1`. |
| `project` | Non-empty project name; written as an Ingress annotation. |
| `default_visibility` | Must be `private` (ER-03). |
| `environments.<env>.visibility_default` | Must be `private` (ER-03). |
| `environments.<env>.allowed_hosts` | Host allowlist; exact or single-label wildcard (ER-09, ER-10). |
| `environments.<env>.allow_wildcard` | Enables `*.zone` matching; never implied in `prod`. |
| `environments.<env>.tls.required` | Required when any service binds a host (ER-TLS). |
| `environments.<env>.tls.termination` | `none`, `internal` or `edge`. |
| `environments.<env>.ingress` | Whether the environment permits Ingress rules. |
| `environments.<env>.services[].name` | Service selected with `--service` (default `api`). |
| `environments.<env>.services[].host` | Bound DNS name; must pass the allowlist. |
| `environments.<env>.services[].visibility` | `private`, `vpn-only` or `public`. |
| `environments.<env>.services[].port` | Service port; not a public URL port. |
| `environments.<env>.services[].allow_sources` | CIDRs; required for `vpn-only`/`public`, forbidden for `private`. |

Environment keys are exactly `dev`, `staging` and `prod` (plus the `local`
plan described below). `local` must not appear in the manifest (ER-02a).

### Visibility decides exposure, not the host

The linker derives every exposure decision from the explicit `visibility`
field, never from the shape or presence of a host (ER-08):

| Visibility | Ingress rule | Host/TLS patch | `allow_sources` |
| --- | --- | --- | --- |
| `private` | None | None, even if a host is present | Must be absent |
| `vpn-only` | Yes | Host + TLS; source CIDRs in annotations | Non-empty, no internet CIDR |
| `public` | Yes | Host + TLS | Must include `0.0.0.0/0` and/or `::/0` |

A service without `visibility` inherits the environment default, which is
always `private` (ER-04). An unknown visibility value is a hard validation
error, never a downgrade. For `vpn-only`, the source CIDRs are written to an
`anti-gravital.io/allow-sources` annotation for a downstream policy tool; the
annotation is metadata and does not itself enforce network access. The chosen
ingress controller or a NetworkPolicy must enforce it.

## What the linker generates

`render_ingress.py` writes one directory:

- `kustomization.yaml`: references `deploy/k8s/overlays/api` and, only for an
  exposed service, the `deploy/k8s/optional/ingress` Component (disabled by
  default in wave 7), then applies a JSON6902 patch targeting
  `Ingress/template` in namespace `anti-gravital`.
- The patch replaces `/spec/rules/0/host`, adds annotations for environment,
  visibility, project and (for `vpn-only`) allowed sources, and adds
  `/spec/tls` when TLS is required with non-`none` termination.

JSON6902 is used instead of a strategic-merge patch so the result does not
depend on Ingress list merge keys or on the ingress controller version. The
generated overlay references `deploy/k8s` by a computed relative path; if you
move it, regenerate it.

TLS configuration does not create a certificate. The Ingress references a
Secret named `ag-tls-<project>-<env>` by default; override it with
`--tls-secret`. The operator must provision that Secret in the target
namespace through the `ag-domains` adapters. No key material appears in the
manifest, compose file or Kubernetes manifest (ER-SECRET).

The renderer prints `KEY=value` lines (`URL`, `HOST`, `VISIBILITY`, `TLS`,
`INGRESS`, `OVERLAY`, ...) for scripts. For a private service it prints
`INGRESS=false`, `HOST=` and `URL=`: there is no DNS surface (ER-08).

## Local: loopback URL, no cluster and no provider

`local` is a loopback-only plan, not a manifest entry (ER-02a, ER-11):

```sh
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env local \
  --out /tmp/ag-domain-link/local
# URL=http://127.0.0.1:8080 ; INGRESS=false ; no Ingress in rendered.yaml
```

Start the dependency-free mock service from the example:

```sh
cd examples/domain-link
BIND=127.0.0.1:8080 cargo run --offline --quiet
```

In another shell, reach it under a custom hostname. Pick one of these:

```sh
# No admin rights, no /etc/hosts change: per-request DNS override.
curl --resolve api.dev.example.invalid:8080:127.0.0.1 \
  http://api.dev.example.invalid:8080/health

# Persistent hostname on the dev machine (requires admin, do not automate):
#   echo '127.0.0.1 api.dev.example.invalid' | sudo tee -a /etc/hosts
curl http://api.dev.example.invalid:8080/health
```

The whole three-environment flow is scripted:

```sh
bash examples/domain-link/mock-hosts.sh
```

### Local with Compose (optional)

Keep the published port on loopback and pass the custom base URL to the app.
`API_PORT` uses the `host_ip:hostPort` form so the port is not exposed on all
interfaces:

```sh
API_PORT=127.0.0.1:8080 \
PUBLIC_BASE_URL=http://api.dev.example.invalid:8080 \
POSTGRES_PASSWORD=local-only-placeholder \
docker compose -f deploy/docker/compose/api.yml \
               -f deploy/link/compose/local-host.yml config

curl --resolve api.dev.example.invalid:8080:127.0.0.1 \
  http://api.dev.example.invalid:8080/health
```

`config` needs only the Compose client, not a daemon. Replacing it with `up`
needs a running daemon, which is not assumed here. No proxy image is pulled.
If you need a real vhost on port 80 or local automatic certificates, add a
local reverse proxy container (for example Caddy or nginx) in your own
compose file, route the Host header to `api:8080`, and publish only
`127.0.0.1:80`. That is an optional local tool, not part of this contract.

## Staging and production: rendered manifests and URL

The same manifest drives the Kubernetes path. Rendering is offline:

```sh
# dev (private zone, internal TLS termination, vpn-only service)
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env dev --service api \
  --out /tmp/ag-domain-link/dev

# staging (edge TLS, vpn-only service)
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env staging --service api \
  --out /tmp/ag-domain-link/staging

# prod (exact host, edge TLS with HSTS, public service)
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env prod --service api \
  --out /tmp/ag-domain-link/prod

# A private service produces no Ingress (ER-08):
bash deploy/link/link.sh \
  --manifest examples/domain-link/domain.yaml \
  --env prod --service metrics \
  --out /tmp/ag-domain-link/prod-metrics
```

Each command prints the resolved `URL` and writes `kustomization.yaml`,
`rendered.yaml` and the generated patch into the output directory. Inspect an
exposed environment with `grep -E 'host:|tls:|secretName:' <out>/rendered.yaml`.

If kustomize is installed, `link.sh` renders and verifies the host/TLS values
(`HOST_OK`, `TLS_OK`, `INGRESS_OK`). If not, it prints `KUSTOMIZE=SKIP`;
`kustomize build` is never faked. You can also run the generator and renderer
directly:

```sh
python3 deploy/link/render_ingress.py \
  --manifest examples/domain-link/domain.yaml \
  --env prod --service api \
  --out /tmp/ag-domain-link/prod
kustomize build /tmp/ag-domain-link/prod
```

Applying to a real cluster is a separate, explicit step that needs an image,
an ingress controller, a DNS record and (for bound names) a TLS Secret. It is
not done by default and is not claimed as verified here:

```sh
# Only on a real cluster, after replacing the placeholder image and creating
# the TLS Secret named by --tls-secret (default ag-tls-<project>-<env>):
kustomize build /tmp/ag-domain-link/prod | kubectl apply -f -
kubectl -n anti-gravital get ingress template
```

`visibility: vpn-only` and `visibility: public` are explicit policy, never
inferred from a host (ER-08). Enforcing the vpn-only source ranges is the job
of the ingress controller or a NetworkPolicy, not of this renderer. Changing
a `prod` service to `public` is a security change requiring review (ER-11b).

## Verification and evidence

`docs/audits/domain-link-validation.md` records a real run: YAML parsing of
every new file, `kustomize build` for all rendered plans, the offline mock
server and three `curl --resolve` checks, the private no-Ingress check and
honest SKIPs for live DNS, cluster apply, `/etc/hosts` editing and a Docker
daemon.

## Limitations

- The linker writes files only. It does not create DNS records, certificates,
  images, ingresses in a cluster or entries in `/etc/hosts`.
- The generated overlay reuses the API family overlay from wave 7. Other
  families can be linked by changing one `resources` line in a copied
  overlay; the manifest format is family-agnostic.
- Hosts must be lowercase DNS names with a dot and pass the environment
  allowlist. Wildcard hosts are never generated as Ingress rules.
- Documentation-only `*.invalid` hosts are for local mocking. Real projects
  must use names they control (ER-10c).
- No external proxy is required or installed by default; the optional local
  proxy alternative is documentation only.

## Related documents

- `docs/dns/ENVIRONMENT_RULES.md`: normative environment/visibility contract.
- `docs/dns/examples/domain-manifest.yaml`: authoritative manifest example.
- `examples/domain-link/README.md`: runnable offline example.
- `deploy/k8s/README.md`: base overlays and the Ingress Component.
- `deploy/docker/README.md`: optional container packaging.
