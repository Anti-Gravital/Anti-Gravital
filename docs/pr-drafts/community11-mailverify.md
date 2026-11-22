# Wire an opt-in independent DKIM check and record honest evidence (#178)

- Fase afectada: 4.6 (ag-mail / MTA), verificacion tooling. No hay avance de
  fase, cambio de alcance ni modificacion del roadmap. La puerta de buzon real
  sigue abierta.
- Tipo de cambio: [x] documentacion y automatizacion de evidencia;
  [x] cambio de CI opt-in (RFC-0035 aceptada); [ ] codigo de producto;
  [ ] decision arquitectonica nueva.

## Resumen

Add scripts/check-dkim-independent.sh and a manual-only DKIM workflow with
PASS/SKIP/FAIL semantics, record a real run, and keep #178 open.

## Documentos relacionados

- `scripts/check-dkim-independent.sh` (nuevo): envoltorio con versiones exactas
  y salidas PASS/SKIP/FAIL; una dependencia ausente nunca se degrada a PASS.
- `.github/workflows/dkim-independent.yml` (nuevo): solo `workflow_dispatch`,
  sin trigger de push/PR; CI por defecto intacta y en verde.
- `docs/rfc/RFC-0035-independent-dkim-ci.md` (nuevo): RFC aceptada con periodo
  de comentarios dispensado, exigida por la regla 28 para el cambio de CI.
- `docs/rfc/README.md`: fila nueva en el indice de RFC.
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`: seccion 7 con el check
  opt-in, sus semanticas y el enlace a la evidencia.
- `docs/audits/dkim-independent-ci.md` (nuevo): corrida real, versiones,
  comandos, salida, prueba controlada del camino SKIP y condicion exacta de
  cierre de #178.
- `docs/pr-drafts/README.md`: fila nueva en el indice de descriptores.
- `CHANGELOG.md`: entrada bajo `[Unreleased]`.
- Issue #178 y verificacion de ola 2 (`tools/dkim_verify.py`).

## Plan de prueba

- `bash -n scripts/check-dkim-independent.sh` -> exit 0.
- Parseo YAML de `.github/workflows/dkim-independent.yml` -> dos documentos
  YAML validos (PyYAML), triggers = `workflow_dispatch` solamente.
- `bash scripts/check-dkim-independent.sh` con dkimpy 1.1.8 + PyNaCl 1.6.2 ->
  `vectors.verified=2`, `RESULT: PASS`, exit 0; salida grabada en la auditoria.
- Camino SKIP: `python3` sustituido por un stub que reporta ambos modulos
  ausentes -> `RESULT: SKIP`, nunca PASS, exit 0; grabado en la auditoria.
- `git diff --check` -> limpio.
- Escaneos de contenido de `.github/workflows/docs.yml`: atribucion de
  herramientas IA, URLs de sesion, prefijos de rama prohibidos, emojis y
  marcas retiradas (ADR-0011) -> sin hallazgos.
- `cargo test --locked -p ag-mail --features mta --lib ... -- --exact --ignored
  --nocapture` -> 1 test pasa, dos lineas `dkim=pass`.

## Criterios de salida que avanza

- Hace reproducible y verificable el item de "verificador independiente" del
  Done-when de #178, sin anadir dependencias Cargo ni tocar la CI por defecto.
- NO marca ninguna puerta como cerrada: `Fuzz 24h`, benchmarks y la puerta de
  correo real no se ven afectados. #178 permanece abierto con `blocked`.

## Cierre de issues

Closes: none
Refs #178

Ningun issue se cierra. #178 requiere ademas una entrega real con el header
`Authentication-Results` de un receptor (egreso de puerto 25), que no se puede
ejecutar en este entorno. El wrapper solo verifica vectores offline.

## Checklist

- [x] Cambio minimo y dentro de la fase correcta.
- [x] Documentacion actualizada en la misma rama (RFC, seguridad, auditoria,
      CHANGELOG e indices).
- [x] Sin evidencia falsificada: la salida PASS es de una corrida real y la
      salida SKIP esta etiquetada como prueba controlada con stub.
- [x] `bash -n`, parseo YAML y `git diff --check` ejecutados.
- [x] Escaneos de contenido ejecutados.
- [x] Sin push ni merge; sin prefijos de herramientas IA ni emojis.
- [x] Ningun archivo de ag-cli ni Cargo modificado (coordinate con split).
