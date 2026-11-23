kind: failure
title: F3 - catalogo desincronizado: regeneracion en ruta protegida requiere humano (needs-human)
fingerprint: kaizen-f3-catalog-desync-needs-human
---
## Que paso
El tablero `kaizen-gates-green` tiene la tarea F3 (`F3-catalogo-needs-human`) por el desajuste del catalogo de agentes. El arreglo mecanico es `scripts/kaizen-org catalog`, pero ese comando ESCRIBE `.kaizen/org/catalog.json`, `.kaizen/org/registry.json` y `docs/kaizen/org/CATALOGO-AGENTES.md`. `.kaizen/` es ruta protegida (L-4), por lo que un agente NO esta autorizado a ejecutarlo: se ESCALA a humano y NO se ejecuta la regeneracion. Ademas, generar desde las definiciones locales actuales puede degradar la matriz de gobernanza heredada (43 entradas) a 25, asi que hace falta decision humana previa sobre la fuente de verdad.

## Donde (archivo:linea, comando)
- `scripts/kaizen-org:320-349` `cmd_check`: compara `jq -r '.count' .kaizen/org/catalog.json` contra `jq 'length' "$REG"`; línea 348 emite `catálogo desc.sincronizado ($cn vs $rn)`.
- `scripts/kaizen-org:207-254` `cmd_catalog`: `rebuild_registry`, escribe `docs/kaizen/org/CATALOGO-AGENTES.md` y `.kaizen/org/catalog.json` (con `count = length`).
- `scripts/kaizen-org:21-33` `rebuild_registry`: escribe `.kaizen/org/registry.json`; `cmd_check:341-343` tambien lo reescribe (idempotente, sin diff).
- Rutas protegidas: `.kaizen/org/catalog.json`, `.kaizen/org/registry.json`, `.kaizen/org/prompts/*.md` (L-4: `.kaizen/`).
- `scripts/kaizen:86,106` `check_org` -> la puerta `organizacion` de `kaizen check` (y `kaizen-swarm:254` la arrastra).

## Impacto
Sin la decision/ejecucion humana, las puertas `organizacion` y (por dependencia) `swarm` de `kaizen check` pueden quedar en rojo y el KPI del charter (`kaizen check FAIL=0`, `kaizen-gates framework+ratchet success en main`) no se puede declarar verde de forma honesta. Si se regenera sin coordinar con gobernanza, se pierde cobertura de roles (43 -> 25) y aparecen errores "rol desconocido" (ver `kaizen-gov-especialistas-y-regresion.md`). El agente no puede forzar el pase (K-10/L-10).

## Evidencia (salida real, SIN secretos ni datos personales)
Reportado previamente (`.kaizen/loop/outbox/kaizen-org-catalogo-desincronizado.md`):
```
$ scripts/kaizen-org check
[kaizen-org] FALLA: catálogo desc.sincronizado (43 vs 21)
[kaizen-org] FALLA: kaizen-org check: FALLO
```
Verificacion actual del auditor (2026-11-22): en `main@b44c9b0` y en la rama `community14/gate-audit@c1b918d` el desajuste ya NO se reproduce; el check esta en verde y `git status --short` queda limpio tras ejecutarlo (no ensucio archivos versionados):
```
$ scripts/kaizen-org check
[kaizen-org] kaizen-org check: todo en verde
$ jq -r '.count' .kaizen/org/catalog.json ; jq 'length' .kaizen/org/registry.json ; ls .kaizen/org/prompts/*.md | wc -l
25
25
26
```
(26 ficheros = 25 definiciones con `id` + `_template.md`, excluido por `files()` en `scripts/kaizen-org:16`.)
Nota de honestidad (K-10): los numeros 43 vs 21 son los del reporte original; la verificacion actual es 25 vs 25 en verde. No se fabrica evidencia: se reportan ambos estados.

## Reproduccion
1. `cd <repo> && scripts/kaizen-org check`  (solo lectura/validacion)
2. Regeneracion (SOLO HUMANO): `cd <repo> && scripts/kaizen-org catalog`

## Propuesta
Comando exacto que debe ejecutar UN HUMANO: `scripts/kaizen-org catalog` (desde la raiz del repo).
Efecto esperado: `rebuild_registry` reescribe `.kaizen/org/registry.json` desde `.kaizen/org/prompts/*.md`; regenera `docs/kaizen/org/CATALOGO-AGENTES.md`; reescribe `.kaizen/org/catalog.json` con `count` igual al numero de definiciones. Con ello `kaizen-org check` queda en verde y se elimina la dependencia que arrastra a `kaizen-swarm check`.
Autoridad: agente NO autorizado (L-4 protege `.kaizen/`; K-10/L-10 prohiben forzar un pase). NO se ejecuto la regeneracion; solo se corrio `check` (lectura) y se verifico que no dejo diffs.
Precondicion humana (importante): decidir la fuente de verdad ANTES de regenerar. La matriz `.kaizen/gov/governance.json` tiene 43 entradas frente a 25 definiciones locales; regenerar todas las derivadas desde las 25 degradaria la cobertura y generaria errores "rol desconocido". Coordinar con el finding de gobernanza. Opciones: (a) declarar el catalogo local como fuente y ampliar las definiciones/ gobernanza a 43, o (b) declarar la instalacion global como fuente y actualizar el framework. Requiere ADR/decision humana, no edicion directa.
