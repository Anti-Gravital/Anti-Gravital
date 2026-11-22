#!/usr/bin/env bash
# Offline domain-link demonstration for dev, staging and prod.
#
# It renders the Kustomize overlay for each environment from the canonical
# domain manifest, starts the dependency-free mock service on a loopback port
# and proves the custom hostname reaches the right environment with
# `curl --resolve` (a per-request DNS override that needs no root and no
# /etc/hosts change). It also checks that a private service gets no Ingress
# (ER-08). No provider, cluster, daemon or external service is used.
set -uo pipefail

EXAMPLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$EXAMPLE_DIR/../.." && pwd)"
MANIFEST="$EXAMPLE_DIR/domain.yaml"
RENDERER="$REPO_ROOT/deploy/link/render_ingress.py"
SERVER="$EXAMPLE_DIR/target/debug/domain-link-demo"
OUT_DIR="${RENDER_OUT:-/tmp/ag-domain-link-example}"

PASS=0
FAIL=0
SKIP=0
PIDS=()

cleanup() {
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT

report() {
  local state="$1" label="$2" detail="${3:-}"
  if [[ "$state" == "PASS" ]]; then PASS=$((PASS + 1)); fi
  if [[ "$state" == "FAIL" ]]; then FAIL=$((FAIL + 1)); fi
  if [[ "$state" == "SKIP" ]]; then SKIP=$((SKIP + 1)); fi
  printf '%s %s%s\n' "$state" "$label" "${detail:+ -- $detail}"
}

printf 'manifest: %s\n' "$MANIFEST"
printf 'output:   %s\n' "$OUT_DIR"

if [[ ! -f "$MANIFEST" ]]; then
  report FAIL "manifest exists" "$MANIFEST"
  printf 'PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
  exit 1
fi
report PASS "manifest exists"

if ! command -v python3 >/dev/null 2>&1; then
  echo "SKIP python3 missing; cannot read manifest"
  printf 'PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
  exit 0
fi

ENTRIES="$(python3 - "$MANIFEST" <<'PY'
import sys
import yaml

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = yaml.safe_load(handle)
for name in ("dev", "staging", "prod"):
    env = manifest["environments"][name]
    service = next(s for s in env["services"] if s["name"] == "api")
    visibility = service.get("visibility", env["visibility_default"])
    print(f"{name}|{service['host']}|{visibility}")
PY
)" || {
  report FAIL "manifest parse"
  printf 'PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
  exit 1
}
report PASS "manifest parse (3 deployable environments)"

if command -v cargo >/dev/null 2>&1; then
  if cargo build --offline --quiet --manifest-path "$EXAMPLE_DIR/Cargo.toml"; then
    report PASS "cargo build --offline"
  else
    report FAIL "cargo build --offline"
  fi
else
  report SKIP "cargo build (cargo missing)"
fi

if ! command -v curl >/dev/null 2>&1; then
  report SKIP "curl checks (curl missing)"
  printf 'PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
  exit 0
fi

mkdir -p "$OUT_DIR"

while IFS='|' read -r env host visibility; do
  [[ -z "$env" ]] && continue
  case "$env" in
    dev) port=8080 ;;
    staging) port=8081 ;;
    prod) port=8082 ;;
  esac
  env_dir="$OUT_DIR/$env"
  mkdir -p "$env_dir"

  if [[ -f "$RENDERER" ]]; then
    info="$(python3 "$RENDERER" --manifest "$MANIFEST" --env "$env" \
      --service api --out "$env_dir" 2>&1)"
    rc=$?
    if [[ "$rc" -eq 0 && "$info" == *"HOST=$host"* && "$info" == *"INGRESS=true"* ]]; then
      report PASS "render overlay $env ($visibility)" "$host"
    else
      report FAIL "render overlay $env ($visibility)" "$info"
    fi
    printf '  hosts-entry: 127.0.0.1 %s\n' "$host"
  else
    report SKIP "render overlay $env (renderer missing)"
  fi

  if [[ ! -x "$SERVER" ]]; then
    report SKIP "start mock $env (binary missing)"
    continue
  fi

  BIND="127.0.0.1:$port" APP_ENV="$env" "$SERVER" > "$env_dir/server.log" 2>&1 &
  PIDS+=("$!")

  body=""
  for _ in $(seq 1 40); do
    if body="$(curl --resolve "$host:$port:127.0.0.1" -fsS "http://$host:$port/health" 2>/dev/null)"; then
      break
    fi
    sleep 0.1
  done

  if [[ "$body" == *"env=$env"* && "$body" == *"host=$host"* ]]; then
    report PASS "curl --resolve $env" "$body"
  else
    report FAIL "curl --resolve $env" "${body:-no response}"
  fi
done <<< "$ENTRIES"

# ER-08: a private service never gets an Ingress rule, even with a host elsewhere.
if [[ -f "$RENDERER" ]]; then
  private_dir="$OUT_DIR/private-metrics"
  info="$(python3 "$RENDERER" --manifest "$MANIFEST" --env prod \
    --service metrics --out "$private_dir" 2>&1)"
  rc=$?
  if [[ "$rc" -eq 0 && "$info" == *"INGRESS=false"* && "$info" == *"VISIBILITY=private"* ]]; then
    report PASS "private service has no Ingress (ER-08)" "prod/metrics"
  else
    report FAIL "private service has no Ingress (ER-08)" "$info"
  fi
  if command -v kustomize >/dev/null 2>&1; then
    kustomize build "$private_dir" > "$private_dir/rendered.yaml"
    if grep -q '^kind: Ingress' "$private_dir/rendered.yaml"; then
      report FAIL "rendered private overlay has no Ingress (ER-08)"
    else
      report PASS "rendered private overlay has no Ingress (ER-08)"
    fi
  else
    report SKIP "rendered private overlay check (kustomize missing)"
  fi
fi

printf 'PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
exit 0
