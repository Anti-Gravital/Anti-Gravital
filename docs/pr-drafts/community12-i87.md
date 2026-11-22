# Resumen

Deliver a complete ACME staging evidence handoff kit for #87 without claiming external CA evidence.

- Fase afectada: soporte de QA para RFC-0011 fases E-F; no altera fases del roadmap ni contratos de crate.
- Tipo de cambio: [x] documentacion y auditoria  [ ] CI/workflow  [ ] codigo de crate  [ ] DSL  [ ] API publica.
- Documentos relacionados: RFC-0011, `docs/ag-domains/how-to/acme-staging-test.md`,
  `docs/ag-domains/acme-mock-evidence.md`,
  `docs/ag-domains/reference/instant-acme-0.8-migration.md`,
  `docs/INDEX.md`, `docs/ag-domains/README.md`, `CHANGELOG.md`,
  CLAUDE.md reglas 17, 18, 36 y 37.

## Cambios

- `docs/audits/acme-staging-handoff.md` (nuevo): definicion de la puerta,
  prerrequisitos exactos (dominio desechable con zone write + DNS externo y
  egress HTTPS al directorio staging; alternativa de CA local solo para
  ensayo, no cierra #87), variables de entorno, comandos de preflight,
  emision DNS-01 y renovacion con reuso de credenciales, verificacion de chain
  y SNI, campos de evidencia (URL de orden, estados del challenge, huella
  SHA-256 del chain, timestamps, ARI, teardown) y plantilla vacia de
  resultados sin valores inventados.
- Seccion separada que declara que la evidencia mock actual (subject=issuer,
  autofirmada en local) NO satisface la puerta de staging de #87/#179.
- Seccion de harness local reutilizado: rutas, verificacion real (`bash -n`,
  `py_compile`, ejecucion con salida 0 y `PASS`), lo que demuestra y lo que no.
- `CHANGELOG.md`, `docs/INDEX.md` y `docs/ag-domains/README.md`: navegacion y
  registro del cambio, sin afirmar que la puerta paso.

## Plan de prueba

- `git diff --check`
- Verificacion real del harness: `bash -n scripts/acme-mock-harness.sh`;
  `python3 -m py_compile scripts/acme-mock-harness.py`;
  `scripts/acme-mock-harness.sh /tmp/w12-i87-mock.log` (salida 0, `PASS`;
  transcribido en la auditoria como evidencia de herramienta, no de staging).
- Enlaces: `acme-staging-test.md`, `acme-mock-evidence.md`,
  `instant-acme-0.8-migration.md`, RFC-0011, modulos `acme/**` y scripts
  existen en el arbol.
- Escaneos de contenido: sin emojis, sin evidencia de herramientas IA, sin
  marcas prohibidas como componentes propios (ADR-0011), segun
  `.github/workflows/docs.yml`.
- No se ejecuto ningun cargo test contra staging: no hay dominio, DNS ni red
  credentialed. No se afirma aprobacion de la puerta.

## Criterios de salida que avanza

- Deja documentado y ejecutable el procedimiento de #87 sin simular evidencia
  externa; la issue permanece abierta con `blocked` y #179 sigue bloqueada.
- Cumple las reglas 17 (sin metricas inventadas), 18/36 (verificacion
  reproducible) y 26 (codigo y documentacion sincronizados).

## Cierre de issues

Refs #87
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (soporte QA de RFC-0011 E-F, ola 12).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: reutiliza el harness existente.
- [x] Compila / no aplica (solo documentacion): comandos y enlaces validados.
- [x] Pasa `git diff --check` y los escaneos de contenido.
- [x] Tiene documentacion (auditoria, indices y changelog).
- [x] No inventa resultados de ejecucion ni cierra la puerta.
