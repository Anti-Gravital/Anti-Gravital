# Fuzz window JSONL evidence schema (version 1)

`scripts/run-fuzz-window.sh` appends newline-delimited JSON to
`docs/fuzz/results/windows.jsonl`. Each line is an independent object; a run
writes one `run` record followed by one `target` record per fuzz target. The
file is append-only: never rewrite historical records.

This schema describes evidence produced by short windows and by full-length
runs. It does not itself assert that the 24-hour gate passed.

## Common fields

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | integer | Schema version; currently `1`. |
| `record_type` | `"run"` or `"target"` | Record kind. |
| `run_id` | string | Unique id: `<duration>s-<UTC stamp>-<pid>`. Joins records of one invocation. |
| `gate_label` | string | See "Gate labeling" below. |

## `run` record

| Field | Type | Meaning |
| --- | --- | --- |
| `duration_is_full_gate` | boolean | `true` only when requested duration >= 86400 s. |
| `started_utc`, `ended_utc` | string | ISO-8601 UTC timestamps of the whole window. |
| `duration_seconds` | integer | Requested `-max_total_time` per target. |
| `commit`, `branch` | string | Git state at run start. |
| `dirty_files` | integer | `git status --porcelain` line count at run start. |
| `toolchain` | string | Toolchain passed to `cargo fuzz` (default `nightly`). |
| `rustc_stable`, `rustc_toolchain`, `cargo_fuzz` | string | Exact tool versions. |
| `host` | object | `uname`, `os`, `cpu`, `logical_cpus`, `ram`, `governor` as strings. |
| `targets` | array of string | The seven target names the script attempted, in order. |
| `all_exit_zero` | boolean | True only if every attempted target exited 0. |
| `total_crashes` | integer | Total new crash artifacts observed across all targets. |
| `run_dir` | string | Repo-relative directory with metadata, logs, timeline and crashes. |
| `evidence_file` | string | Repo-relative path of this JSONL file. |
| `note` | string | Standing disclaimer about short windows. |

## `target` record

| Field | Type | Meaning |
| --- | --- | --- |
| `target` | string | Fuzz target name. |
| `duration_seconds` | integer | Same value as the parent run. |
| `started_utc`, `ended_utc` | string | ISO-8601 UTC timestamps of this target. |
| `exit_code` | integer | Real exit code of `cargo fuzz run` for this target. |
| `log` | string | Repo-relative path to the full stdout+stderr log. |
| `log_sha256` | string | SHA-256 of the log file, for integrity. |
| `final_done_line` | string | Last libFuzzer `DONE` line, verbatim; empty if absent. |
| `termination_line` | string | `Done N runs in M second(s)` line, verbatim; empty if absent. |
| `new_crashes` | array of string | Repo-relative paths to artifacts newly written during this target. |
| `crash_count` | integer | Length of `new_crashes`. |

## Gate labeling

`gate_label` is one of:

- `SHORT_WINDOW_NOT_GATE` - duration < 86400 s. The script never labels this
  as the gate; it is harness smoke evidence only.
- `GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED` - duration >= 86400 s. The time
  requirement was met, but crash triage, documentation updates and the
  `Fuzz 24h` release-gate row are still a human decision. There is no label
  that claims a passed gate.

A target that cannot run (toolchain error, build failure, non-zero exit) is
recorded with its real non-zero `exit_code` and an empty log or partial log;
it is never recorded as a skip or as a pass.

## Validation

```sh
python3 -c 'import json,sys; [json.loads(l) for l in open(sys.argv[1])]' \
  docs/fuzz/results/windows.jsonl
```

Every line must parse and `record_type` must be `run` or `target`.
