#!/usr/bin/env bash
# Verificacion offline del KPI del canal rapido (sin red, sin secretos).
# Ejecutar desde cualquier directorio; usa el install.sh del repo.
set -uo pipefail

ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null || pwd)"
WORK="$(mktemp -d /tmp/opencode/ag-kpi-XXXXXX)"
REL="$WORK/release"
BIN="$WORK/bin"
STUBS="$WORK/stubs"
mkdir -p "$REL" "$BIN" "$STUBS"
trap 'rm -rf "$WORK"' EXIT

printf '#!/bin/sh\necho "ag 0.1.0-alpha.2 (synthetic)"\n' > "$REL/ag-x86_64-unknown-linux-gnu"
chmod 0755 "$REL/ag-x86_64-unknown-linux-gnu"
(cd "$REL" && sha256sum ag-x86_64-unknown-linux-gnu > checksums.txt)

for tool in cargo rustc; do
  cat > "$STUBS/$tool" <<EOF
#!/bin/sh
echo "$tool \$*" >> "\$AG_QA_CMD_LOG"
exit 97
EOF
  chmod 0755 "$STUBS/$tool"
done
: > "$WORK/cmd.log"

cd "$ROOT" || exit 99

echo "=== 1. happy path (file://, headless, stubs de cargo/rustc) ==="
PATH="$STUBS:$PATH" AG_QA_CMD_LOG="$WORK/cmd.log" \
  AG_INSTALLER_BASE_URL="file://$REL" AG_INSTALLER_BIN_DIR="$BIN" \
  AG_INSTALLER_PROGRESS=headless \
  bash -c 'TIMEFORMAT="ELAPSED real=%3R"; time bash install.sh' > "$WORK/out1.txt" 2>&1
rc1=$?
cat "$WORK/out1.txt"
echo "rc=$rc1"

echo "=== 2. determinismo headless (segunda ejecucion) ==="
PATH="$STUBS:$PATH" AG_QA_CMD_LOG="$WORK/cmd.log" \
  AG_INSTALLER_BASE_URL="file://$REL" AG_INSTALLER_BIN_DIR="$BIN" \
  AG_INSTALLER_PROGRESS=headless bash install.sh > "$WORK/out2.txt" 2>&1
rc2=$?
grep -v '^ELAPSED ' "$WORK/out1.txt" > "$WORK/clean1.txt"
grep -v '^ELAPSED ' "$WORK/out2.txt" > "$WORK/clean2.txt"
if diff -q "$WORK/clean1.txt" "$WORK/clean2.txt" >/dev/null; then echo "DETERMINISTA: salidas identicas (excluyendo ELAPSED del cronometro)"; else echo "DIVERGENCE"; diff -u "$WORK/clean1.txt" "$WORK/clean2.txt" | head -30; fi

echo "=== 3. compilaciones detectadas ==="
n_calls="$(grep -c . "$WORK/cmd.log" || true)"
echo "invocaciones cargo/rustc=$n_calls"
[ "$n_calls" = "0" ] && echo "OK: 0 compilaciones" || { echo "FALLO: hubo compilaciones"; cat "$WORK/cmd.log"; }

echo "=== 4. binario instalado ==="
if [ -x "$BIN/ag" ]; then echo "OK: $BIN/ag ejecutable"; else echo "FALLO: no hay binario"; fi

echo "=== 5. ANSI en headless ==="
esc1="$(python3 - "$WORK/out1.txt" <<'PY'
import sys
print(sum(1 for b in open(sys.argv[1],'rb').read() if b == 0x1b))
PY
)"
echo "bytes ESC en headless=$esc1"
[ "$esc1" = "0" ] && echo "OK: sin ANSI" || echo "FALLO: ANSI en headless"

echo "=== 6. fail-closed: checksum invalido ==="
BAD="$WORK/bad"
mkdir -p "$BAD" "$WORK/bin-bad"
cp "$REL/ag-x86_64-unknown-linux-gnu" "$BAD/"
printf '%064d  %s\n' 0 ag-x86_64-unknown-linux-gnu > "$BAD/checksums.txt"
AG_INSTALLER_BASE_URL="file://$BAD" AG_INSTALLER_BIN_DIR="$WORK/bin-bad" \
  AG_INSTALLER_PROGRESS=headless bash install.sh > "$WORK/bad.out" 2>&1
rcbad=$?
tail -5 "$WORK/bad.out"
echo "rc=$rcbad"
if [ "$rcbad" -ne 0 ] && [ ! -e "$WORK/bin-bad/ag" ]; then echo "OK: fail-closed sin binario"; else echo "FALLO: fail-closed"; fi

echo "=== 7. target no publicado ==="
AG_INSTALLER_TARGET="x86_64-pc-windows-msvc" AG_INSTALLER_BASE_URL="file://$REL" \
  AG_INSTALLER_BIN_DIR="$WORK/bin" AG_INSTALLER_PROGRESS=headless \
  bash install.sh > "$WORK/target.out" 2>&1
rctarget=$?
tail -3 "$WORK/target.out"
echo "rc=$rctarget"

echo "=== RESUMEN ==="
echo "happy_rc=$rc1 second_rc=$rc2 bad_rc=$rcbad target_rc=$rctarget cargo_calls=$n_calls esc_headless=$esc1"
