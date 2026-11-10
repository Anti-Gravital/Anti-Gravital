# RFC-0022 - Maintainability guardrails: enforceable rules against degradation

- Status: proposed
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: governance; applies from acceptance
- Affected modules/crates: `CLAUDE.md`, `CONTRIBUTING.md`, CI workflows, all
  crates
- Predecessor RFC: none
- Comment period: seven calendar days, or waived by BDFL decision. Merging the
  pull request that carries this RFC records the decision in section 9.

## 1. Motivation

The repository has forty rules of intent. The audit of 2026-11-10 shows which
kind of rule holds over time: the ones a machine checks. Formatting, lints,
master-document hashes and prohibited content are all enforced by CI and were
all clean. What had degraded was everything that depends on someone
remembering.

This RFC adds a small set of rules, each paired with the mechanism that
enforces it. A rule without a mechanism is not proposed.

## 2. Problem

Findings of the audit that no existing check caught:

1. `cargo audit` had been failing on `main` for months with fifteen advisories,
   because the audit only runs when someone pushes and nobody had pushed.
2. `Cargo.lock` did not match the manifests; CI builds without `--locked`
   (issue #168).
3. `crates/ag-cli/src/main.rs` is 1 829 lines; two `ag-dsl` files are near
   1 500. Nothing limits growth.
4. Documentation drifted from code in several places (issues #170, #171, #172
   and the `ag-cli` module page), each discovered by reading, not by a check.
5. Additive phases stay "in progress" indefinitely (Phase 4.6 since June).
6. A pull request (#167) has been open and stale for five months.
7. The scope keeps widening by accumulation; there is no single statement of
   what the product refuses to be.

## 3. Alternatives considered

**A. Add more prose rules to `CLAUDE.md`.** Rejected: section 2 is the
evidence that unenforced rules decay.

**B. Rely on review.** Rejected: the project has one maintainer.

**C. Rules with mechanical enforcement (chosen).**

## 4. Proposed design

Each rule states the rule, the mechanism and the threshold. On acceptance, the
text of 4.1 to 4.10 is added to `CLAUDE.md` as a new section "Guardrails", and
the mechanisms are implemented in the pull requests of section 5.

### 4.1 Scope statement

Anti-Gravital is a backend tool: runtime, schema language, modules, CLI,
deployment integration. It does not render user interfaces, it does not host
applications as a service, and it does not replace the infrastructure tools it
integrates with. A proposal outside this statement needs an RFC that amends
the statement first.

*Mechanism:* the RFC template gains a mandatory field "Fits the scope
statement: yes / amends it".

### 4.2 Dependency freshness and security

- A scheduled CI job runs `cargo audit` and `cargo deny check` weekly on
  `main` and opens an issue labelled `security` when either fails.
- Compatible updates (`cargo update`) are applied at least monthly, in a
  dedicated pull request.
- Incompatible (major) upgrades are evaluated at the start of each phase;
  each one that is not taken is recorded with its reason in an issue.
- Every cargo invocation in CI uses `--locked`.
- An advisory ignore must carry the advisory id, a reachability argument and a
  removal condition; ignores are reviewed at each phase start.

### 4.3 File and function size

- A source file over 600 lines, or a function over 80 lines, fails CI unless
  listed in an allowlist file with an issue number. The allowlist is seeded
  with today's exceptions and may only shrink.

*Mechanism:* `clippy::too_many_lines` with the configured threshold in
`clippy.toml`; a small script for file length.

### 4.4 Public API stability

- Each publishable crate has its public API checked for SemVer-incompatible
  changes in CI against the last release tag, from the first tagged release.

*Mechanism:* `cargo-semver-checks` (a development tool, not a dependency).

### 4.5 Errors in binaries

- Command functions return a typed error enum, not `String`. Error messages
  state what failed, why, and what the user can do next.

*Mechanism:* review checklist; the `ag-cli` module split of `RFC-0018`
introduces the error type.

### 4.6 Documentation that cannot drift silently

- Derived documents are regenerated in CI and the job fails on any diff
  (`tools/split-masters.sh` followed by `git diff --exit-code`).
- The command tables in the root README and in `crates/ag-cli/README.md` are
  checked against the clap command tree by a test.
- Every crate README states its status in a fixed first-line format that a
  script compares with the crate's actual feature set.

### 4.7 Phases must close

- An additive phase declares its exit criteria when it is created. A stage
  with no progress for ninety days is either closed or moved out by an RFC
  amendment; it does not remain "in progress".

*Mechanism:* the phase-start review of 4.2 includes this check; the roadmap
status table carries a "last progress" date.

### 4.8 Work in progress

- A pull request with no activity for thirty days is closed with a note; its
  branch can be reopened. An issue marked `blocked` names what it is blocked
  on and who can unblock it.

*Mechanism:* a scheduled workflow that labels and then closes stale pull
requests. Issues are not closed automatically.

### 4.9 One way to do each thing

- One interaction library, one error-handling pattern per layer, one
  configuration mechanism, one HTTP stack, one async runtime. Introducing a
  second requires an RFC that also removes the first or justifies both.

*Mechanism:* `cargo deny` `bans` section lists the alternatives that are not
allowed (for example a second prompt library or a second TLS backend).

### 4.10 Tests are part of the change

- A pull request that changes behavior without adding or changing a test
  states why in its descriptor. Coverage of the workspace may not decrease by
  more than one percentage point in a single pull request.

*Mechanism:* the existing coverage job compares against the base branch.

## 5. Implementation plan (only after approval)

1. PR 1 - `CLAUDE.md` "Guardrails" section; RFC template field.
2. PR 2 - CI: `--locked` everywhere (closes issue #168), weekly audit and deny
   job, derived-document diff check.
3. PR 3 - size limits with the seeded allowlist.
4. PR 4 - command-table test; crate status check.
5. PR 5 - stale pull request workflow; `bans` entries.
6. PR 6 - `cargo-semver-checks`, activated with the first release tag.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Too many gates slow a single maintainer | Medium | Medium | Every gate is automatic; none adds manual steps. Thresholds start at today's values plus an allowlist. |
| Size limits cause artificial splitting | Medium | Low | Allowlist with justification; the limit is a prompt to consider structure, the allowlist is the escape. |
| Automated issue creation becomes noise | Low | Low | One issue per failing check, updated in place, not one per run. |
| Scheduled jobs consume CI minutes | Low | Low | Weekly cadence; audit and deny are fast. |

## 7. Impact

- **Scope:** governance only.
- **CI:** two scheduled jobs, three new checks, `--locked`.
- **Contributors:** a pull request can fail for size, lockfile, derived-doc or
  command-table drift; each failure message says how to fix it.

## 8. Rollback

Each guardrail is one CI job or one configuration entry and can be disabled
independently. A guardrail that produces more false failures than real ones
over a phase is removed by amendment.

## 9. Decision

To be completed after the comment period.

- Decider: BDFL.
- Decision date: pending.
- Result: pending.

Open questions: the size thresholds of 4.3 (600 and 80 lines proposed), and
whether stale pull requests are closed automatically (4.8) or only labelled.

## 10. References

- `CLAUDE.md` rules 15, 16, 18, 21, 26, 29, 30, 38.
- Issues #168, #170, #171, #172, #173, #174.
- `RFC-0018` (module split of `ag-cli`).
