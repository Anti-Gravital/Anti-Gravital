#!/usr/bin/env bash
# Reproducible benchmark runner for issue #157 (Stage 6 of the pre-Phase-5 gate).
#
# Two modes:
#   informative (default): reduced samples, no host gating. Every number is
#     labelled informative hardware; none may settle the design-target table.
#   --reference: fail-closed host validation, official criterion defaults and
#     the end-to-end load phase. Only this mode may feed the release gate.
#
# A single invocation writes every CLAUDE.md rule-17 context field to
# metadata.txt and rule17-summary.txt, and the estimates.json mean/std dev to
# estimates-summary.csv. Run from the repository root. The reference profile,
# required tooling and the empty evidence template live in
# docs/benchmarks/reference-host.md.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-benchmarks.sh [--reference] [--runs N] [--help]

  --reference   Reference-hardware mode. Validates the host first and refuses
                to start when any profile check fails. Forces criterion
                defaults (100 samples, 3 s warm-up, 5 s measurement) and runs
                the oha/wrk end-to-end phase with the server restarted between
                independent runs.
  --runs N      Independent end-to-end runs in --reference mode (default 3).
  --help        Show this text.

Environment (informative mode):
  SAMPLE_SIZE, WARMUP_TIME, MEASUREMENT_TIME, RUNS, REPORTER, OUT_DIR,
  CARGO_TARGET_DIR, OHA_DURATION, OHA_CONCURRENCY.

Environment (reference mode):
  AG_QUIET_HOST_ATTESTED=yes  required human attestation of a quiet host.
  REPORTER=<human name>       required, recorded in the rule-17 summary.
  OUT_DIR, CARGO_TARGET_DIR, OHA_DURATION, OHA_CONCURRENCY as above.
USAGE
}

REFERENCE=0
RUNS="${RUNS:-3}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --reference)
      REFERENCE=1
      shift
      ;;
    --runs)
      if [[ $# -lt 2 ]]; then
        echo "ERROR: --runs needs a positive integer." >&2
        exit 2
      fi
      RUNS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! "$RUNS" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: RUNS must be a positive integer." >&2
  exit 2
fi

# Repository guard: all relative paths below assume the repository root.
if [[ ! -f rust-toolchain.toml || ! -e .git ]]; then
  echo "ERROR: run this script from the repository root." >&2
  exit 2
fi

MODE=informative
if [[ "$REFERENCE" -eq 1 ]]; then
  MODE=reference
  # Shortened official runs are refused, never silently ignored or relabelled.
  if [[ -n "${SAMPLE_SIZE:-}" && "${SAMPLE_SIZE}" != "100" ]] ||
     [[ -n "${WARMUP_TIME:-}" && "${WARMUP_TIME}" != "3" ]] ||
     [[ -n "${MEASUREMENT_TIME:-}" && "${MEASUREMENT_TIME}" != "5" ]]; then
    echo "ERROR: --reference requires SAMPLE_SIZE=100 WARMUP_TIME=3 MEASUREMENT_TIME=5." >&2
    exit 2
  fi
  SAMPLE_SIZE=100
  WARMUP_TIME=3
  MEASUREMENT_TIME=5
  if [[ "${AG_QUIET_HOST_ATTESTED:-no}" != "yes" ]]; then
    echo "ERROR: --reference needs AG_QUIET_HOST_ATTESTED=yes (quiet, non-throttling host)." >&2
    exit 2
  fi
  if [[ -z "${REPORTER:-}" || "${REPORTER}" == "unspecified" ]]; then
    echo "ERROR: --reference needs REPORTER=<human name> for rule-17 traceability." >&2
    exit 2
  fi
else
  SAMPLE_SIZE="${SAMPLE_SIZE:-30}"
  WARMUP_TIME="${WARMUP_TIME:-2}"
  MEASUREMENT_TIME="${MEASUREMENT_TIME:-3}"
  REPORTER="${REPORTER:-unspecified}"
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/angel-nereira/Github-local-angelnereira/shared-cargo-target}"
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
CRITERION_DIR="$CARGO_TARGET_DIR/criterion"
DATE_UTC="$(date -u +%Y-%m-%d)"
OUT_DIR="${OUT_DIR:-docs/benchmarks/raw/${DATE_UTC}-${MODE}}"
OHA_DURATION="${OHA_DURATION:-30s}"
OHA_CONCURRENCY="${OHA_CONCURRENCY:-100}"
TIME_BIN=/usr/bin/time

# ---------------------------------------------------------------------------
# Host facts. Collected before the reference gate so the same values feed both
# metadata.txt and the fail-closed validation.
# ---------------------------------------------------------------------------
lscpu_field() {
  lscpu | awk -F: -v key="$1" '$1 == key { gsub(/^[[:space:]]+/, "", $2); print $2; exit }' || true
}

ram_type() {
  local dmi=""
  if command -v dmidecode >/dev/null 2>&1; then
    dmi="$(dmidecode -t 17 2>/dev/null || sudo -n dmidecode -t 17 2>/dev/null || true)"
  fi
  printf '%s\n' "$dmi" | awk -F: '
    $1 ~ /^[[:space:]]*Type$/ {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2)
      if ($2 != "" && $2 != "Other" && $2 != "Unknown") { print $2; exit }
    }' || true
}

DATE_ISO="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
COMMIT="$(git rev-parse HEAD)"
BRANCH="$(git branch --show-current)"
DIRTY="$(git status --porcelain | wc -l | tr -d ' ')"
RUSTC="$(rustc --version)"
CARGO="$(cargo --version)"
TOOLCHAIN="$(tr '\n' ' ' < rust-toolchain.toml)"
RUSTC_NUM="$(printf '%s' "$RUSTC" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)"
TOOLCHAIN_NUM="$(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' rust-toolchain.toml | head -1 || true)"
UNAME_S="$(uname -a)"
OS_NAME="$(. /etc/os-release && printf '%s' "$PRETTY_NAME")"
KERNEL="$(uname -r)"
KERNEL_MAJOR="${KERNEL%%.*}"
ARCH="$(lscpu_field 'Architecture')"
CPU_MODEL="$(lscpu_field 'Model name')"
SOCKETS="$(lscpu_field 'Socket(s)')"
CORES_PER_SOCKET="$(lscpu_field 'Core(s) per socket')"
PHYSICAL_CORES=$(( SOCKETS * CORES_PER_SOCKET ))
LOGICAL_CPUS="$(nproc)"
CPU_MAX_MHZ="$(lscpu_field 'CPU max MHz')"
RAM_HUMAN="$(free -h | awk '/^Mem:/{print $2}')"
RAM_BYTES="$(free -b | awk '/^Mem:/{print $2}')"
RAM_KIND="$(ram_type)"
RAM_KIND="${RAM_KIND:-unknown}"
STORAGE="$(lsblk -d -n -o MODEL,ROTA -e 7 2>/dev/null | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | tr '\n' ';' | sed 's/;$//' || true)"
STORAGE="${STORAGE:-unknown}"
ULIMIT_N="$(ulimit -n)"
SOMAXCONN="$(sysctl -n net.core.somaxconn 2>/dev/null || echo unknown)"
GOVERNOR="$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"

LOAD_TOOL=none
LOAD_TOOL_VERSION=unknown
if command -v oha >/dev/null 2>&1; then
  LOAD_TOOL=oha
  LOAD_TOOL_VERSION="$(oha --version 2>&1 | head -1 || true)"
elif command -v wrk >/dev/null 2>&1; then
  LOAD_TOOL=wrk
  LOAD_TOOL_VERSION="$(wrk --version 2>&1 | head -1 || true)"
fi

# ---------------------------------------------------------------------------
# Fail-closed reference gate. The profile is defined in
# docs/benchmarks/reference-host.md; every item below is checked there. An
# equivalence claims must be recorded in that document and in the allowlist.
# ---------------------------------------------------------------------------
validate_reference() {
  local failures=0
  local ok cpu_ok ram_ok kernel_ok gov_ok tool_ok tc_ok dirty_ok time_ok
  check() {
    local label="$1" actual="$2" result="$3"
    if [[ "$result" == "yes" ]]; then
      printf 'PASS  %-22s %s\n' "$label" "$actual"
    else
      printf 'FAIL  %-22s %s\n' "$label" "$actual"
      failures=$(( failures + 1 ))
    fi
  }

  ok=no
  [[ "$ARCH" == "x86_64" ]] && ok=yes
  check arch "$ARCH" "$ok"

  cpu_ok=no
  case "$CPU_MODEL" in
    *"AMD Ryzen 9 7950X"*) cpu_ok=yes ;;
  esac
  check cpu_allowlist "$CPU_MODEL" "$cpu_ok"

  ok=no
  [[ "$PHYSICAL_CORES" =~ ^[0-9]+$ && "$PHYSICAL_CORES" -eq 16 ]] && ok=yes
  check physical_cores "$PHYSICAL_CORES" "$ok"

  ram_ok=no
  if [[ "$RAM_BYTES" =~ ^[0-9]+$ ]] && (( RAM_BYTES >= 32212254720 )); then
    ram_ok=yes
  fi
  check ram_capacity "$RAM_HUMAN" "$ram_ok"
  ok=no
  [[ "$RAM_KIND" == "DDR5" ]] && ok=yes
  check ram_type "$RAM_KIND" "$ok"

  kernel_ok=no
  if [[ "$KERNEL_MAJOR" =~ ^[0-9]+$ ]] && (( KERNEL_MAJOR >= 6 )); then
    kernel_ok=yes
  fi
  check kernel "$KERNEL" "$kernel_ok"

  gov_ok=no
  [[ "$GOVERNOR" == "performance" ]] && gov_ok=yes
  check cpu_governor "$GOVERNOR" "$gov_ok"

  tool_ok=no
  [[ "$LOAD_TOOL" != "none" ]] && tool_ok=yes
  check load_tool "$LOAD_TOOL $LOAD_TOOL_VERSION" "$tool_ok"

  ok=no
  if command -v python3 >/dev/null 2>&1 && [[ -x "$TIME_BIN" ]]; then
    ok=yes
  fi
  check tooling "python3 + $TIME_BIN" "$ok"
  time_ok="$ok"

  tc_ok=no
  if [[ -n "$TOOLCHAIN_NUM" && "$RUSTC_NUM" == "$TOOLCHAIN_NUM" ]]; then
    tc_ok=yes
  fi
  check rustc_pinned "$RUSTC (pin $TOOLCHAIN_NUM)" "$tc_ok"

  dirty_ok=no
  [[ "$DIRTY" == "0" ]] && dirty_ok=yes
  check clean_tree "$DIRTY dirty files" "$dirty_ok"

  check quiet_attestation "${AG_QUIET_HOST_ATTESTED:-no}" yes

  if [[ $failures -ne 0 || "$time_ok" != "yes" ]]; then
    echo "ERROR: $failures reference-profile check(s) failed; refusing to label this run as reference." >&2
    echo "No numbers were produced. See docs/benchmarks/reference-host.md." >&2
    exit 3
  fi
  echo "Reference profile validated ($failures failures)."
}

if [[ "$MODE" == "reference" ]]; then
  validate_reference
fi

# ---------------------------------------------------------------------------
# Artifacts. Written after validation so a refused reference run leaves no
# misleading output behind.
# ---------------------------------------------------------------------------
echo "Mode=${MODE}; configuration: sample_size=${SAMPLE_SIZE} warmup=${WARMUP_TIME}s measurement=${MEASUREMENT_TIME}s runs=${RUNS} out=${OUT_DIR}"
mkdir -p "$OUT_DIR"

{
  echo "date_utc=$DATE_ISO"
  echo "mode=$MODE"
  echo "reporter=$REPORTER"
  echo "commit=$COMMIT"
  echo "branch=$BRANCH"
  echo "dirty_files_at_start=$DIRTY"
  echo "rustc=$RUSTC"
  echo "cargo=$CARGO"
  echo "toolchain=$TOOLCHAIN"
  echo "uname=$UNAME_S"
  echo "os=$OS_NAME"
  echo "kernel=$KERNEL"
  echo "architecture=$ARCH"
  echo "cpu=$CPU_MODEL"
  echo "sockets=$SOCKETS"
  echo "cores_per_socket=$CORES_PER_SOCKET"
  echo "physical_cores=$PHYSICAL_CORES"
  echo "logical_cpus=$LOGICAL_CPUS"
  echo "cpu_max_mhz=$CPU_MAX_MHZ"
  echo "ram=$RAM_HUMAN"
  echo "ram_bytes=$RAM_BYTES"
  echo "ram_type=$RAM_KIND"
  echo "storage=$STORAGE"
  echo "ulimit_n=$ULIMIT_N"
  echo "somaxconn=$SOMAXCONN"
  echo "governor=$GOVERNOR"
  echo "load_tool=$LOAD_TOOL"
  echo "load_tool_version=$LOAD_TOOL_VERSION"
  echo "release_profile=codegen-units=1,lto=fat,panic=abort,strip=symbols,opt-level=3 (root Cargo.toml)"
  echo "configuration=sample_size=${SAMPLE_SIZE},warmup_s=${WARMUP_TIME},measurement_s=${MEASUREMENT_TIME},e2e_runs=${RUNS},oha_duration=${OHA_DURATION},oha_concurrency=${OHA_CONCURRENCY}"
  echo "criterion_flags=--sample-size ${SAMPLE_SIZE} --warm-up-time ${WARMUP_TIME} --measurement-time ${MEASUREMENT_TIME} --noplot"
  echo "cargo_target_dir=$CARGO_TARGET_DIR"
  echo "quiet_host_attested=${AG_QUIET_HOST_ATTESTED:-not-required-informative}"
} | tee "$OUT_DIR/metadata.txt"

# Compile check (release/bench profile, no execution).
cargo bench --locked -p ag-core -p ag-workers --no-run 2>&1 | tee "$OUT_DIR/no-run.log"

# Criterion suites (all seven benchmarks).
cargo bench --locked -p ag-core --bench shield_hello_world -- \
  --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARMUP_TIME" \
  --measurement-time "$MEASUREMENT_TIME" --noplot 2>&1 \
  | tee "$OUT_DIR/ag-core-criterion.log"
cargo bench --locked -p ag-workers --bench queue_throughput -- \
  --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARMUP_TIME" \
  --measurement-time "$MEASUREMENT_TIME" --noplot 2>&1 \
  | tee "$OUT_DIR/ag-workers-criterion.log"

# Rule 17: the printed "time: [a b c]" triplet is a 95% CI of the mean, not a
# standard deviation. Read the std dev from criterion's estimates.json instead.
{
  echo "benchmark,mean_ns,std_dev_ns,sample_size,criterion_source"
  for b in bare_axum_hello shield_default_hello shield_full_default_hello \
           payload_encode payload_decode enqueue_memory lease_memory_batch_100; do
    python3 - "$b" "$CRITERION_DIR/$b/new/estimates.json" "$SAMPLE_SIZE" <<'PY'
import json, sys
name, path, samples = sys.argv[1], sys.argv[2], sys.argv[3]
try:
    with open(path, encoding="utf-8") as handle:
        data = json.load(handle)
    mean = data["mean"]["point_estimate"]
    std_dev = data["std_dev"]["point_estimate"]
    print(f"{name},{mean:.2f},{std_dev:.2f},{samples},new/estimates.json")
except Exception as exc:  # report the gap; never fabricate a value
    print(f"{name},unavailable,unavailable,{samples},error: {exc}")
PY
  done
} | tee "$OUT_DIR/estimates-summary.csv"

# End-to-end phase. Only reference mode is allowed to collect throughput, p99,
# idle RSS and cold start, and only when a load generator passed the gate.
run_e2e() {
  cargo build --locked --release -p ag-core --example hello_world 2>&1 \
    | tee "$OUT_DIR/build-hello-world.log"
  local bin="$CARGO_TARGET_DIR/release/examples/hello_world"
  if [[ ! -x "$bin" ]]; then
    echo "ERROR: release example not found at $bin" >&2
    exit 4
  fi
  stat -c %s "$bin" > "$OUT_DIR/binary-size.txt"

  local load_cmd
  if [[ "$LOAD_TOOL" == "oha" ]]; then
    load_cmd=(oha -z "$OHA_DURATION" -c "$OHA_CONCURRENCY" http://127.0.0.1:8080/)
  else
    load_cmd=(wrk -d "$OHA_DURATION" -c "$OHA_CONCURRENCY" http://127.0.0.1:8080/)
  fi

  {
    echo "run,cold_start_ms,idle_rss_kb,load_output"
    local i start_ns ready_ns pid rss_kb cold_ms attempt
    for i in $(seq 1 "$RUNS"); do
      : > "$OUT_DIR/server-$i.log"
      start_ns="$(date +%s%N)"
      "$bin" > "$OUT_DIR/server-$i.log" 2>&1 &
      pid=$!
      ready_ns=""
      for attempt in $(seq 1 120); do
        if grep -q "example listening" "$OUT_DIR/server-$i.log" 2>/dev/null; then
          ready_ns="$(date +%s%N)"
          break
        fi
        if ! kill -0 "$pid" 2>/dev/null; then
          break
        fi
        sleep 0.5
      done
      if [[ -z "$ready_ns" ]]; then
        echo "$i,unavailable,unavailable,server_did_not_start"
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        continue
      fi
      cold_ms=$(( (ready_ns - start_ns) / 1000000 ))
      sleep 5
      rss_kb="$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
      rss_kb="${rss_kb:-unavailable}"
      if ! "${load_cmd[@]}" > "$OUT_DIR/oha-run-$i.txt" 2>&1; then
        echo "WARNING: load tool exited nonzero on run $i" >&2
      fi
      echo "$i,$cold_ms,$rss_kb,$OUT_DIR/oha-run-$i.txt"
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    done
  } | tee "$OUT_DIR/e2e-runs.csv"
}

if [[ "$MODE" == "reference" ]]; then
  run_e2e
fi

# Consolidated rule-17 summary. Curating the report
# (docs/benchmarks/pre-fase5-performance.md) remains a human step: raw oha
# output is never reduced to a number by this script.
E2E_ARTIFACTS=""
if [[ "$MODE" == "reference" ]]; then
  E2E_ARTIFACTS=" build-hello-world.log binary-size.txt e2e-runs.csv server-N.log oha-run-N.txt"
fi
{
  echo "# Rule 17 evidence summary"
  echo "generated_at=$DATE_ISO"
  echo "mode=$MODE"
  echo "reporter=$REPORTER"
  echo "commit=$COMMIT"
  echo "branch=$BRANCH"
  echo "dirty_files_at_start=$DIRTY"
  echo "hardware=$ARCH $CPU_MODEL; sockets=$SOCKETS; cores_per_socket=$CORES_PER_SOCKET; physical_cores=$PHYSICAL_CORES; logical_cpus=$LOGICAL_CPUS; cpu_max_mhz=$CPU_MAX_MHZ; ram=$RAM_HUMAN ($RAM_KIND); storage=$STORAGE"
  echo "os=$OS_NAME; kernel=$KERNEL; ulimit_n=$ULIMIT_N; somaxconn=$SOMAXCONN; governor=$GOVERNOR"
  echo "rustc=$RUSTC; cargo=$CARGO; toolchain=$TOOLCHAIN"
  echo "configuration=release profile (codegen-units=1,lto=fat); sample_size=$SAMPLE_SIZE; warmup_s=$WARMUP_TIME; measurement_s=$MEASUREMENT_TIME; e2e_runs=$RUNS; load_tool=$LOAD_TOOL $LOAD_TOOL_VERSION"
  echo "number_of_runs=criterion samples=$SAMPLE_SIZE per benchmark; independent e2e runs=$RUNS"
  if [[ "$MODE" == "reference" ]]; then
    echo "methodology=criterion defaults (100 samples, 3 s warm-up, 5 s measurement); oha/wrk ${OHA_DURATION} at concurrency ${OHA_CONCURRENCY}, server restarted between runs; idle RSS 5 s after start with no traffic; cold start wall time from fork to first listening event; median and sample standard deviation to be curated by the reporter"
  else
    echo "methodology=criterion at reduced samples for harness sanity; end-to-end phase not executed; informative hardware only, no design-target claim"
  fi
  echo "standard_deviation_source=std_dev.point_estimate from target/criterion/<bench>/new/estimates.json (see estimates-summary.csv); the printed time triplet is a 95% CI of the mean"
  echo "artifacts=metadata.txt no-run.log ag-core-criterion.log ag-workers-criterion.log estimates-summary.csv${E2E_ARTIFACTS}"
} | tee "$OUT_DIR/rule17-summary.txt"

echo "Artifacts written to $OUT_DIR; generated target/criterion is not committed."
