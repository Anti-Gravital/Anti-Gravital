# RFC-0033 - Domain-to-deploy loop

- Status: proposed
- Author: Gravital Labs community contributors (wave 8 architecture role)
- Draft date: 2026-11-22
- Target phase: additive Phase 4.8 domain loop; no new crate
- Affected modules/crates: `ag-cli` (`dnsapi` role: `ag dns plan|apply|status`,
  `ag domains attach`), `ag-domains` (reuse only; one additive `TargetKind::Link`
  variant is specified but not implemented here), `ag-edge` (reuse only),
  `templates/` (`ag.toml` manifest contract), `deploy/k8s/` and `deploy/docker/`
  (generated fragments, not checked-in runtime artifacts)
- Predecessor RFCs: RFC-0007, RFC-0011, RFC-0016, RFC-0019, RFC-0020, RFC-0021,
  RFC-0024
- Governing ADRs: ADR-0008 (English canonical for technical docs), ADR-0009
  (native-first, real-state honesty), ADR-0011 (no third-party brands as
  component names)
- Comment period: minimum seven calendar days (committee decision phase 4+)

## 1. Motivation

A developer can scaffold a project with `ag new --template <name>`, build it
offline, and produce optional Docker or Kubernetes packaging. The project is
still not reachable: no contract connects a hostname in the manifest to the
deploy artifacts, and no workflow reaches a real domain from a plan. The
pieces exist separately in `ag-domains` (attachment state machine, DNS
instructions, ownership proof, propagation, ACME, CAA preflight, tombstone),
in `ag-edge` (fail-closed hostname routing and SNI selection) and in the `deploy/`
artifacts (Ingress, compose). What is missing is the contract that composes
them, per environment, with public or private visibility, without making a
live provider, a cluster or a daemon a requirement.

## 2. Problem

1. There is no canonical manifest field set for "this project, in this
   environment, is served at this host, with this visibility, pointing at this
   target". `ag-domains` has an attachment record; the generated project has
   `ag.toml` (RFC-0019) and `config/<environment>.toml` (RFC-0020); the two are
   not connected.
2. `ag-domains` defaults to `NullVerifier` (which never confirms ownership) and
   `DnsMode::Manual`. This is correct fail-closed behaviour but an implementer
   could accidentally treat a draft attachment as live. The contract must make
   the safe path the only default path.
3. The optional Ingress in `deploy/k8s/` ships with a placeholder host and no
   TLS; the compose files publish loopback ports. Neither knows about domains.
4. RFC-0021 requires one unified plan/apply model with a plan identifier and a
   stale-state check. RFC-0024 prerequisite 5 forbids introducing a second
   `deploy.plan`/`deploy.apply` pair next to it. Any new mutating command must
   reuse that model, not invent another.
5. RFC-0024 prerequisite 7 forbids hidden DNS egress from a stdio-only
   interface. A `status` command must be offline unless explicitly opted in.

## 3. Alternatives considered

**A. Put the hostname only in `schema.ag` (DSL `domain` block, RFC-0007 4.6).**
Rejected as the only path: the DSL describes infrastructure intent, but a
custom domain is a deployment-time binding that varies per environment and
carries visibility rules. It also requires compiling the DSL for a simple
lookup. Kept as an optional input source; the manifest remains authoritative
for the loop.

**B. Add a separate `domains.toml` file with no link to `ag.toml`.**
Rejected: two sources of truth for project identity and environment names,
which violates DRY and invites drift with RFC-0019/RFC-0020.

**C. Require a DNS provider adapter and a cluster for the loop to work.**
Rejected: violates rule 12 and ADR-0009 rules 2 and 5. Provider automation and
a cluster stay optional accelerators. The native path prints exact records and
renders fragments; the operator applies them.

**D. Extend `ag.toml` with a `[domain]` contract, rendered by plan/apply into
existing deploy artifacts (chosen).** One manifest, one executor, native by
default, with provider and cluster steps gated behind features and explicit
flags.

## 4. Proposed design

### 4.1 Manifest contract (authoritative fields)

The contract is a `[domain]` table in the project manifest `ag.toml`
(RFC-0019). Non-secret, environment-scoped values may also live in
`config/<environment>.toml`; secrets are never in either file (RFC-0020).

```toml
[domain]
domain = "app.example.com"                  # required FQDN; Hostname::parse
zone = "example.com"                        # optional; derived via registrable_domain (PSL feature)
environment = "production"                  # default environment id for this binding
visibility = "private"                      # public | private; DEFAULT private
target = { kind = "service", ref = "api" }  # kind: service | link
ttl = 300                                   # seconds, clamped 60..=86400; default 300
tls = "managed_http01"                      # disabled | managed_http01 | managed_dns01
allow_wildcard = false                      # default false

[domain.dns]                                # optional explicit DNS record set
TXT = [{ name = "_dmarc", value = "v=DMARC1; p=quarantine" }]
CNAME = [{ name = "docs", value = "docs.example-cloud.net" }]

[domain.environments.staging]
domain = "app.staging.example.com"
visibility = "private"
target = { kind = "link", ref = "https://tunnel.internal.example" }

[domain.environments.production]
visibility = "public"
tls = "managed_http01"
```

Field semantics:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `domain` | FQDN string | none (required) | Canonical hostname. Validated by `Hostname::parse`; ASCII identity and Unicode display are kept separately. |
| `zone` | DNS zone string | derived eTLD+1 | Provider administration unit; passed to `DnsProvider::zone_id`. Derivation uses the single `registrable_domain` source of RFC-0016. |
| `environment` | string | `production` | Default environment id, matching `config/<environment>.toml`. |
| `visibility` | `public` \| `private` | `private` | `public` emits public DNS records and a public certificate path. `private` emits no public record; access is through a link or a local resolver entry. Fail-closed default. |
| `target.kind` | `service` \| `link` | `service` | `service`: `ref` names a service in the generated project (compose service / k8s Service `template`). `link`: `ref` is an `https://` URL to a private access gateway or external upstream; not a project service. |
| `target.ref` | string | `default` | Service name or link URL. A `link` URL must be HTTPS and pass the host allowlist check. |
| `dns` | map of record sets | empty | Extra records beyond the generated ownership and routing records. Keys are `RecordType` names; `name` is relative to `zone` unless it is an FQDN. |
| `ttl` | integer | `300` | Applied to generated and explicit records; bound to `EdgeTargets.ttl`. Out-of-range values are a validation error, never silently clamped. |
| `tls` | `disabled` \| `managed_http01` \| `managed_dns01` | `managed_http01` for exact hosts, `managed_dns01` for wildcards | Reuses `ag-domains::acme`. `private` bindings default to `disabled` unless an internal CA is configured. |
| `allow_wildcard` | bool | `false` | A hostname beginning with `*.` is rejected unless this is `true` **and** `tls = "managed_dns01"`. No wildcard by default. |

Per-environment resolution order (highest first), owned by the `envrules`
role: process environment override, `config/<environment>.toml`,
`[domain.environments.<environment>]`, `[domain]` defaults. An unknown
environment is a validation error. The resolved object is called the
**domain binding** and is the only input to plan generation.

### 4.2 Reuse of the existing control plane

The resolved binding maps directly onto `ag-domains` types; no parallel state
machine is introduced:

| Manifest field | `ag-domains` target |
| --- | --- |
| `domain`, `zone` | `Hostname`, `EdgeTargets` + `DnsProvider::zone_id` |
| `environment` | `DomainAttachment.environment` |
| `target.kind = service`, `ref` | `TargetKind::Service`, `target_ref` |
| `target.kind = link`, `ref` | new additive `TargetKind::Link` (specified here, implemented in a later PR) |
| `dns`, `ttl` | `generate_instructions` + `[domain.dns]` extras, `EdgeTargets.ttl` |
| `tls` | `TlsMode::{Disabled,ManagedHttp01,ManagedDns01}` |
| `visibility` | new binding-level policy; does not change the four readiness enums |

The four readiness dimensions (`ownership`, `dns`, `tls`, `routing`) and the
derived lifecycle from `DomainAttachment` are reused verbatim. Activation still
requires ownership verified, DNS routable, TLS active (or disabled) and routing
ready. `NullVerifier` stays the default: an apply step that has not wired a real
verifier (`PropagationOwnershipVerifier` under the `propagation` feature, or an
equivalent injected verifier) must not mark a binding verified. Tombstone on
detach uses `DEFAULT_TOMBSTONE_SECS` (30 days) to prevent subdomain takeover.

### 4.3 Plan and apply flow (RFC-0021 unified model)

`ag dns plan` and `ag dns apply <PLAN_ID>` use the single `--plan` /
`--apply PLAN_ID` model. They are the domain group of the same executor used by
the CLI and TUI; they do **not** add a second `deploy.plan` pair (RFC-0024
prerequisite 5). Every mutating step requires the plan identifier, checks that
inputs and the native store snapshot are unchanged since the plan was built,
and appends an audit record.

Plan output (`--output json`, `contract_version = "1"`):

```json
{
  "contract_version": "1",
  "plan_id": "sha256:<hash-of-manifest-and-state>",
  "environment": "production",
  "binding": {
    "host": "app.example.com", "zone": "example.com",
    "visibility": "public", "target": { "kind": "service", "ref": "api" },
    "ttl": 300
  },
  "validation": { "ok": true, "errors": [], "warnings": [] },
  "dns": {
    "mode": "manual",
    "records": [
      { "type": "CNAME", "name": "app.example.com", "value": "edge.example-cloud.net", "ttl": 300, "is_ownership": false },
      { "type": "TXT", "name": "_ag-domain.app.example.com", "value": "ag-verification=<token>", "ttl": 300, "is_ownership": true }
    ]
  },
  "ownership": { "method": "txt", "verifier": "null|propagation" },
  "acme": { "mode": "managed_http01", "caa_preflight": "pass|unknown" },
  "deploy": {
    "target": "k8s|compose|none",
    "actions": [ { "kind": "ingress_host|ingress_tls|proxy_route|none", "host": "app.example.com", "file": "relative/path", "secret_ref": "name-only" } ]
  },
  "local": ["render config/domain.<env>.toml", "render /etc/hosts entry for private link"],
  "provider": ["publish TXT _ag-domain.app.example.com", "publish CNAME app.example.com"],
  "requires_online": true,
  "requires_credentials": ["<env var NAME only>"],
  "requires_cluster": false
}
```

Apply order (each step is separately verifiable and resumable):

1. **Validate** the binding; abort before any mutation on validation error.
2. **Create or refresh the attachment** in the native JSON store (`Draft`,
   `PendingOwnership`); reserve the tombstone check first.
3. **Publish instructions**: manual mode prints the exact record set and BIND
   zone fragment; adapter mode applies records through a feature-gated
   `DnsProvider`. No provider token is ever read from the repository.
4. **Verify ownership**: with `NullVerifier`, apply refuses to advance past
   `draft` and reports `pending_ownership` (fail-closed). With a real verifier
   and explicit online opt-in, `PropagationChecker` confirms the TXT token on
   at least N resolvers.
5. **ACME**: reuse `caa` preflight, then `ManagedHttp01` for exact public hosts
   or `ManagedDns01` for wildcards/private internal CA. Certificate key material
   never enters the repository; deploy wiring references a Secret by name.
6. **Render deploy wiring** into a *copied* overlay or compose file (section
   4.4). No `kubectl apply`, `docker compose up` or daemon start by default.
7. **Activate routing** at the edge only when all four dimensions are ready;
   `ag-edge` keeps failing closed for unknown hosts.

`ag dns status` is offline by default and reports the four dimensions plus
lifecycle from the local store. It performs DNS egress only with an explicit
`--online` flag, satisfying RFC-0024 prerequisite 7.

### 4.4 Custom domain to Ingress host or compose proxy rule

`deplink` role consumes the plan's `deploy.actions` to render fragments; it
never edits shared CLI files.

- **Kubernetes**: render an Ingress fragment into a copied overlay with
  `spec.rules[].host = <host>`, `spec.tls[].hosts = [<host>]` and a
  `secretName` reference, backend `Service template:8080` (the unprefixed
  Service produced by every family overlay). The generated file is added to a
  copy under `deploy/k8s/overlays/<family>/`, enabled explicitly via the
  existing `optional/ingress` component pattern. No certificate, token or
  wildcard host is written. TLS secret values are supplied by the cluster
  admin (`SECRETS.md`), by name only.
- **Compose**: render a reverse-proxy route fragment into a copied compose file:
  a host matcher for `<host>`, TLS termination delegated to the proxy, and an
  upstream `http://<service>:<port>` for `target.kind = service`. For
  `target.kind = link` the route forwards to the link URL. Default published
  ports remain bound to loopback; the public route is an explicit opt-in, not a
  change to the shipped compose files.
- **`target = none` / no deploy target**: the loop still produces DNS and
  ownership instructions; no deploy artifact is rendered.

### 4.5 Private access

`visibility = "private"` means:

1. No public routing record (A/AAAA/CNAME) is emitted by the tool. Only the
   ownership TXT (needed for control) and, if configured, an internal-CA
   challenge record are produced.
2. Access is expressed either as `target.kind = link` with an HTTPS private
   gateway URL, or as a local-only mapping: the plan renders an `/etc/hosts`
   entry (or a local resolver note) pointing the host at a loopback/private
   address. Both are local/offline artifacts.
3. TLS defaults to `disabled` for private bindings; when a private CA is used,
   `ManagedDns01` is required. Public ACME HTTP-01 validation is never attempted
   for a private host.
4. The edge resolver fails closed for the host unless a matching active
   binding exists; there is no wildcard catch-all.

### 4.6 What stays local and offline

Always local (no network, no credentials): manifest validation, hostname
normalization/classification, zone derivation, DNS instruction generation,
BIND zone export, ownership token generation, lifecycle/tombstone store,
plan hashing and stale-state checks, Ingress/compose fragment rendering, local
hosts entries, audit records, and `ag dns status` without `--online`.

Requires a real provider or network (marked in every doc and example):
publishing records through an adapter, propagation checks against configured
resolvers, real ACME issuance/renewal, and an HTTPS `curl` to a real host.
Every one is behind a Cargo feature or an explicit online flag and is never
required by `ag new`, `cargo build`, `cargo test` or `ag dev`.

### 4.7 Security constraints

- **No credentials in the repository.** Provider tokens are read from the
  process environment or RFC-0020 `Secret<T>` / encrypted
  `config/<environment>.secrets`; manifests, plans, fragments and descriptors
  contain variable *names* only. Plan/apply output redacts values. `.env` is
  excluded from images (`deploy/docker/.dockerignore`) and k8s Secrets are by
  name only.
- **No wildcard by default.** `allow_wildcard = false`; a wildcard host is
  rejected unless explicitly enabled with DNS-01.
- **Fail-closed.** `NullVerifier` default; unknown edge hosts are not served;
  private bindings emit no public record; validation errors abort before any
  mutation; a stale plan id aborts apply; a `link` target must be HTTPS and on
  an explicit host allowlist (no SSRF through arbitrary fetch URLs).
- **Zone takeover prevention.** Ownership proof before activation and a 30-day
  tombstone after detach (RFC-0011); re-attaching a tombstoned host is refused.
- **No third-party brand names** in component, field or feature identifiers
  (ADR-0011). Provider names appear only as adapter labels behind features.

### 4.8 Compatibility with parallel wave-8 manifests

Other wave-8 roles drafted project-local manifest files before this RFC was
merged. They are **adapters**, not additional sources of truth: each must
resolve to the canonical `ag.toml` binding defined in 4.1.

| Concern | Canonical (this RFC) | envrules adapter | deplink adapter |
| --- | --- | --- | --- |
| File | `ag.toml [domain]` | `domain-manifest.yaml` (`schema_version: 1`) | `domain.yaml` (`schemaVersion: 1`) |
| Environment keys | `dev`, `staging`, `production` | `dev`, `staging`, `prod` | `dev`, `staging`, `production` |
| Visibility | `public`, `private` | `private`, `vpn-only`, `public` | `visibility` string |
| Service | `target = { kind, ref }` | `services[].{ name, host, visibility, port, allow_sources }` | `service`, `localPort` |
| TLS | `disabled` or `managed_*` | `tls.{ required, termination, ... }` | `tls.{ enabled, secretName }` |

Reconciliation rules:

1. `prod` is accepted as an alias of `production` at CLI and manifest
   boundaries and is normalized to `production` in every plan. No runtime
   environment-selector variable is introduced (RFC-0020); `--env` is an
   explicit command argument.
2. `vpn-only` is not a third top-level visibility value. It is `private` plus
   a CIDR `allow_sources` allowlist on the service. An allowlist never makes a
   host public and never emits a public DNS record.
3. `local` is loopback-only: no `[domain]` binding, no DNS and no TLS. It is a
   valid `--env` target that produces an empty domain plan.
4. A YAML adapter is conformant only if its resolved binding equals the
   canonical TOML binding. `ag dns plan` reports the resolved binding so the
   two can be compared.
5. The `.test` suffix used by offline examples is deliberate (RFC 6761) and
   must never be published to a real provider.

## 5. Implementation plan (only after acceptance)

Separate PRs, coordinated across wave 8 roles; this RFC is design only.

1. **dnsarch (this RFC):** contract and end-to-end doc. No code.
2. **dnsapi:** `ag dns plan|apply|status` and `ag domains attach` reading the
   `[domain]` contract, reusing `ag-domains` and the RFC-0021 executor.
3. **envrules:** environment registry and visibility/host precedence in
   `config.toml`/`ag.toml`, including validation errors for unknown
   environments.
4. **deplink:** Ingress/compose fragment rendering from `deploy.actions`.
5. **ag-domains (additive):** `TargetKind::Link` variant, if accepted by the
   owning PR; all existing public contracts stay unchanged.
6. **dnsqa:** `scripts/validate-domain-loop.sh` with a mock DNS/hosts scenario
   and honest SKIPs.
7. **dnssec:** read-only audit (credentials, wildcard, takeover, SSRF).
8. **dnsdocs:** bilingual troubleshooting and template notes.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Second plan model appears alongside RFC-0021 | Medium | High | Reuse one executor and one plan id; RFC-0024 prerequisite 5 is explicit. |
| Private binding accidentally emits a public record | Low | High | `visibility = private` defaults and fail-closed tests; no public record in the plan's `dns.records`. |
| Credentials leak into a plan or descriptor | Medium | High | Variable names only; redaction tests; `.dockerignore` and Secret-by-name rules. |
| Zone/subdomain takeover after detach | Low | High | Ownership proof plus 30-day tombstone; re-attach refused while tombstoned. |
| DNS egress hidden in a "status" command | Medium | Medium | Offline by default; `--online` explicit; stdio-only contract preserved. |
| Wildcard issued without DNS-01 | Low | Medium | `allow_wildcard` gate and TLS mode validation at plan time. |

## 7. Impact

- **Scope:** additive; makes the existing domain and deploy subsystems
  composable. No new crate, no daemon, no mandatory external service.
- **Roadmap:** additive stage of Phase 4.8; no master-document change required
  by this RFC because the contract is a derived deployment document.
- **Operational complexity:** none by default; provider/cluster steps are
  opt-in features.
- **Public APIs:** one additive `TargetKind::Link` variant in `ag-domains`; one
  new versioned plan JSON shape in `ag-cli`. No existing API is broken.
- **Documentation:** this RFC, `docs/dns/DOMAIN_LOOP.md`, `docs/INDEX.md`,
  `docs/rfc/README.md`, `CHANGELOG.md`.

## 8. Rollback

The native store is a JSON file and every apply begins from a snapshot.
Rollback is: (1) `ag dns apply --rollback <PLAN_ID>` restores the binding to its
pre-apply lifecycle and removes only the generated Ingress/compose fragment
tracked in the plan; (2) `ag domains detach` removes routing, stops renewal,
sets `TlsStatus::Retired` and applies the tombstone; (3) in manual mode the
operator is instructed to delete the published provider records. No destructive
data migration is involved. DNS propagation is eventually consistent, so
rollback stops serving immediately at the edge even if an old record is still
cached. Rollback is triggered by a failed ownership check, a misconfigured
record, a security finding, or an operator request.

## 9. Decision

- Decider: technical committee (phase 4+).
- Decision date: pending.
- Result: proposed; not accepted. No implementation is authorized until a
  decision is recorded.

## 10. References

- RFC-0007 (ag-domains scope), RFC-0011 (control plane, tombstone, ACME),
  RFC-0016 (single eTLD+1 source).
- RFC-0019 (`ag.toml`), RFC-0020 (`Secret<T>`, environment config),
  RFC-0021 (plan/apply, audit, JSON output), RFC-0024 (prerequisites 5 and 7).
- ADR-0008 (language), ADR-0009 (native-first), ADR-0011 (brand policy).
- `crates/ag-domains/src/{api,attachment,instructions,propagation,provider,caa,acme}/*`.
- `deploy/k8s/optional/ingress/ingress.yaml`, `deploy/docker/README.md`.
- `docs/dns/DOMAIN_LOOP.md` (end-to-end diagram and worked example).
- `CLAUDE.md` rules 12, 15, 16, 22, 26, 28, 32.
