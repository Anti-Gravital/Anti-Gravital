# ag-domains documentation

Documentation for the `ag-domains` control plane and the `ag-edge` data plane
(ADR-0012 / RFC-0011). Organized Diataxis-style.

This documents the implemented control plane and data plane: the control-plane
library, the data-plane library, the manual CLI flow (phase A), the live edge
listeners (HTTP-01 + routing + HTTPS/SNI, phase B), the REST API (phase C) and
the optional Postgres store (phase D). Provider automation and the registrar
module are later phases (see `docs/DEBT.md`, DEBT-025).

## Structure

- `tutorials/` — learning-oriented, end-to-end.
- `how-to/` — task-oriented guides (including provider guides and
  `how-to/serve-and-api.md` for running the edge and REST API).
- `reference/` — precise descriptions (CLI, state machine, DNS matrix).
- `explanation/` — background and rationale.

## Start here

- Tutorial: `tutorials/attach-first-domain.md`
- How-to:
  - `how-to/connect-providers.md` (per-provider DNS record locations)
  - `how-to/serve-and-api.md` (run the edge listeners + REST API)
  - `how-to/configure-wildcard.md`, `how-to/domain-connect.md`
  - `how-to/troubleshoot.md` (DNS + certificate issuance)
- Reference: `reference/cli.md`, `reference/state-machine.md`,
  `reference/dns-record-matrix.md`, `reference/events-and-metrics.md`,
  `reference/provider-capability-matrix.md`
- Explanation: `explanation/apex-vs-subdomain.md`,
  `explanation/why-txt-ownership.md`

Remaining work and technical debt are tracked in GitHub Issues (CLAUDE.md rule
29), not in repo files: see issue #76 (ag-domains remaining work / tech debt).

## Scope boundary

`ag-domains` attaches, verifies, diagnoses, secures and routes domains. It does
not purchase, transfer or renew them (a future `ag-registrars`), and it is not a
hosting panel (RFC-0011 §3.2).
