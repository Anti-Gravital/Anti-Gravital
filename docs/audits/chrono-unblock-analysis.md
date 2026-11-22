# Chrono unblock analysis - object_store via ag-storage/s3 (issue #185)

- Date: 2026-11-22
- Scope: wave 11 gate-closure analysis, branch `community11/chrono`.
- Language: English (ADR-0008).
- Outcome: NO safe fix is available in this iteration. No code, manifest or
  lockfile change is made. The dated `RUSTSEC-2026-0543` ignore in `deny.toml`
  stays with its removal condition. Issue #185 stays OPEN; this work uses
  `Refs #185`, never `Closes #185`.

## 1. Verified current state

All commands were run in the worktree with the shared Cargo target dir and
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`.

### 1.1 The only remaining chrono edge

```
$ cargo tree --workspace --all-features -i chrono
chrono v0.4.45
+-- object_store v0.14.2
    +-- ag-storage v0.1.0-alpha.2 (.../crates/ag-storage)
        [dev-dependencies]
        +-- ag-integration-tests v0.1.0-alpha.2 (.../tests/integration)
```

Every path from the earlier waves (oauth2 via ag-auth, ag-workers, ag-domains,
sqlx `chrono` feature, DSL codegen, workers-scheduled) is gone. Exactly one
edge remains, and it is upstream of our code: `object_store -> chrono`.

### 1.2 Deny passes only because of the ignore

```
$ cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok      (exit 0)
```

With a temporary copy of `deny.toml` whose `RUSTSEC-2026-0543` ignore block is
removed (the repository file was NOT modified), the same command fails:

```
$ cargo deny --config /tmp/opencode/chrono-noignore-deny.toml \
      --all-features check advisories
error[unmaintained]: chrono is unmaintained
...
| Solution: No safe upgrade is available!
| chrono v0.4.45
  +-- object_store v0.14.2
      +-- ag-storage v0.1.0-alpha.2
          +-- (dev) ag-integration-tests v0.1.0-alpha.2
advisories FAILED                                   (exit 1)
```

### 1.3 Audit state

```
$ cargo audit
warning: 4 allowed warnings found                    (exit 0)
```

The four warnings are 2023-0071, 2024-0436, 2026-0440 and 2026-0543 (chrono).
The advisory class for 2026-0543 is `INFO / Unmaintained`, issued
2026-11-17, with "no patched versions".

### 1.4 Why object_store cannot simply be updated

Verified against the crates.io API and the local sparse index cache:

- `object_store` latest, newest, max stable and default version are all
  `0.14.2`, published 2026-09-15. No newer release exists.
- Every release from 0.6.1 through 0.14.2 declares chrono in its `cloud` /
  `cloud-base` feature (`chrono/serde`), and 0.14.2 declares it as a
  **non-optional** `[dependencies.chrono]` (`0.4.34`, features `clock`,
  `default-features = false`). No released version is chrono-free.
- Upstream `apache/arrow-rs-object-store` issue #909 "RUSTSEC-2026-0543:
  chrono is unmaintained" is OPEN (created 2026-11-17), unassigned, with no
  milestone and no linked PR. The only comment states that chrono is part of
  the public API (`ObjectMeta::last_modified`,
  `GetOptions::if_modified_since` / `if_unmodified_since` and their builder
  methods), so removal needs a **breaking release**; a contributor has an
  unmerged local branch but no PR was opened and no maintainer commitment
  exists. PR #908 in that repository concerns `humantime` (RUSTSEC-2026-0542),
  not chrono, and is also open.

Conclusion: an upstream fix is an external gate, not an available update.

## 2. Options evaluated

### 2.1 Newer object_store release - RULED OUT

See section 1.4. There is no newer release, and upstream removal requires a
breaking major/minor release that has not happened.

### 2.2 Alternative object_store feature set - RULED OUT (empirical)

The manifest marks chrono non-optional, so no feature combination removes it.
This was verified with a throwaway crate under `/tmp/opencode/chrono-featuretest`
containing only:

```
object_store = { version = "0.14.2", default-features = false }
```

With no `aws`, `cloud` or other cloud feature enabled, the dependency graph is
still:

```
chrono v0.4.45
+-- object_store v0.14.2
    +-- chrono-featuretest v0.1.0
```

Therefore even dropping the `aws` feature of `ag-storage` would not remove the
chrono edge, and would also delete the adapter's functionality.

### 2.3 `[patch.crates-io]` / locally vendored chrono-free object_store - RULED OUT (unsafe)

- `chrono::DateTime<Utc>` is in object_store's public API, not just internal
  code: `ObjectMeta::last_modified`, `GetOptions` conditional-request fields
  and their `with_if_*` builders. Removing chrono is an API-breaking fork, not
  a drop-in substitution.
- chrono appears in 18 object_store source files (HTTP date parsing, SigV4
  and SAS timestamps, token expiry, Azure local-time `expiresOn`). The crate
  is about 27k lines across 59 files.
- We would own all future upstream security fixes for a forked HTTP/auth
  client. That is supply-chain debt and, in substance, an unaudited dependency
  we maintain ourselves.
- `deny.toml` allows only the crates.io registry (`allow-git = []`); a path
  patch is source-legal but still creates an untracked local fork, and a git
  patch would need an explicit source exception.
- This is exactly the option RFC-0026 section 3.4 (option B) and RFC-0029
  section 3.3 rejected for oauth2: it removes the advisory edge "on paper"
  while keeping the maintenance problem. It also contradicts CLAUDE.md rules
  12 (integrate, do not reinvent), 13 (minimal dependency/runtime), 21 (no
  unnecessary complexity) and 30 (eliminate over add).

### 2.4 Replace the S3 adapter with a native implementation - technically possible but NOT SAFE here

For completeness, this is the only option that could make the tree empty
without upstream help. A native S3 client behind the existing opt-in `s3`
feature (keeping the filesystem default) would NOT weaken ADR-0009's
native-default/adapter-opt-in rule, and it could reuse already-audited
workspace crates (`hmac`, `sha2`, `base64ct`, `reqwest`), so it would not add
an unaudited dependency on those two specific constraints.

It is nevertheless rejected for this iteration for four independent reasons:

1. **Governance (blocking).** No RFC or ADR designs a native S3 client; no
   RFC/ADR in `docs/rfc/` or `docs/adr/` authorizes removing `object_store`.
   The S3/MinIO adapter and its `object_store` dependency are documented scope
   in `README.md`, `crates/ag-storage/README.md` and
   `docs/modules/ag-storage/README.md`. CLAUDE.md rules 0, 5, 22 and 28
   require documentation and an accepted RFC before a stack/crate change.
   RFC-0026 and RFC-0029 are both still marked "proposed", not accepted.
   Implementing an undesigned, security-sensitive subsystem to close a gate
   would be architecture improvisation, which the repository contract forbids.
2. **Verifiability (blocking).** The only S3 test,
   `S3Store::s3_put_get_roundtrip_with_minio`, is async and returns early
   unless `AWS_ACCESS_KEY_ID` or `MINIO_URL` is set. There is no live S3/MinIO
   in CI, so a hand-written SigV4 signer, XML `ListObjectsV2` parser,
   pagination and error mapping could not be verified against a real server.
   CLAUDE.md rule 18 requires fuzz/regression/integration evidence for a
   critical path; that evidence cannot be produced here.
3. **Security (high risk).** We would replace an Apache-2.0, widely deployed
   Rust S3 implementation with bespoke signing and HTTP-date/timestamp code.
   A single canonicalization or header-order mistake breaks authentication or
   weakens request integrity. Rule 9 ("seguridad por construccion") and rule
   12 (integrate dominant tools) point the other way.
4. **Risk trade is unfavourable.** RUSTSEC-2026-0543 is an `INFO /
   Unmaintained` advisory with no known exploit and no patched version,
   already mitigated by a dated, justified ignore. Trading it for unverified
   security-sensitive code increases total risk instead of reducing it.

A native replacement, if the committee wants one, therefore needs its own
accepted RFC, a security review, AWS SigV4 known-answer tests and a mock/local
S3 integration test suite, followed by the normal green gate. It is not a
minimal gate-closure change.

## 3. Exact blocker

`ag-storage`'s opt-in `s3` feature depends on `object_store 0.14.2`, whose
non-optional chrono dependency (and chrono-typed public API) cannot be removed
by a lockfile update, a feature flag, or any released version. Upstream removal
requires a breaking release that does not exist yet. Removing the dependency
locally means either an API-breaking fork or a from-scratch S3 client, both of
which need an accepted RFC and verification that is unavailable in this
iteration.

## 4. Retained removal condition for the deny ignore

The `RUSTSEC-2026-0543` ignore in `deny.toml` stays exactly as written. It may
be removed only when ALL of the following hold:

1. `cargo tree --workspace --all-features -i chrono` prints nothing.
2. The chrono-free graph comes from either (a) adopting an upstream
   `object_store` release that dropped chrono, or (b) an accepted RFC plus a
   verified native S3 adapter that keeps the native default and opt-in
   feature and passes security review.
3. `cargo deny --all-features check` passes with no chrono ignore.
4. `cargo audit` shows no chrono warning.
5. `cargo test --workspace --all-features`, `cargo fmt --all -- --check` and
   `cargo clippy --workspace --all-targets --all-features -- -D warnings` are
   green.

Until then `Closes #185` is not allowed. The correct keyword is `Refs #185`.

## 5. Scope of this branch

Changed: this analysis document and the PR descriptor only. Not changed:
`deny.toml`, any `Cargo.toml`, `Cargo.lock`, any Rust source, README,
CHANGELOG or RFC status. Issue #185 keeps its `blocked` label and open state.

## 6. Incidental finding (out of scope)

`crates/ag-storage/README.md` and `docs/modules/ag-storage/README.md` still
say `object_store 0.11`, while `Cargo.toml`/`Cargo.lock` use 0.14.2. This is
stale documentation under the code/documentation sync rule, but it is not a
chrono blocker and is left for a separate documentation fix.

## 7. References

- Issue #185: https://github.com/Anti-Gravital/Anti-Gravital/issues/185
- RUSTSEC-2026-0543: https://rustsec.org/advisories/RUSTSEC-2026-0543.html
- Upstream issue: https://github.com/apache/arrow-rs-object-store/issues/909
- RFC-0026: `docs/rfc/RFC-0026-chrono-replacement.md`
- RFC-0029: `docs/rfc/RFC-0029-oauth2-chrono-unblock.md`
- ADR-0009: `docs/adr/0009-gobernanza-correctiva.md`
- Board baseline: `docs/audits/issue-board-reconciliation.md` (wave 4 / wave 9)
