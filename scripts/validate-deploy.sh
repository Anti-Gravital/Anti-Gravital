#!/usr/bin/env bash
# Validates the wave-7 deployment artifacts and the documented `ag deploy`
# behaviour with real tools. The script is read-only with respect to the
# artifacts: it only writes to a temporary work root.
#
# Checks, each reported as PASS, FAIL or SKIP with its own exit code:
#   1. every YAML file under DEPLOY_ROOT parsed with PyYAML (real parser);
#   2. every Dockerfile declares a non-root USER in its final stage;
#   3. every service Dockerfile declares a HEALTHCHECK in its final stage;
#   4. `ag deploy plan` emits a plan id and writes no artifacts;
#   5. `ag deploy apply` refuses to run without a plan;
#   6. `ag deploy status` works without Docker or a Kubernetes cluster;
#   7. kustomize renders every kustomization under DEPLOY_ROOT (when installed)
#      and kubeconform validates k8s manifests (when installed);
#   8. docker build / kubectl server-side dry-run (only when tooling is live).
#
# Rules: a missing artifact is FAIL; an unavailable tool is an honest SKIP.
# A SKIP never counts as a PASS. The script exits non-zero on any FAIL.
#
# Environment:
#   DEPLOY_ROOT   artifact tree to validate (default: <repo>/deploy)
#   AG_BIN        explicit path to the ag binary (auto-detected otherwise)
#   PROJECT_DIR   project to use for CLI checks (default: examples/todo-api)
#   AG_DEPLOY_TARGETS  plan targets to verify (default: docker k8s)
#   KEEP_WORK=1   keep the temporary work root and print its path

set -uo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

DEPLOY_ROOT="${DEPLOY_ROOT:-$REPO_ROOT/deploy}"
AG_BIN="${AG_BIN:-}"
PROJECT_DIR="${PROJECT_DIR:-}"
AG_DEPLOY_TARGETS="${AG_DEPLOY_TARGETS:-docker k8s}"
KEEP_WORK="${KEEP_WORK:-0}"

PASS=0
FAIL=0
SKIP=0
WORK_ROOT=""

usage() {
    cat <<'EOF'
Usage: scripts/validate-deploy.sh [options]

Options:
  -h, --help   Show this help.

Environment:
  DEPLOY_ROOT  Artifact tree (default: <repo>/deploy).
  AG_BIN       Explicit ag binary path; auto-detected when unset.
  PROJECT_DIR  Project used for CLI checks (default: examples/todo-api).
  AG_DEPLOY_TARGETS  Plan targets (default: docker k8s).
  KEEP_WORK=1  Keep the temporary work root for debugging.

Exit code: 0 when no FAIL, 1 when at least one FAIL, 2 on a harness error.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        *) printf 'Unknown argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
done

# Emit one machine-readable result line and update the counters.
report() {
    local status="$1" name="$2" rc="$3" detail="$4"
    printf '[%s] %s (rc=%s): %s\n' "$status" "$name" "$rc" "$detail"
    case "$status" in
        PASS) PASS=$((PASS + 1)) ;;
        FAIL) FAIL=$((FAIL + 1)) ;;
        SKIP) SKIP=$((SKIP + 1)) ;;
    esac
}

cleanup() {
    if [ -n "$WORK_ROOT" ] && [ -d "$WORK_ROOT" ]; then
        if [ "$KEEP_WORK" = "1" ]; then
            printf 'Work root kept at: %s\n' "$WORK_ROOT"
        else
            rm -rf -- "$WORK_ROOT"
        fi
    fi
}
trap cleanup EXIT

WORK_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/ag-deploy-validation.XXXXXX") || {
    printf 'ERROR: could not create a temporary work root\n' >&2
    exit 2
}

# --- environment header (not a step) ---------------------------------------
COMMIT=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || printf 'unknown')
printf 'host: %s\n' "$(hostname 2>/dev/null || printf 'unknown')"
printf 'date_utc: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
printf 'commit: %s\n' "$COMMIT"
printf 'deploy_root: %s\n' "$DEPLOY_ROOT"

PYTHON=""
if command -v python3 >/dev/null 2>&1 && python3 -c 'import yaml' >/dev/null 2>&1; then
    PYTHON=$(command -v python3)
    printf 'yaml_parser: PyYAML %s (%s)\n' \
        "$("$PYTHON" -c 'import yaml; print(yaml.__version__)')" "$PYTHON"
else
    printf 'yaml_parser: unavailable\n'
fi

# --- artifact discovery -----------------------------------------------------
mapfile -d '' YAML_FILES < <(find "$DEPLOY_ROOT" -type f \
    \( -name '*.yaml' -o -name '*.yml' \) -print0 2>/dev/null) || YAML_FILES=()
mapfile -d '' DOCKERFILES < <(find "$DEPLOY_ROOT" -type f \
    \( -name 'Dockerfile' -o -name 'Dockerfile.*' -o -name '*.Dockerfile' \) \
    -print0 2>/dev/null) || DOCKERFILES=()
mapfile -d '' KUSTOMIZATIONS < <(find "$DEPLOY_ROOT" -type f \
    \( -name 'kustomization.yaml' -o -name 'kustomization.yml' \
       -o -name 'Kustomization' \) -print0 2>/dev/null) || KUSTOMIZATIONS=()

if [ ! -d "$DEPLOY_ROOT" ]; then
    report FAIL artifact-tree 1 "deploy root missing: $DEPLOY_ROOT"
elif [ "${#YAML_FILES[@]}" -eq 0 ]; then
    report FAIL artifact-yaml 1 "no YAML artifacts under $DEPLOY_ROOT"
else
    report PASS artifact-tree 0 "deploy root present with ${#YAML_FILES[@]} YAML artifact(s)"
fi

# --- 1. real YAML parse -----------------------------------------------------
if [ -z "$PYTHON" ]; then
    report SKIP yaml-parse 127 "PyYAML not installed; no parser available"
elif [ "${#YAML_FILES[@]}" -eq 0 ]; then
    report FAIL yaml-parse 1 "no YAML artifacts to parse (missing artifact)"
else
    parse_out=$("$PYTHON" - "$DEPLOY_ROOT" <<'PY' 2>&1
import os
import sys

import yaml

root = sys.argv[1]
files = []
for dirpath, dirnames, filenames in os.walk(root):
    dirnames[:] = [d for d in dirnames if d not in (".git", "target")]
    for name in filenames:
        if name.endswith((".yaml", ".yml")):
            files.append(os.path.join(dirpath, name))
files.sort()
if not files:
    print("no YAML files found under " + root)
    sys.exit(3)

bad = []
for path in files:
    try:
        with open(path, encoding="utf-8") as handle:
            list(yaml.safe_load_all(handle))
    except Exception as exc:  # noqa: BLE001 - report the real parser error
        reason = str(exc).replace("\n", " ")[:180]
        bad.append("{}: {}".format(os.path.relpath(path, root), reason))

if bad:
    print("{}/{} file(s) failed to parse:".format(len(bad), len(files)))
    for entry in bad[:10]:
        print("  " + entry)
    sys.exit(1)
print("parsed {}/{} YAML artifact(s)".format(len(files), len(files)))
PY
)
    parse_rc=$?
    if [ "$parse_rc" -eq 0 ]; then
        report PASS yaml-parse "$parse_rc" "$parse_out"
    else
        report FAIL yaml-parse "$parse_rc" "$parse_out"
    fi
fi

# --- Dockerfile inspection helpers -----------------------------------------
# Echo the portion of a Dockerfile from its last FROM onward (the runtime
# stage). An empty result means the file has no FROM and is invalid.
final_stage() {
    local file="$1" from_line
    from_line=$(grep -inE '^[[:space:]]*FROM[[:space:]]+' "$file" 2>/dev/null \
        | tail -1 | cut -d: -f1)
    if [ -z "$from_line" ]; then
        return 1
    fi
    tail -n "+$from_line" "$file"
}

# Echo the argument of the last USER directive in the final stage.
last_user_arg() {
    local stage="$1" line
    line=$(printf '%s\n' "$stage" \
        | grep -iE '^[[:space:]]*USER[[:space:]]+' | tail -1)
    printf '%s\n' "$line" \
        | sed -E 's/^[[:space:]]*[Uu][Ss][Ee][Rr][[:space:]]+([^[:space:]#]+).*/\1/'
}

# --- 2. non-root USER -------------------------------------------------------
if [ "${#DOCKERFILES[@]}" -eq 0 ]; then
    report FAIL dockerfile-user 1 "no Dockerfiles under $DEPLOY_ROOT (missing artifact)"
else
    user_offenders=""
    for df in "${DOCKERFILES[@]}"; do
        rel=${df#"$REPO_ROOT"/}
        stage=$(final_stage "$df")
        stage_rc=$?
        if [ "$stage_rc" -ne 0 ]; then
            user_offenders="$user_offenders $rel:no-FROM"
            continue
        fi
        uarg=$(last_user_arg "$stage")
        if [ -z "$uarg" ]; then
            user_offenders="$user_offenders $rel:no-USER"
        elif printf '%s\n' "$uarg" \
            | grep -qiE '^(0|root|0:0|root:root|:0|[0-9]+:0)$'; then
            user_offenders="$user_offenders $rel:root-USER($uarg)"
        fi
    done
    if [ -n "$user_offenders" ]; then
        report FAIL dockerfile-user 1 \
            "${#DOCKERFILES[@]} Dockerfile(s); offenders:$user_offenders"
    else
        report PASS dockerfile-user 0 \
            "${#DOCKERFILES[@]} Dockerfile(s), all with a non-root final USER"
    fi
fi

# --- 3. HEALTHCHECK where applicable ---------------------------------------
if [ "${#DOCKERFILES[@]}" -eq 0 ]; then
    report FAIL dockerfile-healthcheck 1 "no Dockerfiles under $DEPLOY_ROOT (missing artifact)"
else
    hc_offenders=""
    hc_skipped=""
    hc_ok=0
    for df in "${DOCKERFILES[@]}"; do
        rel=${df#"$REPO_ROOT"/}
        stage=$(final_stage "$df")
        stage_rc=$?
        if [ "$stage_rc" -ne 0 ]; then
            hc_offenders="$hc_offenders $rel:no-FROM"
            continue
        fi
        has_hc=$(printf '%s\n' "$stage" \
            | grep -icE '^[[:space:]]*HEALTHCHECK[[:space:]]+')
        has_entry=$(printf '%s\n' "$stage" \
            | grep -icE '^[[:space:]]*(ENTRYPOINT|CMD)[[:space:]]+')
        if [ "$has_hc" -gt 0 ]; then
            hc_ok=$((hc_ok + 1))
        elif [ "$has_entry" -eq 0 ]; then
            hc_skipped="$hc_skipped $rel:no-service-process"
        else
            hc_offenders="$hc_offenders $rel:no-HEALTHCHECK"
        fi
    done
    if [ -n "$hc_offenders" ]; then
        report FAIL dockerfile-healthcheck 1 \
            "$hc_ok with HEALTHCHECK; offenders:$hc_offenders"
    elif [ -n "$hc_skipped" ]; then
        report SKIP dockerfile-healthcheck 0 \
            "$hc_ok with HEALTHCHECK; not applicable:$hc_skipped"
    else
        report PASS dockerfile-healthcheck 0 \
            "${#DOCKERFILES[@]} service Dockerfile(s), all with HEALTHCHECK"
    fi
fi

# --- ag binary discovery ----------------------------------------------------
find_ag() {
    if [ -n "$AG_BIN" ]; then
        [ -x "$AG_BIN" ] && { printf '%s\n' "$AG_BIN"; return 0; }
        return 1
    fi
    local target_dir="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
    if [ -x "$target_dir/debug/ag" ]; then
        printf '%s\n' "$target_dir/debug/ag"
        return 0
    fi
    if [ -x "$REPO_ROOT/target/debug/ag" ]; then
        printf '%s\n' "$REPO_ROOT/target/debug/ag"
        return 0
    fi
    if command -v ag >/dev/null 2>&1; then
        command -v ag
        return 0
    fi
    return 1
}

make_project() {
    local dest="$1" candidate="$PROJECT_DIR"
    if [ -z "$candidate" ] || [ ! -d "$candidate" ]; then
        candidate="$REPO_ROOT/examples/todo-api"
    fi
    if [ -d "$candidate" ]; then
        cp -a "$candidate"/. "$dest"/
    else
        printf 'name = "deployqa"\nversion = "0.1.0"\n' > "$dest/ag.toml"
    fi
    # The renderer reads deploy/docker and deploy/k8s from the project. Seed
    # them with the artifacts under validation so `plan` has real source sets.
    if [ -d "$DEPLOY_ROOT" ]; then
        mkdir -p "$dest/deploy"
        cp -a "$DEPLOY_ROOT"/. "$dest/deploy"/
    fi
}

project_fingerprint() {
    (cd "$1" && find . -type f -printf '%P\n' 2>/dev/null | sort \
        | sha256sum | cut -d' ' -f1)
}

AG=""
if AG=$(find_ag); then
    printf 'ag_binary: %s\n' "$AG"
else
    printf 'ag_binary: unavailable\n'
fi

# --- 4. ag deploy plan ------------------------------------------------------
if [ -z "$AG" ]; then
    report SKIP ag-deploy-plan 127 "ag binary not found; set AG_BIN"
elif ! "$AG" deploy --help >/dev/null 2>&1; then
    report FAIL ag-deploy-plan 1 \
        "ag binary has no 'deploy' subcommand: $AG"
else
    for target in $AG_DEPLOY_TARGETS; do
        plan_dir="$WORK_ROOT/plan-$target"
        mkdir -p "$plan_dir"
        make_project "$plan_dir"
        before=$(project_fingerprint "$plan_dir")
        plan_out=$(cd "$plan_dir" && "$AG" deploy plan --target "$target" 2>&1)
        plan_rc=$?
        after=$(project_fingerprint "$plan_dir")
        # Prefer the documented dpl-<target>-<16 hex> id. Do not fall back to
        # a generic token: the project name and file fingerprints also look
        # like ids and must never be reported as one.
        plan_id=$(printf '%s\n' "$plan_out" \
            | grep -Eio 'dpl-[a-z]+-[a-f0-9]{16}' | head -1)
        if [ -z "$plan_id" ]; then
            plan_id=$(printf '%s\n' "$plan_out" \
                | grep -Ei 'plan[ _-]*id[[:space:]:=]+[A-Za-z0-9_-]{6,}' | head -1 \
                | sed -E 's/.*[[:space:]:=]([A-Za-z0-9_-]{6,}).*/\1/')
        fi
        if [ "$plan_rc" -ne 0 ]; then
            report FAIL "ag-deploy-plan-$target" "$plan_rc" \
                "plan command failed: $(printf '%s' "$plan_out" | head -c 300)"
        elif [ -z "$plan_id" ]; then
            report FAIL "ag-deploy-plan-$target" "$plan_rc" \
                "no plan id in output: $(printf '%s' "$plan_out" | head -c 300)"
        elif [ "$before" != "$after" ]; then
            report FAIL "ag-deploy-plan-$target" "$plan_rc" \
                "plan wrote artifacts into the project directory (id=$plan_id)"
        else
            report PASS "ag-deploy-plan-$target" "$plan_rc" \
                "plan id=$plan_id; project file set unchanged"
        fi
    done
fi

# --- 5. ag deploy apply refuses without a plan ------------------------------
if [ -z "$AG" ]; then
    report SKIP ag-deploy-apply-noplan 127 "ag binary not found; set AG_BIN"
elif ! "$AG" deploy --help >/dev/null 2>&1; then
    report FAIL ag-deploy-apply-noplan 1 \
        "ag binary has no 'deploy' subcommand: $AG"
else
    apply_dir="$WORK_ROOT/apply"
    mkdir -p "$apply_dir"
    make_project "$apply_dir"
    apply_out=$(cd "$apply_dir" && "$AG" deploy apply 2>&1)
    apply_rc=$?
    refusal=$(printf '%s\n' "$apply_out" \
        | grep -Eio 'plan|required|refus|missing|must|no plan|first' | head -1)
    if [ "$apply_rc" -eq 0 ]; then
        report FAIL ag-deploy-apply-noplan "$apply_rc" \
            "apply succeeded without a plan: $(printf '%s' "$apply_out" | head -c 300)"
    elif [ -z "$refusal" ]; then
        report FAIL ag-deploy-apply-noplan "$apply_rc" \
            "apply failed but did not explain the missing plan: $(printf '%s' "$apply_out" | head -c 300)"
    else
        report PASS ag-deploy-apply-noplan "$apply_rc" \
            "apply refused without a plan ($refusal)"
    fi
fi

# --- 6. ag deploy status offline -------------------------------------------
if [ -z "$AG" ]; then
    report SKIP ag-deploy-status-offline 127 "ag binary not found; set AG_BIN"
elif ! "$AG" deploy --help >/dev/null 2>&1; then
    report FAIL ag-deploy-status-offline 1 \
        "ag binary has no 'deploy' subcommand: $AG"
else
    status_dir="$WORK_ROOT/status"
    mkdir -p "$status_dir"
    make_project "$status_dir"
    status_out=$(cd "$status_dir" && "$AG" deploy status 2>&1)
    status_rc=$?
    net_error=$(printf '%s\n' "$status_out" \
        | grep -Eio 'connection refused|dial tcp|i/o timeout|no such host|unable to connect|could not connect' \
        | head -1)
    if [ "$status_rc" -ne 0 ]; then
        report FAIL ag-deploy-status-offline "$status_rc" \
            "status failed offline: $(printf '%s' "$status_out" | head -c 300)"
    elif [ -n "$net_error" ]; then
        report FAIL ag-deploy-status-offline "$status_rc" \
            "status required a live endpoint ($net_error)"
    else
        report PASS ag-deploy-status-offline "$status_rc" \
            "status answered offline: $(printf '%s' "$status_out" | head -c 200)"
    fi
fi

# --- 7. kustomize dry-run (local, no cluster needed) ------------------------
if ! command -v kustomize >/dev/null 2>&1; then
    report SKIP kustomize-build 127 "kustomize not installed"
elif [ "${#KUSTOMIZATIONS[@]}" -eq 0 ]; then
    report FAIL kustomize-build 1 \
        "kustomize installed but no kustomization manifest under $DEPLOY_ROOT"
else
    kust_bad=""
    kust_ok=0
    for kfile in "${KUSTOMIZATIONS[@]}"; do
        kdir=$(dirname -- "$kfile")
        kout=$(kustomize build "$kdir" 2>&1)
        krc=$?
        if [ "$krc" -eq 0 ]; then
            kust_ok=$((kust_ok + 1))
        else
            kust_bad="$kust_bad ${kfile#"$REPO_ROOT"/}:$(printf '%s' "$kout" | head -c 120)"
        fi
    done
    if [ -n "$kust_bad" ]; then
        report FAIL kustomize-build 1 \
            "$kust_ok rendered; failures:$kust_bad"
    else
        report PASS kustomize-build 0 \
            "$kust_ok kustomization(s) rendered with kustomize build"
    fi
fi

# --- 7b. kubeconform schema validation (when installed) --------------------
if ! command -v kubeconform >/dev/null 2>&1; then
    report SKIP kubeconform 127 "kubeconform not installed; no k8s schema validation"
elif [ "${#YAML_FILES[@]}" -eq 0 ]; then
    report FAIL kubeconform 1 "kubeconform installed but no YAML manifests under $DEPLOY_ROOT"
else
    kc_out=$(kubeconform -summary "${YAML_FILES[@]}" 2>&1)
    kc_rc=$?
    if [ "$kc_rc" -eq 0 ]; then
        report PASS kubeconform "$kc_rc" "all manifests valid: $(printf '%s' "$kc_out" | head -c 200)"
    else
        report FAIL kubeconform "$kc_rc" "$(printf '%s' "$kc_out" | head -c 300)"
    fi
fi

# --- 8a. docker build (requires a running daemon) ---------------------------
if ! command -v docker >/dev/null 2>&1; then
    report SKIP docker-build 127 "docker client not installed"
elif ! docker info >/dev/null 2>&1; then
    report SKIP docker-build 1 \
        "docker client $(docker --version 2>&1 | head -1) present but daemon unreachable"
elif [ "${#DOCKERFILES[@]}" -eq 0 ]; then
    report FAIL docker-build 1 "docker daemon live but no Dockerfiles to build"
else
    docker_bad=""
    docker_ok=0
    for df in "${DOCKERFILES[@]}"; do
        dctx=$(dirname -- "$df")
        dout=$(cd "$dctx" && timeout 300 docker build --check -f "$df" . 2>&1)
        drc=$?
        if [ "$drc" -eq 0 ]; then
            docker_ok=$((docker_ok + 1))
        else
            docker_bad="$docker_bad ${df#"$REPO_ROOT"/}:rc=$drc"
        fi
    done
    if [ -n "$docker_bad" ]; then
        report FAIL docker-build 1 "$docker_ok checked; failures:$docker_bad"
    else
        report PASS docker-build 0 "$docker_ok Dockerfile(s) passed docker build --check"
    fi
fi

# --- 8b. kubectl server-side dry-run (requires a cluster) ------------------
if ! command -v kubectl >/dev/null 2>&1; then
    report SKIP kubectl-dry-run 127 "kubectl not installed"
elif ! kubectl cluster-info >/dev/null 2>&1; then
    report SKIP kubectl-dry-run 1 \
        "kubectl client $(kubectl version --client 2>&1 | head -1) present but no cluster context"
elif [ "${#YAML_FILES[@]}" -eq 0 ]; then
    report FAIL kubectl-dry-run 1 "cluster available but no YAML manifests to validate"
else
    k8s_bad=""
    k8s_ok=0
    for yf in "${YAML_FILES[@]}"; do
        yout=$(kubectl apply --dry-run=server --validate=true -f "$yf" 2>&1)
        yrc=$?
        if [ "$yrc" -eq 0 ]; then
            k8s_ok=$((k8s_ok + 1))
        else
            k8s_bad="$k8s_bad ${yf#"$REPO_ROOT"/}:rc=$yrc"
        fi
    done
    if [ -n "$k8s_bad" ]; then
        report FAIL kubectl-dry-run 1 "$k8s_ok validated; failures:$k8s_bad"
    else
        report PASS kubectl-dry-run 0 "$k8s_ok manifest(s) passed server-side dry-run"
    fi
fi

printf 'summary: PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
[ "$FAIL" -eq 0 ]
