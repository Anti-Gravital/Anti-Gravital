# Secret injection for the Kubernetes manifests

No secret value is stored in this repository. The base Deployment references a
Secret by name only:

```yaml
envFrom:
  - configMapRef:
      name: template-config
  - secretRef:
      name: template-secrets
      optional: true
```

`template-secrets` is deliberately **not defined** by any manifest here. The
reference is marked `optional: true` so a workload can still start with only
non-secret configuration (useful for local development and smoke tests). A
production deployment must create that Secret in namespace `anti-gravital`
before or together with the workload. The name is intentionally the same for
every family overlay; there is no `namePrefix` in these overlays.

Only non-secret values (`BIND`, `RUST_LOG`) live in `base/configmap.yaml`.

## Pattern A: external secret manager (recommended)

Install the External Secrets Operator in the cluster and have an administrator
create a `ClusterSecretStore` pointing at the organization's secret manager.
The store contains credentials; the manifest below contains only references.

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: template-secrets
  namespace: anti-gravital
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: cluster-secret-store
    kind: ClusterSecretStore
  target:
    name: template-secrets
    creationPolicy: Owner
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: anti-gravital/template
        property: database_url
```

Add one `data` entry per key the application reads. `DATABASE_URL` is shown as
an example because the `fullstack` template reads it; it is not required by the
`rest` or `realtime` templates.

## Pattern B: sealed secrets

With a sealed-secrets controller installed in the cluster, encrypt values
locally with the controller's public key and commit only the ciphertext:

```sh
kubectl create secret generic template-secrets \
  --namespace anti-gravital \
  --from-env-file=.env.local \
  --dry-run=client -o yaml | kubeseal -o yaml > sealed-secret.yaml
```

The resulting object uses the controller's API version (check the version
installed in the cluster) and contains ciphertext, never plaintext:

```yaml
apiVersion: <sealed-secrets-controller-api-version>
kind: SealedSecret
metadata:
  name: template-secrets
  namespace: anti-gravital
spec:
  encryptedData:
    DATABASE_URL: <ciphertext-produced-by-kubeseal>
```

Keep `.env.local` out of version control. The `<...>` tokens above are
placeholders, not usable values.

## Pattern C: manual creation (development only)

For a local cluster, create the Secret from a file that is never committed:

```sh
kubectl -n anti-gravital create secret generic template-secrets \
  --from-env-file=.env.local
```

This does not provide rotation or auditability. It is acceptable for a local
kind/minikube cluster, not for shared environments.

## Rotation and scope

- Rotate the source value in the secret manager or the sealed-secret file, then
  restart the workload so the environment is refreshed:
  `kubectl -n anti-gravital rollout restart deployment/template`.
- Grant the workload only the keys it reads. Do not share one Secret across
  unrelated applications in the same namespace if their key sets differ.
- Never use `kubectl describe secret` output in logs or issue reports: it can
  expose referenced keys even when values are redacted.
