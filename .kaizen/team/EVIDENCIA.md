# Ledger de evidencia verificada (fuente de verdad para decisiones)

Toda decisión que afecte cumplimiento, dinero, datos o arquitectura debe apoyarse en **fuentes verificadas**. La opinión del agente no es evidencia (K-10).

## Identificador de fuente

Cada fuente tiene un `source_id` estable en `.kaizen/org/knowledge/SOURCES.md`:

| Campo | Regla |
| --- | --- |
| `source_id` | `XX-NNN`, ej. `PA-001` (Panamá), `ISO-001`, `NIST-001` |
| `publisher` | Organismo emisor (Gaceta Oficial, ANTAI, SBP, DGI, ISO, NIST…) |
| `document` | Nombre exacto + número + año + fecha de publicación |
| `url` | URL oficial; si es Gaceta, incluir número y página/PDF |
| `retrieved` | Fecha de consulta (ISO 8601) |
| `sha256` | Hash del archivo descargado cuando se pueda (PDF) |
| `status` | `verified` \| `pending` \| `conflict` \| `stale` |
| `verified_by` | Agente verificador + humano responsable |

## Reglas duras

1. **REG-EV-01**: una fuente `pending`, `conflict` o `stale` no puede sustentar una decisión; el orquestador bloquea.
2. **REG-EV-02**: las fuentes secundarias sirven para localizar la oficial, nunca para fundar una obligación.
3. **REG-EV-03**: normativa panameña → citar **Gaceta Oficial** (número y fecha) o el texto del órgano competente; si solo hay resumen de un tercero, `pending`.
4. **REG-EV-04**: dos fuentes oficiales que difieren → `conflict`, escalado a humano; nunca se promedia.
5. **REG-EV-05**: revalidar trimestralmente; si cambia la versión, se actualiza `retrieved` y se registra el cambio en un expediente `kaizen-org research`.
6. **REG-EV-06**: todo dato personal, credencial o secreto queda fuera del ledger; se sanea con `kaizen-security redact`.
7. **REG-EV-07**: la decisión guarda `source_ids`, fecha de corte y hash; sin ellos no se considera tomada.

## Verificación automática

`scripts/kaizen-team verify <proyecto>` comprueba que cada `source_id` citado en las decisiones existe en `SOURCES.md`, que su archivo/URL está declarado y que su estado es `verified`. Si falla, la decisión queda `BLOCK` y el proyecto no avanza de etapa.

## Ejemplo de registro en una decisión

```yaml
decision_id: DEC-0007
tema: "Aplicar ITBMS a nuevas suscripciones"
source_ids: [PA-007, PA-009]
retrieved: 2026-11-21
status: verified
evidence: "DGI, ITBMS general 7%; Gaceta/DE según corresponda"
uncertainty: "Tasas especiales a verificar en DGI antes de facturar"
human_owner: "<responsable>"
```
