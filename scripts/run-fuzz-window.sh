#!/usr/bin/env bash
# Run all seven fuzz targets for a configurable window and append JSONL evidence.
#
# This is an automation aid for the issue #156 gate. A short window is NOT the
# 24-hour gate: the script refuses to label a run shorter than 86400 s as such.
# Even an 86400 s run only reaches "gate duration met, manual review required";
# acceptance (triage, docs, release-gate row) stays a human decision.
#
# Usage:
#   scripts/run-fuzz-window.sh [DURATION_SECONDS]
#
#   DURATION_SECONDS   positive integer, default 300; 86400 is the full gate
#                      duration per target. Passed to libFuzzer as
#                      -max_total_time=<DURATION_SECONDS> for every target.
#
# Environment:
#   FUZZ_WINDOW_SECONDS  same as the positional argument (argument wins).
#   RUN_DIR              output directory (default under docs/fuzz/results).
#   EVIDENCE_FILE        JSONL file to append to (default docs/fuzz/results/windows.jsonl).
#   FUZZ_TOOLCHAIN       toolchain passed to cargo fuzz (default nightly).
#
# Exit status: 0 only if every target exits 0 and no new crash artifact appears.

set -uo pipefail

usage() {
  sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FUZZ_DIR="$REPO_ROOT/fuzz"
TARGETS=(
  fuzz_lexer
  fuzz_parser
  fuzz_compile
  fuzz_resp
  fuzz_storage_key
  fuzz_signed_url
  fuzz_workers_payload
)
FULL_GATE_SECONDS=86400

DURATION="${1:-${FUZZ_WINDOW_SECONDS:-300}}"
if ! [[ "$DURATION" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: DURATION must be a positive integer of seconds, got: $DURATION" >&2
  exit 2
fi
FUZZ_TOOLCHAIN="${FUZZ_TOOLCHAIN:-nightly}"

abs_path() {
  case "$1" in
    /*) printf '%s' "$1" ;;
    *) printf '%s/%s' "$REPO_ROOT" "$1" ;;
  esac
}

UTC_STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_ID="${DURATION}s-${UTC_STAMP}-$$"
RUN_DIR="$(abs_path "${RUN_DIR:-docs/fuzz/results/$(date -u +%Y-%m-%d)/window-$RUN_ID}")"
EVIDENCE_FILE="$(abs_path "${EVIDENCE_FILE:-docs/fuzz/results/windows.jsonl}")"

if [[ -e "$RUN_DIR" ]]; then
  echo "ERROR: RUN_DIR already exists, refusing to overwrite: $RUN_DIR" >&2
  exit 2
fi
mkdir -p "$RUN_DIR/crashes" "$(dirname "$EVIDENCE_FILE")"

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\t'/\\t}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\n'/\\n}"
  printf '%s' "$s"
}

# Environment metadata. Each field is recorded once per run.
COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
BRANCH="$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo unknown)"
DIRTY_FILES="$(git -C "$REPO_ROOT" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
RUSTC_STABLE="$(rustc --version 2>&1)"
RUSTC_NIGHTLY="$(rustc +"$FUZZ_TOOLCHAIN" --version 2>&1)"
CARGO_FUZZ="$(cargo fuzz --version 2>&1)"
UNAME_S="$(uname -a 2>&1)"
OS_PRETTY="$(. /etc/os-release 2>/dev/null && echo "${PRETTY_NAME:-unknown}")"
CPU_MODEL="$(lscpu 2>/dev/null | grep '^Model name:' | head -1 | sed 's/^Model name:[[:space:]]*//')"
CPU_MODEL="${CPU_MODEL:-unknown}"
LOGICAL_CPUS="$(nproc 2>/dev/null || echo unknown)"
RAM="$(free -h 2>/dev/null | awk '/^Mem:/{print $2}' || echo unknown)"
GOVERNOR="$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"

if [[ "$DURATION" -ge "$FULL_GATE_SECONDS" ]]; then
  GATE_LABEL="GATE_DURATION_MET_MANUAL_REVIEW_REQUIRED"
  DURATION_IS_FULL_GATE=true
else
  GATE_LABEL="SHORT_WINDOW_NOT_GATE"
  DURATION_IS_FULL_GATE=false
fi

{
  echo "run_id=$RUN_ID"
  echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "duration_seconds=$DURATION"
  echo "gate_label=$GATE_LABEL"
  echo "commit=$COMMIT"
  echo "branch=$BRANCH"
  echo "dirty_files=$DIRTY_FILES"
  echo "rustc_stable=$RUSTC_STABLE"
  echo "rustc_toolchain=$RUSTC_NIGHTLY"
  echo "cargo_fuzz=$CARGO_FUZZ"
  echo "uname=$UNAME_S"
  echo "os=$OS_PRETTY"
  echo "cpu=$CPU_MODEL"
  echo "logical_cpus=$LOGICAL_CPUS"
  echo "ram=$RAM"
  echo "governor=$GOVERNOR"
} | tee "$RUN_DIR/metadata.txt"

echo "NOTICE: $GATE_LABEL. A window shorter than ${FULL_GATE_SECONDS}s is not the 24h gate."

ALL_ZERO=true
CRASH_JSON_PARTS=()
TARGET_JSON_PARTS=()
TOTAL_CRASHES=0

for t in "${TARGETS[@]}"; do
  STARTED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "START $t $STARTED" | tee -a "$RUN_DIR/timeline.txt"

  ARTIFACT_DIR="$FUZZ_DIR/artifacts/$t"
  BEFORE="$(mktemp)"
  AFTER="$(mktemp)"
  find "$ARTIFACT_DIR" -type f -printf '%p\n' 2>/dev/null | LC_ALL=C sort > "$BEFORE" || true

  LOCK_BEFORE="$(sha256sum "$FUZZ_DIR/Cargo.lock" | awk '{print $1}')"
  if ! cargo metadata --locked --manifest-path "$FUZZ_DIR/Cargo.toml" >/dev/null 2>&1; then
    echo "LOCK_STALE before target $t" | tee -a "$RUN_DIR/$t.log"
  fi
  LOG_REL="docs/fuzz/results/$(basename "$(dirname "$RUN_DIR")")/$(basename "$RUN_DIR")/$t.log"
  (
    cd "$FUZZ_DIR" || exit 127
    cargo +"$FUZZ_TOOLCHAIN" fuzz run "$t" -- -max_total_time="$DURATION"
  ) 2>&1 | tee "$RUN_DIR/$t.log"
  RC="${PIPESTATUS[0]}"
  LOCK_AFTER="$(sha256sum "$FUZZ_DIR/Cargo.lock" | awk '{print $1}')"
  [ "$LOCK_BEFORE" = "$LOCK_AFTER" ] || echo "LOCK_MUTATED during $t" | tee -a "$RUN_DIR/$t.log"

  find "$ARTIFACT_DIR" -type f -printf '%p\n' 2>/dev/null | LC_ALL=C sort > "$AFTER" || true
  NEW_FILES="$(comm -13 "$BEFORE" "$AFTER")"
  rm -f "$BEFORE" "$AFTER"

  THIS_CRASH_JSON=()
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    base="$(basename "$f")"
    mkdir -p "$RUN_DIR/crashes/$t"
    cp -a "$f" "$RUN_DIR/crashes/$t/$base" 2>/dev/null || true
    THIS_CRASH_JSON+=("\"docs/fuzz/results/$(basename "$(dirname "$RUN_DIR")")/$(basename "$RUN_DIR")/crashes/$t/$base\"")
    TOTAL_CRASHES=$((TOTAL_CRASHES + 1))
  done <<< "$NEW_FILES"
  CRASH_JSON="[$(IFS=,; echo "${THIS_CRASH_JSON[*]:-}")]"

  ENDED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ "$RC" -ne 0 ]]; then
    ALL_ZERO=false
  fi
  echo "END $t rc=$RC crashes=${#THIS_CRASH_JSON[@]} $ENDED" | tee -a "$RUN_DIR/timeline.txt"

  DONE_LINE="$(grep -aE '^#[0-9]+[[:space:]]+DONE[[:space:]]' "$RUN_DIR/$t.log" 2>/dev/null | tail -1 || true)"
  TERM_LINE="$(grep -aE 'Done [0-9]+ runs in [0-9]+ second' "$RUN_DIR/$t.log" 2>/dev/null | tail -1 || true)"
  LOG_SHA="$(sha256sum "$RUN_DIR/$t.log" 2>/dev/null | awk '{print $1}' || true)"

  TARGET_JSON_PARTS+=("{\"schema_version\":1,\"record_type\":\"target\",\"run_id\":\"$(json_escape "$RUN_ID")\",\"gate_label\":\"$GATE_LABEL\",\"target\":\"$t\",\"duration_seconds\":$DURATION,\"started_utc\":\"$STARTED\",\"ended_utc\":\"$ENDED\",\"exit_code\":$RC,\"log\":\"$(json_escape "$LOG_REL")\",\"log_sha256\":\"$LOG_SHA\",\"final_done_line\":\"$(json_escape "$DONE_LINE")\",\"termination_line\":\"$(json_escape "$TERM_LINE")\",\"new_crashes\":$CRASH_JSON,\"crash_count\":${#THIS_CRASH_JSON[@]}}")
done

ENDED_RUN="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUN_JSON="{\"schema_version\":1,\"record_type\":\"run\",\"run_id\":\"$(json_escape "$RUN_ID")\",\"gate_label\":\"$GATE_LABEL\",\"duration_is_full_gate\":$DURATION_IS_FULL_GATE,\"started_utc\":\"$UTC_STAMP\",\"ended_utc\":\"$ENDED_RUN\",\"duration_seconds\":$DURATION,\"commit\":\"$(json_escape "$COMMIT")\",\"branch\":\"$(json_escape "$BRANCH")\",\"dirty_files\":$DIRTY_FILES,\"toolchain\":\"$(json_escape "$FUZZ_TOOLCHAIN")\",\"rustc_stable\":\"$(json_escape "$RUSTC_STABLE")\",\"rustc_toolchain\":\"$(json_escape "$RUSTC_NIGHTLY")\",\"cargo_fuzz\":\"$(json_escape "$CARGO_FUZZ")\",\"host\":{\"uname\":\"$(json_escape "$UNAME_S")\",\"os\":\"$(json_escape "$OS_PRETTY")\",\"cpu\":\"$(json_escape "$CPU_MODEL")\",\"logical_cpus\":\"$(json_escape "$LOGICAL_CPUS")\",\"ram\":\"$(json_escape "$RAM")\",\"governor\":\"$(json_escape "$GOVERNOR")\"},\"targets\":[$(printf '"%s",' "${TARGETS[@]}" | sed 's/,$//')],\"all_exit_zero\":$ALL_ZERO,\"total_crashes\":$TOTAL_CRASHES,\"run_dir\":\"$(json_escape "${RUN_DIR#$REPO_ROOT/}")\",\"evidence_file\":\"$(json_escape "${EVIDENCE_FILE#$REPO_ROOT/}")\",\"note\":\"Short windows are smoke evidence only; see docs/fuzz/24h-gate-verification.md for the real gate.\"}"

{
  printf '%s\n' "$RUN_JSON"
  printf '%s\n' "${TARGET_JSON_PARTS[@]}"
} >> "$EVIDENCE_FILE"

echo "Evidence appended to ${EVIDENCE_FILE#$REPO_ROOT/}; artifacts in ${RUN_DIR#$REPO_ROOT/}"
if [[ "$ALL_ZERO" == true && "$TOTAL_CRASHES" -eq 0 ]]; then
  echo "RESULT: OK for a ${DURATION}s window ($GATE_LABEL). Not a release-gate pass."
  exit 0
fi
echo "RESULT: FAIL - one or more targets exited non-zero or wrote a crash artifact." >&2
exit 1
