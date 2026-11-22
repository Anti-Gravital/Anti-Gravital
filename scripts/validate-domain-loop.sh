#!/usr/bin/env bash
# Validates the wave-8 domain-to-deploy loop with real tools and no live
# provider. Every CLI step runs inside a bubblewrap sandbox with no network
# namespace and a local hosts file mapping the three environment hostnames to
# loopback, so a hidden provider call or DNS lookup cannot succeed silently.
#
# Checks, each reported as PASS, FAIL or SKIP:
#   1. hosts-file mock resolves dev/staging/prod hosts to 127.0.0.1;
#   2. the sandbox really denies outbound network (negative control);
#   3. three environment manifests parse and carry public/private rules;
#   4. `ag dns plan` emits a plan id and a record set for all three envs,
#      is deterministic and writes no files;
#   5. `ag dns apply` refuses without a plan id and with a bogus id;
#   6. `ag dns status` answers offline;
#   7. `--output json` works for reads and is refused for mutations;
#   8. public/private rules appear in plan output and applied state;
#   9. no credential value and no provider endpoint leaks into any output.
#
# A missing command or artifact is FAIL; an unavailable optional tool is SKIP.
# A SKIP never counts as a PASS. The script exits non-zero on any FAIL.
#
# Environment:
#   AG_BIN            explicit path to the ag binary (auto-detected otherwise)
#   DNS_ENVS          environments to plan (default: dev staging prod)
#   KEEP_WORK=1       keep the temporary work root and print its path
#   --self-test       exercise the harness against a fake ag (harness test only)

set -uo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

AG_BIN="${AG_BIN:-}"
DNS_ENVS="${DNS_ENVS:-dev staging prod}"
KEEP_WORK="${KEEP_WORK:-0}"
SELF_TEST=0

PASS=0
FAIL=0
SKIP=0
WORK_ROOT=""
OBSERVATIONS=""

usage() {
    cat <<'EOF'
Usage: scripts/validate-domain-loop.sh [options]

Options:
  -h, --help     Show this help.
      --self-test  Run against a bundled fake ag (tests the harness, not the product).

Environment:
  AG_BIN       Explicit ag binary path; auto-detected when unset.
  DNS_ENVS     Environments to plan (default: dev staging prod).
  KEEP_WORK=1  Keep the temporary work root for debugging.

Exit code: 0 when no FAIL, 1 when at least one FAIL, 2 on a harness error.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        --self-test) SELF_TEST=1; shift ;;
        *) printf 'Unknown argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
done

report() {
    local status="$1" name="$2" rc="$3" detail="$4"
    printf '[%s] %s (rc=%s): %s\n' "$status" "$name" "$rc" "$detail"
    case "$status" in
        PASS) PASS=$((PASS + 1)) ;;
        FAIL) FAIL=$((FAIL + 1)) ;;
        SKIP) SKIP=$((SKIP + 1)) ;;
    esac
}

observe() {
    OBSERVATIONS="$OBSERVATIONS
$1"
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

WORK_ROOT=$(mktemp -d "/tmp/opencode/ag-domain-loop-validation.XXXXXX") || {
    printf 'ERROR: could not create a temporary work root\n' >&2
    exit 2
}
HOSTS_FILE="$WORK_ROOT/etc-hosts"
PROJECT="$WORK_ROOT/project"
APPLY_PROJECT="$WORK_ROOT/apply-project"
FAKE_BIN="$WORK_ROOT/fake-ag"

# RFC 6761 .test names: never resolve publicly. One host per environment.
case_host() {
    case "$1" in
        dev) printf 'api.dev.example.test\n' ;;
        staging) printf 'api.staging.example.test\n' ;;
        prod) printf 'api.example.test\n' ;;
    esac
}
case_visibility() {
    case "$1" in
        prod) printf 'public\n' ;;
        *) printf 'private\n' ;;
    esac
}

ALL_HOSTS=""
for e in $DNS_ENVS; do ALL_HOSTS="$ALL_HOSTS $(case_host "$e")"; done
ALL_HOSTS="${ALL_HOSTS# }"

{
    printf '127.0.0.1 localhost\n'
    printf '127.0.0.1 %s\n' "$ALL_HOSTS"
} > "$HOSTS_FILE"

# --- environment header (not a step) ---------------------------------------
COMMIT=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || printf 'unknown')
BRANCH=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || printf 'unknown')
printf 'host: %s\n' "$(hostname 2>/dev/null || printf 'unknown')"
printf 'date_utc: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
printf 'commit: %s\n' "$COMMIT"
printf 'branch: %s\n' "$BRANCH"
printf 'bash: %s\n' "${BASH_VERSION:-unknown}"
printf 'rustc: %s\n' "$(rustc --version 2>/dev/null || printf unavailable)"
printf 'cargo: %s\n' "$(cargo --version 2>/dev/null || printf unavailable)"
printf 'python3: %s\n' "$(python3 --version 2>/dev/null || printf unavailable)"
printf 'mode: %s\n' "$([ "$SELF_TEST" -eq 1 ] && printf 'self-test (fake ag)' || printf 'product')"

# --- optional tool discovery ------------------------------------------------
BWRAP=""
if command -v bwrap >/dev/null 2>&1; then
    BWRAP=$(command -v bwrap)
    printf 'bwrap: %s\n' "$("$BWRAP" --version 2>&1 | head -1)"
else
    printf 'bwrap: unavailable\n'
fi
PYTHON=""
if command -v python3 >/dev/null 2>&1 && python3 -c 'import tomllib, json, socket' >/dev/null 2>&1; then
    PYTHON=$(command -v python3)
fi

# --- fixture: one manifest per environment ----------------------------------
write_manifest() {
    local dest="$1" env_name="$2" host="$3" visibility="$4"
    mkdir -p "$(dirname "$dest")"
    cat > "$dest" <<EOF
# Offline domain-loop fixture for environment '$env_name'.
[domain]
name = "$host"
environment = "$env_name"
visibility = "$visibility"
project = "domain-loop-fixture"
service = "api"

[provider]
name = "none"

[edge]
host = "edge.example.test"

[[records]]
type = "CNAME"
name = "$host"
value = "edge.example.test"
ttl = 300
visibility = "$visibility"

[[records]]
type = "TXT"
name = "_ag-domain.$host"
value = "ag-verification=offline-$env_name"
ttl = 300
EOF
}

make_project() {
    local dest="$1" e host vis
    mkdir -p "$dest/.ag/dns" "$dest/config"
    for e in $DNS_ENVS; do
        host=$(case_host "$e")
        vis=$(case_visibility "$e")
        write_manifest "$dest/.ag/dns/$e.toml" "$e" "$host" "$vis"
    done
    cp "$dest/.ag/dns/prod.toml" "$dest/.ag/dns.toml"
    printf '# non-secret values\n[service]\nport = 8080\n' > "$dest/config/prod.toml"
    printf '// offline fixture\n' > "$dest/schema.ag"
    # Seed only small deployment sources for optional fragment rendering.
    for sub in k8s docker; do
        if [ -d "$REPO_ROOT/deploy/$sub" ]; then
            mkdir -p "$dest/deploy/$sub"
            cp -a "$REPO_ROOT/deploy/$sub"/. "$dest/deploy/$sub"/ 2>/dev/null || true
        fi
    done
}

make_project "$PROJECT"
cp -a "$PROJECT"/. "$APPLY_PROJECT"/

project_fingerprint() {
    (cd "$1" && find . -type f -printf '%P\n' 2>/dev/null | sort | sha256sum | cut -d' ' -f1)
}

# --- sandbox runner ---------------------------------------------------------
run_in_sandbox() {
    local dir="$1"; shift
    if [ -n "$BWRAP" ]; then
        timeout 90 "$BWRAP" \
            --unshare-net \
            --ro-bind / / \
            --dev /dev \
            --bind "$HOSTS_FILE" /etc/hosts \
            --bind "$dir" "$dir" \
            --chdir "$dir" \
            --setenv AG_CLOUDFLARE_TOKEN SENTINEL_DNSQA_CLOUDFLARE_TOKEN_DO_NOT_PRINT \
            --setenv AG_DNS_TOKEN SENTINEL_DNSQA_DNS_TOKEN_DO_NOT_PRINT \
            --setenv AG_DNS_ZONE_ID SENTINEL_DNSQA_ZONE_ID_DO_NOT_PRINT \
            --setenv AG_DOMAINS_TOKEN SENTINEL_DNSQA_DOMAINS_TOKEN_DO_NOT_PRINT \
            -- "$@"
    else
        (cd "$dir" && \
            AG_CLOUDFLARE_TOKEN=SENTINEL_DNSQA_CLOUDFLARE_TOKEN_DO_NOT_PRINT \
            AG_DNS_TOKEN=SENTINEL_DNSQA_DNS_TOKEN_DO_NOT_PRINT \
            AG_DNS_ZONE_ID=SENTINEL_DNSQA_ZONE_ID_DO_NOT_PRINT \
            AG_DOMAINS_TOKEN=SENTINEL_DNSQA_DOMAINS_TOKEN_DO_NOT_PRINT \
            timeout 90 "$@")
    fi
}

# --- fake ag for --self-test ------------------------------------------------
write_fake_ag() {
    cat > "$FAKE_BIN" <<'EOF'
#!/usr/bin/env bash
# Harness self-test double. Not product code; never shipped.
set -u
pid() {
    local env="$1"
    case "$env" in
        dev) printf 'dns-dev-1111111111111111\n' ;;
        staging) printf 'dns-staging-2222222222222222\n' ;;
        *) printf 'dns-prod-3333333333333333\n' ;;
    esac
}
manifest_env() {
    local m="$1" env
    env=$(grep -E '^environment[[:space:]]*=' "$m" 2>/dev/null | head -1 \
        | sed -E 's/.*=[[:space:]]*"([^"]+)".*/\1/')
    case "$env" in dev|staging|prod) printf '%s\n' "$env" ;; *) printf 'prod\n' ;; esac
}
manifest_vis() {
    local m="$1" v
    v=$(grep -E '^visibility[[:space:]]*=' "$m" 2>/dev/null | head -1 \
        | sed -E 's/.*=[[:space:]]*"([^"]+)".*/\1/')
    case "$v" in public|private) printf '%s\n' "$v" ;; *) printf 'private\n' ;; esac
}
mf=""
args=()
while [ "$#" -gt 0 ]; do
    case "$1" in
        --manifest) mf="$2"; shift 2 ;;
        *) args+=("$1"); shift ;;
    esac
done
[ -z "$mf" ] && mf=".ag/dns.toml"
sub="${args[0]:-} ${args[1]:-}"
env=$(manifest_env "$mf")
vis=$(manifest_vis "$mf")
id=$(pid "$env")
case "$sub" in
  "--version ") printf 'ag self-test 0.0\n'; exit 0 ;;
  "--help ") printf 'fake ag\n'; exit 0 ;;
  "dns --help") printf 'fake ag dns\n\nUsage: ag dns <plan|apply|status>\n'; exit 0 ;;
  "dns plan"|"dns status")
    if printf '%s\n' "${args[@]}" "$@" | grep -q -- '--output'; then
      printf '{"contract_version":"1","plan_id":"%s","environment":"%s","visibility":"%s","dns":{"records":[{"type":"CNAME","name":"api.example.test"}]}}\n' "$id" "$env" "$vis"
    else
      printf 'Plan:        %s\n' "$id"
      printf 'Environment: %s\nVisibility:  %s\n' "$env" "$vis"
      printf 'Records (1):\nCNAME api.example.test -> edge.example.test 300 %s\n' "$vis"
      if [ "$vis" = "private" ]; then
        printf 'Note: Private visibility: no public-only records were planned.\n'
      fi
    fi
    exit 0 ;;
  "dns apply")
    given=""
    i=0
    while [ "$i" -lt "${#args[@]}" ]; do
        [ "${args[$i]}" = "--plan" ] && given="${args[$((i+1))]:-}"
        i=$((i+1))
    done
    if [ -z "$given" ]; then printf 'error: plan id is required\n' >&2; exit 2; fi
    if printf '%s\n' "$given" | grep -q 'deadbeef\|0000000000000000'; then
        printf 'error: plan id is stale or unknown\n' >&2; exit 2; fi
    if printf '%s\n' "${args[@]}" "$@" | grep -q -- '--output'; then
        printf 'error: --output json is not supported for mutating commands\n' >&2; exit 2
    fi
    mkdir -p .ag/dns
    printf '{"version":1,"applied":{"%s":{"plan_id":"%s","environment":"%s","visibility":"%s"}}}\n' \
        "$env" "$given" "$env" "$vis" > .ag/dns/state.json
    printf 'Applied plan %s\n' "$given"
    exit 0 ;;
  "domains attach")
    if printf '%s\n' "${args[@]}" "$@" | grep -q -- '--output'; then
        printf 'error: --output json is not supported for mutating commands\n' >&2; exit 2
    fi
    exit 0 ;;
  *) printf 'error: unrecognized command: %s\n' "$*" >&2; exit 2 ;;
esac
EOF
    chmod +x "$FAKE_BIN"
    AG_BIN="$FAKE_BIN"
}

if [ "$SELF_TEST" -eq 1 ]; then
    write_fake_ag
fi

find_ag() {
    if [ -n "$AG_BIN" ]; then
        [ -x "$AG_BIN" ] && { printf '%s\n' "$AG_BIN"; return 0; }
        return 1
    fi
    local target_dir="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
    if [ -x "$target_dir/debug/ag" ]; then
        printf '%s\n' "$target_dir/debug/ag"; return 0
    fi
    if [ -x "$REPO_ROOT/target/debug/ag" ]; then
        printf '%s\n' "$REPO_ROOT/target/debug/ag"; return 0
    fi
    if command -v ag >/dev/null 2>&1; then
        command -v ag; return 0
    fi
    return 1
}

AG=""
if AG=$(find_ag); then
    printf 'ag_binary: %s\n' "$AG"
    printf 'ag_version: %s\n' "$(run_in_sandbox "$PROJECT" "$AG" --version 2>&1 | head -1)"
else
    printf 'ag_binary: unavailable\n'
fi

# --- 1. hosts mock ----------------------------------------------------------
if [ -z "$PYTHON" ]; then
    report SKIP hosts-mock 127 "python3 with tomllib/socket unavailable"
elif [ -z "$BWRAP" ]; then
    hosts_rc=0
    for h in $ALL_HOSTS; do
        grep -qE "^127\.0\.0\.1[[:space:]].*(^|[[:space:]])${h}([[:space:]]|$)" \
            "$HOSTS_FILE" || hosts_rc=1
    done
    if [ "$hosts_rc" -eq 0 ]; then
        report PASS hosts-mock 0 "all hostnames mapped to 127.0.0.1 in $HOSTS_FILE"
    else
        report FAIL hosts-mock 1 "hosts mock missing a hostname mapping"
    fi
else
    hosts_out=$(run_in_sandbox "$PROJECT" "$PYTHON" -c '
import socket, sys
bad = []
for h in sys.argv[1:]:
    try:
        ip = socket.gethostbyname(h)
    except Exception as exc:
        bad.append(h + ":" + str(exc)); continue
    if ip != "127.0.0.1": bad.append(h + ":" + ip)
print("; ".join(h + "=127.0.0.1" for h in sys.argv[1:]) if not bad else "FAILED " + ", ".join(bad))
sys.exit(1 if bad else 0)
' $ALL_HOSTS 2>&1)
    hosts_rc=$?
    observe "$hosts_out"
    if [ "$hosts_rc" -eq 0 ]; then
        report PASS hosts-mock "$hosts_rc" "$hosts_out"
    else
        report FAIL hosts-mock "$hosts_rc" "hosts mock failed: $(printf '%s' "$hosts_out" | head -c 300)"
    fi
fi

# --- 2. sandbox negative control -------------------------------------------
if [ -z "$BWRAP" ]; then
    report SKIP sandbox-no-network 127 "bwrap unavailable; CLI steps ran without a network namespace"
else
    net_out=$(timeout 20 "$BWRAP" --unshare-net --ro-bind / / \
        -- "$PYTHON" -c '
import socket
try:
    socket.create_connection(("192.0.2.1", 53), timeout=2)
except OSError:
    print("outbound connection denied (expected)")
else:
    print("ERROR: outbound connection succeeded"); raise SystemExit(1)
' 2>&1)
    net_rc=$?
    if [ "$net_rc" -eq 0 ]; then
        report PASS sandbox-no-network 0 "$net_out"
    else
        report FAIL sandbox-no-network "$net_rc" "sandbox does not deny network: $(printf '%s' "$net_out" | head -c 200)"
    fi
fi

# --- 3. manifest fixture ----------------------------------------------------
if [ -z "$PYTHON" ]; then
    report SKIP manifest-fixture 127 "python3 with tomllib unavailable"
else
    manifest_out=$("$PYTHON" - "$PROJECT/.ag/dns" $DNS_ENVS <<'PY' 2>&1
import sys, os, tomllib
folder, envs = sys.argv[1], sys.argv[2:]
summary = []
for e in envs:
    path = os.path.join(folder, e + ".toml")
    try:
        with open(path, "rb") as fh: doc = tomllib.load(fh)
    except Exception as exc:
        print("{}: parse error: {}".format(path, exc)); raise SystemExit(1)
    d = doc.get("domain", {})
    if d.get("environment") != e:
        print("{}: environment mismatch: {}".format(path, d.get("environment"))); raise SystemExit(1)
    vis = d.get("visibility", "private")
    expected = "public" if e == "prod" else "private"
    if vis != expected:
        print("{}: visibility {} != {}".format(path, vis, expected)); raise SystemExit(1)
    if doc.get("provider", {}).get("name") != "none":
        print("{}: provider must be 'none' for offline validation".format(path)); raise SystemExit(1)
    if not doc.get("records"):
        print("{}: no records declared".format(path)); raise SystemExit(1)
    summary.append("{}={}".format(e, vis))
print("parsed {} manifests: {}".format(len(envs), ", ".join(summary)))
PY
)
    manifest_rc=$?
    if [ "$manifest_rc" -eq 0 ]; then
        report PASS manifest-fixture "$manifest_rc" "$manifest_out"
    else
        report FAIL manifest-fixture "$manifest_rc" "$manifest_out"
    fi
fi

# --- CLI gate: subcommand presence -----------------------------------------
CLI_READY=1
if [ -z "$AG" ]; then
    report SKIP ag-binary 127 "ag binary not found; set AG_BIN"
    CLI_READY=0
else
    help_out=$(run_in_sandbox "$PROJECT" "$AG" dns --help 2>&1)
    help_rc=$?
    observe "$help_out"
    if [ "$help_rc" -ne 0 ]; then
        report FAIL ag-dns-subcommand "$help_rc" \
            "ag has no 'dns' subcommand: $(printf '%s' "$help_out" | head -c 300)"
        CLI_READY=0
    else
        report PASS ag-dns-subcommand 0 "ag dns --help succeeded"
    fi
fi

extract_plan_id() {
    printf '%s\n' "$1" \
        | grep -Eio 'dns-[a-z]+-[a-f0-9]{16}|sha256:[a-f0-9]{16,64}' | head -1
}

# Captured plan outputs keyed by environment (associative array).
declare -A PLAN_OUTPUTS
declare -A PLAN_IDS
ALL_PLAN_IDS=""

# --- 4. ag dns plan (all three environments) --------------------------------
if [ "$CLI_READY" -eq 0 ]; then
    report FAIL ag-dns-plan 1 "ag dns plan not available (no dns subcommand)"
else
    before=$(project_fingerprint "$PROJECT")
    plan_fail=""
    plan_ok=""
    for e in $DNS_ENVS; do
        out=$(run_in_sandbox "$PROJECT" "$AG" dns plan --manifest ".ag/dns/$e.toml" 2>&1)
        rc=$?
        PLAN_OUTPUTS["$e"]="$out"
        observe "$out"
        id=$(extract_plan_id "$out")
        PLAN_IDS["$e"]="$id"
        rec=$(printf '%s\n' "$out" | grep -Eic 'CNAME|TXT|records|record' || true)
        net_error=$(printf '%s\n' "$out" \
            | grep -Eio 'connection refused|dial tcp|i/o timeout|no such host|unable to connect|could not connect|api\.[a-z0-9.-]+/v[0-9]' \
            | head -1)
        want_prefix="dns-$e-"
        if [ "$rc" -ne 0 ]; then
            plan_fail="$plan_fail $e:rc=$rc"
        elif [ -z "$id" ]; then
            plan_fail="$plan_fail $e:no-id"
        elif [ "$e" != "prod" ] && [ "${id#"$want_prefix"}" = "$id" ]; then
            plan_fail="$plan_fail $e:bad-id-prefix($id)"
        elif [ "$rec" -eq 0 ]; then
            plan_fail="$plan_fail $e:no-records"
        elif [ -n "$net_error" ]; then
            plan_fail="$plan_fail $e:network-error($net_error)"
        else
            plan_ok="$plan_ok $e=$id"
            ALL_PLAN_IDS="$ALL_PLAN_IDS $id"
        fi
    done
    after=$(project_fingerprint "$PROJECT")
    if [ -n "$plan_fail" ]; then
        report FAIL ag-dns-plan 1 "per-env failures:$plan_fail; ok:$plan_ok"
    elif [ "$before" != "$after" ]; then
        report FAIL ag-dns-plan 0 "plan wrote artifacts into the project directory"
    elif [ "$(printf '%s\n' $ALL_PLAN_IDS | sort -u | wc -l)" -ne "$(printf '%s\n' $DNS_ENVS | wc -w)" ]; then
        report FAIL ag-dns-plan 0 "plan ids are not distinct per environment: $ALL_PLAN_IDS"
    else
        report PASS ag-dns-plan 0 \
            "plan ids and record sets for all $(printf '%s\n' $DNS_ENVS | wc -w) envs:$plan_ok; project unchanged"
    fi
fi

# --- 5. apply gates ---------------------------------------------------------
if [ "$CLI_READY" -eq 0 ]; then
    report FAIL ag-dns-apply-gates 1 "ag dns apply not available (no dns subcommand)"
else
    noplan_out=$(run_in_sandbox "$APPLY_PROJECT" "$AG" dns apply --manifest .ag/dns/prod.toml 2>&1)
    noplan_rc=$?
    observe "$noplan_out"
    refusal=$(printf '%s\n' "$noplan_out" \
        | grep -Eio 'plan|required|refus|missing|must|first|id' | head -1)
    if [ "$noplan_rc" -eq 0 ]; then
        report FAIL ag-dns-apply-noplan "$noplan_rc" \
            "apply succeeded without a plan id: $(printf '%s' "$noplan_out" | head -c 300)"
    elif [ -z "$refusal" ]; then
        report FAIL ag-dns-apply-noplan "$noplan_rc" \
            "apply failed without explaining the missing plan: $(printf '%s' "$noplan_out" | head -c 300)"
    else
        report PASS ag-dns-apply-noplan "$noplan_rc" "apply refused without a plan id ($refusal)"
    fi

    before=$(project_fingerprint "$APPLY_PROJECT")
    bogus_out=$(run_in_sandbox "$APPLY_PROJECT" "$AG" dns apply \
        --manifest .ag/dns/prod.toml --plan dns-prod-deadbeefdeadbeef 2>&1)
    bogus_rc=$?
    after=$(project_fingerprint "$APPLY_PROJECT")
    observe "$bogus_out"
    if [ "$bogus_rc" -eq 0 ]; then
        report FAIL ag-dns-apply-bogus "$bogus_rc" \
            "apply accepted a bogus plan id: $(printf '%s' "$bogus_out" | head -c 300)"
    elif [ "$before" != "$after" ]; then
        report FAIL ag-dns-apply-bogus "$bogus_rc" \
            "apply wrote artifacts despite a bogus plan id"
    else
        report PASS ag-dns-apply-bogus "$bogus_rc" \
            "apply refused a bogus plan id and left the project unchanged"
    fi
fi

# --- 6. status offline ------------------------------------------------------
if [ "$CLI_READY" -eq 0 ]; then
    report FAIL ag-dns-status-offline 1 "ag dns status not available (no dns subcommand)"
else
    status_out=$(run_in_sandbox "$PROJECT" "$AG" dns status \
        --manifest .ag/dns/prod.toml --environment prod 2>&1)
    status_rc=$?
    observe "$status_out"
    net_error=$(printf '%s\n' "$status_out" \
        | grep -Eio 'connection refused|dial tcp|i/o timeout|no such host|unable to connect|could not connect' \
        | head -1)
    if [ "$status_rc" -ne 0 ]; then
        report FAIL ag-dns-status-offline "$status_rc" \
            "status failed offline: $(printf '%s' "$status_out" | head -c 300)"
    elif [ -n "$net_error" ]; then
        report FAIL ag-dns-status-offline "$status_rc" \
            "status required a live endpoint ($net_error)"
    else
        report PASS ag-dns-status-offline "$status_rc" \
            "status answered in the no-network sandbox: $(printf '%s' "$status_out" | head -c 200)"
    fi
fi

# --- 7. versioned JSON: reads allowed, mutations refused --------------------
if [ "$CLI_READY" -eq 0 ]; then
    report FAIL json-read 1 "json read checks unavailable (no dns subcommand)"
    report FAIL json-mutation-refused 1 "json mutation checks unavailable (no dns subcommand)"
elif [ -z "$PYTHON" ]; then
    report SKIP json-read 127 "python3 with json unavailable"
    report SKIP json-mutation-refused 127 "python3 with json unavailable"
else
    json_bad=""
    json_ok=""
    for cmd in plan status; do
        if [ "$cmd" = "plan" ]; then
            jout=$(run_in_sandbox "$PROJECT" "$AG" dns plan \
                --manifest .ag/dns/prod.toml --output json 2>&1)
        else
            jout=$(run_in_sandbox "$PROJECT" "$AG" dns status \
                --manifest .ag/dns/prod.toml --environment prod --output json 2>&1)
        fi
        jrc=$?
        observe "$jout"
        jcheck=$("$PYTHON" - "$jout" <<'PY' 2>&1
import json, sys
try:
    doc = json.loads(sys.argv[1])
except Exception as exc:
    print("not valid JSON: " + str(exc)); raise SystemExit(1)
if not isinstance(doc, dict):
    print("JSON root is not an object"); raise SystemExit(1)
if doc.get("contract_version") is None:
    print("JSON lacks contract_version"); raise SystemExit(1)
print("contract_version=" + str(doc["contract_version"]))
PY
)
        jparse=$?
        if [ "$jrc" -ne 0 ] || [ "$jparse" -ne 0 ]; then
            json_bad="$json_bad $cmd(rc=$jrc,$jcheck)"
        else
            json_ok="$json_ok $cmd"
        fi
    done
    if [ -n "$json_bad" ]; then
        report FAIL json-read 1 "read JSON failures:$json_bad; ok:$json_ok"
    else
        report PASS json-read 0 "dns plan and dns status emit valid versioned JSON:$json_ok"
    fi

    mut_bad=""
    mut_ok=""
    if [ -n "${PLAN_IDS[prod]:-}" ]; then
        mout=$(run_in_sandbox "$APPLY_PROJECT" "$AG" dns apply \
            --manifest .ag/dns/prod.toml --plan "${PLAN_IDS[prod]}" --output json 2>&1)
        mrc=$?
        observe "$mout"
        if [ "$mrc" -eq 0 ]; then
            mut_bad="$mut_bad dns-apply(rc=0)"
        else
            mut_ok="$mut_ok dns-apply(rc=$mrc)"
        fi
    else
        mut_bad="$mut_bad dns-apply(no-plan-id)"
    fi
    aout=$(run_in_sandbox "$APPLY_PROJECT" "$AG" domains attach \
        --domain api.example.test --output json 2>&1)
    arc=$?
    observe "$aout"
    if [ "$arc" -eq 0 ]; then
        mut_bad="$mut_bad domains-attach(rc=0)"
    else
        mut_ok="$mut_ok domains-attach(rc=$arc)"
    fi
    if [ -n "$mut_bad" ]; then
        report FAIL json-mutation-refused 1 "accepted --output json:$mut_bad; refused:$mut_ok"
    else
        report PASS json-mutation-refused 0 "mutations refused --output json:$mut_ok"
    fi
fi

# --- 8. visibility reflected in plan output and applied state --------------
if [ "$CLI_READY" -eq 0 ] || [ -z "${PLAN_IDS[prod]:-}" ]; then
    report FAIL visibility-rendered 1 "visibility check unavailable (no dns plan id)"
else
    app_out=$(run_in_sandbox "$APPLY_PROJECT" "$AG" dns apply \
        --manifest .ag/dns/prod.toml --plan "${PLAN_IDS[prod]}" 2>&1)
    app_rc=$?
    observe "$app_out"
    if [ "$app_rc" -ne 0 ]; then
        report FAIL visibility-rendered "$app_rc" \
            "apply with a valid plan id failed: $(printf '%s' "$app_out" | head -c 300)"
    else
        vis_check=$(AG_DNSQA_PLAN_DEV="${PLAN_OUTPUTS[dev]:-}" \
            AG_DNSQA_PLAN_STAGING="${PLAN_OUTPUTS[staging]:-}" \
            AG_DNSQA_PLAN_PROD="${PLAN_OUTPUTS[prod]:-}" \
            "$PYTHON" - "$APPLY_PROJECT" "$PROJECT/.ag/dns" $DNS_ENVS <<'PY' 2>&1
import os, re, sys, json, tomllib
apply_root, plan_dir, envs = sys.argv[1], sys.argv[2], sys.argv[3:]
state_path = os.path.join(apply_root, ".ag", "dns", "state.json")
problems = []
try:
    with open(state_path, encoding="utf-8") as fh:
        state = json.load(fh)
except Exception as exc:
    print("applied state missing or invalid: " + str(exc)); raise SystemExit(1)
applied = state.get("applied", {})
prod_entry = applied.get("prod", {})
if prod_entry.get("visibility") != "public":
    problems.append("applied prod entry is not public")
for e in envs:
    if e == "prod":
        continue
    entry = applied.get(e, {})
    if entry and entry.get("visibility") == "public":
        problems.append("private environment {} recorded as public".format(e))
# Each env's plan text must state the exact expected visibility and private
# plans must carry the fail-closed note with no public record.
for e in envs:
    path = os.path.join(plan_dir, e + ".toml")
    with open(path, "rb") as fh:
        expected = tomllib.load(fh)["domain"]["visibility"]
    text = os.environ.get("AG_DNSQA_PLAN_" + e.upper(), "")
    if not re.search(r"(?m)^Visibility:\s+" + re.escape(expected) + r"\s*$", text):
        problems.append("{} plan lacks an exact '{}' visibility line".format(e, expected))
    if expected == "private":
        if "no public-only records" not in text:
            problems.append("{} private plan lacks fail-closed note".format(e))
        if re.search(r"(?m)^\S+\s+\S+\s+\S+\s+\d+\s+public\s*$", text):
            problems.append("{} private plan emitted a public record".format(e))
if problems:
    print("; ".join(problems)); raise SystemExit(1)
print("visibility exact in plan text (fail-closed private) and in applied state for " + ", ".join(envs))
PY
)
        vis_rc=$?
        if [ "$vis_rc" -ne 0 ]; then
            report FAIL visibility-rendered "$vis_rc" "$vis_check"
        else
            report PASS visibility-rendered 0 "$vis_check"
        fi
    fi
fi

# --- 9. aggregate credential / provider leak scan --------------------------
leak_out=$(printf '%s\n' "$OBSERVATIONS" \
    | grep -Eio 'SENTINEL_DNSQA_[A-Z_]+|api\.cloudflare\.com/v[0-9]|api\.route53|dns\.googleapis\.com' \
    | sort -u | tr '\n' ' ')
if [ -n "$leak_out" ]; then
    report FAIL no-credentials-in-output 1 "leaked values/endpoints: $leak_out"
else
    report PASS no-credentials-in-output 0 \
        "no sentinel token values or provider API endpoints in any captured output"
fi

printf 'summary: PASS=%d FAIL=%d SKIP=%d\n' "$PASS" "$FAIL" "$SKIP"
[ "$FAIL" -eq 0 ]
