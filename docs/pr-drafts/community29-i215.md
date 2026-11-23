## Resumen
Verificar issue #215: el fallo de security-audit es un error CLI determinista, no transitorio; escalar el fix de .github a humano.

## Componente afectado
CI semanal `security-audit` y herramientas de auditoria de dependencias (cargo-audit, cargo-deny).

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation

## Documentos relacionados
- RFC-0022 seccion 4.2 (guardarrales de mantenibilidad).
- `docs/audits/issue-215-security-audit-verification.md`.
- Outbox S-11: `.kaizen/loop/outbox/issue-215-cargo-deny-invalid-flag.md`.

## Plan de prueba
- `cargo deny check --all-features`  (reproduce el fallo: exit 2).
- `cargo deny --offline check`  (invocacion correcta: exit 0).
- `cargo audit --no-fetch`  (exit 0).
- `git diff --check`.
- `bash scripts/ag-check`.

## Impacto en calidad (ratchet)
- Cobertura: igual (sin cambios de codigo Rust).
- Advertencias: igual.
- Deuda (TODO/FIXME): igual.

## Criterios de salida que avanza
- Causa raiz identificada con salida real de comandos reproducibles.
- Escalada needs-human documentada por S-11 (`.github/` es ruta protegida L-4).
- Condicion exacta de cierre del issue #215 entregada al coordinador.

## Aprendizaje
Un flag de CLI invalido puede disfrazarse de fallo de seguridad recurrente y mantener abierto un issue para siempre; hay que separar "herramienta mal invocada" de "advisory encontrado".

## Cierre de issues
Closes: none
Refs #215

## Checklist
- [x] Compila (sin cambios de codigo Rust)
- [x] Pasa tests (sin cambios de codigo Rust; verificar con ag-check)
- [ ] `scripts/kaizen check` (se reporta el resultado real de la ejecucion)
- [x] Documentacion actualizada
- [x] Linea base no empeora
