# Environment and visibility rules for the domain loop

Status: **normative design contract, design only**. This document defines rules;
it adds no CLI code, no crate and no live provider integration. Implementation
of the domain loop belongs to other roles in wave 8 (the `ag dns` / `ag domains`
commands and the deploy-artifact wiring). Machine-checkable rules are named
`ER-01` through `ER-11` so an implementation can reference them without
reinterpreting this text.

Authorities this document serves, and does not amend:

- `RFC-0020`: declared environment variables, resolution order, `Secret<T>`,
  native encrypted secrets. In particular, RFC-0020 does **not** standardize a
  runtime environment selector.
- `RFC-0019`: `ag.toml` as the project manifest and the inward dependency rule.
- `RFC-0033` (domain-to-deploy contract, in preparation by the `dnsarch` role):
  this document supplies its environment/visibility vocabulary. If RFC-0033
  lands with a different manifest path, the vocabulary and rules here remain
  authoritative; the path is an implementation detail.
- `docs/templates/TEMPLATE_DESIGN.md`: sections 2 (naming, variables), 3
  (configuration and precedence), 6 (secrets), 7 (template/adapter boundary)
  and 10 (Docker/Kubernetes compatibility matrix).
- `ADR-0008` (English canonical for technical docs), `ADR-0009` (native default,
  adapters behind features, no external service required locally), `ADR-0011`
  (adapter labels only, no third-party brand names as component names).
- `CLAUDE.md` rules 12 (integrate, never replace), 15, 16, 19, 26 and 31.

---

## 1. Purpose and scope

The domain loop needs one deterministic answer to a single question:

> Given a project, an environment and a service, is that service exposed on a
> DNS name, and if so, who can reach it and under what TLS policy?

Without a fixed answer, each deploy target invents its own default and a service
meant to stay internal becomes public by accident. This document fixes the
environment model, the visibility values, the host allowlist, the TLS policy,
where each field lives, and how the domain manifest is selected per environment.

In scope: the policy and the shape of the data that carries it.
Out of scope: DNS record provisioning, certificate issuance, CLI commands,
kustomize/compose rendering and any live provider or cluster operation.

---

## 2. Environment model

Four environments exist. `local` is special: it has no DNS surface and therefore
no entry in the domain manifest. The three deployable environments map to the
per-environment files already defined by `RFC-0020` and
`TEMPLATE_DESIGN.md` section 3 (`config/<env>.toml`,
`config/<env>.secrets`, `config/<env>.recipients`).

| Environment | Purpose | Bind | DNS / host allowlist | Default visibility | TLS | External ingress |
| --- | --- | --- | --- | --- | --- | --- |
| `local` | Developer machine, offline-first | `127.0.0.1:8080` | None; `localhost` and loopback only | `private` | None (no name is bound) | No |
| `dev` | Shared development | `0.0.0.0:8080` in a container | Internal zone, wildcards allowed | `private` | Required for any bound name, internal termination | Restricted by source CIDRs |
| `staging` | Pre-production mirror | `0.0.0.0:8080` in a container | Staging zone, wildcards allowed | `private` | Required, edge termination | Restricted by source CIDRs |
| `prod` | Production | `0.0.0.0:8080` in a container | Exact hosts only | `private` | Required, edge termination, HSTS | Explicit per service |

Rules:

- **ER-02a.** The only environments with a domain-manifest entry are `dev`,
  `staging` and `prod`. An entry for `local` is rejected. `local` exposure is
  loopback-only, needs no domain, no certificate and no credential, which keeps
  the native path buildable offline (`ADR-0009` rule 2, rule 12).
- **ER-02b.** Environment names are lowercase tokens exactly matching the
  `config/<env>.toml` file stem. Introducing a fifth name requires an RFC
  (`CLAUDE.md` rule 22); it is not a config choice.
- **ER-02c.** `dev`, `staging` and `prod` differ in *policy*, not in mechanism.
  The same manifest schema, the same three visibility values and the same
  validation rules apply to all three. Only the allowlist, TLS termination and
  HSTS values change.

`local` is listed in the environment model because it is where a template is
scaffolded and run by default, but it is deliberately absent from the example
manifest: there is no DNS or TLS artifact to describe.

---

## 3. Visibility values and default-deny

A service has exactly one visibility value. Missing or unknown values are not
errors that fail open: they resolve to `private` (ER-04).

| Value | Meaning | DNS record | Ingress / Service | Reachability | Required fields |
| --- | --- | --- | --- | --- | --- |
| `private` | Internal only. The default. | None. An internal service name may be recorded for the deploy graph, never a public record. | `ClusterIP` Service, no Ingress rule | Pod network or loopback only | `name`, `port` |
| `vpn-only` | Reachable from explicitly allowed networks. | Internal or private-zone name, in the environment allowlist | `ClusterIP` Service plus an Ingress rule restricted to `allow_sources` | Only the CIDRs in `allow_sources` | `name`, `port`, `host`, non-empty `allow_sources`, TLS |
| `public` | Reachable from the internet. Explicit opt-in. | Public name, in the environment allowlist | `ClusterIP` Service plus Ingress with TLS | The CIDRs in `allow_sources`, which must explicitly include an internet CIDR | `name`, `port`, `host`, `allow_sources` with `0.0.0.0/0` or `::/0`, TLS |

Default-deny rules:

- **ER-03.** `default_visibility` and every `visibility_default` **must** be
  `private`. An environment may not default to `public` or `vpn-only`. There is
  no implicit public service.
- **ER-04.** A service without a `visibility` field inherits its environment
  default (`private`). A value outside `{private, vpn-only, public}` is a hard
  validation error, never a downgrade to a different value.
- **ER-05.** `public` additionally requires: a `host` present and in the
  environment allowlist; `tls.required: true`; `tls.termination` not `none`;
  and `ingress: true`. If any is missing the manifest is invalid, so "public but
  no TLS" cannot be expressed.
- **ER-07.** `public` must write an internet CIDR (`0.0.0.0/0` and/or `::/0`)
  explicitly in `allow_sources`. `vpn-only` must have a non-empty `allow_sources`
  and must **not** contain an internet CIDR; a service that wants the internet
  must say `public` and satisfy ER-05.
- **ER-08.** `private` must not declare `allow_sources`. A host is optional for
  `private`; if present it still has to pass the allowlist check (ER-10), but it
  never produces an Ingress rule.

Why `vpn-only` is kept as a distinct value: collapsing it into `public` would
force every restricted service to pretend it is internet-facing and would hide
the CIDR list; collapsing it into `private` would make the allowlist invisible
to review. A separate value keeps the exposure decision explicit and auditable.

---

## 4. Host allowlist

- **ER-09.** Each environment declares `allowed_hosts`. A single-label wildcard
  of the form `*.zone` is allowed in `dev` and `staging`. In `prod`, a wildcard
  is rejected unless `allow_wildcard: true` is set **and** a security review is
  recorded in the change that adds it. Wildcard certificates and zone takeover
  are audit concerns; the `dnssec` role reviews them.
- **ER-10.** A service `host` matches the allowlist when it equals an entry
  exactly, or when the environment has `allow_wildcard: true`, the entry is
  `*.zone` and the host has exactly one more label than `zone`. Multi-label
  wildcards are not supported. A host in the wrong zone, outside the allowlist,
  or used by two services in the same environment is rejected.
- **ER-10b.** The zone itself is not a service host: `example.com` is not a
  valid `host` unless it is listed explicitly in `allowed_hosts`. Canonical-host
  redirects (for example `www` to the apex) are expressed in the deploy-layer
  routing config, in the same vocabulary as the `edge-router` template's
  `[[redirects]]` block, and use the same allowlist.
- **ER-10c.** Documentation-only names under `*.invalid` are used in examples.
  A real manifest must use names the operator controls; the validator checks
  syntax and allowlist membership, not DNS ownership.

The allowlist is per environment and is the boundary between "the operator
typed a host" and "the tool accepted a host". It prevents a typo in `prod` from
silently creating a record in a zone the project does not own.

---

## 5. TLS expectations

TLS is required whenever a DNS name is bound. The difference between
environments is the trust source and the strictness, not whether TLS exists.

| Environment | `tls.required` | Termination | Certificate source | Min version | HSTS | Plaintext |
| --- | --- | --- | --- | --- | --- | --- |
| `local` | Not applicable | None | None | Not applicable | No | Loopback only |
| `dev` | `true` for any bound name | `internal` | Internal CA, generated locally | `1.2` | No | Inside the cluster, behind internal termination |
| `staging` | `true` for any bound name | `edge` | Staging certificate from ACME or an internal CA | `1.2` | Optional | HTTP redirects to HTTPS (308) |
| `prod` | `true` | `edge` | Public CA via ACME | `1.2` | `true` | HTTP redirects to HTTPS (308) |

Rules:

- **ER-TLS.** If any service in an environment declares a `host`, that
  environment must set `tls.required: true`. `termination: none` is valid only
  where no host is bound (that is, `local`, which has no manifest entry).
- **ER-05b.** An Ingress that serves a `public` name must not serve it over
  plaintext: the HTTP route redirects with 308 and the HTTPS route carries the
  certificate. The health contract (`TEMPLATE_DESIGN.md` section 4) stays
  unauthenticated; probes use the Service, not the public name.
- **ER-SECRET.** A private key never appears in the manifest, in `config/*.toml`,
  in a Dockerfile, in a compose file or in a Kubernetes manifest. Certificate
  material is provisioned by the operator through the `ag-domains` / `ag-edge`
  adapters behind Cargo features; no certificate service is required to build
  or run the native local path (`ADR-0009`, rule 12).

Certificate issuance and renewal are owned by `ag-domains`; routing by SNI and
canonical redirects by `ag-edge`. This document only states the policy those
adapters must satisfy.

---

## 6. Field placement: ag.toml, config and the domain manifest

| Field | `ag.toml` | `config/<env>.toml` | `config/<env>.secrets` | Domain manifest | Operator |
| --- | --- | --- | --- | --- | --- |
| Variable declaration (`type`, `required`, `secret`, `description`) | Yes (RFC-0020 4.1) | No | No | No | No |
| Non-secret runtime values (`[bind]` host/port, rate limits) | Default only | Yes | No | No | No |
| Secret values (connection strings, signing keys) | No | No | Yes, encrypted | No | Provides via process env or target secret store |
| Service name and `port` | No | No | No | Yes (topology) | No |
| `visibility`, `host`, `allowed_hosts`, `allow_sources` | No | No | No | Yes (topology) | No |
| `tls` policy and `ingress` flag | No | No | No | Yes (topology) | No |
| Actual DNS record and certificate | No | No | No | No | Yes, through adapters |
| Environment selection | No | No | No | Key of `environments` | Yes, explicit `--env` |

Rationale: `ag.toml` answers "what does this program read?"; `config/` answers
"what value does it read here?"; the domain manifest answers "what is exposed
here?". Mixing them would make a topology change look like a code change and a
secret look like documentation.

**No runtime selector variable.** RFC-0020 defines a resolution order but no
selector variable, and `TEMPLATE_DESIGN.md` section 3 forbids inventing one.
This document does not add `AG_ENV`, `ENV`, `APP_ENV` or any equivalent. The
process environment stays the highest-priority *value* source (RFC-0020 4.2,
sources 1 to 5: process env, decrypted secrets file, `config/<env>.toml`,
`config/default.toml`, declared default). Choosing *which* environment's files
and manifest are in play is a tooling decision made outside the process:

- domain-loop commands take an explicit `--env <name>` argument;
- a compose file or kustomize overlay selects the environment by being the
  artifact for that environment, which is how wave 7 already organizes
  `deploy/k8s/overlays/<family>/` and `deploy/docker/compose/<family>.yml`.

Resolution order (RFC-0020 4.2), highest first: process environment, decrypted
`config/<env>.secrets`, `config/<env>.toml`, `config/default.toml`, declared
default in `ag.toml`. All values validate at startup before a listener opens;
failures print names and expected types, never values.

---

## 7. Selecting the domain manifest per environment

One manifest file carries an `environments` map with the three deployable keys.
The machine-readable example is
[`examples/domain-manifest.yaml`](examples/domain-manifest.yaml).

- **ER-01.** The file declares `schema_version: 1`. Unknown top-level or
  environment-level keys are rejected; this is a strict schema, not a bag of
  hints.
- **ER-11.** The environment is selected by an explicit `--env <name>` argument
  (or by the deploy artifact that is already environment-specific). There is no
  implicit default: a command invoked without an environment must refuse to
  guess rather than fall back to `dev`. `--env local` is accepted and produces a
  loopback-only plan with no DNS, TLS or Ingress artifact; it must not emit a
  domain manifest entry.
- **ER-11b.** A change to `prod` exposure is reviewed as a security change. The
  manifest is data, but flipping a service to `public` changes the attack
  surface and therefore needs the same review as a code change (`CLAUDE.md`
  rules 16 and 22).

Selection is a key lookup, not a file-name convention: given `--env staging`,
the tool reads `environments.staging` from the chosen manifest. This keeps one
schema and one validator for all three environments and makes a cross-environment
diff possible (`ag dns plan --env staging` versus `--env prod`).

---

## 8. Decision table

The table resolves the common cases. "Ingress" means an Ingress rule with a
public or restricted listener; the Service remains `ClusterIP` in every case.

| Scenario | Environment | Visibility | Host | TLS | Ingress | `allow_sources` | Outcome |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Run a template on a laptop | `local` | `private` | None | None | No | None | Loopback `127.0.0.1:8080`; no domain artifact |
| Internal API used by other services | any | `private` | Optional, allowlisted | Only if host bound | No | None | Service DNS name only |
| Admin panel reachable from the office VPN | `dev`, `staging`, `prod` | `vpn-only` | Allowlisted | Required, internal in `dev`, edge otherwise | Restricted | Office/CIDR list, no internet CIDR | Ingress with source-range restriction |
| Public marketing docs | `staging`, `prod` | `public` | Allowlisted | Required, edge | Yes | Explicit `0.0.0.0/0` and `::/0` | Public HTTPS name, HTTP 308 to HTTPS |
| A new service with no visibility field | any | `private` (inherited) | None | None required | No | None | Fail closed, no exposure |
| `public` without TLS, or host outside allowlist | any | rejected | n/a | n/a | n/a | n/a | ER-05 / ER-10 fail validation |
| `vpn-only` that allows `0.0.0.0/0` | any | rejected | n/a | n/a | n/a | n/a | ER-07: use `public` and satisfy ER-05 |
| Wildcard host in `prod` | `prod` | any | `*.example.com` | Required | Per visibility | Per visibility | ER-09: rejected unless `allow_wildcard` and review |

---

## 9. Examples

### 9.1 Promote a service from private to vpn-only in prod

Before:

```yaml
- name: admin
  visibility: private
  port: 8080
```

After:

```yaml
- name: admin
  host: admin.example.com          # must be in prod.allowed_hosts (ER-10)
  visibility: vpn-only
  port: 8080
  allow_sources:                   # non-empty, no internet CIDR (ER-06, ER-07)
    - "10.42.0.0/16"
```

`prod.tls.required` is already `true` and `ingress` is `true`, so the change is
valid. The admin name is not made public, and the source list is visible in
review. No secret, certificate or DNS credential is added to the repository.

### 9.2 Invalid examples and the rule they violate

| Manifest fragment | Violation | Result |
| --- | --- | --- |
| `visibility: public` with `tls.required: false` | ER-05 | Invalid: public without TLS cannot be expressed |
| `host: api.other-zone.com` in `prod` | ER-10 | Invalid: not in `prod.allowed_hosts` |
| No `visibility` field on a new service | ER-04 | Valid, resolves to `private`; no exposure |
| `visibility: vpn-only` with `allow_sources: ["0.0.0.0/0"]` | ER-07 | Invalid: must be `public` with TLS |
| `allowed_hosts: ["*.example.com"]` in `prod`, `allow_wildcard: false` | ER-09 | Invalid: wildcard needs explicit opt-in and review |
| `environments.local: {...}` | ER-02a | Invalid: `local` has no domain surface |

### 9.3 Full example

[`docs/dns/examples/domain-manifest.yaml`](examples/domain-manifest.yaml) is a
complete manifest with `dev`, `staging` and `prod`. It shows the three visibility
values, exact and wildcard allowlists, internal and edge TLS termination, HSTS
in prod, and the internet CIDR written out for the two public services.

---

## 10. Verification of the example

The example was parsed with PyYAML 6.0.3 on Python 3.14.4 and checked against
the rule IDs above with a one-off validator (not CLI code, not committed as a
tool; the domain-loop validator belongs to the `dnsqa` role). Result: **77 checks
PASS, 0 FAIL**, covering ER-01 to ER-10, strict key sets, unique service names,
bound-host TLS and the absence of secret material in the file. ER-11 is a
command contract and was checked by inspection, not by parsing the file.

The first validator run reported five failures because it used exact host
matching instead of the wildcard-aware matching defined in ER-10; after
correcting the validator to implement ER-10, all 77 checks pass. The manifest
itself needed no change.

---

## References

- `docs/rfc/RFC-0020-environment-and-secrets.md`
- `docs/rfc/RFC-0019-project-layered-architecture.md`
- `docs/rfc/RFC-0033-*` (domain-to-deploy contract, in preparation)
- `docs/dns/DOMAIN_LOOP.md` (in preparation by the `dnsarch` role)
- `docs/templates/TEMPLATE_DESIGN.md` sections 2, 3, 4, 6, 7 and 10
- `docs/adr/0008-*`, `docs/adr/0009-*`, `docs/adr/0011-*`
- `deploy/k8s/optional/ingress/ingress.yaml` (Ingress shape this policy binds to)
- `templates/edge-router/config.toml` (binding and redirect vocabulary)
- `CLAUDE.md` rules 12, 15, 16, 19, 22, 26 and 31
