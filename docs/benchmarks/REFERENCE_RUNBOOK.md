# Reference benchmark runbook (issue #157)

Status: **procedure only**. This document is a one-session handoff for a
maintainer who owns a host that matches `docs/benchmarks/reference-host.md`.
It contains no benchmark results. Running steps 1-3 does not close #157 or move
the release gate: steps 4-6 (report, gate row, README) belong to the same
session, and step 7 is the close-out checklist.

Language and repository rules (CLAUDE.md): technical documentation is in
English (ADR-0008); no emojis; no AI-tool attribution or session URLs; commit
subjects and PR titles are at most 256 characters. Rule 17 requires hardware,
OS, Rust version, commit, configuration, methodology, number of runs and
standard deviation. Rule 36 requires that a third party can reproduce the
result. Never publish an informative number as a design-target result, and
never invent a value for an unmeasured scenario.

## 0. Fix the stabilization commit before you start

The runner records `git rev-parse HEAD` in every artifact, and the final report
must cite that same commit. Decide it first.

```sh
git rev-parse HEAD                 # record this exact SHA in the report
git branch --show-current
git status --porcelain             # must print nothing
```

- Run from the repository root. A dirty tree fails the reference gate.
- If a later commit changes performance-relevant code, repeat the whole run;
  a report for an older commit does not close #157 for the new one.
- Use a private `CARGO_TARGET_DIR` on the reference host (for example
  `$PWD/target`) so unrelated builds cannot affect the measurements.

## 1. Preflight

### 1.1 Host profile

Every row is checked by `validate_reference()` in `scripts/run-benchmarks.sh`.
The same table, with the fail-closed semantics, is in
`docs/benchmarks/reference-host.md`.

| Runner label | Required value | Exact check |
| --- | --- | --- |
| `arch` | `x86_64` | `lscpu \| grep '^Architecture'` |
| `cpu_allowlist` | `AMD Ryzen 9 7950X` | `lscpu \| grep '^Model name'` |
| `physical_cores` | exactly 16 | `S=$(lscpu \| awk -F: '/Socket\(s\)/{print $2}'); C=$(lscpu \| awk -F: '/Core\(s\) per socket/{print $2}'); echo $((S*C))` |
| `ram_capacity` | at least 32212254720 bytes | `free -b \| awk '/^Mem:/{print $2}'` |
| `ram_type` | `DDR5` | `sudo -n dmidecode -t 17 \| grep 'Type:'` |
| `kernel` | major version 6 or newer | `uname -r` |
| `cpu_governor` | `performance` | `cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` |
| `load_tool` | `oha` or `wrk`, version recorded | `command -v oha \|\| command -v wrk`; then `<tool> --version` |
| `tooling` | `python3` and `/usr/bin/time` | `command -v python3`; `test -x /usr/bin/time` |
| `rustc_pinned` | equals the pin in `rust-toolchain.toml` | `rustc --version`; `grep channel rust-toolchain.toml` |
| `clean_tree` | 0 dirty files | `git status --porcelain \| wc -l` |
| `quiet_attestation` | `AG_QUIET_HOST_ATTESTED=yes` | human decision, see below |
| `reporter` | a human name, not `unspecified` | `REPORTER` is set for the real run |

"Or equivalent" is deliberately narrow. A different CPU becomes a reference
host only after its exact `lscpu` model string is added to the allowlist in
`scripts/run-benchmarks.sh` and to the table in `docs/benchmarks/reference-host.md`,
with the rationale in the same commit. That is a hardware addition, not a
performance-target change; changing a target requires an RFC (rule 22).

The quiet-host attestation is a human claim about things a script cannot see:
no other heavy load, no thermal throttling, stable power. Write the observed
conditions into the `Deviations and anomalies` section of the final report.
Attesting to a loaded or throttling host is a false claim even if every
automated check passes.

### 1.2 Host preparation

```sh
# As root: fix the governor and confirm no throttling.
cpupower frequency-set -g performance
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

# Memory type and capacity (the runner records dmidecode output).
sudo dmidecode -t 17 | grep -E 'Size|Type:'

# Exactly one load generator; record its version.
oha --version    # or: wrk --version

# Toolchain cross-checks.
python3 --version
/usr/bin/time --version | head -1
rustc --version
```

The automated cold-start figure is wall time from process fork to the first
`ag-core hello_world example listening` tracing event. A separate `/usr/bin/time -v`
launch may be recorded in the report as a manual cross-check, not as an extra
independent run.

### 1.3 Fail-closed rehearsal (the dry run)

Validate syntax, then rehearse the gate on this host. `--dry-run` implies
`--reference`: it forces the official criterion defaults, runs the exact same
profile validation, and prints the commands and artifacts of the real run. It
executes no benchmark and creates no directory, log or CSV.

```sh
bash -n scripts/run-benchmarks.sh
./scripts/run-benchmarks.sh --help
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  ./scripts/run-benchmarks.sh --dry-run
echo "exit=$?"
```

Interpretation:

- Exit 0: every check printed `PASS`, followed by the printed plan. Proceed to
  step 2 with the same environment. A bare `--dry-run` without the attestation
  is also honest: it reports `FAIL quiet_attestation no` and exits 3.
- Exit 3: one or more `FAIL` lines; no plan, no numbers, no artifacts. Do not
  retry with a relaxed profile. Fix the host or stop; #157 stays open.
- Exit 2: bad arguments (for example shortened criterion settings) or not run
  from the repository root. This is a usage error, not a host result.

Confirm the rehearsal wrote nothing before continuing:

```sh
git status --porcelain             # unchanged from step 0
test ! -e docs/benchmarks/raw/$(date -u +%Y-%m-%d)-reference && echo "no artifacts"
```

On a non-reference development host a refusal is expected and is **not** Stage 6
evidence. A real example (i7-1255U notebook, 2026-11-22) is recorded in
`docs/benchmarks/gatebench-informative-2026-11-22.md`.

## 2. Run the reference suite

One command; do not interrupt it and keep the host otherwise idle:

```sh
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  CARGO_TARGET_DIR="$PWD/target" \
  ./scripts/run-benchmarks.sh --reference
```

Optional knobs, all recorded in `metadata.txt`:

```sh
AG_QUIET_HOST_ATTESTED=yes REPORTER="<human name>" \
  OHA_DURATION=30s OHA_CONCURRENCY=100 \
  ./scripts/run-benchmarks.sh --reference --runs 5
```

`--reference` refuses `SAMPLE_SIZE`, `WARMUP_TIME` or `MEASUREMENT_TIME` other
than `100`, `3` and `5`. No flag can make an informative host pass. Default
output: `docs/benchmarks/raw/<YYYY-MM-DD>-reference` (`OUT_DIR` overrides it).

If the run fails after host validation (for example the load tool exits
nonzero), do not hand-edit the partial artifacts and do not call it a pass.
Record the failure, keep #157 open and re-run from a clean tree.

## 3. Collect and verify artifacts

Expected files under `OUT_DIR` (the load-output filename uses the selected
tool, so `wrk` produces `wrk-run-N.txt`):

```text
metadata.txt                 # every rule-17 context field
rule17-summary.txt           # consolidated context, configuration, methodology, runs, std-dev source
no-run.log                   # cargo bench --no-run compile check
ag-core-criterion.log        # three ag-core benchmarks, full criterion output
ag-workers-criterion.log     # four ag-workers benchmarks, full criterion output
estimates-summary.csv        # benchmark, mean_ns, std_dev_ns, sample_size, source
build-hello-world.log        # cargo build --release -p ag-core --example hello_world
binary-size.txt              # stat -c %s of the release example
e2e-runs.csv                 # run, cold_start_ms, idle_rss_kb, load_output path
server-N.log                 # one per independent run
<oha|wrk>-run-N.txt          # one per independent run
```

Verification before writing the report:

```sh
OUT=docs/benchmarks/raw/<YYYY-MM-DD>-reference
find "$OUT" -maxdepth 1 -type f | sort
grep -E 'unavailable|error:' "$OUT/estimates-summary.csv" && echo "INCOMPLETE"
grep -c 'example listening' "$OUT"/server-*.log
wc -l "$OUT/e2e-runs.csv"                      # header + RUNS data rows
grep -E 'reporter=|commit=|mode=reference' "$OUT/rule17-summary.txt"
```

All seven benchmark rows in `estimates-summary.csv` must have numeric mean and
std dev. `e2e-runs.csv` must contain one row per independent run with numeric
cold start and idle RSS. The criterion `time: [a b c]` triplet is a 95% CI of
the mean, **not** a standard deviation; rule 17's standard deviation is
`std_dev.point_estimate` from `target/criterion/<bench>/new/estimates.json`.

Commit the raw directory in the same work as the report. The root `.gitignore`
excludes `*.log` globally but re-includes `docs/benchmarks/raw/**/*.log`, so the
criterion and server logs are committed. Generated `target/criterion/` HTML and
JSON are not committed.

## 4. Curate `docs/benchmarks/pre-fase5-performance.md`

Copy the empty evidence template from the `Evidence template` section of
`docs/benchmarks/reference-host.md` and fill it from the artifacts above:

- Every rule-17 field: reporter, UTC date/time, reference-profile validation
  block, commit, branch/dirty state, hardware, OS, governor, Rust, build
  profile, load tool, exact command, configuration, methodology, number of
  runs, standard-deviation source.
- Criterion results table: all seven benchmarks with mean, std dev, samples,
  95% CI and outliers taken from the logs and `estimates-summary.csv`.
- End-to-end table: throughput, p99, idle RSS, cold start, binary size, CRUD
  PostgreSQL, p99 with DB query and WebSocket connections, with median and
  sample standard deviation computed across the independent runs. A scenario
  that was not executed stays `not measured` with a reason; never fill it from
  an informative run or from a target table.
- `Deviations and anomalies`: the observed quiet-host conditions, any
  throttling or other load, discarded outliers with reasons.
- `Reproducibility`: exact command, commit and artifact paths (rule 36).

Check the result: no empty cells, no `unavailable`, and every `not measured`
cell has an explicit reason.

## 5. Edit the release-gate Benchmarks row

File: `docs/audits/PRE_FASE5_RELEASE_GATE.md`. Current row:

```text
| Benchmarks | pending | final report `docs/benchmarks/pre-fase5-performance.md` not yet written; reproducible procedure in `docs/benchmarks/verification-package.md`; informative host run at commit 7cbc28a on 2026-11-22 in `docs/benchmarks/wave9-host-results.md` (non-reference hardware, does not settle the gate); tracked in issue #157 | yes | 6 |
```

Replace only this row, after steps 1-4 are complete, with one of:

```text
| Benchmarks | pass | report docs/benchmarks/pre-fase5-performance.md; YYYY-MM-DD; commit <SHA>; command `AG_QUIET_HOST_ATTESTED=yes REPORTER="<human>" CARGO_TARGET_DIR="$PWD/target" ./scripts/run-benchmarks.sh --reference`; raw artifacts docs/benchmarks/raw/<YYYY-MM-DD>-reference/ | yes | 6 |
| Benchmarks | fail | same report/date/commit/command; gap: <target, measured value, delta for each unmet target> | yes | 6 |
```

Rules for the edit:

- Do not leave the row `pending` once the evidence exists; do not write `pass`
  if any design target is unmet. An unmet target is `fail` with the measured
  gap, never a silent target change (rules 17 and 22).
- Cite the report path, UTC date, exact commit and exact reproducible command.
- Leave every other gate row untouched. A pass here does not close other gates.
- Commit this edit together with the report and the README edits.

## 6. Reconcile the README in English and Spanish

File: `README.md`. Update both sections in the same commit as the gate row;
English is canonical (ADR-0008). If the section values diverge, Spanish is
marked as pending update.

| Section | EN location | ES location |
| --- | --- | --- |
| "What you cannot do / production-readiness" bullet | lines 140-141 | lines 698-699 |
| "No production certification" bullet | lines 197-199 | lines 762-764 |
| "Known limitations and release blockers" performance bullet | lines 525-526 | lines 1055-1057 |

Current blocker text to replace:

```text
- Final performance evidence on stabilized code is pending; published Phase 1
  and Phase 2 targets have not been met on the recorded hardware.
- Falta la evidencia final de rendimiento sobre codigo estabilizado; los
  targets publicados de las Fases 1 y 2 no se han alcanzado en el hardware
  registrado.
```

Write the outcome from the report without inventing numbers: if the gate is
`pass`, state that the measured evidence exists on the cited commit and point
to the report/gate row; if the gate is `fail`, keep the blocker statement and
name the measured gap. Reconcile the two shorter production-readiness bullets
the same way. Do not claim a target was met because a benchmark ran.

## 7. Evidence checklist to close #157

- [ ] `./scripts/run-benchmarks.sh --reference` exited 0 on the stabilization
      commit; host validation printed all `PASS`; command and commit recorded.
- [ ] Every artifact in step 3 exists in the committed raw directory, with
      numeric std dev for all seven criterion benchmarks and complete e2e rows.
- [ ] `docs/benchmarks/pre-fase5-performance.md` is filled from those artifacts
      with all rule-17 fields, standard deviations and explicit `not measured`
      decisions.
- [ ] The Benchmarks row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` says `pass`
      or `fail` with the measured gap and cites report, date, commit, command.
- [ ] README EN and ES blocker bullets were reconciled in the same commit.
- [ ] `bash -n`, `git diff --check` and the `.github/workflows/docs.yml`
      content scans (AI evidence, descriptor prefixes, emojis, retired brands)
      are clean.
- [ ] Only then: `Closes #157`, remove the `blocked` label, and remove the
      descriptor from `docs/pr-drafts/` when the PR merges (rule: descriptor
      lifetime ends with its PR).

Until every box is checked, #157 stays open with `blocked`, the gate row stays
`pending`, and this branch keeps `Refs #157`.

## 8. Anti-patterns

- Copying numbers from `wave9-host-results.md` or
  `gatebench-informative-2026-11-22.md` into the reference report.
- Relaxing the governor, sample size or allowlist to make the gate pass.
- Attesting a quiet host without observing it, or reporting a cross-check
  launch as an independent run.
- Treating a criterion 95% CI as a standard deviation.
- Changing a performance target to match a measurement (requires an RFC).
- Closing #157 or flipping the gate row before steps 1-6 are verified.
