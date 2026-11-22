# Machine-readable interfaces

This directory holds the versioned contract data for machine consumers of the
`ag` binary. It is data, not code: no MCP server or agent protocol lives here.

| File | Purpose |
| --- | --- |
| `cli-json-contract-0.2.0.md` | Current human-readable CLI JSON contract. |
| `cli-json-0.2.0.schema.json` | Current JSON Schema (draft 2020-12) for `--output json`. |
| `cli-json-contract-0.1.0.md` | Superseded contract, kept as a historical artifact. |
| `cli-json-0.1.0.schema.json` | Superseded schema, kept as a historical artifact. |
| `snapshots/` | Golden outputs asserted by `crates/ag-cli/tests/json_output.rs`. |

## Current state

RFC-0021 stage E is being implemented incrementally. The current contract
version is `0.2.0` (unstable until Anti-Gravital 0.5) and covers five read-only
commands: `schema.lint`, `schema.diff`, `workers.list`, `dns.plan` and
`dns.status`. The last two were added by the domain-to-deploy loop (RFC-0033).

The following RFC-0024 section 9 prerequisites are **not** satisfied by this
contract alone and remain tracked in `docs/audits/mcp-readiness-review.md`:

- `--plan` / `--apply` and the shared audit sink (prerequisite 1, partial).
- `Secret<T>` in the CLI executors (prerequisite 2).
- Typed request/executor pairs for every command group (prerequisite 3).
- The remaining design decisions (prerequisites 4-7).

No `contract_version` bump may be made without updating the JSON Schema, the
contract document, the `CONTRACT_VERSION` constant and the snapshots together.
The product owner of this version is Anti-Gravital; Gravital Fabric publishes
its own independently versioned contract under `docs/mcp/` in its repository
(ADR-0016).
