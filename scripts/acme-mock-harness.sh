#!/usr/bin/env bash
# Run the local ACME order-flow mock harness and capture its JSON-line output.
#
# This is offline tooling for issue #87: it exercises the same order states as
# the credentialed staging run (new order, authorization pending/valid,
# challenge, finalize, download) but contacts no CA and proves no domain
# control. The downloaded chain is self-signed and generated on this machine.
#
# Usage:
#   scripts/acme-mock-harness.sh [OUTPUT_LOG]
#
# With OUTPUT_LOG, output is written to that file and echoed to stdout via tee.
# Without it, output goes to stdout only. Exit code is the harness result.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
harness="$repo_root/scripts/acme-mock-harness.py"

for tool in python3 openssl; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "missing required tool: $tool" >&2
        exit 2
    fi
done

if [[ $# -gt 1 ]]; then
    echo "usage: $0 [OUTPUT_LOG]" >&2
    exit 2
fi

if [[ $# -eq 1 ]]; then
    out="$1"
    mkdir -p "$(dirname "$out")"
    python3 "$harness" | tee "$out"
else
    python3 "$harness"
fi
