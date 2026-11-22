#!/usr/bin/env bash
# Standalone tests for the RFC-0022 guardrail scripts. No cargo, no network,
# no real GitHub calls: a fake gh CLI records every invocation.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

failures=0
pass() { printf 'PASS: %s\n' "$1"; }
fail() { printf 'FAIL: %s\n' "$1" >&2; failures=$((failures + 1)); }
expect_status() {
  if [[ "$2" == "$3" ]]; then pass "$1 (exit $2)"; else fail "$1 (expected exit $3, got $2)"; fi
}
expect_contains() {
  if printf '%s' "$1" | grep -Fq -- "$2"; then pass "$3"; else fail "$3 (missing: $2)"; fi
}
expect_not_contains() {
  if printf '%s' "$1" | grep -Fq -- "$2"; then fail "$3 (unexpected: $2)"; else pass "$3"; fi
}
log() { cat "$1" 2>/dev/null || true; }

printf '== file-size allowlist ==\n'
fs="$tmp/filesize"
mkdir -p "$fs"
(
  cd "$fs"
  git init -q
  printf 'fn main() {}\n' > small.rs
  seq 1 700 > big.rs
  git add small.rs big.rs
)
printf '; test allowlist\n' > "$fs/allowlist.txt"

out="$(AG_REPO_ROOT="$fs" AG_ALLOWLIST=allowlist.txt bash tools/check-file-size.sh 2>&1)"; status=$?
expect_status 'oversized file rejected' "$status" 1
expect_contains "$out" 'big.rs has 700 lines' 'oversized file message names the file and size'
expect_contains "$out" '#NNN' 'failure message explains the allowlist format'

printf 'big.rs #142\n' >> "$fs/allowlist.txt"
out="$(AG_REPO_ROOT="$fs" AG_ALLOWLIST=allowlist.txt bash tools/check-file-size.sh 2>&1)"; status=$?
expect_status 'allowlisted file accepted' "$status" 0
expect_contains "$out" '1 allowed exceptions' 'success message reports the exception count'

(
  cd "$fs"
  printf 'fn main() {}\n' > big.rs
  git add big.rs
)
out="$(AG_REPO_ROOT="$fs" AG_ALLOWLIST=allowlist.txt bash tools/check-file-size.sh 2>&1)"; status=$?
expect_status 'stale allowlist entry rejected' "$status" 1
expect_contains "$out" 'big.rs is stale' 'stale entry message names the file'

printf 'big.rs not-a-ref\n' > "$fs/allowlist.txt"
out="$(AG_REPO_ROOT="$fs" AG_ALLOWLIST=allowlist.txt bash tools/check-file-size.sh 2>&1)"; status=$?
expect_status 'malformed allowlist reference rejected' "$status" 1
expect_contains "$out" 'issue number' 'malformed reference message explains the format'

printf '== derived-doc drift ==\n'
dd="$tmp/derived"
mkdir -p "$dd/tools" "$dd/docs/master" "$dd/docs/architecture" "$dd/docs/roadmap"
cat > "$dd/tools/split-masters.sh" <<'FIXTURE'
#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$root/docs/architecture" "$root/docs/roadmap"
printf '# Arch\n\nsource line\n' > "$root/docs/architecture/a.md"
printf '# Road\n\nsource line\n' > "$root/docs/roadmap/b.md"
FIXTURE
printf 'master\n' > "$dd/docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md"
printf 'master\n' > "$dd/docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md"
printf '# Arch\n\nsource line\n' > "$dd/docs/architecture/a.md"
printf '# Road\n\nsource line\n' > "$dd/docs/roadmap/b.md"

out="$(AG_REPO_ROOT="$dd" bash tools/check-derived-docs.sh 2>&1)"; status=$?
expect_status 'matching derived docs accepted' "$status" 0
expect_contains "$out" '2 derived documents match' 'success message reports the document count'

printf '# Arch\n\ndrifted line\n' > "$dd/docs/architecture/a.md"
out="$(AG_REPO_ROOT="$dd" bash tools/check-derived-docs.sh 2>&1)"; status=$?
expect_status 'drifted derived doc rejected' "$status" 1
expect_contains "$out" 'drift detected' 'drift message identifies the problem'
expect_contains "$out" 'split-masters.sh' 'drift message explains the fix'

printf '# Arch\n\nsource line\n' > "$dd/docs/architecture/a.md"
rm "$dd/docs/roadmap/b.md"
out="$(AG_REPO_ROOT="$dd" bash tools/check-derived-docs.sh 2>&1)"; status=$?
expect_status 'missing derived doc rejected' "$status" 1
expect_contains "$out" 'missing from the repository' 'missing file message identifies the problem'

printf '== scheduled security issue ==\n'
mkdir -p "$tmp/fakebin"
cat > "$tmp/fakebin/gh" <<'FIXTURE'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
case "$1 $2" in
  "issue list") printf '%s\n' "${FAKE_ISSUE_NUMBER:-}" ;;
  "issue view")
    if [[ " $* " == *" --json state "* ]]; then
      printf '%s\n' "${FAKE_ISSUE_STATE:-OPEN}"
    else
      cat "${FAKE_ISSUE_BODY_FILE:-/dev/null}"
    fi
    ;;
  "issue create"|"issue edit")
    if [[ -n "${GH_BODY_SNAPSHOT:-}" ]]; then
      while [[ $# -gt 0 ]]; do
        if [[ "$1" == "--body-file" ]]; then shift; cp "$1" "$GH_BODY_SNAPSHOT"; fi
        shift
      done
    fi
    ;;
esac
exit 0
FIXTURE
chmod +x "$tmp/fakebin/gh"
printf 'VULN-FIXTURE advisory\n' > "$tmp/audit.out"

run_report() {
  GH_BIN="$tmp/fakebin/gh" GITHUB_REPOSITORY=acme/repo GITHUB_SHA=abc123 \
  AUDIT_STATUS="$1" DENY_STATUS="$2" AUDIT_OUTPUT="$tmp/audit.out" \
  DENY_OUTPUT="$tmp/deny.out" GH_CALL_LOG="$3" \
  GH_BODY_SNAPSHOT="$4" FAKE_ISSUE_NUMBER="${5:-}" \
  FAKE_ISSUE_BODY_FILE="${6:-/dev/null}" FAKE_ISSUE_STATE="${7:-OPEN}" \
    bash tools/security-report.sh
}

run_report 1 0 "$tmp/calls1" "$tmp/body1" >/dev/null 2>&1
expect_contains "$(log "$tmp/calls1")" 'issue create' 'failing check opens a new issue'
expect_contains "$(log "$tmp/calls1")" '--label security' 'new issue carries the security label'
expect_contains "$(log "$tmp/calls1")" 'Scheduled security audit failures' 'new issue uses the fixed title'
expect_contains "$(cat "$tmp/body1")" 'VULN-FIXTURE' 'issue body includes the audit output'
expect_contains "$(cat "$tmp/body1")" 'ag-weekly-security-audit' 'issue body carries the ownership marker'
expect_not_contains "$(log "$tmp/calls1")" 'issue edit' 'no update when no issue exists'

printf '%s\n' '<!-- ag-weekly-security-audit -->' > "$tmp/marked-body"
run_report 1 1 "$tmp/calls2" "$tmp/body2" 55 "$tmp/marked-body" >/dev/null 2>&1
expect_contains "$(log "$tmp/calls2")" 'issue edit 55' 'existing marked issue is updated'
expect_contains "$(cat "$tmp/body2")" 'cargo deny check' 'issue body includes the deny failure'
expect_not_contains "$(log "$tmp/calls2")" 'issue create' 'no duplicate issue is opened'

run_report 1 0 "$tmp/calls2b" "$tmp/body2b" 55 "$tmp/marked-body" CLOSED >/dev/null 2>&1
expect_contains "$(log "$tmp/calls2b")" 'issue reopen 55' 'closed tracking issue is reopened on a new failure'
expect_contains "$(log "$tmp/calls2b")" 'issue edit 55' 'reopened issue is updated in place'
expect_not_contains "$(log "$tmp/calls2b")" 'issue create' 'no second issue is opened after a closed episode'

printf '%s\n' 'A human security issue, do not touch.' > "$tmp/human-body"
run_report 1 0 "$tmp/calls3" "$tmp/body3" 56 "$tmp/human-body" >/dev/null 2>&1
expect_contains "$(log "$tmp/calls3")" 'issue create' 'human issue is not reused'
expect_not_contains "$(log "$tmp/calls3")" 'issue edit 56' 'human issue is not edited'

run_report 0 0 "$tmp/calls4" "$tmp/body4" 55 "$tmp/marked-body" >/dev/null 2>&1
expect_contains "$(log "$tmp/calls4")" 'issue comment 55' 'passing run comments before closing'
expect_contains "$(log "$tmp/calls4")" 'issue close 55' 'passing run closes the tracking issue'

run_report 0 0 "$tmp/calls4b" "$tmp/body4b" 55 "$tmp/marked-body" CLOSED >/dev/null 2>&1
expect_not_contains "$(log "$tmp/calls4b")" 'issue close 55' 'passing run does not re-close an already closed issue'

run_report 0 0 "$tmp/calls5" "$tmp/body5" >/dev/null 2>&1
expect_not_contains "$(log "$tmp/calls5")" 'issue create' 'passing run with no issue is a no-op'
expect_not_contains "$(log "$tmp/calls5")" 'issue close' 'passing run with no issue closes nothing'

printf '== stale pull requests ==\n'
cat > "$tmp/fakebin/gh" <<'FIXTURE'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
case "$1 $2" in
  "pr list") printf '%b' "${FAKE_PR_ROWS:-}" ;;
esac
exit 0
FIXTURE
chmod +x "$tmp/fakebin/gh"

run_stale() {
  GH_BIN="$tmp/fakebin/gh" GITHUB_REPOSITORY=acme/repo \
  FAKE_PR_ROWS="$1" GH_CALL_LOG="$2" DRY_RUN="$3" \
    bash tools/close-stale-prs.sh
}

rows='42\tOld PR\t2026-09-01T00:00:00Z\n'
run_stale "$rows" "$tmp/stale-calls1" 0 >/dev/null 2>&1
expect_contains "$(log "$tmp/stale-calls1")" 'pr edit 42' 'stale PR gets the stale label'
expect_contains "$(log "$tmp/stale-calls1")" 'pr comment 42' 'stale PR gets a note'
expect_contains "$(log "$tmp/stale-calls1")" 'pr close 42' 'stale PR is closed'
expect_contains "$(log "$tmp/stale-calls1")" 'label create stale' 'stale label is ensured'

run_stale "$rows" "$tmp/stale-calls2" 1 > "$tmp/stale-out2" 2>&1
expect_not_contains "$(log "$tmp/stale-calls2")" 'pr close 42' 'dry run does not close PRs'
expect_contains "$(cat "$tmp/stale-out2")" 'DRY RUN' 'dry run reports intended actions'

run_stale '' "$tmp/stale-calls3" 0 > "$tmp/stale-out3" 2>&1
expect_contains "$(cat "$tmp/stale-out3")" '0 stale pull request(s) processed' 'empty result is a no-op'
expect_not_contains "$(log "$tmp/stale-calls3")" 'pr close' 'no PRs closed when none are stale'

if [[ "$failures" -ne 0 ]]; then
  printf '%d guardrail test(s) failed.\n' "$failures" >&2
  exit 1
fi
printf 'OK: all guardrail script tests passed.\n'
