# DNS / deploy / env / config user journey (wave 10 terminal QA)

Date: 2026-11-22. Auditor role: `dnsdeploy` (real-user terminal test, wave 10).
This is an observation record, not a benchmark. Timings are single wall-clock
observations on one run; no mean, variance or statistical claim is made.

## 1. Environment and method

| Item | Value |
| --- | --- |
| Repository | Anti-Gravital/Anti-Gravital |
| Commit under test | `72fa9d836a6de2f4f96429c11ffcff13142b2ef4` (branch base) |
| Branch / worktree | `community10/dnsdeploy` / `worktrees10/dnsdeploy` |
| Binary | `ag 0.1.0-alpha.2`, built `-p ag-cli --bin ag --offline`, dev profile |
| Build target dir | private `/home/angel-nereira/Github-local-angelnereira/private-target-dnsdeploy` |
| Build result | `Finished dev profile` in 5m37s, exit 0 |
| OS / CPU | Linux 7.0.0-44-generic x86_64 GNU/Linux, 12 cores |
| Rust | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| Fresh project | `ag new dnsdeploy-proj --template rest --no-interactive`, exit 0 |
| Project path | fresh `mktemp -d` under `/tmp`, not inside the worktree |
| State paths | private files under `/home/angel-nereira/Github-local-angelnereira/dnsdeploy-evidence/` |
| Timing method | `date +%s%N` immediately before and after each process; elapsed ms integer |

No live DNS provider, CA, cluster, container runtime or network service was
contacted. No credential was supplied to any DNS command. A canary value was
supplied only as `DATABASE_URL` to `ag env check` to test redaction.

### Project setup performed by the "user"

The `rest` template produced only `Cargo.toml`, `config.toml` and `src/main.rs`.
Following `docs/dns/DOMAIN_LOOP.md` section 2 and `docs/rfc/RFC-0020`, the user
added `ag.toml` with a `[project]`, `[domain]` and two `[env.*]` tables:

```toml
[project]
name = "dnsdeploy-proj"

[domain]
domain = "app.example.com"
zone = "example.com"
environment = "production"
visibility = "public"
target = { kind = "service", ref = "dnsdeploy-proj" }
ttl = 300
tls = "managed_http01"
allow_wildcard = false

[env.DATABASE_URL]
type = "url"
required = true
secret = true
description = "PostgreSQL connection string"

[env.PORT]
type = "int"
default = 8080
description = "HTTP listen port"
```

The repository's `deploy/k8s/` and `deploy/docker/` trees were copied into the
project, since the `rest` template does not ship them and `ag deploy plan`
requires a local source set.

## 2. First contact: fresh project, no manifests

These are the real first-run results before `ag.toml` or deploy artifacts were
added. They are reproduced verbatim.

| Command | Exit | ms | Result |
| --- | --- | --- | --- |
| `ag config get bind` | 0 | 75 | `127.0.0.1:8080` |
| `ag env list` | 1 | 61 | `error: cannot read ag.toml: No such file or directory ...` |
| `ag env check` | 1 | 77 | same error |
| `ag dns plan` | 1 | 65 | `error: could not read project manifest 'ag.toml' ... Add a [domain] table as documented in docs/dns/DOMAIN_LOOP.md` |
| `ag dns status` | 0 | 45 | `Not applied for 'production'. Run 'ag dns plan' then 'ag dns apply --plan ID'.` |
| `ag deploy plan --target k8s` | 1 | 80 | `error: deployment source set for 'k8s' is not available: .../deploy/k8s is missing ... (no file was written)` |
| `ag deploy status` | 0 | 68 | `No deployment artifacts generated under deploy/generated/.` |

Observations:

- `ag config` works on the template's `config.toml`; the env/dns/deploy
  commands all require files the `rest` template does not generate. This is a
  real onboarding gap, though it is consistent with the documented
  "add a `[domain]` table" step. The deploy source gap is not documented in
  the `rest` template README.
- `ag dns status` succeeds with a helpful next-step message even with no
  manifest. Good failure mode.

## 3. Journey with the documented manifest

Plan IDs returned by the tools (deterministic for this manifest):

- DNS plan: `dns-production-e533df963b744233`
- Deploy plan: `dpl-k8s-1ced401258853132`

| Command | Exit | ms | Notable real output |
| --- | --- | --- | --- |
| `ag env list` | 0 | 70 | table with `DATABASE_URL` (secret=true, source=none, present=false) and `PORT` (source=default, present=true); ends `Values are never printed.` |
| `ag env check` (no value) | 1 | 36 | `error: environment validation failed (1 problem(s); values are never printed): 'DATABASE_URL': expected url; missing ...` |
| `ag env check` with canary `DATABASE_URL` set | 0 | 78 | `Environment OK: 2 variable(s) resolved from the process environment and declared defaults.` |
| `ag dns plan` | 0 | (timed separately below) | plan id, binding summary, `Records (1)`: ownership TXT only |
| `ag dns status` (before apply) | 0 | 74 | `Not applied ...` |
| `ag dns apply --plan <DNSID> --state PRIVATE` | 0 | 143 | `Applied plan ...`, `Provider: none (local configuration only; no network access)` |
| `ag dns status --state PRIVATE` | 0 | 153 | `Plan: ...`, `In sync: yes` |
| `ag dns apply` without `--plan` | 2 | 45 | clap: `the following required arguments were not provided: --plan <PLAN>` |
| `ag dns apply --plan dns-production-0000000000000000` | 1 | 107 | `error: plan drift: requested ... but the manifest now resolves to ...; re-run 'ag dns plan'` |
| `ag domains attach` without edge host | 1 | 122 | `error: edge host is required: pass --edge-host or set AG_EDGE_HOST` |
| `ag domains attach app.example.com --edge-host edge.example.com --state PRIVATE` | 0 | 87 | prints CNAME + ownership TXT instructions and `Status: pending_ownership` |
| `ag deploy plan --target k8s` | 0 | (timed separately below) | 26 files, `No files were written. Review, then: ag deploy apply --plan ...` |
| `ag deploy apply --plan <DPLID>` | 0 | 127 | `Applied plan ... (26 files) under deploy/generated/k8s.` + state/audit paths |
| `ag deploy status` | 0 | 109 | 26 generated files, `Sources available: yes`, `In sync with current sources: yes` |
| `ag deploy apply` without `--plan` | 2 | 65 | clap: `the following required arguments were not provided: --plan <PLAN>` |
| `ag deploy apply --plan dpl-k8s-0000000000000000` | 1 | 61 | `error: plan ... is stale: current plan is ...; re-run ag deploy plan ...` |

All mutating commands refused to run without the exact plan id (exit 2 for a
missing id, exit 1 for a wrong/stale id). This is the desired plan/apply gate.

`ag dns plan` prose output (exit 0):

```text
Plan:        dns-production-e533df963b744233
Domain:      app.example.com
Zone:        example.com
Environment: production
Visibility:  public
Project:     dnsdeploy-proj
Service:     dnsdeploy-proj
Target:      service
TLS:         managed_http01

Records (1):
Type   Name                              Value                                 TTL  Ownership
TXT    _ag-domain.app.example.com        ag-verification=1bfe5f4d2bec8827     300  yes
```

Friction: for a `public` binding with `managed_http01`, the plan emits only the
ownership TXT; there is no A/CNAME record pointing at an edge. The separate
`ag domains attach` command does emit the CNAME. A user may reasonably expect
the DNS plan to include the public address record described in
`docs/dns/DOMAIN_LOOP.md` section 5. Recorded as friction, not filed as a
defect, because the plan contract in RFC-0033 lists only validated record
sets and the attachment is a separate step.

## 4. `--output json` read forms

All JSON responses use `contract_version: 0.2.0` and `status: ok` on success.

| Command | Exit | ms | Shape |
| --- | --- | --- | --- |
| `ag --output json env list` | 0 | 66 | `env.list`, 2 variables, no values |
| `ag env check --output json` | 1 | 93 | `env.check`, `ok:false`, `resolved_count:1`, problem `DATABASE_URL`/`missing` |
| `ag dns plan --output json` | 0 | 64 | `dns.plan`, `plan_id`, `records[1]` |
| `ag dns status --output json` | 0 | 75 | `dns.status`, `applied:true`, `in_sync:true` |
| `ag deploy plan --target k8s --output json` | 0 | 67 | `deploy.plan`, 26 `files[]` with bytes/fingerprint |
| `ag --output json deploy status` | 0 | 103 | `deploy.status`, 26 paths, `current_plan_id`, `in_sync:true` |

Note: `ag env check --output json` exits 1 on validation failure while its
envelope still says `status:ok` and places the failure in `data.ok:false`.
This is internally consistent (transport ok, validation failed) but a machine
consumer must read both fields. Recorded as friction.

Refusals (all exit 2, before any work):

| Command | Result |
| --- | --- |
| `ag --output json config get bind` | `unsupported_command` JSON error |
| `ag --output json dns apply --plan <DNSID>` | same refusal |
| `ag --output json deploy apply --plan <DPLID>` | same refusal |

Each refusal message lists: `schema.lint, schema.diff, workers.list,
deploy.plan, deploy.status, dns.plan, dns.status`. It omits `env.list` and
`env.check`, which do support JSON. Filed as issue #201.

## 5. Secret and credential handling

- `DATABASE_URL=postgresql://user:CANARY_SECRET_VALUE@db.internal:5432/app`
  was exported for one `ag env check` run. A scan of every captured
  stdout/stderr file in this journey found no occurrence of the canary or of
  the connection string fragment (`grep -rln` returned no matches).
- `ag env list` prose and JSON report only name/type/flags/source/presence.
  No value, including defaults, is printed.
- The private DNS state file contains declarative fields only: plan id,
  domain, zone, environment, visibility, project, service, tls and the public
  ownership TXT. No token, zone id or credential.
- The domains attach store contains the ownership TXT value
  (`ag-verification=dom_...`), which is a deliberately public DNS record, not
  a credential. No provider token was supplied or stored.
- `ag dns apply` used the default `--provider none` and printed that no
  network access occurred.

## 6. Confusing or contradictory messages (with evidence)

1. `ag --help` and `ag dns --help` show the `dns` description as
   `passed explicitly. See docs/dns/DOMAIN_LOOP.md` - a truncated doc comment
   with no subject. Filed as issue #200.
2. The JSON `unsupported_command` message omits two commands that actually
   support JSON (`env.list`, `env.check`). Filed as issue #201.
3. `docs/dns/DOMAIN_LOOP.md` worked example uses five invocations the CLI
   rejects: positional `dns apply <id>`, `dns apply --rollback`,
   `dns instructions`, `dns status --online`, `domains verify --online`.
   Its header also says the commands are "not yet implemented". Observed exit
   codes were all 2 with clap "unexpected argument"/"unrecognized subcommand"
   errors. Filed as issue #202.
4. Minor: the generated `config.toml` from the `rest` template carries Spanish
   comments while the rest of the product follows the English-canonical
   policy (ADR-0008). No issue filed; it is a generated starter file.

## 7. What works well

- All read commands are fast (36-153 ms) and predictable; no command hung or
  attempted network access in offline mode.
- Both apply gates are strict: missing plan id is a clap error (exit 2) and a
  wrong/stale id is a drift/stale error (exit 1) before any write.
- `ag dns apply --provider none` clearly states that it is local-only.
- `ag domains attach` produces copy-paste DNS instructions and refuses a
  missing edge host.
- `ag deploy plan`/`apply`/`status` round-trip deterministically: the same
  plan id is recomputed and reported as in sync after apply.
- Redaction holds for declared secrets in both prose and JSON.

## 8. Issues opened (none closed)

- #200 - ag dns help text is a truncated fragment (bug, ag-cli, p3, good first issue)
- #201 - JSON unsupported-command message omits env.list and env.check (bug, ag-cli, p3)
- #202 - DOMAIN_LOOP.md worked example uses commands the CLI rejects (documentation, ag-cli, ag-domains, p2)

## 9. Verification of this document

- All outputs in this file were copied from real command execution on the
  environment in section 1; none were simulated.
- No push, no merge, no shared source edits were made. Only this audit, the
  CHANGELOG entry and the branch descriptor were changed in the worktree.
