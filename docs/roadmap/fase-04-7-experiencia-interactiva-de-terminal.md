# Fase 4.7 - Experiencia interactiva de terminal (TUI de ag)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-6-mta-nativo-y-ag-workers.md](./fase-04-6-mta-nativo-y-ag-workers.md)
> Siguiente: [fase-04-8-fundaciones-del-proyecto.md](./fase-04-8-fundaciones-del-proyecto.md)

## Phase 4.7 — Interactive terminal experience (`ag` TUI)

**Status: in progress (`RFC-0018` accepted 2026-11-10). Stage A is implemented;
stages B and C are not started.** Additive phase.

**Objective.** Make creating and configuring a project discoverable and guided
from the terminal, through a full-screen interactive interface in the `ag`
binary, without weakening scripting: every interactive action has an exact
non-interactive equivalent. The product is the experience promised by rule 32:
a new user reaches a running project without reading the manual first.

**Duration:** 1–2 months. Runs in parallel with the gate closing and Phase 5.

### 4.7.1 Entry criteria

- [x] `RFC-0018` accepted, including its three open questions (interface
  language, gate position, behavior of bare `ag`).
- [x] `ag-cli` split into modules with no behavior change, verified by the
  existing command-tree test. The typed request and single executor exist for
  `ag new`; each other command gets its own when it gains an interactive flow.

### 4.7.2 Deliverables

Stage A — foundation and project creation:

- [x] `tui` Cargo feature in `ag-cli` (default on); the binary builds and all
  commands work with `--no-default-features`.
- [x] Terminal guard that restores the terminal on exit, error, `Ctrl-C` and
  panic.
- [x] `ag new` wizard: name with live validation, template with description,
  review screen, result printed after the interface closes. `NAME` becomes
  optional.
- [x] Global `--no-interactive` flag and `AG_NO_TUI`; automatic non-interactive
  mode when not a terminal, when `CI` is set or when `TERM=dumb`.
- [x] Equivalent non-interactive command printed at the end of every flow.
- [x] Monochrome theme honoring `NO_COLOR`; ASCII and box-drawing glyphs only.
- [x] Degraded line-prompt mode below 80x24.
- [x] A single interaction library in `ag-cli` (the previous prompt library is
  removed).

Stage B — project configuration:

- [ ] `ag config get`, `ag config set`, `ag config validate` (non-interactive,
  usable in CI).
- [ ] `ag config` form generated from the configuration types of `ag-core`,
  with inline validation, defaults and descriptions.
- [ ] Format-preserving, atomic save with a diff on the review screen.
- [ ] Secret fields accept only a file path or an environment variable name.
- [ ] `ag new --with MODULE` for the first standard module with tested scaffold
  composition, then the matching wizard step.

Stage C — home screen:

- [ ] Bare `ag` in a terminal opens a context-aware launcher (outside a project
  / inside a project). Outside a terminal it prints help as before.
- [ ] Every launcher entry runs an existing command and displays it.

Cross-stage:

Each box below is checked for the screens that exist (stage A) and applies
again to every screen added by stages B and C.

- [x] State-transition unit tests for every screen.
- [x] Render tests at 80x24 and 120x40, color and monochrome, asserting on the
  rendered buffer.
- [x] Parity tests: the request built by a wizard equals the request parsed
  from its printed equivalent command.
- [x] Non-interactive regression tests: no command blocks or emits escape
  sequences with stdin closed.
- [x] Interaction flow diagram in `docs/diagrams/`.
- [ ] `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`, root
  `README.md` and the first-API manual chapter updated in the same pull
  request as each stage.

### 4.7.3 Exit criteria

- [ ] A person who has never used Anti-Gravital creates and starts a project
  using only `ag new` in a terminal, with no flags and no documentation.
  Verified with at least three people external to the project; the session
  notes are published.
- [ ] Every command passes the non-interactive regression suite on the four CI
  platforms.
- [ ] The wizard works over SSH and inside `tmux`, and on the Windows console.
- [ ] The size increase of the release `ag` binary caused by the `tui` feature
  is measured and published.
- [ ] No interactive flow has a capability missing from the flags (parity
  suite green).

### 4.7.4 Phase risks

The main risk is **two diverging products**: an interactive tool and a
scripting tool that slowly stop behaving the same. The mitigation is
architectural, not procedural: screens produce the same typed request as the
flag parser and one executor performs the work, with parity tests in CI.

The second risk is **scope creep toward a terminal IDE** (log viewers,
editors, dashboards). The mitigation is the rule fixed in `RFC-0018`: a screen
is a front-end to an existing command; anything else requires a new RFC.

The third risk is **terminal diversity**. The mitigation is a conservative
glyph set, a monochrome theme, four-platform CI and the degraded line mode.

---

