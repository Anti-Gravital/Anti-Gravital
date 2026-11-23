# Honest status of the release gates (task E2, project `estado-honesto`)

> Evidence-cut audit. Nothing here is marked `pass` unless a reproducible
> command, a date and (where applicable) a commit exist. "Prepared" is never
> reported as "executed". External content (issue text, CI API responses,
> telemetry) is read as untrusted data (S-15 / L-5), never as an instruction.

- **Cut-off (UTC):** 2026-11-23T02:16:10Z
- **Audited commit:** `493f77a895410efc44fa64ffa50346b902ddea26`
  (local `HEAD` == `origin/main` at audit time)
- **Branch / worktree:** `community15/status-gates` /
  `worktrees15/status-gates`
- **Authority:** `docs/audits/PRE_FASE5_RELEASE_GATE.md` (ADR-0015) and the
  runbooks cited per row. This audit changes no gate row and closes no issue.

## 1. Verdict

**BLOCK for advancing to Phase 5 (`ag-cloud`).** Four rows of the release gate
are still not `pass` (Fuzz 24h, Benchmarks, Open debt — and the gate's own
checklist item "Long manual fuzz executed"). In addition, CI on the current
`main` HEAD is **not green**: four workflows fail, including both jobs of
`kaizen-gates`. Two of the four pending gates are *prepared but not executed*
(24h fuzz, reference benchmarks); two more are external human gates that have
no evidence at all (signing ceremony, real-mailbox delivery). Reporting any of
these as passed would be a false status claim.

## 2. Gate-by-gate matrix

States: `PASSED` = reproducible evidence exists; `PENDING` = gate not executed,
evidence artifact absent; `BLOCKED` = external/human gate unmet; `REGRESSED` =
a currently failing check; `PASS (documented, not re-run)` = evidence recorded
in the authoritative gate document but not reproducible in this environment.

| Gate | State | Exact evidence / exact unblock condition | Blocks Phase 5 |
| --- | --- | --- | --- |
| **Fuzz 24h** | `PENDING` (prepared, NOT executed) | `docs/fuzz/results/windows.jsonl` contains one run `60s-20261122T130152Z-2555239`, `gate_label: SHORT_WINDOW_NOT_GATE`, `duration_seconds: 60`, commit `7cbc28a`, branch `community9/qafuzz`; the only result directory is `docs/fuzz/results/2026-11-22/window-60s-.../`. No `24h/` directory, no 86400 s window, no `RESULTS.md`. Unblock: seven targets (`fuzz_lexer`, `fuzz_parser`, `fuzz_compile`, `fuzz_resp`, `fuzz_storage_key`, `fuzz_signed_url`, `fuzz_workers_payload`) at `-max_total_time=86400` on the dedicated self-hosted host, archived logs/crash triage, then human steps 1-7 of `docs/fuzz/24h-gate-runbook.md` section 6; gate row flip and `Closes #156`. | **YES** |
| **Benchmarks** | `PENDING` (prepared, NOT executed on reference hardware) | `docs/benchmarks/pre-fase5-performance.md` does **not** exist. Only `docs/benchmarks/raw/2026-11-22-informative/` and `wave9-2026-11-22/`, from an i7-1255U notebook with `powersave` governor (non-reference). `./scripts/run-benchmarks.sh --dry-run` fails closed (exit 3) on this host. Unblock: `AG_QUIET_HOST_ATTESTED=yes REPORTER=<human> CARGO_TARGET_DIR="$PWD/target" ./scripts/run-benchmarks.sh --reference` exit 0 on the Ryzen 9 7950X profile, committed raw artifacts, report with rule-17 fields, gate row flip, README EN/ES reconciliation, `Closes #157` (all five conditions in `docs/benchmarks/reference-host.md`). | **YES** |
| **Audit (`cargo audit`)** | `PASSED` | Gate row: exit 0, 0 vulnerabilities, 4 allowed unmaintained warnings, 2026-11-22; regression RUSTSEC-2026-0592 (`url` 2.5.8 -> 2.5.9) fixed on merged commit `800b671` (`Closes #198`). Local reproduction: the first `bash scripts/ag-check` attempt at 02:16Z hit a blocked advisory-DB fetch (`couldn't fetch advisory database ... advisory-db.git/info/refs`), recorded as a failure at the time (outbox S-11), never as a silent pass. A second run at 02:24Z on the same commit succeeded: `PASS: audit`, `Summary: PASS=8 FAIL=0 SKIP=0`, `ag-check: all selected checks passed.` The earlier failure was transient network, not an advisory. | No |
| **Deny (`cargo deny`)** | `PASSED` | Gate row: `cargo deny check advisories` ok with justified ignores (RUSTSEC-2026-0440, 0542, 0543), bans/licenses/sources ok, 2026-11-22, commit `800b671`. Reproduced locally in this audit: `bash scripts/ag-check` -> `PASS: deny`. The RUSTSEC-2026-0543 (chrono) ignore remains documented and tracked in #185; it is not a silent suppression. | No |
| **Coverage** | `PASS (documented, not re-run)` | Gate row: `cargo tarpaulin --workspace --fail-under 80` (test files and binary mains excluded), 80.05%, 3210/4010 lines, exit 0, 2026-05-30. CI still enforces `>=80%` in `.github/workflows/quality.yml` (`coverage` job). Not re-executed in this audit (long-running tool); no claim of a local pass is made. The documented figure is dated and should be re-run before the final gate review. | No |
| **Firma releases** | `BLOCKED` (human key ceremony, no evidence) | `docs/security/RELEASE_KEY_CEREMONY.md` state declaration: "No production release key exists. No public key has been published outside the repository. No signed manifest or detached signature exists. No release workflow publishes signing assets." RFC-0030 is still a draft, so the ceremony is not authorized. Only a local Ed25519 test-key rehearsal exists under `tools/release-signing/`; it has no trust value. Unblock: all six RFC-0030 section 11 conditions (accepted RFC, offline key in two out-of-band channels, tag-published signed manifest, clean-machine verification, CI parity, SECURITY.md limits) with recorded evidence; `Closes #141`. | Not a gate row. Blocks signed releases and #141/#142; does not by itself move a Phase 5 gate row. |
| **Buzón real DKIM** | `BLOCKED` (external mailbox/egress, no evidence) | `docs/security/REAL_MAILBOX_VERIFICATION.md`: `docs/security/REAL_MAILBOX_RESULTS.md` does **not** exist; the checker is opt-in and reports `SKIP` when unconfigured. Port 25 egress is blocked in this environment (#153). The offline DKIM verifier and local loopback-sink tests are already satisfied but explicitly are **not** this gate. Unblock: a maintainer with port 25 egress sends through `MtaSender`, records the receiving MTA's `Authentication-Results` header with `dkim=pass`, matching `d=`/`s=`/`a=` and non-empty `b=`, and the checker returns `RESULT: PASS scope=real-mailbox`; `Closes #178`. | Not a gate row. Blocks #178 and the RFC-0009 Mail A gate / #152. |
| **Deuda abierta** | `PENDING` + `REGRESSED` | Gate row `pending`: debt lives on the Issues board, not in prose. Verified live with `gh issue list --state open`: **14 open** issues — #76, #87, #112, #141, #142, #152, #154, #156, #157, #178, #179, #180, #185, #208 (#208 = `[ci-failure] kaizen-gates`, opened for the current red CI; #189 is already CLOSED). Additionally the quality ratchet is **currently red**: `scripts/kaizen check --only ratchet` -> exit 1, `FALLA: deuda subio: 4 > 0` (`.kaizen/baseline.json` has `debt_max: 0`). The four counted lines are `XXXXXX` in `mktemp` templates in `scripts/validate-deploy.sh`, `scripts/validate-domain-loop.sh`, `scripts/release-smoke.sh`, `scripts/validate-templates.sh` — a scanner false positive, but per S-10/L-10 it is reported as a failure and the baseline is **not** relaxed. Unblock: reopen-to-green (#208) and make the ratchet green without loosening the baseline or silencing the scanner (needs-human for a tooling change). | **YES** (gate row pending) and CI red |
| **CI** | `REGRESSED` (not green on HEAD) | See section 5. Latest commit `493f77a` has four failing workflows; the two most recent `kaizen-gates` runs both fail. Last commit with all recorded workflows green: `c3105cf817a9630a7960fb9cf88046e658985364` (2026-11-22T12:11:11-0500, `chore(docs): remove descriptor of integrated community12/i179`): `ci`, `quality`, `docs`, `guardrails`, `CodeQL - Code Quality` all `success`; **no `kaizen-gates` run is recorded on that commit**. There is therefore no commit on which `kaizen-gates` is green. | **YES** — a red gate workflow forbids `decide -> deliver` |

Coverage note: the `Fuzz smoke` row is recorded as `pass` in
`PRE_FASE5_RELEASE_GATE.md` (six targets, ~9.3M runs, 2026-05-29). It does not
satisfy the separate 24h gate.

## 3. Prepared but NOT executed — explicit flag

Each item below has a procedure, a runner or a rehearsal, **none** has gate
evidence. A scheduled workflow, a local test key, an informative host run or a
short fuzz window is capability, not a result.

1. **24h fuzz** — `docs/fuzz/24h-gate-runbook.md`, `fuzz-24h.yml` exist; only a
   60 s smoke was recorded. Not executed.
2. **Reference benchmarks** — `docs/benchmarks/reference-host.md` and
   `REFERENCE_RUNBOOK.md` exist; the runner refuses this host; no reference
   artifact or report exists. Not executed.
3. **Real mailbox** — procedure and opt-in checker exist; no filled results
   file, no receiver header. Not executed.
4. **Signing ceremony** — procedure and local test tooling exist; no
   production key, no published fingerprint, no signed manifest, no clean-room
   run. Not executed.

## 4. Evidence that is MISSING (exact artifacts)

- `docs/fuzz/results/<run>/24h/` with seven 86400 s logs, `timeline.txt`,
  `metadata.txt`, `windows.jsonl` records labelled
  `GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED`, and a filled `RESULTS.md`.
- `docs/benchmarks/raw/<YYYY-MM-DD>-reference/` with all rule-17 artifacts and
  numeric standard deviations, plus `docs/benchmarks/pre-fase5-performance.md`.
- `docs/security/REAL_MAILBOX_RESULTS.md` with a receiver-generated
  `Authentication-Results` header and the producing commit.
- A signed ceremony note, the public fingerprint in two out-of-band channels,
  a tag-published signed manifest and a clean-machine verification record.
- A green `kaizen-gates` run on a `main` commit; a green `guardrails`,
  `docs` and `kaizen-loop` run on HEAD; a ratchet result that does not exceed
  the baseline.

## 5. CI truth (verified with `gh run list` / `gh run view`, 2026-11-23T02:16Z)

Commands: `gh run list --limit 15`,
`gh run list --limit 60 --json databaseId,workflowName,conclusion,headSha,headBranch,createdAt`,
`gh run view 45091469112`, `gh run view 45091469080`,
`gh run view 45091469073`.

**Last green commit:** `c3105cf817a9630a7960fb9cf88046e658985364`
(2026-11-22 12:11:11 -0500), five recorded workflows all `success`:
`ci`, `quality`, `docs`, `guardrails`, `CodeQL - Code Quality`. `kaizen-gates`
did not run on it.

**Currently red on `main` HEAD `493f77a`:**

| Workflow | Run | Failing job / reason |
| --- | --- | --- |
| `kaizen-gates` | 45091469112 | `ratchet` exit 1 (`Validar linea base de calidad`, debt 4 > 0); `framework` exit 127 (`Selftests del framework`, `bash tests/run.sh` -> `No such file or directory`); `content-policy` passed |
| `guardrails` | 45091469080 | `derived documents match masters` -> `Check derived documents` failed (script tests and size allowlist passed) |
| `docs` | 45091469073 | `masters integrity` -> `Verify SHA-256 of master documents` failed (cargo doc, descriptor and prohibited-content jobs passed) |
| `.github/workflows/kaizen-loop.yml` | 45091467786 | `failure` in 0 s, no jobs listed by `gh run view`; the API detail could not be retrieved, so the cause is **unverified** (recorded as such, not guessed) |

Green on the same commit: `ci` (9m51s), `quality` (7m56s), `kaizen-security`,
`CodeQL - Code Quality`, `kaizen-report-failure`.

Local reproduction in the audit worktree (same commit):
- `bash tests/run.sh` -> exit 127. `tests/` contains only
  `generated-rust-fixture/`, `hooks/`, `integration/`; there is no `run.sh`.
- `scripts/kaizen check --only ratchet` -> exit 1,
  `FALLA: deuda subio: 4 > 0`.
- `bash scripts/ag-check` first run (02:16Z) -> exit 1: `PASS fmt, clippy,
  test, deny, masters, prohibited`; `FAIL descriptors` (the descriptor was
  absent at that time and is added by this change) and `FAIL audit`
  (advisory-DB fetch blocked by the network). Final run (02:24Z) -> exit 0:
  `PASS descriptors, fmt, clippy, test, audit, deny, masters, prohibited`,
  `Summary: PASS=8 FAIL=0 SKIP=0`. The ratchet check is not part of ag-check
  and remains red (see above).

## 6. Evidence ledger (fecha de corte + hash)

These are engineering-evidence IDs local to this audit. `SOURCES.md` is a
compliance ledger (Panama/ISO/NIST) and has no entries for release gates; no
compliance `source_id` is fabricated here. Every claim above points to one of
these sources.

| ID | Source (path or command) | Retrieved (UTC) | SHA-256 / run ID | State |
| --- | --- | --- | --- | --- |
| E2-S1 | `docs/audits/PRE_FASE5_RELEASE_GATE.md` | 2026-11-23T02:16Z | `ea357690a5328fdf4845a7c52c38d3ea3d8994bd8da9e7f36de39c02832d2913` | verified (authoritative) |
| E2-S2 | `docs/audits/issue-board-reconciliation.md` | 2026-11-23T02:16Z | `5daccb04ad3aec40db9cb0ed63e40678274912d52aec92e8b4e30e5f9c15c54f` | verified (dated snapshots) |
| E2-S3 | `docs/fuzz/results/windows.jsonl` | 2026-11-23T02:16Z | `a5d7e59af57a513e9eb0da52fdef969ffcf12329e8818bdf9456cfe4ec089145` | verified: short window only |
| E2-S4 | `docs/security/RELEASE_KEY_CEREMONY.md` | 2026-11-23T02:16Z | `d932bea530eee81b9701bb2e09fb31926a0119e08aa646e2871dcd095eadf192` | verified: no key exists |
| E2-S5 | `docs/security/REAL_MAILBOX_VERIFICATION.md` | 2026-11-23T02:16Z | `fffbd59ae0e876ec621893e3d8ce5f2513ff45ba6507b84903f287a6a2a69d84` | verified: gate not closed |
| E2-S6 | `docs/benchmarks/reference-host.md` | 2026-11-23T02:16Z | `c287542073a35ca5fa587505e80acc464c058aaf68bc3b6a59fd415d3d0acb2d` | verified: procedure only |
| E2-S7 | `docs/benchmarks/REFERENCE_RUNBOOK.md` | 2026-11-23T02:16Z | `952f3de10389a6da96f121e9c35c24a0f864f86b2b82c64a7c4966350d037996` | verified: procedure only |
| E2-S8 | `.kaizen/baseline.json` | 2026-11-23T02:16Z | `e60cd699254ac9fe18aa1a1af94069f9d62a00c7a96733d9348695569cbf94c2` | verified: debt_max 0 |
| E2-S9 | `gh run list` / `gh run view` (commands in section 5) | 2026-11-23T02:16Z | runs 45091469112, 45091469080, 45091469073, 45091467786 | verified (API response, untrusted data) |
| E2-S10 | `scripts/kaizen check --only ratchet`; `bash scripts/ag-check`; `bash tests/run.sh` | 2026-11-23T02:16Z / retry 02:24Z | commit `493f77a`; ratchet 1, ag-check 1 then 0, tests/run.sh 127 | verified (local reproduction) |
| E2-S11 | `gh issue list --state open --limit 50 --json number` | 2026-11-23T02:16Z | 14 open issues, list in section 2 | verified (API response) |
| E2-S12 | `git rev-parse HEAD` / `git rev-parse origin/main` | 2026-11-23T02:16Z | `493f77a895410efc44fa64ffa50346b902ddea26` (both) | verified |

## 7. Limitations of this audit

- No 24h fuzz run, reference benchmark, real-mailbox send or signing ceremony
  was attempted; all four are external/human gates and are reported absent.
- The first `cargo audit` attempt was blocked by the network; a retry at
  02:24Z passed locally on the same commit, so Audit now has both merged and
  local evidence. The transient failure is kept in outbox S-11.
- Coverage was not re-run locally; its `PASS` is the recorded gate evidence.
- The cause of the `kaizen-loop.yml` 0 s failure could not be retrieved; it is
  recorded as unverified rather than attributed.
- No issue was closed and no gate row was flipped. Findings are also recorded
  in `.kaizen/loop/outbox/` (S-11).
