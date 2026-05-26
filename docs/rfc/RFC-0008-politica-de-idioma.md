# RFC-0008 - Politica de idioma: ingles canonico y vitrina bilingue

- Estado: aceptada
- Autor: Angel Nereira
- Fecha de borrador: 2026-05-24
- Fase objetivo: Cierre documental Fase 4.0-4.5 (previo a Fase 5)
- Modulos o crates afectados: todos (comentarios de codigo); docs/
- RFC predecesora: ninguna
- ADR relacionada: supersede parcialmente ADR-0002
- Periodo de comentarios: abreviado por decision del BDFL (cierre de fase)

## 1. Motivacion

`ADR-0002` (2026-05-19) fijo documentacion bilingue con **espanol como
predeterminado** y dejo escrito que la decision "se revisa cuando el comite
tecnico se forme en Fase 4". Estamos cerrando la Fase 4.5; es el momento
previsto para revisar la politica.

Durante las fases 1-4.5 el codigo acumulo ~3200 lineas de comentarios en
espanol distribuidas en 118 archivos `.rs`, y la documentacion crecio a 147
archivos markdown (~29 000 lineas) mayoritariamente en espanol. Sin una
politica de idioma clara para el **codigo** (ADR-0002 solo cubria docs), el
proyecto queda con una barrera de entrada para contribuidores
internacionales justo cuando se prepara para abrir la Fase 5 (`ag-cloud`) y
buscar adopcion mas amplia.

## 2. Problema

- El ingles es el idioma franco del open source. React, Rust, Kubernetes,
  Tokio y practicamente todo proyecto de infraestructura con adopcion global
  mantienen codigo y documentacion tecnica en ingles. Comentarios en espanol
  excluyen a la mayoria de contribuidores potenciales.
- ADR-0002 no legislo el idioma de los **comentarios de codigo**; quedo
  como vacio normativo que se lleno por defecto con espanol.
- Hacer *toda* la documentacion bilingue inline (EN+ES en cada archivo)
  duplicaria ~29 000 lineas a ~58 000, perjudicaria la escaneabilidad,
  ensuciaria los diffs y garantizaria desincronizacion entre idiomas.

## 3. Alternativas consideradas

1. **No hacer nada (mantener espanol por defecto).** Ventaja: cero trabajo.
   Desventaja: perpetua la barrera de entrada internacional. Rechazada: el
   objetivo declarado es adopcion amplia en Fase 5+.
2. **Todo en ingles, sin espanol.** Ventaja: simplicidad maxima. Desventaja:
   contradice el posicionamiento del proyecto (nace en Panama, primer foco
   Latinoamerica). Rechazada.
3. **Todo bilingue inline.** Ventaja: cada archivo autocontenido en dos
   idiomas. Desventaja: duplica volumen, mata escaneabilidad, diffs ruidosos,
   desincronizacion. Rechazada como mala practica para un corpus de 147
   archivos.
4. **Carpetas espejo `docs/es/` + `docs/en/` completas.** Ventaja: limpio.
   Desventaja: obliga a traducir y mantener 147 archivos en paralelo.
   Rechazada por costo de mantenimiento desproporcionado en esta etapa.
5. **Ingles canonico + vitrina bilingue (elegida).** Detalle en seccion 4.

## 4. Diseno propuesto

### 4.1 Codigo

- Todos los comentarios de codigo (`//`, `///`, `//!`) en **ingles**.
- Los identificadores ya estaban en ingles; se mantiene.
- Los mensajes de error orientados al usuario final pueden permanecer en el
  idioma que la UX requiera, pero los comentarios que los rodean van en
  ingles.

### 4.2 Documentacion

- **Ingles es el idioma canonico** de la documentacion tecnica profunda
  (`docs/architecture/`, `docs/modules/`, `docs/dsl/`, `docs/rfc/`,
  `docs/adr/`, `docs/benchmarks/`, `docs/security/`, `docs/governance/`).
- **Documentos vitrina bilingues (EN+ES, mismo archivo, EN primero,
  separados por regla horizontal):**
  - `README.md` (raiz)
  - Los tres maestros en `docs/master/`
  - Capitulos del manual en `docs/manual/`
- Patron de vitrina: ancla de idioma al inicio (`English | Espanol`),
  seccion inglesa primero (canonica), seccion espanola despues. La seccion
  espanola es traduccion de la inglesa; si divergen, gana la inglesa.
- Las carpetas `docs/es/` y `docs/en/` se mantienen como indices por idioma
  (uso pre-existente de ADR-0002).

### 4.3 Cambios en CI o tooling

- `.github/workflows/docs.yml` ya valida ausencia de emojis y de evidencia
  de herramientas IA; no requiere cambios por esta RFC.
- Los hashes SHA-256 de los maestros en `VERSION.md` y en el workflow se
  recalculan al actualizar los maestros (procedimiento existente).

### 4.4 Cambios en documentacion maestra

- `CLAUDE.md`: nueva regla de idioma (codigo en ingles; vitrina bilingue).
- `ADR-0002`: marcado como superseded por `ADR-0008`.
- Los tres maestros pasan a formato vitrina bilingue.

## 5. Plan de implementacion

PR unico de cierre documental (rama `docs-cierre-fase-4.5`):

1. RFC-0008 + ADR-0008 + actualizacion de ADR-0002 + regla en CLAUDE.md.
2. Maestros actualizados a estado real 4.0-4.5 y formato bilingue;
   Blueprint v4.1.md fuente; VERSION.md + workflow con hashes nuevos.
3. README raiz bilingue y fiel a la implementacion.
4. Conversion de comentarios de codigo a ingles, crate por crate.

## 6. Riesgos

- **Desincronizacion EN/ES en vitrina.** Probabilidad: media. Impacto: bajo.
  Mitigacion: la seccion inglesa es canonica; la espanola se marca pendiente
  si la inglesa avanza.
- **Error al traducir comentarios tecnicos.** Probabilidad: baja. Impacto:
  medio (un comentario erroneo confunde). Mitigacion: traduccion conservadora
  que preserva el significado; verificacion `cargo build/test/clippy/fmt`
  tras cada crate (los comentarios no afectan compilacion, pero el doc-test
  si).
- **Rechazo de la comunidad hispanohablante.** Probabilidad: baja. Impacto:
  bajo. Mitigacion: la vitrina (lo primero que ve un recien llegado)
  permanece bilingue.

## 7. Impacto

- **Alcance del producto:** sin cambios; es politica documental.
- **Cronograma:** una fase documental de cierre antes de Fase 5.
- **Complejidad operacional:** se reduce (un idioma canonico de codigo).
- **APIs publicas:** sin cambios de firma; los doc-comments cambian de idioma.
- **Documentacion existente:** maestros y README pasan a bilingue; docs
  tecnicos profundos quedan en ingles canonico (migracion gradual permitida).

## 8. Rollback

Revertir el PR de cierre documental restaura el estado espanol-por-defecto.
Indicador de rollback: caida medible en contribuciones o quejas formales de
la comunidad hispanohablante por perdida de accesibilidad. No se anticipa,
dado que la vitrina permanece bilingue.

## 9. Decision

- Decisor: BDFL (Angel Nereira).
- Fecha de decision: 2026-05-24.
- Resultado: aceptada.
- Justificacion: el ingles canonico abre el proyecto a contribuidores
  globales de cara a Fase 5, mientras la vitrina bilingue preserva la
  accesibilidad para el foco inicial latinoamericano. Cumple el punto de
  revision anticipado por ADR-0002.

## 10. Referencias

- `docs/adr/0002-bilingual-documentation.md` (superseded en parte)
- `docs/adr/0008-politica-de-idioma.md` (registro de esta decision)
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 1
- `CLAUDE.md` regla de idioma
