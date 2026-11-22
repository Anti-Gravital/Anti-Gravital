# Terminal User Review - Wave 10 (real-user journeys)

> Consolidated, evidence-backed improvement plan for the real terminal QA of
> wave 10. This audit does not change the architectural contract. It ranks what
> real users actually observed, separates reproducible defects from friction,
> and makes every recommendation testable with a command that proves the fix.

- **Date:** 2026-11-22
- **Branch:** `community10/dxreview`
- **Scope:** the five Anti-Gravital terminal journeys executed in wave 10
  (install, TUI, scaffold, DNS/deploy/env/config, lockfiles) plus the
  cross-repository Fabric install journey. This review reads the artifacts
  produced by the peer roles; it does not repeat their runs.
- **Sources:** peer notes under `/tmp/opencode/agents/w10-<role>.md` and the
  evidence docs on each peer branch (referenced as
  `community10/<role>:docs/audits/...`, pending merge into main). Independently
  verified in this worktree: the live GitHub issue board, the version strings
  in the README/manual/workspace manifest, and the generated template bindings.
- **Baseline commit under test:** `72fa9d836a6de2f4f96429c11ffcff13142b2ef4`.

## Severity rubric

| Severity | Meaning |
| --- | --- |
| Blocker | Prevents the documented quickstart from completing, or exposes the user to immediate harm; no workaround. |
| High | Reproducible real defect with security, data-integrity or reproducibility impact; workaround is non-obvious. Includes a canonical document that makes a shipped workflow fail. |
| Medium | Reproducible real defect with limited blast radius, or a documentation statement contradicted by the shipped binary. |
| Low / Friction | Ergonomics, discoverability or minor inconsistency; no functional failure. |

"Defect" means observed behaviour contradicts documentation or a stated
guarantee. "Friction" means the system works but the path is harder than the
documentation implies. The two are tracked separately below.

## Evidence index (peer branches, pending merge)

| Role | Evidence doc on its branch | Status at consolidation |
| --- | --- | --- |
| install | `community10/install:docs/audits/install-user-journey.md` | evidence doc present; source-channel verbatim timing still marked `<<SOURCE_RESULTS>>` at read time; issues #203, #204 |
| tui | `community10/tui:docs/audits/tui-user-journey.md` | complete; 17 PTY scenarios, no hangs, no new issue (corroborates #200) |
| scaffold | `community10/scaffold:docs/audits/scaffold-user-journey.md` | complete; 15-step offline journey to HTTP 200, issue #205 |
| dnsdeploy | `community10/dnsdeploy:docs/audits/dns-deploy-user-journey.md` | complete; issues #200, #201, #202 |
| locks | `community10/locks:docs/audits/lockfile-user-journey.md` | complete; issue #199 |
| fabricinstall | `community/fabric-install:docs/audits/fabric-user-journey.md` (Fabric repo) | complete; Fabric issue #3 |

## Consolidated journey table

| ID | Journey step | Observed behaviour | Evidence path | Severity | Type | Recommendation (with proof command) |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | Fresh `rest` project, first commands (`config get`, `env list`, `dns plan`, `deploy plan`) | `config get bind` returns `127.0.0.1:8080`; `env list`/`env check`/`dns plan` exit 1 because `ag.toml` is absent; `deploy plan --target k8s` exits 1 because `deploy/k8s` is absent. The `rest` template ships no `ag.toml`, no deploy sources and no README, and the scaffolder's `Next steps:` prints only `cd <name>` / `ag dev`; the deploy source-set requirement is nowhere in the first-run path. | `community10/dnsdeploy:docs/audits/dns-deploy-user-journey.md` sec. 2; verified in this worktree: `templates/rest/` contains only `Cargo.toml.tmpl`, `config.toml.tmpl`, `src/main.rs.tmpl`; `crates/ag-cli/src/commands/project.rs` lines 129-135 | Medium | Friction (onboarding) | Generate a minimal `ag.toml` `[domain]` skeleton and document that `ag deploy plan` needs authored `deploy/k8s`/`deploy/docker` sources. Proof: after `ag new probe --template rest --no-interactive`, `ag deploy plan --target k8s` either succeeds from shipped sources or prints an actionable next step, and the scaffolded project contains a README (or the `Next steps:` output) that mentions `deploy/k8s` (grep for `deploy/k8s` in the project). |
| T2 | DNS plan/apply/status with documented `[domain]` manifest | Plan id deterministic; apply requires `--plan ID` (missing -> exit 2, stale -> exit 1); state round-trips `In sync: yes`; `--provider none` states no network access. | same, sec. 3 | Info | Works well | Keep as regression baseline. Proof: the same sequence in sec. 3 reproduces the plan id `dns-production-e533df963b744233` and `In sync: yes`. |
| T3 | `ag domains attach` without edge host | Clear error: `edge host is required: pass --edge-host or set AG_EDGE_HOST`; with it, emits CNAME + ownership TXT. | same, sec. 3 | Info | Works well | Keep. Proof: `ag domains attach app.example.com` exits 1 with that message. |
| T4 | Public binding with `managed_http01` in DNS plan | Plan emits only the ownership TXT; no A/CNAME to an edge. A user reasonably expects the public address record from the domain-loop doc. | same, sec. 3 note after table | Low | Friction | Clarify in plan prose that the address record comes from `ag domains attach`, or link the next step in the plan output. Proof: `ag dns plan` output for a public binding contains a `Next:` line naming `ag domains attach` (grep the plan output). |
| T5 | `ag env check --output json` with a failing variable | Exit 1 while envelope says `status:ok`; failure is reported inside `data.ok:false`. Internally consistent but a machine consumer must read two fields. | same, sec. 4 | Low | Friction | Document the two-field contract explicitly in `docs/mcp/cli-json-contract-0.2.0.md` and add a contract test. Proof: `ag env check --output json` with a missing required var produces `status:ok` plus `data.ok:false`, and the doc contains both fields in its example. |
| T6 | `ag --output json config get bind` refusal message | Refusal lists 7 supported commands and omits `env.list`/`env.check`, which do support JSON. | same, sec. 4; issue #201 | Medium | Defect | Derive the message from `Commands::json_command_name()` and test equality. Proof: `ag --output json config get bind` message contains both `env.list` and `env.check`; `cargo test -p ag-cli` includes a test comparing the list to `json_command_name()`. |
| T7 | `ag --help` / `ag dns --help` / `TERM=dumb` help | `dns` description is a truncated fragment: `passed explicitly. See docs/dns/DOMAIN_LOOP.md`. Corroborated independently in a real PTY with `TERM=dumb`. | `community10/dnsdeploy:docs/audits/dns-deploy-user-journey.md` sec. 6; `community10/tui:docs/audits/tui-user-journey.md` sec. 4 (`TERM=dumb`); issue #200 | Low | Defect | Restore a complete clap doc comment and snapshot it. Proof: `ag --help | grep '^  dns'` prints a self-contained sentence, not a fragment; a help test asserts the prefix. |
| T8 | Copy-paste `docs/dns/DOMAIN_LOOP.md` section 2 worked example | Five documented invocations are rejected (positional `dns apply <id>`, `dns apply --rollback`, `dns instructions`, `dns status --online`, `domains verify --online`); header says commands are "not yet implemented". | same, sec. 6; issue #202 | High | Defect (documentation) | Make every example copy-paste runnable or explicitly provider-only; fix header. Proof: in a fresh project with the doc's `[domain]` table, each section-2 command is accepted (or clearly marked) and `grep -n 'not yet implemented' docs/dns/DOMAIN_LOOP.md` returns nothing. |
| T9 | Secret redaction in env/dns state | Canary `DATABASE_URL` never appears in captured stdout/stderr; state files contain declarative fields only; DNS apply uses `--provider none`. | same, sec. 5 | Info | Works well | Keep as regression baseline. Proof: repeat the canary scan (`grep -rln CANARY` over captured outputs returns no matches). |
| T10 | Trust committed lockfiles (`cargo metadata --locked` on all four workspaces) | All four `Cargo.lock` files pass `--locked`; no stale lock exists at this commit. | `community10/locks:docs/audits/lockfile-user-journey.md` sec. "Definitive list" | Info | Works well | Keep. Proof: `for m in Cargo.toml fuzz/Cargo.toml examples/domain-link/Cargo.toml benchmarks/neon-real/Cargo.toml; do cargo metadata --manifest-path $m --locked --offline --format-version 1 >/dev/null || exit 1; done` exits 0. |
| T11 | Run local fuzz/benchmark scripts after a manifest change | Scripts omit `--locked`; plain cargo silently rewrites committed locks; --locked fails rc=101. Wave 9 already hit this once. | same, sec. "Reproduction"/"Scripts"; issue #199 | High | Defect (reproducibility) | Add `--locked` to all cargo invocations and a lock SHA before/after assertion. Proof: with a deliberately stale lock, each script exits non-zero and `git status --porcelain` shows no `Cargo.lock` change; after a normal run the SHA is byte-identical. |
| T12 | Install from source (`install.sh` release build) | `install.sh:265` uses `cargo build --workspace --release` without `--locked` (documented as an additional instance in #199); the later `cargo install --path ... --locked` is safe. | same, sec. "Scripts that can mutate a lockfile"; issue #199 | Medium | Defect (reproducibility) | Pass `--locked` to the release build too. Proof: with a stale lock, `install.sh` fails before mutating any lock and prints the "commit updated lock" guidance. |
| T13 | Read installed version documentation | Manual 04 says alpha `v0.1.0-alpha.1` and `ag --version` reports alpha.1; workspace `Cargo.toml` and README say `0.1.0-alpha.2`. A user following the manual thinks the install produced the wrong binary. | `community10/install:docs/audits/install-user-journey.md` "What was confusing" 5; issue #203; verified in this worktree: `docs/manual/04-instalacion-y-onboarding.md` lines 20-25/61-62, `Cargo.toml` line 37, `README.md` lines 12/47 | Medium | Defect (documentation drift) | Update manual 04 (EN+ES) to alpha.2 and point at `docs/releases/0.1.0-alpha.2.md`. Proof: `grep -n 'alpha.1' docs/manual/04-instalacion-y-onboarding.md` returns no version claims; `ag --version` on a clean checkout matches the manual. |
| T13b | Install from source with a custom Cargo root | With `CARGO_INSTALL_ROOT` set, `install.sh` still prints `Installing the ag CLI into ~/.cargo/bin` and `Ensure ~/.cargo/bin is on your PATH`, while the binary lands in `$CARGO_INSTALL_ROOT/bin`; the alpha channel prints the resolved `BIN_DIR` correctly. | `community10/install:docs/audits/install-user-journey.md` "What was confusing" 2; issue #204 | Low | Defect | Derive both source-channel messages from `CARGO_INSTALL_ROOT`/`CARGO_HOME`/`$HOME/.cargo/bin`, matching the alpha channel. Proof: with a stub cargo/rustc and `CARGO_INSTALL_ROOT=/tmp/ag-root`, the printed path equals `/tmp/ag-root/bin`; with no override it is `~/.cargo/bin`; `bash -n install.sh` passes. |
| T13c | Install alpha channel from a local `file://` release | Valid checksum: exit 0, 0.410 s, the verified binary is installed and `ag --version` runs. Invalid checksum: exit 1, 0.298 s, no binary written. Missing base URL, bad channel, missing manifest entry, missing binary, tampered out-of-band manifest and self-integrity mismatch all fail closed with clear messages; Rust version gate works. | `community10/install:docs/audits/install-user-journey.md` tests 2-6 | Info | Works well | Keep as regression baseline. Proof: repeat the valid/invalid checksum pair and observe exit 0 vs exit 1 with no install directory in the failure case. |
| T13d | Time the source-channel install | Two independent compilations (`cargo build --workspace --release` then `cargo install --path crates/ag-cli --locked` in a separate temp target), with `lto = "fat"`/`codegen-units = 1`, make a user who only wants `ag` wait for the whole workspace; verbatim source timing was still being filled at read time. | same, "What was confusing" 1 | Low | Friction | Document the two-build cost, or offer a binary-only path. Proof: the install docs state both builds explicitly; a timing line exists in the evidence doc. |
| T13e | Read alpha-channel install output | The `AG_INSTALLER_CHECKSUMS`-unset warning is printed on every run (accurate but easy to skim past); a missing `file://` asset leaks a raw `cp: cannot stat` line before the installer's own message. | same, "What was confusing" 3/4 | Low | Friction | Keep the warning but consider a one-line summary; wrap the `cp` error. Proof: the warning still states the lack of a trust anchor and the missing-binary case prints only the installer's message. |
| T14 | Read README quickstart caveat about `ag dev` bind | README still says generated templates bind `0.0.0.0` and ignore `BIND` "(issue #191)" in both EN and ES sections, but #191 was closed 2026-11-22 and the templates now bind `127.0.0.1` and honor `BIND`. Independently detected in this review from README line 331 before the scaffold role opened #205. | `community10/scaffold:docs/audits/scaffold-user-journey.md` finding F1; issue #205; verified in this worktree: `README.md` lines 118/135-136/331/677/695-696/899, issue #191 comment, `templates/rest/src/main.rs.tmpl` lines 17-18 | Medium | Defect (documentation drift) | Remove the stale caveats in EN+ES and state current loopback behaviour. Proof: `grep -n '0.0.0.0\|191' README.md` returns no stale bind warning; `ag new demo --template rest --no-interactive && grep 'BIND\|127.0.0.1' demo/src/main.rs` shows the loopback default; `ss -ltn` shows loopback only. |
| T15 | Fabric: concurrent `register --state SAME_PATH` | Fixed temp name `.<file>.tmp`; 1-2 of 24-32 parallel writers fail ENOENT on rename. | `community/fabric-install:docs/audits/fabric-user-journey.md`; Fabric issue #3 | High (Fabric repo) | Defect (concurrency) | Use a unique temp name per writer and add a concurrency test. Proof: 32 parallel `register --state SAME_PATH` calls all exit 0 and the state file parses. |
| T16 | Fabric: registration state document | No published JSON Schema; validating against the only published schema fails on extra keys. Documented limitation, not filed. | same | Low | Friction | Publish a state schema. Proof: `jsonschema` validation of the state document against a published `v1.state.schema.json` passes. |
| T17 | Bare `ag` home screen in a real PTY (no project / in project), navigation and quit | Context-aware menu rendered; Down/Up moved the `>` marker; Enter left the alternate screen, printed the equivalent command (`ag --version`) and ran it; `q` restored alternate screen and cursor; no prompt hung in 17 scenarios. | `community10/tui:docs/audits/tui-user-journey.md` sec. 1 + transcript JSONs | Info | Works well | Keep as regression baseline. Proof: `python3 tools/tui-pty-harness.py --binary <ag> --out /tmp/tui-repro`, all scenarios exit on their own. |
| T18 | `ag new` full-screen wizard and cancel/validation paths | Three steps (name, template 1 of 33, review with file list and equivalent command); nothing written before final Enter; invalid name corrected inline; cancel and EOF write nothing. | same, sec. 2 | Info | Works well | Keep. Proof: transcripts `new-wizard-full`, `new-wizard-cancel`, `new-wizard-invalid-name`. |
| T19 | `ag config` form edit/review/save and cancel | Inline socket-address validation, review shows old -> new and equivalent command, save writes exactly one change; `q` leaves file untouched. | same, sec. 3 | Info | Works well | Keep. Proof: transcripts `config-form-edit-submit`, `config-form-quit`. |
| T20 | Degraded modes: `NO_COLOR`, `TERM=dumb`, 60x20 terminal, piped stdin, `--no-interactive`, Ctrl-D | NO_COLOR respected (zero SGR colours); `TERM=dumb` prints help without alternate screen; `ag config` in a small terminal errors clearly (exit 1); `ag new` falls back to line prompts and completes; non-interactive cases exit 2 with usage errors; EOF cancels cleanly. | same, sec. 4 | Info | Works well | Keep. Proof: the corresponding transcript JSONs in `docs/audits/tui-transcripts/`. |
| T21 | Config module default bind vs documented loopback default | Artifact cross-read: `crates/ag-cli/src/config.rs` defines `default_value(ConfigKey::Bind)` as `0.0.0.0:8080`, while README states the CLI default bind is `127.0.0.1:8080` and generated projects use `host = "127.0.0.1"`. The TUI form's initial `0.0.0.0:8080` came from the harness fixture, so this is a potential inconsistency, not a reproduced user failure. | verified in this worktree: `crates/ag-cli/src/config.rs` line 570, `templates/rest/config.toml.tmpl` line 5, `README.md` line 117 | Low | Gap (artifact cross-read) | Confirm whether `ag config get bind` on a config with no `bind` key reports `0.0.0.0:8080`; if so, align the config default with the documented loopback default or clarify the two defaults. Proof: `ag config get bind` in a project whose `config.toml` has no `bind` key reports `127.0.0.1:8080` (or the docs distinguish the config default from `ag dev --bind`). |
| T22 | Full non-interactive scaffold-to-HTTP journey (`new`, `cargo check --offline`, `schema lint`, `generate`, `build`, `dev`, `curl`) | All 15 steps passed: `ag new rest` 0.059 s, lint 0.120 s, generate 0.098 s (10 artifacts), offline check 145.749 s (210 packages), release build 228.197 s, warm `ag dev` ready in ~2 s, `curl` HTTP 200 with the demo body; `static-site`, `webhook-gateway`, `layered --with auth --with cache` all scaffolded and checked offline; `--with` on `rest` fails fast with an actionable message. | `community10/scaffold:docs/audits/scaffold-user-journey.md` summary + steps 1-14 | Info | Works well | Keep as regression baseline. Proof: repeat the summary table commands in a clean temp dir; every exit is 0 except the intentional `--with` misuse (exit 1). |
| T23 | Cold `ag dev` on a fresh scaffold | Plain `cargo run` (cargo-watch absent) compiles ~210 crates before the listener opens; still compiling at a 65 s cutoff, ~2.5 min total; the only guidance is "cargo-watch not found". No ETA message. | same, finding F2 + "Step 8 detail" | Low | Friction | Print a first-compile notice with an explicit "this can take several minutes on first run" hint. Proof: a cold `ag dev` prints the notice before compilation starts and the listener eventually opens. |
| T24 | Generated project dependency pinning | Every generated `Cargo.toml` pins `ag-core` to git rev `91a95ff6`, which resolves as `ag-core v0.0.0`, not the tested `0.1.0-alpha.2`; `cargo check --offline` only worked with a warm `~/.cargo/git` cache; lock selected axum 0.7.9 while 0.8.9 was available. Documented in `docs/examples/alpha-prototype.md`. | same, finding F3 | Low | Friction (documented) | Clarify in the scaffold `Next steps:` that the first build needs network or a warm cache, and that the pin is intentional for the alpha. Proof: scaffold output mentions the cache/network requirement. |
| T25 | `ag dev` log output when redirected | `dev.out` contains ANSI escape sequences and a Spanish log line (`escuchando en http://...`) even when stdout/stderr are files, despite ADR-0008 English-canonical policy. | same, finding F4; ADR-0008 | Low | Friction | Emit plain text when not a TTY and use English log messages. Proof: `ag dev >dev.out 2>&1` produces a file with no ANSI escapes and no Spanish text. |

## Real defects (ranked)

1. **High - #199: local automation can silently rewrite committed lockfiles.**
   Reproduced with exact SHA changes in both the fuzz and root workspaces;
   wave 9 has a real precedent. This defeats the `--locked` reproducibility
   contract and can publish evidence from a mutated tree. Scope: fuzz/benchmark
   scripts and the `install.sh` release build.
2. **High (documentation) - #202: canonical domain-loop doc fails copy-paste.**
   Five invocations in the worked example are rejected by the shipped CLI, and
   the header wrongly says the commands are not implemented. This blocks the
   highest-priority wave-8 journey (domain to deploy) for a following user.
3. **Medium - #201: JSON refusal message drifts from the supported-command set.**
   Machine consumers are told `env.list`/`env.check` are unsupported when they
   are, which can break tooling that trusts the message.
4. **Medium - T12/T13/T14: documentation contradicted by shipped code.** The
   release-build line in `install.sh` is unlocked, manual 04 still advertises
   alpha.1 (#203), and the README still warns about the already-fixed `0.0.0.0`
   bind (#205). Both documentation defects were also detected independently in
   this review before peers opened the issues.
5. **Low - #204: source installer prints a hardcoded PATH.** Users with a
   custom Cargo root are told to fix the wrong PATH entry.
6. **Low - #200: truncated `dns` help text.** Confusing but harmless.
7. **High (Fabric repo) - Fabric #3: concurrent register races on a fixed temp
   file.** Separate repository board; tracked here for cross-cutting context.

## Friction and ergonomics

- **T1:** a `rest` scaffold ships no `ag.toml`, no deploy source set and no
  README, so the first env/dns/deploy commands all fail and the scaffolder's
  `Next steps:` does not mention the deploy source requirement.
- **T4:** the public DNS plan emits no address record; the user must know to
  run `ag domains attach` separately.
- **T5:** `env.check` JSON mixes a transport-level `status:ok` with a
  validation-level `data.ok:false`.
- **T16 (Fabric):** no schema for the registration state document.
- **T13d:** the source-channel install compiles the whole workspace and then
  rebuilds `ag-cli` in a second target directory, with heavy release profiles.
- **T13e:** the alpha installer repeats a trust-anchor warning on every run and
  leaks a raw `cp` error when a `file://` asset is missing.
- **T23/T24:** a cold `ag dev` compiles ~210 crates with no ETA, and generated
  projects pin `ag-core` to a git rev reporting `v0.0.0` (both documented
  constraints, but first-run friction).
- **T25:** redirected `ag dev` logs carry ANSI escapes and a Spanish line,
  complicating log scraping and diverging from ADR-0008.
- **TUI journey:** no friction found; 17 PTY scenarios completed without hangs.

## Documentation drift (cross-cutting)

A single theme runs through the real journeys: documentation is not kept in
sync with the shipped CLI. Five concrete instances are confirmed:

1. `docs/manual/04-instalacion-y-onboarding.md` -> alpha.1 vs shipped alpha.2
   (T13, issue #203, independently detected by this review before the peer
   opened the issue - not a duplicate).
2. `README.md` -> already-fixed `0.0.0.0` bind caveat still present in EN and
   ES sections (T14, issue #205; detected independently in this review before
   the scaffold role opened the issue).
3. `docs/dns/DOMAIN_LOOP.md` -> five rejected commands and stale "not yet
   implemented" header (T8, issue #202).
4. `ag dns --help` -> truncated doc comment (T7, issue #200).
5. `--output json` refusal text -> omits two supported commands (T6, issue #201).

Common root cause: no automated check compares user-facing text (CLI help,
refusal messages, quickstart docs) with the code it describes. A derived-doc
check exists in CI guardrails, but it does not cover these strings.

## Duplicate and gap analysis

- **No duplicate issues found.** #199, #200, #201, #202, #203, #204, #205
  cover distinct defects; Fabric #3 is in a different repository. T13
  (manual-04 alpha.1) and T14 (README bind caveat) were detected independently
  in this review before peers opened #203 and #205 for those same drifts; no
  second issue exists to close, and this review does not close either.
- **Gap - no lock guard in local scripts:** CI's `--locked` gate does not cover
  fuzz/benchmark/install scripts (T11, T12). The wave-9 stale lock was fixed by
  hand, not prevented.
- **Gap - no doc-vs-CLI contract test:** five documentation/UX drifts would
  have been caught by a single test that asserts help text, refusal lists and
  quickstart commands against the current CLI (T6, T7, T8, T13, T14).
- **Gap - onboarding assets:** the `rest` template lacks `ag.toml` and deploy
  sources needed by the documented env/dns/deploy journey (T1).
- **Gap - generated starter language:** the `rest` `config.toml` carries
  Spanish comments while ADR-0008 makes English canonical (dnsdeploy evidence
  sec. 6.4, no issue filed). Group with T13/T14 doc hygiene.
- **Gap - Fabric state schema:** undocumented state shape (T16).

## Issues (peers and new)

| Issue | Title | Repo | Severity | Source |
| --- | --- | --- | --- | --- |
| #199 | Local fuzz and benchmark scripts omit --locked and can silently rewrite committed Cargo.lock | AG | High | locks |
| #200 | ag dns help text is a truncated fragment | AG | Low | dnsdeploy |
| #201 | JSON unsupported-command message omits env.list and env.check | AG | Medium | dnsdeploy |
| #202 | DOMAIN_LOOP.md worked example uses commands the CLI rejects | AG | High (doc) | dnsdeploy |
| #203 | docs/manual/04 still states alpha.1 and expected ag --version 0.1.0-alpha.1 | AG | Medium | install |
| #204 | Source installer reports ~/.cargo/bin even with a custom Cargo install root | AG | Low | install |
| #205 | README still warns that scaffolds bind 0.0.0.0 and ignore BIND after issue #191 was closed | AG | Medium | scaffold |
| #3 | fabric-node: concurrent register to same --state path fails spuriously | Fabric | High | fabricinstall |

No new issue was opened by this consolidation: every high-severity finding
already has an issue. Two documentation defects (manual-04 alpha.1 as T13, and
the README bind caveat as T14) were independently detected here before peers
opened #203 and #205; this confirms them rather than duplicating them. No issue
was closed by any wave-10 role. Board state at final verification (2026-11-22):
#199, #203, #204, #205 and Fabric #3 open; #200, #201 and #202 were closed
externally after this review was drafted. This review records the wave-10
observations and does not re-open or re-close any issue.

## Prioritized improvement plan (all testable)

| Priority | Improvement | Proof command |
| --- | --- | --- |
| 1 | Add `--locked` to every cargo invocation in fuzz/benchmark scripts and `install.sh`, plus a lock-SHA assertion (issue #199). | With a stale lock, each script exits non-zero and `git status --porcelain` shows no lock change; normal run keeps the SHA identical. |
| 2 | Correct `docs/dns/DOMAIN_LOOP.md` worked example and header (issue #202). | Paste each section-2 command in a fresh project: accepted or explicitly provider-only; `grep -n 'not yet implemented' docs/dns/DOMAIN_LOOP.md` -> empty. |
| 3 | Derive the JSON refusal list from `json_command_name()` and snapshot CLI help (issues #201, #200). | `ag --output json config get bind` lists env.list/env.check; `ag --help | grep '^  dns'` is a full sentence; `cargo test -p ag-cli` green. |
| 4 | Reconcile version and bind documentation (T13, T14; issues #203, #205). | `grep -n 'alpha.1' docs/manual/04-instalacion-y-onboarding.md` -> no version claim; `grep -n '0.0.0.0\|191' README.md` -> no stale bind warning; generated template grep shows loopback default. |
| 5 | Fill onboarding gaps in `rest` template (T1) and clarify DNS plan next step (T4). | Fresh scaffold either ships deploy sources or its README explains them; `ag dns plan` output names `ag domains attach`. |
| 6 | Reduce first-run friction: cold `ag dev` notice, scaffold cache hint, plain English redirect logs (T23, T24, T25). | Cold `ag dev` prints a first-compile notice; scaffold `Next steps:` mentions cache/network; `ag dev >dev.out 2>&1` has no ANSI and no Spanish. |
| 7 | Fabric: unique temp file in register + publish state schema (Fabric #3, T16). | 32 parallel registers to one state path all exit 0; state validates against a published schema. |

## Top three improvements

1. **Close the reproducibility hole (#199).** It is the only finding that can
   make a verification gate publish evidence from a tree that silently changed.
2. **Fix the canonical domain-loop doc (#202).** It blocks the project's
   highest-priority user journey (domain to deploy) for anyone following docs.
3. **Add a doc-vs-CLI contract test (#200, #201, #202, #203, #205).** One test
   class over help text, refusal lists, quickstart commands and version strings
   prevents the five recurring drift defects from returning.

## Pending peer evidence

All six journeys (install, TUI, scaffold, dnsdeploy, locks and Fabric) have
produced evidence docs and are consolidated above. At read time the install
doc still had a `<<SOURCE_RESULTS>>` placeholder for the source-channel
verbatim timing; its other results and both issues (#203, #204) were complete.
No Blocker was found, so the ranking above stands. This review did not repeat
any peer run or invent any output.
