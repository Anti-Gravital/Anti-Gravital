# ADR-0002 - Documentacion bilingue espanol e ingles

- Estado: superseded por ADR-0008 (2026-05-24)
- Fecha: 2026-05-19
- RFC origen: ninguna; decision fundacional alineada con la seccion 1
  del maestro de arquitectura ("documentacion bilingue desde el dia
  cero").

> **Nota de supersesion (2026-05-24).** El aspecto de "espanol como
> predeterminado" de esta ADR fue revisado al cierre de la Fase 4.5,
> en el punto de revision que esta misma ADR dejo anticipado. `ADR-0008`
> establece el ingles como idioma canonico del codigo y de la documentacion
> tecnica profunda, manteniendo bilingue (EN+ES, mismo archivo) solo la
> vitrina: README raiz, maestros y manual. La estructura de carpetas
> `docs/es/` y `docs/en/` se conserva. Lo que sigue es el texto original.

## Contexto

El proyecto nace en Panama y se posiciona explicitamente para
adopcion en Latinoamerica como primer foco. El maestro de arquitectura
declara documentacion bilingue espanol e ingles desde el dia cero. La
decision aqui es como estructurar esa bilinguidad en el repositorio.

## Decision

- Los documentos maestros bajo `docs/master/` se publican en su idioma
  original (espanol).
- El `README.md` del repositorio es bilingue, con bloques separados
  para espanol y para ingles.
- Las carpetas de paquetes (`docs/es/`, `docs/en/`) actuan como
  indices por idioma. Las traducciones de capitulos individuales viven
  ahi conforme se producen.
- Los archivos de gobernanza (`CONTRIBUTING.md`, `GOVERNANCE.md`,
  `SECURITY.md`, `CODE_OF_CONDUCT.md`) incluyen frases clave en ambos
  idiomas pero no se duplican enteros; cada uno usa el idioma que
  resulta mas claro para su audiencia, con espanol como predeterminado.
- Las plantillas de issue, pull request, RFC y ADR aceptan contenido
  en cualquiera de los dos idiomas.

## Consecuencias

Positivas:

- Adopcion inicial natural en Latinoamerica sin barrera de idioma.
- Mensaje claro al ecosistema de que el proyecto no esta atado a un
  unico idioma o cultura.
- Posibilidad de traducir incrementalmente.

Negativas:

- Mantener traducciones sincronizadas es costoso. Mitigacion: los
  maestros gobiernan, las traducciones se marcan como pendientes
  cuando el maestro avanza.
- Riesgo de redaccion inconsistente entre idiomas. Mitigacion: revisar
  con hablantes nativos antes de aceptar traducciones.

## Alternativas consideradas

- Solo ingles: rechazado por contradecir el posicionamiento del
  proyecto.
- Solo espanol: rechazado por limitar adopcion fuera de
  Latinoamerica.
- Traduccion automatica: rechazada por riesgo de imprecisiones en
  documentacion normativa.

## Notas

Esta decision se revisa cuando el comite tecnico se forme en Fase 4.
