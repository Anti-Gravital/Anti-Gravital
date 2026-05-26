# ADR-0009 — Reglas de gobernanza correctiva pre-Fase 5

## Contexto

Una auditoria externa de las fases 0-4.5 detecto fuerte desalineacion entre la
documentacion y el codigo: modulos funcionales marcados como "skeleton"/"vacio",
dependencias externas (Redis) tratadas como requisito en lugar de adaptador, y
documentacion que se actualizaba despues del codigo. Se requiere fijar reglas que
impidan reincidir antes de iniciar la Fase 5.

## Decision

Se incorporan a `CLAUDE.md` cinco reglas (estado real, adaptadores tras features con
modo nativo, docs en la misma PR que el codigo, instaladores auditados/firmados,
infraestructura externa reemplazable por nativa salvo RFC). Se crea `docs/DEBT.md`
como registro unico de deuda tecnica.

## Consecuencias

- Los README y cabeceras `//!` deben mantenerse fieles al codigo.
- Cada nueva integracion externa exige feature + modo nativo, reforzando la
  independencia de proveedores.
- La deuda tecnica deja de esconderse en comentarios "skeleton" dispersos.

## Alternativas

- No formalizar (rechazada: la auditoria mostro que la desalineacion reincide).
- Solo documentar sin regla en CLAUDE.md (rechazada: no es vinculante para agentes).

## Estado

Aceptada (2026-05-26). Relacionada con ADR-0007 (ag-mail/ag-domains) y RFC-0005.
