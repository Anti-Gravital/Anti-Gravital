#!/usr/bin/env bash
# Opt-in local guard for the Anti-Gravital crate dependency direction (rule 15).
#
# Reads every crates/*/Cargo.toml and asserts the documented direction policy:
#   - the 19-crate roster is intact (CLAUDE.md rule 14);
#   - ag-core has no outgoing ag-* edge (CLAUDE.md rule 15, first rule);
#   - every ag-* edge is in the reviewed allow-list derived from canonical docs;
#   - explicitly forbidden reverse edges are absent (rules 2/6/7, RFC-0012,
#     ADR-0012);
#   - ag-edge -> ag-domains is present (control plane, ADR-0012/RFC-0011);
#   - the internal graph is acyclic (CLAUDE.md rule 15).
#
# Design: this is a manifest-text guard, not a Rust test, because it must stay
# offline, toolchain-free, deterministic and additive (no new crate or dep).
# A self dev-dependency (ag-mail in its own dev-dependencies) is exempt and
# counted; it is a test-only Cargo pattern, not a crate cycle.
#
# Exit codes: 0 = PASS, 1 = FAIL (a rule was violated), 2 = ERROR (bad usage
# or unreadable repository). Every failure names the offending crate pair and
# the manifest line. Nothing is skipped silently.
#
# Policy and sources: docs/architecture/dependency-direction-guard.md
# Usage: scripts/check-dependency-direction.sh [--help]

set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)" || {
    printf 'dependency-direction: ERROR cannot resolve repository root\n' >&2
    exit 2
}
cd "$repo_root" 2>/dev/null || {
    printf 'dependency-direction: ERROR cannot enter %s\n' "$repo_root" >&2
    exit 2
}

case "${1:-}" in
    "") ;;
    --help|-h)
        printf '%s\n' \
            'Usage: scripts/check-dependency-direction.sh [--help]' \
            'Offline, toolchain-free guard for CLAUDE.md rules 14/15.' \
            'Exit: 0 PASS, 1 FAIL, 2 ERROR. Policy: docs/architecture/dependency-direction-guard.md'
        exit 0;;
    *)
        printf 'dependency-direction: ERROR unknown argument %s (use --help)\n' "$1" >&2
        exit 2;;
esac

if [ ! -d crates ]; then
    printf 'dependency-direction: ERROR crates/ not found under %s\n' "$repo_root" >&2
    exit 2
fi

# --- Policy data: from|to|source ----------------------------------------------
# Source of truth: CLAUDE.md rules 14/15, docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md
# section 5.3, ADR-0007/0012 and RFC-0011/0012. Every entry cites its source.
ROSTER=(
    ag-core ag-dsl ag-cli ag-auth ag-data ag-realtime ag-cache ag-storage
    ag-observe ag-cloud ag-ai ag-mobile ag-migrate ag-wasm-host ag-lsp
    ag-mail ag-workers ag-domains ag-edge
)

ALLOWED=(
    'ag-auth|ag-data|Arq-Tecnica 5.3 regla 2'
    'ag-auth|ag-mail|CLAUDE.md:347 y Arq-Tecnica 5.3 regla 6'
    'ag-cache|ag-core|Arq-Tecnica 5.3 regla 2'
    'ag-cli|ag-core|Arq-Tecnica 5.3 reglas 2 y 4'
    'ag-cli|ag-dsl|Arq-Tecnica 5.3 regla 4'
    'ag-cli|ag-domains|Arq-Tecnica 5.3 regla 7 y docs/graph/crates.md:19'
    'ag-cli|ag-mail|Arq-Tecnica 5.3 regla 4 y docs/graph/crates.md:19'
    'ag-cli|ag-workers|RFC-0012 seccion 4'
    'ag-data|ag-core|Arq-Tecnica 5.3 regla 2'
    'ag-domains|ag-core|Arq-Tecnica 5.3 regla 2'
    'ag-edge|ag-domains|ADR-0012 y RFC-0011'
    'ag-lsp|ag-dsl|CLAUDE.md:331 y docs/graph/crates.md:22'
    'ag-mail|ag-core|Arq-Tecnica 5.3 regla 2'
    'ag-mail|ag-workers|CLAUDE.md:353 y RFC-0012 seccion 4'
    'ag-observe|ag-core|Arq-Tecnica 5.3 regla 2 (observabilidad -> core); RFC-0020 PR4'
    'ag-storage|ag-auth|docs/graph/crates.md:30 y spec ag-storage:348'
    'ag-storage|ag-core|Arq-Tecnica 5.3 regla 2 (estandar -> core); RFC-0020 PR4'
    'ag-workers|ag-core|Arq-Tecnica 5.3 regla 2 (workers -> core); RFC-0020 PR4'
    'ag-workers|ag-data|CLAUDE.md:352 y RFC-0012 seccion 4'
)

FORBIDDEN=(
    'ag-mail|ag-auth|CLAUDE.md:348 y Arq-Tecnica 5.3 regla 6'
    'ag-data|ag-auth|Arq-Tecnica 5.3 regla 2 y docs/modules/ag-data/README.md:14'
    'ag-auth|ag-storage|spec 2026-05-22-ag-storage-design.md:349'
    'ag-workers|ag-mail|RFC-0012 seccion 4'
    'ag-workers|ag-auth|RFC-0012 seccion 4'
    'ag-workers|ag-cloud|RFC-0012 seccion 4'
    'ag-workers|ag-domains|RFC-0012 seccion 4'
    'ag-workers|ag-edge|RFC-0012 seccion 4'
    'ag-workers|ag-cli|RFC-0012 seccion 4'
    'ag-domains|ag-edge|ADR-0012 y docs/audits/oportunidades-arquitectura.md:24'
    'ag-domains|ag-cloud|Arq-Tecnica 5.3 regla 7'
    'ag-domains|ag-cli|docs/audits/oportunidades-arquitectura.md:24'
    'ag-domains|ag-mail|docs/modules/ag-mail/README.md:169'
)

REQUIRED=(
    'ag-edge|ag-domains|ADR-0012 y RFC-0011 (plano de control)'
)

# --- Helpers ------------------------------------------------------------------
violations=0
violation() {
    printf 'FAIL %s\n' "$1" >&2
    violations=$((violations + 1))
}
in_roster() {
    local needle=$1 item
    for item in "${ROSTER[@]}"; do [ "$item" = "$needle" ] && return 0; done
    return 1
}
# --- 1. Parse manifests -------------------------------------------------------
declare -a EDGE_FROM=() EDGE_TO=() EDGE_SECTION=() EDGE_LINE=()
declare -a PARSE_ERRORS=() FOUND_CRATES=()

for manifest in crates/*/Cargo.toml; do
    [ -f "$manifest" ] || continue
    crate="$(basename "$(dirname "$manifest")")"
    FOUND_CRATES+=("$crate")
    while IFS=$'\t' read -r kind fa fb fc fd; do
        case "$kind" in
            edge)
                EDGE_FROM+=("$fa"); EDGE_TO+=("$fb")
                EDGE_SECTION+=("$fc"); EDGE_LINE+=("$fd")
                ;;
            parse_error)
                PARSE_ERRORS+=("$manifest:$fb: $fa")
                ;;
        esac
    done < <(awk -v me="$crate" '
        function dep_section(s) {
            if (s == "dependencies" || s == "dev-dependencies" || s == "build-dependencies") return 1
            if (s ~ /^target\..*\.(dependencies|dev-dependencies|build-dependencies)$/) return 1
            return 0
        }
        {
            line = $0
            if (line ~ /^[[:space:]]*\[/) {
                s = line
                sub(/^[[:space:]]*\[/, "", s)
                sub(/\].*$/, "", s)
                section = s
                next
            }
            if (!dep_section(section)) next
            if (line ~ /^[[:space:]]*#/) next
            if (line ~ /^[[:space:]]*$/) next
            if (line ~ /^[[:space:]]*ag-[A-Za-z0-9_-]+[[:space:]]*=/) {
                dep = line
                sub(/^[[:space:]]*/, "", dep)
                sub(/[[:space:]]*=.*$/, "", dep)
                printf "edge\t%s\t%s\t%s\t%d\n", me, dep, section, NR
                next
            }
            if (line ~ /^[[:space:]]*ag-/) {
                printf "parse_error\tunparsed ag-* declaration\t%d\n", NR
            }
        }
    ' "$manifest")
done

for pe in "${PARSE_ERRORS[@]}"; do
    violation "manifest parse: $pe : policy needs a parser update (fail closed)"
done

# --- 2. Roster ----------------------------------------------------------------
mapfile -t sorted_found < <(printf '%s\n' "${FOUND_CRATES[@]}" | sort)
mapfile -t sorted_roster < <(printf '%s\n' "${ROSTER[@]}" | sort)
roster_ok=1
if [ "${#sorted_found[@]}" -ne "${#sorted_roster[@]}" ]; then
    roster_ok=0
else
    for i in "${!sorted_found[@]}"; do
        [ "${sorted_found[$i]}" = "${sorted_roster[$i]}" ] || { roster_ok=0; break; }
    done
fi
if [ "$roster_ok" -eq 1 ]; then
    printf 'PASS roster: %d crates match CLAUDE.md rule 14\n' "${#ROSTER[@]}"
else
    violation "roster: found {${sorted_found[*]}} but expected {${sorted_roster[*]}} (new crate needs RFC, CLAUDE.md rule 28)"
fi

# --- 3. Per-edge direction policy --------------------------------------------
internal=0
self_deps=0
for i in "${!EDGE_FROM[@]}"; do
    from="${EDGE_FROM[$i]}"; to="${EDGE_TO[$i]}"
    section="${EDGE_SECTION[$i]}"; line="${EDGE_LINE[$i]}"
    loc="crates/$from/Cargo.toml:$line [$section]"

    if [ "$from" = "$to" ]; then
        self_deps=$((self_deps + 1))
        continue
    fi
    internal=$((internal + 1))

    if ! in_roster "$to"; then
        violation "unknown crate: $from -> $to at $loc (target outside the 19-crate roster)"
        continue
    fi

    if [ "$from" = "ag-core" ]; then
        violation "core isolation: ag-core -> $to at $loc (CLAUDE.md:382, first dependency rule)"
    fi

    for entry in "${FORBIDDEN[@]}"; do
        IFS='|' read -r f t src <<< "$entry"
        if [ "$from" = "$f" ] && [ "$to" = "$t" ]; then
            violation "forbidden direction: $from -> $to at $loc (source: $src)"
        fi
    done

    allowed_src=""
    for entry in "${ALLOWED[@]}"; do
        IFS='|' read -r f t src <<< "$entry"
        if [ "$from" = "$f" ] && [ "$to" = "$t" ]; then
            allowed_src="$src"
            break
        fi
    done
    if [ -z "$allowed_src" ]; then
        violation "unlisted edge: $from -> $to at $loc (update the allow-list with an RFC/ADR citation; no silent new direction)"
    fi
done

if [ "$violations" -eq 0 ]; then
    printf 'PASS directions: %d internal edges reviewed, %d self dev-dependency ignored\n' "$internal" "$self_deps"
fi

# --- 4. Required edges --------------------------------------------------------
required_ok=1
for entry in "${REQUIRED[@]}"; do
    IFS='|' read -r f t src <<< "$entry"
    present=0
    for i in "${!EDGE_FROM[@]}"; do
        if [ "${EDGE_FROM[$i]}" = "$f" ] && [ "${EDGE_TO[$i]}" = "$t" ]; then
            present=1; break
        fi
    done
    if [ "$present" -eq 0 ]; then
        required_ok=0
        violation "required edge missing: $f -> $t (source: $src)"
    fi
done
[ "$required_ok" -eq 1 ] && printf 'PASS required-edges: ag-edge -> ag-domains control plane present\n'

# --- 5. Acyclicity (DFS over internal edges, self deps excluded) ---------------
declare -A ADJ=()
for i in "${!EDGE_FROM[@]}"; do
    from="${EDGE_FROM[$i]}"; to="${EDGE_TO[$i]}"
    [ "$from" = "$to" ] && continue
    ADJ[$from]="${ADJ[$from]:-} $to"
done

declare -A STATE=()
declare -a STACK=()
CYCLE_MSG=""
dfs() {
    local node="$1" dep k idx=-1 path="" deps
    STATE[$node]=1
    STACK+=("$node")
    deps="$(for d in ${ADJ[$node]:-}; do printf '%s\n' "$d"; done | sort -u | tr '\n' ' ')"
    for dep in $deps; do
        case "${STATE[$dep]:-0}" in
            1)
                for k in "${!STACK[@]}"; do
                    if [ "${STACK[$k]}" = "$dep" ]; then idx=$k; break; fi
                done
                for ((k=idx; k<${#STACK[@]}; k++)); do
                    path="${path:+$path -> }${STACK[$k]}"
                done
                CYCLE_MSG="$path -> $dep"
                return 1
                ;;
            0) dfs "$dep" || return 1;;
        esac
    done
    unset 'STACK[-1]'
    STATE[$node]=2
    return 0
}

cycle_ok=1
for node in "${sorted_roster[@]}"; do
    if [ "${STATE[$node]:-0}" = "0" ]; then
        if ! dfs "$node"; then
            cycle_ok=0
            violation "cycle: $CYCLE_MSG (CLAUDE.md:383, no circular dependencies)"
            break
        fi
    fi
done
[ "$cycle_ok" -eq 1 ] && printf 'PASS acyclic: no cycle among the 19 crates\n'

# --- Result -------------------------------------------------------------------
if [ "$violations" -ne 0 ]; then
    printf 'dependency-direction: FAIL (%d violation(s))\n' "$violations" >&2
    exit 1
fi
printf 'dependency-direction: PASS (19 crates, %d edges, %d self dev-dependency ignored)\n' "$internal" "$self_deps"
exit 0
