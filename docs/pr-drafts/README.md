# Descriptores de Pull Request

Esta carpeta contiene un archivo Markdown por cada pull request o por
cada rama de trabajo. El archivo es la version pre-rellenada del
descriptor del PR: resumen, fase afectada, tipo de cambio, documentos
relacionados, plan de prueba, criterios de salida y checklist final.

## Por que existe esta carpeta

La plantilla en `.github/PULL_REQUEST_TEMPLATE.md` describe la
estructura esperada del descriptor. En la practica, dejar la plantilla
vacia produce PRs sin contexto y sin trazabilidad. Esta carpeta
resuelve el problema: cada commit o cada rama que va a producir un PR
trae su descriptor ya escrito por la persona o agente que hizo el
cambio, con los detalles concretos en lugar de los placeholders.

Cuando se abre el PR, el contenido del descriptor se copia al cuerpo
del PR en GitHub. Asi el revisor recibe la informacion sin que nadie
tenga que improvisar.

## Convencion de nombre

Un archivo por unidad de PR:

- `<nombre-de-rama>.md` cuando un PR consolida toda la rama.
- `<nombre-de-rama>-<n>.md` cuando una rama produce varios PRs.

Si la rama es larga y aglomera varios cambios en commits separados, el
descriptor refleja el conjunto, no cada commit individual. Si se
desea descomponer en PRs mas pequenos, se divide la rama y cada
subrama tiene su archivo.

## Cuando se escribe

- Antes o durante el commit, no despues.
- Si los detalles cambian en commits subsiguientes, se actualiza el
  archivo en el mismo commit que introduce el cambio.
- Al abrir el PR en GitHub, el workflow
  `.github/workflows/pr-autofill.yml` se dispara automaticamente:
  busca el descriptor por nombre de rama aplanada (las `/` se
  convierten en `-`), y si lo encuentra reemplaza el cuerpo del PR
  con su contenido completo. Si no existe, comenta el PR avisando y
  marca el job como warning.

## Convencion de nombre del archivo (importante para el autofill)

El workflow busca dos rutas, en este orden:

1. `docs/pr-drafts/<rama-aplanada>.md` donde la rama del PR tiene sus
   `/` reemplazadas por `-`. Ejemplo: para la rama
   `phase-1/shield-mvp` el archivo es
   `docs/pr-drafts/phase-1-shield-mvp.md`.
2. `docs/pr-drafts/<rama-literal>.md` como respaldo, conservando las
   `/` (lo que crearia subdirectorios). Se recomienda usar la forma
   aplanada para evitar subdirectorios y simplificar la navegacion.

## Que debe contener cada descriptor

- **Resumen**: una sola linea, maximo 256 caracteres, usable tambien
  como titulo del PR.
- **Fase afectada**: que fase de la Hoja de Ruta tocan estos cambios.
- **Tipo de cambio**: checklist marcado con `[x]` segun corresponda.
- **Documentos relacionados**: RFC, ADR y maestros referenciados.
- **Plan de prueba**: comandos exactos que el revisor puede ejecutar.
- **Criterios de salida que avanza**: casillas concretas de
  `docs/roadmap/STATUS.md` que esta unidad de cambio marca.
- **Cierre de issues** (`## Cierre de issues`): obligatorio. Una linea por
  issue con palabra clave de cierre, porque GitHub solo auto-cierra desde
  el cuerpo de la PR con `Closes`/`Fixes`/`Resolves`:
  - `Closes #NNN` por cada issue que la PR resuelve por completo.
  - `Refs #NNN` por cada issue que solo avanza sin cerrar.
  - `Closes: none` si la PR no resuelve ningun issue.
  Una referencia suelta (`#NNN`, `(#NNN)`) enlaza pero NO cierra. El job
  `descriptor closing-keywords` de CI rechaza descriptores sin esta seccion.
- **Checklist**: las cinco casillas siempre, marcadas o explicitas.

## Ciclo de vida

Un descriptor vive solo mientras su PR esta abierto. Cuando el PR se
fusiona o se cierra, su descriptor se elimina (preferentemente en la
misma PR de merge; si no, en la siguiente barrida de mantenimiento).
Esta carpeta contiene unicamente este `README.md` y los descriptores
de PRs/ramas actualmente abiertos; no se conservan descriptores
historicos. La trazabilidad del trabajo fusionado vive en el historial
de git, el CHANGELOG y los Issues. Regla fijada en CLAUDE.md, seccion
"Descriptor pre-rellenado y autofill por PR".

Ademas, el nombre de un descriptor nunca puede incluir un prefijo de
rama de herramienta IA (`claude-`, `gpt-`, `ai-`, `copilot-`): el job
`prohibited content scan` de CI rechaza esas rutas.

## Indice

No hay PRs abiertos con descriptor en este momento.
