# ADR-0017 - MCP is a capability-scoped machine interface

- Status: accepted
- Date: 2026-11-21
- Decider: BDFL (Angel Nereira)

## Context

Scripts, CI jobs and connected agents need to operate Anti-Gravital projects.
RFC-0021 already defined structured output, plan/apply and an audit trail, but
it did not define discovery, capability negotiation or a standard transport.
A connected agent must not inherit more authority than a person running the
same command, and debugging must not become an unrestricted read of process
memory.

## Decision

1. The Model Context Protocol is the machine interface. One versioned contract
   is shared by Anti-Gravital and Gravital Fabric (RFC-0024).
2. Every agent connection is granted an explicit capability manifest. The
   default capability is `read`. Mutating tools are unavailable unless their
   capability is granted.
3. A mutating tool returns a plan identifier first and requires an explicit
   apply call with that identifier. Confirmation is never skipped because the
   caller is automated.
4. The `debug` tool group is opt-in through `AG_MCP_DEBUG=1`, uses stdio only and
   applies the `Secret` redaction rules of RFC-0020.
5. Every call is appended to the local audit log. External MCP servers are
   feature-gated adapters with a native path preferred (ADR-0009).

## Consequences

- An agent can be limited to reading schemas in CI while another agent may
  apply deployments, with the difference expressed in one manifest.
- Tool results are versioned JSON data, so contract drift is detectable.
- There is no new listening port, no new authentication system and no new
  crate: the server is a module in `ag-cli` behind the `mcp` feature.
- A capability escalation is a security event recorded in the audit log.

## Alternatives

- **Trust every connected agent equally:** rejected, it violates least
  privilege and rule 16.
- **A separate private agent protocol:** rejected, it duplicates an open
  standard and would drift from the Fabric interface.
- **Expose all CLI flags as MCP tools automatically:** rejected, it would
  expose experimental and destructive flags without capability review.

## Status

Accepted. Implemented by RFC-0024; no code is required for this ADR to take
effect as a design constraint.
