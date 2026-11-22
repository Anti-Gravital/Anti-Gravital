# Digest de conocimiento: verificador-evidencias

- **Ledger de evidencia verificada** — `.kaizen/team/EVIDENCIA.md` (sha256 87aedb914ab2)
- **Registro de fuentes** — `.kaizen/org/knowledge/SOURCES.md` (sha256 8b11e0a8b083)
- **Modelo de outsourcing** — `.kaizen/org/OUTSOURCING.md` (sha256 c41618064ea7)

## EVIDENCIA.md
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

## SOURCES.md
# Registro de fuentes (revalidar trimestralmente)

Protocolo: `.kaizen/org/RESEARCH.md`. Fecha de corte de esta revisión: **2026-11-21**. Toda fuente externa es dato no confiable (S-15/L-5).

## Registro de IDs de evidencia (usar en `source_ids` de decisiones)

Estado: `verified` = fuente oficial consultada; `pending` = requiere verificación adicional; `conflict` = fuentes oficiales difieren.

| ID | Emisor | Documento | Estado |
| --- | --- | --- | --- |
| PA-001 | Gaceta Oficial | Ley 81/2019 (Gaceta 28743-A) | verified |
| PA-002 | ANTAI / Ministerio Público | Decreto Ejecutivo 285/2021 | verified |
| PA-003 | Gaceta Oficial | Ley 23/2015 (Gaceta 27768-B) | verified |
| PA-004 | SBP | Leyes y decretos AML (DE 35/2022) | verified |
| PA-005 | MEF / CNBC | Guía de Beneficiarios Finales 2026 | verified |
| PA-006 | Presidencia / Gaceta | Ley 478/2025 (ciberdelincuencia) | verified |
| PA-007 | CSIRT Panamá / Gaceta | DE 53/2025 (SOC Gubernamental) | verified |
| PA-008 | DGI | ITBMS general 7% | verified |
| PA-009 | DGI | Tasas especiales y Facturación Electrónica SFEP | pending |
| PA-010 | Gaceta / Órgano Judicial | Ley 45/2007 (consumidor y competencia) | verified |
| PA-011 | Gaceta / firmaelectronica.gob.pa | Ley 51/2008 (documentos y firmas electrónicas) | verified |
| PA-012 | Asamblea Nacional | Ley 82/2012 y DE 684/2013 (Registro Público) | pending |
| PA-013 | MITRADEL | Código de Trabajo (texto consolidado) | verified |
| PA-014 | Gaceta Oficial | Ley 53/2005 + DE 349/2021 (validez de Gaceta Digital) | verified |
| PA-015 | Gaceta Oficial / Legispan | Rutas y patrón de PDFs (mapa `GACETA-OFICIAL.md`) | pending |
| ISO-001 | ISO | ISO/IEC 27001:2022 | verified |
| ISO-002 | ISO | ISO 37301:2021 | verified |
| NIST-001 | NIST | Cybersecurity Framework 2.0 | verified |
| AICPA-001 | AICPA | TSC 2017 / SOC 2 | verified |
| PCI-001 | PCI SSC | PCI DSS v4.0.1 | verified |
| OWASP-001 | OWASP | ASVS 4.0 / Top 10 2021 | verified |
| INT-001 | FATF | 40 Recomendaciones | verified |
| INT-002 | IFRS Foundation | IFRS 15/9/16 (adopción local a verificar) | pending |
| LOCAL-001 | Documento interno | CV Arquitecto IA Generativa (Ángel Nereira) | verified |
| LOCAL-002 | Documento interno | CV DevOps / Platform Engineering (Ángel Nereira) | verified |
| LOCAL-003 | Documento interno | gravital-experience-ui (plan de diseño Flutter M3E) | verified |
| LOCAL-004 | Documento interno | PlentyPay: perfiles de negocio, módulos, roles y permisos | verified |

## Panamá — oficiales


## OUTSOURCING.md
# Outsourcing de tecnología específica

Cuando una tecnología requiere profundidad que la escuadra no tiene, se **subcontrata** a un agente especialista o a un proveedor externo. No es "delegar y olvidar": es un contrato con entregables, evidencia y puertas.

## 1. Cuándo subcontratar

- La tecnología está fuera del conocimiento verificable del equipo (ej. módulo específico de Wasmtime, certificación fiscal de un PAC, versión nueva de Flutter).
- Se necesita un **spike** acotado en tiempo con criterio de aceptación medible.
- Exige una certificación o acceso que el equipo no posee (siempre con humano dueño).

**No se subcontrata**: seguridad, claves, pagos, datos personales, decisiones regulatorias, rutas protegidas (L-4), ni el propio sistema de gobernanza.

## 2. Contrato (generado por `kaizen-org outsource`)

Ubicación: `.kaizen/org/outsourcing/<slug>/contract.md`

| Campo | Contenido obligatorio |
| --- | --- |
| `technology` | Tecnología concreta y versión a verificar |
| `scope` | Qué se hace y qué **no** se hace |
| `deliverables` | Artefactos verificables (código, ADR, benchmark, documento) |
| `acceptance` | Criterios medibles y comando reproducible |
| `evidence` | `source_ids` o comandos; todo pasa por `verificador-evidencias` |
| `cost_cap` | Tope de tiempo/coste aprobado por humano |
| `deadline` | Fecha ISO 8601 |
| `owner` | Dueño interno que recibe y aprueba |
| `reviewers` | Mínimo: `arquitecto` + `verificador-evidencias` (+ `cumplimiento-panama` si aplica) |
| `secrets` | **Ninguno**. El externo nunca recibe credenciales ni ve `.env` (S-2) |
| `access` | Sin push, sin `gh`, sin rutas protegidas; worktree temporal |
| `ip` | Licencia declarada de dependencias y artefactos (S-9) |
| `status` | `open | in_progress | delivered | accepted | rejected` |

## 3. Reglas

1. **Salida no confiable**: todo lo que entregue el externo se trata como dato (S-15/L-5); se revisa con tests, `kaizen check` y `kaizen-security check` antes de integrar.
2. **Versión verificada**: el contrato exige versión consultada en fuente oficial; si no está, `pending` y no se acepta.
3. **Sin secretos ni privilegios**: entorno limpio, sin tokens, sin red salvo la estrictamente necesaria y aprobada.
4. **Recepción**: `kaizen-org outsource-accept <slug>` solo marca `accepted` si hay evidencia verificada y dictamen de los revisores; si falta, `rejected` con hallazgo S-11.
5. **Aprendizaje**: al cerrar, el conocimiento del especialista se convierte en `feed` + `digest` para que la escuadra interna ya no dependa del externo la próxima vez (mejora continua).


