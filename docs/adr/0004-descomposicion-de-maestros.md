# ADR-0004 - Descomposicion verbatim de los documentos maestros

- Estado: aceptado
- Fecha: 2026-05-19
- RFC origen: ninguna; decision fundacional alineada con CLAUDE.md
  reglas 2, 3, 4, 33 y 34.

## Contexto

Los documentos maestros del proyecto son extensos y unicos. La
arquitectura tecnica tiene veinte capitulos y la hoja de ruta once
fases mas extras. Un solo archivo monolitico dificulta la navegacion,
la deteccion automatica de progreso y la integracion con tooling. Al
mismo tiempo, fragmentar los maestros conlleva riesgo de
inconsistencia.

## Decision

Los maestros permanecen intactos en `docs/master/` como fuente de
verdad. Se publica una descomposicion verbatim en archivos
navegables bajo:

- `docs/architecture/` - un archivo por capitulo del maestro de
  arquitectura.
- `docs/roadmap/` - un archivo por fase del maestro de la hoja de
  ruta.
- `docs/modules/<crate>/` - un README por crate con punteros al
  capitulo de arquitectura correspondiente.
- `docs/dsl/`, `docs/security/`, `docs/governance/`,
  `docs/benchmarks/` - subsecciones derivadas con su capitulo de
  origen.

Reglas duras de la descomposicion:

1. El contenido se copia palabra por palabra desde el maestro al
   derivado.
2. No se anade informacion. Si surge necesidad de anadir, se hace en
   el maestro via RFC.
3. Cada derivado abre con una linea de procedencia que apunta al
   maestro y a la seccion exacta.
4. Si un derivado contradice un maestro, gana el maestro. La
   regeneracion se hace ejecutando `bash tools/split-masters.sh`.

## Consecuencias

Positivas:

- Navegacion granular sin perder el contrato monolitico del maestro.
- Tooling y agentes pueden enlazar a archivos por capitulo o fase.
- Integridad documental verificable: hashes SHA-256 en `VERSION.md` y
  workflow `docs.yml`.

Negativas:

- Riesgo de desincronizacion si alguien edita un derivado a mano.
  Mitigacion: regla de regeneracion, revision en pull request,
  workflow que valida la integridad de los maestros.
- Doble peso del repositorio para texto similar. Aceptado por
  ergonomia.

## Alternativas consideradas

- Tooling de extraccion bajo demanda (sin commitear derivados):
  rechazado porque rompe la navegabilidad en GitHub y la lectura
  offline.
- Reemplazar los maestros por la descomposicion: rechazado porque
  CLAUDE.md y la arquitectura exigen mantener los maestros con sus
  nombres exactos.

## Notas

El script de regeneracion es `tools/split-masters.sh`. El generador
de READMEs de modulos y de subsecciones es `tools/scaffold-docs.sh`.
