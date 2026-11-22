# Wave 9 fuzz window - real partial evidence (issue #156)

This document records a real 60-second-per-target fuzz window executed on
2026-11-22. It is **not** the 24-hour gate of issue #156 and no 24-hour result
is inferred from it. The gate label written by the automation is
`SHORT_WINDOW_NOT_GATE`.

## Command and automation

Run from the repository root with the shared Cargo target directory:

```sh
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
bash scripts/run-fuzz-window.sh 60
```

The script attempted all seven targets declared in `fuzz/Cargo.toml`, in this
order: `fuzz_lexer`, `fuzz_parser`, `fuzz_compile`, `fuzz_resp`,
`fuzz_storage_key`, `fuzz_signed_url`, `fuzz_workers_payload`. No target was
skipped. Each target ran as:

```sh
cd fuzz && cargo +nightly fuzz run <target> -- -max_total_time=60
```

## Metadata recorded by the script (verbatim)

```
run_id=60s-20261122T130152Z-2555239
duration_seconds=60
gate_label=SHORT_WINDOW_NOT_GATE
commit=7cbc28a35a5b0ceac033fbab4aefb5736d00fd99
branch=community9/qafuzz
dirty_files=3
rustc_stable=rustc 1.99.1 (58cf8f9c3 2026-10-13)
rustc_toolchain=rustc 1.101.0-nightly (080cc2714 2026-10-11)
cargo_fuzz=cargo-fuzz 0.13.2
uname=Linux angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC 7.0.0-44-generic #44-Ubuntu SMP PREEMPT_DYNAMIC Fri Nov  6 02:38:04 UTC 2026 x86_64 GNU/Linux
os=Ubuntu 26.04.1 LTS
cpu=12th Gen Intel(R) Core(TM) i7-1255U
logical_cpus=12
ram=30Gi
governor=powersave
NOTICE: SHORT_WINDOW_NOT_GATE. A window shorter than 86400s is not the 24h gate.
```

The host is the same informative (non-reference) hardware class used for the
30-second smoke in `docs/fuzz/results/2026-11-22-smoke.md`. The three dirty
files at start were the new script, the new JSONL schema document and the stale
`fuzz/Cargo.lock` described below. Core effort was not isolated; `exec/s` and
RSS must not be treated as reference measurements.

## Artifact paths

- Run directory: `docs/fuzz/results/2026-11-22/window-60s-20261122T130152Z-2555239/`
- Environment metadata: `.../metadata.txt`
- Start/end timeline: `.../timeline.txt`
- Full stdout+stderr logs: `.../<target>.log` (seven files, 1.8 MiB total)
- Crash artifacts: `.../crashes/` (empty; zero files)
- JSONL evidence: `docs/fuzz/results/windows.jsonl` (8 records: 1 run + 7 targets)
- JSONL schema: `docs/fuzz/windows-jsonl-schema.md` (schema_version 1)

The root `.gitignore` ignores `*.log`; `docs/fuzz/results/.gitignore`
re-includes log files under this evidence tree so the raw logs are committed,
and `docs/fuzz/results/.gitattributes` excludes only those logs from git's
whitespace check so `git diff --check` stays clean without altering a single
log byte. The recorded log SHA-256 values correspond to the committed files.

Corpora live in the gitignored `fuzz/corpus/<target>/` directories and are not
committed. Observed counts at the end of the window: `fuzz_lexer` 1659 files /
47387 bytes, `fuzz_parser` 1628 / 39959, `fuzz_compile` 1744 / 43537,
`fuzz_resp` 318 / 16407, `fuzz_storage_key` 338 / 59532, `fuzz_signed_url`
59 / 2462, `fuzz_workers_payload` 2859 / 66550.

## Results

All seven targets exited 0 and wrote no new crash artifact. Total executions:
**76,367,567** across the seven 60-second windows. Every log contains both the
final libFuzzer `DONE` line and the `Done N runs in M second(s)` termination
line.

| Target | Start UTC | End UTC | Exit | Runs | Crashes | Log SHA-256 (first 16) |
| --- | --- | --- | --- | --- | --- | --- |
| fuzz_lexer | 2026-11-22T13:01:52Z | 2026-11-22T13:02:54Z | 0 | 273,592 | 0 | d63acb4037aa65ea |
| fuzz_parser | 2026-11-22T13:02:54Z | 2026-11-22T13:03:55Z | 0 | 238,439 | 0 | d732acc7047e0a2c |
| fuzz_compile | 2026-11-22T13:03:55Z | 2026-11-22T13:04:57Z | 0 | 248,923 | 0 | 0a8ca9fdc4bf4e19 |
| fuzz_resp | 2026-11-22T13:04:57Z | 2026-11-22T13:05:59Z | 0 | 1,056,399 | 0 | 7b3bd7d3109a761f |
| fuzz_storage_key | 2026-11-22T13:05:59Z | 2026-11-22T13:07:00Z | 0 | 28,896,132 | 0 | aae1b83e5558034e |
| fuzz_signed_url | 2026-11-22T13:07:00Z | 2026-11-22T13:08:02Z | 0 | 39,539,476 | 0 | 90859e6eb955fdc3 |
| fuzz_workers_payload | 2026-11-22T13:08:02Z | 2026-11-22T13:09:04Z | 0 | 6,115,606 | 0 | dc43b201ae717219 |

Final `DONE` lines, pasted verbatim from the committed logs (tabs preserved as
whitespace):

```
fuzz_lexer:          #273592	DONE   cov: 1887 ft: 7410 corp: 1568/45Kb lim: 191 exec/s: 4485 rss: 328Mb
fuzz_parser:         #238439	DONE   cov: 1950 ft: 7227 corp: 1566/38Kb lim: 149 exec/s: 3908 rss: 309Mb
fuzz_compile:        #248923	DONE   cov: 2559 ft: 8415 corp: 1655/42Kb lim: 149 exec/s: 4080 rss: 313Mb
fuzz_resp:           #1056399	DONE   cov: 662 ft: 1374 corp: 216/13319b lim: 4096 exec/s: 17318 rss: 479Mb
fuzz_storage_key:    #28896132	DONE   cov: 145 ft: 359 corp: 95/7094b lim: 4096 exec/s: 473707 rss: 515Mb
fuzz_signed_url:     #39539476	DONE   cov: 165 ft: 255 corp: 51/1786b lim: 4096 exec/s: 648188 rss: 517Mb
fuzz_workers_payload: #6115606	DONE   cov: 2285 ft: 6113 corp: 2234/60Kb lim: 4096 exec/s: 100255 rss: 316Mb
```

Each log's termination line is `Done N runs in M second(s)` with N equal to the
run count above and M equal to 61. A case-insensitive scan of all seven raw
logs for `crash|panic|ERROR:|leak|oom` returned 0 matching lines in each file.
`fuzz/artifacts/` contained 0 files after the run.

## Finding: stale fuzz lockfile (fixed in this branch)

The `dirty_files=3` count at start included a stale `fuzz/Cargo.lock`: the
workspace crates had been bumped to `0.1.0-alpha.2` and `ag-core` gained new
dependencies, but the fuzz lockfile still pinned `0.0.0` versions. The first
`cargo fuzz run` regenerated it (299 insertions, 19 deletions). This violates
the gate requirement that `fuzz/Cargo.lock` be committed and in sync so the
build does not mutate the tree mid-gate.

Action taken: the regenerated lockfile is committed on this branch. A follow-up
1-second window (temporary output outside the repository) confirmed the lock
SHA-256 stayed `6426df12e74e14bc853ba1862cf7c0800b4561282dffca7ee5db6d6037b801ec`
before and after a full seven-target invocation. The 24-hour run must start
from a commit that includes this synchronized lockfile and a clean tree.

Note: `cargo metadata --manifest-path fuzz/Cargo.toml --locked --offline`
cannot complete in this environment because the unrelated wasm dependency
`js-sys v0.3.99` is not cached; this does not affect the seven Linux targets,
which all built and ran.

## What this proves and what it does not

Proves: all seven harnesses build and run at commit
`7cbc28a35a5b0ceac033fbab4aefb5736d00fd99`, each for 60 seconds, with exit 0,
no crash artifact and no panic/crash indicator in the logs. The automation
records real exit codes, per-target logs with integrity hashes, host metadata
and a machine-readable JSONL trail, and it refuses to call a short run the gate.

Does not prove: 24-hour coverage, deep-state bug freedom, or any release-gate
row. The `Fuzz 24h` row in `docs/audits/PRE_FASE5_RELEASE_GATE.md` remains
`pending` and issue #156 remains open with `Refs #156`.

## Exact condition to close issue #156

All of the following must hold (see `docs/fuzz/24h-gate-verification.md`):

1. At the recorded clean-tree commit, each of the seven targets has one
   uninterrupted run with `-max_total_time=86400` on Linux x86-64 reference
   hardware; no target may be omitted, in particular `fuzz_workers_payload`.
2. Every `.rc` is 0 and no `crashes/` input exists, or every crash has a fix,
   a regression test and a fresh full-length re-run of the affected target.
3. Raw logs (containing the final `DONE` and `Done N runs` lines), metadata,
   timeline, crash triage and a filled `RESULTS.md` are committed under
   `docs/fuzz/results/YYYY-MM-DD/24h/`; the JSONL records are appended with
   `gate_label=GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED`.
4. The `Fuzz 24h` row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` is updated with
   the date, commit and result path and only then moved to `pass`.
5. README "Known limitations and release blockers" no longer claims the gate
   was not executed. `docs/DEBT.md` DEBT-015 already points to issue #156.

Until every condition is met, the issue stays open and labelled `blocked`.
