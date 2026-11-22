#!/usr/bin/env bash
# swarm-common: configuracion y primitivas compartidas por el orquestador.
set -uo pipefail

KSW_ROOT="${KSW_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || printf '%s' "$PWD")}"
[ -f "$KSW_ROOT/.kaizen/config.env" ] && . "$KSW_ROOT/.kaizen/config.env"
[ -f "$KSW_ROOT/.kaizen/swarm/swarm.env" ] && . "$KSW_ROOT/.kaizen/swarm/swarm.env"

KSW_ROOM_ROOT="$KSW_ROOT/.kaizen/swarm/issues"
KSW_ROLES_ALL=(architect product innovation dev1-core dev2-api dev3-ui dev4-data dev5-glue systems lowlevel ai data-expert infra qa security dx finops)
KSW_ROLES_R1=(product architect innovation systems lowlevel ai data-expert infra security dx)
KSW_ROLES_R2=(dev1-core dev2-api dev3-ui dev4-data dev5-glue qa)
KSW_ROLES_R3=(architect qa security dx finops)

klog()  { printf '[kaizen-swarm] %s\n' "$*"; }
kfail() { printf '[kaizen-swarm] FALLA: %s\n' "$*" >&2; }
know()  { printf '[kaizen-swarm] AVISO: %s\n' "$*" >&2; }

is_dry() { [ "${KSW_DRY_RUN:-0}" = "1" ]; }
run_cmd() { if is_dry; then klog "[dry-run] $*"; else "$@"; fi; }

room() { printf '%s/%s' "$KSW_ROOM_ROOT" "$1"; }
room_ok() { [ -d "$(room "$1")" ] || { kfail "sala inexistente para issue $1 (ejecuta init)"; return 1; }; }
round_roles() {
  case "$1" in
    1) printf '%s\n' "${KSW_ROLES_R1[@]}" ;;
    2) printf '%s\n' "${KSW_ROLES_R2[@]}" ;;
    3) printf '%s\n' "${KSW_ROLES_R3[@]}" ;;
  esac
}
role_agent() {
  case "$1" in
    architect) printf '%s' "${KSW_ROLE_AGENT_ARCHITECT:-opencode}" ;;
    dev*|dev5-glue) printf '%s' "${KSW_ROLE_AGENT_DEV:-opencode}" ;;
    qa) printf '%s' "${KSW_ROLE_AGENT_QA:-opencode}" ;;
    security) printf '%s' "${KSW_ROLE_AGENT_SECURITY:-opencode}" ;;
    *) printf '%s' "${KSW_ROLE_AGENT_DEFAULT:-opencode}" ;;
  esac
}
valid_role() {
  local r; for r in "${KSW_ROLES_ALL[@]}"; do [ "$r" = "$1" ] && return 0; done
  return 1
}
valid_issue() { case "${1:-}" in ''|*[!0-9]*) return 1;; *) return 0;; esac; }

# Numero de secuencia monotono por sala.
seq_next() {
  local d last; d="$(room "$1")/mailbox"; mkdir -p "$d"
  last="$(find "$d" -maxdepth 1 -type f -name '[0-9][0-9][0-9][0-9]-*' -printf '%f\n' 2>/dev/null | cut -d- -f1 | sort -n | tail -1)"
  printf '%04d' "$(( ${last:-0} + 1 ))"
}

# mail_send ISSUE FROM TO TYPE BODY [REFS...]
mail_send() {
  local issue="$1" from="$2" to="$3" type="$4" body="$5"; shift 5
  local d f seq ts refs="[]"
  d="$(room "$issue")/mailbox"; mkdir -p "$d"; seq="$(seq_next "$issue")"
  ts="$(date -u +%FT%TZ)"
  # construccion simple y segura del array JSON
  local x; refs="[]"; for x in "$@"; do
    if [ "$refs" = "[]" ]; then refs="[\"$x\"]"; else refs="${refs%]}, \"$x\"]"; fi
  done
  f="$d/${seq}-${ts}-${from}-to-${to}-${type}.md"
  if is_dry; then klog "[dry-run] mail $f"; return 0; fi
  {
    printf '%s\n' '---' "seq: $seq" "from: $from" "to: $to" "type: $type" 'status: open' "refs: $refs" "created: $ts" '---'
    printf '%s\n' "$body"
  } > "$f"
  printf '%s\n' "$f"
}

mail_find() { # ISSUE SEQ -> path
  find "$(room "$1")/mailbox" -maxdepth 1 -type f -name "$2-*.md" -print -quit 2>/dev/null
}
mail_count_type() { # ISSUE TYPE [open]
  local d; d="$(room "$1")/mailbox"; mkdir -p "$d"
  find "$d" -maxdepth 1 -type f -name "*-*-*-*-*$2.md" -print0 2>/dev/null \
    | xargs -0 -r grep -l "^type: $2\$" 2>/dev/null \
    | { if [ "${3:-}" = open ]; then xargs -r grep -l '^status: open$' 2>/dev/null || :; else cat; fi; } \
    | grep -c . || :
}
mail_pending_acks() { mail_count_type "$1" handoff open; }

# Veto/vista de bloqueos: dictamenes con BLOCK.
open_blocks() {
  local d; d="$(room "$1")/reviews"; mkdir -p "$d"
  grep -rl '^BLOCK$' "$d" 2>/dev/null | grep -c . || :
}
mark_verdict_ack() { # ISSUE FILE
  [ "${KSW_DRY_RUN:-0}" = "1" ] && return 0
  sed -i 's/^status: open$/status: ack/' "$2" 2>/dev/null || :
}

# leases.json: array de {task,owner,expires_at,status}
lease_add() { # ISSUE TASK OWNER [TTL]
  local issue="$1" task="$2" owner="$3" ttl="${4:-${KSW_LEASE_TTL_SEC:-1800}}" f exp
  f="$(room "$issue")/contracts/leases.json"
  exp="$(date -u -d "@$(( $(date -u +%s) + ttl ))" +%FT%TZ 2>/dev/null || date -u +%FT%TZ)"
  mkdir -p "$(dirname "$f")"; [ -f "$f" ] || printf '[]\n' > "$f"
  if is_dry; then klog "[dry-run] lease $task -> $owner (expira $exp)"; return 0; fi
  jq --arg t "$task" --arg o "$owner" --arg e "$exp" \
     '. + [{task:$t,owner:$o,expires_at:$e,status:"active"}]' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
}
lease_reap() { # ISSUE -> tareas reasignadas
  local issue="$1" f now n=0 t o
  f="$(room "$issue")/contracts/leases.json"; [ -f "$f" ] || { printf '0\n'; return 0; }
  now="$(date -u +%FT%TZ)"
  if is_dry; then klog "[dry-run] reap leases expirados antes de $now" >&2; printf '0\n'; return 0; fi
  while IFS=$'\t' read -r t o; do
    [ -z "$t" ] && continue
    mail_send "$issue" orchestrator all blocker "Lease caducado: $t (antes $o). Se libera y reasigna." >/dev/null || :
    n=$((n + 1))
  done < <(jq -r --arg now "$now" '.[] | select(.status=="active" and .expires_at < $now) | [.task,.owner] | @tsv' "$f")
  jq --arg now "$now" 'map(if .status=="active" and .expires_at < $now then .status="reaped" else . end)' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
  printf '%d\n' "$n"
}

# ownership.json por defecto: cada rol escribe solo su espacio de trabajo.
default_ownership() { # ISSUE
  local issue="$1" f; f="$(room "$issue")/contracts/ownership.json"
  mkdir -p "$(dirname "$f")"
  if is_dry; then klog "[dry-run] escribir ownership.json"; return 0; fi
  jq -n '{
    "architect":["contracts/interfaces.md","docs/kaizen/adr/*"],
    "product":["brief.md","docs/kaizen/pr-drafts/*"],
    "innovation":["docs/kaizen/research/*"],
    "dev1-core":["domain/*","crates/*/src/domain/*"],
    "dev2-api":["api/*","src/api/*"],
    "dev3-ui":["ui/*","src/ui/*"],
    "dev4-data":["migrations/*","src/data/*"],
    "dev5-glue":["docs/kaizen/pr-drafts/*","contracts/*"],
    "systems":["docs/kaizen/research/*"],
    "lowlevel":["docs/kaizen/research/*"],
    "ai":["docs/kaizen/swarm/prompts/*"],
    "data-expert":["docs/kaizen/data/*"],
    "infra": [".github/workflows/*"],
    "qa":["tests/*"],
    "security":[".kaizen/security/*","docs/kaizen/runbooks/*"],
    "dx":[".opencode/*",".claude/*",".agents/*"],
    "finops":["docs/kaizen/pr-drafts/*"]
  }' > "$f"
}

# check_ownership ISSUE BASE HEAD: falla si un rol toca rutas ajenas.
check_ownership() {
  local issue="$1" base="$2" head="$3" f rc=0 p role glob
  f="$(room "$issue")/contracts/ownership.json"
  [ -f "$f" ] || { kfail "falta ownership.json"; return 1; }
  # Los worktrees de rol se nombran wt-issue-N-ROL; se inspecciona cada rama.
  while IFS= read -r br; do
    role="${br##*-}"
    valid_role "$role" || continue
    git -C "$KSW_ROOT" rev-parse --verify -q "$br" >/dev/null 2>&1 || continue
    while IFS= read -r p; do
      [ -z "$p" ] && continue
      local owner_ok=0
      while IFS= read -r glob; do
        # comparacion con patron glob de jq
        [[ "$p" == $glob ]] && owner_ok=1 && break
      done < <(jq -r --arg r "$role" '.[$r] // [] | .[]' "$f")
      [ $owner_ok = 1 ] || { kfail "$role toca ruta no declarada: $p"; rc=1; }
    done < <(git -C "$KSW_ROOT" diff --name-only "$base" "$br" 2>/dev/null)
  done < <(git -C "$KSW_ROOT" for-each-ref --format='%(refname:short)' 'refs/heads/wt-issue-*' | grep "^wt-issue-$issue-" || :)
  return $rc
}
