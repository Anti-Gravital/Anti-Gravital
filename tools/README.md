# tools/

Support scripts for the repository. They are not part of the distributed
product; they regenerate derived documents, verify integrity and enforce the
maintainability guardrails of RFC-0022.

## Current scripts

- `split-masters.sh`: regenerates the verbatim files under `docs/architecture/`
  and `docs/roadmap/` from the masters in `docs/master/`. Idempotent. Its line
  ranges must be recalculated whenever a master section moves.
- `check-derived-docs.sh`: regenerates those files in a scratch copy and fails
  on any drift. It never modifies the working tree (CI job `guardrails /
  derived documents match masters`).
- `check-file-size.sh`: fails when a tracked Rust source file exceeds 600 lines
  without an entry in `file-size-allowlist.txt`. Stale entries also fail, so
  the allowlist may only shrink (RFC-0022 section 4.3).
- `close-stale-prs.sh`: labels, notes and closes pull requests with no activity
  for 30 days; never deletes branches (RFC-0022 section 4.8).
- `security-report.sh`: maintains the single issue labelled `security` used by
  the weekly `security-audit` workflow. It creates, updates, reopens or closes
  that issue in place and never reuses a human issue (RFC-0022 section 4.2).
- `test-guardrails.sh`: standalone tests for all four guardrail scripts using a
  fake `gh` CLI and temporary fixtures. No cargo and no network.
- `scaffold-crates.sh`: regenerates the crate skeletons. Idempotent; used only
  when the workspace is initialized.
- `scaffold-docs.sh`: regenerates the README files under `docs/modules/` and
  the verbatim views under `docs/security/`, `docs/governance/`,
  `docs/benchmarks/` and `docs/dsl/`. Idempotent.
- `check-installers.sh`: verifies installer checksums, line endings and MSRV
  parity.
- `release-signing/`: test-only rehearsal of the RFC-0030 signed-manifest flow
  using the OpenSSL CLI and a throwaway Ed25519 key. It never creates or
  publishes a production key and does not change the release workflow.

## Rules

- Scripts live here while they are needed to keep the repository consistent.
  When one becomes permanent and critical, evaluate moving it into a dedicated
  crate under `crates/` after an RFC.
- No secrets. No credentials. No external resource access except the explicit
  GitHub CLI calls in the two scheduled workflows.
- Compatible with bash on Linux and macOS.
- No emojis in output.
- Every guardrail script must be safe to run twice and must print a clear fix
  instruction on failure. `tools/test-guardrails.sh` checks that behavior.
