# Kubernetes packaging (optional)

Static Kubernetes manifests for the Anti-Gravital template families. They are
an **optional deployment target**, not a requirement: the default way to build
and run a project is still `ag new`, `cargo run` and `ag dev` on the host. This
directory is only read when a user explicitly chooses Kubernetes, in keeping
with the repository rule to integrate with dominant tools rather than replace
them.

No cluster deployment was performed while producing these manifests. The
verification performed was offline rendering and YAML parsing only; see
[`EVIDENCE.md`](EVIDENCE.md).

## Layout

```
deploy/k8s/
|-- kustomization.yaml          root: renders the family-agnostic base
|-- base/                       namespace, ConfigMap, Deployment, Service
|-- overlays/
|   |-- web/                    web family (static, marketing, blog, docs, ...)
|   |-- api/                    API family (rest, fullstack, auth, ...)
|   |-- realtime/               realtime family (ws, chat, workers, ...)
|   |-- infrastructure/         mail, DNS, monitoring, object storage, ...
|   `-- ai/                     ai backend, vector search, rag, ...
|-- optional/
|   |-- hpa/                    HPA component, not enabled by any overlay
|   `-- ingress/                Ingress component, not enabled by any overlay
|-- examples/
|   `-- api-with-addons/        example that enables both optional components
|-- SECRETS.md                  external-secret / sealed-secret patterns
`-- EVIDENCE.md                 exact verification commands and outputs
```

## What the base provides

- Namespace `anti-gravital`, Service `template`, ConfigMap `template-config`.
- A Deployment with requests/limits, TCP liveness and readiness probes on the
  named port `http` (8080), and `BIND=0.0.0.0:8080` in the ConfigMap. Generated
  templates default to `127.0.0.1:8080`, which a Service cannot reach.
- Hardening: `runAsNonRoot`, uid/gid/fsGroup `10001`, `RuntimeDefault`
  seccomp, `allowPrivilegeEscalation: false`, all capabilities dropped,
  `readOnlyRootFilesystem: true` with an `emptyDir` mounted at `/tmp`, and
  `automountServiceAccountToken: false`.
- Secret injection by name only (`template-secrets`, `optional: true`). No
  secret values are inlined. Read `SECRETS.md` before applying to a real
  cluster.

TCP probes are used because only the `fullstack` and `layered` templates
currently expose an HTTP `/health` route; `rest` and `realtime` do not, and no
template exposes `/ready`. A user whose application does expose `/health` may
replace the liveness probe with an `httpGet` path in their own copy of an
overlay. Do not assume an endpoint that the template does not serve.

## Prerequisites

- `kubectl` and/or `kustomize` for rendering and applying.
- A container image built from the chosen template and pushed to a registry
  the cluster can pull. The image names in the overlays are placeholders on
  `registry.example.invalid` and **must** be replaced before applying.
- A cluster, only for `apply`. Rendering (`kustomize build`) needs no cluster.
- Optional: a metrics server for the HPA; an ingress controller for Ingress;
  a secret controller or external secret store for production secrets.

## Build (offline, no cluster)

```sh
kustomize build deploy/k8s
kustomize build deploy/k8s/overlays/web
kustomize build deploy/k8s/overlays/api
kustomize build deploy/k8s/overlays/realtime
kustomize build deploy/k8s/overlays/infrastructure
kustomize build deploy/k8s/overlays/ai
kustomize build deploy/k8s/examples/api-with-addons
```

`kubectl kustomize <dir>` produces the same output as `kustomize build <dir>`.

## Configure the image

The overlays set a placeholder repository and tag. Replace them in your own
copy of an overlay, for example:

```sh
cp -r deploy/k8s/overlays/api my-k8s/api
cd my-k8s/api
kustomize edit set image registry.example.invalid/your-org/api-template=ghcr.io/acme/my-api:1.0.0
kustomize build .
```

If the image is private, add an `imagePullSecrets` entry to the pod spec in
your copy and create the corresponding Secret in the cluster. Do not commit
registry credentials.

## Apply and delete

One family is applied per namespace. The overlays intentionally do not use
`namePrefix`, so that the optional HPA and Ingress components can reference the
unprefixed `template` workload. To run two families in the same cluster, apply
each to a different namespace by overriding `namespace` in a copied overlay.

```sh
# Create the Secret first (see SECRETS.md), then render and apply a family.
kustomize build deploy/k8s/overlays/api | kubectl apply -f -
kubectl -n anti-gravital rollout status deployment/template

# Inspect
kubectl -n anti-gravital get deployment,service,pods

# Delete only the objects from that overlay
kustomize build deploy/k8s/overlays/api | kubectl delete -f -
```

Replace `api` with `web`, `realtime`, `infrastructure` or `ai` as needed.
`kubectl delete -f -` on a rendered manifest only removes the objects present
in that render; it does not delete the Secret created separately.

## Optional HPA and Ingress

Both are Kustomize `Component`s and are **disabled by default**: no family
overlay lists them. Enable them by adding a `components` block to a copied
overlay, or build the provided example:

```sh
kustomize build deploy/k8s/examples/api-with-addons | kubectl apply -f -
```

The example binds the HPA to `deployment/template` and the Ingress to
`service/template`. Before applying it, set a real `host` in
`optional/ingress/ingress.yaml` (the default is `template.example.invalid`),
choose an `ingressClassName` for the installed controller, and add TLS
configuration if the controller requires it. The HPA needs a metrics server;
without one it will render but will not scale.

## Limitations

- These are starting points, not a production platform. Do not apply them
  unchanged: replace the image, the host name and the namespace as needed.
- No PersistentVolumeClaim is included. Stateful templates such as the
  `fullstack` template use an external database through `DATABASE_URL`; the
  Secret pattern above covers that value.
- No NetworkPolicy, PodDisruptionBudget or topology spread is included. Add
  them in an overlay if the cluster policy requires them.
- No health endpoint is invented for templates that do not expose one.
- `kubectl apply --prune` is not used; deletion is explicit via the rendered
  manifest.

## Related documents

- `deploy/docker/` is the peer packaging path for containers without a cluster.
- `docs/INDEX.md` links the deployment documentation from the master index.
- `docs/rfc/RFC-0021-*` describes the plan/apply model used by the CLI; none of
  that CLI machinery is required by these static manifests.
