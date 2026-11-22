# RFC-0029 - Replace oauth2 5.0.0 with a native Authorization Code + PKCE client in ag-auth

- Status: proposed (blocking prerequisite for RFC-0026 and issue #185; no implementation authorized by this RFC)
- Author: Gravital Labs engineering
- Draft date: 2026-11-22
- Target phase: maintenance / dependency hygiene (post Phase 4.6)
- Affected crates: ag-auth only; workspace dependency table; indirectly issue #185 and RFC-0026
- Predecessor RFC: RFC-0026 (chrono replacement, deferred on this prerequisite)
- Comment period: minimum seven calendar days
- Language: English (ADR-0008)

## 1. Motivation

Issue #185 asks for the `RUSTSEC-2026-0543` (chrono is unmaintained) ignore to
be removed from `deny.toml` once no workspace crate depends on chrono. RFC-0026
selected `time` 0.3.x as the replacement but is explicitly deferred because
`oauth2 5.0.0` forces chrono into the graph through `ag-auth`. This RFC resolves
that one prerequisite so RFC-0026 can be executed.

If no decision is taken, the deny ignore remains permanent, the advisory edge
stays in every shipped binary, and issue #185 cannot be closed regardless of how
carefully the rest of the migration is performed.

## 2. Problem

### 2.1 Verified state of the oauth2 dependency

Evidence collected on this tree and from upstream on 2026-11-21/22:

- `crates/ag-auth/Cargo.toml:39` declares `oauth2 = { workspace = true }`; the
  workspace entry at `Cargo.toml:110` is `version = "5", default-features = false`.
- The published manifest of `oauth2 5.0.0` (local registry copy,
  `[dependencies.chrono]`) declares
  `chrono = { version = "0.4.31", default-features = false,
  features = ["clock", "serde", "std", "wasmbind"] }`. There is no feature gate
  and no optionality: `default-features = false` does not remove the edge.
- crates.io reports `max_stable_version`, `newest_version` and `max_version` all
  equal to `5.0.0`, published 2025-01-21. No lockfile update can remove chrono.
- Upstream `ramosbugs/oauth2-rs` has restructured `main` into `oauth2` and
  `oauth2-reqwest` workspace members. The unpublished `oauth2 5.1.0` manifest on
  `main` still contains the same non-optional chrono dependency.
- Upstream issue #351 and PR #355 ("Replace chrono with jiff", opened
  2026-11-18) are both open and unmerged. The maintainer stated on 2026-11-21
  that chrono is part of oauth2's public API (for example
  `TokenIntrospectionResponse::exp -> DateTime<Utc>`), so the migration needs a
  major version bump of both oauth2 and openidconnect, and that it is "not
  urgent as long as there are no known security issues in chrono". Work is
  planned "around the holidays". No date is committed.
- `cargo tree -i chrono` therefore keeps an `oauth2 5.0.0 -> ag-auth` edge no
  matter what our own crates do. This is the finding recorded in RFC-0026
  section 3.4 and in the issue comments.

### 2.2 The actually used oauth2 surface is very small

`crates/ag-auth/src/oauth.rs` imports only `BasicClient`, `AuthUrl`, `ClientId`,
`ClientSecret`, `CsrfToken`, `EndpointNotSet`, `EndpointSet`,
`PkceCodeChallenge`, `PkceCodeVerifier`, `RedirectUrl`, `Scope` and `TokenUrl`.
It uses oauth2 for exactly three things:

1. `PkceCodeChallenge::new_random_sha256()` plus `CsrfToken::new_random` to
   build the authorization URL with `code_challenge`, `code_challenge_method`,
   `state`, scopes and redirect URI.
2. Typed newtypes (`CsrfToken`, `PkceCodeVerifier`) passed back to the caller
   for session storage.
3. `BasicClient` as the URL/configuration holder.

The token exchange (`exchange_code`) and the user-info fetch
(`fetch_user_info`) are already implemented directly with `reqwest` and
`serde_json`; oauth2's HTTP client, token response types and refresh grant are
not used. No `oauth2::` symbol appears anywhere else in the workspace
(repo-wide search of `crates/**/*.rs`).

There is also no OAuth provider refresh flow to preserve. `ag-auth`'s refresh
support (`src/refresh.rs`, `RefreshBlacklist`) is internal JWT rotation with
`u64` timestamps and does not touch chrono or oauth2.

### 2.3 Supply-chain constraints from the repository contract

- `deny.toml` allows only crates.io registry sources (`allow-git = []`,
  `unknown-git = "warn"`), so a git fork must be justified explicitly.
- CLAUDE.md rules 12, 13 and 21 favour integrating minimal, explicit
  components over adding frameworks or speculative abstractions.
- ADR-0008 requires English for this RFC; ADR-0018 governs the caller-supplied
  reqwest client and the TLS provider installation used by the OAuth module.

## 3. Alternatives considered

### 3.1 Wait for a newer oauth2 release that dropped chrono

- Advantages: zero local change; upstream maintains the code.
- Disadvantages: no released version exists; upstream `main` (5.1.0) still has
  chrono; PR #355 is unmerged and the maintainer says chrono removal needs a
  public-API-breaking major bump; he has no committed date and does not
  consider the advisory urgent.
- Assessment: not available as an unblock. Keep as a long-term watch item: if
  a chrono-free oauth2 major is released before the native client ships, the
  committee may re-evaluate, but waiting is not a plan for issue #185.

### 3.2 Replace oauth2 with another permissively licensed client

Each candidate was checked by reading its published manifest/source, not by
reputation:

| Candidate | License | chrono path | Verdict |
| --- | --- | --- | --- |
| `openidconnect 4.1.0` | MIT | depends on `chrono` directly **and** on `oauth2` | rejected: does not remove the edge; adds a large OIDC stack |
| `oauth2-reqwest 0.1.0-alpha.3` | MIT | depends on `oauth2` | rejected: same edge, alpha quality; it is only an HTTP adapter for oauth2 |
| `oauth-client 0.8.0` | MIT OR Apache-2.0 | uses `time`, no chrono | rejected: badge says `passively-maintained`, no PKCE symbol found in its single-file `src/lib.rs`, pins `reqwest 0.11` and `ring`, incompatible with our reqwest 0.13/TLS setup |
| `oauth2-client 0.2.0` | Apache-2.0 OR MIT | no chrono; uses `oauth2-core 0.2` | rejected: last release 2023-03-13 (about 3.5 years stale), roughly 1.16k recent downloads/month, introduces six new crates (`oauth2-core`, `http-api-client`, `http-api-client-endpoint`, `dyn-clone`, `serde_qs`, `serde_urlencoded`) and its own HTTP abstraction that still must be adapted to our caller-supplied reqwest client; PKCE exists but the maintenance signal is worse than the problem |
| `oidc-rs 0.1.1` | MIT | no chrono | rejected: pre-1.0 OIDC-only crate; would be a full rewrite with little adoption |
| `axum-login 0.19.0` | MIT | no chrono | rejected: full session/login framework (tower-sessions, cookies, middleware); wrong abstraction level and large surface increase |
| `yup-oauth2 12.1.2` | MIT OR Apache-2.0 | uses `time`, no chrono | rejected: GCP-focused installed/service-account/device flows on hyper-rustls; no web authorization-code + PKCE path in `src`; would not preserve the Google/GitHub contract |

- Assessment: no maintained, permissive, small client covers the narrow surface
  we actually use. Replacing one dependency with a larger or staler one trades
  an advisory for supply-chain debt.

### 3.3 `[patch.crates-io]` or vendoring oauth2 without chrono

- Advantages: keeps the oauth2 API unchanged for our call sites; a path patch
  can be pinned and reviewed.
- Disadvantages: chrono is in oauth2's public API, so a chrono-free fork is an
  API-breaking fork (`TokenIntrospectionResponse::exp` and related traits),
  not a drop-in. A git patch would need an explicit `allow-git` entry and a
  pinned revision, and would make builds depend on a third-party branch. A
  vendored copy means we own upstream security fixes for a 6.5k-line HTTP/auth
  crate. RFC-0026 option B already identified this as supply-chain burden.
- Assessment: rejected. It removes the advisory edge on paper while keeping
  the maintenance problem, and contradicts the simplicity and minimality
  rules.

### 3.4 Make the oauth2 dependency optional and off by default

- Advantages: the default `cargo tree -i chrono` and the default `cargo deny`
  graph could become chrono-free without code changes.
- Disadvantages: `cargo deny --all-features check` (part of the RFC-0026
  verification) would still fail; Google/GitHub login would silently disappear
  from default builds; "no workspace crate depends on chrono" would be true
  only by wording, not in the artifact we ship when the feature is enabled.
- Assessment: rejected. It games the gate rather than fixing the problem.

### 3.5 Native, narrow Authorization Code + PKCE client in ag-auth

- Advantages: zero new dependencies (all primitives are already direct
  dependencies of ag-auth); removes the oauth2/chrono edge completely; keeps
  token exchange and user-info code exactly as it is; the logic is roughly one
  hundred lines and can be covered by RFC 7636 known-answer tests; no new
  license or source policy exception.
- Disadvantages: we own PKCE/CSRF correctness and must keep up with provider
  endpoint changes; public newtypes in `ag-auth` change.
- Assessment: **recommended**. See section 4.

## 4. Proposed design (recommendation: 3.5)

No production code or manifest changes are part of this RFC. This section is
the contract a later implementation PR must satisfy.

### 4.1 Dependency changes

- Remove `oauth2 = { workspace = true }` from `crates/ag-auth/Cargo.toml`.
- Remove `oauth2 = { version = "5", default-features = false }` from
  `[workspace.dependencies]` in the root `Cargo.toml`, and drop the two
  `oauth2` entries from `Cargo.lock`.
- Add no new dependency. The implementation uses primitives already present
  in ag-auth:
  - `base64ct 1.x` with the existing `alloc` feature, specifically
    `Base64UrlUnpadded` (verified present in base64ct 1.8.3);
  - `sha2` (workspace 0.11) for `Sha256`;
  - `getrandom` (workspace 0.4) for CSPRNG bytes;
  - `url` (workspace 2) for `Url::parse_with_params`;
  - `reqwest` 0.13 (caller-supplied client; process-default ring provider
    installed per ADR-0018) and `serde_json`, already used by `exchange_code`
    and `fetch_user_info`;
  - `thiserror` for the existing `OAuthError`.
- The workspace `chrono` entry stays until RFC-0026 is executed; this RFC only
  removes the oauth2 edge.

### 4.2 Public API changes

Replace the three oauth2 newtypes exposed by `ag-auth` with local ones:

| Current | Proposed | Notes |
| --- | --- | --- |
| `oauth2::CsrfToken` in `authorization_url` return | `OAuthState` (secret newtype, `expose_secret() -> &str`) | Same 128-bit random value; caller stores it in the session |
| `oauth2::PkceCodeVerifier` in `authorization_url` return and `exchange_code` argument | `PkceVerifier` (secret newtype, `expose_secret() -> &str`) | Same 32-byte-derived verifier, single use by convention |
| `oauth2::basic::BasicClient` field | private provider configuration (endpoint URLs and credentials) | Internal only, never public |

`OAuthProvider`, `OAuthUser`, `OAuthError` and the method names
`authorization_url` / `exchange_code` remain unchanged. `OAuthError::OAuth`
stays for URL-construction errors so existing match arms keep compiling.

`ag-auth` is `publish = false` and no crate outside `ag-auth` references these
types, so the break is contained. The implementation PR must still record it
as an API change in the CHANGELOG.

### 4.3 Compatibility contract

This is the part that must not drift. All values below match what oauth2 5.0.0
produces today (verified by reading `oauth2-5.0.0/src/types.rs` lines 435-498
and 554-574).

#### 4.3.1 PKCE (RFC 7636, S256 only)

- Generate 32 cryptographically random bytes with `getrandom`.
- `verifier = base64url_nopad(random_32_bytes)`; this is exactly 43 characters,
  inside the RFC 7636 43..=128 range, and uses only `A-Z a-z 0-9 - _`.
- `challenge = base64url_nopad(SHA256(ASCII(verifier)))`.
- Query parameters: `code_challenge=<challenge>` and
  `code_challenge_method=S256`.
- No `plain` method: the `pkce-plain` feature was never enabled.
- Known-answer test (RFC 7636 Appendix B) must be committed before/alongside
  the change: verifier
  `dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk` produces challenge
  `E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM`.
- Additional tests: verifier is 43 chars and URL-safe; two consecutive calls
  produce different verifiers and challenges (existing test already checks
  verifier uniqueness); generated authorization URL contains
  `code_challenge_method=S256` and the expected scopes.

#### 4.3.2 CSRF state (RFC 6749 section 10.12)

- Generate 16 cryptographically random bytes (128 bits).
- `state = base64url_nopad(random_16_bytes)` (22 characters, no padding).
- The value is returned to the caller, which persists it in the session and is
  responsible for comparing it on callback, exactly as today. The native
  client must not silently verify or drop it; changing caller responsibility
  is out of scope for this RFC.
- Tests: 22-character URL-safe value, uniqueness across calls.

#### 4.3.3 Authorization endpoint

- Google: `https://accounts.google.com/o/oauth2/v2/auth`, scopes
  `openid email profile`.
- GitHub: `https://github.com/login/oauth/authorize`, scopes
  `read:user user:email`.
- Parameters: `client_id`, `redirect_uri`, `response_type=code`, `scope`
  (space-separated), `state`, `code_challenge`, `code_challenge_method`.
- Encoding must be application/x-www-form-urlencoded-compatible via
  `Url::parse_with_params`; a golden-string test pins the full query for a
  fixed endpoint, client id, redirect URI and injected state/verifier.

#### 4.3.4 Token endpoint

- Google: `https://oauth2.googleapis.com/token`.
  GitHub: `https://github.com/login/oauth/access_token`.
- POST with `Accept: application/json` and form body, in the current order:
  `grant_type=authorization_code`, `code`, `redirect_uri`, `code_verifier`,
  `client_id`, `client_secret`.
- Success: parse JSON, require a non-empty `access_token` string; then call
  `fetch_user_info`. Non-2xx: return `OAuthError::InvalidResponse` with the
  response body. Transport errors: `OAuthError::Http`. Missing fields:
  `OAuthError::InvalidResponse`. These mappings stay unchanged.
- A local mock-server test must assert method, path, headers and the exact
  form body so the wire contract is verified without live provider calls.
- User-info endpoints and parsing (`https://www.googleapis.com/oauth2/v2/userinfo`
  and `https://api.github.com/user` with its `User-Agent`) remain untouched.

#### 4.3.5 Refresh

- There is no provider refresh-token grant in the current implementation:
  `exchange_code` discards `refresh_token` from the token response and returns
  only `OAuthUser`.
- `ag-auth`'s `RefreshBlacklist` is internal JWT rotation and is unaffected by
  this RFC; its `u64` timestamps must not be converted to RFC 3339 strings.
- Any future OAuth provider refresh flow requires its own RFC. The native
  client neither adds nor removes refresh capability.

### 4.4 Tests required in the implementation PR

- RFC 7636 Appendix B known-answer vector (section 4.3.1).
- Golden authorization URL string and golden token-request body.
- Mock token endpoint: success, non-2xx, malformed JSON, missing
  `access_token`, transport failure.
- Existing `authorization_url` tests (Google, GitHub, unconfigured provider,
  verifier uniqueness) must keep passing.
- `cargo tree -i chrono` must no longer list `oauth2 5.0.0 -> ag-auth`. The
  remaining ag-domains/ag-workers/sqlx edges are expected until RFC-0026 is
  executed.
- No live provider credentials or network calls in CI; provider behaviour
  checks stay `#[ignore]` and manually documented.

### 4.5 Documentation and files touched by the implementation PR

- `crates/ag-auth/src/oauth.rs`, `crates/ag-auth/Cargo.toml`, root
  `Cargo.toml`, `Cargo.lock`.
- `crates/ag-auth/README.md`: OAuth section and the stale chrono expression at
  line 23.
- `CHANGELOG.md`: dependency removal, API change, issue reference.
- RFC-0026 section 3.4 gets a follow-up note that its prerequisite has a
  decision; `deny.toml` keeps the dated ignore until all other chrono edges
  are gone (RFC-0026 steps 2-7).

No DSL grammar, master document, phase, crate roster or diagram changes are
proposed. No new ADR is needed: the decision is an RFC-level dependency choice.

## 5. Migration steps

1. **This PR (design only).** Land RFC-0029 and its documentation references.
   Issue #185 stays open with `Refs #185`; no manifest or production code
   changes.
2. **Implementation PR (after acceptance).** Introduce the native client,
   add the tests in 4.4, remove the oauth2 dependency. Verify the oauth2 edge
   disappears from `cargo tree -i chrono`. This PR still does not close #185.
3. **RFC-0026 execution PRs (steps 2-7).** Swap sqlx feature and workspace
   types to `time`, migrate ag-workers and ag-domains, add golden serde and
   TIMESTAMPTZ round-trip tests, decide DSL codegen, then remove the
   `RUSTSEC-2026-0543` ignore from `deny.toml`.
4. **Close #185** only when `cargo tree -i chrono` is empty,
   `cargo deny --all-features check advisories` passes without the ignore,
   and `cargo test --workspace --all-features` passes. This RFC alone does not
   satisfy those criteria.

Step 2 must not start before the comment period and committee decision on
this RFC. Review by seguridad is required before merging step 2 because the
change touches PKCE and CSRF token generation.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Incorrect PKCE derivation or encoding | low | high (login breaks or PKCE weakened) | RFC 7636 Appendix B known-answer test, golden URL test, mock token endpoint asserting `code_verifier`; security review |
| CSRF state handling weakened by the new newtype | low | high | Keep 16 CSPRNG bytes and base64url-no-pad; caller responsibility unchanged; uniqueness/length tests |
| Provider-specific request differences (GitHub token response shape) | low | medium | Keep the existing hand-written form body and user-info parsing byte-for-byte; mock-server tests; no new abstraction over providers |
| Public API break in `ag-auth` | medium | low | `publish = false`; repo-wide search shows no external callers; CHANGELOG and migration note in the implementation PR |
| Hidden oauth2 usage reappears in generated code | low | medium | Repo-wide grep gate in the implementation PR; no DSL/codegen change in this RFC |
| Upstream ships a chrono-free oauth2 first | low | low | Re-evaluate at the committee; the native client remains a valid, smaller solution and can still be kept |
| Security review delays the implementation | medium | medium | Schedule seguridad review with the implementation PR, not after it |

## 7. Impact

- **Scope:** no product feature changes; dependency and auth-internals only.
- **Roadmap:** no phase change. Maintenance work outside the phase plan.
- **Issue #185:** removes the oauth2 edge, which is the blocking prerequisite,
  but does not by itself close the issue (RFC-0026 work remains).
- **RFC-0026 / RFC-0025 / data-layer note:** consistent with the selected
  `time` replacement. The native client introduces no second date/time type
  and no competing timestamp representation, so it does not conflict with the
  DSL `Timestamp` mapping or the generated-model serde contract. The
  `chrono::DateTime<Utc>` -> `time::OffsetDateTime` decision in RFC-0026 and
  the golden-string constraints in `docs/modules/ag-data/data-layer-rls-tenancy.md`
  are unchanged by this RFC.
- **Operations:** one fewer unmaintained-dependency edge in shipped binaries;
  smaller dependency surface; no configuration or deployment change.
- **Public APIs:** `authorization_url` and `exchange_code` signatures change
  from oauth2 newtypes to local newtypes.
- **Documentation:** this RFC, RFC index, CHANGELOG; later the ag-auth README,
  RFC-0026 status note and (at the very end) `deny.toml`.

## 8. Rollback

Revert the implementation PR: restore the `oauth2` dependency and the original
`oauth.rs`, then regenerate `Cargo.lock`. No data migration is involved because
the token and user-info wire formats are unchanged. Rollback signals: any
failing PKCE known-answer or golden request test, provider 400/401 responses
in a staging check, or a `cargo tree -i chrono` regression that reintroduces
an unexpected edge. After rollback, the dated deny ignore stays and #185 stays
open. Reverting this RFC document alone has no runtime effect.

## 9. Decision

Deferred pending the seven-day comment period and committee decision. The
explicit recommendation of this RFC is to adopt the native Authorization Code
+ PKCE client (section 3.5) as the unblocking path for issue #185, with the
compatibility contract in section 4.3 treated as non-negotiable and security
review required before implementation merges. Waiting for upstream oauth2 is
tracked but is not an actionable unblock.

- Decisor: technical committee (post Phase 4).
- Decision date: pending.
- Result: pending.
- Issue state: #185 remains open; this PR uses `Refs #185`, not `Closes #185`.

## 10. References

- Issue #185: https://github.com/Anti-Gravital/Anti-Gravital/issues/185
- RFC-0026 (chrono replacement, deferred): `docs/rfc/RFC-0026-chrono-replacement.md`
- RFC-0025 and the data-model note:
  `docs/rfc/RFC-0025-ag-data-orm-rls.md`,
  `docs/modules/ag-data/data-layer-rls-tenancy.md`
- RUSTSEC-2026-0543: https://rustsec.org/advisories/RUSTSEC-2026-0543
- oauth2 crate: https://crates.io/crates/oauth2/5.0.0
- Upstream chrono migration: https://github.com/ramosbugs/oauth2-rs/issues/351
  and https://github.com/ramosbugs/oauth2-rs/pull/355
- RFC 7636 (PKCE): https://www.rfc-editor.org/rfc/rfc7636
- RFC 6749 section 10.12 (state/CSRF): https://www.rfc-editor.org/rfc/rfc6749
- ADR-0008 (language), ADR-0018 (TLS provider / caller-built HTTP client)
