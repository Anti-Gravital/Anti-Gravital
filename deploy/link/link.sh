#!/usr/bin/env bash
# One-command wrapper: domain manifest -> rendered Kubernetes artifacts + URL.
# Contract: docs/dns/ENVIRONMENT_RULES.md (ER-01..ER-11). No network, no cluster
# and no live DNS provider are contacted. If kustomize is absent the render
# step is an honest SKIP, never a fake PASS.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: bash deploy/link/link.sh --manifest FILE --env NAME [--service NAME] --out DIR

  --manifest    Domain manifest (YAML, schema_version 1).
  --env         dev | staging | prod (deployable) or local (loopback only).
  --service     Service name in the environment (default: api).
  --tls-secret  Override the derived Ingress TLS Secret name (optional).
  --out         Directory for the generated overlay and rendered.yaml.

Examples:
  bash deploy/link/link.sh --manifest examples/domain-link/domain.yaml \
    --env prod --service api --out /tmp/ag-domain-link/prod
  bash deploy/link/link.sh --manifest examples/domain-link/domain.yaml \
    --env local --out /tmp/ag-domain-link/local
EOF
}

MANIFEST=""
ENV_NAME=""
SERVICE_NAME="api"
TLS_SECRET=""
OUT_DIR_ARG=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest) MANIFEST="${2:-}"; shift 2 ;;
    --env) ENV_NAME="${2:-}"; shift 2 ;;
    --service) SERVICE_NAME="${2:-}"; shift 2 ;;
    --tls-secret) TLS_SECRET="${2:-}"; shift 2 ;;
    --out) OUT_DIR_ARG="${2:-}"; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "$MANIFEST" || -z "$ENV_NAME" || -z "$OUT_DIR_ARG" ]]; then
  usage >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RENDERER="$REPO_ROOT/deploy/link/render_ingress.py"

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 is required to read the domain manifest" >&2
  exit 2
fi

mkdir -p "$OUT_DIR_ARG"
OUT_DIR="$(cd "$OUT_DIR_ARG" && pwd)"

EXTRA=()
if [[ -n "$TLS_SECRET" ]]; then
  EXTRA+=(--tls-secret "$TLS_SECRET")
fi

INFO="$(python3 "$RENDERER" --manifest "$MANIFEST" --env "$ENV_NAME" \
  --service "$SERVICE_NAME" --out "$OUT_DIR" "${EXTRA[@]}")"
printf '%s\n' "$INFO"

HOST="$(printf '%s\n' "$INFO" | sed -n 's/^HOST=//p')"
TLS="$(printf '%s\n' "$INFO" | sed -n 's/^TLS=//p')"
INGRESS="$(printf '%s\n' "$INFO" | sed -n 's/^INGRESS=//p')"

if command -v kustomize >/dev/null 2>&1; then
  kustomize build "$OUT_DIR" > "$OUT_DIR/rendered.yaml"
  echo "RENDERED=$OUT_DIR/rendered.yaml"
  if [[ "$INGRESS" == "true" ]]; then
    if grep -q "host: $HOST" "$OUT_DIR/rendered.yaml"; then
      echo "HOST_OK=true"
    else
      echo "HOST_OK=false"
      exit 1
    fi
    if [[ "$TLS" == "true" ]]; then
      if grep -q "secretName:" "$OUT_DIR/rendered.yaml"; then
        echo "TLS_OK=true"
      else
        echo "TLS_OK=false"
        exit 1
      fi
    else
      echo "TLS_OK=n/a"
    fi
  else
    if grep -q '^kind: Ingress' "$OUT_DIR/rendered.yaml"; then
      echo "INGRESS_OK=false"
      exit 1
    fi
    echo "INGRESS_OK=true (no Ingress for private/local, ER-08/ER-02a)"
    echo "HOST_OK=n/a"
    echo "TLS_OK=n/a"
  fi
else
  echo "KUSTOMIZE=SKIP"
fi
