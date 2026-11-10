# RFC-0018 - Interactive terminal experience (TUI) for the `ag` CLI

- Status: accepted
- Decision date: 2026-11-10
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: new additive Phase 4.7 (stages A-C); later phases add screens
- Affected modules/crates: `ag-cli` (new `tui` module and feature, new
  `ag config` command, optional `NAME` in `ag new`, bare `ag` home screen)
- Predecessor RFC: none
- Comment period: waived by BDFL decision.

## 1. Motivation

Rule 32 of `CLAUDE.md` sets the ergonomics target of the project: a developer
types `ag new my-api`, then `ag dev`, and starts quickly. Today that is true
only for someone who already knows the template names, the flags of each
command and the keys of `config.toml`. Everything else has to be learned from
the documentation before the first useful action.

The end user of Anti-Gravital works from the terminal. The terminal therefore
has to be the place where creating and configuring a project is discoverable,
guided and hard to get wrong, without giving up what makes a CLI valuable:
scriptability, reproducibility and operation without a display server.

This RFC defines an interactive, full-screen terminal interface (TUI) for the
`ag` binary, starting with the two flows that matter most to a new user:
creating a project and configuring a project.

## 2. Problem

Verified against the current `ag-cli` (`crates/ag-cli/src/main.rs`):

1. The only interactive element is a single template selector in `ag new`,
   built with a line-prompt library. There is no review step, no way to go
   back, and the project name is a mandatory positional argument, so the
   command cannot be started without already knowing what to type.
2. Project configuration is a hand-edited `config.toml`. Nothing validates it
   before `ag dev` fails at startup, and nothing lists which keys exist.
3. The command surface has grown to eight top-level commands and more than
   twenty-five subcommands (`schema`, `domains`, `mail`, `workers`, `dlq`).
   Discovery is `--help` only. A user inside a project has no single place
   that shows what can be done next.
4. Upcoming phases add flows with many decisions and irreversible effects
   (`ag deploy`, `ag rollback`, `ag migrate from`, `ag plugin add`). Without a
   shared interaction model each one would invent its own prompts.

## 3. Alternatives considered

**A. Do nothing; keep flags and `--help`.** Zero cost and zero new
dependencies. Rejected: it leaves rule 32 unmet for new users and pushes the
problem into every later phase.

**B. Extend line prompts (the current approach) to more questions.** Small
and simple. Rejected as the end state: line prompts cannot show a persistent
summary, cannot navigate backwards, cannot render a form with inline
validation, and cannot host a home screen. It remains the degraded mode when
the terminal is too small (section 4.6).

**C. A separate `ag-tui` crate.** Clean separation. Rejected: it changes the
crate roster (rule 14) for code that has exactly one consumer. A module behind
a Cargo feature inside `ag-cli` gives the same isolation at lower cost.

**D. A local web UI served by `ag dev`.** Richer widgets. Rejected for this
purpose: it needs a browser, does not work over SSH without port forwarding,
and does not exist before a project is created. The Phase 6 knowledge-graph
dashboard is a different deliverable and is not affected by this RFC.

**E. A full-screen TUI inside `ag-cli`, as a front-end over the existing
commands (chosen).** Described below.

## 4. Proposed design

### 4.1 Principles (normative)

1. **Flag parity.** Every decision that can be made in the TUI can be made
   with a flag or a configuration key. The TUI never has a capability the
   non-interactive path lacks. Automation, CI and documentation always use the
   non-interactive form.
2. **One executor.** A screen does not perform work. It produces the same
   typed request value that the flag parser produces (for example
   `NewProjectRequest`), and a single function executes it. The TUI and the
   flags cannot drift because they share the request type and the executor.
3. **Reproducibility.** When an interactive flow finishes, `ag` prints the
   equivalent non-interactive command so the action can be repeated, scripted
   and pasted into documentation (rule 36).
4. **Never interactive by surprise.** The TUI starts only when all of these
   hold: stdin and stdout are terminals, `TERM` is not `dumb`, the `CI`
   environment variable is unset, `AG_NO_TUI` is unset and `--no-interactive`
   was not passed. In every other case the command behaves exactly as it does
   today. Existing scripts keep working unchanged.
5. **Terminal safety.** Raw mode and the alternate screen are always restored:
   on normal exit, on `Ctrl-C`, on error and on panic (guard value plus panic
   hook). A crashed `ag` must never leave a broken terminal.
6. **Accessible by construction.** Keyboard only; no mouse required. `NO_COLOR`
   is honored and no information is conveyed by color alone. Glyphs are limited
   to ASCII and box-drawing characters; no emojis (repository rule). Minimum
   supported size is 80x24; below it the flow degrades to line prompts.
7. **Explicit side effects.** Nothing is written, sent or deployed before a
   review screen shows exactly what will happen and the user confirms. No
   network access happens during a wizard unless the user triggers it.
8. **No hidden state.** The TUI writes project files only. It keeps no global
   configuration, history or cache in this RFC's scope.

### 4.2 Command surface changes

| Command | Change | Stage |
| --- | --- | --- |
| `ag new [NAME]` | `NAME` becomes optional. With every input given the command is unchanged. With inputs missing and an interactive terminal, the creation wizard opens. With inputs missing and no terminal, it fails with a message naming the missing argument (today: clap error). | A |
| `--no-interactive` (global flag) | Forces the non-interactive path on any command. | A |
| `ag config` | New command. In a terminal: configuration form for the project's `config.toml`. | B |
| `ag config get KEY`, `ag config set KEY VALUE`, `ag config validate` | Non-interactive parity for the form (principle 1). `validate` exits non-zero on an invalid file and is usable in CI. | B |
| `ag` (no arguments) | In a terminal: home screen. Otherwise: prints help, as today. | C |

No existing command, flag or exit code changes meaning.

### 4.3 Stage A - foundation and project creation wizard

Scope is deliberately limited to what the scaffolder supports today (three
templates). The wizard does not invent options the scaffolder cannot honor.

Steps:

1. **Name.** Validated on every keystroke with the existing
   `validate_project_name`; also rejects an existing directory.
2. **Template.** `rest`, `realtime`, `fullstack`, each with a one-paragraph
   description and the list of files it creates.
3. **Review.** Target directory, template, file list, and the equivalent
   command. `Enter` confirms, `Esc`/`b` goes back, `q` aborts without writing.
4. **Result.** Files written, next steps (`cd NAME`, `ag dev`), and the
   reproducible command line printed after the alternate screen closes.

Foundation delivered with this stage: the `tui` module, terminal guard, key
map, theme (color and monochrome), the screen/state-machine pattern of section
4.7, and the test harness. The line-prompt dependency is removed; `ag-cli`
keeps exactly one interaction library.

Prerequisite refactor (no behavior change, own pull request): split the
single 1 800-line `main.rs` into `cli.rs` (clap definitions), one module per
command group, and `requests.rs` (typed requests + executors). Principle 2
cannot be implemented cleanly on a single-file binary.

### 4.4 Stage B - project configuration

`ag config` renders `config.toml` as a form grouped by section (bind address,
TLS, rate limiting, CORS, CSRF, authentication keys). Rules:

- The form is generated from the same configuration types `ag-core` uses to
  load the file. There is no second schema to keep in sync.
- Each field shows its current value, its default and a one-line description.
  Validation uses the loader's own validation and is shown inline.
- Saving preserves comments and key order of the existing file
  (format-preserving TOML editing), shows a diff on the review screen and
  writes atomically (temporary file plus rename).
- Secrets are never typed into or displayed by the form. Fields that hold
  secret material accept a file path or an environment variable name only
  (rule 16).

Extending `ag new` with optional module selection (for example adding
`ag-auth` or `ag-cache` to a new project) belongs to this stage, and only for
modules whose scaffold composition has been implemented and tested. Each
selectable module gets a `--with MODULE` flag first, then a checkbox.

### 4.5 Stage C - home screen

Bare `ag` in a terminal opens a home screen that depends on context:

- **Outside a project:** create a project; open documentation paths; version.
- **Inside a project** (detected by `config.toml` plus `Cargo.toml` depending
  on `ag-core`): run dev server, generate from `schema.ag`, lint schema,
  configure, and the read-only status commands that already exist
  (`ag domains list/status`, `ag workers list`, `ag workers doctor`).

The home screen is a launcher: selecting an entry runs the same command the
user could have typed and shows that command. It adds no functionality.

### 4.6 Degraded and non-interactive modes

| Condition | Behavior |
| --- | --- |
| Not a terminal, `CI` set, `AG_NO_TUI` set, `--no-interactive`, `TERM=dumb` | No prompts of any kind. Missing required input is an error with exit code 2. |
| Terminal smaller than 80x24 | Sequential line prompts built on the same key-reading layer; same request type, same executor. |
| `NO_COLOR` set | Monochrome theme; selection shown with markers and reverse video. |
| Built with `--no-default-features` | No `tui` module compiled; behaves as the first row. |

### 4.7 Internal architecture

```
key event -> Msg -> update(&mut Model, Msg) -> Option<Effect>
                         |
                     view(&Model, &mut Frame)
```

- `Model` is plain data, `update` is a pure function, `view` only reads the
  model. No I/O in either. Effects (`Submit(Request)`, `Quit`) are returned to
  the event loop, which is the only place that touches the terminal or the
  file system.
- Location: `crates/ag-cli/src/tui/` (`mod.rs`, `terminal.rs`, `theme.rs`,
  `keys.rs`, one file per screen). Cargo feature `tui`, enabled by default.
- The diagram of this flow is added under `docs/diagrams/` in the stage A
  pull request (rule 34).

### 4.8 Dependencies

| Dependency | Purpose | Justification (rule 15) |
| --- | --- | --- |
| `ratatui` | Layout, widgets, buffer rendering | The maintained community successor of `tui-rs`; MIT; widely deployed in Rust terminal tools; ships a `TestBackend` that makes rendering testable without a terminal. |
| `crossterm` | Raw mode, key events, alternate screen | Pure Rust, MIT, supports Linux, macOS and Windows consoles, which are the four CI targets. |
| `toml_edit` (stage B) | Format-preserving edits of `config.toml` | Same maintainers as the `toml` crate already in the workspace. |

Removed: the current line-prompt library. All three additions are optional
dependencies of the `tui` feature, are checked by `cargo deny` and
`cargo audit` like any other, and do not reach any library crate: only the
`ag` binary links them.

### 4.9 Testing (rule 18)

- **Unit:** `update` transition tests for every screen, including back
  navigation, abort and validation errors.
- **Render:** buffer snapshot tests on `TestBackend` at 80x24 and 120x40, in
  color and `NO_COLOR` themes.
- **Parity:** for each flow, a test proves that the request built by the wizard
  equals the request built by parsing the printed equivalent command.
- **Non-interactive regression:** the existing command-tree test plus tests
  that every command run with stdin closed never blocks and never emits escape
  sequences.
- **Terminal restoration:** a test that forces a panic inside the event loop
  and asserts the guard ran.
- **Platforms:** the suite runs on the four CI targets; no pseudo-terminal is
  required because rendering and state are tested below the terminal layer.

### 4.10 Documentation (rule 19)

Each stage updates, in the same pull request: `crates/ag-cli/README.md`,
`docs/modules/ag-cli/README.md`, the root `README.md` (CLI table and quick
start, EN+ES), the manual chapter on the first API, and
`docs/graph/knowledge-graph.json` (new commands).

### 4.11 Master document changes

The roadmap gains Phase 4.7 (stages A-C) and one TUI deliverable in each later
phase that introduces a multi-decision command (Phases 5, 7 and 9). The
technical architecture gains the `ag config` command in the `ag-cli` row when
stage B is implemented, not before (rule 26: documentation does not simulate
capabilities that do not exist).

## 5. Implementation plan (only after approval)

1. PR 1 - `ag-cli` module split, no behavior change.
2. PR 2 - stage A: `tui` feature, foundation, `ag new` wizard, removal of the
   line-prompt library, tests, docs.
3. PR 3 - stage B: `ag config` (`get`/`set`/`validate` first, then the form).
4. PR 4 - stage B: `--with MODULE` composition for the first module, then the
   wizard checkbox.
5. PR 5 - stage C: home screen.

Each PR is independently releasable and independently revertible.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| The TUI gains capabilities the flags lack, breaking automation | Medium | High | Principles 1 and 2 enforced by the parity tests of 4.9; review checklist item. |
| Terminal left in raw mode after a crash | Low | High | Guard plus panic hook; dedicated test. |
| Rendering differences across terminals and Windows consoles | Medium | Medium | ASCII and box-drawing only; monochrome theme; four-platform CI; degraded line mode. |
| Binary size and build time growth of `ag` | Medium | Low | Feature-gated; size delta measured and recorded in the stage A pull request. |
| Scope creep toward a terminal IDE (log viewers, editors, dashboards) | Medium | Medium | The home screen is a launcher only (4.5). Any screen that is not a front-end to an existing command requires a new RFC. |
| Maintenance cost of two paths per command | Medium | Medium | One executor per request; screens contain no business logic. |

## 7. Impact

- **Scope:** no new product capability; a new way to reach existing ones, plus
  one new command (`ag config`).
- **Roadmap:** adds additive Phase 4.7. It does not gate the start of Phase 5.
  Stages A and B are required for the v0.5 public beta, because the beta is
  the first public impression of the tool; stage C is required before 1.0.
- **Operational complexity:** none at runtime; the TUI exists only in the
  developer-side binary.
- **Public APIs:** additive CLI surface only. No library API changes.
- **Documentation:** as listed in 4.10 and 4.11.

## 8. Rollback

The `tui` feature can be turned off by default in one line, restoring today's
behavior, because principle 4 guarantees the non-interactive path is complete.
Signals that would justify it: recurring terminal-corruption reports, parity
regressions, or evidence that the wizard is not used. `ag config
get/set/validate` are useful without the TUI and would be kept.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-10.
- Result: accepted. The comment period was waived by BDFL decision.

Resolution of the open questions:

1. **Interface language.** English in stages A to C. All user-facing strings
   live in one module (`crates/ag-cli/src/tui/text.rs`) so a later RFC can add
   Spanish without touching screen logic.
2. **Gate position.** Stages A and B are exit criteria of the v0.5 public
   beta; stage C is an entry criterion of Phase 10.
3. **Bare `ag`.** In an interactive terminal it opens the home screen
   (stage C). Outside a terminal it prints help as before.

Implementation status: stage A is implemented (module split, `tui` feature,
`ag new` wizard, `--no-interactive`). Stages B and C are not started.
One deviation from section 4.8: `crossterm` is used through the re-export of
`ratatui` instead of being declared as a second direct dependency, so the two
versions cannot diverge.

## 10. References

- `CLAUDE.md` rules 13, 15, 18, 19, 28, 32, 36.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, Phase 4.7.
- `crates/ag-cli/README.md`, `docs/modules/ag-cli/README.md`.
- `ADR-0009` (native-by-default, real-state documentation).
