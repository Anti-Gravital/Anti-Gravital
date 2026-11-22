# Resumen

Add an opt-in TEST-KEY-only release-signature verifier with real evidence; #141 stays open.

- Fase afectada: cambio de herramientas de release y documentacion de seguridad (ola 12); no altera fases del roadmap.
- Tipo de cambio: [x] documentacion y gobernanza  [ ] CI/workflow  [x] herramienta/script local  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: RFC-0030 (seccion 11), `docs/security/RELEASE_KEY_CEREMONY.md`,
  `docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md`, `docs/security/INSTALLATION_INTEGRITY.md`,
  `tools/release-signing/README.md`, `CHANGELOG.md`, CLAUDE.md reglas 16, 19, 22, 26 y 28.

## Cambios

- `scripts/check-release-signature.sh`: verificador opt-in de una firma Ed25519 de
  prueba producida por `tools/release-signing/release-signing.sh`. Acepta clave
  publica, manifiesto y firma por entorno o argumentos posicionales. Imprime
  `TEST KEY ONLY - NOT A PRODUCTION SIGNATURE` en cada ejecucion. Resultados
  PASS (firma valida, etiquetada como ensayo), SKIP (sin claves o manifiesto;
  la ausencia nunca es PASS) y FAIL (archivo ausente, openssl no disponible o
  firma no valida, incluido el caso de manipulacion). No se llama desde CI ni
  desde los instaladores.
- `tools/release-signing/README.md`: documenta el verificador, sus entradas,
  su disciplina de resultados y el comando de ejemplo.
- `docs/security/RELEASE_SIGNING_TEST_EVIDENCE.md`: nueva seccion Check 5 con una
  ejecucion real genkey -> sign -> verify (PASS) -> tamper -> verify (FAIL),
  metadatos de host, Codigos de salida y la comprobacion SKIP por ausencia.
- `docs/security/INSTALLATION_INTEGRITY.md` y `CHANGELOG.md`: navegacion y registro.

## Plan de prueba

- `bash -n scripts/check-release-signature.sh`
- Sin entradas: `env -u AG_TEST_RELEASE_PUBKEY -u AG_SIGNED_MANIFEST -u AG_MANIFEST_SIGNATURE bash scripts/check-release-signature.sh`
  imprime `RESULT: SKIP` y sale 0 (nunca PASS).
- Cadena real: `AG_SIGN_STATE_DIR=/tmp/opencode/ag-verify-i141-state bash tools/release-signing/release-signing.sh genkey`,
  `sign checksums/installers.sha256` (copia), `bash scripts/check-release-signature.sh` (PASS),
  modificar un byte del manifiesto y repetir (FAIL, salida 1). Verbatim en el documento de evidencia.
- `git diff --check`
- Escaneos de contenido prohibido: emojis, evidencia de herramientas IA y marcas
  comerciales retiradas (ADR-0011), segun `.github/workflows/docs.yml`.
- No se modifico `.github/workflows/release.yml` y no se genera clave de produccion.

## Criterios de salida que avanza

- Hace ejecutable y auditable la verificacion local opt-in de la firma de ensayo
  sin afirmar que existe un release firmado.
- Las seis condiciones de cierre de RFC-0030 seccion 11 siguen sin cumplirse; la
  issue #141 permanece abierta con `blocked`.

## Cierre de issues

Refs #141
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (ola 12, trabajo aditivo en herramientas de release).
- [x] Respeta la documentacion (RFC-0030 y ceremony sin cambios de estado).
- [x] No rompe arquitectura (script independiente, sin crates ni CI).
- [x] No anade complejidad innecesaria (un verificador, una via de entrada).
- [x] Compila / `bash -n` en verde.
- [x] Ejecucion real con evidencia verbatim.
