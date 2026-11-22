# CLI JSON contract 0.1.0

- Status: unstable (pre-0.5). Versioned from 0.5 per the RFC-0021 risk table.
- Owner: Anti-Gravital (this repository). `contract_version` is bumped here and
  mirrored in `crates/ag-cli/src/output.rs` (`CONTRACT_VERSION`).
- Machine schema: `docs/mcp/cli-json-0.1.0.schema.json`.
- Governing documents: RFC-0021 section 4.3, RFC-0024 section 4.2 and section 9,
  ADR-0008 (English canonical), ADR-0011 (no third-party brand names).

## 1. Scope

`ag --output json` is the machine-readable output of **read-only** commands.
The flag is global and may appear before or after the subcommand. Without it,
every command keeps its existing prose output byte-for-byte.

This contract version covers exactly three commands:

| Command | Name in `command` | Reads |
| --- | --- | --- |
| `ag schema lint [--schema PATH]` | `schema.lint` | one DSL schema file |
| `ag schema diff REF [--schema PATH]` | `schema.diff` | two DSL schema files |
| `ag workers list [--schema PATH]` | `workers.list` | one DSL schema file |

Other commands have no versioned shape yet. Passing `--output json` to them is
a usage error (exit code 2) produced before dispatch; no file, database or
network operation is performed. The refusal is itself a versioned JSON error,
so a script never receives prose when it asked for JSON.

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
| `contract_version` | string | SemVer of this contract. `0.1.0` in this version. |
| `command` | string | Stable name from the table above, or `unknown` for a refusal. |
| `status` | `ok` \| `error` | Discriminator. |
| `data` | object | Present only when `status` is `ok`. Command-specific shape below. |
| `error` | object | Present only when `status` is `error`. `{ code, message }`. |

Error codes in this version:

| Code | Exit code | Meaning |
| --- | --- | --- |
| `unsupported_command` | 2 | `--output json` was used with a command that has no versioned shape. |
| `read_failed` | 1 | The read itself failed (for example the schema file is missing or invalid). |

Exit codes in JSON mode: `0` success, `1` read or validation failure, `2`
usage error. This is the beginning of the RFC-0021 exit-code table, not the
full table: the remaining categories (precondition, remote failure) are not
defined until their commands exist. In prose mode existing exit codes are
unchanged.

## 3. Command shapes

### 3.1 `schema.lint`

```json
{"contract_version":"0.1.0","command":"schema.lint","status":"ok","data":{"schema":"schema.ag","result":"clean","error_count":0,"warning_count":0,"diagnostics":[]}}
```

`data.result` is `clean`, `warnings` or `errors`. `errors` still emits a
complete document with `status: ok` and exits 1: the diagnostics are data, and
the exit code carries the validation failure. Diagnostics carry `severity`,
`message`, `hint` (nullable), `span_start` and `span_end` (byte offsets, UTF-8
safe as the DSL already uses byte spans).

### 3.2 `schema.diff`

```json
{"contract_version":"0.1.0","command":"schema.diff","status":"ok","data":{"reference":"reference.ag","schema":"schema.ag","changes":[{"severity":"breaking","message":"model 'Old' removed"}]}}
```

`severity` is `breaking` or `additive`. The prose `[BREAKING]` and `[additive]`
prefixes are derived from the same structured change list, so the two outputs
cannot drift.

### 3.3 `workers.list`

```json
{"contract_version":"0.1.0","command":"workers.list","status":"ok","data":{"schema":"schema.ag","workers":[{"name":"Mailer","queue":"mail","mode":"static","concurrency":4,"input":["to"],"retry_max_attempts":3}]}}
```

`queue` defaults to `default`, `mode` defaults to `static`, and absent
`concurrency` or `retry_max_attempts` are `null`. Only local schema data is
read: no database connection and no `DATABASE_URL`.

### 3.4 Refusal

```json
{"contract_version":"0.1.0","command":"unknown","status":"error","error":{"code":"unsupported_command","message":"--output json is not supported by this command in contract version 0.1.0; supported commands: schema.lint, schema.diff, workers.list"}}
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
code, plan/apply framework or audit sink is implemented here. This is only the
versioned read-output substrate that a later MCP implementation can reuse. See
RFC-0024 section 9 for the remaining prerequisites.
