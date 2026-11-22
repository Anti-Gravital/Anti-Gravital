# Alpha release security review (wave 5)

- Reviewer role: security (read-only audit; no push, no merge)
- Date: 2026-11-21
- Base commit reviewed: `a041479feca2454e87d7d54ec5fb93e6fae00e0b`
- Scope: the wave 5 alpha release path (`release`, `installer`, `examples`,
  `docs`, `qa`, `roadmap` branches) and the generated prototype quickstart.
- Method: read-only inspection of each role's worktree and notes, baseline
  verification of the existing installer/checksum/CI controls, and local
  reproduction of defects. No files outside this role's worktree were edited.

## Scope and limitation

At the time of this review the wave 5 branches were still accumulating
uncommitted work. Findings below are grounded in the actual file contents
observed in those worktrees; each issue records reproduction steps and
verification commands so the coordinator can re-check the committed tips.
Where a defect was still in progress, the affected branch is named explicitly.

## Mission checklist

| Check | Result | Notes |
| --- | --- | --- |
| Installer downloads verified before execution | PASS | Alpha channel downloads to a temp dir, verifies SHA-256, then copies to the bin dir; the binary is never executed during install. |
| No signing claimed without a key | PASS | Installer headers, RFC-0032, release notes and CHANGELOG all state checksum-only, no signing key, no signature; #141 stays open. |
| Checksum manifest cannot be sourced from the same untrusted channel | OBSERVATION (O1) | Alpha binary and its `checksums.txt` both come from `AG_INSTALLER_BASE_URL`. Honestly documented as a known limitation, not a false claim. |
| Generated prototypes run with loopback defaults | FAIL (D1) | All four templates hardcode `0.0.0.0:8080`; `ag dev` sets `BIND` that no template reads. |
| `--output json` never echoes secrets | PASS | Baseline canary test passes (12/12). No wave 5 branch changes the JSON path. |
| No new network calls in the smoke path | PASS (with caveat O4) | QA smoke uses cached dependencies and a local loopback curl; the generated scaffold still carries a pinned git dependency that needs a warm cache. |
| Workflows use minimal permissions and pinned actions | PASS | All workflows SHA-pin actions. Top-level `contents: read`; release job adds only `contents: write` and `actions: read`. |

## Findings

### D1 - Generated prototypes bind all interfaces despite loopback claim

- Severity: medium (p2)
- Affected branches: `community5/examples` (templates), `community5/docs` (claim)
- Status: DEFECT found; issue #191 opened. `community5/examples` has an
  uncommitted fix that binds all four templates to `127.0.0.1:8080` and honors
  `BIND`; re-verify on that branch's commit before closing the issue.

All four project templates bind `0.0.0.0:8080` in
`templates/{rest,realtime,fullstack,layered}/src/main.rs.tmpl` and set
`host = "0.0.0.0"` in their `config.toml.tmpl`. `ag dev` exports
`BIND=127.0.0.1:8080` (`crates/ag-cli/src/commands/project.rs::cmd_dev`), but
no template reads `BIND` for the listener, so the variable has no effect.
The alpha README (EN and ES) and `docs/releases/0.1.0-alpha.1.md` claim a
loopback default.

Impact: a tester following the documented quickstart exposes an unauthenticated
development HTTP server on every network interface. Reproduction, acceptance
criteria and verification commands are in issue #191. QA independently
confirmed the behavior and will not open a duplicate.

Recommendation: change the template default to `127.0.0.1:8080`, honor `BIND`
in the generated listener, and keep all-interface binding as an explicit opt-in.

### D2 - Release checksum manifest cannot be consumed by the alpha installer

- Severity: medium (p2)
- Affected branches: `community5/release`, `community5/installer`
- Status: DEFECT found; issue #192 opened. `community5/release` commit
  `0b39282` changes asset names to bare `ag-<target>` and emits `checksums.txt`,
  which matches the installer parser. Verified end to end locally: a release-
  style asset tree plus that manifest is accepted by the alpha installer and
  the binary installs after checksum verification. Pending both branches landing
  together before closing #192.

The release job (`docs/rfc/RFC-0032-alpha-release-process.md` section 4.3 and
`.github/workflows/release.yml`) writes manifest entries keyed by target label
(`ag-<label>/<binary>`, e.g. `ag-linux-x86-64/ag`) into a file named
`ag-<tag>-checksums.sha256`. The alpha installer looks for a bare target name
(`ag-<target>`, e.g. `ag-x86_64-unknown-linux-gnu`) in a file that defaults to
`checksums.txt`. No entry matches, so the installer aborts with "No checksum
entry". `tools/test-alpha-installer.sh` synthesizes its own bare-name manifest,
so the test passes while the real release path fails.

The failure is fail-closed, so no unverified binary is installed; this is not a
security hole, but it breaks the documented prebuilt-binary install path and its
verification step. Reproduction and acceptance criteria are in issue #192.

Recommendation: agree one manifest naming scheme across both branches and make
the installer test consume a byte-identical copy of the release manifest format.

### O1 - Alpha checksum manifest is sourced from the same channel as the binary

- Severity: observation
- Affected branches: `community5/installer`
- Resolution: `community5/installer` adds `AG_INSTALLER_BINARY_CHECKSUMS`, a
  local out-of-band binary manifest that skips the channel download; documented
  and tested in the WIP.

Both the prebuilt binary and its `checksums.txt` are downloaded from the same
`AG_INSTALLER_BASE_URL` (or the same GitHub release tag). An attacker controlling
that channel can replace both and the checksum check still passes. This is
explicitly and honestly documented as a limitation: the alpha offers checksum
integrity, not authenticity, and #141 remains open. It is recorded here because
the release checklist asks whether the manifest can come from the same untrusted
channel. Recommendation: add an optional local/out-of-band manifest path for the
alpha binary so a tester can separate the two trust boundaries.

### O2 - Release tag input is interpolated into a shell step

- Severity: observation
- Affected branch: `community5/release`
- Resolution: `community5/release` commit `0b39282` passes the tag through
  `INPUT_TAG`/`EVENT_REF` environment variables instead.

`.github/workflows/release.yml` interpolates `inputs.tag` directly into a `run:`
block in the "Resolve and validate tag" step. `workflow_dispatch` requires write
access, so the actor is trusted; still, passing the value through `env:` (as
other steps in the same workflow already do) removes the injection surface.

### O3 - Documentation says "no prebuilt binaries" while the release produces them

- Severity: observation
- Affected branches: `community5/docs`, `community5/release`, `community5/installer`

The README and release notes describe a source-only alpha with no prebuilt
binary, while the release workflow builds eight binaries and the installer adds
a prebuilt-binary alpha channel. Reconcile at merge and make sure the checksum-
only trust model of that channel is stated wherever it is documented.

### O4 - Generated scaffold carries a pinned git dependency

- Severity: observation
- Affected branches: `community5/examples`, `community5/qa`

`templates/rest/Cargo.toml.tmpl` depends on `ag-core` from a GitHub URL at a
pinned revision. The pin is good supply-chain practice, but a generated project
needs a warm Cargo cache to build offline; QA's smoke relies on that cache.
"Runs offline" should be worded as "builds offline with cached dependencies".

### O5 - Generated config.toml still advertises `host = "0.0.0.0"`

- Severity: observation
- Affected branch: `community5/examples`

After the D1 fix, the generated server binds loopback and honors `BIND`, but
`templates/*/config.toml.tmpl` still contains `host = "0.0.0.0"`. The generated
`main.rs` does not read that key, so there is no exposure, but the value is
misleading to a tester who expects the config file to control the listener.
Recommend aligning the config default with the actual loopback behavior.

## Controls verified clean on the baseline

- `install.sh` / `install.ps1`: fail-closed self integrity via externally
  supplied `AG_INSTALLER_CHECKSUMS`; repository-local manifest explicitly labelled
  as a non-trust-anchor development fallback; no download-and-execute in the
  source channel; integrity check runs before any Cargo action.
- `tools/check-installers.sh`: verifies LF endings, MSRV parity, external-manifest
  precedence and fail-closed behavior on missing/mismatched manifests.
- `cargo test --locked -p ag-cli --test json_output`: 12 passed, including
  `canary_secrets_are_never_echoed_by_a_refusal`; mutating commands refuse
  `--output json` before doing work.
- Workflows: all third-party actions pinned to full SHA; top-level least
  privilege; release job scoped to `contents: write` and `actions: read`.

## Go/no-go recommendation (interim)

Conditional no-go until the two defects are confirmed fixed on their committed
branches:

- D2 is fixed in `community5/release` commit `0b39282` and verified locally end
  to end; it clears once `community5/installer` lands with its matching parser.
- D1 has a verified-intent fix in `community5/examples` (uncommitted at review
  time); it clears once that commit lands and the templates are re-checked.

The source-install path is sound and honest. No false signing or authenticity
claim was found anywhere in the release path. With D1 and D2 confirmed and the
limitations in O1-O4 stated explicitly, the alpha is acceptable for testing.
This is not a production go: the pre-Phase-5 release gate remains open.
