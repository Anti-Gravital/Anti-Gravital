# CLI JSON contract 0.2.0

- Status: unstable (pre-0.5). Versioned from 0.5 per the RFC-0021 risk table.
- Owner: Anti-Gravital (this repository). `contract_version` is bumped here and
  mirrored in `crates/ag-cli/src/output.rs` (`CONTRACT_VERSION`).
- Machine schema: `docs/mcp/cli-json-0.2.0.schema.json`.
- Governing documents: RFC-0021 section 4.3, RFC-0024 section 4.2 and section 9,
  RFC-0033 (domain-to-deploy loop), ADR-0008 (English canonical), ADR-0011 (no
  third-party brand names).
- Predecessor: `cli-json-contract-0.1.0.md`. The 0.1.0 document and schema stay
  in the repository as historical artifacts.

## 1. Scope

`ag --output json` is the machine-readable output of **read-only** commands.
The flag is global and may appear before or after the subcommand. Prose output
is unchanged when the flag is absent. Mutating commands refuse JSON before any
dispatch: no file, network or provider operation is performed.

This version adds the two read-only domain-loop commands to the three covered
by 0.1.0:

| Command | Name in `command` | Reads |
| --- | --- | --- |
| `ag schema lint [--schema PATH]` | `schema.lint` | one DSL schema file |
| `ag schema diff REF [--schema PATH]` | `schema.diff` | two DSL schema files |
| `ag workers list [--schema PATH]` | `workers.list` | one DSL schema file |
| `ag dns plan [--manifest PATH]` | `dns.plan` | a local domain manifest |
| `ag dns status [--manifest PATH] [--state PATH] [--environment ENV]` | `dns.status` | local state and manifest |

`dns.apply` and all `domains.*` commands remain mutating and refuse JSON.
Commands with no versioned shape are refused with `unsupported_command` before
dispatch; the refusal message is a closed constant that never echoes an
argument, path, environment value or secret.

## 2. Envelope

Unchanged from 0.1.0: one compact JSON line with `contract_version`, `command`,
`status` and exactly one of `data` or `error`. Field order is part of the
contract. `contract_version` is `0.2.0` in this version.

Error codes: `unsupported_command` (exit 2), `read_failed` (exit 1). JSON-mode
exit codes are 0, 1 and 2. Prose-mode exit codes are unchanged.

## 3. New command shapes

### 3.1 `dns.plan`

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

### 3.2 `dns.status`

```json
{"contract_version":"0.2.0","command":"dns.status","status":"ok","data":{"environment":"prod","applied":false,"plan_id":null,"domain":null,"visibility":null,"provider":null,"record_count":0,"in_sync":false}}
```

`applied: false` means no local state exists for the environment; nullable
fields are `null` in that case. `in_sync` compares the applied plan id with a
fresh computation from the manifest; it is `false` when either is absent.
`dns.status` never contacts a provider.

## 4. Stability and change policy

Rules of 0.1.0 still apply: any field rename, removal, type change, newly
required field or changed enum value bumps `contract_version`; snapshots under
`docs/mcp/snapshots/` are byte-exact and change in the same commit. Prose output
is not part of the contract.

## 5. Non-goals

RFC-0033 sketches a richer plan document with `contract_version = "1"` and
`deploy.actions`, `ownership`, `acme` and online-step fields. That shape is
still proposed. This contract keeps the RFC-0021 envelope as the single
versioned machine boundary; the RFC-0033 fields can be added in a later minor
version after the RFC is accepted, without a second plan model.

No MCP server, transport or tool registry is added by this version. `dns.apply`
follows RFC-0021 section 4.3 with a mandatory plan id, but the shared audit
sink, `Secret<T>`, typed requests for every command and `ag describe` remain
open prerequisites (see `docs/audits/mcp-readiness-review.md`).
