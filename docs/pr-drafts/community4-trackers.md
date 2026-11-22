# Descriptor de PR: community4/trackers

## Resumen

Reconcile the 13 open issues so the board reflects reality after wave 4

## Fase afectada

Gobernanza y trazabilidad del tablero de issues (post Fase 4.6), sin cambio de fase ni de roadmap.

## Tipo de cambio

- [ ] Codigo
- [ ] Dependencias de Cargo
- [ ] Cambio de DSL
- [ ] Cambio de arquitectura
- [ ] Cambio de security / stack
- [x] Documentacion tecnica y CHANGELOG (`docs/audits/issue-board-reconciliation.md`, `docs/INDEX.md`)
- [x] Etiquetas y comentarios de estado en GitHub (sin cierre de issues)
- [x] Cuerpos de los issues padre #76 y #142 (checklist y conteos verificados)

## Documentos relacionados

- `docs/audits/issue-board-reconciliation.md` (matriz, etiquetas, comentarios y condiciones de cierre)
- `docs/INDEX.md` (referencia al nuevo registro de auditoria)
- `CHANGELOG.md` (entrada "Issue board reconciliation (wave 4)")
- `docs/adr/0020-ag-data-orm-rls-decision.md` (difiere #154)
- `docs/adr/0021-chumsky-upgrade-deferral.md` (difiere #180)
- `docs/rfc/RFC-0009-ag-mail-native-mta.md` y `docs/modules/ag-mail/managed-api-dx-design.md` (puerta Mail A, #152)
- `docs/rfc/RFC-0012-ag-workers.md` seccion 17.4 / S6, `docs/adr/0013-ag-workers-execution-model.md` (difiere #112)
- `docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md` seccion 11 (bloquea #141)
- `docs/rfc/RFC-0011-ag-domains-control-plane.md` (#87, #179), `docs/audits/PRE_FASE5_RELEASE_GATE.md` (#156, #157)
- `docs/rfc/RFC-0026-chrono-replacement.md`, `docs/rfc/RFC-0029-oauth2-chrono-unblock.md` (#185)

## Plan de prueba

Verificacion realizada contra el tablero en vivo (sin acceso a codigo):

1. `gh issue view N --json body,labels,state` para los 13 issues: cuerpo, etiquetas y estado.
2. `gh api repos/Anti-Gravital/Anti-Gravital/issues/N/comments` para los comentarios recientes.
3. Conteo de hijos verificado uno por uno: #76 = 16 hijos (#78-#93), 15 CLOSED y #87 OPEN;
   #142 = 31 hijos, 26 CLOSED y 5 OPEN (#141, #152, #154, #156, #157). #149 CLOSED.
4. Los 13 comentarios de estado tienen <= 256 caracteres, sin emojis ni atribucion de herramientas.
5. `git diff --check` sobre los archivos modificados.
6. La seccion de cierre del descriptor coincide con `Closes: none` (regex de `.github/workflows/docs.yml`).

## Criterios de salida que avanza

- Ninguna casilla de `docs/roadmap/STATUS.md` cambia: no se cierra ninguna puerta.
- Tablero honesto: siete issues bloqueados por puerta externa, cuatro diferidos por decision aceptada,
  dos padres de seguimiento con conteos correctos.
- `blocked` anadido a #141, unico bloqueado externo que no lo tenia; se conservan p1/p2/p3, area y tipo.
- Condiciones de cierre exactas registradas para los 13 issues; ninguna cerrada falsamente.

## Cierre de issues

Refs #76
Refs #87
Refs #112
Refs #141
Refs #142
Refs #152
Refs #154
Refs #156
Refs #157
Refs #178
Refs #179
Refs #180
Refs #185
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (gobernanza, sin cambio de fase).
- [x] Respeta la documentacion (ADR-0020/0021, RFC-0009/0011/0012/0030).
- [x] No rompe arquitectura (no se toca codigo).
- [x] No anade complejidad innecesaria.
- [x] No crea dependencias circulares.
- [x] Compila (no aplica: sin cambios de codigo).
- [x] Pasa tests (no aplica: sin cambios de codigo).
- [x] Pasa fmt (no aplica: sin cambios de codigo).
- [x] Pasa clippy (no aplica: sin cambios de codigo).
- [x] Pasa audit (no aplica: sin cambios de codigo).
- [x] Tiene documentacion (`docs/audits/issue-board-reconciliation.md`, `docs/INDEX.md`, `CHANGELOG.md`).
- [x] Tiene trazabilidad (comentarios en los 13 issues y matriz de condiciones de cierre).
- [x] Mantiene coherencia con Anti-Gravital v4.1 (Blueprint canonico intacto; master docs sin cambios).
