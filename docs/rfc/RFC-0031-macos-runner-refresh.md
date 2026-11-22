# RFC-0031 - Refresh the macOS CI runner

- Status: accepted
- Decision date: 2026-11-22
- Author: Angel Nereira (BDFL), Gravital Labs
- Target phase: governance / CI
- Affected files: `.github/workflows/ci.yml`
- Predecessor RFC: RFC-0022 (maintainability guardrails)
- Comment period: waived by BDFL decision.

## 1. Motivation

The `build (macos-arm64)` job of `ci.yml` is cancelled one second after starting
on every run. The job log reports: `The configuration 'macos-14-arm64-us-default'
is not supported`. The other four platform jobs pass, so this is a runner-image
availability problem, not a compile failure (issue #186).

## 2. Problem

A matrix entry pinned to `macos-14` can no longer be provisioned. CI never
reports a green run, which hides real macOS regressions and violates rule 36
(reproducibility) and the four-platform exit criterion of Phase 0.

## 3. Alternatives considered

- **Remove the macOS job:** rejected, it silently reduces coverage and breaks
  the documented four-platform matrix.
- **Add `continue-on-error`:** rejected, a cancelled job would look acceptable
  and regressions would be missed.
- **Re-run until it works:** rejected, the failure is deterministic.
- **Move the matrix to `macos-15` (chosen):** keeps the same target
  (`aarch64-apple-darwin`) and the same guarantees, only the runner label
  changes.

## 4. Design

Change line 36 of `.github/workflows/ci.yml` from `os: macos-14` to
`os: macos-15`. No other matrix entry, target, toolchain step or test command
changes. The job must build and test the workspace with `--locked`, as before.

## 5. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| `macos-15` is also retired later | Medium | Medium | Keep the label in one place; a future RFC updates it again. |
| Cross-compilation differs from native arm64 | Low | Low | The target triple is unchanged and cargo build/test runs on that runner. |

## 6. Impact

- CI: one label change; four-platform coverage restored.
- Documentation: CHANGELOG entry and issue #186 closure evidence.
- No public API, crate or roadmap change.

## 7. Rollback

Revert the single label change if the new runner is unavailable; the previous
state was already non-functional, so rollback has no functional benefit.

## 8. References

- Issue #186, CI run `44972917396`, job `build (macos-arm64)`.
- RFC-0022 section 4.x (machine-checked guardrails).
