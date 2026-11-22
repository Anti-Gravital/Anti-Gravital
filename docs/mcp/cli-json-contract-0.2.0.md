# CLI JSON contract 0.2.0

- Status: unstable (pre-0.5). Versioned from 0.5 per the RFC-0021 risk table.
- Owner: Anti-Gravital (this repository). `contract_version` is defined by the
  `CONTRACT_VERSION` constant in `crates/ag-cli/src/output.rs` and mirrored in
  the schema and snapshots under `docs/mcp/`.
- Machine schema: `docs/mcp/cli-json-0.2.0.schema.json`.
- Governing documents: RFC-0021 section 4.3, RFC-0018 (`ag config`), RFC-0024
  section 4.2 and section 9, RFC-0033 (domain loop), RFC-0034 (local deploy),
  ADR-0008 (English canonical), ADR-0011 (no third-party brand names).
- Supersedes `cli-json-contract-0.1.0.md`, kept as a historical record.

## 1. Scope

`ag --output json` is the machine-readable output of **read-only** commands.
The flag is global and may appear before or after the subcommand. Without it,
every command keeps its existing prose output byte-for-byte.

This contract version declares ten shapes. Seven are emitted by the CLI:

| Command | Name in `command` | Reads |
| --- | --- | --- |
| `ag schema lint [--schema PATH]` | `schema.lint` | one DSL schema file |
| `ag schema diff REF [--schema PATH]` | `schema.diff` | two DSL schema files |
| `ag workers list [--schema PATH]` | `workers.list` | one DSL schema file |
| `ag deploy plan --target docker\|k8s` | `deploy.plan` | authored files under `deploy/docker/` or `deploy/k8s/` |
| `ag deploy status` | `deploy.status` | local state under `deploy/generated/` |
| `ag dns plan [--manifest PATH] [--env ENV]` | `dns.plan` | a local domain manifest |
| `ag dns status [--manifest PATH] [--state PATH] [--env ENV]` | `dns.status` | local state and manifest |

Three more are **defined contract fixtures**, not yet emitted:

| Command | Name in `command` | Reads | Status |
| --- | --- | --- | --- |
| `ag config get KEY` | `config.get` | `config.toml` and compiled-in defaults | shape and schema published in 0.2.0; CLI dispatch wiring pending |
| `ag env list` | `env.list` | declared env variables in `ag.toml` and process presence | shape and schema published in 0.2.0; CLI dispatch wiring pending |
| `ag env check` | `env.check` | declared env variables and their parseability | shape and schema published in 0.2.0; CLI dispatch wiring pending |

These shapes are additive: no existing field, enum or command shape changed,
so `contract_version` stays `0.2.0`. Their JSON Schema definitions and snapshot
fixtures are published so consumers can prepare against the final shapes, but
until the dispatch mapping is added, `--output json` with these commands
returns `unsupported_command` (exit 2). The fixtures are validated by
`crates/ag-cli/tests/json_output.rs`; they are not captured CLI documents.
Emission requires small changes in files owned by another wave-9 role:
`json_command_name()` in `cli.rs` and `run_json_command()` in `main.rs`. Those
changes are wiring only: no prose output and no mutating behaviour may change
(RFC-0024 prerequisite 3).

Commands with no shape at all are refused with `unsupported_command` before
dispatch; no file, database or network operation is performed. In particular
`ag deploy apply` and `ag dns apply` are mutating and always refuse the flag.
`project.list`, `project.describe`, `schema.describe` and all `debug.*`,
`domains.*`, `mail.*` and `workers.*` runtime shapes remain undefined: they
have no CLI counterpart or require an RFC decision (RFC-0024 Q4, audit section
12.3). This is not an implicit promise of future tools.

## 2. Envelope

Every document is one compact JSON line on stdout with these fields in this
order. Field order is part of the snapshot contract.

| Field | Type | Meaning |
| --- | --- | --- |
| `contract_version` | string | SemVer of this contract. `0.2.0` in this version. |
| `command` | string | Stable name from the tables above, or `unknown` for a refusal. |
| `status` | `ok` \| `error` | Discriminator. |
| `data` | object | Present only when `status` is `ok`. Command-specific shape below. |
| `error` | object | Present only when `status` is `error`. `{ code, message }`. |

Error codes in this version:

| Code | Exit code | Meaning |
| --- | --- | --- |
| `unsupported_command` | 2 | `--output json` was used with a command that has no emitted shape. |
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

### 3.6 `dns.plan`

```json
{"contract_version":"0.2.0","command":"dns.plan","status":"ok","data":{"plan_id":"dns-production-43b9759da593b763","domain":"api.example.com","zone":"example.com","environment":"production","visibility":"public","project":"demo","service":"api","target_kind":"service","tls":"managed_http01","records":[{"record_type":"CNAME","name":"api.example.com","value":"edge.example-cloud.net","ttl":300,"is_ownership":false},{"record_type":"TXT","name":"_ag-domain.api.example.com","value":"ag-verification=72b9827f7999fd6b","ttl":300,"is_ownership":true}],"notes":[]}}
```

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

### 3.7 `dns.status`

```json
{"contract_version":"0.2.0","command":"dns.status","status":"ok","data":{"environment":"prod","applied":false,"plan_id":null,"domain":null,"visibility":null,"provider":null,"record_count":0,"in_sync":false}}
```

`applied: false` means no local state exists for the environment; nullable
fields are `null` in that case. `in_sync` compares the applied plan id with a
fresh computation from the manifest; it is `false` when either is absent.
`dns.status` never contacts a provider.

### 3.8 `config.get` (defined fixture, emission pending)

```json
{"contract_version":"0.2.0","command":"config.get","status":"ok","data":{"key":"bind","kind":"bind","value":"127.0.0.1:8080","source":"file"}}
```

| Field | Meaning |
| --- | --- |
| `key` | Canonical dotted RFC-0018 config key, e.g. `bind`, `rate_limit.per_ip_rps`. Command input is parsed against the closed key set before it can reach this field. |
| `kind` | One of `bool`, `integer`, `string`, `string_list`, `bind`. Consumers parse `value` with this hint; the prose path prints the same plain value. |
| `value` | The resolved value as data. Never a field name and never interpreted by the serializer. |
| `source` | `file` when the key was present in `config.toml`, `default` when the compiled-in default was used. |

The RFC-0018 key set deliberately excludes inline secret material: it exposes
paths and non-secret settings only. This shape therefore contains no token,
password or PEM body, and command input cannot introduce a new field because
`additionalProperties` is `false` and `key`/`kind` are drawn from closed sets.
See section 1 for the dispatch wiring boundary.

### 3.9 `env.list` (defined fixture, emission pending)

```json
{"contract_version":"0.2.0","command":"env.list","status":"ok","data":{"manifest":"ag.toml","environment":null,"variables":[{"name":"DATABASE_URL","type":"url","required":true,"secret":true,"has_default":false,"source":"process","present":true}]}}
```

| Field | Meaning |
| --- | --- |
| `manifest` | Manifest path as displayed to the user, normally `ag.toml`. |
| `environment` | Selected environment name, or `null` when none is selected. |
| `variables` | Declared variables, in manifest order. One entry per declaration. |
| `variables[].name` | Declared variable name. Never the value. |
| `variables[].type` | One of `string`, `int`, `bool`, `url`, `duration`, `path` (the `EnvType` display names from `ag-core`). |
| `variables[].required` | Whether a value or default is required. |
| `variables[].secret` | Whether the declaration is marked secret. |
| `variables[].has_default` | Whether a default exists. The default itself is never emitted. |
| `variables[].source` | `process`, `default` or `none` - where a value would come from. |
| `variables[].present` | Whether the process variable is currently set. Membership only; no value read. |

This shape contains no value, default value or raw environment content. It is
computed from declarations plus `std::env::var_os` presence, which avoids ever
loading a secret into the output path (RFC-0020, RFC-0024 section 4.1.5).

### 3.10 `env.check` (defined fixture, emission pending)

```json
{"contract_version":"0.2.0","command":"env.check","status":"ok","data":{"manifest":"ag.toml","environment":"dev","ok":false,"resolved_count":2,"problems":[{"name":"MISSING","type":"url","reason":"missing"}]}}
```

| Field | Meaning |
| --- | --- |
| `manifest` | Manifest path as displayed to the user, normally `ag.toml`. |
| `environment` | Selected environment name, or `null` when none is selected. |
| `ok` | `true` when every declared variable resolves and parses. |
| `resolved_count` | Number of declarations that resolved successfully. |
| `problems` | Failed declarations, in manifest order. Empty when `ok` is `true`. |
| `problems[].name` | Declared variable name. Never the value. |
| `problems[].type` | Same closed type set as `env.list`. |
| `problems[].reason` | One of `missing`, `invalid_syntax`, `not_unicode`. |

Problem reasons are stable codes with no interpolated text, so an invalid
value can never enter a message (RFC-0024 prompt-injection risk). See section 1
for the dispatch wiring boundary.

### 3.11 Refusal

```json
{"contract_version":"0.2.0","command":"unknown","status":"error","error":{"code":"unsupported_command","message":"--output json is not supported by this command in contract version 0.2.0; supported commands: schema.lint, schema.diff, workers.list, deploy.plan, deploy.status, dns.plan, dns.status"}}
```

The message is a closed, constant string. It never includes arguments, paths,
environment values or secret material, even when the refused command would
have accepted them (RFC-0020 redaction, RFC-0024 section 4.1.5). The list names
emitted commands only, so it does not mention `config.get`, `env.list` or
`env.check` while those shapes are fixtures.

## 4. Stability and change policy

1. Before Anti-Gravital 0.5 the contract is unstable but versioned: consumers
   must pin `contract_version` and must not assume fields will be added
   compatibly.
2. Any field rename, removal, type change, newly required field or changed
   enum value on an **existing** shape is an incompatible change and bumps
   `contract_version`. Use the smallest bump that fits (an additive 0.2.x patch
   with an explicit `contract_version` in every document), not a major bump.
3. Adding a brand-new command shape does not bump the version while the contract
   is pre-0.5, provided no existing shape changes. The new `command` value and
   its `data` definition are added to the same versioned schema and the
   snapshots are updated in the same commit. From 0.5 onward, new command
   shapes bump the minor version.
4. A shape that is documented but not emitted is a contract fixture, not a
   supported command. It must be labelled as such in this document and in the
   schema description, and it must not appear in the refusal message's
   supported-command list. Emitting it is a separate change.
5. Snapshots live in `docs/mcp/snapshots/` and are compared by
   `crates/ag-cli/tests/json_output.rs`. A change to emitted JSON requires a
   reviewed snapshot change in the same commit.
6. Prose output is not part of this contract and may change freely.

## 5. Non-goals

No MCP server, transport, capability manifest, tool registry, agent protocol
code, audit sink for other mutating commands or cluster deployment is
implemented here. This is only the versioned read-output substrate plus the
local renderers of RFC-0021 stage E, RFC-0033 and RFC-0034. See RFC-0024
section 9 and `docs/audits/mcp-readiness-review.md` for the remaining
prerequisites.
