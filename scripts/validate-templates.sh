#!/usr/bin/env bash
# Validates every catalog template that declares template.toml.
#
# For each template the script:
#   1. checks the minimal metadata contract,
#   2. scaffolds a throwaway project with `ag new <name> --template <name>
#      --no-interactive`,
#   3. generates a lockfile and runs `cargo check --offline --locked` plus a
#      stricter `--all-targets` pass.
#
# Template sources are only read. Every write goes to a temporary work root.
# The script prints one PASS/FAIL line per template, a final summary, and
# exits non-zero when any template fails.

set -uo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "$SCRIPT_DIR/.." && pwd)

TEMPLATES_DIR="${TEMPLATES_DIR:-$REPO_ROOT/templates}"
STRICT_CLI="${STRICT_CLI:-0}"          # 1 = fail when ag new does not know a template
DEEP_CHECK="${DEEP_CHECK:-0}"          # 1 = also check test/example/bench targets
KEEP_WORK="${KEEP_WORK:-0}"            # 1 = keep the temporary work root for debugging
AG_BIN="${AG_BIN:-}"                   # explicit path to the ag binary

WORK_ROOT=""
PASS=0
PASS_AG=0
PASS_FALLBACK=0
FAIL=0
SKIP=0

usage() {
    cat <<'EOF'
Usage: scripts/validate-templates.sh [options]

Options:
  -h, --help        Show this help.
Environment:
  AG_BIN           Path to the ag binary. Auto-detected when unset.
  TEMPLATES_DIR    Override the templates directory (default: <repo>/templates).
  CARGO_TARGET_DIR Shared cargo target dir. A per-run temp dir is used when unset.
  STRICT_CLI=1     Fail instead of using the local scaffold fallback when
                   `ag new` rejects a template name.
  DEEP_CHECK=1     Also run `cargo check --offline --locked --all-targets`
                   to compile test, example and benchmark targets.
  KEEP_WORK=1      Do not delete the temporary work root; its path is printed.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

log() { printf '%s\n' "$*"; }
fail_line() { printf '[FAIL] %s\n' "$*"; }

cleanup() {
    if [ -n "$WORK_ROOT" ] && [ -d "$WORK_ROOT" ] && [ "$KEEP_WORK" != "1" ]; then
        rm -rf -- "$WORK_ROOT"
    elif [ -n "$WORK_ROOT" ] && [ "$KEEP_WORK" = "1" ]; then
        log "Work root kept at: $WORK_ROOT"
    fi
}
trap cleanup EXIT

WORK_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/ag-template-validation.XXXXXX") || {
    echo "ERROR: could not create a temporary work root" >&2
    exit 2
}

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
    export CARGO_TARGET_DIR="$WORK_ROOT/cargo-target"
fi
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0

find_ag() {
    if [ -n "$AG_BIN" ]; then
        if [ -x "$AG_BIN" ]; then printf '%s\n' "$AG_BIN"; return 0; fi
        return 1
    fi
    local target_dir="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
    if [ -x "$target_dir/debug/ag" ]; then printf '%s\n' "$target_dir/debug/ag"; return 0; fi
    if command -v ag >/dev/null 2>&1; then command -v ag; return 0; fi
    return 1
}

AG=$(find_ag) || {
    echo "ERROR: ag binary not found. Build it or set AG_BIN." >&2
    exit 2
}

# Validate the minimal template.toml contract. Echoes a space-separated
# reason list on stdout; non-empty output means invalid.
validate_metadata() {
    local dir="$1"
    local name="$2"
    local file="$dir/template.toml"
    local reason="" declared description category requirements

    if [ ! -f "$file" ]; then
        printf 'missing template.toml'
        return 1
    fi

    declared=$(sed -nE 's/^[[:space:]]*name[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$file" | head -1)
    if [ -z "$declared" ]; then
        reason="$reason name:missing"
    elif [ "$declared" != "$name" ]; then
        reason="$reason name:mismatch($declared)"
    fi

    description=$(sed -nE 's/^[[:space:]]*description[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$file" | head -1)
    [ -n "$description" ] || reason="$reason description:missing"

    category=$(sed -nE 's/^[[:space:]]*category[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$file" | head -1)
    case "$category" in
        web|api|realtime|infrastructure|ai|testing) ;;
        *) reason="$reason category:invalid($category)" ;;
    esac

    requirements=$(sed -nE 's/^[[:space:]]*external_requirements[[:space:]]*=[[:space:]]*(\[.*\]).*/\1/p' "$file" | head -1)
    [ -n "$requirements" ] || reason="$reason external_requirements:not_array"

    printf '%s' "$reason"
    [ -z "$reason" ]
}

# Local fallback with the same semantics as ag new: copy the tree, drop the
# .tmpl suffix, substitute {{name}} in paths and contents. Never writes to src.
scaffold_fallback() {
    local src="$1" project_dir="$2" project="$3" file rel target
    mkdir -p -- "$project_dir" || return 1
    while IFS= read -r -d '' file; do
        rel=${file#"$src"/}
        [ "$rel" = "template.toml" ] && continue
        rel=${rel%.tmpl}
        rel=${rel//'{{name}}'/"$project"}
        target="$project_dir/$rel"
        mkdir -p -- "$(dirname -- "$target")" || return 1
        sed "s/{{name}}/$project/g" "$file" > "$target" || return 1
    done < <(find "$src" -type f -print0)
    [ -f "$project_dir/Cargo.toml" ]
}

# Generate a lockfile and compile the scaffolded project with no network.
check_project() {
    local project_dir="$1"
    local rc=0
    (
        cd -- "$project_dir" || exit 1
        if [ ! -f Cargo.lock ]; then
            cargo generate-lockfile --offline || exit 1
        fi
        cargo check --offline --locked || exit 1
        if [ "$DEEP_CHECK" = "1" ]; then
            cargo check --offline --locked --all-targets || exit 1
        fi
    )
    rc=$?
    return "$rc"
}

log "Template catalog validation"
log "templates dir: $TEMPLATES_DIR"
log "ag binary:     $AG"
log "cargo target:  $CARGO_TARGET_DIR"
log ""

if [ ! -d "$TEMPLATES_DIR" ]; then
    fail_line "templates directory does not exist: $TEMPLATES_DIR"
    log "Summary: 0 passed, 1 failed"
    exit 1
fi

shopt -s nullglob
for template_dir in "$TEMPLATES_DIR"/*/; do
    [ -f "$template_dir/template.toml" ] || continue
    template_dir=${template_dir%/}
    name=$(basename -- "$template_dir")
    project="validate-$name"

    reason=$(validate_metadata "$template_dir" "$name") || {
        fail_line "$name - metadata:$reason"
        FAIL=$((FAIL + 1))
        continue
    }

    case_dir="$WORK_ROOT/$name"
    mkdir -p -- "$case_dir" || {
        fail_line "$name - could not create case directory"
        FAIL=$((FAIL + 1))
        continue
    }

    scaffold="ag new"
    if ! (cd -- "$case_dir" && "$AG" new "$project" --template "$name" --no-interactive) >"$case_dir/scaffold.log" 2>&1; then
        if [ "$STRICT_CLI" = "1" ]; then
            fail_line "$name - ag new rejected the template (STRICT_CLI=1)"
            FAIL=$((FAIL + 1))
            continue
        fi
        scaffold="fallback"
        if ! scaffold_fallback "$template_dir" "$case_dir/$project" "$project"; then
            fail_line "$name - scaffolding failed (ag new and fallback)"
            FAIL=$((FAIL + 1))
            continue
        fi
    fi

    if [ ! -f "$case_dir/$project/Cargo.toml" ]; then
        fail_line "$name - scaffold produced no Cargo.toml ($scaffold)"
        FAIL=$((FAIL + 1))
        continue
    fi

    if ! check_project "$case_dir/$project" >"$case_dir/check.log" 2>&1; then
        fail_line "$name - offline cargo check failed ($scaffold)"
        sed 's/^/    /' "$case_dir/check.log"
        FAIL=$((FAIL + 1))
        continue
    fi

    PASS=$((PASS + 1))
    if [ "$scaffold" = "ag new" ]; then
        PASS_AG=$((PASS_AG + 1))
        log "[PASS] $name (ag new, offline check ok)"
    else
        PASS_FALLBACK=$((PASS_FALLBACK + 1))
        log "[PASS] $name (fallback scaffold, offline check ok; ag new not wired yet)"
    fi
done

total=$((PASS + FAIL + SKIP))
log ""
log "Summary: $PASS passed ($PASS_AG via ag new, $PASS_FALLBACK via fallback), $FAIL failed, $SKIP skipped, total $total"

if [ "$PASS_FALLBACK" -gt 0 ] && [ "$STRICT_CLI" != "1" ]; then
    log "Note: fallback scaffolds prove template usability but not CLI registry wiring."
    log "      Re-run with STRICT_CLI=1 after the catalog registers every template."
fi

if [ "$FAIL" -ne 0 ] || [ "$total" -eq 0 ]; then
    if [ "$total" -eq 0 ]; then
        fail_line "no directory with template.toml was found"
    fi
    exit 1
fi

exit 0
