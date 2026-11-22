# Domain-loop validation - real evidence

Date (UTC): 2026-11-22T05:21:06Z
Commit under test: `9675ad86aef1a9bc52c242335b8dc7a4e2b1e5f2`
Branch: `community8/dnsqa` (no push, no merge)
Host: `angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC`
Harness: `scripts/validate-domain-loop.sh` (mode 100755)
Contract: `docs/rfc/RFC-0033-domain-to-deploy-loop.md` (dnsarch, wave 8)

## 1. Tool versions (measured, not assumed)

| Tool | Version | Role in this validation |
| --- | --- | --- |
| bash | GNU bash 5.3.9(1)-release | harness interpreter |
| rustc | 1.99.1 (58cf8f9c3 2026-10-13) | toolchain context |
| cargo | 1.99.1 (5f94df478 2026-08-27) | binary build |
| python3 | 3.14.4 | TOML/JSON parsing, hosts resolution probe |
| bwrap (bubblewrap) | 0.11.1 | no-network sandbox and hosts-file mock |
| ag | 0.1.0-alpha.1 | binary under test (`dns`, `domains attach`) |

Cargo environment for every build and run:
`CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target`
`CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0`

## 2. Method

1. The harness builds a fresh temporary project under
   `/tmp/opencode/ag-domain-loop-validation.XXXXXX` with three manifests,
   one per environment (`dev`, `staging`, `prod`), using RFC 6761 `.test`
   hostnames. `dev` and `staging` are `private`; `prod` is `public`.
   `[provider] name = "none"` is mandatory in every fixture so no adapter can
   contact a provider.
2. A local hosts file maps all three hostnames to `127.0.0.1`. Every `ag`
   invocation is executed through
   `bwrap --unshare-net --ro-bind / / --dev /dev --bind <hosts> /etc/hosts`,
   which gives the process **zero network access** and the mock hosts file as
   its only name-resolution source. A negative control confirms a TCP connect
   to `192.0.2.1:53` is denied inside the sandbox.
3. Sentinel credential values are injected as process environment variables
   (`AG_CLOUDFLARE_TOKEN`, `AG_DNS_TOKEN`, `AG_DNS_ZONE_ID`,
   `AG_DOMAINS_TOKEN`). Every captured output is scanned for those values and
   for provider API endpoints.
4. Run A is the baseline against the pre-existing shared binary (before the
   `dns` subcommand existed). Run S is a self-test of the harness against a
   bundled fake `ag` (not product evidence). Run B is the authoritative run
   against a real binary built from the dnsapi worktree.

No live DNS provider, registrar, cluster, container daemon or external network
was contacted. No file outside the temporary work root and the shared Cargo
target directory was modified.

## 3. Run A - baseline, no `dns` subcommand (honest history)

Command:

```bash
cd /home/angel-nereira/Github-local-angelnereira/worktrees8/dnsqa
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
./scripts/validate-domain-loop.sh
```

```text
date_utc: 2026-11-22T05:15:51Z
commit: 9675ad86aef1a9bc52c242335b8dc7a4e2b1e5f2
ag_binary: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
ag_version: ag 0.1.0-alpha.1
[PASS] hosts-mock (rc=0): api.dev.example.test=127.0.0.1; api.staging.example.test=127.0.0.1; api.example.test=127.0.0.1
[PASS] sandbox-no-network (rc=0): outbound connection denied (expected)
[PASS] manifest-fixture (rc=0): parsed 3 manifests: dev=private, staging=private, prod=public
[FAIL] ag-dns-subcommand (rc=2): ag has no 'dns' subcommand: error: unrecognized subcommand 'dns'
[FAIL] ag-dns-plan (rc=1): ag dns plan not available (no dns subcommand)
[FAIL] ag-dns-apply-gates (rc=1): ag dns apply not available (no dns subcommand)
[FAIL] ag-dns-status-offline (rc=1): ag dns status not available (no dns subcommand)
[FAIL] json-read (rc=1): json read checks unavailable (no dns subcommand)
[FAIL] json-mutation-refused (rc=1): json mutation checks unavailable (no dns subcommand)
[FAIL] visibility-rendered (rc=1): visibility check unavailable (no dns plan id)
[PASS] no-credentials-in-output (rc=0): no sentinel token values or provider API endpoints in any captured output
summary: PASS=4 FAIL=7 SKIP=0
```

Exit code: 1. A missing command is reported as FAIL, never SKIP, exactly as
required. No issue was opened for this baseline: the command was implemented by
the dnsapi role during this same validation loop, and the authoritative Run B
below verifies it. Opening an issue for a command that now exists and passes
would describe a false state. If a later merge regresses any check, the harness
returns FAIL and an issue with this reproduction must be opened.

## 4. Run S - harness self-test (synthetic, not product evidence)

Command: `./scripts/validate-domain-loop.sh --self-test`

The harness was exercised against a bundled fake `ag` that emits a valid plan,
refuses mutations without a plan id and with a bogus id, produces versioned
JSON for reads and writes an applied-state file. All 12 checks passed and the
script exited 0. This proves PASS is reachable and that the harness is not
hardwired to fail; it is a test of the harness only. Run B is the product
evidence.

## 5. Build provenance for the authoritative binary

The dnsapi role had not yet committed its CLI changes at validation time, so
the binary was built from its worktree with a locked, offline, read-only build:

```bash
cd /home/angel-nereira/Github-local-angelnereira/worktrees8/dnsapi
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo build --locked --offline -p ag-cli --bin ag
```

Result: `BUILD_RC=0`, `Finished 'dev' profile [unoptimized + debuginfo] ... in
2.23s` for the final rebuild (an earlier build of the same worktree took 6.92s
before dnsapi made a further source edit; the binary was rebuilt and the
harness re-run afterwards). No worktree file was edited by this validator. The first attempt used
a private target directory under `/tmp` and hit the container disk quota before
any product code was reached; it was abandoned, its directory removed, and the
shared target directory (already populated with all dependencies) was used.
That failed build is not evidence of any product defect.

## 6. Run B - final authoritative validation (verbatim)

Command: same as Run A, after the successful build above.

```text
host: angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC
date_utc: 2026-11-22T05:21:06Z
commit: 9675ad86aef1a9bc52c242335b8dc7a4e2b1e5f2
branch: community8/dnsqa
bash: 5.3.9(1)-release
rustc: rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo: cargo 1.99.1 (5f94df478 2026-08-27)
python3: Python 3.14.4
mode: product
bwrap: bubblewrap 0.11.1
ag_binary: /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
ag_version: ag 0.1.0-alpha.1
[PASS] hosts-mock (rc=0): api.dev.example.test=127.0.0.1; api.staging.example.test=127.0.0.1; api.example.test=127.0.0.1
[PASS] sandbox-no-network (rc=0): outbound connection denied (expected)
[PASS] manifest-fixture (rc=0): parsed 3 manifests: dev=private, staging=private, prod=public
[PASS] ag-dns-subcommand (rc=0): ag dns --help succeeded
[PASS] ag-dns-plan (rc=0): plan ids and record sets for all 3 envs: dev=dns-dev-2754c16fb09784a5 staging=dns-staging-2f2af7f8acd5a361 prod=dns-prod-1940af08a8d504e0; project unchanged
[PASS] ag-dns-apply-noplan (rc=2): apply refused without a plan id (required)
[PASS] ag-dns-apply-bogus (rc=1): apply refused a bogus plan id and left the project unchanged
[PASS] ag-dns-status-offline (rc=0): status answered in the no-network sandbox: Environment: prod
Not applied for 'prod'. Run 'ag dns plan' then 'ag dns apply --plan ID'.
[PASS] json-read (rc=0): dns plan and dns status emit valid versioned JSON: plan status
[PASS] json-mutation-refused (rc=0): mutations refused --output json: dns-apply(rc=2) domains-attach(rc=2)
[PASS] visibility-rendered (rc=0): visibility exact in plan text (fail-closed private) and in applied state for dev, staging, prod
[PASS] no-credentials-in-output (rc=0): no sentinel token values or provider API endpoints in any captured output
summary: PASS=12 FAIL=0 SKIP=0
```

Exit code: 0. The `status` detail line beginning `Not applied...` is part of
the status output (multi-line), not an extra check.

## 7. Exact commands behind each check

All commands run with cwd set to the fresh temporary project and inside the
no-network bwrap sandbox. Manifest flags point at `.ag/dns/<env>.toml`.

| Check | Command | Expected and observed |
| --- | --- | --- |
| hosts-mock | `python3 -c 'socket.gethostbyname(...)'` for all 3 hosts | each returns `127.0.0.1` from mock `/etc/hosts` |
| sandbox-no-network | `python3 -c 'socket.create_connection(("192.0.2.1",53),2)'` | connection denied (rc != 0) |
| manifest-fixture | inline `tomllib` parse of 3 manifests | dev/private, staging/private, prod/public, provider `none` |
| ag-dns-subcommand | `ag dns --help` | rc=0 |
| ag-dns-plan | `ag dns plan --manifest .ag/dns/<env>.toml` (x3) | `dns-<env>-<16hex>`, record table, project fingerprint unchanged, distinct ids |
| ag-dns-apply-noplan | `ag dns apply --manifest .ag/dns/prod.toml` | rc=2, message names the missing plan id |
| ag-dns-apply-bogus | `ag dns apply --manifest .ag/dns/prod.toml --plan dns-prod-deadbeefdeadbeef` | rc=1, no file written |
| ag-dns-status-offline | `ag dns status --manifest .ag/dns/prod.toml --environment prod` | rc=0, no connection error |
| json-read | `ag dns plan ... --output json`, `ag dns status ... --output json` | both parse as JSON with `contract_version` (observed `0.2.0`) |
| json-mutation-refused | `ag dns apply ... --output json`, `ag domains attach ... --output json` | rc=2 for both |
| visibility-rendered | apply with the valid prod id, then inspect plan text and `.ag/dns/state.json` | exact `Visibility:` lines, fail-closed private note, no public record in private plans, prod recorded public |
| no-credentials-in-output | grep all captured output for sentinel names and provider API hosts | no match |

## 8. Offline and credential evidence

- The sandbox has no network namespace. A direct outbound TCP connect inside
  it fails, so any `plan`, `apply` or `status` path that tried to reach a
  provider would have failed. All three succeeded.
- The only name resolution available is the mock hosts file; all three `.test`
  hosts resolve to `127.0.0.1` inside the sandbox.
- Sentinel values for four credential environment variables and known provider
  API endpoint patterns were scanned across every captured output with zero
  matches. The applied state contains only identity fields (domain, host,
  visibility, plan id, record data); no token or zone id.
- `[provider] name = "none"` in every fixture means the configured native
  path never selects an adapter.

## 9. Honest limitations

- No live DNS provider, registrar, ACME server, Kubernetes cluster or Docker
  daemon was used. This is by design: the mission is to prove the loop is
  testable offline and that its gates cannot be bypassed, not to publish a
  domain. Live publication remains explicitly opt-in.
- `--online` propagation checks and real certificate issuance were not run;
  they are out of scope for an offline harness.
- The hosts mock proves local resolution of `.test` names inside the sandbox;
  it does not prove global DNS propagation.
- The authoritative binary was built from in-progress dnsapi source, not from
  a merged commit. The audit records the exact build command so the result can
  be reproduced after merge.
- Contract divergence observed during validation: envrules authored a single
  YAML manifest with three environments, while dnsapi reads one TOML
  environment per `.ag/dns/*.toml` file (RFC-0033 places `[domain]` in
  `ag.toml`). The harness covers three environments as three manifests that
  match the implemented CLI. This is an integration item for the coordinator,
  not a defect the validator may fix by editing another role's files.
- No issue was opened, because no FAIL remains in the authoritative run. The
  baseline FAILs are recorded above with reproduction commands.

## 10. Re-run after the wave-8 branches merge

```bash
cd <repo>
bash -n scripts/validate-domain-loop.sh
git diff --check
AG_BIN=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag \
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
./scripts/validate-domain-loop.sh
```

Rebuild `ag` first so the binary contains `ag dns`. Expected outcome after a
correct merge: 12 PASS, 0 FAIL, 0 SKIP. `KEEP_WORK=1` preserves the fixture.
`--self-test` tests the harness itself; it must never be quoted as product
evidence.

## 11. Verdict

The domain loop is testable offline. `ag dns plan` produced distinct,
deterministic plan ids and record sets for dev, staging and prod with no
network and no file writes. `ag dns apply` refused to run without a plan id and
refused a bogus id without touching state. `ag dns status` answered offline.
Versioned JSON worked for both read commands and was refused for both mutation
commands. Visibility was reflected exactly in plan text and applied state,
with private plans emitting no public-only record. No credential value or
provider endpoint appeared in any output. The harness self-test proves PASS is
reachable, and the baseline proves missing commands are reported as FAIL.
Final result: 12 PASS, 0 FAIL, 0 SKIP. No result was fabricated and no blocked
check was marked PASS.
