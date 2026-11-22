#!/usr/bin/env bash
# Enforce RFC-0022 section 4.6: derived documents must match the masters.
# Regeneration happens in a scratch copy so this check never touches the
# working tree. The masters themselves are hash-pinned by the docs workflow.
set -euo pipefail

REPO_ROOT="${AG_REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$REPO_ROOT"

MASTERS=(
  "docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md"
  "docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md"
)

for required in tools/split-masters.sh "${MASTERS[@]}"; do
  if [[ ! -f "$required" ]]; then
    printf 'ERROR: required input for the derived-doc check is missing: %s\n' "$required" >&2
    exit 1
  fi
done

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/tools" "$scratch/docs/master"
cp tools/split-masters.sh "$scratch/tools/split-masters.sh"
for master in "${MASTERS[@]}"; do
  cp "$master" "$scratch/docs/master/$(basename "$master")"
done

if ! bash "$scratch/tools/split-masters.sh" >/dev/null; then
  printf 'ERROR: tools/split-masters.sh failed; derived documents cannot be verified.\n' >&2
  exit 1
fi

status=0
compared=0
for section in architecture roadmap; do
  for generated in "$scratch/docs/$section"/*.md; do
    [[ -e "$generated" ]] || continue
    name="$(basename "$generated")"
    committed="docs/$section/$name"
    compared=$((compared + 1))
    if [[ ! -f "$committed" ]]; then
      printf 'ERROR: generated derived document is missing from the repository: %s\n' "$committed" >&2
      printf 'Fix: run "bash tools/split-masters.sh" and commit the new file.\n' >&2
      status=1
      continue
    fi
    if ! cmp -s "$generated" "$committed"; then
      printf 'ERROR: derived document drift detected: %s\n' "$committed" >&2
      printf 'Fix: run "bash tools/split-masters.sh" and commit the result. If a master section moved, recalculate the line ranges in tools/split-masters.sh in the same commit.\n' >&2
      diff -u "$committed" "$generated" 2>/dev/null | head -n 40 >&2 || true
      status=1
    fi
  done
done

if [[ "$status" -ne 0 ]]; then
  exit 1
fi

printf 'OK: %d derived documents match the master documents.\n' "$compared"
