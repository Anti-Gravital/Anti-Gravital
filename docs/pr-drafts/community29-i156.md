## Resumen
Entregar paquete unificado de ejecucion de las puertas #156 y #157: prerrequisitos, comandos, perfiles de host, evidencia, aprobadores y cierre exacto, sin ejecutar ni fingir ninguna puerta.

## Componente afectado
Puertas de release Fase 5 (Fuzz 24h y Benchmarks) y documentacion de auditoria.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation

## Documentos relacionados
- `docs/audits/fase5-gates-execution-package.md` (nuevo, entregable principal).
- `docs/fuzz/24h-gate-runbook.md`, `docs/fuzz/self-hosted-runner.md`, `docs/fuzz/24h-gate-verification.md`.
- `docs/benchmarks/REFERENCE_RUNBOOK.md`, `docs/benchmarks/reference-host.md`, `docs/benchmarks/verification-package.md`.
- `.github/workflows/fuzz-24h.yml`, `scripts/run-benchmarks.sh`, `scripts/run-fuzz-window.sh`.
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (ADR-0015), RFC-0036, ADR-0008.
- `docs/kaizen/journal/2026-11-24-fase5-gates-package-0013.md`.

## Plan de prueba
- `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/fuzz-24h.yml"))'`.
- `bash -n` sobre `scripts/run-benchmarks.sh`, `scripts/run-fuzz-window.sh`, `scripts/ag-check`, `scripts/kaizen`.
- `git diff --check`.
- `bash scripts/ag-check`.
- Escaneos de contenido: atribucion IA, URLs de sesion, emojis, marcas retiradas, prefijos de descriptor.
- Inventario real de evidencia: `find docs/fuzz/results ...`, `find docs/benchmarks/raw ...` (solo smoke 60 s y datos informativos; sin 24h ni reference).

## Impacto en calidad (ratchet)
- Cobertura: igual (sin cambios de codigo Rust).
- Advertencias: igual.
- Deuda (TODO/FIXME): igual.

## Criterios de salida que avanza
- Paquete de decision unico con checklist, comandos, perfil de host, artefactos, aprobador humano y condicion de cierre por puerta.
- Marcado explicito de "preparado, NO ejecutado" para ambas puertas; sin numeros de benchmark ni fuzz de 24 h fingidos.
- Issues #156 y #157 permanecen ABIERTAS con `blocked` y `Refs #156, #157`.
- Escalada needs-human al coordinador con el bloqueo compartido de hardware/ventana.

## Aprendizaje
K-4 nombra `docs/kaizen/pr-drafts/` pero `ag-check` y los workflows usan `docs/pr-drafts/`; se escriben ambos descriptores identicos, como en las ramas community anteriores, para que la verificacion local y CI pasen.

## Cierre de issues
Closes: none
Refs #156, #157

## Checklist
- [x] Compila (sin cambios de codigo Rust)
- [x] Pasa tests (sin cambios de codigo Rust; verificar con ag-check)
- [x] `scripts/kaizen check` en verde (exit 0, deuda 0 <= 0)
- [x] Documentacion actualizada
- [x] Linea base no empeora
