# ADR-0001: autonomía nivel 3 (registro de decisión ya vigente)

Estado: aprobado (registro de decision ya vigente; no introduce cambios de autonomia)
Fecha: 2026-11-22
Aprobado por: Angel Nereira (responsable del repositorio), decisión humana de la sesión de instalación de Kaizen

## Fuentes (source_ids)

- `.kaizen/loop/LOOP.md:9-10` — L-3 "Niveles de autonomía": fuente canónica; declara que el nivel 3 ya está aprobado por ADR-0001 y fija su alcance y límites.
- `.kaizen/loop/LOOP.md:12-13` — L-4 "Rutas protegidas": lista canónica de rutas vedadas.
- `.kaizen/swarm/PROTOCOL.md` sección 7 — el push y la apertura de PR corresponden al orquestador, no al agente.
- `docs/kaizen/adr/0002-veto-de-cumplimiento-e-investigacion-normativa.md` y `docs/kaizen/adr/0003-modelo-de-equipo-proactivo.md` — vetos y límites que el nivel 3 no elimina.
- `.kaizen/team/OWNER.md:10-13` y `.kaizen/team/FLUJOS.md:34` — tabla de autoridad y regla "Autonomía vs. humano".
- `.kaizen/org/CONSENSUS.md:18` — las decisiones sobre rutas protegidas, pagos, seguridad o el propio loop nunca se toman sin humano, incluso en autonomía 3.
- `.kaizen/loop/outbox/kaizen-swarm-adr0001-faltante.md` — hallazgo S-11 que motivó este registro (el archivo no existía en el bootstrap).
- `scripts/kaizen-swarm:253` — comprobación que fallaba por la ausencia del archivo.

## Contexto

El loop de Kaizen gobierna el nivel de autonomía mediante `KL_AUTONOMY`. El texto canónico de `.kaizen/loop/LOOP.md` L-3 afirma explícitamente que **el nivel 3 ya está aprobado por ADR-0001** y describe su alcance; L-4 enumera las rutas protegidas. La sección 7 de `.kaizen/swarm/PROTOCOL.md` y los ADR-0002/0003 refuerzan que esa autonomía no suprime al humano en seguridad, cumplimiento, dinero, datos personales ni rutas protegidas.

Sin embargo, el archivo `docs/kaizen/adr/0001-autonomia-nivel-3.md` no existía, y `scripts/kaizen-swarm check` fallaba con `falta ADR-0001 (autonomia nivel 3)`. El hallazgo ya estaba documentado en el outbox S-11 indicado en las fuentes. La decisión estaba tomada; faltaba únicamente su registro documental en la ruta que la puerta verifica (K-1/K-7/S-11).

Este ADR documenta esa decisión ya vigente. **No crea, amplía ni reduce autonomía, no añade derechos de agente ni exime a ningún humano de las puertas existentes.** No es un ADR nuevo de cambio de autonomía: esa clase de cambio requiere decisión humana (L-4/K-13 y `AGENT_SCHEMA.md:41`).

## Decisión

1. Registrar como vigente el nivel 3 de autonomía con el alcance exacto ya aprobado, sin modificarlo.
2. Dejar constancia literal de los límites de ejecución y de las exclusiones que el nivel 3 **no** puede franquear, según L-3 (`.kaizen/loop/LOOP.md:10`):

   > Respeta KL_AUTONOMY. El nivel 3 está aprobado por ADR-0001: auto-merge de bajo riesgo y bugfix con prueba de regresión. Nunca fusiones, publiques ni despliegues tú: el orquestador abre el PR y la protección de rama decide. Nunca, en ningún nivel, sin humano: rutas protegidas (L-4), seguridad y autenticación, pagos, migraciones destructivas, borrado de datos, publicar versiones a producción sin pipeline con gate humano, y modificar el propio loop o sus reglas.

3. Dejar constancia literal de la lista de rutas protegidas según L-4 (`.kaizen/loop/LOOP.md:13`), que este ADR no modifica:

   > No modifiques: .github/, .githooks/, .kaizen/, scripts/kaizen*, docs/kaizen/loop/, .kaizen/swarm/, scripts/kaizen-swarm, scripts/lib/, CODEOWNERS, archivos de instrucciones de agentes (AGENTS.md, CLAUDE.md, GEMINI.md, opencode.json, .claude/, .agents/, .opencode/), infraestructura, migraciones, secretos, autenticación ni pagos. Si una mejora requiere tocar esas rutas, no la hagas: crea un issue con la etiqueta needs-human y la propuesta.

4. Confirmar que este documento no altera ADR-0002, ADR-0003 ni la sección 7 de `PROTOCOL.md`; en particular, el agente sigue sin fusionar, publicar ni desplegar por sí mismo, y el orquestador sigue siendo el único que hace push y abre PR.

## Alternativas consideradas

- No publicar el ADR y mantener la puerta en rojo: descartado; la decisión ya está vigente y la ausencia del archivo es un defecto de documentación verificable (K-1/K-7), no una decisión nueva.
- Redactar un ADR que cambie o amplíe la autonomía: descartado; L-4/K-13 exigen decisión humana y este registro no tiene esa autoridad.
- Reutilizar `docs/adr/0001-monorepo-workspace.md`: descartado; pertenece a otro namespace (`docs/adr/`) y documenta el workspace Cargo, no la autonomía del loop.

## Consecuencias

- Positivas: cierra el hallazgo S-11 sin tocar rutas protegidas; `scripts/kaizen-swarm check` deja de reportar el fallo ADR-0001; queda trazabilidad documental de la decisión ya aprobada.
- Negativas: el documento es declarativo y no añade funcionalidad; existe un ADR-0001 ajeno en `docs/adr/`, por lo que se mitiga la confusión con el título explícito y las fuentes citadas.

## Pendiente de decisión humana

- La definición cuantitativa u operativa de "bajo riesgo" (umbral, tamaño de diff, criterios de clasificación) no está especificada en las fuentes canónicas citadas. Este ADR **no la define** y no afirma ningún umbral. Cualquier definición o cambio requiere decisión humana mediante ADR.
- El detalle de implementación de cómo el orquestador y la protección de rama ejecutan el auto-merge queda fuera de este registro; L-3 ya determina **quién** lo hace, sin especificar el mecanismo, y este documento no lo inventa.

## Cómo se verificará

- `scripts/kaizen-swarm check` ya no reporta `falta ADR-0001 (autonomia nivel 3)`. Otros fallos de la puerta, si los hay, se registran por separado en el descriptor.
- `git diff --check` sin salida.
- `bash scripts/ag-check`: resultado real registrado en `docs/kaizen/pr-drafts/community14-f2-adr.md`.

## Estado

aprobado (registro de decision ya vigente; no introduce cambios de autonomia)
