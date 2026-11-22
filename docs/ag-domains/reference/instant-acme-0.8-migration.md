# Reference — instant-acme 0.7 to 0.8 migration surface

Scope: the ACME order flow in `crates/ag-domains/src/acme/renewal.rs` and the
optional dependency in `crates/ag-domains/Cargo.toml` (currently `0.7`).
Tracked by issue #179, blocked by #87 (live ACME staging evidence).

This document records the API surface to port. It does **not** change code:
until the staging test of #87 passes before and after, the order-flow rewrite
would be an unverified change to the code that keeps certificates valid.

## Reproduction of the failures

A throwaway copy outside the repository with
`instant-acme = { version = "0.8", optional = true }` was checked offline on
2026-11-22 (base `72b97ad`, rustc 1.99.1). Both 0.8.0 and 0.8.5 produce the
same eight compiler errors in `acme/renewal.rs`:

```
cargo check -p ag-domains --features acme --message-format=short --offline
```

The eight line numbers from the 2026-11-10 attempt are kept below as historical
evidence. On the current tree the same eight errors are at 67, 89, 255, 264,
285, 287, 295 and 327:

```
67:43  E0599  no associated function `create` on `Account`
89:28  E0599  no associated function `from_credentials` on `Account`
252:21 error  cannot construct `NewOrder<'_>` with struct literal syntax (private fields)
261:10 E0277  `Authorizations<'_>` is not a future
282:31 E0599  no method `key_authorization` on `Order`
284:72 E0308  expected `&str`, found `String`
292:14 E0599  no method `set_challenge_ready` on `Order`
324:10 E0061  `finalize` takes 0 arguments, 1 supplied
```

Issue #179 records nine compile errors from the 2026-11-10 attempt; the current
tree reproduces eight with the published 0.8 releases. After those eight are
fixed, `cargo check` passes and the existing `refresh()` / `certificate()` wait
loop compiles unchanged. The documented ninth point is therefore a behavioral
item to verify with the staging run, not a ninth compiler error on this tree.
The verbatim spike output, the verified nine-point table and the file/line-level
patch plan are in `docs/audits/instant-acme-08-compile-spike.md`.

## The nine migration points

1. **Account creation** (line 67). `Account::create(&NewAccount { .. },
   server_url, None)` becomes `Account::builder()?.create(&NewAccount { .. },
   server_url.to_owned(), None).await`. The builder returns `Result` because
   the default HTTP client is constructed there; `directory_url` is now
   `String`, not `&str`.

2. **Account restoration** (line 89). `Account::from_credentials(credentials)`
   becomes `Account::builder()?.from_credentials(credentials).await`.
   `AccountCredentials` remains serde-serializable, so the renewal task's
   serialize-once/deserialize-per-iteration trick stays valid.

3. **New order construction** (line 252). `NewOrder` now has private fields
   (`identifiers`, `replaces`, `profile`). Use `NewOrder::new(&[identifier])`;
   ARI replacement (`NewOrder::replaces`) and certificate profiles
   (`NewOrder::profile`) are opt-in and must not be enabled silently.

4. **Authorization retrieval** (lines 259-262). `order.authorizations().await?`
   becomes a synchronous call returning the `Authorizations<'_>` stream:
   `let mut authorizations = order.authorizations();` then
   `while let Some(authz) = authorizations.next().await { let mut authz = authz?; ... }`.
   The method needs `&mut Order`, so the `&mut order` binding must be in scope
   before the loop. The old `Vec<Authorization>` shape disappears; this changes
   the lifetime structure of the whole challenge loop.

5. **DNS-01 challenge selection** (lines 275-279). Replace
   `authz.challenges.iter().find(|c| c.r#type == ChallengeType::Dns01)` with
   `authz.challenge(ChallengeType::Dns01)`, which returns
   `Option<ChallengeHandle<'_>>`. Keep that handle alive until step 8; it owns
   the mutable borrow used to signal readiness.

6. **Key authorization** (line 282). `order.key_authorization(challenge)` moved
   to the challenge handle: `challenge.key_authorization().dns_value()`.
   `KeyAuthorization::dns_value()` still returns the base64url SHA-256 digest
   required for the TXT record.

7. **Challenge identifier** (lines 281-284). `authz.identifier` is no longer an
   `Identifier` to destructure. Use `challenge.identifier().to_string()` (an
   `AuthorizedIdentifier` implements `Display`) and pass `&domain` to
   `set_dns01_challenge`, which expects `&str`; passing the owned `String`
   directly is the E0308 at line 284.

8. **Signal challenge readiness** (lines 290-295).
   `order.set_challenge_ready(challenge_url).await` becomes
   `challenge.set_ready().await` on the `ChallengeHandle`. The
   `challenge_records` vector is still needed for the TXT record ids used at
   teardown, but no longer supplies a URL for this call. Call `set_ready` while
   the corresponding authorization/challenge handles are still borrowed.

9. **Finalize and wait for the certificate** (lines 315-378).
   `order.finalize(csr.der()).await` becomes `order.finalize().await?`, which
   generates the CSR internally and returns the private key PEM (requires the
   `rcgen` feature). To keep the current explicit `rcgen` parameters, use
   `order.finalize_csr(csr.der()).await?` instead and keep using
   `private_key.serialize_pem()`. The hand-rolled `refresh()` +
   `wait_for_order_ready` loop can be replaced by
   `order.poll_ready(&RetryPolicy)` and
   `order.poll_certificate(&RetryPolicy)`; `order.certificate()` still exists
   for a manual polling loop. That replacement is optional: the existing
   `refresh()` / `certificate()` loops compile unchanged on 0.8 (compile spike).
   This is the point that also affects `acme::ari` renewal scheduling, so it
   must be verified by the live run.

## Current source locations (verified 2026-11-22)

All nine migration points above still apply semantically to the current tree;
the dependency is still `instant-acme = "0.7"` at `crates/ag-domains/Cargo.toml:86`.
No production ACME code was changed. Line numbers in the nine points above and
in the scratch-checkout diagnostics are historical; the current
`crates/ag-domains/src/acme/renewal.rs` locations are:

| Point | Documented line | Current line | Still applies |
| --- | --- | --- | --- |
| 1 `Account::create` | 67 | 67 | yes |
| 2 `Account::from_credentials` | 89 | 89 | yes |
| 3 `NewOrder` literal | 252 | 255 | yes |
| 4 `authorizations().await` | 259-262 | 262-265 | yes |
| 5 manual challenge scan | 275-279 | 278-282 | yes |
| 6 `key_authorization` | 282 | 285 | yes |
| 7 `authz.identifier` | 281-284 | 287 (E0308 call site) | yes |
| 8 `set_challenge_ready` | 290-295 | 293-298 | yes |
| 9 `finalize` + wait loop | 315-378 | 326-381 (certificate at 334) | yes |

The drift is additive (the file grew by a few lines), not a change of contract.
The eight compiler diagnostics in the historical block above keep their
2026-11-10 line numbers; the current-tree lines observed in the 2026-11-22 spike
are listed in the reproduction section and in the table. The E0308 of point 7
is at line 287 (the `set_dns01_challenge` call), not at the `Identifier::Dns`
destructure at line 284. Full evidence:
`docs/audits/instant-acme-08-compile-spike.md`.

## Exit criteria for the port

- `cargo test -p ag-domains --all-features` passes on 0.8.
- The `#[ignore]` staging test of #87 passes before and after the port, with
  both logs recorded (see `how-to/acme-staging-test.md`).
- No production ACME directory is used at any point.
