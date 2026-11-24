# Chrono edge monitor - upstream re-check and drift baseline (issue #185)

- Date: 2026-11-22
- Scope: wave 12, branch `community12/i185`. Local, opt-in drift check.
- Language: English (ADR-0008).
- Outcome: no upstream change since the definitive analysis, so that analysis is
  not repeated here. This document records a fresh upstream probe and a small
  automated check that detects future drift. Issue #185 stays OPEN; this work
  uses `Refs #185`, never `Closes #185`.

## 1. Why this document exists

The definitive investigation is `docs/audits/chrono-unblock-analysis.md`
(wave 11). It concluded that the only remaining chrono edge is upstream of our
code (`object_store -> chrono`) and that no safe in-repository fix exists.
This wave re-checked the external facts rather than restating that analysis.
Nothing had changed, so instead of more prose the deliverable is an automated
check plus a recorded baseline.

## 2. Upstream re-check (real registry metadata, 2026-11-22)

All values below were read from live endpoints on 2026-11-22.

| Probe | Result |
| --- | --- |
| `https://crates.io/api/v1/crates/object_store` | `default_version`, `newest_version`, `max_version` and `max_stable_version` all `0.14.2`; `num_versions` 37; crate `updated_at` 2026-09-15. No newer release. |
| `object_store` 0.14.2 feature map | `cloud-base` still contains `chrono/serde`; `aws` still derives from `aws-base` -> `cloud-base`. No chrono-free feature set. |
| Workspace `Cargo.lock` | `object_store` is `0.14.2` and its dependency list includes `chrono` (line 4351). |
| `https://api.github.com/repos/apache/arrow-rs-object-store/issues/909` | `state=open`, no assignee, no milestone, `comments=1`, created 2026-11-17, updated 2026-11-17. |
| GitHub PR search, `repo:apache/arrow-rs-object-store is:pr chrono in:title` | `total_count=0`. No upstream fix PR. |
| `https://crates.io/api/v1/crates/chrono` | newest stable `0.4.45` (published 2026-06-04). |
| `RUSTSEC-2026-0543` advisory file | `informational = "unmaintained"`, `patched = []`. Still no patched version. |

Conclusion: there is no newer `object_store` release, no upstream PR removing
the non-optional chrono dependency, and no maintained chrono-free release of
that dependency. The blocked state and the removal condition in
`docs/audits/chrono-unblock-analysis.md` section 4 are unchanged. A native S3
replacement still needs an accepted RFC, a security review and verifiable
integration evidence; it is not a minimal fix. No manifest, lockfile, source
file or `deny.toml` ignore was changed.

## 3. The monitor

`scripts/check-chrono-edge.sh` is an **opt-in local check**. It is not wired
into any default CI workflow. Run it from the repository root:

```
bash scripts/check-chrono-edge.sh
bash scripts/check-chrono-edge.sh --print-baseline   # print a new baseline line
```

It records two commands:

1. `cargo tree --workspace --all-features -i chrono`
2. `cargo deny --all-features check`

It normalizes ANSI colour, the checkout root (`<repo>`) and `$HOME` (`<home>`),
then hashes:

```
tree:  <normalized cargo tree -i chrono output>
deny.exit=<exit status>
deny.status=<sorted unique advisories/bans/licenses/sources summary and error lines>
```

The full verbose `cargo deny` output is not hashed: it contains
duplicate-version reverse trees that change on unrelated lockfile updates and
would produce false drift. The compact signal keeps the security-relevant
result (per-section statuses, advisory ids, exit code).

The SHA-256 is compared with line 1 of
`docs/audits/chrono-edge-baseline.sha256`.

### Result semantics

| Result | Exit | Meaning |
| --- | --- | --- |
| `PASS` | 0 | Current hash equals the baseline. The chrono edge is still present. **PASS is not a closure signal.** |
| `FAIL` | 1 | Real drift: a policy verdict changed the deny outcome. Inspect the report and re-examine before updating the baseline. |
| `SKIP` | 0 | A required tool is missing. A SKIP is never reported as verification. |
| `ERROR` | 2 | Baseline file missing or malformed, an unknown argument, OR `cargo deny` exited non-zero without producing a policy verdict (a tool/fetch error such as a cold advisory-cache fetch). An ERROR is neither a PASS nor a drift FAIL. |

`chrono.tree.empty=false` in the report is expected while #185 is open. The
unblock condition is the conjunction of an empty tree, an accepted upstream or
RFC-backed fix, a removed ignore and green gate commands; see
`docs/audits/chrono-unblock-analysis.md` section 4. That prose is not repeated
here.

## 4. Recorded baseline (real run in this worktree)

```
tool.cargo=cargo 1.99.1 (5f94df478 2026-08-27)
tool.cargo-deny=cargo-deny 0.20.2
command.tree=cargo tree --workspace --all-features -i chrono
command.deny=cargo deny --all-features check
chrono.tree.empty=false
--- cargo tree -i chrono (normalized) ---
chrono v0.4.45
└── object_store v0.14.2
    └── ag-storage v0.1.0-alpha.2 (<repo>/crates/ag-storage)
        [dev-dependencies]
        └── ag-integration-tests v0.1.0-alpha.2 (<repo>/tests/integration)
--- cargo tree -i chrono (end) ---
deny.exit=0
deny.status=advisories ok, bans ok, licenses ok, sources ok;
hash.algorithm=sha256
hash.current=dd4fb95d59b23b3d47aeb7a18f742e091227b7e93076cbba3e015fff26ccc83f
hash.baseline=dd4fb95d59b23b3d47aeb7a18f742e091227b7e93076cbba3e015fff26ccc83f
RESULT: PASS reason=no-drift-from-baseline detail=chrono edge and deny outcome unchanged; issue #185 REMAINS OPEN (PASS is not a closure signal)
```

The hash was produced twice with identical output before the baseline was
written. The drift path was also exercised by temporarily substituting a wrong
baseline hash: the script returned exit 1 with `RESULT: FAIL`, and the original
baseline was restored and re-verified with `RESULT: PASS` / exit 0.

## 5. Scope and non-changes

Added: this evidence document, the baseline hash file and the monitor script;
one CHANGELOG entry and this branch's PR descriptor. Unchanged: `deny.toml`,
every `Cargo.toml`, `Cargo.lock`, all Rust sources, README, RFC statuses and
CI workflows. No issue was closed and no gate was claimed.

## 6. References

- Issue #185: https://github.com/Anti-Gravital/Anti-Gravital/issues/185
- Definitive analysis: `docs/audits/chrono-unblock-analysis.md`
- RUSTSEC-2026-0543: https://rustsec.org/advisories/RUSTSEC-2026-0543.html
- Upstream issue: https://github.com/apache/arrow-rs-object-store/issues/909
- Registry metadata: https://crates.io/api/v1/crates/object_store

## 7. Revalidation 2026-11-23 (wave 29, branch `community29/i185`)

This is an additive, dated re-check. It does not restate the wave-11 definitive
analysis or the wave-12 upstream probe; it only records what was observed today
and the exact condition that would allow #185 to be closed. Issue #185 stays OPEN
and `blocked`; this work uses `Refs #185`, never `Closes #185`.

### 7.1 Local state (commands run in this worktree)

```
$ cargo tree --workspace --all-features -i chrono
chrono v0.4.45
└── object_store v0.14.2
    └── ag-storage v0.1.0-alpha.2 (<repo>/crates/ag-storage)
        [dev-dependencies]
        └── ag-integration-tests v0.1.0-alpha.2 (<repo>/tests/integration)
```

The single edge is byte-for-byte the one in section 4. No manifest, `Cargo.lock`,
Rust source, `deny.toml` ignore or CI workflow was changed.

### 7.2 Upstream probe (live, read-only, 2026-11-23)

| Probe | Result today | Change vs wave 12 |
| --- | --- | --- |
| `cargo search object_store --limit 3` | `object_store = "0.14.2"` | none |
| `https://crates.io/api/v1/crates/object_store` | `default_version`/`newest_version`/`max_version`/`max_stable_version` all `0.14.2`; `updated_at` 2026-09-15; `num_versions` 37 | none |
| 0.14.2 feature map | `cloud-base` still lists `chrono/serde`; `aws` still derives from `cloud-base` | none |
| `https://api.github.com/repos/apache/arrow-rs-object-store/issues/909` | `state=open`, no assignee, no milestone, `comments=1`, created 2026-11-17, updated 2026-11-17 | none |
| GitHub PR search `repo:apache/arrow-rs-object-store is:pr chrono in:title` | `total_count=0` | none |

There is no newer `object_store` release and no upstream PR removing its
non-optional chrono dependency. The external gate is unchanged, so no minimal
safe fix became available.

### 7.3 Monitor result and an honest transient finding

First invocation of `bash scripts/check-chrono-edge.sh` on a cold advisory cache
returned `FAIL` (exit 1) with `deny.exit=1`, `deny.status=none` and hash
`0b8710…c39368`. A direct `cargo deny --all-features check` immediately after
returned exit 0 with `advisories ok, bans ok, licenses ok, sources ok`, and a
second monitor invocation returned:

```
chrono.tree.empty=false
deny.exit=0
deny.status=advisories ok, bans ok, licenses ok, sources ok;
hash.current=dd4fb95d59b23b3d47aeb7a18f742e091227b7e93076cbba3e015fff26ccc83f
hash.baseline=dd4fb95d59b23b3d47aeb7a18f742e091227b7e93076cbba3e015fff26ccc83f
RESULT: PASS reason=no-drift-from-baseline detail=chrono edge and deny outcome unchanged; issue #185 REMAINS OPEN (PASS is not a closure signal)
```

The recorded baseline hash was NOT changed. The cold-cache `FAIL` is reported as
a tool-reliability finding (S-11 outbox `chrono-monitor-cold-cache-false-fail.md`,
also summarised in the PR descriptor): the script maps both a policy violation
and a `cargo deny` tool/fetch failure to exit 1, and an empty `deny.status`, so
an infrastructure hiccup can look like dependency drift. Root cause was not
reproduced on a warm cache; the hypothesis is a first-run advisory-DB fetch
error. This does not affect the #185 conclusion, which rests on the reproducible
warm-cache PASS and the unchanged upstream facts above.

### 7.4 Exact condition to close #185

`Closes #185` becomes permissible only when the conjunction in
`docs/audits/chrono-unblock-analysis.md` section 4 is fully met: the reverse tree
is empty; the chrono-free graph comes from an upstream chrono-free
`object_store` release or an accepted RFC plus a verified native S3 adapter;
`cargo deny --all-features check` passes with the `RUSTSEC-2026-0543` ignore
removed; `cargo audit` shows no chrono warning; and test, fmt and clippy gates
are green. Until then the ignore stays and the keyword remains `Refs #185`.

## 8. Cold-cache false FAIL fix (wave 30, branch `community30/chronomonitor`)

Section 7.3 recorded a tool-reliability finding (S-11 outbox
`chrono-monitor-cold-cache-false-fail.md`): on a cold advisory cache `cargo deny`
exited non-zero with an empty `deny.status`, and the monitor reported a drift
`FAIL`. That conflated two different causes. `cargo deny` exits non-zero both for
a REAL policy verdict (an advisory/bans/licenses/sources violation, which always
emits a recognizable verdict line: a per-section status, an `error[...]` line or
a `N allowed` warning) and for a tool/fetch failure that happens BEFORE any
verdict (no verdict line, empty signal). A network hiccup must not be presented
as dependency drift.

`scripts/check-chrono-edge.sh` now classifies the outcome before hashing:

- non-zero `deny` exit AND empty `deny.status` -> `RESULT: ERROR
  reason=deny-tool-error-no-verdict`, exit 2. This also applies to
  `--print-baseline`, so a broken run can never be baked into a new baseline.
- non-zero exit WITH a verdict, or exit 0 -> unchanged hash comparison; a hash
  mismatch is still `FAIL` (real drift) and a match is still `PASS`.

Fail-closed is preserved: an ERROR is not a PASS, and real drift detection is
not weakened. The baseline hash (`dd4fb95d...`) is unchanged. Issue #185 stays
OPEN and `blocked`; this change uses `Refs #185`, never `Closes #185`.

Verification (commands run 2026-11-23 in this worktree):

```
$ bash -n scripts/check-chrono-edge.sh                    -> exit 0
$ git diff --check                                        -> exit 0
$ bash scripts/check-chrono-edge.sh                       -> RESULT: PASS, exit 0
    hash.current = hash.baseline = dd4fb95d... (real run, warm cache)

Controlled stub `cargo` on PATH (no real network), three scenarios:
  cold fetch error  -> deny.exit=1, deny.status=none, deny.verdict=absent
                       RESULT: ERROR reason=deny-tool-error-no-verdict, exit 2
  real drift        -> deny.exit=1, deny.status=advisories fail;...,
                       deny.verdict=present, hashes differ -> RESULT: FAIL, exit 1
  clean             -> deny.exit=0, hashes match -> RESULT: PASS, exit 0
  cold + --print-baseline -> RESULT: ERROR, exit 2, NO hash emitted
  unknown argument  -> RESULT: ERROR reason=unknown-argument, exit 2
```

The cold-cache FAIL is fixed at the monitor layer only; the upstream blocker in
section 7.4 is unchanged, so #185 remains open.
