#!/usr/bin/env bash
# Standalone tests for the local hook workflow. No network and no Rust toolchain
# are needed: cargo-backed checks are redirected through AG_*_CMD overrides.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
PASS=0
FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ok() { printf 'ok - %s\n' "$1"; PASS=$((PASS + 1)); }
not_ok() { printf 'not ok - %s: %s\n' "$1" "$2" >&2; FAIL=$((FAIL + 1)); }
expect_rc() {
    local want=$1 got=$2 label=$3 out=$4
    if [ "$got" -eq "$want" ]; then
        ok "$label (exit $got)"
    else
        not_ok "$label" "expected exit $want, got $got; output: $(head -c 400 "$out")"
    fi
}

# --- commit-msg -------------------------------------------------------------
printf 'feat(cli): add check command\n\nBody line.\n' > "$TMP/valid.msg"
bash "$REPO/.githooks/commit-msg" "$TMP/valid.msg" >"$TMP/out" 2>&1
expect_rc 0 $? "commit-msg accepts a valid message" "$TMP/out"

printf '%257s' '' | tr ' ' 'a' > "$TMP/long-subject"
printf '%s\n' "$(cat "$TMP/long-subject")" > "$TMP/long.msg"
bash "$REPO/.githooks/commit-msg" "$TMP/long.msg" >"$TMP/out" 2>&1
expect_rc 1 $? "commit-msg rejects a 257-character subject" "$TMP/out"
grep -q 'limit is 256' "$TMP/out" && ok "commit-msg reports the 256 limit" || not_ok "commit-msg limit" "missing message"

# Prohibited literals are assembled at runtime so this test file itself stays
# clean under the repository's prohibited-content scan.
AI_LINE="$(printf 'Co-Authored-%s: Clau%s' 'By' 'de')"
printf 'feat: x\n\n%s\n' "$AI_LINE" > "$TMP/ai.msg"
bash "$REPO/.githooks/commit-msg" "$TMP/ai.msg" >"$TMP/out" 2>&1
expect_rc 1 $? "commit-msg rejects AI attribution" "$TMP/out"

printf 'feat: x \360\237\230\200\n' > "$TMP/emoji.msg"
bash "$REPO/.githooks/commit-msg" "$TMP/emoji.msg" >"$TMP/out" 2>&1
expect_rc 1 $? "commit-msg rejects emojis" "$TMP/out"

bash "$REPO/.githooks/commit-msg" "$TMP/missing.msg" >"$TMP/out" 2>&1
expect_rc 1 $? "commit-msg rejects a missing message file" "$TMP/out"

# --- ag-check list and honesty ----------------------------------------------
LIST="$(bash "$REPO/scripts/ag-check" --list)"
for name in descriptors fmt clippy test audit deny masters prohibited; do
    printf '%s\n' "$LIST" | grep -qx "$name" \
        && ok "ag-check --list includes $name" \
        || not_ok "ag-check --list" "missing $name"
done

bash "$REPO/scripts/ag-check" --bogus >"$TMP/out" 2>&1
expect_rc 2 $? "ag-check rejects an unknown argument" "$TMP/out"

# --- descriptor checks in a throwaway git repo ------------------------------
D="$TMP/descriptor-repo"
mkdir -p "$D"
git -C "$D" init -q
git -C "$D" checkout -q -b topic/foo
mkdir -p "$D/docs/pr-drafts"

AG_CHECK_ROOT="$D" bash "$REPO/scripts/ag-check" --only descriptors >"$TMP/out" 2>&1
expect_rc 1 $? "descriptor check fails on a branch without a descriptor" "$TMP/out"
grep -q 'docs/pr-drafts/topic-foo.md' "$TMP/out" \
    && ok "descriptor failure names the expected file" \
    || not_ok "descriptor failure" "expected path not reported"

cat > "$D/docs/pr-drafts/topic-foo.md" <<'EOF'
## Resumen

Add a local verification gate.

## Plan de prueba

Run tests/hooks/test-hooks.sh.

## Cierre de issues

Closes: none
EOF
AG_CHECK_ROOT="$D" bash "$REPO/scripts/ag-check" --only descriptors >"$TMP/out" 2>&1
expect_rc 0 $? "descriptor check passes with a valid descriptor" "$TMP/out"

grep -v 'Closes: none' "$D/docs/pr-drafts/topic-foo.md" > "$TMP/no-close.md"
cp "$TMP/no-close.md" "$D/docs/pr-drafts/topic-foo.md"
AG_CHECK_ROOT="$D" bash "$REPO/scripts/ag-check" --only descriptors >"$TMP/out" 2>&1
expect_rc 1 $? "descriptor check rejects a missing closing keyword" "$TMP/out"

cat > "$D/docs/pr-drafts/topic-foo.md" <<'EOF'
## Resumen

Add a local verification gate.

## Plan de prueba

Run tests/hooks/test-hooks.sh.

## Cierre de issues

Closes: none
EOF

# Honesty: a skipped check must never be printed as passed.
AG_CHECK_ROOT="$D" bash "$REPO/scripts/ag-check" --only descriptors >"$TMP/out" 2>&1
if grep -q 'SKIP: fmt' "$TMP/out" && ! grep -q 'PASS: fmt' "$TMP/out"; then
    ok "skipped checks are reported as SKIP, never as PASS"
else
    not_ok "skip honesty" "output: $(head -c 400 "$TMP/out")"
fi
grep -q 'Summary: PASS=1 FAIL=0 SKIP=7' "$TMP/out" \
    && ok "summary reflects one selected check and seven skips" \
    || not_ok "summary" "unexpected counts: $(tail -n3 "$TMP/out")"

# A descriptor on the default branch is not required.
git -C "$D" checkout -q -b main 2>/dev/null || git -C "$D" checkout -q main
rm -f "$D/docs/pr-drafts/topic-foo.md"
AG_CHECK_ROOT="$D" bash "$REPO/scripts/ag-check" --only descriptors >"$TMP/out" 2>&1
expect_rc 0 $? "descriptor check does not require one on the default branch" "$TMP/out"

# --- masters hashes ----------------------------------------------------------
M="$TMP/masters-repo"
mkdir -p "$M/docs/master"
cp "$REPO"/docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf "$M/docs/master/"
cp "$REPO"/docs/master/ANTI-GRAVITAL-Blueprint-v4.1.md "$M/docs/master/"
cp "$REPO"/docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md "$M/docs/master/"
cp "$REPO"/docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md "$M/docs/master/"
AG_CHECK_ROOT="$M" bash "$REPO/scripts/ag-check" --only masters >"$TMP/out" 2>&1
expect_rc 0 $? "masters check passes on unmodified documents" "$TMP/out"
printf '\n' >> "$M/docs/master/ANTI-GRAVITAL-Blueprint-v4.1.md"
AG_CHECK_ROOT="$M" bash "$REPO/scripts/ag-check" --only masters >"$TMP/out" 2>&1
expect_rc 1 $? "masters check fails on a tampered document" "$TMP/out"

# --- prohibited content ------------------------------------------------------
P="$TMP/policy-repo"
mkdir -p "$P"
printf '# clean\n' > "$P/README.md"
AG_CHECK_ROOT="$P" bash "$REPO/scripts/ag-check" --only prohibited >"$TMP/out" 2>&1
expect_rc 0 $? "prohibited-content check passes on clean content" "$TMP/out"
printf 'emoji \360\237\230\200\n' > "$P/emoji.md"
AG_CHECK_ROOT="$P" bash "$REPO/scripts/ag-check" --only prohibited >"$TMP/out" 2>&1
expect_rc 1 $? "prohibited-content check fails on an emoji" "$TMP/out"

# No detector available: must fail loudly instead of reporting a silent PASS.
BIN="$TMP/no-detector-bin"
mkdir -p "$BIN"
for c in grep find git bash sed awk tr cat head; do
    target="$(command -v "$c" 2>/dev/null)" || continue
    ln -sf "$target" "$BIN/$c"
done
cat > "$BIN/perl" <<'WRAPPER'
#!/usr/bin/env bash
exit 127
WRAPPER
rm -f "$BIN/grep"
cat > "$BIN/grep" <<'WRAPPER'
#!/usr/bin/env bash
for arg in "$@"; do
    case "$arg" in
        -P|-qP|-IPq) exit 2 ;;
    esac
done
exec /usr/bin/grep "$@"
WRAPPER
chmod +x "$BIN/perl" "$BIN/grep"
PATH="$BIN" AG_CHECK_ROOT="$P" bash "$REPO/scripts/ag-check" --only prohibited >"$TMP/out" 2>&1
expect_rc 1 $? "prohibited-content check fails when no emoji detector exists" "$TMP/out"
grep -q 'no emoji detector available' "$TMP/out" \
    && ok "missing emoji detector is reported explicitly" \
    || not_ok "missing emoji detector" "no explicit failure message"

# --- cargo-backed command overrides -----------------------------------------
AG_TEST_CMD='false' bash "$REPO/scripts/ag-check" --only test >"$TMP/out" 2>&1
expect_rc 1 $? "a failing test command fails the check" "$TMP/out"
AG_TEST_CMD='true' bash "$REPO/scripts/ag-check" --only test >"$TMP/out" 2>&1
expect_rc 0 $? "a passing test command passes the check" "$TMP/out"

# --- install and removal -----------------------------------------------------
I="$TMP/install-repo"
mkdir -p "$I"
git -C "$I" init -q
cp -R "$REPO/.githooks" "$I/.githooks"
mkdir -p "$I/scripts"
cp "$REPO/scripts/ag-check" "$I/scripts/ag-check"
(cd "$I" && bash "$REPO/scripts/install-hooks.sh") >"$TMP/out" 2>&1
expect_rc 0 $? "install-hooks.sh succeeds in a local clone" "$TMP/out"
[ "$(git -C "$I" config --local core.hooksPath)" = '.githooks' ] \
    && ok "install sets core.hooksPath locally" \
    || not_ok "install" "core.hooksPath is $(git -C "$I" config --local core.hooksPath)"
[ -x "$I/.githooks/commit-msg" ] && [ -x "$I/.githooks/pre-push" ] \
    && ok "install marks hooks executable" \
    || not_ok "install" "hooks are not executable"
[ -z "$(git -C "$REPO" config --local core.hooksPath 2>/dev/null || true)" ] \
    && ok "install does not touch the working repository config" \
    || not_ok "install" "working repository core.hooksPath was changed"
git -C "$I" config --unset core.hooksPath
[ -z "$(git -C "$I" config --local core.hooksPath 2>/dev/null || true)" ] \
    && ok "documented removal command clears core.hooksPath" \
    || not_ok "removal" "core.hooksPath still set"

# pre-push propagates the ag-check exit code without a silent bypass.
printf '#!/usr/bin/env bash\nexit 7\n' > "$I/scripts/ag-check"
chmod +x "$I/scripts/ag-check"
(cd "$I" && .githooks/pre-push) >"$TMP/out" 2>&1
expect_rc 7 $? "pre-push propagates a failing ag-check exit code" "$TMP/out"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
