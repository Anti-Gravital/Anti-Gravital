#!/usr/bin/env bash
# Opt-in drift monitor for the last chrono edge (issue #185).
#
# Records two reproducible signals and compares them with a checked-in baseline:
#   1. cargo tree --workspace --all-features -i chrono
#   2. cargo deny --all-features check
#
# A normalized representation of both is hashed (SHA-256). The hash is compared
# with docs/audits/chrono-edge-baseline.sha256. A mismatch means the dependency
# edge or the deny outcome changed and the state must be re-examined; it does
# NOT by itself mean #185 is resolved.
#
# Result lines:
#   PASS  current hash equals the baseline: no drift, edge still present.
#   FAIL  drift detected: inspect the report, then update the baseline only
#         after an intentional and fully verified change.
#   SKIP  a required tool is missing; never reported as PASS.
#   ERROR baseline file missing or malformed, or an unknown argument.
#
# Exit statuses: PASS=0, SKIP=0, FAIL=1, ERROR=2. A SKIP is not a pass.
#
# This check is deliberately opt-in and local; it is not wired into default CI.
# See docs/audits/chrono-edge-monitor.md and the definitive analysis in
# docs/audits/chrono-unblock-analysis.md.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

baseline_file="docs/audits/chrono-edge-baseline.sha256"
report() { printf '%s\n' "$*"; }

case "${1:-}" in
    ""|--print-baseline) ;;
    *)
        report "RESULT: ERROR reason=unknown-argument detail=use no argument or --print-baseline"
        exit 2
        ;;
esac

# --- tool detection -----------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1 || ! command -v sha256sum >/dev/null 2>&1; then
    report "tool.cargo=$(cargo --version 2>&1 || printf 'missing')"
    report "RESULT: SKIP reason=required-tool-missing detail=cargo and sha256sum are needed; a missing tool is never PASS"
    exit 0
fi
if ! cargo deny --version >/dev/null 2>&1; then
    report "tool.cargo=$(cargo --version 2>&1)"
    report "RESULT: SKIP reason=cargo-deny-missing detail=install cargo-deny and re-run; a missing tool is never PASS"
    exit 0
fi

cargo_version="$(cargo --version 2>&1)"
deny_version="$(cargo deny --version 2>&1)"
sha_version="$(sha256sum --version 2>&1 | sed -n '1p')"

# Normalize machine-specific paths, ANSI colour and trailing whitespace so the
# hash is stable across checkouts and depends only on the graph and deny result.
normalize() {
    local text="$1"
    text="$(printf '%s\n' "$text" | sed -E 's/\x1B\[[0-9;]*[[:alpha:]]//g')"
    text="${text//"$repo_root"/<repo>}"
    if [ -n "${HOME:-}" ]; then
        text="${text//"$HOME"/<home>}"
    fi
    printf '%s\n' "$text" | sed -E 's/[[:space:]]+$//' | grep -vE '^[[:space:]]*$' || true
}

# --- signal 1: the reverse dependency tree of chrono --------------------------
tree_cmd="cargo tree --workspace --all-features -i chrono"
raw_tree="$(cargo tree --workspace --all-features -i chrono 2>&1 || true)"
tree_norm="$(normalize "$raw_tree")"
if printf '%s\n' "$raw_tree" | grep -aqE '^chrono v[0-9]'; then
    tree_empty=false
else
    tree_empty=true
fi

# --- signal 2: cargo deny result ----------------------------------------------
deny_cmd="cargo deny --all-features check"
if deny_raw="$(cargo deny --all-features check 2>&1)"; then
    deny_rc=0
else
    deny_rc=$?
fi
# Keep only the deterministic, security-relevant lines. The full verbose output
# contains duplicate-version reverse trees that change on unrelated updates.
deny_signal="$(printf '%s\n' "$deny_raw" \
    | sed -E 's/\x1B\[[0-9;]*[[:alpha:]]//g' \
    | grep -aE '^(advisories|bans|licenses|sources) |^error\[|^warning: [0-9]+ allowed' \
    | sort -u | tr '\n' ';' || true)"

# --- hash ---------------------------------------------------------------------
hash_input="$(printf 'tree:\n%s\ndeny.exit=%s\ndeny.status=%s\n' \
    "$tree_norm" "$deny_rc" "$deny_signal")"
current_hash="$(printf '%s' "$hash_input" | sha256sum | awk '{print $1}')"

if [ "${1:-}" = "--print-baseline" ]; then
    report "$current_hash  chrono-edge-baseline"
    exit 0
fi

# --- baseline comparison ------------------------------------------------------
if [ ! -f "$baseline_file" ]; then
    report "RESULT: ERROR reason=baseline-file-missing detail=$baseline_file"
    exit 2
fi
baseline_hash="$(awk 'NR==1 {print $1}' "$baseline_file")"
if ! printf '%s' "$baseline_hash" | grep -Eq '^[0-9a-f]{64}$'; then
    report "RESULT: ERROR reason=baseline-hash-malformed detail=$baseline_file"
    exit 2
fi

report "tool.cargo=$cargo_version"
report "tool.cargo-deny=$deny_version"
report "tool.sha256sum=$sha_version"
report "command.tree=$tree_cmd"
report "command.deny=$deny_cmd"
report "chrono.tree.empty=$tree_empty"
report "--- cargo tree -i chrono (normalized) ---"
printf '%s\n' "$tree_norm"
report "--- cargo tree -i chrono (end) ---"
report "deny.exit=$deny_rc"
report "deny.status=${deny_signal:-none}"
report "hash.algorithm=sha256"
report "hash.current=$current_hash"
report "hash.baseline=$baseline_hash"

if [ "$current_hash" = "$baseline_hash" ]; then
    report "RESULT: PASS reason=no-drift-from-baseline detail=chrono edge and deny outcome unchanged; issue #185 REMAINS OPEN (PASS is not a closure signal)"
    exit 0
fi

report "RESULT: FAIL reason=drift-detected detail=dependency edge or deny outcome changed; inspect the report and update $baseline_file only after an intentional fully verified change"
exit 1
