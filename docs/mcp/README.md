# Machine-readable interfaces

This directory holds the versioned contract data for machine consumers of the
`ag` binary. It is data, not code: no MCP server or agent protocol lives here.

| File | Purpose |
| --- | --- |
| `cli-json-contract-0.2.0.md` | Current human-readable versioned CLI JSON contract. |
| `cli-json-0.2.0.schema.json` | Current JSON Schema (draft 2020-12) for `--output json`. |
| `cli-json-contract-0.1.0.md`, `cli-json-0.1.0.schema.json` | Historical 0.1.0 record. |
| `snapshots/` | Golden outputs and contract fixtures asserted by `crates/ag-cli/tests/json_output.rs`. |

## Current state

RFC-0021 stage E is being implemented incrementally. This directory covers
`ag --output json` for ten emitted read-only commands: `config.get`,
`schema.lint`, `schema.diff`, `workers.list`, `deploy.plan`, `deploy.status`,
`dns.plan`, `dns.status`, `env.list` and `env.check`. The contract version is
`0.2.0` and is unstable until Anti-Gravital 0.5. The `deploy.*` shapes are
local artifact rendering only (RFC-0034) and the `dns.*` shapes are offline
manifest reads (RFC-0033). `config.get` resolves keys from `config.toml` and
compiled-in defaults; the env shapes contain no value or default value, only
presence, source and stable problem codes.

The mutating `ag deploy apply`, `ag dns apply` and `ag config set` deliberately
have no JSON shape and are refused with exit 2 before any executor runs.
`ag config validate` is read-only but has no published shape yet, so it is
refused too. `config.get`, `env.list` and `env.check` were published as
additive 0.2.0 fixtures and are now emitted: no new command and no change to an
existing shape, so there is no version bump (contract section 4, rule 3).

The following RFC-0024 section 9 prerequisites are **not** satisfied by this
contract alone and remain tracked in `docs/audits/mcp-readiness-review.md`:

- `--plan` / `--apply` and the shared audit sink (prerequisite 1, partial).
- `Secret<T>` in the CLI executors (prerequisite 2).
- Typed request/executor pairs for every command group (prerequisite 3); the
  read shapes advance this but do not replace the general request pipeline.
- The remaining design decisions (prerequisites 4-7).

No `contract_version` bump may be made without updating the JSON Schema, the
contract document, the `CONTRACT_VERSION` constant and the snapshots together.
Adding a new command shape without changing an existing one is additive and
stays inside the current pre-0.5 version (contract section 4, rules 2-4).
The product owner of this version is Anti-Gravital; Gravital Fabric publishes
its own independently versioned contract under `docs/mcp/` in its repository
(ADR-0016).
