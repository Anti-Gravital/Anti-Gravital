#!/usr/bin/env bash
# Enforce RFC-0022 section 4.8: a pull request with no activity for thirty days
# is labelled, given a note, and closed. The branch is never deleted.
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-}"
if [[ -z "$REPO" ]]; then
  printf 'ERROR: GITHUB_REPOSITORY is not set.\n' >&2
  exit 1
fi

GH_BIN="${GH_BIN:-gh}"
STALE_DAYS="${AG_STALE_DAYS:-30}"
DRY_RUN="${DRY_RUN:-0}"
if [[ "$DRY_RUN" == "true" ]]; then
  DRY_RUN=1
fi

if date -u -d "${STALE_DAYS} days ago" +%Y-%m-%d >/dev/null 2>&1; then
  cutoff="$(date -u -d "${STALE_DAYS} days ago" +%Y-%m-%d)"
else
  cutoff="$(date -u -v-${STALE_DAYS}d +%Y-%m-%d)"
fi

gh_write() {
  if [[ "$DRY_RUN" == "1" ]]; then
    printf 'DRY RUN: gh %s\n' "$*"
    return 0
  fi
  "$GH_BIN" "$@"
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

"$GH_BIN" label create stale --repo "$REPO" --color fedf00 \
  --description 'No activity for 30 days' >/dev/null 2>&1 || true

"$GH_BIN" pr list --repo "$REPO" --state open \
  --search "updated:<$cutoff" --limit 100 \
  --json number,title,updatedAt --jq '.[] | [.number, .title, .updatedAt] | @tsv' \
  > "$tmp/stale.tsv"

closed=0
while IFS=$'\t' read -r number title updated || [[ -n "${number:-}" ]]; do
  [[ -n "${number:-}" ]] || continue
  printf 'Stale PR #%s (%s), last activity %s: labelling and closing.\n' "$number" "$title" "$updated"
  {
    printf 'This pull request has had no activity for %s days (last activity %s).\n' "$STALE_DAYS" "$updated"
    printf 'Closing it under RFC-0022 section 4.8. The branch is kept; reopen the pull request when work resumes.\n'
  } > "$tmp/comment.md"
  gh_write pr edit "$number" --repo "$REPO" --add-label stale
  gh_write pr comment "$number" --repo "$REPO" --body-file "$tmp/comment.md"
  gh_write pr close "$number" --repo "$REPO"
  closed=$((closed + 1))
done < "$tmp/stale.tsv"

printf 'OK: %d stale pull request(s) processed (cutoff %s).\n' "$closed" "$cutoff"
