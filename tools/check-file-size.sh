#!/usr/bin/env bash
# Enforce RFC-0022 section 4.3: a tracked Rust source file over 600 lines fails
# unless it is listed in tools/file-size-allowlist.txt with an issue number.
# The allowlist may only shrink, so entries for files that are back under the
# limit (or no longer tracked) are failures too.
set -euo pipefail

REPO_ROOT="${AG_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$REPO_ROOT"

MAX_LINES="${AG_MAX_LINES:-600}"
ALLOWLIST="${AG_ALLOWLIST:-tools/file-size-allowlist.txt}"

if [[ ! -f "$ALLOWLIST" ]]; then
  printf 'ERROR: file-size allowlist not found: %s\n' "$ALLOWLIST" >&2
  exit 1
fi

declare -A allowed=()
line_no=0
while IFS= read -r line || [[ -n "$line" ]]; do
  line_no=$((line_no + 1))
  trimmed="${line#"${line%%[![:space:]]*}"}"
  if [[ -z "$trimmed" || "$trimmed" == \;* ]]; then
    continue
  fi
  path="${trimmed%%[[:space:]]*}"
  ref="${trimmed#"$path"}"
  ref="${ref#"${ref%%[![:space:]]*}"}"
  if [[ ! "$ref" =~ ^#[0-9]+$ ]]; then
    printf 'ERROR: %s:%d: entry needs a path and an issue number, e.g. "path/to/file.rs #123".\n' \
      "$ALLOWLIST" "$line_no" >&2
    exit 1
  fi
  if [[ -n "${allowed[$path]:-}" ]]; then
    printf 'ERROR: %s:%d: duplicate allowlist entry for %s.\n' "$ALLOWLIST" "$line_no" "$path" >&2
    exit 1
  fi
  allowed["$path"]="$ref"
done < "$ALLOWLIST"

status=0
checked=0
kept=()
while IFS= read -r -d '' file; do
  checked=$((checked + 1))
  lines="$(wc -l < "$file")"
  lines="${lines//[[:space:]]/}"
  if (( lines > MAX_LINES )); then
    if [[ -n "${allowed[$file]:-}" ]]; then
      kept+=("$file")
      continue
    fi
    printf 'ERROR: %s has %s lines (limit %s) and is not in %s.\n' \
      "$file" "$lines" "$MAX_LINES" "$ALLOWLIST" >&2
    printf 'Fix: split the file, or add "%s #NNN" to the allowlist with the issue tracking the refactor.\n' \
      "$file" >&2
    status=1
  fi
done < <(git ls-files -z '*.rs')

for path in "${!allowed[@]}"; do
  stale=1
  for used in "${kept[@]:-}"; do
    if [[ "$used" == "$path" ]]; then
      stale=0
      break
    fi
  done
  if [[ "$stale" -ne 0 ]]; then
    printf 'ERROR: allowlist entry %s is stale (file is now <= %s lines or no longer tracked).\n' \
      "$path" "$MAX_LINES" >&2
    printf 'Fix: remove the entry. The allowlist may only shrink.\n' >&2
    status=1
  fi
done

if [[ "$status" -ne 0 ]]; then
  exit 1
fi

printf 'OK: %d tracked Rust files checked, %d allowed exceptions, none over %d lines.\n' \
  "$checked" "${#allowed[@]}" "$MAX_LINES"
