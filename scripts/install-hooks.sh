#!/usr/bin/env bash
# Enable the Anti-Gravital local git hooks for the current repository clone.
#
# Opt-in and local only: this writes core.hooksPath in the local git config,
# never in the global config. It downloads nothing and needs no privileges.
#
# Usage:  bash scripts/install-hooks.sh
# Remove: git config --unset core.hooksPath
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
    echo "install-hooks: run this from inside a git repository." >&2
    exit 1
fi
cd "$ROOT"

MISSING=0
for required in .githooks/commit-msg .githooks/pre-push scripts/ag-check; do
    if [ ! -f "$required" ]; then
        echo "install-hooks: missing required file $required." >&2
        MISSING=1
    fi
done
[ "$MISSING" -eq 0 ] || exit 1

chmod +x .githooks/commit-msg .githooks/pre-push scripts/ag-check
git config core.hooksPath .githooks

echo "Hooks enabled for this clone (core.hooksPath=.githooks)."
echo "commit-msg: enforces subject length, AI-attribution and emoji policy."
echo "pre-push:   runs scripts/ag-check before every push."
echo "Run 'scripts/ag-check' any time to verify without pushing."
echo "To disable: git config --unset core.hooksPath"
echo "To re-enable: bash scripts/install-hooks.sh"
