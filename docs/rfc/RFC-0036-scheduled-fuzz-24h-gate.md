# RFC-0036 - Scheduled fuzz 24h gate

- Status: accepted. Comment period waived by recorded BDFL decision
  (2026-11-22). The waiver is recorded in section 10.
- Decision date: 2026-11-22 (wave 11 gate closure)
- Author: Gravital Labs community; decision recorded by the BDFL
- Target phase: additive CI; closes no phase by itself. Supports issue #156.
- Affected files: `.github/workflows/fuzz-24h.yml` (new),
  `docs/fuzz/self-hosted-runner.md` (new), `scripts/run-fuzz-window.sh`
  (backward-compatible `FUZZ_TARGETS` subset), `docs/fuzz/windows-jsonl-schema.md`,
  `docs/fuzz/README.md`, `docs/INDEX.md`, `docs/rfc/README.md`, `CHANGELOG.md`
- Predecessors: RFC-0022 (maintainability guardrails), RFC-0028 (opt-in CI
  verification), RFC-0031 (runner refresh), `docs/fuzz/24h-gate-verification.md`,
  `docs/fuzz/scheduled-24h-gate-draft.md`, `docs/fuzz/windows-jsonl-schema.md`

## 1. Title

Enable an executable, scheduled 24-hour fuzz gate on a dedicated self-hosted
runner, with least privilege and reproducible evidence, without claiming a pass
that has not been observed.

## 2. Motivation

Issue #156 is still open. The verification package
(`docs/fuzz/24h-gate-verification.md`) defines seven targets, 86400 s each, and
explicitly states that no 24-hour run has happened. The only automation so far
is `scripts/run-fuzz-window.sh` (short-window evidence) and a design note that
says **DRAFT / NOT ENABLED** (`docs/fuzz/scheduled-24h-gate-draft.md`).

Without an executable workflow, the gate depends on a person remembering to run
seven commands on a quiet machine. That is not reproducible and not auditable.
The change described here makes the runner executable and leaves the gate row
`pending` until a real run is recorded by a human.

## 3. Problem

1. A public GitHub-hosted runner cannot host the gate: job timeouts are far
   shorter than 86400 s and seven sequential targets need at least 168 h.
2. Enabling the workflow is a CI change, so CLAUDE.md rule 28 requires an RFC
   before implementation.
3. A naive workflow would use broad `GITHUB_TOKEN` scopes, unpinned actions or
   extra secrets. That violates least privilege and the project's supply-chain
   rules (rules 16, 22, 36).
4. A workflow that "turns green" can be mistaken for a gate pass. The gate
   acceptance criteria require human triage, a filled `RESULTS.md`, the release
   gate row and the README blocker text. Automation must not fake any of those.

## 4. Alternatives considered

- **A. Keep the draft doc only.** Rejected: leaves #156 permanently blocked on
  manual discipline and does not make the procedure executable.
- **B. Run on `ubuntu-latest` with `timeout-minutes: 10080`.** Rejected:
  GitHub terminates hosted jobs long before 86400 s; the run would be cut off
  and produce incomplete logs that the verification package forbids as
  evidence.
- **C. Pin to a public runner label plus a long timeout.** Rejected: the
  environment is not quiet or isolated, and `exec/s` and RSS become
  meaningless. It also contradicts the draft's own host requirements.
- **D. Dedicated self-hosted runner, label-gated, least privilege (chosen).**
  Meets duration and isolation requirements, keeps secrets to the built-in
  token, and still requires a human to record the result.

## 5. Design

### 5.1 Triggers and scheduling

`.github/workflows/fuzz-24h.yml` has two triggers:

- `workflow_dispatch` (manual start, with an optional `toolchain` input).
- `schedule` with `cron: "0 3 1 * *"` (monthly, 03:00 UTC on the 1st). The
  cadence is a policy decision; changing it is a CI change needing an RFC.

`concurrency` uses group `fuzz-24h-gate` with `cancel-in-progress: false`, so a
new schedule cannot truncate an in-flight 168 h run.

### 5.1.1 Why one target per job

GitHub enforces a 5-day execution limit per self-hosted job and a 24-hour
maximum lifetime for `GITHUB_TOKEN`. Seven sequential 86400 s windows need
168 h, so a single job cannot host them. The gate job is therefore a matrix of
seven jobs with `max-parallel: 1` and `fail-fast: false`: each job runs exactly
one full 86400 s target on the dedicated host, and the workflow run spans about
seven days (well inside the 35-day workflow-run limit). A job timeout of 1500
minutes covers setup plus the window without hitting the 5-day cap. Because the
token may expire at hour 24, the artifact upload is best effort
(`continue-on-error: true`); the on-host evidence under `docs/fuzz/results/`
is authoritative and is archived by a human per the verification package.

### 5.2 Runner selection and public-runner refusal

Both jobs declare `runs-on: [self-hosted, linux, x64, fuzz-dedicated]`.
GitHub-hosted runners never carry the `self-hosted` label, so the workflow
cannot be selected by public runners by default. As defence in depth, the first
step of each job fails when `runner.environment != 'self-hosted'`.

Runner preparation, labels, network isolation and disk sizing are specified in
`docs/fuzz/self-hosted-runner.md`.

### 5.3 Permissions and secrets

- Top level `permissions: {}` (no default grants).
- Gate job: `contents: read` only, for checkout.
- Failure-notification job: `contents: read` and `issues: write`. It needs
  `issues: write` solely to comment on and label issue #156 when the gate fails.
- The only token used is the built-in `${{ github.token }}` (`GITHUB_TOKEN`).
  No repository secret, no PAT, no cloud credential, no third-party token.

### 5.4 Pinned actions

Third-party and first-party actions are referenced by commit SHA with a version
comment, matching `.github/workflows/quality.yml`:

- `actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5 # v4`
- `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4`

No new action dependency is introduced.

### 5.5 Gate execution

The jobs do not reimplement the fuzz loop. They call the existing harness,
once per target:

```
FUZZ_TARGETS=<matrix target> bash scripts/run-fuzz-window.sh "$DURATION"
# DURATION=86400
```

`FUZZ_TARGETS` is a new, optional, backward-compatible input to the harness.
It accepts only names from the canonical seven-target list and fails closed on
an unknown name. With it unset, the harness still runs all seven targets in
one invocation, exactly as before, so existing short-window evidence and the
wave 9 procedure are unchanged. In the workflow each matrix job selects one
target; the harness still owns metadata, per-target logs, `.rc` files, crash
copying, SHA-256 hashing and JSONL evidence.

Workflow environment:

- `DURATION: "86400"` - passed as the script's positional duration, so
  libFuzzer receives `-max_total_time=86400` for all seven targets.
- `FUZZ_TOOLCHAIN` - resolved as `inputs.toolchain` (manual runs), else
  `vars.FUZZ_NIGHTLY_TOOLCHAIN`, else the pinned default
  `nightly-2026-11-22`. A dated nightly is required for reproducibility; the
  floating `nightly` in `fuzz/rust-toolchain.toml` is not acceptable for the
  official run.
- `CARGO_INCREMENTAL: "0"`, `CARGO_PROFILE_TEST_DEBUG: "0"` - consistent with
  the shared-Cargo conventions used elsewhere in the repo.

Preflight fails loudly (non-zero exit) when the tree is dirty, the dated
toolchain is missing, `cargo fuzz` is unavailable, or `fuzz/Cargo.lock` is out
of sync with `fuzz/Cargo.toml`. The commit under test is printed into the log.

### 5.6 Evidence and artifact upload

`scripts/run-fuzz-window.sh` writes per-target logs, exit codes, timeline,
metadata, crash copies and an append-only JSONL record under
`docs/fuzz/results/`, following `docs/fuzz/windows-jsonl-schema.md`. Each
matrix job uploads `docs/fuzz/results/` and `fuzz/artifacts/` with `if: always()`
as artifact `fuzz-24h-<run_id>-<target>` with 90-day retention. The upload is
best effort because the token can expire at hour 24; a human archives the
on-host evidence and aggregates all seven target records before accepting the
gate. A missing log is never converted into a pass; the JSONL gate label never
claims a passed gate.

### 5.7 Idempotent failure notification

A separate job runs only `if: failure()`. It posts **one** comment per run to
issue #156, using an HTML marker keyed on `github.run_id`
(`<!-- fuzz-24h-gate run <run_id> -->`). Before commenting it reads existing
comments and exits early if the marker is present, so a re-run cannot duplicate
the notice. It also ensures the `blocked` label is present. The notice states
that the `Fuzz 24h` row stays `pending`, that #156 stays open, and gives the run
URL and artifact name. It never closes the issue and never edits the gate row.

### 5.8 What this RFC does not do

- It does not claim the gate passed or ran. No run happened in the worktree
  that authored this RFC.
- It does not close #156, move the `Fuzz 24h` row to `pass`, or change the README
  blocker text. Those remain human steps in
  `docs/fuzz/24h-gate-verification.md`.
- It does not add, remove or rename crates, change the DSL, or touch public APIs.

## 6. Risks

| Risk | Mitigation |
| --- | --- |
| Runner mislabeled or public runner used | Label-gated `runs-on` plus `runner.environment` guard in both jobs. |
| Comment period skipped | Explicit BDFL waiver recorded in section 10, following RFC-0022/RFC-0023/RFC-0032. |
| Floating nightly breaks reproducibility | Dated default plus dispatch input and repo variable override; preflight prints exact versions. |
| A single job cannot fit 168 h or a 24 h token | Matrix of one target per job, `max-parallel: 1`, 1500-minute job timeout, best-effort upload and authoritative on-host evidence. |
| Long job dies silently (reboot, OOM) | Job timeout, `if: always()` artifact upload, JSONL `.rc` per target; incomplete logs are rejected by the verification package. |
| Token over-permission | Empty top-level grants; `issues: write` only in the failure job; no other secret. |
| Workflow mistaken for a gate pass | Notice text, JSONL gate label and RFC scope statement all deny a pass claim; human acceptance criteria unchanged. |
| Concurrent runs corrupt evidence | Concurrency group with `cancel-in-progress: false`; unique run directory per invocation. |

## 7. Impact

- Adds one workflow file and one runner-preparation document.
- Touches navigation/changelog only; no code, no crate, no public API.
- Operational cost: one dedicated, quiet Linux host with a scratch volume and a
  registered self-hosted runner. This cost is accepted at enablement time and
  is documented before the workflow can be used.
- Issue #156 stays open with `blocked`; the gate row stays `pending`.

## 8. Rollback

Disable or delete `.github/workflows/fuzz-24h.yml` and remove its entries from
`docs/INDEX.md`, `docs/fuzz/README.md`, RFC index and CHANGELOG. No state
outside the runner and the GitHub run record is created: corpora and artifacts
live on the dedicated scratch volume and in uploaded artifacts. Rollback does
not alter #156 or the gate row, which were unchanged by enabling the workflow.

## 9. Verification performed for this change

- YAML parse of the new workflow with Python `yaml` (fallback: Ruby psych).
- `bash -n` on `scripts/run-fuzz-window.sh` (the harness gained the optional
  `FUZZ_TARGETS` subset; unknown names exit 2 before any fuzzing starts).
- `git diff --check` on the whole change.
- Prohibited-content scans: no emojis, no AI-tool evidence, no retired third
  party brand names.
- No fuzz target was executed for 86400 s in this worktree. This RFC records
  capability, not evidence.

## 10. Decision

- Decisor: BDFL (Angel Nereira, project maintainer).
- Date: 2026-11-22.
- Comment period: waived by BDFL decision, recorded here as required by the
  governance rules. This is the same waiver pattern used by RFC-0022,
  RFC-0023, RFC-0027 and RFC-0032.
- Result: accepted.
- Rationale: the gate has been blocked since the pre-Phase-5 audit; an
  executable, least-privilege, self-hosted runner is the minimum change that
  makes the already-documented procedure runnable without claiming a result.
  Issue #156 remains open until a human records a real full-length run.

## 11. References

- Issue #156: `Execute the 24h fuzz manual gate per target and record logs`.
- `docs/fuzz/24h-gate-verification.md` - procedure and acceptance criteria.
- `docs/fuzz/scheduled-24h-gate-draft.md` - design note superseded by this RFC.
- `docs/fuzz/windows-jsonl-schema.md` - evidence schema and gate labels.
- `docs/fuzz/self-hosted-runner.md` - host preparation and verification.
- `docs/fuzz/README.md`, `docs/INDEX.md`, `CHANGELOG.md`.
- CLAUDE.md rule 28 (RFC before CI changes), rule 16 (security), rule 36
  (reproducibility).
