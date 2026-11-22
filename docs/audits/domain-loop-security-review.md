# Domain loop security review (wave 8)

- Reviewer role: dnssec (read-only audit; no push, no merge)
- Date: 2026-11-22
- Base commit reviewed: `9675ad8`, rebased onto main `001e956` (envrules merged)
  and cross-checked against the unmerged dnsarch tip `4e96e4d`.
- Scope: the existing DNS/domain provisioning infrastructure that wave 8 builds
  on (`crates/ag-domains`, `crates/ag-edge`), the merged `envrules`, `dnsarch`,
  `deplink` and `dnsdocs` work, the dnsapi worktree (uncommitted at review), and
  the design published in the `dnsqa` note.
- Method: read-only source inspection, repository content scans and string-logic
  reproduction. No files outside this role's worktree were edited.

## Scope and limitation

This review grades the security posture of the domain-to-deploy loop before it
ships. It audits (a) the pre-existing provisioning code the loop reuses,
(b) the merged `envrules`, `dnsarch`, `deplink` and `dnsdocs` work, (c) the
uncommitted `dnsapi` CLI worktree and (d) the `dnsqa` note. Two real defects were
found: issue #196 in pre-existing edge code and issue #197 in the uncommitted
dnsapi CLI. Findings are grounded in observed code at the commit tips recorded
per component; the `dnsapi` finding must be re-verified on its committed tip and
`dnsqa` must be audited after commit before a final whole-loop GO.

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
| No wildcard records by default | PASS (library) / FAIL (dnsapi attach) | `instructions.rs` emits a wildcard CNAME only for an explicitly wildcard hostname. However dnsapi's `ag domains attach` skips its opt-in when no manifest is passed (issue #197). |
| Ownership verification before apply | PARTIAL | TXT proof at `_ag-domain.<registrable>` with UUID token; `NullVerifier` fails closed; activation requires `OwnershipStatus::Verified`; tombstone blocks takeover. `ag dns apply` does not create/verify ownership proofs itself - it delegates zone control to provider credentials (token + zone id) and keeps parallel local state. See A1 below. |
| Private-by-default / default-deny visibility | PASS (merged design) / REQUIREMENT (validator) | `envrules` (merged `f22ddb1`) specifies `private` default, unknown -> private, no implicit public, TLS required for public. Must be enforced by the manifest validator. |
| No SSRF through user-supplied provider URLs | PASS (dnsapi) / REQUIREMENT (library) | dnsapi accepts only provider names `none`/`cloudflare` and calls `CloudflareProvider::new` with the hardcoded production endpoint; no user-supplied URL. The library seam (D3) must still never be exposed to config. |
| Plan/apply cannot be bypassed | PASS (dnsapi) | `cmd_dns_apply` requires a non-empty `--plan` id, recomputes the plan and aborts on drift before any write or provider call. `ZoneAdapter::apply` also takes an explicit `ZonePlan`. |
| No real DNS calls in local/tests | PASS | Default provider is `none`; plan/status are filesystem-only. Live paths are `#[ignore]` or wiremock/mock resolvers; `lookup_observed(Mx)` short-circuits before any lookup. |
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

### Component: `dnsapi` (uncommitted worktree, read-only review)

Reviewed `crates/ag-cli/src/commands/dns.rs` and the `domains.rs`/`cli.rs`/
`main.rs` diffs at review time. Strong points: manifest structs use
`deny_unknown_fields` (a test proves a canary `token`/`zone_id` field is
rejected); token comes only from `AG_CLOUDFLARE_TOKEN` and zone id from
`--zone-id`/`AG_DNS_ZONE_ID`, never written to a file (state JSON and plan id
contain no secrets); provider is an allowlist of `none`/`cloudflare` with a
hardcoded endpoint (no SSRF, D3 satisfied); `apply` requires and revalidates a
plan id before any write; default provider `none` means plan/apply/status are
fully offline; wildcard manifest records/domains are rejected without
`allow_wildcard = true`; `domains attach` checks tombstones and creates an
unguessable ownership token with `Pending` status.

#### A1 (defect, p2, security) - issue #197: wildcard opt-in bypass on `domains attach`

The wildcard gate in `cmd_domains_attach` is
`hostname.starts_with("*.") && plan.is_some() && !allow_wildcard`. When
`--manifest` is not supplied, `plan` is `None` and the check is skipped, so
`ag domains attach '*.example.com' ...` succeeds with `--allow-wildcard` still
false at its default. The resulting attachment emits a wildcard CNAME and sets
`TlsMode::ManagedDns01` with no explicit opt-in. This violates the "no wildcard
by default" contract and is the surface made dangerous by issue #196. Filed
with reproduction and verification criteria.

Re-check: after dnsapi refactored its worktree, the condition was renamed from
`plan.is_some()` to `hints.is_some()` but `hints` is still derived solely from
the `--manifest` path, so the bypass remains. The manifest path was improved
(wildcards now also require `tls = managed_dns01`), but the no-manifest attach
path is still ungated; issue #197 remains open.

#### A2 (observation, p3) - partial ownership integration and partial provider writes

`ag dns apply` writes declared records directly through the Cloudflare CRUD
adapter into `.ag/dns/state.json`; it does not create an ag-domains attachment,
an ownership TXT proof, or consult `OwnershipStatus`. Zone control is delegated
to the provider token + zone id, so a caller cannot write outside zones their
credential administers; this prevents classic zone takeover but diverges from
RFC-0033 section 4.7 ("ownership proof before activation"). This parallel state
is also what the dnsarch note warned against. Additionally, `apply_provider`
performs mutating CRUD calls without the SDK inverse/rollback: a mid-plan
failure leaves partial zone changes with no state file; it is idempotent on
retry, so this is an availability/consistency observation, not a blocker.
Recommendation: reconcile the resolved binding with the canonical `ag.toml`
contract and route provider writes through the reviewed `ProviderAdapter`
seam, or document why the local-config path intentionally differs.

### Component: `dnsarch` (committed, tip `4e96e4d`)

RFC-0033 section 4.7 defines a sound security contract: no credentials in the
repository (env var names / `Secret<T>` only, redacted output), no wildcard by
default (`allow_wildcard = false`, DNS-01 required), fail-closed (`NullVerifier`
default, stale plan id aborts apply, link targets must be HTTPS on a host
allowlist - an explicit SSRF guard), and zone-takeover prevention via ownership
proof plus a 30-day tombstone. Sections 4.5/4.6 keep private bindings free of
public records and the validation/plan/rendering path offline. This satisfies
the audit requirements at the design level.

Observation (RFC numbering, resolved): the `deplink` note claimed RFC-0033 was
taken by wave 7 (`RFC-0033-local-deploy-artifact-rendering.md`). `dnsarch`
verified no such file or git history exists and that 0032 was the highest
number. The coordinator has since renumbered the local deploy RFC to 0034
(main `8a90b46`), leaving RFC-0033 for the domain loop. No collision remains.

### dnsapi requirement disposition (carried over from design review)

1. Deterministic printable plan, `--plan` id revalidated at apply - SATISFIED.
2. Refuse apply when ownership is not Verified - NOT IMPLEMENTED in `ag dns
   apply` (see A2; provider credential auth is the only gate).
3. Scope `observed` before `diff` - N/A: dnsapi uses CRUD matching, not `diff`,
   so unrelated records are not deleted; this avoids the footgun but also
   bypasses the reviewed SDK seam (A2).
4. No user-supplied provider URL / no live provider by default - SATISFIED.
5. No credentials or zone ids in files - SATISFIED.

### Component: `deplink` (merged at `5c3bc48` / `a3e4add`)

Reviewed `deploy/link/render_ingress.py` and `examples/domain-link/domain.yaml`.
Strong points: strict schema keys, `private` defaults and unknown-visibility
hard errors; ER-08 enforced (a `private` service never gets an Ingress rule even
with a host); `public` requires TLS, ingress and an internet CIDR; `vpn-only`
requires non-empty non-internet sources; TLS Secret is a name only; URL scheme
and the `TLS=true|false` output follow the actual config, so HTTPS is not
falsely claimed; hosts use RFC 2606 `.invalid` names. Notably its
`host_allowed` one-label wildcard check uses `endswith("." + zone)` with a dot
count, so it does NOT have the issue #196 boundary bug.

#### P1 (observation, p3) - vpn-only source ranges are annotation-only

For `vpn-only`, `render_ingress.py` writes the CIDRs only to a custom
`anti-gravital.io/allow-sources` annotation and states in `deploy/link/README.md`
that the annotation does not itself enforce access; the ingress controller or a
NetworkPolicy must. This is honestly documented, so it is not a silent
fail-open, but operators must be aware that a rendered `vpn-only` Ingress is not
network-restricted by the generated artifacts alone. No issue filed; record as a
deployment caveat.

### Component: `envrules` (merged into main at `f22ddb1`)

Reviewed `docs/dns/ENVIRONMENT_RULES.md` and the example manifest. The design is
default-deny and machine-checkable: `default_visibility` and every
`visibility_default` must be `private` (ER-03), missing/unknown visibility
resolves to private and unknown values are a hard error (ER-04), `public`
requires host + TLS + ingress + an explicit internet CIDR (ER-05/ER-07),
`vpn-only` maps to private plus a CIDR allowlist (ER-07/4.8), private keys never
appear in the manifest (ER-SECRET), and `--env` is explicit with no runtime
selector variable (ER-11). The example uses `*.invalid` documentation hosts and
no real credentials. This satisfies default-deny at the design level.

Observation (low): the example permits single-label wildcards in dev and staging
(`allow_wildcard: true`). That is intentional, but it is exactly the surface
exposed by issue #196; the wildcard boundary fix must land before any such
binding reaches a real edge. No defect in the envrules files themselves.

Requirement for implementers: the manifest *validator* (not just the document)
must enforce ER-01..ER-11 - reject unknown visibility, bound `CIDR` syntax,
require TLS for public, and never infer public from a host alone.

### Component: `dnsdocs` (merged at `8da264f`)

Added `docs/dns/README.md` and manual/INDEX links. Scanned for overclaiming:
no "TLS always/secure by default/HTTPS-only" assertions; TLS is described as
environment-dependent. No credential or zone-id values present. No defect.

### Component: `dnsqa` (committed at `01bb297`, unmerged)

Reviewed `scripts/validate-domain-loop.sh` and `docs/audits/domain-loop-validation.md`.
The harness is honest: it reports PASS/FAIL/SKIP, treats a missing command as
FAIL (never SKIP), labels its fake-`ag` self-test as harness-only, records a
baseline FAIL run, and builds the authoritative binary from the dnsapi worktree
with a real no-network `bwrap` sandbox (12 PASS, 0 FAIL, 0 SKIP). Its credential
leak scan searches captured output for sentinel tokens and provider API
endpoints; plan ids and apply gates (missing/bogus plan id) are asserted. This
is credible evidence for the offline path.

#### Q1 (observation, p3) - missing wildcard and ownership assertions

The harness does not assert the #196 wildcard boundary, the #197
`--allow-wildcard` gate, or the ownership/pending-ownership status. Recommend
adding those checks so the QA gate covers the two filed defects and the
takeover-protection contract. No issue filed (coverage gap, not a false claim).

## Issues opened

- #196 - ag-edge wildcard hostname matching accepts non-subdomain hosts
  (p2, security, ag-edge). Reproduction and verification criteria in the issue.
- #197 - dnsapi `ag domains attach` accepts wildcard hostnames without the
  opt-in flag when no manifest is used (p2, security, ag-cli). Reported against
  the uncommitted dnsapi worktree; re-verify on the committed tip before close.

## Recommendation

- Baseline provisioning code (`ag-domains`): GO for the domain loop provided
  the D3/D4/D5 constraints are respected by the CLI layer.
- Edge routing/TLS (`ag-edge`): NO-GO for wildcard hostnames until issue #196 is
  fixed and verified. Non-wildcard paths are unaffected.
- Merged wave-8 design (`envrules`): GO with the validator requirement above.
- Committed design (`dnsarch`): GO as a security contract.
- `dnsapi`: NO-GO until issues #196 and #197 are fixed and verified; then
  GO-with-follow-up on A2 (ownership/state reconciliation). Everything else in
  the reviewed code satisfies the provisioning, SSRF, plan/apply and offline
  checks.
- Merged `deplink` and `dnsdocs`: GO with the P1 deployment caveat above.
- Committed `dnsqa`: GO with the Q1 coverage-gap recommendation. Its evidence
  is honest and reproducible; it is not a substitute for the #196/#197 fixes.
- Whole-loop final GO remains blocked by #196 and #197.
