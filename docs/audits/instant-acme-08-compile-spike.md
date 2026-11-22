# Audit — instant-acme 0.7 to 0.8 compile spike (issue #179)

Scope: a throwaway compile spike only. No production ACME code and no
dependency pin were changed in this worktree. The live ACME staging gate of
#87 is still blocked, so the order-flow port is deliberately NOT applied to
the repository. This document records real compiler output, verifies which of
the nine documented migration points still hold, and leaves a file/line-level
patch plan ready for when #87 supplies before/after staging evidence.

## Environment

| Field | Value |
| --- | --- |
| Base commit | `72b97adbdccf8cab3fbd5f411f54343beab40672` (branch `community12/i179`) |
| Date (UTC) | 2026-11-22 17:05 |
| rustc | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| OS | Linux x86_64 (same host as the shared Cargo target) |
| Dependency under test | `instant-acme` 0.8.5 (`Updating instant-acme v0.7.2 -> v0.8.5`, `Adding untrusted v0.7.1`) and 0.8.0 (`Updating instant-acme v0.7.2 -> v0.8.0 (available: v0.8.5)`) |
| Cargo env | `CARGO_TARGET_DIR=/tmp/opencode/i179-spike-target`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0` |
| Network | none (`--offline`; both 0.8.0 and 0.8.5 are in the local registry cache) |

The throwaway copies were made outside the repository, so none of their
artifacts can leak into a commit:

```
cd /home/angel-nereira/Github-local-angelnereira/worktrees12/i179
rm -rf /tmp/opencode/i179-spike && mkdir -p /tmp/opencode/i179-spike
tar --exclude=.git --exclude=target -cf - . | tar -xf - -C /tmp/opencode/i179-spike
# in the copy only: crates/ag-domains/Cargo.toml
#   instant-acme = { version = "0.7", optional = true }
#   -> instant-acme = { version = "0.8", optional = true }
export CARGO_TARGET_DIR=/tmp/opencode/i179-spike-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
cargo check -p ag-domains --features acme --message-format=short --offline
```

## Real pre-patch compiler output (0.8.5, verbatim)

```
crates/ag-domains/src/acme/renewal.rs:67:43: error[E0599]: no associated function or constant named `create` found for struct `Account` in the current scope: associated function or constant not found in `Account`
crates/ag-domains/src/acme/renewal.rs:89:28: error[E0599]: no associated function or constant named `from_credentials` found for struct `Account` in the current scope: associated function or constant not found in `Account`
crates/ag-domains/src/acme/renewal.rs:255:21: error: cannot construct `NewOrder<'_>` with struct literal syntax due to private fields
crates/ag-domains/src/acme/renewal.rs:264:10: error[E0277]: `Authorizations<'_>` is not a future: `Authorizations<'_>` is not a future
crates/ag-domains/src/acme/renewal.rs:285:31: error[E0599]: no method named `key_authorization` found for struct `Order` in the current scope
crates/ag-domains/src/acme/renewal.rs:287:72: error[E0308]: mismatched types: expected `&str`, found `String`
crates/ag-domains/src/acme/renewal.rs:295:14: error[E0599]: no method named `set_challenge_ready` found for struct `Order` in the current scope: method not found in `Order`
crates/ag-domains/src/acme/renewal.rs:327:10: error[E0061]: this method takes 0 arguments but 1 argument was supplied
error: could not compile `ag-domains` (lib) due to 8 previous errors
```

Exit code: `101`. The exact same eight diagnostics, at the exact same current
tree line numbers, were reproduced with `instant-acme = "=0.8.0"`. This
confirms the reference document's claim that 0.8.0 and 0.8.5 fail identically.

The compiler's own suggestions match the documented API shape: `NewOrder::new`,
remove `.await` from `authorizations()`, and `finalize()` with no argument
(order.rs:46 and order.rs:65 of the 0.8.5 source).

## Verification of the nine documented migration points

All nine points still apply semantically to base `72b97ad`. Eight are
reproducible compiler errors; the ninth is a behavioral verification item,
not a compiler error on this tree (see the correction below).

| Point | Reference doc line | Spike diagnostic line | Result |
| --- | --- | --- | --- |
| 1 `Account::create` | 67 (doc/current) | 67 E0599 | holds; builder required, `directory_url` is `String` |
| 2 `Account::from_credentials` | 89 (doc/current) | 89 E0599 | holds; builder required |
| 3 `NewOrder` literal | 252 doc / 255 current | 255 error | holds; private fields, use `NewOrder::new(&[identifier])` |
| 4 `authorizations().await` | 259-262 doc / 262-265 current | 264 E0277 | holds; sync stream returning `Authorizations<'_>` |
| 5 manual challenge scan | 275-279 doc / 278-282 current | (no direct diagnostic) | holds; `AuthorizationHandle::challenge(Dns01)` |
| 6 `key_authorization` | 282 doc / 285 current | 285 E0599 | holds; method is on `ChallengeHandle` |
| 7 `authz.identifier` / `&str` | 281-284 doc / 284 current | 287 E0308 | holds, but the current E0308 is at **287**, not 284 (correction) |
| 8 `set_challenge_ready` | 290-295 doc / 293-298 current | 295 E0599 | holds; `ChallengeHandle::set_ready()` |
| 9 finalize and wait | 315-378 doc / 326-381 current | 327 E0061 | holds for finalize; the wait/certificate path is source-compatible (correction) |

Corrections to `docs/ag-domains/reference/instant-acme-0.8-migration.md`:

1. The "Reproduction of the current failures" block lists historical lines
   (252, 261, 282, 284, 292, 324) from the 2026-11-10 attempt. On the current
   tree the eight errors are at 67, 89, 255, 264, 285, 287, 295 and 327. The
   reference now labels the block as historical and records the spike's lines.
2. Point 7's "current line" was 284; the actual E0308 is at 287 (the
   `set_dns01_challenge(..., domain, ...)` call), because `domain` is an owned
   `String` and the helper takes `&str`.
3. The issue records nine compile errors, but only eight reproduce on the
   current tree. After applying the eight fixes, `cargo check` passes: the
   hand-rolled `wait_for_order_ready` loop (`order.refresh()`) and the
   `order.certificate()` polling loop remain source-compatible with 0.8.
   Point 9 is therefore a behavioral item to verify with the staging run,
   not a ninth compiler error. `poll_ready`/`poll_certificate` with
   `RetryPolicy` are an available simplification, not a required port.

## Verified patch plan (throwaway only; apply after #87 evidence exists)

Two files change:

1. `crates/ag-domains/Cargo.toml:86`:
   `instant-acme = { version = "0.7", optional = true }` becomes
   `instant-acme = { version = "0.8", optional = true }`.
   Cargo.lock resolves 0.7.2 to 0.8.5 and adds `untrusted v0.7.1`.
2. `crates/ag-domains/src/acme/renewal.rs`: apply the diff below. It covers
   points 1-8 plus the `finalize_csr` choice for point 9. Existing log and
   error strings are preserved so the patch changes no observable text.

The patch was applied verbatim to the throwaway copy. Key decisions:

- `acme_url()` keeps returning `&'static str` (unit tests assert on it); the
  builder call uses `server_url.to_owned()` to satisfy the `String` parameter.
- The authorization stream is wrapped in a scoped block. `ChallengeHandle`
  holds the mutable borrow of the order state, so the stream and handles must
  be dropped before `wait_for_order_ready(&mut order)` and `finalize_csr`.
- `challenge_records` drops the URL element: 0.8 signals readiness on the
  handle, so no `challenge.url` is needed. Teardown only needs the record id.
- `finalize_csr(csr.der())` is used instead of `finalize()`: it preserves the
  existing explicit `rcgen` `CertificateParams`, `KeyPair` and
  `private_key.serialize_pem()` path. `finalize()` would generate CSR and
  private key internally (and requires instant-acme's `rcgen` feature).
- `wait_for_order_ready()` and the `certificate()` polling loop are unchanged;
  they compile and pass tests on 0.8.
- Caveat for a future wildcard order: `AuthorizedIdentifier::Display` renders
  a wildcard identifier as `*.example.com`, so `challenge.identifier().to_string()`
  would build `_acme-challenge.*.example.com`. The current code only creates
  `Identifier::Dns(config.domain)` (non-wildcard), so this is out of scope here.

```diff
--- a/crates/ag-domains/src/acme/renewal.rs
+++ b/crates/ag-domains/src/acme/renewal.rs
@@ -64,17 +64,19 @@
     let server_url = acme_url(config.staging);
     let contact = format!("mailto:{}", config.contact_email);

-    let (account, credentials) = Account::create(
-        &NewAccount {
-            contact: &[contact.as_str()],
-            terms_of_service_agreed: true,
-            only_return_existing: false,
-        },
-        server_url,
-        None,
-    )
-    .await
-    .map_err(|e| AgDomainsError::Acme(e.to_string()))?;
+    let (account, credentials) = Account::builder()
+        .map_err(|e| AgDomainsError::Acme(e.to_string()))?
+        .create(
+            &NewAccount {
+                contact: &[contact.as_str()],
+                terms_of_service_agreed: true,
+                only_return_existing: false,
+            },
+            server_url.to_owned(),
+            None,
+        )
+        .await
+        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

     let cert = issue_order(&account, config, provider).await?;
     Ok((cert, credentials))
@@ -86,7 +88,9 @@
     credentials: AccountCredentials,
     provider: &P,
 ) -> Result<IssuedCert, AgDomainsError> {
-    let account = Account::from_credentials(credentials)
+    let account = Account::builder()
+        .map_err(|e| AgDomainsError::Acme(e.to_string()))?
+        .from_credentials(credentials)
         .await
         .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

@@ -252,56 +256,44 @@
 ) -> Result<IssuedCert, AgDomainsError> {
     let identifier = Identifier::Dns(config.domain.clone());
     let mut order = account
-        .new_order(&NewOrder {
-            identifiers: &[identifier],
-        })
+        .new_order(&NewOrder::new(&[identifier]))
         .await
         .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

-    // Collect the pending DNS-01 challenges.
-    let authorizations = order
-        .authorizations()
-        .await
-        .map_err(|e| AgDomainsError::Acme(e.to_string()))?;
-
-    let mut challenge_records: Vec<(
-        String, /* domain */
-        String, /* url */
-        String, /* record_id */
-    )> = Vec::new();
-
-    for authz in &authorizations {
-        if !matches!(authz.status, AuthorizationStatus::Pending) {
-            continue;
-        }
-
-        let challenge = authz
-            .challenges
-            .iter()
-            .find(|c| c.r#type == ChallengeType::Dns01)
-            .ok_or_else(|| AgDomainsError::Acme("no se encontro challenge DNS-01".to_owned()))?;
-
-        let Identifier::Dns(domain) = &authz.identifier;
-        let dns_value = order.key_authorization(challenge).dns_value();
-
-        let record_id = set_dns01_challenge(provider, &config.zone_id, domain, &dns_value).await?;
+    // Publishing the TXT record and signalling readiness must happen while the
+    // ChallengeHandle is alive: it owns the mutable borrow of the order state.
+    // The stream is scoped so those borrows are released before polling.
+    let mut challenge_records: Vec<(String /* domain */, String /* record_id */)> = Vec::new();
+    {
+        let mut authorizations = order.authorizations();
+        while let Some(authz) = authorizations.next().await {
+            let mut authz = authz.map_err(|e| AgDomainsError::Acme(e.to_string()))?;
+            if !matches!(authz.status, AuthorizationStatus::Pending) {
+                continue;
+            }

-        challenge_records.push((domain.clone(), challenge.url.clone(), record_id));
-    }
+            let mut challenge = authz.challenge(ChallengeType::Dns01).ok_or_else(|| {
+                AgDomainsError::Acme("no se encontro challenge DNS-01".to_owned())
+            })?;
+
+            let domain = challenge.identifier().to_string();
+            let dns_value = challenge.key_authorization().dns_value();
+            let record_id =
+                set_dns01_challenge(provider, &config.zone_id, &domain, &dns_value).await?;
+            challenge
+                .set_ready()
+                .await
+                .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

-    // Notify the server that the challenges are ready.
-    for (_, challenge_url, _) in &challenge_records {
-        order
-            .set_challenge_ready(challenge_url)
-            .await
-            .map_err(|e| AgDomainsError::Acme(e.to_string()))?;
+            challenge_records.push((domain, record_id));
+        }
     }

     // Wait with exponential backoff until the order is ready.
     let result = wait_for_order_ready(&mut order).await;

     // Clean up TXT records regardless of the result.
-    for (_, _, record_id) in &challenge_records {
+    for (_, record_id) in &challenge_records {
         if let Err(e) = remove_dns01_challenge(provider, &config.zone_id, record_id).await {
             warn!(record_id, error = %e, "no se pudo eliminar el registro challenge DNS-01");
         }
@@ -312,7 +304,7 @@
     // Generate CSR and finalize.
     let names = challenge_records
         .iter()
-        .map(|(d, _, _)| d.clone())
+        .map(|(d, _)| d.clone())
         .collect::<Vec<_>>();

     let mut params =
@@ -324,7 +316,7 @@
         .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

     order
-        .finalize(csr.der())
+        .finalize_csr(csr.der())
         .await
         .map_err(|e| AgDomainsError::Acme(e.to_string()))?;

```

## Post-patch verification in the throwaway (real output)

```
cargo check -p ag-domains --features acme --message-format=short --offline
    Checking ag-domains v0.1.0-alpha.2 (/tmp/opencode/i179-spike/crates/ag-domains)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.46s
EXIT=0

rustfmt --edition 2021 --check crates/ag-domains/src/acme/renewal.rs
EXIT=0

cargo test -p ag-domains --all-features --offline
test result: ok. 171 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out (sql_store_pg, requires DATABASE_URL)
test result: ok. 2 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out (doc-tests)
EXIT=0
```

These runs prove source and unit-test compatibility of the patch on 0.8. They
do NOT prove certificate issuance or renewal against a CA. No ACME directory
was contacted at any point.

## Status and exact closure sequence for #179

#179 stays OPEN with `blocked` and `Refs #179`; no production code is merged
by this audit. The gate is unchanged from
`docs/ag-domains/reference/instant-acme-0.8-migration.md` and
`docs/ag-domains/how-to/acme-staging-test.md`.

Once #87 provides a real staging run, the exact sequence to close #179 is:

1. Implement `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` (the #87
   contract) on the current 0.7 tree and run the credentialed staging command
   from `how-to/acme-staging-test.md`, satisfying the 12 evidence fields of
   `docs/audits/acme-staging-handoff.md`; save `staging-before.log` with the
   template filled.
2. Apply the Cargo.toml pin (`0.8`) and the renewal.rs diff above to the real
   worktree; update Cargo.lock via `cargo update -p instant-acme --precise 0.8.5`.
3. Run `cargo fmt --all -- --check`,
   `cargo clippy -p ag-domains --all-targets --all-features -- -D warnings`
   and `cargo test -p ag-domains --all-features`; all must exit 0.
4. Re-run the same credentialed staging command on the 0.8 tree and save
   `staging-after.log`. Both logs must show a Let's Encrypt **staging** issuer,
   matching SNI subject/issuer, and no leftover `_acme-challenge` TXT record.
5. Update `docs/ag-domains/reference/instant-acme-0.8-migration.md` and
   `docs/ag-domains/acme-mock-evidence.md` to reflect that the port is applied;
   update the README if the observable dependency/feature state changes.
6. Only then open a PR with `Closes #179` (and `Closes #87` only if #87's own
   evidence criteria are met) and the descriptor in `docs/pr-drafts/`.
