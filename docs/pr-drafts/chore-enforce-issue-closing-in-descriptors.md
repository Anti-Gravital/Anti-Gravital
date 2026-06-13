# Descriptor de PR

## Resumen

Hacer obligatorio el cierre automatico de issues: cada descriptor declara `Closes #NNN` y un job de CI lo verifica

## Fase afectada

Gobernanza / proceso. No toca codigo ni avanza una fase de la Hoja de Ruta.

## Tipo de cambio

- [ ] Correccion de bug
- [x] Documentacion / gobernanza (CLAUDE.md, pr-drafts README, plantilla PR)
- [x] CI (`docs.yml`: nuevo job `descriptor closing-keywords`)
- [ ] Nueva feature
- [ ] Cambio que rompe compatibilidad

## Contexto

Tras fusionar PR #159, los 16 issues resueltos no se auto-cerraron porque los
commits/descriptor usaban `(#NNN)` (referencia) en vez de `Closes #NNN`
(palabra clave). GitHub solo auto-cierra desde el cuerpo de la PR con
`Closes`/`Fixes`/`Resolves`. Esta PR previene que vuelva a pasar.

## Cambios

- `CLAUDE.md`: seccion de descriptor exige `## Cierre de issues` con palabra
  clave de cierre.
- `docs/pr-drafts/README.md` y `.github/PULL_REQUEST_TEMPLATE.md`: misma regla.
- `.github/workflows/docs.yml`: job `descriptor closing-keywords` que rechaza
  cualquier descriptor sin esa seccion.
- Eliminado el descriptor fusionado `issues-repo-audit-resolution.md` (ciclo de
  vida del descriptor).

## Documentos relacionados

- `CLAUDE.md` (Descriptor pre-rellenado y autofill por PR)
- `docs/pr-drafts/README.md`

## Plan de prueba

```sh
# El nuevo job valida que cada descriptor tenga su seccion de cierre.
for f in docs/pr-drafts/*.md; do
  [ "$(basename "$f")" = README.md ] && continue
  grep -qiE "^[[:space:]>*-]*(closes|fixes|resolves)[[:space:]:]+(#[0-9]+|none|ninguno)\b" "$f" \
    || echo "FALTA cierre: $f"
done
```

## Criterios de salida que avanza

- Ningun issue resuelto vuelve a quedar abierto por olvidar la palabra clave.

## Cierre de issues

Closes: none
Refs #142 (mejora del proceso que destapo la reconciliacion de la auditoria).

## Checklist final

- [x] Pertenece a la fase correcta y respeta la documentacion.
- [x] No rompe arquitectura ni anade dependencias.
- [x] CI valido (YAML correcto; el nuevo job pasa con este mismo descriptor).
- [x] Documentacion actualizada junto al cambio.
- [x] Sin evidencia de herramientas IA; atribuido a la persona autora.
