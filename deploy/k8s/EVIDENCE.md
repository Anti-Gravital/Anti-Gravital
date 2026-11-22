# Verification evidence for deploy/k8s

Date: 2026-11-21 (repository session date). All commands were run from the
repository root. No cluster was contacted and no object was applied anywhere.

## Tool versions (exact output)

```
$ kustomize version
v5.5.0

$ kubectl version --client
Client Version: v1.37.1
Kustomize Version: v5.8.1

$ python3 -c 'import yaml; print("PyYAML " + yaml.__version__)'
PyYAML 6.0.3

$ kubectl config current-context
error: current-context is not set
```

The last command is the reason no `kubectl apply` was attempted: there is no
current context, and the mission forbids claiming a deployment that did not
happen.

## Command 1: render every target with kustomize and parse with PyYAML

Command (Python 3, using `subprocess` and `yaml.safe_load_all`):

```python
import subprocess, yaml, sys
targets = ['.', 'overlays/web', 'overlays/api', 'overlays/realtime',
           'overlays/infrastructure', 'overlays/ai', 'optional/hpa',
           'optional/ingress', 'examples/api-with-addons']
for t in targets:
    p = subprocess.run(['kustomize','build','deploy/k8s/'+t],
                       capture_output=True, text=True)
    if p.returncode != 0:
        print('FAIL', t, p.stderr); sys.exit(1)
    docs = list(yaml.safe_load_all(p.stdout))
    kinds = [d['kind'] for d in docs if d]
    print(f'OK  kustomize build deploy/k8s/{t:26s} {len(docs)} docs: {", ".join(kinds)}')
print('PyYAML parse: all rendered documents OK')
```

Output:

```
OK  kustomize build deploy/k8s/.                          4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/overlays/web               4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/overlays/api               4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/overlays/realtime          4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/overlays/infrastructure    4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/overlays/ai                4 docs: Namespace, ConfigMap, Service, Deployment
OK  kustomize build deploy/k8s/optional/hpa               1 docs: HorizontalPodAutoscaler
OK  kustomize build deploy/k8s/optional/ingress           1 docs: Ingress
OK  kustomize build deploy/k8s/examples/api-with-addons   6 docs: Namespace, ConfigMap, Service, Deployment, HorizontalPodAutoscaler, Ingress
PyYAML parse: all rendered documents OK
```

## Command 2: assert hardening and wiring invariants

For each of the five families, the rendered Deployment was checked for
`runAsNonRoot: true`, `readOnlyRootFilesystem: true`,
`allowPrivilegeEscalation: false`, `capabilities.drop: [ALL]`, non-empty
resource requests and limits, TCP liveness and readiness probes, an optional
`secretRef`, `BIND=0.0.0.0:8080`, a Service selector equal to the family label,
and a placeholder image on `registry.example.invalid`. Output:

```
OK  invariants web: nonroot/readonly/limits/probes/secret-optional/BIND/label/image
OK  invariants api: nonroot/readonly/limits/probes/secret-optional/BIND/label/image
OK  invariants realtime: nonroot/readonly/limits/probes/secret-optional/BIND/label/image
OK  invariants infrastructure: nonroot/readonly/limits/probes/secret-optional/BIND/label/image
OK  invariants ai: nonroot/readonly/limits/probes/secret-optional/BIND/label/image
OK  example add-ons: HPA target and Ingress backend match workload names; namespaces set
```

The example check verified that `HorizontalPodAutoscaler.spec.scaleTargetRef.name`,
`Ingress` backend service name, Deployment name and Service name are all
`template`, and that every rendered object except the Namespace carries
`namespace: anti-gravital`.

## Command 3: kubectl kustomize parity

For each target, the byte output of `kustomize build <target>` was compared
with `kubectl kustomize <target>`:

```
OK   kustomize build == kubectl kustomize: deploy/k8s/.
OK   kustomize build == kubectl kustomize: deploy/k8s/overlays/web
OK   kustomize build == kubectl kustomize: deploy/k8s/overlays/api
OK   kustomize build == kubectl kustomize: deploy/k8s/overlays/realtime
OK   kustomize build == kubectl kustomize: deploy/k8s/overlays/infrastructure
OK   kustomize build == kubectl kustomize: deploy/k8s/overlays/ai
OK   kustomize build == kubectl kustomize: deploy/k8s/optional/hpa
OK   kustomize build == kubectl kustomize: deploy/k8s/optional/ingress
OK   kustomize build == kubectl kustomize: deploy/k8s/examples/api-with-addons
```

## Command 4: no inline secret values

```
$ grep -rniE '(^|[[:space:]])(password|token|api[_-]?key|secretKey|clientSecret)[[:space:]]*:' deploy/k8s --include='*.yaml' | grep -v 'automountServiceAccountToken'
OK: no inline credential values in YAML manifests
```

The only Secret-related line in the rendered manifests is `secretRef.name:
template-secrets` with `optional: true`; no Secret object with data is defined.

## SKIPs and non-claims

- `kubeconform`: SKIP, binary not installed (`command -v kubeconform` empty).
  No schema validation beyond Kustomize parsing and PyYAML parsing is claimed.
- `kubectl apply`, `rollout status`, `kubectl delete`: NOT RUN. There is no
  current kubeconfig context, and no cluster deployment is claimed.
- Container image builds and registry pushes: NOT RUN. The images referenced
  by the overlays are placeholders that users must replace.
- `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny`: NOT
  RUN. This change adds no Rust code; those gates belong to code changes.
- External Secrets Operator, sealed-secrets controller, metrics server and
  ingress controller: not installed or tested. The manifests for them are
  documentation patterns only.
