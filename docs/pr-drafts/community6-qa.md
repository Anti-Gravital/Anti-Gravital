# Descriptor de PR: community6/qa

## Resumen

Add testing-suite template and offline validation harness for the wave 6 template catalog

## Fase afectada

Ola 6, catálogo de plantillas. No cambia el estado de ninguna fase ni ninguna
casilla de `docs/roadmap/STATUS.md`. No toca los maestros de `docs/master/`.

## Tipo de cambio

- [x] Plantilla nueva de catálogo (`templates/testing-suite/`)
- [x] Script de validación (`scripts/validate-templates.sh`)
- [x] Evidencia de auditoría (`docs/audits/template-validation.md`)
- [x] CHANGELOG
- [ ] Código de producción de crates
- [ ] Cambio de arquitectura, DSL, seguridad, stack o CI
- [ ] Dependencias de Cargo del workspace

## Documentos relacionados

- `templates/testing-suite/template.toml` y `templates/testing-suite/README.md`
- `scripts/validate-templates.sh`
- `docs/audits/template-validation.md` (evidencia real de la ejecución)
- `CHANGELOG.md` (sección `[Unreleased]`)
- `/tmp/opencode/agents/PROTOCOL.md` (contrato `template.toml` de la ola 6)

## Plan de prueba

Ejecutados en este worktree con el target compartido y `CARGO_INCREMENTAL=0`:

1. `bash -n scripts/validate-templates.sh`: exit 0.
2. `./scripts/validate-templates.sh`: exit 0, un template descubierto,
   `[PASS] testing-suite`, resumen `1 passed, 0 failed`.
3. `DEEP_CHECK=1 ./scripts/validate-templates.sh`: exit 0. Añade
   `cargo check --offline --locked --all-targets` y compila también los
   targets de test y benchmark.
4. Scaffold generado por el arnés: `cargo test --offline --locked` con 9 tests
   en verde (4 unitarios, 3 de integración, 2 de propiedad) y
   `cargo run --offline --locked` con salida observable.
5. `git diff --check`: exit 0.
6. Escaneos de contenido: sin emojis, sin atribución de herramientas IA, sin
   nombres de marcas retiradas, descriptor con palabra clave de cierre.

## Criterios de salida que avanza

- Ninguna casilla de `STATUS.md`. Aporta la comprobación automática de que
  cada plantilla del catálogo genera un proyecto y compila sin red; el
  coordinador puede rerunear el arnés a medida que se integran las demás ramas.

## Cierre de issues

Closes: none

## Notas para el coordinador

- El binario `ag` disponible durante esta rama aún no tiene el registro de la
  ola 6, por lo que el arnés usó su fallback local (mismas reglas que
  `ag new`: quitar `.tmpl` y sustituir `{{name}}`) y lo informó explícitamente.
  Tras integrar `community6/catalog`, reconstruir `ag` y ejecutar
  `STRICT_CLI=1 ./scripts/validate-templates.sh`: cualquier plantilla que aún
  necesite el fallback falla, lo que prueba la conexión del registro.
- No se añade job de CI: el arnés depende aún de un binario preconstruido, una
  caché offline caliente y ramas en integración. Un job obligatorio sería
  frágil. La decisión y el comando manual quedan registrados en la auditoría.

## Checklist

- [x] Compila y pasan los tests del scaffold generado
- [x] `git diff --check` limpio
- [x] Sin emojis ni evidencia de herramientas IA
- [x] Descriptor con sección de cierre de issues
- [x] Documentación y CHANGELOG actualizados en la misma rama
