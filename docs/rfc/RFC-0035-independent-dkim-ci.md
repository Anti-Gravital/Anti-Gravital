# RFC-0035 - Opt-in CI check for independent DKIM verification (#178)

- Status: accepted
- Author: maintainer delegate, issue #178
- Draft date: 2026-11-22
- Target phase: 4.6 (mail); verification tooling only, no phase/scope/roadmap change
- Affected modules: `scripts/check-dkim-independent.sh` (new),
  `.github/workflows/dkim-independent.yml` (new), `docs/security/`,
  `docs/audits/`, `docs/rfc/README.md`
- Predecessor: none. Records the CI part of the work begun in wave 2
  (`tools/dkim_verify.py`, `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`).
- Comment period: waived (see section 8)

## 1. Motivation

Issue #178 asks that a DKIM signature produced by the upgraded mail stack
verify with an independent implementation. Wave 2 supplied that evidence with
dkimpy 1.1.8 + PyNaCl 1.6.2 over the embedded known-answer vectors, but only as
a manual `#[ignore]` test invocation. There is no repeatable, discoverable
command that reports a clean PASS/SKIP/FAIL verdict with the exact tool
versions, and CI has no opt-in way to reproduce the evidence after merge.

## 2. Problem

- The dkimpy test is `#[ignore]` so the default `cargo test` stays hermetic.
  Nothing enumerates its prerequisites or distinguishes "verifier absent" from
  "signature rejected".
- A bare `cargo test ... --ignored` run can exit 0 while executing zero tests
  if the wrong feature set is used, which would be a false PASS.
- Adding a mandatory CI job that installs Python packages would change the
  default pipeline and is not justified for an offline known-answer check.

## 3. Alternatives considered

- **Document the manual command only.** Rejected: no reproducible verdict,
  and the SKIP/FAIL distinction is easy to lose.
- **Add a required job to `ci.yml`.** Rejected: changes default CI, installs
  test-only Python packages on every push, and would imply a gate that this
  check does not close.
- **Opt-in manual workflow plus a wrapper script (chosen).** Default CI is
  untouched; a maintainer can reproduce the evidence on demand with pinned
  tool versions and explicit outcome semantics.

## 4. Design

1. `scripts/check-dkim-independent.sh`
   - Detects python3, dkimpy and PyNaCl in one probe and prints exact versions
     (python3, cargo, rustc, dkimpy, pynacl).
   - Reports `PASS` only when both embedded vectors verify: it requires the
     cargo exit status to be 0, two `dkim=pass` lines and no `dkim=fail` or
     `dkim=error` line. Full test path plus `--exact` makes a zero-test run
     detectable instead of a silent success.
   - Reports `SKIP` (exit 0) when a dependency is missing. SKIP is never
     rendered as PASS; the result line names the missing dependency.
   - Reports `FAIL` (exit 1) on a rejected vector, a verifier error or a
     nonzero cargo status, and prints the verifier output.
   - Uses `cargo test --locked` with no new Cargo dependency.
2. `.github/workflows/dkim-independent.yml`
   - `workflow_dispatch` only; no `push` or `pull_request` trigger.
   - Least privilege `contents: read`, SHA-pinned checkout and toolchain.
   - Creates a venv, installs `dkimpy==1.1.8` and `pynacl==1.6.2`, then runs
     the script in the same step so the venv interpreter is the one probed.
3. Documentation: this RFC, a section in
   `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`, and a real-run record in
   `docs/audits/dkim-independent-ci.md`.

Explicit constraints:

- No change to `ci.yml` or any other default workflow. Default CI stays green
  and hermetic with respect to Python tooling.
- No Cargo dependency, no crate, no CLI command, no DSL or master-document
  change. No public API or product behaviour change.
- The check operates on offline embedded vectors only. It does not contact a
  receiver and does not close #178.

## 5. Risks

- **SKIP read as PASS by a human or job.** Low. Mitigation: the result line is
  literally `RESULT: SKIP`, versions show `missing`, and the audit doc states
  the semantics explicitly.
- **Opt-in job rots because nobody dispatches it.** Low. Mitigation: the
  command is a single local script, so the evidence can be reproduced without
  GitHub; the audit doc records a real local run.
- **Pinned Python versions stale.** Low. Mitigation: pins are recorded in the
  workflow and audit doc; updating them is a documented, reviewable change.

## 6. Impact

- Product scope, roadmap, public APIs: none.
- CI: one new manual-only workflow; default pipeline unchanged.
- Complexity: one shell script and one short workflow.
- Documentation: RFC index, security doc, audit evidence, CHANGELOG.

## 7. Rollback

Delete `scripts/check-dkim-independent.sh`, `.github/workflows/dkim-independent.yml`
and this RFC; revert the index/doc entries. No state, keys or external
services are involved.

## 8. Decision

- Decisor: project maintainer, consistent with RFC-0028 precedent for an
  additive verification job.
- Date: 2026-11-22.
- Result: accepted.
- Comment period: waived. Justification: additive, opt-in verification over an
  already accepted verifier choice; no API, scope, roadmap or default-CI
  change, and no reversibility cost.
- Scope note: acceptance authorizes the opt-in check only. It does not assert
  that the #178 real-mailbox gate passed.

## 9. References

- Issue #178 (upgrade the mail protocol stack in the native MTA).
- `tools/dkim_verify.py` and `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`
  (wave 2 independent verifier evidence).
- RFC-0028 (precedent for feature-verification CI coverage).
- CLAUDE.md rules 17, 19, 28 and 35 (benchmark honesty, continuous docs, RFC
  for CI changes, ADR/RFC persistence).
