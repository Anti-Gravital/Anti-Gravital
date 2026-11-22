#!/usr/bin/env bash
# Reproducible criterion run for issue #157 on the local host.
#
# This is an informative-hardware run. It is NOT the Stage 6 reference-hardware
# procedure in docs/benchmarks/verification-package.md and its numbers must not
# be published as official Anti-Gravital design-target results.
#
# Run from the repository root. Every knob is configurable via environment:
#   SAMPLE_SIZE=30 WARMUP_TIME=2 MEASUREMENT_TIME=3 OUT_DIR=... ./scripts/run-benchmarks.sh
set -euo pipefail

SAMPLE_SIZE="${SAMPLE_SIZE:-30}"
WARMUP_TIME="${WARMUP_TIME:-2}"
MEASUREMENT_TIME="${MEASUREMENT_TIME:-3}"
OUT_DIR="${OUT_DIR:-docs/benchmarks/raw/$(date -u +%Y-%m-%d)-wave9}"
CRITERION_DIR="${CARGO_TARGET_DIR:-/home/angel-nereira/Github-local-angelnereira/shared-cargo-target}/criterion"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/angel-nereira/Github-local-angelnereira/shared-cargo-target}"
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0

echo "Configuration: sample_size=${SAMPLE_SIZE} warmup=${WARMUP_TIME}s measurement=${MEASUREMENT_TIME}s out=${OUT_DIR}"
mkdir -p "$OUT_DIR"

# 1. Metadata dump (rule 17 fields that are environment-derived).
{
  echo "date_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "commit=$(git rev-parse HEAD)"
  echo "branch=$(git branch --show-current)"
  echo "dirty_files=$(git status --porcelain | wc -l)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "toolchain=$(tr '\n' ' ' < rust-toolchain.toml)"
  echo "uname=$(uname -a)"
  echo "os=$(. /etc/os-release && echo "$PRETTY_NAME")"
  echo "kernel=$(uname -r)"
  echo "cpu=$(lscpu | grep '^Model name:' | head -1 | sed 's/^Model name:[[:space:]]*//')"
  echo "physical_cores=$(lscpu | grep '^Core(s) per socket:' | head -1 | sed 's/.*:[[:space:]]*//')"
  echo "logical_cpus=$(nproc)"
  echo "ram=$(free -h | awk '/^Mem:/{print $2}')"
  echo "ulimit_n=$(ulimit -n)"
  echo "somaxconn=$(sysctl -n net.core.somaxconn 2>/dev/null || echo unknown)"
  echo "governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
  echo "load_tool=none (oha/wrk/bombardier/hey not installed in this environment)"
  echo "release_profile=codegen-units=1,lto=fat (root Cargo.toml)"
  echo "criterion_flags=--sample-size ${SAMPLE_SIZE} --warm-up-time ${WARMUP_TIME} --measurement-time ${MEASUREMENT_TIME} --noplot"
} | tee "$OUT_DIR/metadata.txt"

# 2. Compile check (release/bench profile, no execution).
cargo bench -p ag-core -p ag-workers --no-run 2>&1 | tee "$OUT_DIR/no-run.log"

# 3. The two criterion suites (all seven benchmarks).
cargo bench -p ag-core --bench shield_hello_world -- \
  --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARMUP_TIME" \
  --measurement-time "$MEASUREMENT_TIME" --noplot 2>&1 \
  | tee "$OUT_DIR/ag-core-criterion.log"
cargo bench -p ag-workers --bench queue_throughput -- \
  --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARMUP_TIME" \
  --measurement-time "$MEASUREMENT_TIME" --noplot 2>&1 \
  | tee "$OUT_DIR/ag-workers-criterion.log"

# 4. Standard deviation from criterion estimates.json (rule 17). The printed
#    "time: [a b c]" triplet is a 95% CI of the mean, not a standard deviation.
{
  echo "benchmark,mean_ns,std_dev_ns"
  for b in bare_axum_hello shield_default_hello shield_full_default_hello \
           payload_encode payload_decode enqueue_memory lease_memory_batch_100; do
    python3 - "$b" "$CRITERION_DIR/$b/new/estimates.json" <<'PY'
import json, sys
name, path = sys.argv[1], sys.argv[2]
try:
    d = json.load(open(path))
    mean = d["mean"]["point_estimate"]
    sd = d["std_dev"]["point_estimate"]
    print(f"{name},{mean:.2f},{sd:.2f}")
except Exception as exc:  # noqa: BLE001 - report the gap, never fabricate a value
    print(f"{name},unavailable,unavailable  # {exc}")
PY
  done
} | tee "$OUT_DIR/estimates-summary.csv"

echo "Artifacts written to $OUT_DIR; generated target/criterion is not committed."
