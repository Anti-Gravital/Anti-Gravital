#!/usr/bin/env bash
# Opt-in detector for documentation-vs-CLI drift (OP-24 / FEAT A1, task P0).
#
# Offline, deterministic and read-only. It asserts that a curated set of
# already-confirmed drift markers is ABSENT from the docs and that the
# corresponding CLI behaviour is PRESENT in the code. It never edits files,
# never touches .github/ (L-4 protected) and is NOT a CI gate: run it manually.
#
# Roots of truth (verified 2026-11-23, repo @ 8cf7f1d):
#   - crates/ag-cli/src/cli.rs:49-83  json_command_name() emits config.get and
#     env.list/env.check; cli.rs:119 dev bind default is 127.0.0.1:8080.
#   - crates/ag-cli/src/json_mode.rs:70-189 run_json_command() dispatches all of
#     them and the refusal list names env.list/env.check.
#   - templates/*/src/main.rs.tmpl  generated servers bind loopback and honour
#     BIND (#205); issue #191 bind caveats are stale.
#   - docs/mcp/cli-json-contract-0.2.0.md:34-42  shapes are wired (b81ce3b).
#
# Exit 0 = coherent, exit 1 = at least one drift detected, exit 2 = bad root.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

PASS=0
FAIL=0

pass() { printf 'PASS  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1" >&2; FAIL=$((FAIL + 1)); }

# assert_absent FILE REGEX LABEL -- fail (with file:line evidence) when a stale
# marker is present. Fails closed: a missing file is also a failure.
assert_absent() {
    local file=$1 re=$2 label=$3 hits
    if [ ! -f "$file" ]; then
        fail "$label | missing file: $file"
        return
    fi
    hits="$(grep -nE "$re" "$file" 2>/dev/null)"
    if [ -n "$hits" ]; then
        fail "$label | stale marker in $file (regex: $re)"
        printf '%s\n' "$hits" | sed 's/^/        /' >&2
    else
        pass "$label | absent in $file"
        PASS=$((PASS + 1))
    fi
}

# assert_present FILE REGEX LABEL -- fail when a ground-truth anchor is missing.
assert_present() {
    local file=$1 re=$2 label=$3
    if [ ! -f "$file" ]; then
        fail "$label | missing file: $file"
        return
    fi
    if grep -nE "$re" "$file" >/dev/null 2>&1; then
        pass "$label | present in $file"
        PASS=$((PASS + 1))
    else
        fail "$label | required anchor missing in $file (regex: $re)"
    fi
}

echo "doc-vs-CLI coherence detector (OP-24 / FEAT A1)"
echo "root: $ROOT"
echo

echo "[code] emitted JSON shapes"
assert_present crates/ag-cli/src/cli.rs 'Some\("config\.get"\)' 'cli.rs config.get'
assert_present crates/ag-cli/src/cli.rs 'Some\("env\.list"\)' 'cli.rs env.list'
assert_present crates/ag-cli/src/cli.rs 'Some\("env\.check"\)' 'cli.rs env.check'
assert_present crates/ag-cli/src/json_mode.rs 'dns\.status, env\.list, env\.check' 'json_mode.rs refusal list'
assert_present crates/ag-cli/src/json_mode.rs 'EnvCommands::List' 'json_mode.rs env.list arm'
assert_present crates/ag-cli/src/json_mode.rs 'EnvCommands::Check' 'json_mode.rs env.check arm'
echo

echo "[code] loopback bind default (#191/#205)"
assert_present crates/ag-cli/src/cli.rs 'default_value = "127\.0\.0\.1:8080"' 'cli.rs dev bind default'
assert_present templates/rest/src/main.rs.tmpl 'std::env::var\("BIND"\)' 'rest template honours BIND'
assert_present templates/rest/src/main.rs.tmpl '127\.0\.0\.1:8080' 'rest template loopback default'
echo

echo "[docs] stale markers (must be absent)"
assert_absent docs/mcp/cli-json-contract-0.2.0.md 'dispatch wiring pending|not yet emitted|Three more are' 'MCP contract 0.2.0 dispatch'
assert_absent docs/mcp/README.md 'dispatch wiring pending|not yet emitted' 'MCP README dispatch'
assert_absent README.md 'issue #191|hardcode.*0\.0\.0\.0|ignore.*BIND|bind all interfaces until fixed' 'README bind caveat'
assert_absent docs/manual/04-instalacion-y-onboarding.md 'issue #191|hardcode.*0\.0\.0\.0|ignoran esa variable|fijan.*0\.0\.0\.0|network-exposed until' 'manual bind caveat'
assert_absent CHANGELOG.md 'dispatch wiring is owned by the clidev role and is' 'CHANGELOG dispatch claim'
echo

echo "[docs] replacement truth (must be present)"
assert_present docs/mcp/cli-json-contract-0.2.0.md 'All eleven are emitted by the CLI' 'MCP contract emitted count'
assert_present docs/mcp/cli-json-contract-0.2.0.md 'now wired into the dispatch layer' 'MCP contract wiring'
assert_present docs/mcp/README.md 'are now emitted' 'MCP README emitted'
assert_present README.md 'scaffolds bind `127\.0\.0\.1:8080` by default and honour `BIND`' 'README EN bind truth'
assert_present README.md '`BIND` se respeta' 'README ES bind truth'
assert_present docs/manual/04-instalacion-y-onboarding.md 'honour `BIND`' 'manual EN bind truth'
assert_present docs/manual/04-instalacion-y-onboarding.md 'respetan `BIND`' 'manual ES bind truth'
assert_present CHANGELOG.md 'all ten 0\.2\.0 shapes are' 'CHANGELOG superseded note'
echo

printf 'RESULT: pass=%d fail=%d\n' "$PASS" "$FAIL"
if [ "$FAIL" -gt 0 ]; then
    echo "COHERENCE: FAIL (curated drift detected; see FAIL lines above)"
    exit 1
fi
echo "COHERENCE: PASS (no curated drift detected)"
exit 0
