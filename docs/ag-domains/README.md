# ag-domains documentation

Documentation for the `ag-domains` control plane and the `ag-edge` data plane
(ADR-0010 / RFC-0009). Organized Diataxis-style.

This documents phase A (RFC-0009): the control-plane library, the data-plane
library and the manual CLI flow. Live edge listeners, the REST API, the SQL
store and provider automation are later phases (see `docs/DEBT.md`, DEBT-018).

## Structure

- `tutorials/` — learning-oriented, end-to-end.
- `how-to/` — task-oriented guides (including provider guides).
- `reference/` — precise descriptions (CLI, state machine, DNS matrix).
- `explanation/` — background and rationale.

## Start here

- Tutorial: `tutorials/attach-first-domain.md`
- Reference: `reference/cli.md`, `reference/state-machine.md`,
  `reference/dns-record-matrix.md`
- Explanation: `explanation/apex-vs-subdomain.md`,
  `explanation/why-txt-ownership.md`

## Scope boundary

`ag-domains` attaches, verifies, diagnoses, secures and routes domains. It does
not purchase, transfer or renew them (a future `ag-registrars`), and it is not a
hosting panel (RFC-0009 §3.2).
