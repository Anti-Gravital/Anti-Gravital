# Documentacion en espanol

El ingles es el idioma canonico del codigo y de la documentacion
tecnica profunda (`ADR-0008`, que supersede el espanol predeterminado
de `ADR-0002`). Esta carpeta funciona como indice rapido en espanol y
como contenedor de traducciones especificas que no encajen en la
estructura general.

## Atajos

- Indice general: `docs/INDEX.md`.
- Maestros (bilingues EN+ES, ingles canonico): `docs/master/`.
- Arquitectura: `docs/architecture/`.
- Hoja de ruta: `docs/roadmap/`.
- Modulos: `docs/modules/`.
- DSL: `docs/dsl/`.
- Seguridad: `docs/security/`.
- Gobernanza: `docs/governance/`.
- Benchmarks: `docs/benchmarks/`.
- RFC: `docs/rfc/`.
- ADR: `docs/adr/`.
- Diagramas: `docs/diagrams/`.

## Politica (ADR-0008)

Los documentos vitrina (README raiz, los tres maestros de
`docs/master/` y los capitulos de `docs/manual/`) son bilingues EN+ES
en el mismo archivo, con la seccion inglesa primero y canonica. La
documentacion tecnica preexistente en espanol se migra a ingles de
forma gradual al tocarse. Si una seccion espanola diverge de la
inglesa, gana la inglesa.
