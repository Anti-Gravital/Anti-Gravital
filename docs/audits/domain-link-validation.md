# Domain-link validation evidence (wave 8)

This record covers the artifacts on branch `community8/deplink`. All results
below were produced by real commands in the worktree; no result is
reconstructed or assumed. Commands that need a live provider, a cluster, a
Docker daemon or root privileges are recorded as SKIP, never PASS.

## Context

| Item | Value |
| --- | --- |
| Date (UTC) | 2026-11-22 05:18 UTC |
| Host | Linux 7.0.0-44-generic x86_64 |
| Worktree | `/home/angel-nereira/Github-local-angelnereira/worktrees8/deplink` |
| Verified commit | `5c3bc48dfed78824d05e11ea629daae70249f923` (`feat(deploy): link generated artifacts to domains with offline evidence`) |
| Scope | `deploy/link/`, `examples/domain-link/`, `docs/INDEX.md`, `CHANGELOG.md`, branch descriptor |
| Contract | `docs/dns/ENVIRONMENT_RULES.md` (ER-01..ER-11), read from main via coordinator relay; envrules files not modified |

This evidence document is added in the commit following `5c3bc48`; all checks
were run against that commit's tree state plus the committed artifacts it
contains.

## Tool versions (measured)

| Tool | Version |
| --- | --- |
| `python3` | 3.14.4 |
| PyYAML | 6.0.3 |
| `kustomize` | v5.5.0 |
| `kubectl` (client only) | v1.37.1, embedded Kustomize v5.8.1 |
| `cargo` / `rustc` | 1.99.1 (5f94df478 2026-08-27) |
| Docker Compose | v5.5.1 |
| `curl` | 8.18.0 (x86_64-pc-linux-gnu) |
| `git` | worktree on branch `community8/deplink` |

## Checks and real results

### 1. Static checks

```
$ python3 -m py_compile deploy/link/render_ingress.py
$ bash -n deploy/link/link.sh
$ bash -n examples/domain-link/mock-hosts.sh
$ git diff --check
(exit 0 for all four; no output)
```
Result: **PASS (4/4)**. Generated `__pycache__` was removed before commit.

### 2. YAML parsing with a real parser

```
$ python3 - <<'PY'
from pathlib import Path
import yaml
roots = [Path("deploy/link"), Path("examples/domain-link")]
files = sorted(p for r in roots for p in r.rglob("*") if p.suffix in {".yaml", ".yml"})
for p in files:
    docs = list(yaml.safe_load_all(p.read_text(encoding="utf-8")))
    print(f"PARSE {p}: {len(docs)} doc(s)")
print(f"TOTAL={len(files)}")
PY
PARSE deploy/link/compose/local-host.yml: 1 doc(s)
PARSE examples/domain-link/domain.yaml: 1 doc(s)
TOTAL=2
```
Result: **PASS (2/2)**.

### 3. One-command render for every environment

```
$ bash deploy/link/link.sh --manifest examples/domain-link/domain.yaml \
    --env local --service api --out /tmp/ag-domain-link/local-api
ENV=local
VISIBILITY=private
TLS=false
INGRESS=false
URL=http://127.0.0.1:8080
RENDERED=/tmp/ag-domain-link/local-api/rendered.yaml
INGRESS_OK=true (no Ingress for private/local, ER-08/ER-02a)
HOST_OK=n/a
TLS_OK=n/a

$ bash deploy/link/link.sh ... --env dev --service api --out /tmp/ag-domain-link/dev-api
HOST=api.dev.example.invalid
VISIBILITY=vpn-only
TLS=true
INGRESS=true
URL=https://api.dev.example.invalid
HOST_OK=true
TLS_OK=true

$ bash deploy/link/link.sh ... --env staging --service api --out /tmp/ag-domain-link/staging-api
HOST=api.staging.example.invalid
VISIBILITY=vpn-only
TLS=true
INGRESS=true
URL=https://api.staging.example.invalid
HOST_OK=true
TLS_OK=true

$ bash deploy/link/link.sh ... --env prod --service api --out /tmp/ag-domain-link/prod-api
HOST=api.example.invalid
VISIBILITY=public
TLS=true
INGRESS=true
URL=https://api.example.invalid
HOST_OK=true
TLS_OK=true

$ bash deploy/link/link.sh ... --env prod --service metrics --out /tmp/ag-domain-link/prod-metrics
VISIBILITY=private
TLS=false
INGRESS=false
URL=
INGRESS_OK=true (no Ingress for private/local, ER-08/ER-02a)
```
Result: **PASS (5/5 plans)**. Every plan rendered with `kustomize build`; the
exposed plans were checked for the exact host and a `secretName:` entry, and
the private/local plans were checked to contain no `kind: Ingress`.

The generated JSON6902 patch for `prod/api`, as applied by kustomize, produced:

```
$ grep -nE 'host:|tls:|secretName:|anti-gravital.io/' \
    /tmp/ag-domain-link/prod-api/rendered.yaml
118:    anti-gravital.io/environment: prod
119:    anti-gravital.io/project: domain-link-demo
120:    anti-gravital.io/visibility: public
125:  - host: api.example.invalid
135:  tls:
138:    secretName: ag-tls-domain-link-demo-prod
```

### 4. Renderer parity

```
$ diff -q <(kustomize build /tmp/ag-domain-link/prod-api) \
          <(kubectl kustomize /tmp/ag-domain-link/prod-api)
(no output, exit 0)
$ kustomize build /tmp/ag-domain-link/prod-api | sha256sum
7566e71f8c89005c04f78a388a66fdbfde9eeed29cc8bbbb443c3f7f7bed49cc  -
$ kubectl kustomize /tmp/ag-domain-link/prod-api | sha256sum
7566e71f8c89005c04f78a388a66fdbfde9eeed29cc8bbbb443c3f7f7bed49cc  -
```
Result: **PASS**, byte-identical output.

### 5. Offline mock DNS/hosts flow

```
$ rm -rf /tmp/ag-domain-link-example
$ bash examples/domain-link/mock-hosts.sh
manifest: .../examples/domain-link/domain.yaml
output:   /tmp/ag-domain-link-example
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
Result: **PASS (11/11)**. `curl --resolve` supplied a per-request DNS
override; no `/etc/hosts` file was modified. The three responses prove that
the custom hostname reached the service whose `APP_ENV` matched the manifest
environment.

### 6. Compose custom-host override

```
$ API_PORT=127.0.0.1:8080 \
  PUBLIC_BASE_URL=http://api.dev.example.invalid:8080 \
  POSTGRES_PASSWORD=local-only-placeholder \
  docker compose -f deploy/docker/compose/api.yml \
                 -f deploy/link/compose/local-host.yml config | grep -nE ...
15:      AG_PUBLIC_HOST: api.dev.example.invalid
20:      PUBLIC_BASE_URL: http://api.dev.example.invalid:8080
35:        host_ip: 127.0.0.1
37:        published: "8080"
```
Result: **PASS**. Only the Compose client was used; no daemon, image or proxy
container was required, and the published port stayed on loopback.

### 7. Negative cases (fail-closed behavior)

Each manifest was a modified copy of `examples/domain-link/domain.yaml`, fed
to `python3 deploy/link/render_ingress.py --env prod --service api --out /tmp/...`:

| Mutation | Real stderr | Exit |
| --- | --- | --- |
| key `schemaVersion` instead of `schema_version` | `unknown key(s) ['schemaVersion'] (ER-01 strict schema)` | 2 |
| `visibility: unlisted` | `unknown visibility 'unlisted' (ER-04 hard error)` | 2 |
| `environments.local` added | `environment 'local' has no domain manifest entry (ER-02a)` | 2 |
| `host: api.other.invalid` | `host 'api.other.invalid' not in allowed_hosts (ER-10)` | 2 |
| private `metrics` given `allow_sources` (with `--service metrics`) | `private service must not declare allow_sources (ER-08)` | 2 |

Result: **PASS (5/5 rejected)**. Missing visibility resolves to the private
environment default (ER-04); a host never infers public visibility (ER-08).

### 8. Honest SKIP path

```
$ PATH=/usr/bin:/bin bash deploy/link/link.sh \
    --manifest examples/domain-link/domain.yaml --env dev --service api \
    --out /tmp/ag-domain-link/skip | tail -3
OVERLAY=/tmp/ag-domain-link/skip
K8S_ROOT=.../deploy/k8s
KUSTOMIZE=SKIP
```
Result: **PASS** (the wrapper correctly reports SKIP instead of inventing a
render). `kustomize` lives in `~/.local/bin`, outside the reduced PATH.

## Honest SKIPs

| Operation | Reason |
| --- | --- |
| Real DNS record creation | No provider credentials; out of scope and never attempted. |
| `kubectl apply` to a cluster | No cluster or API server configured. |
| `/etc/hosts` modification | Requires root; the example prints the line but never writes it. |
| `docker build` / `docker compose up` | No reachable daemon; only `docker compose config` was used. |
| Live TLS certificate issuance | No ACME or CA integration; only a Secret reference is rendered. |

## Contract alignment note

The first draft used `schemaVersion`, `production` and `tls.enabled`. After
the coordinator relayed the envrules contract, all artifacts were renamed to
`schema_version`, `prod` and `tls.required`, the full strict field set and
the `{private, vpn-only, public}` vocabulary were adopted, and non-canonical
fields such as `localPort` were removed. Verification above was re-run from
scratch after the rename. No compatibility shim exists: the linker rejects
the old names (see check 7). The precise ER-04 reading (missing -> private,
unknown value -> hard error) and ER-08 (no Ingress for private) are recorded
in `docs/pr-drafts/community8-deplink.md` and the board notes.
