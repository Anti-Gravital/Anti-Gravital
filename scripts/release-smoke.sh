#!/usr/bin/env bash
# Offline end-to-end smoke test for the Anti-Gravital alpha.
#
# Proves, with no network and no external services, that:
#   1. the workspace builds with the committed lock file (--locked --offline);
#   2. `ag new ... --no-interactive` scaffolds a buildable project;
#   3. `ag generate` compiles a small DSL schema and emits all artifact kinds;
#   4. the scaffolded project passes `cargo check` and `cargo build` offline;
#   5. `ag dev` starts a local server and GET / answers on loopback.
#
# Every step prints PASS or FAIL with an exit code. The script exits non-zero
# if any step fails. All generated files live in a mktemp directory that is
# removed on exit (including on failure or interrupt).
#
# Usage:
#   bash scripts/release-smoke.sh
#
# Optional environment:
#   SMOKE_BIND         bind address passed to `ag dev` (default 127.0.0.1:8080)
#   SMOKE_URL          URL curled for the hello endpoint
#                      (default http://127.0.0.1:8080/)
#   SMOKE_TIMEOUT      seconds to wait for the server (default 120)
#   CARGO_TARGET_DIR   shared target dir; defaults to <repo>/target
#
# NOTE: the generated `rest` template currently hardcodes a 0.0.0.0:8080
# listener and ignores the BIND variable that `ag dev` exports. The curl target
# is loopback (which succeeds), but the process really listens on all
# interfaces. This is tracked by the alpha release security review; the smoke
# test does not hide it.

set -u
set -o pipefail

# --- configuration --------------------------------------------------------

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIND="${SMOKE_BIND:-127.0.0.1:8080}"
URL="${SMOKE_URL:-http://127.0.0.1:8080/}"
TIMEOUT="${SMOKE_TIMEOUT:-120}"
# Unique per run: the hello response embeds the project name, so a stale
# server left on the port by a different process can never satisfy the curl
# check below.
PROJECT="agsmoke$$"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
AG="$TARGET_DIR/debug/ag"

# Force every cargo invocation, including the one spawned by `ag dev`, to stay
# offline. --locked is added explicitly to the build/check commands below.
export CARGO_NET_OFFLINE=true
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0

PASS_COUNT=0
FAIL_COUNT=0
DEV_PID=""
WORKDIR=""
DEV_LOG=""

# --- helpers --------------------------------------------------------------

pass() {
    local name="$1" code="${2:-0}"
    echo "PASS: $name (exit $code)"
    PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
    local name="$1" code="${2:-1}" detail="${3:-}"
    echo "FAIL: $name (exit $code)"
    if [ -n "$detail" ]; then
        echo "      $detail"
    fi
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

# Run a command as a named step, printing PASS/FAIL with its exit code.
step() {
    local name="$1"
    shift
    echo
    echo "==> STEP: $name"
    if "$@"; then
        pass "$name" 0
    else
        local code=$?
        fail "$name" "$code" "command: $*"
    fi
}

cleanup() {
    # Stop ag dev and the cargo run / server children it spawned. setsid makes
    # the background job a process-group leader, so a negative kill reaches
    # the whole tree; fall back to the plain pid where setsid is unavailable.
    if [ -n "$DEV_PID" ]; then
        if kill -0 "$DEV_PID" 2>/dev/null; then
            kill -TERM -"$DEV_PID" 2>/dev/null || kill -TERM "$DEV_PID" 2>/dev/null || true
            sleep 1
            kill -KILL -"$DEV_PID" 2>/dev/null || kill -KILL "$DEV_PID" 2>/dev/null || true
        fi
        wait "$DEV_PID" 2>/dev/null || true
    fi
    if [ -n "$WORKDIR" ] && [ -d "$WORKDIR" ]; then
        rm -rf "$WORKDIR"
    fi
}
trap cleanup EXIT

print_header() {
    echo "=== Anti-Gravital alpha release smoke test ==="
    echo "date:    $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "host:    $(uname -srm)"
    echo "rustc:   $(rustc --version 2>/dev/null || echo unavailable)"
    echo "cargo:   $(cargo --version 2>/dev/null || echo unavailable)"
    echo "commit:  $(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "repo:    $REPO_ROOT"
    echo "bind:    $BIND"
    echo "url:     $URL"
    echo "target:  $TARGET_DIR"
    echo "network: disabled (CARGO_NET_OFFLINE=true, --offline --locked)"
}

# --- step 0: preconditions -------------------------------------------------

echo
echo "==> STEP: preconditions"
PRE_OK=1
for tool in cargo rustc curl mktemp setsid bash; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "      missing tool: $tool"
        PRE_OK=0
    fi
done
if [ ! -f "$REPO_ROOT/Cargo.lock" ]; then
    echo "      missing Cargo.lock in $REPO_ROOT"
    PRE_OK=0
fi
if [ "$PRE_OK" -ne 1 ]; then
    fail "preconditions" 1
else
    pass "preconditions" 0
fi

print_header

# The remaining steps are meaningful only when the basic toolchain exists.
if [ "$PRE_OK" -ne 1 ]; then
    echo
    echo "RESULT: FAIL (preconditions unmet)"
    exit 1
fi

WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/ag-release-smoke.XXXXXX")"
echo "workdir: $WORKDIR"

cd "$REPO_ROOT" || { fail "cd repo" 1; exit 1; }

# --- step 1: workspace build with the committed lock file -----------------

step "build workspace (cargo build --workspace --locked --offline)" \
    cargo build --workspace --locked --offline

if [ ! -x "$AG" ]; then
    fail "ag binary present after build" 1 "$AG is not executable"
    echo
    echo "RESULT: FAIL"
    exit 1
else
    pass "ag binary present after build" 0
fi

# --- step 2: scaffold a project non-interactively -------------------------

echo
echo "==> STEP: scaffold project (ag new --no-interactive)"
cd "$WORKDIR" || { fail "cd workdir" 1; exit 1; }
if "$AG" new "$PROJECT" --template rest --no-interactive; then
    if [ -f "$WORKDIR/$PROJECT/Cargo.toml" ] \
        && [ -f "$WORKDIR/$PROJECT/src/main.rs" ] \
        && [ -f "$WORKDIR/$PROJECT/config.toml" ]; then
        pass "scaffold project" 0
    else
        fail "scaffold project" 1 "expected scaffold files are missing"
    fi
else
    fail "scaffold project" $? "ag new returned non-zero"
fi

# --- step 3: compile a small schema and emit artifacts --------------------

SCHEMA="$WORKDIR/$PROJECT/schema.ag"
cat > "$SCHEMA" <<'EOF'
# Minimal schema used by the alpha release smoke test.
model Note {
    id UUID @primary @auto
    title String @min(1) @max(200)
    body String?
}
EOF

echo
echo "==> STEP: compile schema (ag generate)"
cd "$WORKDIR/$PROJECT" || { fail "cd project" 1; exit 1; }
if "$AG" generate --schema "$SCHEMA" --output generated; then
    GEN_OK=1
    for expected in \
        "generated/src/models.rs" \
        "generated/src/mod.rs" \
        "generated/migrations/0001_initial.sql" \
        "generated/openapi.json" \
        "generated/openapi.yaml" \
        "generated/clients/typescript/types.ts"; do
        if [ ! -f "$expected" ]; then
            echo "      missing generated artifact: $expected"
            GEN_OK=0
        fi
    done
    if [ "$GEN_OK" -eq 1 ] && grep -q "pub struct Note" generated/src/models.rs; then
        pass "compile schema and emit artifacts" 0
    else
        fail "compile schema and emit artifacts" 1 "artifact set or Rust model incomplete"
    fi
else
    fail "compile schema and emit artifacts" $? "ag generate returned non-zero"
fi

# --- step 4: build the scaffolded/generated project offline ---------------

# The scaffold produced by `ag new` is the project `ag dev` will run. The DSL
# artifacts from step 3 are validated structurally above; this step proves the
# concrete Cargo project is resolvable and buildable with no network access.
# A fresh scaffold has no Cargo.lock, so resolve it offline first; --locked is
# then a meaningful reproducibility assertion for check and build.
cd "$WORKDIR/$PROJECT" || { fail "cd project" 1; exit 1; }
step "resolve generated project lockfile (cargo generate-lockfile --offline)" \
    cargo generate-lockfile --offline
step "cargo check generated project (--locked --offline)" \
    cargo check --locked --offline
step "cargo build generated project (--locked --offline)" \
    cargo build --locked --offline

# --- step 5: ag dev on loopback, then curl the hello endpoint -------------

echo
echo "==> STEP: start ag dev on loopback and curl $URL"
# Refuse to run if something already answers on the probe URL. Otherwise a
# stale server could satisfy the readiness check even though our own `ag dev`
# failed to bind (for example because the port was occupied).
if curl -fsS --max-time 2 "$URL" >/dev/null 2>&1; then
    echo "      $URL already answers; stop the existing server before running the smoke test"
    fail "loopback port free before dev start" 1 "$URL is already serving"
    echo
    echo "=== RESULT: $((PASS_COUNT + FAIL_COUNT)) steps, $PASS_COUNT PASS, $FAIL_COUNT FAIL ==="
    echo "RESULT: FAIL"
    exit 1
fi
pass "loopback port free before dev start" 0

if command -v setsid >/dev/null 2>&1; then
    setsid "$AG" dev --bind "$BIND" >"$WORKDIR/ag-dev.log" 2>&1 &
else
    "$AG" dev --bind "$BIND" >"$WORKDIR/ag-dev.log" 2>&1 &
fi
DEV_PID=$!
DEV_LOG="$WORKDIR/ag-dev.log"

BODY=""
CURL_CODE=1
ELAPSED=0
while [ "$ELAPSED" -lt "$TIMEOUT" ]; do
    if BODY="$(curl -fsS --max-time 3 "$URL" 2>/dev/null)"; then
        CURL_CODE=0
        break
    fi
    if ! kill -0 "$DEV_PID" 2>/dev/null; then
        echo "      ag dev exited before the server answered"
        break
    fi
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

if [ "$CURL_CODE" -eq 0 ] && printf '%s' "$BODY" | grep -q "$PROJECT"; then
    echo "      response body: $BODY"
    pass "ag dev + curl loopback hello endpoint" 0
else
    echo "      curl exit: $CURL_CODE; waited ${ELAPSED}s"
    if [ -f "$DEV_LOG" ]; then
        echo "      --- ag dev log (tail) ---"
        tail -n 20 "$DEV_LOG" | sed 's/^/      /'
    fi
    fail "ag dev + curl loopback hello endpoint" "$CURL_CODE" \
        "no matching response from $URL"
fi

# --- result ---------------------------------------------------------------

echo
echo "=== RESULT: $((PASS_COUNT + FAIL_COUNT)) steps, $PASS_COUNT PASS, $FAIL_COUNT FAIL ==="
if [ "$FAIL_COUNT" -gt 0 ]; then
    echo "RESULT: FAIL"
    exit 1
fi
echo "RESULT: PASS"
exit 0
