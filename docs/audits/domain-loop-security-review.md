# Domain loop security review (wave 8)

- Reviewer role: dnssec (read-only audit; no push, no merge)
- Date: 2026-11-22
- Base commit reviewed: `9675ad8` (all `community8/*` branches were still at
  this tip at review time; no wave-8 commits were available to audit).
- Scope: the existing DNS/domain provisioning infrastructure that wave 8 builds
  on (`crates/ag-domains`, `crates/ag-edge`) plus the *designs* published in the
  wave-8 notes for `dnsapi`, `deplink`, `envrules`, `dnsarch`, `dnsqa` and
  `dnsdocs`.
- Method: read-only source inspection, repository content scans and string-logic
  reproduction. No files outside this role's worktree were edited.

## Scope and limitation

This review grades the security posture of the domain-to-deploy loop before it
ships. At the time of review every wave-8 branch (`dnsapi`, `deplink`,
`envrules`, `dnsarch`, `dnsqa`, `dnsdocs`) was at the base commit, so this
document audits (a) the pre-existing provisioning code the loop reuses and
(b) the stated designs in `/tmp/opencode/agents/w8-*.md`. Findings against
committed wave-8 code are necessarily deferred; the "Requirements per
component" section records the conditions each branch must satisfy, and the
recommendation is conditional on a re-audit of the committed tips. One real
defect was found in the pre-existing edge code and filed as issue #196.

## Threat model

- Provisioning: no credentials, tokens or zone ids in version control; provider
  APIs reached only through explicit, feature-gated adapters; no live provider
  or DNS call on the local/default path.
- Ownership: control of a domain must be proven before records are applied or
  traffic is served; proofs are unguessable and verified independently.
- Exposure: public visibility is explicit; private/default-deny is the default;
  wildcard hostnames are never generated implicitly.
- Plan/apply: `plan` is a reviewable artifact and `apply` is a real gate; a
  caller must not be able to apply without a reviewed plan or to delete records
  outside the managed scope.
- Transport: TLS is never claimed when disabled; ACME leaf keys are fresh per
  order and never logged or reused.

## Mission checklist

| Check | Result | Notes |
| --- | --- | --- |
| No credentials / zone ids committed | PASS | Repository-wide scan found no `CF_TOKEN`, `API_TOKEN`, `ZONE_ID`, password or key values; only placeholder env names in template docs. |
| No wildcard records by default | PASS | `instructions.rs` emits a wildcard CNAME only for an explicitly wildcard hostname; apex/subdomain output is A/AAAA/CNAME + ownership TXT. |
| Ownership verification before apply | PASS (library) / REQUIREMENT (CLI) | TXT proof at `_ag-domain.<registrable>` with UUID token; `NullVerifier` fails closed; activation requires `OwnershipStatus::Verified`. `dnsapi` must still gate `apply` on Verified status. |
| Private-by-default / default-deny visibility | PASS (design) / REQUIREMENT (CLI) | `envrules` design specifies `private` default, unknown -> private, no implicit public. Must be enforced by manifest validation. |
| No SSRF through user-supplied provider URLs | REQUIREMENT | Adapter `with_base_url`/`with_endpoints` accept arbitrary URLs (including `http://`) and attach API tokens. Safe as test seams; `dnsapi` must never expose them via config without https + allowlist. |
| Plan/apply cannot be bypassed | PASS (library) / REQUIREMENT (CLI) | `ZoneAdapter::apply` takes an explicit `ZonePlan`; `diff` is pure. `dnsapi` must not add a direct-apply shortcut and must scope `observed` before `diff`. |
| No real DNS calls in local/tests | PASS | Live paths are `#[ignore]` or wiremock/mock resolvers; `lookup_observed(Mx)` short-circuits before any lookup. |
| TLS expectation not falsely claimed | PASS (code) / REQUIREMENT (deploy link) | `is_active()` allows `TlsMode::Disabled` to be active, so status must label TLS as disabled, not "secure". `deplink` must keep TLS opt-in explicit. |
| ACME reuse does not leak keys | PASS | `renewal.rs` generates a fresh `KeyPair` per CSR; leaf keys held as rustls signing keys in `cert::CertStore`; no key logging. |
| Wildcard hostname matching boundary | FAIL | Issue #196: `strip_suffix(base)` without a dot boundary matches `notexample.com` for a `*.example.com` binding in `router.rs`, `tls.rs` and `cert.rs`. |

## Findings by component

### Baseline: `crates/ag-edge` (pre-existing, base `9675ad8`)

#### E1 (defect, p2, security) - issue #196: wildcard matching accepts non-subdomain hosts

`RouterCache::match_wildcard` (`router.rs`), `SniCertStore::select` (`tls.rs`)
and `CertStore::resolve` (`cert.rs`) all match a wildcard binding with
`sni.strip_suffix(base)` followed by a check that the remaining prefix has no
dot. For `base = "example.com"` and `sni = "notexample.com"`, `strip_suffix`
returns `"not"`, which passes the check. The same host also matches
`evil-example.com` and `fooexample.com`. Consequences: a wildcard route binding
can capture an unrelated hostname (host confusion / cross-tenant routing) and a
wildcard certificate can be selected for a non-subdomain SNI (wrong certificate,
handshake failure). Existing tests cover only `api.example.com` (match) and
`a.b.example.com` (no match), so they miss the missing-dot case.

Reproduction and verification criteria are in issue #196. This is a blocker for
exposing wildcard hostnames through the domain loop; it is not a wave-8
regression but becomes reachable once `dnsapi`/`deplink` route wildcard hosts.

### Baseline: `crates/ag-domains` (pre-existing, base `9675ad8`)

#### D1 (pass) - ownership and activation

`ownership.rs` publishes a per-attachment UUID token at
`_ag-domain.<registered-domain>`; `api.rs` defaults to `NullVerifier` (never
confirms) so ownership cannot be granted without a wired verifier.
`DomainAttachment::is_active` requires ownership `Verified`, DNS `Routable`,
routing `Ready` and TLS `Active` or explicitly `Disabled`. The four-dimension
state machine is a sound gate at the data layer.

#### D2 (pass) - plan/apply seam and deletion scope

`provider/sdk.rs` `diff` is pure and documented as authoritative only for the
scope that `observed` represents; callers must scope `observed` (the ACME
wildcard flow correctly restricts to `_acme-challenge` TXT records). `apply`
takes a `ZonePlan` and builds an inverse for rollback. No side effects in
`diff`, and no network in the SDK unit tests.

#### D3 (observation, low) - provider URL seam

`CloudflareProvider::with_base_url`, `Route53Provider::with_base_url`,
`GoogleCloudDnsProvider::with_endpoints`, `AzureDnsProvider::with_endpoints` and
`NamecheapProvider::with_base_url` accept arbitrary base URLs with no scheme
allowlist, and attach API credentials to requests. This is intentional for
wiremock tests, but it is a credential-exfiltration / SSRF primitive if a URL is
ever taken from user input. Requirement for `dnsapi`: provider endpoints must be
compile-time/operator-controlled constants or validated as `https://` with an
allowlist; never read from a project manifest, env var or CLI flag without that
validation.

#### D4 (observation, low) - Domain Connect settings URL and token disclosure

`domain_connect.rs::parse_settings` accepts any `urlSyncUX` string (no scheme or
host validation) and `build_sync_apply_url` embeds the ownership verification
token in the query string. A malicious or compromised settings response could
redirect the operator to an attacker host carrying the token. Discovery record
parsing is stricter (no scheme/path/whitespace) but does not enforce https.
Requirement: validate `urlSyncUX`/`urlAPI` as `https://` on a trusted host and do
not auto-fetch or auto-redirect in the local/default path.

#### D5 (observation, low) - unauthenticated REST control plane

`api.rs::build_router` exposes create/detach/verify with no auth middleware.
The feature is opt-in (`api`) and the local CLI uses the native store, so this
is not on the wave-8 default path. If the control plane is ever exposed on a
network, add authentication and bind to loopback.

#### D6 (observation, low) - private key in Debug-derived types

`acme::renewal::IssuedCert` and `issuance::IssuedCertificate` derive `Debug`
while holding `private_key_pem: String`. No current code logs these values, but
a future `Debug` format would print key material. Consider a redacted `Debug`.

### Component: `dnsarch` (design only)

No committed code. Design task is RFC-0033 + `docs/dns/DOMAIN_LOOP.md`.
Observation: the `dnsarch` note says "RFC-0033 (verify next free number)", while
the `deplink` note states RFC-0033 is already used by wave 7
(`RFC-0033-local-deploy-artifact-rendering.md`). The coordinator must resolve
the number collision at integration; the security contract itself is not
affected.

### Component: `dnsapi` (design only - highest risk)

Design: `ag dns plan|apply|status` and `ag domains attach` using `ag-domains`.
Requirements for a GO:

1. `plan` is a deterministic, printable artifact; `apply` accepts a plan id or
   recomputes and compares it, and rejects a stale/missing plan (no bypass).
2. `apply` refuses a hostname whose ownership status is not `Verified`.
3. Before `diff`, `observed` is scoped to records ag-domains manages; a
   full-zone read passed to `diff` would delete unrelated records.
4. No provider URL is taken from user input without https + allowlist (D3), and
   no live provider call on the local/default path; manual mode stays default.
5. No credentials or zone ids written to manifests, config or logs.

### Component: `deplink` (design only)

Design uses `schemaVersion` manifest with per-env host/TLS/visibility, `.test`
hosts, loopback-only compose binding and a JSON6902 Ingress patch. These are
sound: `.test` is reserved by RFC 6761, and no external proxy is required.
Requirements: keep TLS opt-in explicit; never render a "secure/HTTPS" claim when
`tls.enabled` is false; do not commit cert/key material into the generated
overlay; preserve the wildcard boundary fix (E1) if wildcard hosts are routed.

### Component: `envrules` (design only)

Design specifies environments local/dev/staging/prod, visibility
`private|vpn-only|public`, `private` default, unknown/missing -> private, and no
runtime selector variable. This satisfies default-deny. Requirements: the
manifest validator must reject unknown visibility values and must not infer
`public` from the presence of a host alone; `vpn-only` CIDR lists must be
validated; no private key in the manifest.

### Component: `dnsqa`, `dnsdocs`

No committed code. `dnsqa` must assert the E1 boundary cases, the ownership gate
and the no-real-DNS property in its local mock scenario. `dnsdocs` must not claim
TLS for disabled attachments and must document the SSRF/credential constraints
of D3/D4.

## Issues opened

- #196 - ag-edge wildcard hostname matching accepts non-subdomain hosts
  (p2, security, ag-edge). Reproduction and verification criteria in the issue.

## Recommendation

- Baseline provisioning code (`ag-domains`): GO for the domain loop provided
  the D3/D4/D5 constraints are respected by the CLI layer.
- Edge routing/TLS (`ag-edge`): NO-GO for wildcard hostnames until issue #196 is
  fixed and verified. Non-wildcard paths are unaffected.
- Wave-8 branches: DEFERRED. Designs are consistent with the security
  requirements above, but branch code was not available at review time. A
  follow-up audit of the committed tips is required before a final GO, with
  special attention to the `dnsapi` requirements 1-5.
