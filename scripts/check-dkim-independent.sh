#!/usr/bin/env bash
# Opt-in independent DKIM verification for ag-mail (issue #178).
#
# The ignored unit test signs the embedded known-answer vectors with mail-auth
# 0.15 and verifies both the recorded 0.9 signature and the fresh 0.15 one with
# dkimpy, a separate implementation. This wrapper adds explicit outcomes:
#
#   PASS  dkimpy + PyNaCl are available and both vectors verified.
#   SKIP  python3, dkimpy or PyNaCl is unavailable. Never reported as PASS.
#   FAIL  a vector was rejected, the verifier errored, or the test did not run.
#
# Exit status is 0 for PASS and SKIP, 1 for FAIL. A SKIP is not a pass: the
# result line is unambiguous and the calling job must not read exit 0 as
# verified DKIM evidence.
#
# This check is deliberately opt-in. Default CI does not install Python test
# tooling. See docs/security/DKIM_INDEPENDENT_VERIFICATION.md,
# docs/rfc/RFC-0035-independent-dkim-ci.md and docs/audits/dkim-independent-ci.md.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

test_name="sender::mta::dkim::tests::dkim_signatures_verify_with_dkimpy"
expected_pass_lines=2

report() { printf '%s\n' "$*"; }

# --- tool versions and dependency detection ----------------------------------
if ! command -v python3 >/dev/null 2>&1; then
    report "tool.python3=missing"
    report "RESULT: SKIP reason=python3-not-found detail=independent verification unavailable; a missing dependency is never PASS"
    exit 0
fi

python_version="$(python3 --version 2>&1)"
cargo_version="$(cargo --version 2>&1 || printf 'missing')"
rustc_version="$(rustc --version 2>&1 || printf 'missing')"

# dkimpy 1.1.8 silently returns False for ed25519 signatures when PyNaCl is
# absent, so import both modules in a single probe and treat either miss as a
# SKIP rather than a verifier verdict.
probe="$(
    python3 - <<'PY' 2>&1
import importlib
import importlib.metadata as metadata

for module, distribution in (("dkim", "dkimpy"), ("nacl", "pynacl")):
    try:
        importlib.import_module(module)
    except ImportError:
        print(f"MISSING {distribution}")
        continue
    try:
        version = metadata.version(distribution)
    except metadata.PackageNotFoundError:
        version = "unknown"
    print(f"VERSION {distribution} {version}")
PY
)"

dkimpy_version="$(printf '%s\n' "$probe" | sed -n 's/^VERSION dkimpy //p')"
pynacl_version="$(printf '%s\n' "$probe" | sed -n 's/^VERSION pynacl //p')"

report "tool.python3=$python_version"
report "tool.cargo=$cargo_version"
report "tool.rustc=$rustc_version"
report "tool.dkimpy=${dkimpy_version:-missing}"
report "tool.pynacl=${pynacl_version:-missing}"

if printf '%s\n' "$probe" | grep -q '^MISSING '; then
    report "RESULT: SKIP reason=missing-dependency detail=dkimpy and PyNaCl are both required; missing dependency is never PASS"
    exit 0
fi

# --- run the embedded vectors through the independent verifier ---------------
command_line="cargo test --locked -p ag-mail --features mta --lib $test_name -- --exact --ignored --nocapture"
report "command=$command_line"

if output="$(cargo test --locked -p ag-mail --features mta --lib "$test_name" \
    -- --exact --ignored --nocapture 2>&1)"; then
    test_status=0
else
    test_status=$?
fi

pass_count="$(printf '%s\n' "$output" | grep -c '^dkim=pass ' || true)"
reject_lines="$(printf '%s\n' "$output" | grep -E '^dkim=(fail|error) ' || true)"

# --exact makes zero-test runs possible to detect; require both vector lines
# and the cargo exit status before declaring PASS.
if [ "$test_status" -eq 0 ] && [ "$pass_count" -ge "$expected_pass_lines" ] && [ -z "$reject_lines" ]; then
    report "cargo.status=$test_status"
    report "vectors.verified=$pass_count"
    report "RESULT: PASS reason=dkimpy-verified-recorded-0-9-and-fresh-0-15-vectors"
    exit 0
fi

report "cargo.status=$test_status"
report "vectors.pass_lines=$pass_count"
if [ -n "$reject_lines" ]; then
    printf '%s\n' "$reject_lines" | while IFS= read -r line; do
        report "reject.$line"
    done
fi
report "--- verifier output (start) ---"
printf '%s\n' "$output"
report "--- verifier output (end) ---"
report "RESULT: FAIL reason=dkim-independent-check-failed"
exit 1
