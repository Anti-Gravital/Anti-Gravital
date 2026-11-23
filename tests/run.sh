#!/usr/bin/env bash
# Selftests deterministicos del framework Kaizen instalado en este repo.
# No usa red, secretos ni toolchain de Rust: verifica la integridad del gate
# local (scripts/ag-check) y la sintaxis de los scripts del framework.
# Todo check omitido se reporta, nunca se cuenta como PASS (L-10).
set -euo pipefail

# Guardia anti-recursion: `kaizen check` ejecuta KZ_TEST_CMD = tests/run.sh.
if [ "${KZ_RUNSH_TESTING:-0}" = "1" ]; then exit 0; fi
export KZ_RUNSH_TESTING=1

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PASS=0
FAILN=0
ok()  { PASS=$((PASS + 1)); printf 'ok   %s\n' "$1"; }
bad() { FAILN=$((FAILN + 1)); printf 'FALLA %s\n' "$1" >&2; }

# 1) Archivos canonicos del framework presentes.
for f in .kaizen/RULES.md .kaizen/loop/LOOP.md .kaizen/security/SECURITY-RULES.md \
         scripts/kaizen scripts/ag-check scripts/kaizen-security scripts/kaizen-swarm \
         scripts/kaizen-team tests/hooks/test-hooks.sh; do
  [ -f "$f" ] && ok "existe $f" || bad "falta $f"
done

# 2) Sintaxis de los scripts del gate, incluido este propio runner.
for f in scripts/kaizen scripts/ag-check scripts/kaizen-security scripts/kaizen-swarm \
         scripts/kaizen-team tests/hooks/test-hooks.sh tests/run.sh; do
  bash -n "$f" && ok "sintaxis $f" || bad "sintaxis $f"
done

# 3) Modulo Python del framework compila (solo parseo, sin artefactos .pyc).
python3 -c "import ast; ast.parse(open('scripts/kaizen-insights.py', encoding='utf-8').read())" \
  && ok "sintaxis scripts/kaizen-insights.py" || bad "sintaxis scripts/kaizen-insights.py"

# 4) ag-check expone las ocho compuertas canonicas.
LIST="$(bash scripts/ag-check --list)"
for name in descriptors fmt clippy test audit deny masters prohibited; do
  printf '%s\n' "$LIST" | grep -qx "$name" && ok "ag-check --list: $name" || bad "ag-check --list: $name"
done

# 5) Honestidad del gate: un fallo nunca se reporta como PASS y un skip nunca
#    se reporta como PASS. Los checks se fuerzan con AG_TEST_CMD para no
#    depender de cargo, red ni del estado del repositorio.
if AG_TEST_CMD='false' bash scripts/ag-check --only test >/dev/null 2>&1; then
  bad "test command con fallo debe devolver exit 1"
else
  ok "test command con fallo devuelve exit 1"
fi
if AG_TEST_CMD='true' bash scripts/ag-check --only test >/dev/null 2>&1; then
  ok "test command con exito devuelve exit 0"
else
  bad "test command con exito debe devolver exit 0"
fi
if bash scripts/ag-check --bogus >/dev/null 2>&1; then
  bad "argumento desconocido debe devolver exit 2"
else
  ok "argumento desconocido devuelve exit 2"
fi
SKIPOUT="$(AG_TEST_CMD='true' bash scripts/ag-check --only test 2>&1)"
if printf '%s\n' "$SKIPOUT" | grep -q 'SKIP: fmt' \
   && ! printf '%s\n' "$SKIPOUT" | grep -q 'PASS: fmt'; then
  ok "checks omitidos se reportan SKIP, nunca PASS"
else
  bad "honestidad de checks omitidos"
fi

printf '\n%d pruebas pasadas, %d falladas\n' "$PASS" "$FAILN"
[ "$FAILN" -eq 0 ]
