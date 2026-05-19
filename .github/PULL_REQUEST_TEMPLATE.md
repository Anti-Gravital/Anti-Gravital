<!--
  Esta plantilla es solo un fallback. Cada pull request del proyecto
  trae su descriptor pre-rellenado en `docs/pr-drafts/<rama>.md`.
  Antes de abrir el PR:
    1. Localice `docs/pr-drafts/<nombre-de-rama>.md` en la rama.
    2. Copie su contenido completo y peguelo aqui, reemplazando este
       comentario y todas las secciones que siguen.
    3. Verifique que el titulo del PR (campo de arriba en GitHub) no
       excede 256 caracteres y coincide con el "Resumen" del
       descriptor.
  Si el descriptor no existe, hay que crearlo en el mismo commit que
  abre la rama. Vease `docs/pr-drafts/README.md` para la convencion.
-->

## Resumen

Una linea, maximo 256 caracteres. Misma frase usable como titulo del
PR. Sin emojis ni atribuciones a herramientas IA.

## Fase afectada

Fase N. Vease `docs/roadmap/`.

## Tipo de cambio

- [ ] Documentacion
- [ ] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- RFC: `docs/rfc/RFC-XXXX-...md` o N/A.
- ADR: `docs/adr/XXXX-...md` o N/A.
- Maestro afectado: `docs/master/...` o N/A.

## Plan de prueba

Comandos exactos que el revisor puede ejecutar para validar el cambio
(por ejemplo `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo deny check`).

## Criterios de salida que avanza

Casillas concretas de `docs/roadmap/STATUS.md` que esta PR marca,
copiadas con su estado actualizado (`[x]` para las completas,
`[/]` para las parciales con explicacion).

## Checklist

- [ ] Titulo de PR de 256 caracteres o menos.
- [ ] Sin emojis en ningun archivo modificado.
- [ ] Sin atribuciones de herramientas IA.
- [ ] Documentacion actualizada en el mismo PR.
- [ ] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [ ] CLAUDE.md respetado (alcance, fase, dependencias, seguridad).
- [ ] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
