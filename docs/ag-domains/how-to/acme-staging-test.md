# How-to — live ACME staging issuance and SNI verification

This is the reproducible procedure for the credentialed `#[ignore]` end-to-end
test tracked in issue #87 (child of #76, RFC-0011 phases E-F). It is the
prerequisite for the `instant-acme` 0.8 port tracked in issue #179.

Status: **blocked**. The credentialed `#[ignore]` test target described below
does not exist in the repository yet. The offline commands in "Preflight" are
runnable today; the staging commands are the contract that #87 must implement
and execute. Never report the test as passing until the evidence template at
the end is filled from a real staging run.

## Why staging must be live

The default CI has no controlled domain, DNS write access or outbound access to
a CA, so it can only exercise the mocked DNS-01 driver
(`acme::wildcard::Dns01OrderDriver`) and the deterministic scheduling helpers.
Certificate issuance and renewal change the code that keeps certificates valid;
a port of the ACME order flow is not verifiable from unit tests alone. The live
run is performed before and after the 0.8 port (#179), and both runs are
recorded.

Use the Let's Encrypt **staging** directory only. Staging certificates are not
trusted by browsers and are rate-limited separately. Never point this procedure
at the production directory.

## Environment prerequisites

| Requirement | Why |
| --- | --- |
| A controlled domain you can publish TXT records for, e.g. `staging.example.net` | DNS-01 validation of `_acme-challenge.<domain>` |
| Write access to that zone through a `DnsProvider` adapter or the native manual path | Publish and remove the short-lived challenge record |
| A DNS resolver that can see the published TXT record | The CA validates from the outside; local `/etc/hosts` is not enough |
| Outbound HTTPS (TCP 443) to the Let's Encrypt staging directory and its OCSP/ARI endpoints | ACME order, challenge, finalize and certificate download |
| A contact email for the ACME account | Required by `CertConfig.contact_email` |
| A stable checkout (commit SHA) and a known Rust toolchain | Evidence must be reproducible |
| An isolated runner with no production ACME credentials in the environment | Prevent an accidental production issuance |

The DNS provider credentials must come from the credentialed runner's secret
store, never from a committed file. The test must fail early with a clear error
if a required variable is missing; it must never silently skip validation and
never fall back to the production directory.

### Environment variables (contract for #87)

| Variable | Meaning | Example |
| --- | --- | --- |
| `AG_DOMAINS_STAGING_DOMAIN` | Fully qualified hostname to certify | `staging.example.net` |
| `AG_DOMAINS_STAGING_ZONE_ID` | Zone identifier understood by the selected provider | `z123` |
| `AG_DOMAINS_STAGING_EMAIL` | ACME account contact | `admin@example.net` |
| `AG_DOMAINS_STAGING_PROVIDER` | Adapter to use: `manual` or a feature-gated adapter name | `manual` |
| `RUST_LOG` | Optional tracing filter for the run log | `ag_domains=debug` |

Provider-specific credentials are named by the adapter that is selected and are
documented in `reference/provider-capability-matrix.md`. With `manual`, the
operator publishes the printed TXT value before the test signals readiness and
removes it after the run.

## Preflight (offline, runs in default CI)

```
cd <repo-root>
cargo fmt --all -- --check
cargo clippy -p ag-domains --all-targets --all-features -- -D warnings
cargo test -p ag-domains
cargo test -p ag-domains --all-features
```

All four commands must exit 0 before the credentialed run is attempted. The
last command exercises the existing ACME unit tests without network access.

Set the shared build environment used by the repository:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
```

## Credentialed run (blocked on #87)

1. Confirm the checkout and toolchain and record them for the evidence:

   ```
   git rev-parse HEAD
   rustc --version
   cargo --version
   uname -a
   ```

2. Export the variables from the table above in the credentialed runner. Verify
   that the staging directory is selected by checking that the URL contains
   `staging` (the existing `acme_url_staging` unit test asserts this).

3. Run the ignored staging test target. The target name is part of the #87
   contract: `tests/acme_staging.rs`, test function
   `staging_dns01_issue_and_serve_sni`.

   ```
   AG_DOMAINS_STAGING_DOMAIN=staging.example.net \
   AG_DOMAINS_STAGING_ZONE_ID=z123 \
   AG_DOMAINS_STAGING_EMAIL=admin@example.net \
   AG_DOMAINS_STAGING_PROVIDER=manual \
   RUST_LOG=ag_domains=debug \
   cargo test -p ag-domains --features acme --test acme_staging \
     -- --ignored --exact staging_dns01_issue_and_serve_sni --nocapture 2>&1 \
     | tee staging-before.log
   ```

   Replace `manual` with the feature-gated adapter name to automate challenge
   publication, adding its feature to the `--features` flag (for example
   `--features acme,cloudflare`).

4. Run the same command with the candidate code under test and save the output
   as `staging-after.log`. Both logs are part of the evidence for #179.

5. Inspect the issued chain without writing the private key to the log:

   ```
   openssl x509 -in <issued-chain.pem> -noout -subject -issuer -dates -fingerprint -sha256
   ```

   The issuer must be a Let's Encrypt **staging** CA. Record only the SHA-256
   fingerprint; never paste the private key, account credentials or ACME tokens
   into a log, issue comment or pull request.

6. If the edge listener is started with the issued chain, verify SNI selection
   against the controlled hostname:

   ```
   openssl s_client -connect <edge-host>:443 -servername "$AG_DOMAINS_STAGING_DOMAIN" \
     -showcerts </dev/null 2>/dev/null \
     | openssl x509 -noout -subject -issuer -fingerprint -sha256
   ```

   The subject must match the controlled hostname and the issuer must be the
   staging CA. A mismatch means the SNI certificate bridge is wrong even if
   issuance succeeded.

7. Confirm teardown: query the zone and assert that no `_acme-challenge` TXT
   record remains for the domain. For the native/manual path, remove it by hand
   and record that in the evidence. Stale challenge records are a defect, not
   an acceptable leftover.

## Failure handling

- Missing or wrong env var: fail with the variable name in the error; do not
  skip the test.
- ACME order invalid: keep the run log, confirm the TXT value and propagation,
  then rerun. Do not switch to production "to check".
- Challenge record not removed: treat the run as failed and clean the zone
  before retrying.
- Network unavailable: label the run `blocked`, keep the issue open and record
  the exact connection error. This is the current state of #87.

## Evidence template

Copy this block into the issue comment or pull request description for each run.

```
ACME staging run
- Issue: #87 (prerequisite for #179)
- Date (UTC):
- Commit SHA:
- rustc --version / cargo --version:
- OS (uname -a):
- Domain (redact only if required by policy):
- Provider / feature flags:
- ACME directory: https://acme-staging-v02.api.letsencrypt.org/directory
- Command (exact, with env vars shown but secrets redacted):
- Exit code:
- Chain subject / staging issuer / notBefore / notAfter / SHA-256 fingerprint:
- SNI check result (subject + issuer) or "edge listener not part of this run":
- Challenge TXT removed: yes / no (evidence):
- Log artifact name (e.g. staging-before.log):
- Result: pass / fail / blocked (with reason)
```

A run counts as verified only when every field is filled, both `before` and
`after` logs exist for a dependency change, and no credential or private key is
present in the artifacts. Keep #87 and #179 open with the `blocked` label until
that evidence exists.
