# RFC-0024 - Shared MCP machine interface with capability-scoped agents

- Status: accepted
- Decision date: 2026-11-21
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-21
- Target phase: additive Phase 4.8 stage F; later phases add tools
- Affected modules/crates: `ag-cli` (server, client and debug commands),
  `ag-core` (configuration primitives), `ag-observe` (debug resources)
- Predecessor RFC: RFC-0021 (machine interface), RFC-0018 (terminal interface)
- Comment period: waived by BDFL decision.

## 1. Motivation

RFC-0021 defined a machine-readable surface for the `ag` binary: structured
output, versioned JSON shapes, plan/apply for mutating commands, an audit trail
and no privilege by virtue of being a machine. That surface is consumed by
scripts and CI today. The next class of consumer is a connected model or agent
that needs to discover what Anti-Gravital can do, ask for a plan, approve it and
inspect the result without parsing prose.

The Model Context Protocol is an open protocol for exactly this exchange. This
RFC defines one shared MCP contract that both Anti-Gravital and the separate
Gravital Fabric platform implement. An agent connects once and can operate a
Anti-Gravital project, an infrastructure managed by Gravital Fabric, or both,
with the capabilities it was granted and no others.

A second requirement is self-debug: Anti-Gravital must be able to expose its
own state, diagnostics and traces to a connected agent so that the same
interface used to build a project can be used to investigate it.

## 2. Problem

1. Every operation is currently reached through prose CLI output or `--output
   json`. An agent has to know command names, argument order and the meaning of
   exit codes before it can do anything useful.
2. There is no discovery step: no way for a connected agent to enumerate the
   operations, their inputs and their risk level.
3. There is no capability boundary. A tool that can only read a schema and a
   tool that can apply a deployment are indistinguishable to the caller.
4. Debugging requires reading local logs directly. There is no supported way to
   ask the running tool for its state, its last diagnostics or a trace.
5. Without a shared contract, the infrastructure platform would invent a
   second agent protocol and users would need two integrations.

## 3. Alternatives considered

**A. Keep only `--output json` (RFC-0021 stage E).** Zero new surface. Rejected
as the end state: scripts work, but an agent still has no discovery,
capability negotiation or standard transport.

**B. MCP server only in Anti-Gravital.** Smaller first step. Rejected because
the infrastructure platform would need a different protocol, and the user
would have to configure two agents for one workflow.

**C. A long-running agent daemon with a private protocol.** Full control.
Rejected: it adds a listening port, its own authentication and a wire format,
all security critical, and it duplicates what the open protocol already does.

**D. One shared MCP contract, implemented by both products (chosen).** The
contract is versioned and published as data. Anti-Gravital and Gravital Fabric
implement it independently, with no compile-time dependency between them.

## 4. Proposed design

### 4.1 Principles (normative)

1. **Capability-scoped.** Every connected agent is granted an explicit set of
   capabilities. The default set is `read`. A tool is unavailable if the agent
   lacks its capability; it is not merely rejected after the fact.
2. **No privilege by being a machine.** The MCP surface exposes exactly the
   operations a person can run. A mutating operation requires the same
   plan identifier and explicit apply step defined in RFC-0021.
3. **One executor.** MCP tools call the same request types and executors as the
   CLI and the TUI. The three surfaces cannot drift.
4. **Native first.** External MCP servers are adapters behind Cargo features.
   Every capability offered through an external server has a native path or a
   documented reason why none exists (ADR-0009 rules 2 and 5).
5. **Redaction by construction.** Secrets are never placed in tool results,
   resources or error messages.

### 4.2 The shared contract

The contract is published as versioned JSON Schema documents under
`docs/mcp/` in each product, with identical tool names and shapes for the
operations both products share. Tool names use verbs and nouns, never product
names or third-party brands (ADR-0011).

| Group | Tools | Capability |
| --- | --- | --- |
| `project` | `project.list`, `project.describe`, `project.create` | `read`, `project.create` |
| `schema` | `schema.lint`, `schema.generate`, `schema.describe` | `read`, `schema.write` |
| `config` | `config.get`, `config.set` | `read`, `config.write` |
| `workers` | `workers.list`, `workers.enqueue` | `read`, `workers.enqueue` |
| `deploy` | `deploy.plan`, `deploy.apply`, `deploy.status` | `read`, `deploy.apply` |
| `debug` | `debug.state`, `debug.diagnostics`, `debug.trace` | `debug.read` |

A tool result is the versioned JSON shape already used by `--output json`,
plus a `contract_version` field. A mutating tool returns a plan identifier on
the first call and requires that identifier on the apply call.

### 4.3 Server

`ag mcp serve` starts a stdio MCP server. It is opt-in and is not started by
`ag dev` or any other command. Capabilities are read from a local manifest
that names the connected agent and its grants. The server records every call
in the same local audit log used by RFC-0021 mutating commands.

### 4.4 Client

`ag mcp client add` registers an external MCP server. The client is used for
provisioning adapters (databases, vector stores, mail systems, DNS, web
server runtimes) when a native implementation is not available. The adapter
runs behind a Cargo feature and must be declared in the project manifest.

### 4.5 Self-debug

`AG_MCP_DEBUG=1` enables the `debug` tool group on the local stdio server. It
exposes: the loaded configuration with secrets redacted, the last DSL
diagnostics, worker and queue state, and the most recent tracing spans. It is
never enabled by default and never listens on a network socket.

### 4.6 Diagram

See `docs/diagrams/mcp-layer-flow.md` for the logical flow between an agent,
the shared contract, the CLI executor and the debug sources.

## 5. Implementation plan

Only after this RFC is accepted:

1. Publish the versioned tool schemas and the capability manifest format.
2. Add `ag mcp serve` with stdio transport and read-only default capabilities.
3. Add the mutating tool groups with plan/apply and audit records.
4. Add `ag mcp client add` and one native provisioning adapter.
5. Add `debug` tools behind `AG_MCP_DEBUG=1`.
6. Update RFC-0021 stage E to reference this contract.

No new crate is introduced: the implementation is a module in `ag-cli`
behind an `mcp` Cargo feature, following the precedent of RFC-0018.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| An agent obtains a capability it should not have | Medium | High | Deny-by-default manifest, one capability per tool, audit log, tests that every mutating tool refuses a plan without apply. |
| A malicious tool result injects instructions into the agent | Medium | Medium | Results are data with versioned JSON shapes; never free-form prose; no tool output is executed as a command. |
| Secret leakage through debug resources | Low | High | Secret type from RFC-0020 never serializes; redaction tests on every debug tool. |
| External MCP server is untrusted | Medium | Medium | Feature-gated adapters, explicit registration, no ambient authority, native path preferred. |
| Contract drift between Anti-Gravital and Fabric | Medium | Medium | Versioned schemas, snapshot tests per tool, contract version in every result. |

## 7. Impact

- **Scope:** amends the scope statement of RFC-0022 section 4.1 to state that a
  capability-scoped machine interface for agents is part of the backend tool.
  It does not turn Anti-Gravital into an AI product.
- **Roadmap:** additive stage F of Phase 4.8.
- **Complexity:** one optional Cargo feature and three CLI commands.
- **APIs:** new versioned JSON schemas; no existing public Rust API changes.
- **Documentation:** this RFC, ADR-0016, ADR-0017, the MCP diagram, README
  command table and the module reference for `ag-cli`.

## 8. Rollback

The `mcp` Cargo feature is off by default in a rollback build and all three
commands disappear. The manifests and schemas remain as data. Rollback is
triggered by an audit finding, a capability escalation or contract drift that
cannot be fixed in a patch release.

## 9. Implementation prerequisites (review of 2026-11-21)

A readiness review found that this revision does not yet contain the substrate
the design assumes. The decision stands, but no MCP code is authorized until
these prerequisites are met:

1. `--output json` with versioned shapes, `--plan` and `--apply` (RFC-0021
   stage E) and an audit sink exist in `ag-cli`.
2. `Secret<T>` from RFC-0020 is available to the CLI executors.
3. The request/executor split covers every command group, not only
   `NewProjectRequest`. Today one of eight groups has a typed request.
4. `deploy.*` tools stay unimplemented until Phase 5 (`ag-cloud`) exists. They
   must not be used to expose the Fabric control plane through a second
   contract (ADR-0016).
5. The plan model is unified: a mutating tool either returns a plan identifier
   for a follow-up apply call, or the `deploy.plan` / `deploy.apply` pair is
   used consistently — not both in the same contract version.
6. `contract_version` has a named owner in each product; Fabric and
   Anti-Gravital publish compatible but independently versioned schemas.
7. `debug.diagnostics` must not perform DNS egress while the interface is
   documented as stdio-only; either resolve names before startup or document
   the egress explicitly.

The review (with the tool-by-tool mapping, capability manifest schema and
nineteen test cases) is recorded in `docs/audits/mcp-readiness-review.md`.

## 10. References

- RFC-0021 (machine interface, plan/apply, audit trail).
- RFC-0018 (one executor shared by CLI and TUI).
- RFC-0020 (`Secret` type).
- RFC-0022 section 4.1 (scope statement).
- ADR-0009 rule 2, ADR-0011, ADR-0008.
