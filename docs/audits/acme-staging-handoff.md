# Audit — ACME staging evidence handoff (#87)

Status: **BLOCKED (external)**. This document is a handoff kit, not evidence.
It gathers, in one place, the exact prerequisites, commands, evidence fields
and an empty results template for the credentialed Let's Encrypt staging run
that gates issue #87 (child of #76, RFC-0011 phases E-F) and blocks the
`instant-acme` 0.8 port in #179.

No staging run has been performed. No certificate issued by a certification
authority is claimed anywhere in this audit. The only executable artifact
present today is an offline local mock, verified in section 5; section 6 states
precisely why that mock does not satisfy the gate.

## 1. Gate definition

Issue #87 closes only when all of the following hold:

1. The ignored integration target
   `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` exists in
   `crates/ag-domains` and is exercised against the **staging** directory
   `https://acme-staging-v02.api.letsencrypt.org/directory`.
2. A real DNS-01 challenge is validated by that CA from outside the runner,
   a chain signed by a staging CA is downloaded, and SNI selection is checked
   against the controlled hostname (phases E and F of RFC-0011).
3. The evidence template in section 4 is filled from the real run, with both
   `staging-before.log` and `staging-after.log` for any dependency change
   (#179), no private key, account credential or ACME token in an artifact.
4. The challenge TXT record is removed and teardown is evidenced.

Until then, #87 and #179 remain open with the `blocked` label. Nothing in the
default CI can substitute for this gate.

## 2. Exact prerequisites

Two setups can exercise the order-flow client. Only option A satisfies #87 as
written, because the contract names the Let's Encrypt staging directory and
external DNS validation.

### Option A — disposable registered domain (required to close #87)

| Requirement | Concrete check |
| --- | --- |
| A disposable registered domain you can publish TXT records for, e.g. `staging.example.net` | `dig +short TXT _acme-challenge.<domain>` reflects a value you just published |
| Zone write access through a `DnsProvider` adapter or the native `manual` path | Adapter credentials in the runner secret store, or a human ready to publish/remove the TXT value on signal |
| External DNS visibility | Resolve `_acme-challenge.<domain>` with a resolver outside your network; `/etc/hosts` is not enough |
| Outbound TCP 443 to the staging CA directory and its OCSP/ARI endpoints | `curl -fsS https://acme-staging-v02.api.letsencrypt.org/directory` returns JSON |
| A contact email for the ACME account | `CertConfig.contact_email` / `AG_DOMAINS_STAGING_EMAIL` |
| A stable checkout and toolchain | `git rev-parse HEAD`, `rustc --version`, `cargo --version`, `uname -a` recorded |
| An isolated runner with no production ACME credentials or production DNS tokens | Runner is credentialed only for the disposable staging zone |

Stop and label the run `blocked` if any row fails. Never switch to the
production directory to check whether something works.

### Option B — ACME-compatible local CA (development only, does NOT close #87)

A local RFC 8555 CA is acceptable for *tooling rehearsal* of the client flow:
it must serve a directory with `newAccount` and `newOrder`, issue authorizations
with a `dns-01` challenge, accept finalize, and serve a chain signed by a root
you install as a trust anchor on the runner. Because that root is private to
your machine and no external resolver validates your domain, it cannot produce
CA-issued staging evidence or prove SNI serving of a staging chain. Record such
a run as local-dev only; it belongs with the mock evidence of section 5.

The repository's existing `scripts/acme-mock-harness.py` is explicitly **not**
such a CA: it emits a self-signed chain whose subject equals its issuer.

### Environment variables (contract for #87)

| Variable | Meaning | Example |
| --- | --- | --- |
| `AG_DOMAINS_STAGING_DOMAIN` | Fully qualified hostname to certify | `staging.example.net` |
| `AG_DOMAINS_STAGING_ZONE_ID` | Zone identifier understood by the selected provider | `z123` |
| `AG_DOMAINS_STAGING_EMAIL` | ACME account contact | `admin@example.net` |
| `AG_DOMAINS_STAGING_PROVIDER` | `manual` or a feature-gated adapter name | `manual` |
| `RUST_LOG` | Optional tracing filter (`ag_domains=debug`) | `ag_domains=debug` |

Provider-specific credentials come from the runner secret store, never from a
committed file. A missing variable must fail the test with the variable name;
it must not skip validation or fall back to production.

## 3. Commands

Shared build environment:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
```

### 3.1 Preflight (offline, runnable today)

```
cd <repo-root>
bash -n scripts/acme-mock-harness.sh
scripts/acme-mock-harness.sh /tmp/acme-mock-run.log
cargo fmt --all -- --check
cargo clippy -p ag-domains --all-targets --all-features -- -D warnings
cargo test -p ag-domains
cargo test -p ag-domains --all-features
curl -fsS https://acme-staging-v02.api.letsencrypt.org/directory | head -c 200
```

All commands must exit 0 before the credentialed run. The last command is the
only one that requires network egress; it is a reachability probe, not a test
pass.

### 3.2 Record the environment

```
git rev-parse HEAD
rustc --version
cargo --version
uname -a
```

### 3.3 Credentialed issuance + DNS-01 + SNI (still to be implemented)

The target name is part of the #87 contract: file
`crates/ag-domains/tests/acme_staging.rs`, function
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

Replace `manual` with the feature-gated adapter name to automate TXT
publication and add its feature (for example `--features acme,cloudflare`).

The test must assert, in order: new order (record its URL), authorization
pending, a `dns-01` challenge offered, TXT published at
`_acme-challenge.<domain>`, challenge valid, authorization valid, order ready,
finalize, a chain whose issuer is the staging CA downloaded, and TXT removed.
Fail closed on a missing/invalid variable, an invalid order or a stale TXT.

### 3.4 Renewal path against staging

The renewal code (`acme::renewal::issue_with_credentials` and
`spawn_renewal_task_with_ari`) must be exercised with the account credentials
returned by the first issuance, not only with a fresh account. The same ignored
target must, after the first chain is saved:

1. Reuse the serialized `AccountCredentials` to issue a second order for the
   same controlled hostname against the same staging directory.
2. Record the second order URL, its challenge status and the second chain's
   SHA-256 fingerprint; the fingerprint must differ from the first issuance.
3. Feeding an ARI `RenewalInfo` (RFC 9773) into `next_renewal_sleep` must be
   logged with its suggested window and the resulting sleep seconds. The HTTP
   fetch is the live boundary; the scheduling decision itself is unit-tested
   and is not a substitute for the live fetch.
4. Repeat the command as `staging-after.log` for the #179 candidate code.

No production directory is used at any point.

### 3.5 Chain, SNI and teardown verification

```
openssl x509 -in <issued-chain.pem> -noout -subject -issuer -dates -fingerprint -sha256

openssl s_client -connect <edge-host>:443 \
  -servername "$AG_DOMAINS_STAGING_DOMAIN" -showcerts </dev/null 2>/dev/null \
  | openssl x509 -noout -subject -issuer -fingerprint -sha256

dig +short TXT _acme-challenge."$AG_DOMAINS_STAGING_DOMAIN"
```

The issuer must be a Let's Encrypt **staging** CA; the SNI subject must match
the controlled hostname; the final `dig` must show no challenge TXT. Record
only the SHA-256 fingerprint. Never paste a private key, account credential or
ACME token into a log, issue comment or pull request. Keep raw logs in the
credentialed runner's restricted store; commit only the redacted template.

## 4. Evidence fields to capture

| Field | Where it comes from | Why it is required |
| --- | --- | --- |
| Run date (UTC) and wall-clock timestamps per event | Harness/test log | Ordering of challenge publication, validation and finalize |
| Commit SHA, `rustc`, `cargo`, `uname -a` | Section 3.2 | Reproducibility (rules 36-37) |
| Controlled domain, provider, feature flags | Env vars | Scope of the certificate |
| ACME directory URL | Test log | Proves staging, not production |
| First and second **order URL** | CA order objects | Traceable to the CA; distinct per issuance (renewal) |
| **Challenge status** transitions (pending -> valid) and challenge type `dns-01` | Authorization objects | Proves the CA validated DNS-01 |
| Time TXT was published and time it was removed | Provider log / `dig` | Proves propagation window and teardown |
| Chain **SHA-256 fingerprint** (first and second), subject, issuer, notBefore, notAfter | `openssl x509` | Proves a staging-signed chain and distinct renewal |
| ARI suggested window and computed sleep seconds | Test log | Proves the live renewal boundary was exercised |
| SNI result (subject + issuer) or "edge listener not part of this run" | `openssl s_client` | Proves the certificate bridge, phase F |
| Exit code and log artifact names | `tee` output | Distinguishes pass, fail and blocked |
| Result: pass / fail / blocked (with reason) | Operator | No silent skip |

### Empty results template

Copy this block for each run. Every field must be filled from a real run;
nothing below is pre-filled.

```
ACME staging run
- Issue: #87 (prerequisite for #179)
- Date (UTC):
- Commit SHA:
- rustc --version / cargo --version:
- OS (uname -a):
- Domain (redact only if required by policy):
- Provider / feature flags:
- ACME directory:
- First order URL:
- Challenge (type / status transitions / timestamps):
- TXT published at (time):
- First chain subject / staging issuer / notBefore / notAfter / SHA-256 fingerprint:
- Second order URL (renewal):
- Second chain SHA-256 fingerprint (must differ from first):
- ARI suggested window / computed sleep seconds:
- SNI check result (subject + issuer) or "edge listener not part of this run":
- Challenge TXT removed: yes / no (evidence):
- Exit code:
- Log artifact names (staging-before.log / staging-after.log):
- Secret scan of artifacts (no key, credential or token): pass / fail
- Result: pass / fail / blocked (with reason)
```

## 5. Existing local ACME mock harness (verified, reused, not duplicated)

The repository already contains an offline harness; this audit references it
and does not add a second one:

- `scripts/acme-mock-harness.py`: in-memory RFC 8555-shaped HTTP server on
  `127.0.0.1` with an ephemeral port, plain HTTP, and a locally generated
  self-signed chain. Asserts the exact state sequence
  `new_order:pending -> authorization:pending -> challenge:pending ->
  challenge:valid -> authorization:valid -> order:ready -> order:processing ->
  order:valid`.
- `scripts/acme-mock-harness.sh`: wrapper that checks `python3` and `openssl`,
  tees the JSON-line transcript to an optional log and propagates the exit
  code (0 PASS, 1 assertion failure, 2 missing tool).
- Prior recorded run: `docs/ag-domains/acme-mock-evidence.md` (three consecutive
  PASS runs, base checkout `7cbc28a3...`).

Verification performed for this handoff (tooling check only):

```
bash -n scripts/acme-mock-harness.sh
python3 -m py_compile scripts/acme-mock-harness.py
scripts/acme-mock-harness.sh /tmp/w12-i87-mock.log
```

Observed exit code `0`. Verbatim final transcript line (timestamps and
fingerprint differ per run by design):

```
{"at": "2026-11-22T17:00:52.466487+00:00", "event": "harness:result", "result": "PASS", "sequence": ["new_order:pending", "authorization:pending", "challenge:pending", "challenge:valid", "authorization:valid", "order:ready", "order:processing", "order:valid"], "statement": "Local mock flow only. No real CA issued a certificate; the chain was self-signed on this machine."}
```

What it proves: the documented order state machine can be driven
deterministically offline; links are read from directory/order objects rather
than hardcoded; the downloaded PEM parses and is self-signed; the wrapper
fails closed on a missing tool.

What it cannot prove: any external DNS validation or propagation, any CA
signature, issuance or renewal by a real CA, ARI fetch against a live CA,
SNI serving of a CA-issued chain, or correctness of the Rust crate's own
tests (the harness executes Python only, not `crates/ag-domains`). It does not
execute the ignored target of #87 and does not produce a staging chain.

## 6. Why the current mock-only evidence does not satisfy this gate

This separation is explicit and must not be collapsed:

- `docs/ag-domains/acme-mock-evidence.md` and the run in section 5 are local
  tooling evidence. The chain's **subject equals its issuer**; it was generated
  on this machine by `openssl` and no certification authority was contacted.
- Issue #87 requires a chain signed by the **Let's Encrypt staging CA**, a
  DNS-01 challenge validated by that CA through external DNS, and SNI serving
  of that chain. A self-signed loopback transcript cannot display a staging
  issuer, a real order URL at the staging CA or external validation timestamps.
- The renewal requirement (section 3.4) needs a live CA order and ARI fetch;
  the mock has no ARI resource and no account reuse against a CA.
- Therefore no mock run, unit test or CI job can close #87 or #179. Both stay
  open with `blocked` until the filled template of section 4 and both before
  and after logs exist.

## 7. Exact external evidence required to close #87

1. `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` implemented and
   passing against the staging directory with real DNS-01 validation.
2. Filled section 4 template with both order URLs, challenge statuses,
   timestamps, two distinct staging-signed chain fingerprints, ARI window and
   sleep, SNI subject/issuer, teardown `dig` output and exit code.
3. `staging-before.log` and `staging-after.log` retained in the credentialed
   runner's restricted store, each secret-scanned and referenced by name.
4. `cargo test -p ag-domains --all-features` green on the candidate code for
   #179 (that issue stays open until the port is verified).

Until all four items exist, #87 is externally blocked and is not closed.

## References

- `docs/ag-domains/how-to/acme-staging-test.md` (canonical credentialed
  procedure and original evidence template)
- `docs/ag-domains/acme-mock-evidence.md` (offline mock, no CA)
- `docs/ag-domains/reference/instant-acme-0.8-migration.md` (port checklist,
  blocked by #87)
- `docs/rfc/RFC-0011-ag-domains-control-plane.md` (phases E-F, ACME and SNI)
- `crates/ag-domains/src/acme/` (`crates/ag-domains/src/acme/renewal.rs`,
  `crates/ag-domains/src/acme/ari.rs`,
  `crates/ag-domains/src/acme/challenge.rs`,
  `crates/ag-domains/src/acme/wildcard.rs`)
- `scripts/acme-mock-harness.sh`, `scripts/acme-mock-harness.py`
