# Interactive `ag` terminal journey - wave 10 real-user audit

Scope: exercise the interactive interface of the `ag` binary as a real user in a
pseudo-terminal, with real keystrokes, and record only captured output. No
screen in this document was invented or reconstructed from source.

## Method and provenance

- Binary: built from worktree `worktrees10/tui`, branch `community10/tui`,
  commit `72fa9d8`, into private target
  `/home/angel-nereira/Github-local-angelnereira/.w10-tui-target`.
  Build command used
  `--no-default-features --features tui` because the tmpfs `/tmp` was under a
  quota and the full default feature set could not link there. The only
  behavioural difference is that the `ag workers doctor` menu entry (gated by
  `workers-runtime`) is absent from the in-project home screen.
- Harness: `tools/tui-pty-harness.py` (Python standard library). It uses
  `pty.fork()`, sets the window size with `TIOCSWINSZ`, injects key bytes into
  the master fd, captures the raw byte stream with timestamps, and rebuilds the
  visible screen with a small ANSI interpreter that accumulates ratatui's
  differential redraws. Alternate-screen enter/leave events reset the buffer.
- Environment: `TERM=xterm-256color` by default; `CI`/`AG_NO_TUI`/`NO_COLOR`
  removed unless the scenario sets them. Working directory is a fresh temp
  project per scenario.
- Raw machine-readable transcripts for every scenario are in
  `docs/audits/tui-transcripts/` (one JSON per scenario plus `summary.json`).
  They contain raw byte reprs, key sequences, exit codes and detected escape
  sequences (`alt_screen_enter`, `alt_screen_leave`, `color_sgr_count`).
- Every process exited on its own. **No scenario timed out and no prompt
  hung.** The harness applies a 2 s post-script wait before killing a process;
  that kill path was never taken.

## Result overview

| Scenario (transcript) | Keys sent | Exit | Hung? | Outcome |
| --- | --- | --- | --- | --- |
| `home-nav-launch` | Down, Down, Enter | 0 | no | Home menu navigated; selected entry ran |
| `home-in-project` | q | 0 | no | Context-aware menu shown |
| `home-quit` | q | 0 | no | Clean exit, terminal restored |
| `new-wizard-full` | name, Enter, Down, Enter, Enter | 0 | no | Three-step wizard completed, project created |
| `new-wizard-cancel` | name, Enter, q | 0 | no | Aborted, nothing written |
| `new-wizard-invalid-name` | invalid name, Enter, Backspace, name, Esc | 0 | no | Inline error, corrected, cancelled |
| `config-form-edit-submit` | Enter, Backspace x30, value, Enter, r, Enter | 0 | no | Form edited, review shown, config.toml written |
| `config-form-quit` | q | 0 | no | Cancelled, no file change |
| `config-small-terminal` | none | 1 | no | Size error, no form |
| `new-small-terminal-lines` | name, Enter, Enter, y | 0 | no | Line-mode fallback completed |
| `new-small-terminal-eof` | Ctrl-D | 0 | no | Cancelled on EOF |
| `home-no-color` | q | 0 | no | Interactive, zero colour sequences |
| `home-term-dumb` | none | 0 | no | Help text, no TUI |
| `new-noninteractive-flag` | none | 2 | no | Usage error |
| `config-noninteractive-flag` | none | 2 | no | Usage error |
| `new-stdin-pipe` | none (EOF) | 2 | no | Usage error |
| `home-pipe-stdin-tty-out` | none (EOF) | 0 | no | Help text, no TUI |

## 1. Bare `ag` home screen

Transcript: `home-nav-launch.json`, `home-in-project.json`, `home-quit.json`.
Terminal 100x30, no project fixture.

Initial captured screen (verbatim excerpt):

```text
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│Anti-Gravital  |  Home                                                                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│No project detected. What would you like to do?                                                   │
│                                                                                                  │
│> Create a new project                                                                            │
│    Guided creation with a name and a starting template.                                          │
│  Help and documentation                                                                          │
│    Full command reference and usage.                                                             │
│  Show the version                                                                                │
│    Print the installed ag version.                                                               │
│                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│Up/Down: select    Enter: run command    Esc/q: close                                             │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Key sequence `\x1b[B` (Down) moved the `>` marker from "Create a new project"
to "Help and documentation", then to "Show the version". The captured screen
after the second Down shows:

```text
│  Help and documentation                                                                          │
│    Full command reference and usage.                                                             │
│> Show the version                                                                                │
│    Print the installed ag version.                                                               │
```

Enter (`\r`) left the alternate screen and ran the selected entry. The exact
captured bytes were:

```text
b'\x1b[?25h\x1b[?1049l\x1b[?25hag --version\r\nag 0.1.0-alpha.2\r\n'
```

Observation: the chosen command line is printed before it is executed
(`ag --version` above). This is the documented launcher behaviour; it is a
visible extra line, not a failure.

Inside a project (fixture with `config.toml` and a `Cargo.toml` containing
`ag-core`), the same bare `ag` showed a different menu:

```text
│Project detected. What would you like to do?                                                      │
│                                                                                                  │
│> Run the development server                                                                      │
│    Hot reload while you edit (requires cargo-watch).                                             │
│  Generate artifacts from schema.ag                                                               │
│    Rust, SQL, TypeScript and OpenAPI output.                                                     │
│  Lint the DSL schema                                                                             │
│    Report best-practice warnings in schema.ag.                                                   │
│  Configure this project                                                                          │
│    Open the config.toml form.                                                                    │
│  List attached domains                                                                           │
│    Read-only status from the local attachment store.                                             │
│  List background workers                                                                         │
│    Workers declared in schema.ag, no infrastructure needed.                                      │
```

Quitting with `q` produced `\x1b[?25h\x1b[?1049l\x1b[?25h` followed by the
normal CLI help, so the alternate screen and cursor were restored on every
interactive exit path exercised.

## 2. `ag new` wizard with no NAME and no TEMPLATE

Transcript: `new-wizard-full.json`. Terminal 100x30. Keys:
`demo-app`, Enter, Down, Enter, Enter.

Step 1 captured:

```text
│Anti-Gravital  |  New project  |  Step 1 of 3: Name                                               │
│Project name                                                                                      │
│> _                                                                                               │
│Letters, numbers, hyphens and underscores. It is also the directory name.                         │
│Enter: continue    Esc: cancel                                                                    │
```

Typing `demo-app` updated the field in place (`> demo-app_`). Enter advanced to
step 2:

```text
│Anti-Gravital  |  New project  |  Step 2 of 3: Template                                           │
│Choose a starting template                                                                        │
│  Template 1 of 33                                                                                │
│> rest                                                                                            │
│    [api] Minimal REST API on ag-core (Shield + axum) with one example route.                     │
│  realtime                                                                                        │
│    [realtime] WebSocket server on ag-core with a /ws echo route.                                 │
│  ... 27 more below                                                                               │
│Up/Down: select     Enter: continue    Esc: back    q: cancel                                     │
```

Down moved the marker to `> realtime` and the counter to "Template 2 of 33".
Enter advanced to step 3:

```text
│Anti-Gravital  |  New project  |  Step 3 of 3: Review                                             │
│Review before creating                                                                            │
│Directory: ./demo-app                                                                             │
│Template: realtime [realtime]                                                                     │
│Files to be created                                                                               │
│  demo-app/Cargo.toml                                                                             │
│  demo-app/config.toml                                                                            │
│  demo-app/src/main.rs                                                                            │
│Equivalent command                                                                                │
│  ag new demo-app --template realtime                                                             │
│Nothing has been written yet.                                                                     │
│Enter: create project    Esc: back    q: cancel                                                   │
```

Enter created the project; captured output:

```text
Creating project 'demo-app' with template 'realtime'...

Project 'demo-app' created.

Next steps:
  cd demo-app
  ag dev

Equivalent non-interactive command:
  ag new demo-app --template realtime
```

The transcript's `final_files` records `demo-app/` containing `Cargo.toml`,
`config.toml` and `src`. Nothing was created before the final Enter.

Cancel and validation transcripts: `new-wizard-cancel.json` (name typed,
Enter, `q` -> `Cancelled. Nothing was written.`, no directory) and
`new-wizard-invalid-name.json` (`bad name` produced the inline message
`Error: invalid project name 'bad name': only letters, numbers, hyphens, and
underscores are allowed`; Enter did not advance; Backspace and Esc cancelled
without writing).

## 3. `ag config` form with no subcommand

Transcript: `config-form-edit-submit.json`, terminal 100x30. Keys: Enter,
Backspace x30, `127.0.0.1:9000`, Enter, `r`, Enter.

Initial fields screen:

```text
│Anti-Gravital  |  Configure project  |  Step 1 of 2: Fields                                       │
│[Bind address]                                                                                    │
│> bind                        0.0.0.0:8080                                                        │
│    Listener address as host:port.                                                                │
│[Runtime]                                                                                         │
│  runtime.workers                                                                                 │
│  runtime.blocking_threads    512                                                                 │
│[CORS]                                                                                            │
│  cors.enabled                false                                                               │
│Up/Down: select    Enter: edit    r: review    Esc/q: cancel                                      │
```

Enter entered edit mode (`> bind  0.0.0.0:8080_`, footer
`Type: replace value    Enter: apply    Esc: cancel`). Clearing the value
showed inline validation `Error: bind: expected host:port (invalid socket
address syntax)`; typing the new value cleared it. Enter committed, `r` opened
review:

```text
│Anti-Gravital  |  Configure project  |  Step 2 of 2: Review                                       │
│Changes to write to config.toml                                                                   │
│bind: 0.0.0.0:8080 -> 127.0.0.1:9000                                                              │
│Equivalent command                                                                                │
│  ag config set bind 127.0.0.1:9000                                                               │
│Nothing has been written yet.                                                                     │
│Enter: save    Esc: back to fields    q: cancel                                                   │
```

Enter wrote the file:

```text
Updated config.toml (1 change(s)).
Equivalent non-interactive command:
  ag config set bind 127.0.0.1:9000
```

`final_files` contains `config.toml` with exactly
`bind = "127.0.0.1:9000"`. `config-form-quit.json` confirms `q` prints
`Cancelled. config.toml was not changed.` and leaves no file.

Secret handling: the form exposes 21 keys that are either paths, addresses,
numbers, booleans or lists. No inline secret field exists; the config source
explicitly excludes inline JWT public key material. No typed value was echoed
in a way that reveals a secret, and no secret-bearing field was present to
test. This is a source-and-observation statement, not a claim that a secret
masking feature exists.

## 4. Degraded modes

### NO_COLOR=1

Transcript `home-no-color.json`: the home screen still opened
(`alt_screen_enter` = 1, exit 0, `q` quit cleanly) with the same menu and
`color_sgr_count` = 0 in every captured step. Borders are Unicode box drawing,
which is a glyph choice rather than colour, so NO_COLOR is respected.

### TERM=dumb

Transcript `home-term-dumb.json`: no alternate screen was entered
(`alt_screen_enter` = 0). The CLI help was printed and the process exited 0.
The command list contained this real, user-visible line:

```text
  dns       passed explicitly. See `docs/dns/DOMAIN_LOOP.md`
```

This is a truncated clap doc comment (`crates/ag-cli/src/cli.rs`, `Dns`
variant). It is already tracked as issue #200;
a corroborating PTY observation was added to that issue. No duplicate issue
was opened.

### Small terminal (60x20)

- `config-small-terminal.json`: `ag config` printed
  `error: the terminal is too small for the configuration form (minimum 80x24)`
  and exited 1. It did not enter the alternate screen and did not hang.
- `new-small-terminal-lines.json`: `ag new` degraded to line prompts. Captured:
  `Project name: `, then a numbered list of 33 templates ending with
  `33. testing-suite Integration, fuzz and benchmark test harness scaffold.`,
  then `Template number [1]: `, then a review with `Directory: ./lineproj`,
  `Template: rest` and `Create this project? [Y/n]: `. Answering `y` created
  `lineproj/` with `Cargo.toml`, `config.toml` and `src`.
- `new-small-terminal-eof.json`: Ctrl-D at `Project name: ` produced
  `Cancelled. Nothing was written.` and exit 0. No hang.

### Non-interactive stdin / flags

- `new-stdin-pipe.json`: piped stdin produced
  `error: the project name is required when the session is not interactive.`
  plus the usage line, exit 2, no prompt.
- `home-pipe-stdin-tty-out.json`: piped stdin with a TTY stdout printed the
  help text, exit 0, no alternate screen.
- `new-noninteractive-flag.json`: `ag new --no-interactive` in a full 100x30
  PTY produced the same usage error, exit 2, no prompt.
- `config-noninteractive-flag.json`: `ag config --no-interactive` printed
  ``error: `ag config` needs a terminal for the form.`` and the non-interactive
  usage line, exit 2, no form.

## 5. Defects and issue status

New defects opened from this journey: none. The only reproducible defect
observed (truncated `dns` help text) was already open as issue #200 with the
same reproduction, so this audit adds evidence to that issue instead of
duplicating it. No issue was closed.

Checked and found working: arrow/j/k navigation, Enter/Esc/q key handling,
inline validation, cancel paths with no side effects, review-before-write,
alternate-screen and cursor restoration on every exit, NO_COLOR behaviour,
small-terminal and non-TTY fallbacks, and EOF handling without hangs.

## 6. Reproduce

```sh
cd worktrees10/tui
python3 tools/tui-pty-harness.py \
  --binary /home/angel-nereira/Github-local-angelnereira/.w10-tui-target/debug/ag \
  --out /tmp/opencode/w10-tui-repro
# Per-scenario raw captures: /tmp/opencode/w10-tui-repro/<id>.json
```

The harness is deterministic in its key sequences; timing fields in the
transcripts (`started_at`, `finished_at`, `delay_after_s`) are observations,
not assertions. To inspect a screen without a terminal emulator, read the
`screen` field of a step in the corresponding JSON file.
