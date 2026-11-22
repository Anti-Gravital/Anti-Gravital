# Wave 4 technical review: security and low-level audit

- Date: 2026-11-21
- Reviewer role: read-only combined security and low-level review (wave 4)
- Base commit reviewed from: `42db3438e2a36e61ef16665c0414fee3f2ab1c25`
- Branches reviewed: `community4/tui`, `community4/machinejson`,
  `community4/layered`, `community4/envsecrets`, `community4/guardrails`,
  `community4/hooks`, `community4/qa`
- Method: read-only diff review plus read-only execution of each branch's own
  test scripts and Cargo tests in the shared target directory. No source file
  owned by another role was edited. No push and no merge were performed.
- Governing documents: RFC-0018, RFC-0019, RFC-0020, RFC-0021, RFC-0022,
  ADR-0008, ADR-0011 and the repository `CLAUDE.md` rules.

## 1. Executive summary

| Branch | Commits reviewed | Verdict |
| --- | --- | --- |
| guardrails | `e1bd2f1` | No defect. 3 low-severity observations. |
| qa | `f111c5e` | No defect in the branch; one cross-branch defect in its test (D1). |
| envsecrets | `254256f`, `b65e624` | No defect. 2 important/low observations. |
| layered | `e7b5f84`, `ce133f7` | No defect. Triggers D1 in qa. |
| machinejson | `20d4f79` | No defect. No secret leakage found. |
| tui | `54440ef`, `576cd6a` | One real defect (D2) plus 2 observations. |
| hooks | `1679ea0` | One real defect (D3) plus 1 observation. |

Three real defects were found and reported as focused GitHub issues:
#187, #188 and #190. None of them warrants blocking the integration of the
other work; each has a small, local fix.

## 2. Security-critical checks

### 2.1 `Secret<T>` and declared environment (envsecrets)

Passed. `crates/ag-core/src/secret.rs` implements `Debug` and `Display` as
`[redacted]` (verified for `{}`, `{:?}`, `{:#?}` and inside containers), no
`Serialize` implementation (proved by a `compile_fail` doctest that actually
runs in `cargo test`), `Zeroize::zeroize` on drop (probe test), an explicit
`expose() -> &T` with no mutable accessor, and no comparison traits. The
loader reads only declared keys, reports non-UTF-8 values as errors instead of
panicking, aggregates every problem in one error and never prints a value,
including for secrets. `parse_duration`'s suffix slicing is safe: the recognised
suffix bytes are ASCII, so `&raw[..len - 1]` is always a character boundary.
`cargo test --locked -p ag-core` passed, including all 8 doctests.

### 2.2 Machine output and secret leakage (machinejson)

Passed. `run_json_command` uses an allowlist (`schema.lint`, `schema.diff`,
`workers.list`); every other command is refused before dispatch with a
versioned JSON error and exit code 2, so no mutating executor, file, database
or network operation runs. The `canary_secrets_are_never_echoed_by_a_refusal`
test injects fake token, password and `DATABASE_URL` values and confirms none
reach stdout or stderr on refusal. Prose output is unchanged when the flag is
absent. The collision between the global `--output` format flag and
`generate --output DIR` is resolved by argument-id renaming and
`--output-format`; no clap duplicate-id panic occurs (verified by the passing
test suite). The contract document honestly limits version 0.1.0 to three
commands and records that `ag describe` and plan/apply are not implemented.

### 2.3 Hook scripts and exit codes (hooks)

Passed with one exception (D3). `.githooks/pre-push` uses `exec`, so ag-check's
exit code propagates and there is no in-script silent bypass. `commit-msg`
checks subject length, AI attribution and emojis, and fails loudly when no
emoji detector is available. `install-hooks.sh` is idempotent, writes only the
local git config and downloads nothing. ag-check reports SKIP separately from
PASS. The exception: ag-check's own emoji sub-check silently degrades on
macOS (see D3).

### 2.4 CI permissions, pinned actions and network (guardrails, hooks)

Passed. The three new guardrail workflows pin every action to a SHA and use
workflow-level least privilege: `guardrails` has `contents: read`;
`security-audit` adds `issues: write`; `stale-prs` adds `issues: write` and
`pull-requests: write`. No new product runtime downloads were introduced. The
weekly job installs `cargo-audit` and `cargo-deny` in CI, mirroring the
pre-existing `quality.yml` pattern and the explicit authorisation of RFC-0022.
The hooks branch changes no CI file. The guardrail scripts passed on their own
branch: `check-file-size.sh` (247 files, 14 seeded exceptions),
`check-derived-docs.sh` (39 derived documents match) and `test-guardrails.sh`
(all 43 assertions).

## 3. Low-level correctness checks

- **Config validation before listener open.** `ag config validate` validates a
  file with the same `ag_core::ShieldConfig` loader the runtime uses, and every
  `set` is validated before an atomic temp-file-plus-rename write; an invalid
  result never touches the file (unit-tested). However, neither envsecrets nor
  tui wires validation into a generated project's startup before a listener
  binds; both record this as a future stage (observations O4 and O7). The
  roadmap box must not be checked as complete until that call lands.
- **Template dependency acyclicity.** The layered workspace graph is
  `domain` (no local deps), `app -> domain`, `infra -> app, domain, ag-data`,
  `api -> app, domain, ag-core, axum`, root -> all four. There is no path back
  to an outer layer. `layered_dependencies_point_inward` asserts the direction,
  and a freshly generated project passes `cargo metadata --no-deps --offline`.
  The `axum 0.7` choice matches the existing templates and the pinned ag-core
  revision, so no Router type mismatch is introduced.
- **Guardrail script idempotency.** `security-report.sh` maintains one marked
  issue and is idempotent across create/update/reopen/close; `close-stale-prs.sh`
  is idempotent and keeps branches; `install-hooks.sh` is idempotent. All were
  exercised by their branch's own tests.
- **No panics on malformed CLI input.** clap handles argument parsing; tui's
  `parse_key` returns a usage error, malformed TOML returns a failure error and
  `default_value` only uses `expect` on compile-time constants; machinejson's
  refusal path parses before dispatch. No new panic on user-controlled input
  was found.

## 4. Findings

### D1 - qa layered regression test silently skips the shipped feature

- Severity: medium (silent loss of regression coverage)
- Affected branch: `community4/qa` (triggered by `community4/layered`)
- Issue: #187 (`bug`, `ag-cli`, `p2`, `good first issue`)

`wave4_regression.rs::layered_template_dependency_direction` probes only
`fullstack` and `realtime` for `ag.toml` or `crates/`. RFC-0019 stage A landed
the layout as an opt-in fifth template `--template layered`, and the layered
branch's own `default_and_legacy_templates_are_unchanged` test asserts that
`fullstack` and `realtime` still have neither. After the two branches merge,
the test prints SKIP and executes zero assertions while the feature is present.
Recommendation: add `layered` to the probe list and confirm the test fails when
a layer dependency edge is deliberately broken.

### D2 - `ag config set` silently corrupts integers above `i64::MAX`

- Severity: medium (silent data corruption)
- Affected branch: `community4/tui`
- Issue: #188 (`bug`, `ag-cli`, `p2`)

`config.rs` validates integer values with `parse::<u64>()` but `build_value`
uses `change.value.parse::<i64>().unwrap_or(1)`. `ShieldConfig` stores these
fields as `usize`, so values in `(i64::MAX, u64::MAX]` are legitimate and pass
validation, then silently become `1`. The post-write validation sees `1` and
passes, so the command reports success. Reproduced on the branch:

```
ag --no-interactive config set runtime.blocking_threads 18446744073709551615
Updated config.toml (1 change(s)).
[runtime]
blocking_threads = 1
```

Recommendation: parse as `u64` (or `usize`) in both paths, or reject out-of-range
values with a non-zero exit and leave the file unchanged. Add a unit test for a
value above `i64::MAX`.

### D3 - ag-check silently skips the emoji policy check on macOS

- Severity: low-to-medium (silent omission of a documented local gate)
- Affected branch: `community4/hooks`
- Issue: #190 (`bug`, `workflow`, `p2`)

`check_prohibited` runs its emoji scan only under
`if printf '%s' 'x' | grep -qP 'x' ... && grep -IPq ...`. There is no `else`, no
Perl fallback and no SKIP report, so on a system without GNU `grep -P` (macOS)
the check prints `PASS: prohibited` while omitting emoji detection. This
contradicts the script's stated contract and the commit-msg hook, which fails
loudly when no detector exists. Recommendation: use the portable Perl detector
from commit-msg or fail loudly; add a no-detector test case.

## 5. Observations (no issue opened)

These are low-severity or scope-limited and should be handled at merge time, not
as blockers.

| ID | Branch | Observation | Recommendation |
| --- | --- | --- | --- |
| O1 | guardrails | `gh issue list` runs in process substitution; under `set -e` a transient failure is not propagated, so a duplicate security issue could be created. | Capture the list into a variable and check its exit status. |
| O2 | guardrails | Both report/close scripts call `gh label create` even in DRY_RUN mode. Harmless and idempotent, but dry run is not strictly side-effect free. | Route label creation through the dry-run wrapper or skip it. |
| O3 | guardrails | The stale-PR workflow has no exemption for `blocked`-labelled PRs. Per RFC-0022 section 4.8 this is intended (branch kept, PR reopenable). | Consider an exemption so externally blocked work is not auto-closed. |
| O4 | envsecrets | The loader is not wired into any startup path; "validate before listener opens" is a documented contract, not enforced yet. | Verify the before-bind call lands with PR 4/5 before checking the roadmap box. |
| O5 | envsecrets | `resolve` clones `RawValue::Present(String)`; the secret moved into `Secret` is zeroized, but the original in the `provided` map is dropped plainly. | Take the value out of the map instead of cloning (defense in depth). |
| O6 | tui | RFC-0018 status says "stages A, B and C are implemented", but section 4.4's `--with MODULE` composition is not delivered; the qa reconciliation correctly keeps the box partial. | Reconcile the wording in the merge PR (qa section 5 already flags it). |
| O7 | tui | `ag config validate` is on-demand only and is not called before a listener binds in generated projects. | Same as O4: verify the startup wiring before checking the box. |
| O8 | hooks | `tests/hooks/test-hooks.sh` asserts the working repo's `core.hooksPath` is unset. A contributor who installed the hooks sees 33 passed / 1 failed. This is environmental, not a code defect. | Save and restore the original value so the suite is deterministic. |

## 6. Issues opened

- #187 - qa: layered regression test probes wrong templates and skips silently.
- #188 - ag config set silently corrupts integers above `i64::MAX`.
- #190 - ag-check silently skips emoji policy check without `grep -P`.

No issue was opened for style preferences. No issue was closed by this review.

## 7. Verification record for this review

All verification commands were run read-only against the other roles'
worktrees; only this review worktree was modified.

- `bash tools/check-file-size.sh` (guardrails): exit 0.
- `bash tools/check-derived-docs.sh` (guardrails): exit 0.
- `bash tools/test-guardrails.sh` (guardrails): all assertions passed.
- `cargo test --locked -p ag-cli --test wave4_regression` (qa): 6 passed.
- `cargo test --locked -p ag-core` (envsecrets): all tests and 8 doctests passed.
- `cargo test --locked -p ag-cli --test layered` (layered): 3 passed.
- `cargo metadata --no-deps --offline` on a fresh layered scaffold: exit 0.
- `cargo test --locked -p ag-cli` (machinejson): 64 + 12 + 6 tests passed.
- `cargo test --locked -p ag-cli` (tui): 89 + 11 tests passed.
- Reproduction of D2 with the built `ag` binary: success message plus `1`
  written to `config.toml`.
- `bash tests/hooks/test-hooks.sh` (hooks): 33 passed, 1 failed; the failure is
  the environment-dependent O8, reproduced and explained above.
- `git diff --check`, emoji scan, AI-attribution scan and retired-brand scan on
  this review worktree: clean (see commit verification).
