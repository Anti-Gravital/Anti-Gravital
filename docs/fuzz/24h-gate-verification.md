# 24-hour fuzz gate: verification package (issue #156)

Status: **BLOCKED**. The 24-hour gate has not been executed. This document
defines the exact procedure, the required environment, the artifact paths and
the empty results template. It is not evidence of a pass and must never be
read as such.

## Scope and target count

The gate covers **seven** fuzz targets, not six:

| Target | Invariant |
| --- | --- |
| `fuzz_lexer` | DSL lexer terminates without panic on arbitrary UTF-8 |
| `fuzz_parser` | DSL parser terminates without panic or returns a controlled `Err` |
| `fuzz_compile` | DSL lint + compile + generate pipeline does not panic |
| `fuzz_resp` | RESP2 wire parser (native cache server) does not panic or allocate without bound |
| `fuzz_storage_key` | `validate_key` anti path-traversal boundary does not panic and rejects unsafe keys |
| `fuzz_signed_url` | signed-URL verification does not panic and accepts no false positives |
| `fuzz_workers_payload` | `ag-workers` payload decode turns malformed input into `InvalidPayload`, never a panic |

Earlier audit records (`docs/audits/pre-fase5-fuzzing.md`, release gate
`Fuzz smoke` row) describe six targets because `fuzz_workers_payload` was added
later (RFC-0012 / ADR-0013). All seven are declared in `fuzz/Cargo.toml` and all
seven must run for 24 hours before issue #156 can be closed. A run that omits
`fuzz_workers_payload` does not satisfy the gate.

## Required environment

- Host: dedicated Linux x86-64 machine, quiet (no other heavy load). Record
  CPU model, physical/logical cores, RAM, storage type and CPU governor.
- OS: Linux kernel 6.0 or newer. Record `uname -a` and `/etc/os-release`.
- Toolchain: `fuzz/rust-toolchain.toml` selects `nightly`; the version floats.
  Record the exact resolved versions with `rustc +nightly --version` and
  `cargo fuzz --version` in the report. For stricter reproducibility, install a
  dated toolchain and replace `+nightly` with `+nightly-YYYY-MM-DD` in every
  command below, consistently for all seven targets.
- Workspace state: clean tree at the stabilization commit under test. Record
  `git rev-parse HEAD` and `git status --porcelain`. `fuzz/Cargo.lock` must be
  committed and in sync with `fuzz/Cargo.toml` so the build does not mutate the
  tree mid-gate.
- Duration: `-max_total_time=86400` seconds per target (24 h). Seven targets
  run sequentially require at least 168 h of wall time. Parallel runs are
  acceptable only if each target gets a full 86400 s and CPU/RSS are isolated;
  in that case record the allocation per target, since `exec/s` and RSS are
  resource-sensitive.
- Disk: enough space for seven libFuzzer corpora and any crash artifacts.

## Exact commands

From the repository root, on the commit under test. Replace `YYYY-MM-DD` with
the start date in UTC. Use one directory per gate run; never overwrite a
previous run.

```sh
RUN=docs/fuzz/results/YYYY-MM-DD/24h
mkdir -p "$RUN"

{
  echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "commit=$(git rev-parse HEAD)"
  echo "branch=$(git branch --show-current)"
  echo "dirty_files=$(git status --porcelain | wc -l)"
  echo "rustc_stable=$(rustc --version)"
  echo "rustc_nightly=$(rustc +nightly --version)"
  echo "cargo_fuzz=$(cargo fuzz --version 2>&1)"
  echo "uname=$(uname -a)"
  echo "os=$(. /etc/os-release && echo "$PRETTY_NAME")"
  echo "cpu=$(lscpu | grep '^Model name:' | head -1)"
  echo "logical_cpus=$(nproc)"
  echo "ram=$(free -h | awk '/^Mem:/{print $2}')"
  echo "governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
} | tee "$RUN/metadata.txt"

cd fuzz
set -o pipefail
for t in fuzz_lexer fuzz_parser fuzz_compile fuzz_resp \
         fuzz_storage_key fuzz_signed_url fuzz_workers_payload; do
  echo "START $t $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$RUN/timeline.txt"
  cargo +nightly fuzz run "$t" -- -max_total_time=86400 2>&1 | tee "$RUN/$t.log"
  echo "${PIPESTATUS[0]}" > "$RUN/$t.rc"
  echo "END $t rc=$(cat "$RUN/$t.rc") $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$RUN/timeline.txt"
done
```

The loop keeps each target's real exit code in `$RUN/<target>.rc`. `PIPESTATUS`
requires `bash`. With `set -o pipefail` a failed fuzz target does not abort the
loop unless the script is also run with `-e`; do not use `-e` here, because the
remaining targets still need their evidence. A non-zero `.rc` is a gate failure.

Reproduce any reported crash:

```sh
cd fuzz
cargo +nightly fuzz run <target> artifacts/<target>/<crash-file>
```

## Artifacts to keep for each run

| Artifact | Path | Committed? |
| --- | --- | --- |
| Environment metadata | `docs/fuzz/results/YYYY-MM-DD/24h/metadata.txt` | yes |
| Start/end timeline | `docs/fuzz/results/YYYY-MM-DD/24h/timeline.txt` | yes |
| Full stdout+stderr per target | `docs/fuzz/results/YYYY-MM-DD/24h/<target>.log` | yes |
| Exit code per target | `docs/fuzz/results/YYYY-MM-DD/24h/<target>.rc` | yes |
| Filled results report | `docs/fuzz/results/YYYY-MM-DD/24h/RESULTS.md` (template below) | yes |
| Crash inputs, if any | `fuzz/artifacts/<target>/` copied into `docs/fuzz/results/YYYY-MM-DD/24h/crashes/<target>/` | yes, crash inputs only |
| Corpora | `fuzz/corpus/<target>/` (gitignored) | no; record file count, byte size and optional tarball SHA-256 |

Raw logs are the audit trail: they must contain the final `#N DONE cov: ...`
line and the `Done N runs in M second(s)` line for every target. A log that
ends early (OOM, SIGKILL, host reboot, toolchain error) is not a completed run,
even if no crash was written; restart that target for a full 86400 s.

## Results template (copy to `.../24h/RESULTS.md`)

Fill one block per target. Paste the real final lines from the log; do not
retype or extrapolate numbers.

```markdown
# 24h fuzz gate results — YYYY-MM-DD

- Commit: <git rev-parse HEAD>
- Nightly: <rustc +nightly --version>; cargo-fuzz: <cargo fuzz --version>
- Host: <CPU / cores / RAM / OS / kernel / governor>
- Tree clean at start: <yes/no>
- Overall: <PASS if all seven exit 0 with no crash inputs, otherwise FAIL>

| Target | Start UTC | End UTC | Exit | Executions | Crashes | Artifacts |
| --- | --- | --- | --- | --- | --- | --- |
| fuzz_lexer |  |  |  |  |  |  |
| fuzz_parser |  |  |  |  |  |  |
| fuzz_compile |  |  |  |  |  |  |
| fuzz_resp |  |  |  |  |  |  |
| fuzz_storage_key |  |  |  |  |  |  |
| fuzz_signed_url |  |  |  |  |  |  |
| fuzz_workers_payload |  |  |  |  |  |  |

### <target>

- Command: `cargo +nightly fuzz run <target> -- -max_total_time=86400`
- Log: `<target>.log`; exit code: `<target>.rc`
- Final libFuzzer line (paste): `#N DONE cov: ... ft: ... corp: ... lim: ... exec/s: ... rss: ...`
- Termination line (paste): `Done N runs in M second(s)`
- Corpus: <files / bytes / optional sha256>
- Crash inputs: <none, or paths under crashes/<target>/>
- Notes: <restarts, host events, reduced time, deviations>
```

## Acceptance criteria for issue #156

- [ ] All seven targets have one uninterrupted 86400 s run at the recorded commit.
- [ ] Every `.rc` is 0 and no `crashes/` input exists, or every crash has a fix,
      a regression test and a fresh full-length re-run of the affected target.
- [ ] Logs, metadata, timeline and filled `RESULTS.md` are committed under
      `docs/fuzz/results/YYYY-MM-DD/24h/`.
- [ ] The release gate `Fuzz 24h` row is updated with the date, commit and the
      result path, and only then moved to `pass`.
- [ ] `docs/DEBT.md` DEBT-015 already points to issue #156 (done).
- [ ] README "Known limitations and release blockers" no longer claims the
      gate is unexecuted.

Until every box is checked, issue #156 stays open and labelled `blocked`.

## Short smoke evidence collected in this worktree

A 30-second-per-target smoke was run locally on 2026-11-22 on an
Intel i7-1255U host, not on reference hardware. It verifies that all seven
harnesses build and run; it is **not** the 24-hour gate and no result is
extrapolated from it. See `docs/fuzz/results/2026-11-22-smoke.md`.
