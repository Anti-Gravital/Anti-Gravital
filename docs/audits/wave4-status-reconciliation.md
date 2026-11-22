# Wave 4 STATUS reconciliation

## 1. Purpose and rule

This report maps wave-4 branches to the boxes of `docs/roadmap/STATUS.md`.
It is a proposal for the coordinator, not an edit of `STATUS.md` or of
`docs/master/`. No box is claimed as advanced here. In line with rule 26 of
`CLAUDE.md`, a box may be checked only after the corresponding code is merged
into `main` **and** the evidence below is reproduced on that merged tree.

This report deliberately records two kinds of limit:

- boxes that become checkable **only after** a branch merges, and
- boxes that **still cannot be fully checked even after** these branches merge,
  because the branch scope is narrower than the roadmap box.

The second category is the important one: a branch title such as "stage B" is
not proof that the whole roadmap box is satisfied.

## 2. Baseline evidence collected before merge

All commands were run from the `community4/qa` worktree on base `42db343`.
The numbers are a baseline, not a post-merge result.

| Command | Result |
| --- | --- |
| `cargo test -p ag-cli --test wave4_regression -- --nocapture` | 6 passed, 0 failed; 1 active, 5 print SKIP |
| `cargo test -p ag-cli --all-features --test wave4_regression -- --nocapture` | 6 passed, 0 failed; same 5 SKIPs |
| `cargo test -p ag-cli --test non_interactive` | baseline non-interactive suite passes |
| `cargo test --workspace --all-features` | exit 0, 0 failed (ignored tests remain `#[ignore]`) |
| `cargo fmt --all -- --check` | exit 0, clean |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0, no warnings |
| `git diff --check` | exit 0, clean |

The regression suite (`crates/ag-cli/tests/wave4_regression.rs`) uses runtime
feature detection rather than `#[ignore]`. When a feature lands and its branch
is merged, the same test executes its real assertions with no source change;
the SKIP lines below must then disappear.

## 3. Boxes that can be checked only after a merge

### 3.1 Fase 4.7 Etapa B - `ag config` and `ag new --with` (currently `[ ]`)

Branch: `community4/tui` (RFC-0018 stage B). The branch mission covers
`ag config` and flag parity. RFC-0018 section 4.4 also assigns `--with MODULE`
to stage B, so that flag must be confirmed in the merged diff before the whole
box is checked; otherwise the box stays partial.

Evidence commands on the merged tree:

- `cargo test -p ag-cli --test wave4_regression -- --nocapture`
  - `config_validate_non_interactive` must pass **without** printing SKIP.
  - `config_get_set_non_interactive` must pass without SKIP or the "key
    rejected" NOTE.
- `ag --no-interactive config validate` from a generated project: exit 0 for
  the scaffold config, non-zero for a malformed file (CI-usable validation,
  RFC-0018 section 4.2).
- `ag --no-interactive config get bind.host` and
  `ag --no-interactive config set bind.host 127.0.0.1`: value round-trips and
  is persisted.
- `ag --no-interactive new demo --with <module>`: confirm the flag exists and
  composes a module whose scaffold is implemented and tested.

### 3.2 Fase 4.7 Etapa C - bare `ag` home screen (currently `[ ]`)

Branch: `community4/tui` (RFC-0018 stage C).

Evidence commands on the merged tree:

- `cargo test -p ag-cli --test non_interactive` and
  `cargo test -p ag-cli --test wave4_regression`: still green, proving the
  launcher did not break the flag-only path (RFC-0018 principles 1 and 4).
- `ag` with no arguments and stdin closed: prints help, emits no escape
  sequences and does not block (already covered by the regression tests).
- Manual, in an interactive terminal: bare `ag` opens the home screen, and
  selecting an entry prints the equivalent command it runs. This is a manual
  check; the automated suite cannot open a pseudo-terminal.

### 3.3 Fase 4.8 Etapa A - `ag.toml` and Cargo-enforced layered layout (currently `[ ]`)

Branch: `community4/layered` (RFC-0019 stage A, branch mission: layered
template plus `ag.toml` manifest).

Scope caveat from the branch notes: to keep default `ag new` behavior
unchanged, the branch adds an **opt-in fifth template** `ag new
<name> --template layered`; `rest`, `realtime` and `fullstack` stay
byte-identical. RFC-0019 section 9 instead makes layered the default for
`realtime` and `fullstack`. The roadmap box says "layout por capas **forzado**
por Cargo", so an opt-in template demonstrates the dependency direction but
does not force the layout for generated projects by default. At most a partial
marker is honest after this merge.

Evidence commands on the merged tree:

- `cargo test -p ag-cli --test wave4_regression -- --nocapture`
  - `layered_template_dependency_direction` must pass **without** printing
    SKIP. It probes the `layered`, `fullstack` and `realtime` templates, then
    asserts the generated `domain`, `app`, `infra` and `api` crates respect the
    inward-only dependency rule of RFC-0019 section 4.1. The check accepts both
    plain dependency names and renamed dependencies
    (`domain = { package = "<name>-domain", ... }`).
- `ag new shop --template layered --no-interactive`: produces `ag.toml`,
  `crates/domain/`, `crates/app/`, `crates/infra/` and `crates/api/`.
- `cargo build` inside the freshly generated project: RFC-0019 PR 3 requires
  each layered template to build and run from a fresh `ag new`.
- To check the box rather than only the template, also show that the default
  `ag new` uses the layered layout, or record the opt-in deviation explicitly.
- `ag generate --check`: RFC-0019 PR 4 routes generated artifacts per layer.
  If the merged branch does not include that PR, this part of Etapa A remains
  open and the box must stay partial.

### 3.4 Fase 4.8 Etapa E (partial) - `--output json` (currently `[ ]`)

Branch: `community4/machinejson` (RFC-0021 stage E, branch mission: versioned
JSON for read commands; no MCP server code).

Evidence commands on the merged tree:

- `cargo test -p ag-cli --test wave4_regression -- --nocapture`
  - `output_json_refuses_mutating_commands` and
    `output_json_read_commands_carry_a_version_field` must pass **without**
    printing SKIP.
- `ag --output json domains list` (or another instrumented read command):
  stdout is a JSON object carrying a version marker. The branch notes name the
  field `contract_version`; full shape snapshots belong in `docs/mcp/`. Prose
  output is unchanged when the flag is absent.
- `ag new probe --output json --no-interactive`: non-zero exit, an explanatory
  refusal, and no files written.

### 3.5 Fase 4.7 exit criterion - non-interactive regression suite on four platforms (currently `[/]`)

My new tests add coverage, but this criterion explicitly requires Linux
x86-64, Linux ARM64, macOS ARM64 and Windows x64. It stays `[/]` until the CI
matrix is green on all four after the merges.

Evidence: the `ci.yml` / `quality.yml` matrix runs of
`cargo test --workspace --all-features` on each target; the current note in
`STATUS.md` records only Linux x86-64 as verified.

## 4. Boxes that stay open even after these branches merge

These are narrow-scope branches. Do not check their roadmap boxes on merge
alone.

| Roadmap box | Branch | Missing for a genuine `[x]` |
| --- | --- | --- |
| Fase 4.8 Etapa A - `ag.toml` and Cargo-enforced layered layout | `community4/layered` | The branch adds an opt-in `--template layered`; default `ag new` templates stay unchanged, so the layout is not forced. Generator routing (`ag generate --check`, RFC-0019 PR 4) is also outside the branch mission. Partial at most. |
| Fase 4.8 Etapa B - env, `Secret<T>`, encrypted secrets, `ag env` | `community4/envsecrets` | The role's own notes state the encrypted file and the `ag env` CLI are **not** in scope; the branch delivers only declared env vars plus `Secret<T>` in `ag-core` and does not touch `ag-cli`. At most a partial marker for the loader is honest. |
| Fase 4.8 Etapa E - `--output json`, `ag describe`, plan/apply | `community4/machinejson` | The branch covers only structured output and refusal on mutating commands. `ag describe` (RFC-0021 section 4.3 item 3) and `--plan` / `--apply` (item 4) are not implemented. The box stays open. |
| Fase 4.7 exit criteria - three external testers, SSH/tmux/Windows console, binary-size delta | `community4/tui` | Human/platform evidence; not produced by any branch's automated tests. |
| Fase 4.8 Etapas C and D - signed release binaries / `ag setup`, `ag servers` / `ag connect` | none in wave 4 | No branch in this wave targets them; RFC-0021 open questions on `ag setup` scope and release-key custody are still undecided. |

## 5. Documentation consistency to fix at merge time (not in this report)

- `RFC-0018` section 9 currently records "stage A is implemented; stages B and
  C are not started". After the `tui` merge this sentence, the Phase 4.7 lines
  in `STATUS.md`, and the bilingual README/CLI tables must be updated together
  (rules 19, 26 and the README synchronization rule).
- `RFC-0021` has no implementation-status paragraph; after the `machinejson`
  merge, add one limited to the structured-output part if the owner wants the
  RFC to reflect reality. Do not imply `ag describe` or plan/apply exist.
- `RFC-0019` and `RFC-0020` status lines must reflect only what actually
  landed, with the scope limits of section 4. For `RFC-0019` the branch notes
  record an explicit deviation: layered becomes an opt-in template instead of
  the default for `realtime` and `fullstack`; that deviation must be stated, not
  silently presented as full stage-A delivery.

## 6. Verification record for this branch

Run on 2026-11-21 from `worktrees4/qa` with the shared Cargo target directory
and `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0`:

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo test --workspace --all-features`: exit 0, 0 failures. Tests that need a
  live database or an external service remain `#[ignore]` and were not run.
- `cargo test -p ag-cli --test wave4_regression -- --nocapture`: 6 passed. The
  single unconditional test protects the existing `--no-interactive` flow.
- `cargo test -p ag-cli --all-features --test wave4_regression -- --nocapture`:
  6 passed, confirming the suite is feature-flag independent.
- `git diff --check`: exit 0.

The 5 SKIPs are expected on this branch: no wave-4 feature branch is merged
into it, so `ag config`, global `--output` and the layered template are absent.
They are the tests the coordinator must see go green on the merged tree.
