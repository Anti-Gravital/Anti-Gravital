# Capitulo 10. Subsistema de despliegue (ag-cloud)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 10
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [09-plugins-wasi.md](./09-plugins-wasi.md)
> Siguiente: [11-ai-knowledge-graph.md](./11-ai-knowledge-graph.md)

## 10. Deployment subsystem (`ag-cloud` + `ag-domains`)

One of the most important structural corrections derived from the critical analysis is that `ag-cloud` is not a competitor of Terraform or of Kubernetes. Its target range is the same one covered by Railway, Fly.io, Render, and Coolify: simplify the deployment of backend applications to typical environments without forcing the team to operate complete infrastructure. Since Phase 4.5 (`ADR-0007`), `ag-cloud` cooperates with `ag-domains` to resolve domain, TLS, and mail records within the `ag deploy` flow itself, without replacing the dominant providers (Let's Encrypt, Cloudflare) and without becoming a hosting panel.

### 10.1 Philosophy of `ag-cloud`

The typical operator of an Anti-Gravital project, especially in its first years of life, does not need or want to operate a Kubernetes cluster. They need to bring up their API on a VPS, connect it to a database, put it behind TLS, and forget about it. `ag-cloud` solves this case.

For more complex cases (multi-region deployments, high availability, centralized secret management, IAM policies, infrastructure shared between multiple applications), `ag-cloud` is not the correct tool and the project must declare it openly: use Terraform, Pulumi, or Helm.

### 10.2 The `deploy.ag` file

The deployment subsystem is controlled with a declarative `deploy.ag` file separate from the project schema:

```yaml
app:
  name: payments-api
  domain: api.example.com

runtime:
  replicas: 3
  port: 8080
  health_check: /health
  resources:
    cpu: 1
    memory: 512MB

database:
  type: postgres
  version: "16"
  size: 20GB
  backup_schedule: "daily"

cache:
  type: redis
  version: "7"
  size: 1GB

storage:
  type: s3
  bucket: payments-api-uploads

secrets:
  source: vault
  path: secret/payments-api

observability:
  metrics: prometheus
  traces: tempo
  logs: loki

deployment:
  target: docker-compose      # opciones: docker-compose, fly, railway, k8s
  strategy: rolling
  max_surge: 1
  max_unavailable: 0
```

### 10.3 Supported deployment targets

`ag-cloud` supports four deployment targets, each with a different level of abstraction.

The **docker-compose** target generates a complete `docker-compose.yml` with services, networks, volumes, healthchecks, secrets loaded from `.env` files or from a secret manager, a reverse proxy (Caddy by default) with automatic TLS via Let's Encrypt, and backup scripts for the database. It is the recommended target for self-hosting on a single VPS.

The **fly** target generates a `fly.toml` and runs the `flyctl` commands needed to deploy to Fly.io. It is the recommended target for global edge computing with low operational overhead.

The **railway** target generates the configuration for Railway and triggers the deployment via its API. It is the recommended target for teams that prefer PaaS without operation.

The **k8s** target generates standard Kubernetes manifests (Deployment, Service, Ingress, ConfigMap, Secret, HorizontalPodAutoscaler) with reasonable values. For advanced configurations, this target is a starting point that the team customizes, not a complete solution.

### 10.4 Deployment pipeline

The `ag deploy` command runs a standardized pipeline: schema validation, compilation with `cargo build --release --target <target>`, construction of the Docker image from a `scratch` or `distroless` base, execution of smoke tests, push of the image to a registry, application of database migrations in order, rolling deployment with healthchecks, and post-deployment verification.

### 10.5 Reverse proxy and TLS

For docker-compose deployments, `ag-cloud` configures Caddy as a reverse proxy with automatic TLS. Caddy obtains and renews Let's Encrypt certificates without explicit configuration. For environments where TLS is managed by an external load balancer (Cloudflare, AWS ALB), Caddy is disabled.

### 10.6 Integration with `ag-domains`

Introduced by `ADR-0007`. When a project declares domains in its `.ag` contract (`domain` block of DSL v0.7), `ag deploy` resolves a six-step flow coordinated with `ag-domains`:

1. **Validate domain control.** Insertion of a verification TXT record via the configured `DnsProvider` and confirmation of its presence.
2. **Configure application DNS.** `upsert_record` to point the domain to the deployment target (CNAME to the Fly/Railway host, or A/AAAA records in docker-compose).
3. **Issue or renew TLS.** ACME client against Let's Encrypt (DNS-01 preferred). The certificate is stored in filesystem or `ag-storage`.
4. **Associate the domain to the target.** Configure the reverse proxy (Caddy in docker-compose, fly cert in Fly, etc.) to serve the domain with the issued certificate.
5. **Materialize SPF/DKIM/DMARC** that `ag-mail` has declared in its `MailSender::dns_requirements`.
6. **Verify propagation** against multiple public resolvers before marking the deployment as successful.

`ag-cloud` does **NOT depend rigidly** on `ag-domains` in all targets: if the project does not declare domains, the flow is omitted. If the target is one where TLS is managed by an external load balancer (Cloudflare in front, AWS ALB), `ag-cloud` can skip step 3 without affecting the rest of the pipeline. This flexibility is what keeps `ag-domains` as an optional module, not as a mandatory piece of the runtime.

---

