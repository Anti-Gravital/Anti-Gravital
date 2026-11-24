#!/usr/bin/env bash
# Maintain exactly one tracking issue for the weekly cargo audit / cargo deny
# job (RFC-0022 section 4.2). Idempotent: an existing marked issue is updated
# in place (and reopened if needed), a new one is created only when none ever
# existed, and it is closed when both checks pass again. Human issues are never
# reused.
#
# Inputs (environment):
#   AUDIT_STATUS, DENY_STATUS  exit codes (0 = pass)
#   AUDIT_OUTPUT, DENY_OUTPUT  files with the captured command output
#   GITHUB_REPOSITORY, GITHUB_SHA, GITHUB_SERVER_URL, GITHUB_RUN_ID
#   AG_RUN_URL                 optional full run URL override
#   GH_BIN                     optional gh path override (tests)
#   DRY_RUN                    1 = print writes instead of performing them
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-}"
if [[ -z "$REPO" ]]; then
  printf 'ERROR: GITHUB_REPOSITORY is not set.\n' >&2
  exit 1
fi

GH_BIN="${GH_BIN:-gh}"
TITLE='Scheduled security audit failures (cargo audit / cargo deny)'
MARKER='<!-- ag-weekly-security-audit -->'
AUDIT_STATUS="${AUDIT_STATUS:-1}"
DENY_STATUS="${DENY_STATUS:-1}"
AUDIT_OUTPUT="${AUDIT_OUTPUT:-}"
DENY_OUTPUT="${DENY_OUTPUT:-}"
SHA="${GITHUB_SHA:-unknown}"
RUN_URL="${AG_RUN_URL:-${GITHUB_SERVER_URL:-}/${REPO}/actions/runs/${GITHUB_RUN_ID:-}}"
DRY_RUN="${DRY_RUN:-0}"

gh_write() {
  if [[ "$DRY_RUN" == "1" ]]; then
    printf 'DRY RUN: gh %s\n' "$*"
    return 0
  fi
  "$GH_BIN" "$@"
}

append_output() {
  local name="$1" status="$2" file="$3"
  printf '### %s (exit %s)\n\n' "$name" "$status"
  if [[ -n "$file" && -f "$file" ]]; then
    printf '```\n'
    head -n 200 "$file"
    printf '```\n\n'
  else
    printf '(no output captured)\n\n'
  fi
}

body_file="$(mktemp)"
trap 'rm -f "$body_file"' EXIT
{
  printf '%s\n' "$MARKER"
  printf 'This issue is maintained automatically by the `security-audit` workflow (RFC-0022 section 4.2).\n\n'
  printf 'Last run: %s UTC\n' "$(date -u '+%Y-%m-%d %H:%M:%S')"
  printf 'Commit: `%s`\n' "$SHA"
  printf 'Run: %s\n\n' "$RUN_URL"
  printf '## Failing checks\n\n'
  if [[ "$AUDIT_STATUS" != "0" ]]; then
    append_output 'cargo audit' "$AUDIT_STATUS" "$AUDIT_OUTPUT"
  fi
  if [[ "$DENY_STATUS" != "0" ]]; then
    append_output 'cargo deny check' "$DENY_STATUS" "$DENY_OUTPUT"
  fi
  printf 'The issue is updated in place on the next weekly run and closed automatically when both checks pass.\n'
} > "$body_file"

failing=0
[[ "$AUDIT_STATUS" == "0" ]] || failing=1
[[ "$DENY_STATUS" == "0" ]] || failing=1

filter='.[] | select(.title == "'"$TITLE"'") | .number'
issue=''
issue_state=''
while IFS= read -r candidate; do
  [[ -n "$candidate" ]] || continue
  existing_body="$("$GH_BIN" issue view "$candidate" --repo "$REPO" --json body --jq .body 2>/dev/null || true)"
  if printf '%s' "$existing_body" | grep -Fq "$MARKER"; then
    issue="$candidate"
    issue_state="$("$GH_BIN" issue view "$candidate" --repo "$REPO" --json state --jq .state 2>/dev/null || true)"
    break
  fi
done < <("$GH_BIN" issue list --state all --label security --limit 100 \
  --repo "$REPO" --json number,title --jq "$filter")

if [[ "$failing" -ne 0 ]]; then
  "$GH_BIN" label create security --repo "$REPO" \
    --color ededed --description 'Security findings' >/dev/null 2>&1 || true
  if [[ -n "$issue" ]]; then
    if [[ "$issue_state" == "CLOSED" ]]; then
      gh_write issue reopen "$issue" --repo "$REPO"
    fi
    gh_write issue edit "$issue" --repo "$REPO" --body-file "$body_file"
    printf 'Updated security issue #%s with %s failing check(s).\n' "$issue" "$failing"
  else
    gh_write issue create --repo "$REPO" --title "$TITLE" --label security --body-file "$body_file"
    printf 'Opened security issue for %s failing check(s).\n' "$failing"
  fi
else
  if [[ -n "$issue" && "$issue_state" != "CLOSED" ]]; then
    gh_write issue comment "$issue" --repo "$REPO" \
      --body "Both weekly checks pass as of $(date -u '+%Y-%m-%d'). Closing this tracking issue."
    gh_write issue close "$issue" --repo "$REPO" --reason completed
    printf 'Closed security issue #%s: both checks pass.\n' "$issue"
  else
    printf 'Both checks pass; no tracking issue to update.\n'
  fi
fi
