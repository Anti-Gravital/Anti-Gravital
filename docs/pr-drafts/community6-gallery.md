# docs: add the wave-6 template catalog gallery and bilingual discovery links

**Resumen:** Add a 33-template decision table and bilingual links so the
catalog is discoverable and chooseable.

La tabla cubre los 33 nombres del catalogo de la ola 6 y senala con `confirm`
los requisitos externos inferidos. Ningun maestro ni documento de estado alpha
se modifica.

## Fase afectada

Ninguna fase bloqueante. Documentacion de apoyo al catalogo de plantillas de la
ola 6, sobre el alpha `v0.1.0-alpha.1`. No toca `docs/master/`.

## Tipo de cambio

- [ ] Codigo
- [x] Documentacion
- [ ] CI / tooling
- [ ] RFC / ADR

## Documentos relacionados

- `docs/templates/GALLERY.md` (nuevo)
- `docs/templates/README.md` (nuevo)
- `docs/INDEX.md`
- `README.md` (ingles y espanol)
- `docs/manual/02-primera-api.md` (ingles y espanol)
- `CHANGELOG.md`
- `templates/README.md`
- `docs/examples/alpha-prototype.md` (matriz de infraestructura verificada)

## Plan de prueba

- `git diff --check`
- Verificar que todos los enlaces relativos nuevos resuelven a archivos
  existentes (galeria, indice, INDEX raiz de docs, README en ambos idiomas,
  capitulo del manual).
- Verificar que los 33 nombres del tablero de la ola 6 aparecen en la galeria
  y que cada fila tiene caso de uso, requisitos externos y comando.
- Verificar consistencia bilingue: seccion inglesa canonica primero y misma
  informacion en la seccion espanola.
- Verificar ausencia de emojis y de marcas de herramientas externas en los
  archivos tocados.

## Criterios de salida que avanza

Ninguna casilla de `docs/roadmap/STATUS.md` se cierra con este cambio. Es una
entrega documental de apoyo al catalogo de plantillas.

## Cierre de issues

Closes: none

## Checklist

- [x] El cambio pertenece a la ola 6 y no adelanta fases.
- [x] No modifica maestros ni el texto de estado alpha del README.
- [x] No anade dependencias ni complejidad.
- [x] Documentacion bilingue con ingles canonico primero.
- [x] Enlaces relativos verificados.
- [x] Sin emojis ni evidencia de herramientas externas.
- [x] `git diff --check` limpio.
