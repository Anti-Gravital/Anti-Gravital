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
- Al abrir el PR en GitHub, el contenido del archivo se copia al
  cuerpo del PR. Los `<!-- comentarios -->` del template no aparecen
  porque ya estan reemplazados por texto concreto.

## Que debe contener cada descriptor

- **Resumen**: una sola linea, maximo 256 caracteres, usable tambien
  como titulo del PR.
- **Fase afectada**: que fase de la Hoja de Ruta tocan estos cambios.
- **Tipo de cambio**: checklist marcado con `[x]` segun corresponda.
- **Documentos relacionados**: RFC, ADR y maestros referenciados.
- **Plan de prueba**: comandos exactos que el revisor puede ejecutar.
- **Criterios de salida que avanza**: casillas concretas de
  `docs/roadmap/STATUS.md` que esta unidad de cambio marca.
- **Checklist**: las cinco casillas siempre, marcadas o explicitas.

## Indice

| Archivo | Rama | Resumen |
| --- | --- | --- |
| `phase-0-foundations-and-governance.md` | `phase-0/foundations-and-governance` | Setup Fase 0 completo (docs maestros, gobernanza, workspace de 15 crates, CI multiplataforma, plantillas) mas implementacion de Fase 1 PRs 1-7 (Shield bootstrap, validation, CORS, CSRF, rate-limit, JWT Ed25519, TLS 1.3). |
