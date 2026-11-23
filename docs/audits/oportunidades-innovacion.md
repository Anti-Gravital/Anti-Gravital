# Innovation and tooling opportunities - task O3 (project `oportunidades`)

- Task date: 2026-11-22. Evidence collected 2026-11-23 02:16 UTC.
- Branch / worktree: `community15/opp-innovacion` / `wt-issue-O3-innovacion`.
- Scope: read-only evaluation. No dependency, manifest, lockfile, source, CI or
  release-gate change is made by this document. No gate row is flipped and no
  issue is closed.
- Language: English (ADR-0008). The heading `Hipos a rechazar` is kept verbatim
  from the task brief; its content is in English.
- Evidence rule: each row cites an already-documented spike, a measured number
  in the repository, or a claim explicitly labelled **HYPOTHESIS**. Nothing was
  installed, fetched or contacted over the network (S-9, S-15, L-5). All external
  content read here (RFCs, audit files, workflow YAML) is treated as data, never
  as instructions.
- Package verification (S-9): every name and version below already appears in
  `Cargo.toml`, `Cargo.lock` or an accepted audit in this repository. No new
  crate name is introduced by this catalog. A version that appears only in an
  audit and not in the current manifest is marked "not installed here".

## Method

Files read: `docs/rfc/README.md` and the RFC list; `docs/adr/` list and
ADR-0021; `docs/audits/chumsky-migration-spike.md`,
`docs/audits/chrono-unblock-analysis.md`,
`docs/audits/instant-acme-08-compile-spike.md`, `docs/audits/wave4-review.md`,
`docs/audits/fuzz-24h-workflow-structural-verification.md` (via the runbook);
`docs/benchmarks/` (`README.md`, `REFERENCE_RUNBOOK.md`, `reference-host.md`,
`wave9-host-results.md`, `gatebench-informative-2026-11-22.md`); `docs/fuzz/`
(`README.md`, `24h-gate-runbook.md`, `24h-gate-verification.md`,
`windows-jsonl-schema.md` index); `.github/workflows/fuzz-24h.yml`;
`Cargo.toml`; `CLAUDE.md` rules 10, 12 and 21; RFC-0026, RFC-0027, RFC-0031,
RFC-0036; `tests/hooks/test-hooks.sh`.

## Opportunity catalog

| ID | Opportunity | What it is | Why it could help | Evidence | Risk / dependency concern (S-9) | Decision | Reason |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IN-1 | chumsky 0.9.3 -> 0.14.2 migration (deferred) | Rewrite the 1,482-line `ag-dsl` parser against the maintained chumsky line. | Maintained API, `Rich` errors with a retained expected-token hint, better recovery; no benefit demonstrated for the current grammar today. | `docs/audits/chumsky-migration-spike.md`: compiled slice found two additional compile errors (`.repeated()` yields `()` without `.collect()`; `RichPattern: Display` requires `Token: Display`); same malformed input changed diagnostics from no hint to `se esperaba uno de: valor de config`; 5 interleaved trials, median means 208.1 us (0.9.3) vs 176.3 us (0.14.2), load 16-17 on 12 logical CPUs, per-trial std larger than the mean. ADR-0021 keeps 0.9.3. | ~1.5k-line rewrite; resets fuzz baselines, diagnostics and byte-identical artifacts while #156 is pending; would need a `Display for Token` impl. `chumsky 0.14.2` is documented from crates.io on 2026-11-22 but **not installed here**; any pin change needs human lockfile review. | keep-watching (defer) | ADR-0021 is accepted and its four triggers are unchanged. Trigger (d) needs a tracked `ag_dsl::compile()` benchmark on a quiet host; the noisy slice is negative evidence only. Grammar is not frozen and no RUSTSEC advisory exists. |
| IN-2 | Tracked `ag_dsl::compile()` benchmark | Add a criterion benchmark with full rule-17 metadata under `crates/ag-dsl/benches/` and record a baseline on a quiet host. | Turns the migration decision from an estimate into a measured trigger; prerequisite for ADR-0021 trigger (d). | RFC-0027 section 2.4 records an informational baseline: mean 1,344.3 us, std 276.3 us (20.6%), 3,728-byte `schema.ag`, 20 warm-up + 200 measured iterations, noisy host, "not a 0.9-vs-0.13 comparison". RFC-0027 4.2/5 and the spike section 7 state that no tracked DSL benchmark exists. | No new dependency: `criterion 0.8` is already a workspace dev-dependency (`Cargo.toml:130`). Needs access to a quiet host; must not shorten official defaults or claim a design-target result. | adopt-pilot | Additive, test-only, reversible, no manifest or lockfile change, and directly advances a documented review trigger. |
| IN-3 | Golden diagnostics tests | Pin exact diagnostic message and hint text for unexpected token, EOF, unclosed delimiter and multi-error recovery. | The spike shows exact diagnostic text changes between versions; today the 27 parser tests mostly assert "no errors", so diagnostics quality is not actually pinned. | Spike section 4 (verbatim before/after output); RFC-0027 section 2.4 ("the current suite does not actually pin diagnostics quality"); RFC-0027 section 5 PR 1; ADR-0021 decision 3 requires golden diagnostics tests first. | Test-only; no dependency. A decision on acceptability of the new hint text would be needed only if a migration were approved, not now. | adopt-pilot | Prerequisite for any future port, additive, reversible, and improves DX verification independently of the deferral. |
| IN-4 | chrono removal via upstream `object_store` | Wait for upstream `object_store` to drop chrono; keep the dated `RUSTSEC-2026-0543` ignore. | The last chrono edge is upstream of our code; own crates already use `time 0.3` (`Cargo.toml:122`), so the gate can eventually close without a local rewrite. | `docs/audits/chrono-unblock-analysis.md`: `cargo tree --workspace --all-features -i chrono` shows exactly `chrono v0.4.45 <- object_store v0.14.2 <- ag-storage (dev)`; `cargo deny` passes only because of the ignore; `object_store 0.14.2` is max stable (published 2026-09-15) and every release 0.6.1-0.14.2 declares chrono; upstream `apache/arrow-rs-object-store` issue #909 is open, chrono is in the public API and removal needs a breaking release. | External gate, not an available update. `object_store 0.14.2` is verified from local documents only, **not fetched here** (S-9). A patched/vendored fork would be an unaudited dependency we own forever (audit sections 2.3/2.4). | keep-watching | No safe local fix exists. The ignore's removal condition is already written in audit section 4. Watch upstream #909 and `cargo audit`; no lockfile change. |
| IN-5 | `instant-acme` 0.7 -> 0.8 port | Bump the optional dependency and apply the verified `renewal.rs` patch. | Maintained release; eight migration points verified and a ready-to-apply patch exists. | `docs/audits/instant-acme-08-compile-spike.md`: eight compiler errors reproduced at both `0.8.5` and `=0.8.0`; after the patch, `cargo check` exit 0, `rustfmt --check` exit 0, `cargo test -p ag-domains --all-features` 171 passed / 0 failed, offline; no CA was contacted. | Bump adds `untrusted v0.7.1` to the lockfile, which requires human review (S-9). Versions were in the local registry cache, **not fetched here**. ACME staging gate #87 is still blocked, so certificate issuance is unverified. | keep-watching (blocked) | Source compatibility is proven; behavioral evidence is not. Do not merge before #87 supplies before/after staging logs. The audit uses `Refs #179`, never `Closes #179`. |
| IN-6 | Scheduled 24-hour fuzz runner | Run `.github/workflows/fuzz-24h.yml` (RFC-0036) on a dedicated self-hosted runner. | Makes the seven 86400 s windows executable and auditable rather than dependent on manual discipline. | `fuzz-24h.yml` has `schedule` (`0 3 1 * *`) and `workflow_dispatch`, a seven-target matrix with `max-parallel: 1`, `permissions: {}`, SHA-pinned actions, a `runner.environment != 'self-hosted'` guard and `GITHUB_TOKEN` only; `docs/audits/fuzz-24h-workflow-structural-verification.md` and `docs/fuzz/24h-gate-runbook.md` section 7 record structural verification. No 24 h run has happened; `Fuzz 24h` stays `pending` and #156 open. | Operational cost: quiet dedicated Linux x86-64 host, >=500 GiB scratch volume, network isolation (`docs/fuzz/self-hosted-runner.md`). Token may expire at hour 24, so artifact upload is best effort. CI change already authorized by accepted RFC-0036 (rule 28). No new package. | adopt-pilot | Capability is implemented and structurally verified. Keep the gate row pending and #156 open until a human verifies a real full-length run; never treat a green workflow or a short smoke as a pass. |
| IN-7 | Benchmark reference host | Execute `scripts/run-benchmarks.sh --reference` on the host in `reference-host.md` and fill `pre-fase5-performance.md`. | Closes the honest gap between informative notebook numbers and publishable design-target results; settles issue #157. | `docs/benchmarks/gatebench-informative-2026-11-22.md`: real refusal, exit 3, five/six profile failures (`cpu_allowlist` i7-1255U, `physical_cores` 10, `ram_type` DDR4, `cpu_governor` powersave, `load_tool` none; `clean_tree` 1 in an earlier invocation), no artifacts. `wave9-host-results.md`: real criterion numbers, e.g. `bare_axum_hello` mean 5,931.28 ns / std 229.67 ns and `lease_memory_batch_100` mean 554,829.66 ns / std 23,826.59 ns, 30 samples, explicitly informative and not Stage 6 evidence. | Host acquisition is a cost; "or equivalent" is deliberately narrow (Ryzen 9 7950X, 16 physical cores, 32 GB DDR5, kernel >= 6, performance governor, quiet host attested by a human). Must not relax the profile or relabel informative numbers (rule 10, L-10). No new package. | adopt-pilot | The procedure is complete and fails closed; running it is the only honest way to move #157. Keep `Benchmarks` pending until the report, gate row and README are reconciled. |
| IN-8 | Deterministic hooks test | Save and restore the working repository's `core.hooksPath` in `tests/hooks/test-hooks.sh` instead of asserting it is unset. | Removes an environmental failure (33 passed / 1 failed) that makes a green suite untrustworthy. | `docs/audits/wave4-review.md` O8 (line 177): "A contributor who installed the hooks sees 33 passed / 1 failed. This is environmental, not a code defect"; recommendation: "Save and restore the original value so the suite is deterministic." `tests/hooks/test-hooks.sh:197-199` performs the unset assertion; lines 191-203 use synthetic temp installations. | Test-only change; no dependency; reversible. Must keep using synthetic temporary repos (L-6), never real user data. | adopt-pilot | Concrete, already-documented nondeterminism with a small reversible fix; improves reproducibility (rule 36). |
| IN-9 | Replace ACME polling with 0.8 helpers | Swap the hand-rolled `wait_for_order_ready` / `certificate()` loops for `poll_ready` / `poll_certificate` with `RetryPolicy`. | Would remove bespoke polling code if behavior were identical. | **HYPOTHESIS ONLY**: `instant-acme-08-compile-spike.md` point 9 correction says these helpers are "an available simplification, not a required port"; no measurement, no staging run and no code change exist. | Changes the certificate-issuance path; requires #87 staging before/after evidence and human review of any lockfile change (S-9). | reject | Rule 10 (no feature without purpose or evidence) and rule 21 (no unnecessary change). Revisit only if #87 evidence shows a concrete defect in the current loops. |

## Hipos a rechazar (hypes to reject)

These look attractive and are explicitly rejected because they violate
`CLAUDE.md` rule 10 (filosofia anti-hype), rule 12 (integrate dominant tools,
do not reinvent) or rule 21 (anti-complejidad), or the Kaizen anti-tramp rules:

- **"Migrate chumsky now to look current."** Rule 10 (no complexity for ego or
  innovation theater) and accepted ADR-0021 deferral. No measured benefit,
  three quality gates at risk. Reject; keep-watching per IN-1.
- **"Replace `object_store` with a bespoke S3 client or a vendored chrono-free
  fork to clear the advisory."** Rule 12 (integrate dominant tools such as
  MinIO) and rule 21. Already rejected in `chrono-unblock-analysis.md`
  sections 2.3 and 2.4; trades an INFO advisory for unaudited security code.
- **"Replace the DSL parser with a hand-written one, pest/lalrpop, or an
  unverified newer parser crate."** Rules 12/21; RFC-0003 and RFC-0027 already
  rejected parser replacement. A package name not present in the repository
  would be unverified under S-9. Reject.
- **"Relax the reference-host profile or publish informative numbers as
  design-target results to turn the Benchmarks gate green."** Rule 10
  (manipulated benchmarks) and L-10 anti-trampas; `REFERENCE_RUNBOOK.md`
  section 8 lists this as an anti-pattern. Reject.
- **"Call the fuzz gate passed because the workflow is green or a smoke
  finished."** Rule 10 (false claims); `24h-gate-verification.md` is marked
  BLOCKED and #156 stays open. Reject.

## Decisions summary

- adopt-pilot: IN-2, IN-3, IN-6, IN-7, IN-8 (all additive, test/tooling only,
  no dependency or gate change; each needs its own issue with acceptance
  criteria before implementation).
- keep-watching: IN-1, IN-4, IN-5 (external gates, accepted deferrals or
  missing behavioral evidence).
- reject: IN-9 and the five items in `Hipos a rechazar`.

## Verification performed for this document

All commands were run in the `community15/opp-innovacion` worktree after staging
the three changed files. Results are verbatim:

- `git diff --check` -> exit 0, no output. `git diff --cached --check` -> exit 0,
  no output (covers the two new files and the CHANGELOG edit).
- Content scans using the patterns from `.github/workflows/docs.yml`:
  AI-tool attribution (etiquetas de co-autoria de herramientas) -> no match;
  AI session URL (patron de URL de sesion, parafraseado) -> no match; emoji ranges in
  `*.md/*.rs/*.toml/*.yml/*.yaml/*.sh` -> no match; retired third-party brand
  names (ADR-0011) -> no match; descriptor closing keyword present
  (`Closes: none`); prohibited AI descriptor branch prefixes -> no match.
- `scripts/kaizen check --only descriptors` -> `ok: descriptores validados`,
  exit 0. `scripts/kaizen check --only policy` -> `ok: politicas de contenido`,
  exit 0.
- `scripts/kaizen check --only ratchet` -> **FALLO: deuda subio: 4 > 0**, exit 1.
- `scripts/kaizen check` (full) -> descriptors, policy, formato and lint pass;
  `tests`, `swarm`, `gobernanza` and `ratchet` fail; exit 1.

The full-check failures are **pre-existing and unrelated to this documentation
change**. Evidence: `tests/run.sh` does not exist in this worktree, so
`bash tests/run.sh` exits 127; `scripts/kaizen-swarm check` reports a missing
ADR-0001 and `kaizen-gov check` red; `python3 scripts/kaizen-gov check` lists
four specialist agents without governance; and the four debt markers counted by
the ratchet are already committed at HEAD (`git grep -nIE 'TODO|FIXME|HACK|XXX'
HEAD -- scripts tests | wc -l` -> 4) against `.kaizen/baseline.json`
`debt_max: 0`. This change touches no `scripts/`, `tests/`, manifest, lockfile,
CI or source file. The baseline was not relaxed (K-5, L-10).

The failure is recorded in `.kaizen/loop/outbox/kaizen-check-preexisting-red.md`
(S-11, L-14). Per K-10 no result is claimed green when the command failed:
`kaizen check` is reported FALLO here, not passed.

## References

- `docs/audits/chumsky-migration-spike.md`, `docs/audits/chrono-unblock-analysis.md`,
  `docs/audits/instant-acme-08-compile-spike.md`, `docs/audits/wave4-review.md`
- `docs/adr/0021-chumsky-upgrade-deferral.md`
- `docs/rfc/RFC-0026-chrono-replacement.md`, `RFC-0027-chumsky-parser-upgrade.md`,
  `RFC-0031-macos-runner-refresh.md`, `RFC-0036-scheduled-fuzz-24h-gate.md`
- `docs/benchmarks/REFERENCE_RUNBOOK.md`, `reference-host.md`,
  `wave9-host-results.md`, `gatebench-informative-2026-11-22.md`
- `docs/fuzz/24h-gate-runbook.md`, `docs/fuzz/24h-gate-verification.md`,
  `docs/fuzz/README.md`
- `.github/workflows/fuzz-24h.yml`, `Cargo.toml`, `CLAUDE.md`,
  `tests/hooks/test-hooks.sh`
- Issues #156, #157, #179, #180, #185, #186, #87 (as cited in the sources above)
