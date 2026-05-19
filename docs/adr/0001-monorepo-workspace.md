# ADR-0001 - Monorepo Cargo workspace

- Estado: aceptado
- Fecha: 2026-05-19
- RFC origen: ninguna; decision fundacional alineada con la seccion 5
  del maestro de arquitectura.

## Contexto

El maestro `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` describe
el proyecto como un ecosistema de 15 crates con versionado semantico
independiente. La decision arquitectonica clave del v4.0 fue separar
el nucleo de los ecosistemas. Falta decidir si esos 15 crates viven en
15 repositorios separados, en un monorepo Cargo workspace, o en una
combinacion.

## Decision

Anti-Gravital adopta un monorepo Cargo workspace con 15 miembros bajo
`crates/`. El versionado semantico de cada crate sigue siendo
independiente, pero todos viven en el mismo repositorio git.

## Consecuencias

Positivas:

- Un solo CI cubre todo el ecosistema.
- Cambios atomicos que cruzan varios crates son posibles en una sola
  pull request.
- Tooling de calidad (fmt, clippy, audit, deny) se ejecuta una vez por
  workspace.
- Documentacion centralizada en `docs/`.
- Releases independientes via cargo publish por crate.

Negativas:

- El historial git contiene cambios de todos los crates mezclados.
  Mitigacion: convencion `tipo(scope):` en commits.
- Cualquier pull request grande arriesga tocar varios crates a la vez.
  Mitigacion: regla de una unidad logica por PR.
- Los issues necesitan etiquetas por crate para no convertirse en cola
  comun. Mitigacion: labels obligatorias.

## Alternativas consideradas

- 15 repositorios separados: rechazado por friccion de coordinacion y
  por la necesidad de releases atomicos durante Fase 0 a 4.
- Mixta (algunos crates en el monorepo, otros aparte): rechazado por
  complejidad innecesaria en una primera version.

## Notas

- Workspace definido en `Cargo.toml` raiz, con `resolver = "2"` y
  miembros listados explicitamente.
- Politica de versionado independiente registrada en la seccion 5 del
  maestro de arquitectura.
