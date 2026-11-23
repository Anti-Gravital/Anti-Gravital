## Resumen
Catalogo priorizado de 7 oportunidades de arquitectura e ingenieria con evidencia, coste, riesgo, RFC requerida y criterio de exclusion.

## Componente afectado
Documentacion de auditoria (proyecto `oportunidades`, tarea O1-arquitectura). Sin cambios de codigo.

## Tipo de cambio
- [x] documentation

## Documentos relacionados
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (secciones 4, 5.3, 8.8)
- `CLAUDE.md` reglas 5/8/14/15/21/22/28/31
- RFC-0009, RFC-0012, RFC-0020, RFC-0022, RFC-0028, ADR-0009, ADR-0010, ADR-0012, ADR-0019
- `docs/audits/pre-fase5-architecture.md`, `tools/file-size-allowlist.txt`

## Plan de prueba
- `git diff --check` (sin errores de whitespace)
- `bash scripts/kaizen check --only descriptors policy` (descriptor y politicas de contenido)
- `grep -rnE 'set_var|remove_var' crates/ --include='*.rs'` (evidencia de A1)
- `grep -rnE '^ag-[a-z-]+ *=' crates/*/Cargo.toml` (evidencia de A5, 19 crates)
- `grep -rn 'producer::emit\\|DeferredJobProducer' crates/ --include='*.rs'` (evidencia de A3a)
- `find docs/audits -iname 'estado-honesto*'` (sin entregables de pares al corte)

## Impacto en calidad (ratchet)
- Cobertura: igual (solo documentacion; no se tocan `src/`/`crates/`)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- Catalogo de oportunidades priorizadas, cada una con valor, coste, riesgo, RFC requerido y criterio de exclusion (KPI del charter de `oportunidades`).
- Cada oportunidad marcada como `implementable now`, `needs RFC`, `needs human (protected path)` o `defer to Phase 5+`.
- Ninguna propuesta anade crates, dependencias nuevas ni dependencias inversas (reglas 14/15/21, S-9).

## Aprendizaje (opcional)
La verificacion de direccion de dependencias del workspace sigue siendo manual y data de la epoca de 17 crates; la nueva arista `ag-edge -> ag-domains` y el puerto `producer` sin consumidor son ejemplos concretos de por que conviene una guarda automatica (A5).

## Cierre de issues
Closes: none

## Checklist
- [x] Compila (no aplica: cambio solo documental)
- [x] Pasa tests (no aplica: cambio solo documental)
- [x] `scripts/kaizen check --only descriptors policy` en verde
- [x] Documentacion actualizada
- [x] Linea base no empeora
