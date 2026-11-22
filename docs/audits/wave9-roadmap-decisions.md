# Wave 9 roadmap decisions (architect)

> Scope: decide which roadmap boxes can genuinely advance in wave 9, based on
> what other roles actually deliver. This document does **not** check a box.
> `docs/roadmap/STATUS.md` stays unchanged except for a planning commentary
> note. Masters and code are untouched.
>
> Base snapshot: commit `7cbc28a` (HEAD of every wave-9 worktree at the time of
> writing). At that snapshot `git log 7cbc28a..community9/<role>` was empty for
> all eight peer branches and no peer worktree had staged or unstaged changes.
> Peer notes read: `clidev`, `coredev`, `mcpdev`, `qafuzz`, `trackers`,
> `dnsacme`. Their deliverables are **planned**, not merged, so none of them is
> accepted as evidence here. While this document was written, `trackers`
> committed `7560b71` on its own branch and opened issue **#198** (gate rows
> cite resolved advisories; the real current blocker is RUSTSEC-2026-0592).
> That branch is also unmerged, so #198 is recorded as a finding and its gate
> correction is prepared as edit E8, not applied here. The board moved from 14
> to 15 open issues during the wave.

## Decision legend

- **ADVANCE-EDIT**: the box can move to `[x]` only after the named role's branch
  is merged and the listed evidence commands pass on the merged tree. Exact
  edit supplied below; do not apply before merge.
- **KEEP-PARTIAL**: the wave-9 role advances one bullet, but other bullets of
  the same box remain open. The box stays `[/]`.
- **BLOCKED**: an external gate, a live service, human testing, or an undecided
  BDFL question prevents advancement regardless of wave-9 output.
- **NO-CHANGE**: box already `[x]`, or no wave-9 role owns it. Recorded for
  completeness; no action.

Evidence rule: a role's note or unmerged commit is not evidence. The
coordinator checks each box only after merging the branch and running the
commands in "Evidence commands for the coordinator".

---

## A. Phase 4.7 — Interactive terminal experience

### D1 — Entry criteria 4.7.1

- Current state: `STATUS.md` marks both boxes `[x]`; the verbatim phase doc and
  the master still show `[ ]`.
- Evidence required: `RFC-0018` accepted (section 9, 2026-11-10) and the CLI
  module split verified by the command-tree test. Both already exist.
- Wave-9 role: none needed.
- Decision: **NO-CHANGE**. The divergence is a documentation-sync question for
  the coordinator, not a box to advance (see "Requests to the coordinator").

### D2 — Stage A (`tui` feature, wizard, guard, flags, theme, degraded mode)

- Current state: `[x]` in `STATUS.md`, with 52 unit + 6 regression tests and
  manual pseudo-terminal verification recorded.
- Evidence required: already recorded in wave 4.
- Wave-9 role: none.
- Decision: **NO-CHANGE**.

### D3 — Stage B (`ag config` and `ag new --with MODULE`)

- Current state: `[/]` in `STATUS.md`. `ag config get|set|validate` and the
  full-screen form are merged (wave 4). `ag new --with MODULE` is absent from
  the merged tree (`grep -rn -- '--with' crates/ag-cli/src crates/ag-cli/tests`
  returns nothing). The config form registry holds 21 non-secret leaf keys;
  the bullet "secret fields accept only a file path or an environment variable
  name" is not evidenced.
- Evidence required to check the box: (a) `ag new --with MODULE` present in the
  merged tree, (b) flag-parity test proving the wizard request equals the
  parsed non-interactive request, (c) the matching wizard step of RFC-0018
  section 4.4, (d) secret fields restricted to a path or env-var name, (e) the
  four documents named in the cross-stage bullet updated in the same PR,
  (f) `cargo test -p ag-cli`, `cargo clippy --workspace --all-targets --all-features
  -- -D warnings`, `git diff --check`.
- Wave-9 role: `clidev` owns (a) and (b) and the non-interactive tests;
  `coredev` provides the `Secret<T>` substrate but not the CLI restriction.
  No wave-9 role owns (c) or (d).
- Decision: **KEEP-PARTIAL**. Even after `clidev` merges, bullets (c) and (d)
  remain open. Apply edit E1 only as an annotation, never as a box check.

### D4 — Stage C (bare `ag` home screen)

- Current state: `[x]` in `STATUS.md`; interactive verification is tracked as
  an exit criterion (D6), not as an implementation box.
- Evidence required: already recorded for implementation.
- Wave-9 role: none.
- Decision: **NO-CHANGE**.

### D5 — Cross-stage bullets (state/render/parity/regression tests, diagram, docs)

- Current state: the phase doc and master show these unchecked. The merged tree
  does contain render assertions at 80x24 and 120x40 (`tui/new_wizard/tests.rs`,
  `tui/home.rs`, `tui/config_form.rs`), `docs/diagrams/cli-interactive-flow.md`,
  `crates/ag-cli/tests/wave4_regression.rs` and updated CLI docs. `STATUS.md`
  does not expose these as individual boxes.
- Evidence required: re-run the render/parity/regression suites on the merged
  tree, including after `clidev` adds `--with`.
- Wave-9 role: `clidev` keeps parity and non-interactive tests green for its new
  flag; no role owns a master/phase-doc update.
- Decision: **KEEP-PARTIAL** until the merged-tree run exists. No box change.

### D6 — Exit criteria 4.7.3

- Current state: parity suite `[x]`; three-person test `[ ]`; four-platform
  regression `[/]` (only Linux x86-64 green, evidence predates the wave-4
  merge); SSH/tmux/Windows `[ ]`; binary-size measurement `[ ]`.
- Evidence required: three published external session notes; green
  `wave4_regression.rs` on Linux x86-64, Linux ARM64, macOS ARM64 and Windows
  x64 after merge; SSH/tmux/Windows console session notes; a published release
  binary size delta.
- Wave-9 role: none (no wave-9 role owns CI platforms or human testing).
- Decision: **BLOCKED** for the human/platform rows; **NO-CHANGE** for parity.
  Do not mark four-platform green on a single-platform run.

---

## B. Phase 4.8 — Project foundations

### D7 — Entry criteria 4.8.1

- Current state: phase doc/master `[ ]`. `RFC-0019` and `RFC-0020` have their
  open questions resolved; `RFC-0021` section 9 leaves two questions undecided:
  scope of `ag setup` and release-key custody. Phase 4.7 stage A is complete.
- Evidence required: a BDFL decision recorded in `RFC-0021` section 9 for both
  questions.
- Wave-9 role: none (decision, not implementation).
- Decision: **BLOCKED**. Exact requested decision text in the coordinator notes.

### D8 — Stage A (project structure)

- Current state: `[/]`. The opt-in `layered` template and `ag.toml` manifest
  exist. `grep -rn 'ag.toml\|AgManifest\|manifest' crates/ag-cli/src/commands/project.rs`
  returns nothing: there is no reader in the generation path. No
  `ag generate --check` flag (`grep -rn -- '--check' crates/ag-cli/src/cli.rs`
  returns nothing). The layout is not forced for `fullstack`/`realtime`.
- Evidence required: `ag.toml` read by every command; layered layout enforced
  by Cargo for `fullstack` and `realtime`; compact layout for `rest`;
  `ag generate --check` green in CI; a failing compile when a domain crate
  imports a database or HTTP crate.
- Wave-9 role: `clidev` records `[modules]` from `ag new --with MODULE` but does
  not add the reader, `--check`, or the forced layouts.
- Decision: **KEEP-PARTIAL**. Edit E2 annotates the `--with` advance only.

### D9 — Stage B (environment and secrets)

- Current state: `[/]`. Wave 4 delivered the declared loader and `Secret<T>`
  (14 unit + 2 doctests). No `ag env` command exists in the merged CLI
  (`grep -rn 'Env' crates/ag-cli/src/cli.rs` finds only unrelated `environment`
  doc strings). No encrypted per-environment secrets file. `coredev` states
  explicitly that all 82 `std::env::var` call sites stay and `[env]` parsing
  from `ag.toml` is not in its branch.
- Evidence required: encrypted per-environment file with recipient management;
  `ag env list/check/set/unset/recipients/keygen/export`; one crate per PR
  migrated off direct process-env reads; loader called from the composition
  root before the listener opens; three invalid variables reported at once
  with no listener opened; no secret value in logs or output.
- Wave-9 role: `clidev` (`ag env list|check`, read-only, no values), `coredev`
  (secret-copy fix and opt-in `ShieldConfig::from_path_with_env`).
- Decision: **KEEP-PARTIAL**. Both branches are useful hardening but neither
  delivers the encrypted file, the full command set, or the migration. The box
  stays `[/]`. Apply E3 only as an annotation after merge.

### D10 — Stage C (installation)

- Current state: `[ ]`. Alpha installer keeps fail-closed verification without
  a signing key; not equivalent to signed release binaries.
- Evidence required: release workflow publishing four static binaries,
  checksum manifest and detached signature; installer verifying both without a
  compiler; `ag setup --check` and `ag setup`; clean-server run per supported
  distribution; tampered binary/manifest rejected.
- Wave-9 role: none. Blocked additionally by issue #141 (no out-of-band trust
  anchor) and by the two undecided `RFC-0021` questions (D7).
- Decision: **BLOCKED**.

### D11 — Stage D (remote console)

- Current state: `[ ]`, not started. No wave-9 role owns it.
- Evidence required: `ag servers add/list/remove` and `ag connect NAME` /
  `ag connect NAME -- COMMAND` over SSH; second-machine and phone-SSH session
  evidence.
- Wave-9 role: none.
- Decision: **NO-CHANGE** (out of wave-9 scope).

### D12 — Stage E (machine interface)

- Current state: `[/]`. `--output json` exists for `schema.lint`, `schema.diff`,
  `workers.list`, plus wave-8 shapes (`deploy.plan`, `deploy.status`, `dns.plan`,
  `dns.status`); `CONTRACT_VERSION` is `0.2.0` in `crates/ag-cli/src/output.rs`,
  while the `STATUS.md` annotation still says `0.1.0` (historically correct for
  wave 4). `ag describe` is absent; a general `--plan`/`--apply` pair exists
  only in `dns` and `deploy`; no documented exit-code table; no universal
  audit log.
- Evidence required: JSON shape on every command with versioned schemas;
  exit-code table; `ag describe`; `--plan`/`--apply` on every mutating command;
  an automated create-configure-validate cycle using only `ag describe` and
  JSON output, with no state change without a plan identifier.
- Wave-9 role: `mcpdev` defines, documents and tests the `config.get` shape
  (no wiring: `output.rs` is not the dispatch site); `clidev` owns the JSON
  dispatch in `main.rs`/`cli.rs` and can wire `config.get`, `env.list` and
  `env.check`. No wave-9 role owns `ag describe`, the exit-code table, or
  universal plan/apply.
- Decision: **KEEP-PARTIAL**. Edit E4 records the contract-version drift for
  the coordinator; do not check the box.

---

## C. Pre-Phase 5 release gate

### D13 — Audit row (`fail`, #173; stale evidence, #198)

- Current state: the row still cites the 2026-11-10 failures and closed
  issues #173/#174. Issue #198 records that those advisories are resolved in
  the lockfile and that the real current blocker is RUSTSEC-2026-0592
  (`url` 2.5.8, published 2026-11-20). That finding is on the unmerged
  `trackers` branch, so the gate text is not edited here.
- Evidence required: `cargo audit` exit 0 on the final commit, including
  RUSTSEC-2026-0592 (`url` >= 2.5.9), with command, date and commit in the row;
  issue #198 closed with that evidence.
- Wave-9 role: `trackers` documented the inconsistency; no wave-9 role owns the
  lockfile fix.
- Decision: **NO-CHANGE**; row stays `fail`. Edit E8 corrects the stale
  evidence text after the coordinator reproduces it.

### D14 — Deny row (`fail`, #173, #174, #185, #198)

- Current state: same staleness as D13. The `deny` failure is now reported on
  RUSTSEC-2026-0592, not only on the advisories named in the row.
- Evidence required: `cargo deny check` exit 0 with no dated ignores for
  RUSTSEC-2026-0387, RUSTSEC-2026-0440 and RUSTSEC-2026-0543 (chrono), and with
  RUSTSEC-2026-0592 resolved; row cites command, date and commit.
- Wave-9 role: none (`#185` has no wave-9 owner; `chrono` is not a leaf).
- Decision: **NO-CHANGE**; row stays `fail`. Edit E8 applies to both rows.

### D15 — Fuzz 24h row (`pending`, #156)

- Current state: only 30 s and 60 s smokes exist. `qafuzz` declares a window
  script and a short real window whose evidence explicitly refuses to label
  itself as the 24h gate.
- Evidence required: for all seven targets,
  `cargo +nightly fuzz run <target> -- -max_total_time=86400` on Linux x86-64,
  logs and corpus/crash findings archived under `docs/fuzz/results/`, zero
  unresolved crashes, date and commit recorded, `docs/DEBT.md` DEBT-015 pointing
  to #156, issue #156 closed with the evidence.
- Wave-9 role: `qafuzz` can provide automation and a partial window only.
- Decision: **KEEP-PARTIAL** (row stays `pending`). Edit E5 is ready for the
  day a real 24h run is merged.

### D16 — Benchmarks row (`pending`, #157)

- Current state: `docs/benchmarks/pre-fase5-performance.md` does not exist.
  Only a reduced-sample smoke and the empty verification package exist.
- Evidence required: final report with all rule-17 metadata (hardware, OS,
  Rust version, commit, configuration, methodology, run count, standard
  deviation), gate row citing it with command, date and commit, README claim
  reconciled, issue #157 closed.
- Wave-9 role: `qabench` can provide reproducible reduced-sample evidence; only
  a full stabilized-code run closes the gate.
- Decision: **BLOCKED** until the full report exists; row stays `pending`.
  Edit E6 is ready.

### D17 — Open debt row (`pending`)

- Current state: five open children of #142 remain (#141, #152, #154, #156,
  #157), plus issues outside #142 (#76, #87, #112, #178, #179, #180, #185,
  #189) and #198, opened during the wave by `trackers`.
- Evidence required: every `tech-debt` issue closed with verification; #142
  parent closed.
- Wave-9 role: `trackers` can reconcile labels, cross-links and the board audit,
  but must not close unverified issues.
- Decision: **KEEP-PARTIAL**; row stays `pending`.

---

## D. RFC status review

| RFC | Status | Wave-9 effect | Decision |
| --- | --- | --- | --- |
| RFC-0018 | accepted 2026-11-10 | `clidev` adds `--with` (D3) | NO-CHANGE |
| RFC-0019 | accepted 2026-11-10; stage A partial | `clidev` records `[modules]` (D8) | KEEP-PARTIAL |
| RFC-0020 | accepted 2026-11-10; stage B partial | `coredev` + `clidev` (D9) | KEEP-PARTIAL |
| RFC-0021 | accepted 2026-11-10; **two open questions undecided** | none | BLOCKED (BDFL decision) |
| RFC-0024 | accepted 2026-11-21; **7 prerequisites not met** | `mcpdev` + `clidev` advance prerequisite 3 only | BLOCKED (no MCP code) |

RFC-0024 prerequisite status against the merged tree: (1) partial - the
`--output json` flag exists but `--plan`/`--apply` are not universal and there is no audit
sink; (2) met - `Secret<T>` exists in `ag-core`; (3) partial - one of eight
command groups has a typed request (`clidev` adds two command surfaces but not
the full split); (4) unmet by design - `deploy.*` waits for Phase 5; (5)
partial - dns/deploy use plan identifiers, no unified model; (6) unmet -
`contract_version` has no named Fabric owner; (7) unmet - `debug.diagnostics`
DNS egress is not resolved or documented. The protocol decision "MCP code waits
for RFC-0024's seven prerequisites" therefore stands.

---

## E. Open issue disposition (14 baseline issues + #198 opened during the wave)

| Issue | Current state | Evidence needed to close | Wave-9 role | Decision |
| --- | --- | --- | --- | --- |
| #198 | open, p1, opened by `trackers` mid-wave | `cargo audit` and `cargo deny check advisories` exit 0 with RUSTSEC-2026-0592 resolved; gate Audit/Deny rows updated with current evidence | `trackers` documented it; no wave-9 role fixes the lockfile | NO-CHANGE |
| #189 | open; allowlist has 8 wave-9-relevant entries; `cli.rs` 810, `config.rs` 760, `env.rs` 776 lines | three named files at <=600 lines or split, allowlist shrunk, `bash tools/check-file-size.sh` + all-features tests green | `clidev` (cli.rs, config.rs, main.rs), `coredev` (env.rs), `refactor` (deploy.rs, dns.rs) | KEEP-PARTIAL; no single role covers all files |
| #185 | open, blocked, p2 | `cargo tree -i chrono` empty, `cargo deny check advisories` clean without the RUSTSEC-2026-0543 ignore, formats unchanged | none | BLOCKED |
| #180 | open, p3 | RFC amending RFC-0003 recorded; DSL tests and ecommerce regen unchanged | none | NO-CHANGE (needs RFC) |
| #179 | open, blocked, p3 | `cargo test -p ag-domains --all-features` on 0.8 + live staging certificate | `dnsacme` can document a local mock only; code is owned by nobody in wave 9 | BLOCKED (depends on #87) |
| #178 | open, blocked, p2 | new stack tests green + DKIM known-answer verified by independent verifier | none | BLOCKED |
| #157 | open, blocked, p2 | D16 evidence | `qabench` (partial) | BLOCKED |
| #156 | open, blocked, p2 | D15 evidence | `qafuzz` (automation + short window) | KEEP-PARTIAL |
| #154 | open, p3, Phase 5+ | generated typed ORM/RLS/multitenancy tested on live PG | none | NO-CHANGE |
| #152 | open, p2 | REST routes + marketing objects behind `api`, local PG tests green | none | NO-CHANGE |
| #142 | open, tracking | all children closed | `trackers` (board only) | KEEP-PARTIAL |
| #141 | open, blocked, p3 | out-of-band checksum/signature artifact + clean-machine end-to-end check | none | BLOCKED |
| #112 | open, p3 | concrete `ag-edge` consumer + `ag-edge -> ag-workers` dependency | none | NO-CHANGE (speculative by design) |
| #87 | open, blocked, p3 | `#[ignore]` staging test passing with credentialed domain | `dnsacme` (local mock harness only) | BLOCKED (live service) |
| #76 | open, tracking | #87 closed | `trackers` (board only) | KEEP-PARTIAL |

No issue is closed by this change. `trackers` must keep `Refs #NNN` unless a
branch actually carries full verification.

---

## Exact edits to apply after merged evidence

Do not apply any of these before the coordinator merges the corresponding
branch and runs the commands in the next section.

### E1 — `STATUS.md`, Phase 4.7 Stage B annotation (not the box marker)

Append to the existing Stage B line, after "no existe en el arbol mergeado":

```text
Tras la ola 9, `ag new --with MODULE` queda registrado en la rama community9/clidev; el box sigue parcial hasta que existan el paso de wizard de RFC-0018 seccion 4.4 y la restriccion de secretos a ruta de archivo o variable de entorno.
```

### E2 — `STATUS.md`, Phase 4.8 Stage A annotation

Append after "faltan el reader de `ag.toml` y `ag generate --check`":

```text
La ola 9 anadio el registro de `[modules]` desde `ag new --with MODULE` (rama community9/clidev); el reader, `ag generate --check` y el layout forzado siguen pendientes.
```

### E3 — `STATUS.md`, Phase 4.8 Stage B annotation

Append after "la llamada al loader desde el composition root antes de abrir el listener":

```text
La ola 9 endurecio `Secret<T>` y anadio `ShieldConfig::from_path_with_env` (community9/coredev) y `ag env list|check` de solo lectura (community9/clidev). No hay archivo cifrado, comandos `ag env` completos ni migracion de los 82 llamados a `std::env::var`.
```

### E4 — `STATUS.md`, Phase 4.8 Stage E annotation

Replace `contrato 0.1.0` with:

```text
contrato 0.1.0 en la entrega de la ola 4, 0.2.0 en el arbol actual (`docs/mcp/cli-json-contract-0.2.0.md`)
```

### E5 — `PRE_FASE5_RELEASE_GATE.md`, Fuzz 24h row

Only after a real 24h run per target is merged: change `pending` to `pass` and
replace the evidence cell with:

`cargo +nightly fuzz run <target> -- -max_total_time=86400 for all 7 targets on Linux x86-64; logs under docs/fuzz/results/<date>-24h.md; 0 unresolved crashes; <date>, commit <sha>; issue #156 closed.`

### E6 — `PRE_FASE5_RELEASE_GATE.md`, Benchmarks row

Only after `docs/benchmarks/pre-fase5-performance.md` exists with rule-17
metadata: change `pending` to `pass` (or `fail` with the measured gap) and cite
the report with command, date and commit; close #157.

### E7 — Phase docs / master (coordinator only, never applied by this role)

If the coordinator accepts the parity change, mirror the `STATUS.md` Stage A/B
annotations into `docs/roadmap/fase-04-7-experiencia-interactiva-de-terminal.md`
and `docs/roadmap/fase-04-8-fundaciones-del-proyecto.md`, then into
`docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, recomputing master hashes per
`docs/master/VERSION.md`. This role does not edit masters.

### E8 — `PRE_FASE5_RELEASE_GATE.md`, Audit and Deny evidence cells

Only after the coordinator reproduces `cargo audit` and `cargo deny check`
locally: keep both states as `fail`, but replace the stale text that attributes
the failure only to the resolved RUSTSEC-2026-0387, RUSTSEC-2026-0459 and
RUSTSEC-2026-0440 with the current blocker RUSTSEC-2026-0592 (`url` 2.5.8,
fixed in 2.5.9), command, date, commit and a link to issue #198. Change the
state to `pass` only when both commands exit 0 and #198 is closed.

---

## Evidence commands for the coordinator

Run on the merged tree, not on a peer branch:

```bash
# Common
git diff --check
git status --short --branch

# D3 / D5 / D8 / D9 / D12 - clidev
cargo test -p ag-cli
cargo test -p ag-cli --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
grep -rn -- '--with' crates/ag-cli/src crates/ag-cli/tests
grep -rn 'enum .*Env\|EnvList\|env_check' crates/ag-cli/src/cli.rs

# D9 - coredev
cargo test -p ag-core --all-features
grep -rn 'from_path_with_env' crates/ag-core/src
grep -rn 'std::env::var' crates/ag-cli/src crates/ag-core/src | wc -l

# D12 - mcpdev + clidev wiring
cargo test -p ag-cli --test json_output
grep -rn 'CONTRACT_VERSION' crates/ag-cli/src/output.rs
grep -rn 'config.get\|env.list\|env.check' crates/ag-cli/src crates/ag-cli/tests docs/mcp

# D15 - qafuzz (partial only)
bash -n scripts/run-fuzz-window.sh
cargo test --workspace
grep -rn '24h\|86400' docs/fuzz/wave9-window-evidence.md | head

# D16 - qabench (partial only)
test -f docs/benchmarks/pre-fase5-performance.md && echo FULL_REPORT_PRESENT || echo FULL_REPORT_MISSING

# D13 / D14 / #198 - gate rows still expected to fail
cargo audit
cargo deny check advisories
cargo tree -i url

# Content scans from .github/workflows/docs.yml
grep -RIE 'Co-Authored-By: (Claude|GPT|Copilot)' --exclude-dir=.git --exclude-dir=target .
grep -RIE 'claude\.ai/code/session_' --exclude-dir=.git --exclude-dir=target .
```

A `KEEP-PARTIAL` box stays `[/]` even when these commands are green: the
command proves the new bullet, not the whole box.

---

## Requests to the coordinator (do not relitigate here)

1. RFC-0021 section 9: record a BDFL decision for (a) which prerequisites
   `ag setup` may install and the first distributions, and (b) release-key
   custody. Without both, Phase 4.8 entry criteria and Stage C cannot advance.
2. Reconcile the master/phase-doc entry criteria: `STATUS.md` marks 4.7.1 as
   `[x]`, while `docs/roadmap/fase-04-7-experiencia-interactiva-de-terminal.md`
   and the master section keep the boxes unchecked. The verbatim-copy rule means
   the master must change first.
3. Reconcile the 4.8 Stage E contract-version annotation (E4).
4. Decide whether `ag new --with MODULE` belongs to Phase 4.7 stage B (RFC-0018
   section 4.4) or Phase 4.8 stage A (RFC-0019): `clidev` records `[modules]`
   in `ag.toml`, so both documents reference it. This decision changes which
   annotation (E1 or E2) is applied, not the no-check rule.
5. Keep MCP code unimplemented until all seven RFC-0024 prerequisites are met.
6. Reproduce issue #198 with `cargo audit` and `cargo deny check advisories`,
   then apply E8 so the gate describes the real blocker instead of closed
   issues. No row may move to `pass` without a green command and a date.
7. Expect a CHANGELOG conflict between this branch and `community9/trackers`
   (both add an `[Unreleased]` entry); merge both entries textually.

## Verification performed for this document

- `git diff --check` on this branch.
- No master, code, fuzz, benchmark or CLI file modified.
- `STATUS.md` changed only by a commentary note; no checkbox marker added,
  removed or replaced.
- Content scans for prohibited attribution and session URLs.
- All 14 baseline open issues retrieved from the GitHub board and mapped above,
  plus #198 opened by `trackers` during the wave (15 open at the final check).
