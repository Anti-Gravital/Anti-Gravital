# MCP readiness review (RFC-0024 / ADR-0017)

> Scope: technical gap analysis to support a later implementation of the shared,
> capability-scoped machine interface for connected agents.
>
> Date: 2026-11-21. Source revision: `0c4d808` (branch snapshot used for this review).
> Reviewer role: connected-agent and interface analysis.
>
> **No code was implemented by this review.** No Cargo feature, CLI command,
> schema file, request type, executor, transport or dependency was added. This
> document is a design/contract review only. Its proposals require the RFC
> amendment described in section 10 before any of them become normative.
>
> **Implementation update (2026-11-21).** The read-only part of prerequisite 1
> has landed: `ag --output json` now emits a versioned shape (contract `0.1.0`)
> for `schema.lint`, `schema.diff` and `workers.list`, with byte-exact snapshots
> and a refusal for every other command. See section 12. The rest of
> prerequisite 1 (`--plan` / `--apply`, audit sink), prerequisite 2 (`Secret<T>`)
> and prerequisite 3 (typed requests for all groups) remain open. No MCP code is
> authorised yet.

## 1. Verdict

RFC-0024 and ADR-0017 define a coherent target, but the repository does **not**
currently contain the substrate the design assumes. The RFC is written as if
RFC-0021 stage E and RFC-0020 stage B already exist. They do not.

Concretely, on the reviewed revision:

- There is no `--output json`, no `--plan`, no `--apply`, no `ag describe`, no
  exit-code table and no audit log. Every command prints prose with `println!`
  and returns `Result<(), String>`.
- `crates/ag-cli/src/request.rs` contains exactly one request type
  (`NewProjectRequest`) and there is exactly one typed executor
  (`execute_new` in `commands/project.rs`). The other seven command groups
  dispatch raw clap values directly into `cmd_*` functions.
- The `Secret<T>` type of RFC-0020 does not exist. The sensitive values the MCP
  surface would handle are plain strings in clap arguments and environment
  reads (`AG_CLOUDFLARE_TOKEN`, `AG_SMTP_PASS`, `DATABASE_URL`).
- There is no `docs/mcp/` directory, no tool schema, no capability manifest and
  no MCP dependency in `Cargo.toml` or `Cargo.lock`.
- `ag deploy` does not exist and is explicitly out of scope until Phase 5; the
  command-tree test in `crates/ag-cli/src/main.rs` asserts its absence.

Therefore the "one executor" principle (RFC-0024 section 4.1.3) and the "tool
result is the versioned JSON shape already used by `--output json`" statement
(section 4.2) cannot be honoured today. Implementing the MCP server first would
create a second, divergent execution path, which is the exact failure mode the
RFC sets out to prevent. Stage F must be preceded by the stage E/B substrate
(section 8, P0).

## 2. Evidence: RFC assumptions versus repository state

| RFC-0024 assumption | Repository evidence on `0c4d808` | Status |
| --- | --- | --- |
| A versioned JSON shape exists per command (4.2) | Update 2026-11-21: contract `0.1.0` in `docs/mcp/` covers three read commands with snapshots; all other commands still have prose only | Partial (3 of the read tools) |
| Every mutating command has a plan id and explicit apply (4.1.2, 4.2) | No `--plan`/`--apply` in `cli.rs`; no plan store anywhere | Missing |
| The same local audit log is used by RFC-0021 mutating commands (4.3) | No audit record type or log writer in `ag-cli`, `ag-core` or `ag-observe` | Missing |
| MCP tools call the same request types and executors as CLI and TUI (4.1.3) | Only `ag new` has a request/executor pair; TUI itself only covers `ag new` (`tui/new_wizard/`) | Partially true (1 of 8 groups) |
| Secrets never serialize (4.1.5, RFC-0020) | No `Secret<T>`; tokens and passwords are `String` | Missing |
| Debug resources can reuse existing diagnostics/traces (4.5) | `ag-observe` builds a tracing subscriber but exposes no in-memory span buffer or query API | Missing |
| `deploy.*` tools can be bound to a CLI command | `main.rs` test asserts `!root_commands.contains(&"deploy")`; `docs/modules/ag-cli/README.md` lists `ag deploy` as not available | Contradiction |
| Phase 4.8 stage F follows stage E | `docs/roadmap/STATUS.md`: stages A-E all unchecked, "sin iniciar" | Blocked by predecessor |

## 3. Tool-by-tool mapping to the current CLI surface

### 3.1 Summary table

"Executor" below is the function that currently performs the work. `*` marks a
typed request/executor pair; all other rows take raw arguments directly.

| MCP tool (RFC-0024 4.2) | Capability | CLI counterpart | Current executor / library call | Mutating | Mapping state |
| --- | --- | --- | --- | --- | --- |
| `project.list` | `read` | none | no discovery API; would read workspace/filesystem | no | **New** |
| `project.describe` | `read` | none (`ag describe` is stage E, command-tree only) | TBD | no | **New + ambiguous** |
| `project.create` | `project.create` | `ag new <name> [--template ...]` | `resolve_new_request` -> `NewProjectRequest`* -> `execute_new`* -> `scaffold_*` | yes (files in cwd) | Direct, needs plan/apply |
| `schema.lint` | `read` | `ag schema lint [--schema ...]` | `cmd_schema_lint` -> `ag_dsl::lint` -> `Diagnostic::display` | no | Direct, needs JSON shape |
| `schema.generate` | `schema.write` | `ag generate [--schema ...] [--output ...]` | `cmd_generate` -> `ag_dsl::compile` + `ag_dsl::generate` + `write_file` | yes (writes files) | Direct, needs plan/apply |
| `schema.describe` | `read` | none (stage E `ag describe` covers commands, not the DSL) | TBD (`docs/dsl/` is prose) | no | **New + ambiguous** |
| `config.get` | `read` | none (`ag env list` is RFC-0020 stage B, not implemented) | TBD | no | **New + ambiguous** |
| `config.set` | `config.write` | none (`ag env set` is RFC-0020 stage B, not implemented) | TBD | yes | **New + secret risk** |
| `workers.list` | `read` | `ag workers list [--schema ...]` | `cmd_workers_list` -> `ag_dsl::compile`; no database | no | Direct |
| `workers.enqueue` | `workers.enqueue` | `ag workers enqueue KIND [--queue ...] [--payload ...] [--payload-version ...]` (feature `workers-runtime`) | `cmd_workers_runtime` -> `workers_backend` (reads `DATABASE_URL`) -> `PostgresQueue::enqueue` | yes (database) | Direct, needs plan/apply + path confinement |
| `deploy.plan` | `deploy.apply` (per table) | none | none; `ag deploy` is Phase 5 | yes (planning) | **No implementation, RFC needed** |
| `deploy.apply` | `deploy.apply` | none | none | yes | **No implementation, RFC needed** |
| `deploy.status` | `read` | none | none | no | **No implementation** |
| `debug.state` | `debug.read` | none | TBD: loaded config, attachment store, feature set | no | **New** |
| `debug.diagnostics` | `debug.read` | partial: `ag schema lint`, `ag domains diagnose`, `ag workers doctor` | `ag_dsl::lint`, `ag_domains::diagnostics::diagnose`, `cmd_workers_doctor` | no (but `diagnose` does DNS egress) | **New aggregation** |
| `debug.trace` | `debug.read` | none | `ag-observe` tracing subscriber only, no buffer/query | no | **New crate capability** |

### 3.2 Notes per group

- **project.** `project.create` is the only tool with a ready-made request and
  executor. Its current executor writes directly; a plan phase must be added or
  the tool must be explicitly exempted because it writes inside the working
  directory (RFC-0021 4.3.4 scopes plan/apply to changes *outside* the working
  directory). RFC-0024 4.1.2 does not state the exemption. This needs a decision
  (section 10, Q3).
- **schema.** `schema.lint` and `schema.generate` have clean executor mappings.
  `Diagnostic::display` produces prose; a versioned structured diagnostic (code,
  severity, span, message) is needed for the JSON result. `schema.generate`
  writes files and must be plan-gated. `schema.describe` has no counterpart.
- **config.** Neither `ag.toml` (RFC-0019 stage A) nor `ag env` (RFC-0020 stage
  B) is implemented. The RFC does not say whether `config.*` targets application
  settings in `ag.toml` or declared environment variables. A `config.set` whose
  value is a declared secret must be refused when the value arrives as a tool
  argument: RFC-0020 4.6 deliberately forbids secret values in CLI arguments
  because they are visible in process listings and shell history; MCP tool
  arguments are equally recorded in tool-call logs and audit records.
- **workers.** `workers.list` is purely local and safe. `workers.enqueue` reads
  `DATABASE_URL` from the process environment, which is itself a secret in
  RFC-0020 terms. The CLI takes a **file path** for the payload; an MCP tool
  must confine that path to the project root and must never echo payload bytes.
  Note the asymmetry: destructive `workers dlq retry/purge` commands are
  intentionally not exposed; document that as a deliberate capability decision.
- **deploy.** No CLI command exists. `docs/modules/ag-cli/README.md` says
  `ag deploy` arrives with its roadmap phase and requires an approved RFC.
  Including `deploy.*` in the stage F contract either lies to the connected
  agent or forces a Phase 5 feature into 4.8. It also removes the only
  end-to-end example of the plan/apply mechanism the RFC uses in section 4.2.
- **debug.** `ag-observe` has the tracing subscriber but no in-memory buffer;
  `debug.trace` therefore needs new code in a crate the RFC lists as affected.
  `debug.diagnostics` aggregating `ag domains diagnose` would introduce DNS
  network egress into a group documented as "stdio only" (section 6).

### 3.3 Commands deliberately not exposed

`ag domains sync/attach/detach/verify`, `ag mail test`, `ag dev`, `ag build`,
`ag schema diff` and `workers dlq` are absent from the tool table. Some are
mutating (`domains sync` uses `AG_CLOUDFLARE_TOKEN`, `mail test` uses
`AG_SMTP_PASS`, `domains attach` produces a takeover-sensitive ownership token).
The subset is defensible under least privilege, but the RFC should state that
this is an intentional stage F boundary and that each later addition needs a
capability review and an RFC, not an automatic flag-to-tool conversion
(ADR-0017 alternatives).

## 4. Capability manifest schema proposal

Location proposed: `docs/mcp/capability-manifest.schema.json`, version 1.0.0,
loaded by `ag mcp serve` from a local file with owner-only permissions. The
manifest is configuration data, not code, and contains no executable or prose
instruction fields (RFC-0024 risk table, prompt injection).

### 4.1 Example (read-only CI agent)

```json
{
  "manifest_version": "1.0.0",
  "contract_version": "1.0.0",
  "agent": { "id": "ci-schema-check" },
  "grants": ["read"],
  "scope": {
    "project_root": ".",
    "allow_write_outside_root": false
  },
  "issued_at": "2026-11-21T00:00:00Z",
  "expires_at": "2026-12-21T00:00:00Z"
}
```

### 4.2 JSON Schema sketch (draft 2020-12)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://anti-gravital.dev/mcp/capability-manifest-1.0.0.schema.json",
  "title": "AG MCP capability manifest",
  "type": "object",
  "additionalProperties": false,
  "required": ["manifest_version", "contract_version", "agent", "grants", "scope"],
  "properties": {
    "manifest_version": { "const": "1.0.0" },
    "contract_version": { "type": "string", "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$" },
    "agent": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id"],
      "properties": {
        "id": { "type": "string", "pattern": "^[A-Za-z0-9._-]{1,64}$" }
      }
    },
    "grants": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "enum": ["read", "project.create", "schema.write", "config.write",
                 "workers.enqueue", "deploy.apply", "debug.read"]
      }
    },
    "scope": {
      "type": "object",
      "additionalProperties": false,
      "required": ["project_root", "allow_write_outside_root"],
      "properties": {
        "project_root": { "type": "string", "minLength": 1 },
        "allow_write_outside_root": { "const": false }
      }
    },
    "issued_at": { "type": "string", "format": "date-time" },
    "expires_at": { "type": "string", "format": "date-time" }
  }
}
```

### 4.3 Loading rules (normative for the later implementation)

1. Missing or empty `grants` means exactly `["read"]` (deny by default).
2. An unknown grant string rejects the whole manifest; the server does not
   start and advertises no tools. Unknown strings are never ignored.
3. `grants` is subtractive only. The tool-to-capability map is a static,
   reviewed constant in code; a manifest can only withhold capabilities, never
   invent one.
4. `contract_version` must match the server's contract version exactly; a
   mismatch is a fail-closed startup error and an audit event.
5. `agent.id` is required because it is the plan owner and the audit subject.
   The bounded character set prevents log forging and injection. No display
   name, instructions or free-form description field is accepted in v1.
6. `scope.project_root` is canonicalised at startup. Every path argument of
   every tool is canonicalised and must remain under it; `..`, symlink escapes
   and absolute paths outside the root are refused.
7. An expired `expires_at` rejects the connection. Absence is allowed for local
   development but must be flagged in the audit record.
8. Capability revocation between plan and apply takes effect at apply time:
   apply re-checks the grant, not just plan time.

## 5. Security checks before each mutating tool

### 5.1 Uniform plan/apply pipeline

For every mutating tool the server applies this pipeline, in order. Steps 1 and
6 run for **all** calls, including read tools.

1. **Capability gate before argument parsing.** If the tool's required
   capability is not in the manifest grants, the tool is not listed in
   `tools/list` and a direct call returns a versioned `capability_denied` error.
   The handler re-checks even after advertisement (defence in depth). No
   executor, database connection or file write is reached.
2. **Input validation and path confinement.** Validate enums, bounds, sizes and
   canonical paths as in section 4.3.6. Reject shell metacharacters only where
   a shell is actually invoked (none should be).
3. **Plan phase.** Build the typed request (reusing the CLI request type).
   Compute `plan_id = SHA-256(contract_version | agent.id | tool | canonical
   request JSON | canonical project_root | state_digest)`, where `state_digest`
   covers the exact files or database state the tool will change. Persist a
   plan record `{plan_id, tool, request, state_digest, agent_id, created_at,
   expires_at, status: planned}` under `.ag/mcp/plans/` with 0600 permissions.
   Return only the plan descriptor and id. **No side effects.**
4. **Apply phase.** Require `plan_id`. Refuse if it is absent, unknown,
   expired, already consumed or owned by a different `agent.id` (`plan_required`,
   `plan_unknown`, `plan_expired`, `plan_consumed`, `plan_owner_mismatch`).
   Re-check the capability. Recompute `state_digest`; a mismatch returns
   `state_changed` and writes nothing. Atomically mark the plan consumed
   **before** invoking the executor so concurrent double-apply cannot both run.
5. **Redaction on the way out.** Serialize results and errors through the
   versioned schema; fields typed `Secret<T>` cannot serialize at all. Apply an
   explicit redaction pass to known sensitive values: environment values,
   provider tokens, SMTP credentials, `DATABASE_URL`, domain ownership tokens,
   payload bytes, private-key material. Errors from lower layers must be
   sanitised before they enter a result (database and HTTP client errors can
   include connection strings and headers). A canary test proves no secret
   value reaches any output (section 9).
6. **Audit record for every call.** Append one JSON line to the local audit log
   used by RFC-0021: timestamp (RFC 3339), `agent.id`, tool, capability checked,
   plan id (if any), decision (`allowed`/`denied`), refusal reason, contract
   version, executor outcome and a non-sensitive result summary. Denied calls
   are recorded: ADR-0017 declares capability escalation a security event. The
   log is append-only with 0600 permissions and never contains a secret value.

### 5.2 Per-tool requirements

| Tool | Capability | Plan state digest must cover | Redaction / audit note |
| --- | --- | --- | --- |
| `project.create` | `project.create` | absence of target directory, template, project name | Validate name with `validate_project_name`; audit the name, not file contents |
| `schema.generate` | `schema.write` | schema file bytes + output directory entry list | Diagnostics may quote schema text; redact canary secrets before returning |
| `config.set` | `config.write` | target config file bytes + declared secret-name set | Refuse a secret value passed as a tool argument; audit only the variable name and source, never the value |
| `workers.enqueue` | `workers.enqueue` | payload bytes hash + relevant queue depth | Never return or audit payload bytes; confine `--payload` path to project root; `DATABASE_URL` never in errors |
| `deploy.apply` | `deploy.apply` | deployment state digest defined by Phase 5 RFC | Provider credentials never in results or audit; blocked until Phase 5 |

`deploy.plan` follows the same plan-phase checks; `deploy.apply` is the only
tool that consumes that plan. Because RFC-0024 is internally inconsistent about
whether mutating tools are two-phase calls or two separate tools (section 10,
Q1), the final mapping must be fixed in the RFC amendment before coding.

## 6. Self-debug boundaries

The `debug` group is the highest-risk part of the surface. Required boundaries:

1. **Two independent switches.** A debug tool is available only when both
   `AG_MCP_DEBUG=1` is set on the local stdio server **and** the manifest grants
   `debug.read`. The environment variable enables the group; it does not grant
   the capability. A manifest that grants `debug.read` without the variable
   still sees no debug tools.
2. **Transport.** Local stdio only. No TCP listener, no socket path, no remote
   client, no `ag mcp client` forwarding of debug tools. `debug.*` is never
   advertised over an external MCP connection.
3. **Default off in discovery.** Without both switches the tools are absent
   from `tools/list`, not present-and-rejected. The audit record still logs any
   direct call attempt.
4. **`debug.state` allowlist.** May expose: contract version, active Cargo
   features, loaded schema path, module flags, worker mode/backend, attachment
   store summary (hostnames, lifecycle states), non-secret configuration keys.
   May not expose: raw environment values, secret-file contents, private keys,
   provider tokens, full payload bytes, arbitrary file reads. The tool accepts
   no path argument.
5. **`debug.diagnostics`**. Aggregate `ag_dsl` diagnostics plus `ag domains
   diagnose` findings plus `ag workers doctor`. Two constraints: (a) `diagnose`
   queries public resolvers, so this tool has network egress and must be
   documented as such or gated behind a separate explicit flag; (b) diagnostics
   text can quote source lines containing a secret, so the redaction pass is
   mandatory on every finding.
6. **`debug.trace`**. Requires a new bounded in-memory span buffer in
   `ag-observe` (ring buffer with a fixed span count and byte ceiling; default
   sink unchanged). Export by field allowlist, not denylist. No tool argument
   may change sampling or sinks. Because RFC-0020 makes `Secret` unformattable,
   traces cannot contain secret values by construction.
7. **No debug execution primitive.** There is no `debug.exec`, `debug.env` raw,
   memory dump or file-read tool. The group is observation only.

## 7. Boundary with Gravital Fabric (ADR-0016)

- ADR-0016 decision 2 forbids any dependency in either direction between
  `ag-*` and `fabric-*` crates. The MCP contract must therefore be **versioned
  data** (JSON Schema) under `docs/mcp/` in each product, with generated types
  consumed locally. A shared Rust crate holding tool request types is
  explicitly rejected by the ADR alternatives and must not be introduced.
- ADR-0016 decision 1 requires contracts to be published in a versioned,
  exportable format. `contract_version` in every tool result (RFC-0024 4.2) is
  the right mechanism, but the rules for bumping it and the owning product are
  not stated. See section 10, Q5.
- Tool names must remain product-neutral (RFC-0024 4.2, ADR-0011): no `ag.` or
  `fabric.` prefix on shared tools, no third-party brand in a tool name. A
  third-party name is allowed only as an explicit adapter label behind a Cargo
  feature, confined to that adapter module.
- External MCP servers registered by `ag mcp client add` are feature-gated
  adapters with a native path preferred (ADR-0009 rules 2 and 5). They carry no
  ambient authority and cannot grant a capability the local manifest withholds.
- **Contract risk to arbitrate:** Fabric natively owns deployment; Anti-Gravital
  has no `ag deploy` and none until Phase 5. If both products publish identical
  `deploy.*` tool shapes, one implementation advertises a capability it cannot
  honour, which is contract drift in the ADR-0016 sense. Recommended resolution:
  split the contract into a shared core (tools both products implement) and
  product extensions with distinct names, and defer `deploy.*` in Anti-Gravital
  to the Phase 5 RFC. This is posted to the architect role.

## 8. Prioritized implementation checklist

**P0 - blockers, no MCP code before these are resolved**

1. Implement RFC-0021 stage E in `ag-cli`: `--output json` with a versioned
   shape per command, stable exit codes and `ag describe`. Without it there is
   no shape to reuse and no self-description to negotiate. **Update
   2026-11-21:** the JSON shapes, snapshots and the 0/1/2 exit codes for three
   read commands are done (section 12); `ag describe`, the full exit-code table
   and shapes for the remaining commands are still missing.
2. Implement the plan/apply framework: a single plan store and a single
   append-only audit log in `ag-cli`, used by every mutating CLI command first.
   MCP must be a caller of this framework, never a second implementation.
3. Implement `Secret<T>` in `ag-core` (RFC-0020 stage B) and convert the
   sensitive values the CLI currently holds as `String`. Redaction "by
   construction" (RFC-0024 4.1.5) is impossible without it.
4. Generalise `request.rs`: one typed request and one `execute_*(request)`
   function per command group, and refactor `cmd_*` into thin front-ends. This
   makes RFC-0024 principle 3 true for all groups, not just `ag new`.
5. Resolve the RFC contradictions and naming gaps in section 10 through an RFC
   amendment (especially `deploy.*`, the two plan models and the new tools).

**P1 - contract as data**

6. Publish `docs/mcp/` JSON Schemas for tool inputs, tool outputs, error codes
   and the capability manifest; add per-tool snapshot tests and a CI contract
   check against the Fabric-published version.
7. Implement the manifest loader with the fail-closed rules of section 4.3.

**P2 - server and tools**

8. Add `ag mcp serve` (stdio, opt-in, `mcp` Cargo feature off by default) and
   the audit hook; default grants are `["read"]`.
9. Wire the read tools that have executors: `project.list`, `project.describe`
   (once defined), `schema.lint`, `schema.describe` (once defined),
   `workers.list`, `deploy.status` (once Phase 5 exists).
10. Wire mutating tools through the full section 5 pipeline: `project.create`,
    `schema.generate`, `config.set`, `workers.enqueue`, `deploy.apply`.

**P3 - debug and client adapters**

11. Add the bounded span buffer in `ag-observe` and the three debug tools behind
    both switches from section 6.
12. Add `ag mcp client add` and one native provisioning adapter behind a Cargo
    feature (RFC-0024 4.4), with ADR-0011-compliant naming.

**P4 - verification and documentation**

13. Run the test suite in section 9, including parity against the CLI executor
    and the canary redaction tests.
14. Update the README command table, `docs/modules/ag-cli/README.md`, the RFC
    index cross-reference in RFC-0021 stage E, and the CHANGELOG in the same
    change that adds behaviour (CLAUDE.md rules 19, 26).

## 9. Test cases

The following are required before the surface can be considered safe. Tests
that need a database, a DNS resolver or an external provider should be
`#[ignore]` with documented manual verification, per the repository issue
lifecycle rules; everything else must run in CI.

1. **`agent_without_capability_is_refused`** - manifest grants `["read"]`;
   `workers.enqueue` is absent from `tools/list`, a direct call returns
   `capability_denied`, the audit log contains a denial record, and no
   `DATABASE_URL` connection is attempted. This is the explicit
   "agent without capability is refused" test.
2. **`unknown_capability_in_manifest_fails_closed`** - a grant string not in
   the closed enum makes the server refuse to start with no tools advertised.
3. **`mutating_tool_without_plan_is_refused`** - for each of `project.create`,
   `schema.generate`, `config.set`, `workers.enqueue`, a call without a plan id
   returns `plan_required` and leaves no file or database change.
4. **`plan_is_single_use`** - applying the same plan id twice succeeds once and
   returns `plan_consumed` the second time, including under concurrent calls.
5. **`plan_state_changed`** - modify the schema file after planning, then
   apply: `state_changed`, no generated files written.
6. **`plan_owner_mismatch`** - plan created by agent A, applied by agent B on
   the same server: refused and audited.
7. **`revoked_capability_between_plan_and_apply`** - plan with
   `workers.enqueue`, remove the grant, apply: refused.
8. **`contract_version_mismatch`** - manifest version differs from server:
   fail-closed startup plus audit event.
9. **`secret_never_in_output`** - seed a canary value as a declared secret;
   assert it appears in no tool result, no error message and no debug resource.
10. **`secret_never_in_audit`** - same canary across a mutating call; scan the
    audit log and plan records for it.
11. **`config_set_refuses_secret_argument`** - setting a declared secret via a
    tool argument is refused with a message pointing to the native secret path.
12. **`path_traversal_refused`** - `../../etc/passwd` and absolute paths
    outside `project_root` are refused for every path-taking tool.
13. **`debug_group_off_by_default`** - without `AG_MCP_DEBUG=1` there are no
    `debug.*` tools even when the manifest grants `debug.read`.
14. **`debug_capability_required`** - with the environment variable set but no
    `debug.read` grant there are no debug tools.
15. **`debug_trace_bounded_and_redacted`** - emit more spans than the ring
    buffer holds and assert the ceiling, plus the canary assertion.
16. **`no_listening_socket`** - `ag mcp serve` opens no TCP or Unix listener
    (static check plus runtime assertion).
17. **`contract_snapshot`** - each tool input/output schema and the manifest
    schema match the checked-in `docs/mcp/` snapshots; the same version is
    compared with the Fabric-published contract.
18. **`executor_parity`** - for each tool, perform the equivalent CLI request
    and assert identical state change, proving both surfaces share one
    executor.
19. **`audit_records_all_calls`** - successful, denied and failed calls each
    produce exactly one well-formed audit line.

## 10. Open questions and contract risks

These need an RFC amendment before implementation. They are reported to the
architect and interface roles through the shared community board.

- **Q1 (RFC internal contradiction).** Section 4.2 says a mutating tool returns
  a plan id on the first call and requires it on the apply call, but the tool
  table lists `deploy.plan` and `deploy.apply` as two separate tools. Choose one
  model for all mutating tools and rewrite the RFC accordingly.
- **Q2 (`deploy.*`).** No CLI command or backend exists and Phase 5 requires its
  own RFC. Decide whether stage F ships without deploy tools (recommended) or
  the RFC amends the Phase 4.8 scope. ADR-0016/Fabric overlap applies.
- **Q3 (plan scope).** RFC-0021 scopes plan/apply to changes outside the working
  directory; `project.create` and `schema.generate` write inside it. Decide
  whether these are exempt or plan-gated anyway.
- **Q4 (undefined tools).** `project.list`, `project.describe`,
  `schema.describe`, `config.get`, `config.set` and all `debug.*` tools have no
  CLI counterpart. Define their inputs, outputs and state sources. Clarify
  whether `config.*` targets `ag.toml` or declared environment variables.
- **Q5 (contract ownership).** ADR-0016 requires versioned data owned by a
  product. Which repository owns `contract_version` for the shared MCP schema,
  and what is the bump policy? Snapshot tests need a single authority.
- **Q6 (debug egress).** `debug.diagnostics` as specified would perform DNS
  queries through `ag domains diagnose`. Reconcile that with "stdio only" in
  RFC-0024 4.5 and ADR-0017 decision 4, or gate the network part separately.
- **Q7 (audit subject).** RFC-0021 records a "user"; MCP has an `agent.id` from
  a local manifest. Confirm that the same audit record field carries both and
  that no OS user impersonation is implied.

## 11. Traceability

Reviewed sources: `docs/rfc/RFC-0024-mcp-shared-machine-interface.md`,
`docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
(stage E), `docs/rfc/RFC-0020-environment-and-secrets.md` (`Secret` type),
`docs/rfc/RFC-0022` section 4.1, `docs/adr/0016`, `docs/adr/0017`,
`docs/diagrams/mcp-layer-flow.md`, `docs/roadmap/STATUS.md` (Phase 4.8),
`docs/modules/ag-cli/README.md`, and
`crates/ag-cli/src/{main.rs,cli.rs,request.rs,commands/*.rs}` plus
`crates/ag-observe/src/`.

Governing rules: CLAUDE.md sections 4, 16, 19, 22, 26, 28, 33 and the ADR-0008
language policy (this document is in canonical technical English). This review
proposes no scope change and implements no feature; all proposals remain
subject to the RFC amendment in section 10.

## 12. Implementation update: read-only JSON substrate (2026-11-21)

This section records what a later change actually implemented. It does not
change the verdict of section 1: the MCP surface itself is still blocked.

### 12.1 Now satisfied

- **Prerequisite 1, read-only portion.** `ag --output json` emits a versioned
  document with `contract_version = "0.1.0"`, a stable `command` name, a
  `status` discriminator and either `data` or a typed `error`. The three covered
  commands are `schema.lint`, `schema.diff` and `workers.list`. Their shapes,
  JSON Schema and byte-exact snapshots are published in `docs/mcp/` and
  asserted by `crates/ag-cli/tests/json_output.rs` (12 integration tests).
- **Machine-safe refusal.** Every other command, including all mutating ones,
  refuses `--output json` before dispatch with exit code 2 and a closed
  `unsupported_command` message. The tests assert that a refused `ag new` and a
  refused `ag generate` create nothing, and that a canary secret passed as an
  argument or environment value never appears in stdout or stderr (RFC-0020,
  RFC-0024 section 4.1.5).
- **Exit codes (partial).** JSON mode uses 0 for success, 1 for read or
  validation failure and 2 for usage/unsupported. `schema.lint` emits its
  diagnostics as data and exits 1 when errors exist. The remaining categories
  in RFC-0021 (precondition, remote failure) are undefined until their commands
  exist.
- **Prose parity.** With the flag absent, output is unchanged. A test pins the
  exact clean-lint prose line.

### 12.2 Still open (unchanged)

- Prerequisite 1, remaining: `--plan` / `--apply`, the single plan store and the
  shared append-only audit sink. No mutating command has a plan identifier.
- Prerequisite 2: `Secret<T>` from RFC-0020 in the CLI executors. The refusal
  path never echoes secrets, but sensitive arguments are still plain strings.
- Prerequisite 3: typed request/executor pairs for all command groups. This
  change adds read-only report builders, not the general request pipeline.
- Prerequisites 4-7 and the open questions Q1-Q7 in section 10 are untouched.

### 12.3 Deliberate scope boundary

No MCP server, transport, capability manifest, tool registry, `ag describe`,
plan/apply or agent protocol code was added. `schema.describe`, `config.get`,
`project.list` and `project.describe` have no CLI counterpart and therefore no
shape in `0.1.0`; inventing one here would violate the "no feature outside the
documentation" rule. The next step is a separate RFC-0021 stage E change (and
the RFC-0024 amendment for Q1-Q7), not an MCP implementation.

### 12.4 Contract ownership

Anti-Gravital owns `0.1.0`; the owner field is the `CONTRACT_VERSION` constant
in `crates/ag-cli/src/output.rs`, mirrored by `docs/mcp/cli-json-0.1.0.md`,
`docs/mcp/cli-json-0.1.0.schema.json` and `docs/mcp/snapshots/`. Fabric
publishes its own independently versioned contract (ADR-0016). This answers the
mechanism half of Q5; the shared bump policy still needs the RFC amendment.
