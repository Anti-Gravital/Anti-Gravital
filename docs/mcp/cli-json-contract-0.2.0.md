# CLI JSON contract 0.2.0

- Status: unstable (pre-0.5). Versioned from 0.5 per the RFC-0021 risk table.
- Owner: Anti-Gravital (this repository). `contract_version` is bumped here and
  mirrored in `crates/ag-cli/src/output.rs` (`CONTRACT_VERSION`).
- Machine schema: `docs/mcp/cli-json-0.2.0.schema.json`.
- Governing documents: RFC-0021 section 4.3, RFC-0024 section 4.2 and section 9,
  RFC-0034 (local `ag deploy`), ADR-0008 (English canonical), ADR-0011 (no
  third-party brand names).
- Supersedes `cli-json-contract-0.1.0.md`, kept as a historical record.

## 1. Scope

`ag --output json` is the machine-readable output of **read-only** commands.
The flag is global and may appear before or after the subcommand. Without it,
every command keeps its existing prose output byte-for-byte.

This contract version covers exactly five commands:

| Command | Name in `command` | Reads |
| --- | --- | --- |
| `ag schema lint [--schema PATH]` | `schema.lint` | one DSL schema file |
| `ag schema diff REF [--schema PATH]` | `schema.diff` | two DSL schema files |
| `ag workers list [--schema PATH]` | `workers.list` | one DSL schema file |
| `ag deploy plan --target docker\|k8s` | `deploy.plan` | authored files under `deploy/docker/` or `deploy/k8s/` |
| `ag deploy status` | `deploy.status` | local state under `deploy/generated/` |

Other commands have no versioned shape yet. Passing `--output json` to them is
a usage error (exit code 2) produced before dispatch; no file, database or
network operation is performed. The refusal is itself a versioned JSON error,
so a script never receives prose when it asked for JSON. In particular
`ag deploy apply` is mutating and always refuses the flag.

Commands absent from this version: `config.get`, `project.list`,
`project.describe`, `schema.describe`, `ag describe`, all `domains.*`,
`workers.*` runtime commands and every mutating command. They need their own
contract version and, where a command does not exist yet, their own RFC-stage
work. This is not an implicit promise of future tools (RFC-0024 prerequisites
remain open; see `docs/audits/mcp-readiness-review.md`).

## 2. Envelope

Every document is one compact JSON line on stdout with these fields in this
order. Field order is part of the snapshot contract.

| Field | Type | Meaning |
| --- | --- | --- |
| `contract_version` | string | SemVer of this contract. `0.2.0` in this version. |
| `command` | string | Stable name from the table above, or `unknown` for a refusal. |
| `status` | `ok` \| `error` | Discriminator. |
| `data` | object | Present only when `status` is `ok`. Command-specific shape below. |
| `error` | object | Present only when `status` is `error`. `{ code, message }`. |

Error codes in this version:

| Code | Exit code | Meaning |
| --- | --- | --- |
| `unsupported_command` | 2 | `--output json` was used with a command that has no versioned shape. |
| `read_failed` | 1 | The read itself failed (missing or invalid input, missing deployment source set). |

Exit codes in JSON mode: `0` success, `1` read or validation failure, `2`
usage error. This is the beginning of the RFC-0021 exit-code table, not the
full table: the remaining categories (precondition, remote failure) are not
defined until their commands exist. In prose mode existing exit codes are
unchanged.

## 3. Command shapes

### 3.1 `schema.lint`

```json
{"contract_version":"0.2.0","command":"schema.lint","status":"ok","data":{"schema":"schema.ag","result":"clean","error_count":0,"warning_count":0,"diagnostics":[]}}
```

`data.result` is `clean`, `warnings` or `errors`. `errors` still emits a
complete document with `status: ok` and exits 1: the diagnostics are data, and
the exit code carries the validation failure. Diagnostics carry `severity`,
`message`, `hint` (nullable), `span_start` and `span_end` (byte offsets, UTF-8
safe as the DSL already uses byte spans).

### 3.2 `schema.diff`

```json
{"contract_version":"0.2.0","command":"schema.diff","status":"ok","data":{"reference":"reference.ag","schema":"schema.ag","changes":[{"severity":"breaking","message":"model 'Old' removed"}]}}
```

`severity` is `breaking` or `additive`. The prose `[BREAKING]` and `[additive]`
prefixes are derived from the same structured change list, so the two outputs
cannot drift.

### 3.3 `workers.list`

```json
{"contract_version":"0.2.0","command":"workers.list","status":"ok","data":{"schema":"schema.ag","workers":[{"name":"Mailer","queue":"mail","mode":"static","concurrency":4,"input":["to"],"retry_max_attempts":3}]}}
```

`queue` defaults to `default`, `mode` defaults to `static`, and absent
`concurrency` or `retry_max_attempts` are `null`. Only local schema data is
read: no database connection and no `DATABASE_URL`.

### 3.4 `deploy.plan`

```json
{"contract_version":"0.2.0","command":"deploy.plan","status":"ok","data":{"target":"docker","project":"demo","plan_id":"dpl-docker-1f4c9a02b3d5e6f7","files":[{"path":"deploy/generated/docker/Dockerfile","source":"deploy/docker/Dockerfile","bytes":312,"fingerprint":"9a1b2c3d4e5f6071"}]}}
```

Read-only. `target` is `docker` or `k8s`. `files` is sorted by destination
path and describes each artifact by destination `path`, `source` path, `bytes`
and a 16-hex-digit FNV-1a fingerprint of the source bytes. `plan_id` is
`dpl-<target>-<16 hex digits>` and changes when the source set, file contents
or detected project name changes. No file is written. A missing source set is
a versioned `read_failed` error (exit 1).

### 3.5 `deploy.status`

```json
{"contract_version":"0.2.0","command":"deploy.status","status":"ok","data":{"generated":true,"target":"docker","plan_id":"dpl-docker-1f4c9a02b3d5e6f7","project":"demo","files":["deploy/generated/docker/Dockerfile"],"current_plan_id":"dpl-docker-1f4c9a02b3d5e6f7","in_sync":true,"sources_available":true}}
```

Read-only. When nothing has been applied, `generated` is `false`, the scalar
fields are `null`, `files` is empty and `in_sync` is `false`. `sources_available`
reports whether the current source set can be read; `in_sync` is true only
when the recorded plan id equals the plan id recomputed from current sources.
Both plan and status make no network, cluster or container-runtime call
(RFC-0034).

### 3.6 Refusal

```json
{"contract_version":"0.2.0","command":"unknown","status":"error","error":{"code":"unsupported_command","message":"--output json is not supported by this command in contract version 0.2.0; supported commands: schema.lint, schema.diff, workers.list, deploy.plan, deploy.status"}}
```

The message is a closed, constant string. It never includes arguments, paths,
environment values or secret material, even when the refused command would
have accepted them (RFC-0020 redaction, RFC-0024 section 4.1.5).

## 4. Stability and change policy

1. Before Anti-Gravital 0.5 the contract is unstable but versioned: consumers
   must pin `contract_version` and must not assume fields will be added
   compatibly.
2. Any field rename, removal, type change, newly required field or changed
   enum value bumps `contract_version` (minor for additive-compatible
   changes, major for breaking ones).
3. Adding a new command shape bumps the minor version; the new `command` value
   and its `data` `$defs` are added to the JSON Schema in the same change.
4. Snapshots live in `docs/mcp/snapshots/` and are compared by
   `crates/ag-cli/tests/json_output.rs`. A change to emitted JSON requires a
   reviewed snapshot change in the same commit.
5. Prose output is not part of this contract and may change freely.

## 5. Non-goals

No MCP server, transport, capability manifest, tool registry, agent protocol
code, audit sink for other mutating commands or cluster deployment is
implemented here. This is only the versioned read-output substrate plus the
local renderer of RFC-0021 stage E and RFC-0034. See RFC-0024 section 9 for the
remaining prerequisites.
  RFC-0033 (domain-to-deploy loop), ADR-0008 (English canonical), ADR-0011 (no
- Predecessor: `cli-json-contract-0.1.0.md`. The 0.1.0 document and schema stay
  in the repository as historical artifacts.
The flag is global and may appear before or after the subcommand. Prose output
is unchanged when the flag is absent. Mutating commands refuse JSON before any
dispatch: no file, network or provider operation is performed.
This version adds the two read-only domain-loop commands to the three covered
by 0.1.0:
| `ag dns plan [--manifest PATH]` | `dns.plan` | a local domain manifest |
| `ag dns status [--manifest PATH] [--state PATH] [--environment ENV]` | `dns.status` | local state and manifest |
`dns.apply` and all `domains.*` commands remain mutating and refuse JSON.
Commands with no versioned shape are refused with `unsupported_command` before
dispatch; the refusal message is a closed constant that never echoes an
argument, path, environment value or secret.
Unchanged from 0.1.0: one compact JSON line with `contract_version`, `command`,
`status` and exactly one of `data` or `error`. Field order is part of the
contract. `contract_version` is `0.2.0` in this version.
Error codes: `unsupported_command` (exit 2), `read_failed` (exit 1). JSON-mode
exit codes are 0, 1 and 2. Prose-mode exit codes are unchanged.
## 3. New command shapes
### 3.1 `dns.plan`
{"contract_version":"0.2.0","command":"dns.plan","status":"ok","data":{"plan_id":"dns-production-43b9759da593b763","domain":"api.example.com","zone":"example.com","environment":"production","visibility":"public","project":"demo","service":"api","target_kind":"service","tls":"managed_http01","records":[{"record_type":"CNAME","name":"api.example.com","value":"edge.example-cloud.net","ttl":300,"is_ownership":false},{"record_type":"TXT","name":"_ag-domain.api.example.com","value":"ag-verification=72b9827f7999fd6b","ttl":300,"is_ownership":true}],"notes":[]}}
Fields:
| Field | Meaning |
| --- | --- |
| `plan_id` | `dns-<env>-<16 lowercase hex>`, FNV-1a over a length-prefixed canonical stream. An identity, not a security primitive. |
| `domain`, `zone` | Canonical ASCII host from `[domain].domain`; the zone is declared or derived via the single registrable-domain source. |
| `environment` | `dev`, `staging`, `production` or `local`; `prod` is normalized to `production`. |
| `visibility` | `public` or `private`; private is the safe default and drops all public routing records. |
| `project`, `service` | `[project].name` and `[domain].target.ref`; never guessed. |
| `target_kind`, `tls` | `service` and the resolved TLS mode. `link` is specified by RFC-0033 but not implemented. |
| `records` | Validated records, sorted deterministically; `is_ownership` marks the derived verification TXT record. |
| `notes` | Deterministic advisories (for example private visibility or local binding). |
No credential, zone id, token or absolute filesystem path appears in this
shape. A missing or invalid manifest is `read_failed` with exit 1.
### 3.2 `dns.status`
{"contract_version":"0.2.0","command":"dns.status","status":"ok","data":{"environment":"prod","applied":false,"plan_id":null,"domain":null,"visibility":null,"provider":null,"record_count":0,"in_sync":false}}
`applied: false` means no local state exists for the environment; nullable
fields are `null` in that case. `in_sync` compares the applied plan id with a
fresh computation from the manifest; it is `false` when either is absent.
`dns.status` never contacts a provider.
Rules of 0.1.0 still apply: any field rename, removal, type change, newly
required field or changed enum value bumps `contract_version`; snapshots under
`docs/mcp/snapshots/` are byte-exact and change in the same commit. Prose output
is not part of the contract.
RFC-0033 sketches a richer plan document with `contract_version = "1"` and
`deploy.actions`, `ownership`, `acme` and online-step fields. That shape is
still proposed. This contract keeps the RFC-0021 envelope as the single
versioned machine boundary; the RFC-0033 fields can be added in a later minor
version after the RFC is accepted, without a second plan model.
No MCP server, transport or tool registry is added by this version. `dns.apply`
follows RFC-0021 section 4.3 with a mandatory plan id, but the shared audit
sink, `Secret<T>`, typed requests for every command and `ag describe` remain
open prerequisites (see `docs/audits/mcp-readiness-review.md`).
