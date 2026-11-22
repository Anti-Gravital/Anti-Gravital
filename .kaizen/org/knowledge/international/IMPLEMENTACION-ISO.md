# Guía práctica de implementación de ISO (enfoque actual, 2026)

Complemento de `ESTANDARES.md`. No sustituye la lectura de la norma pagada ni al auditor Certificado. Cada fase termina con **evidencia**, no con un documento bonito.

## Principio

Implementar "como se hace hoy" significa: **gestionar por riesgo, automatizar la evidencia y mantener el sistema vivo** (PDCA). Un ISMS o un sistema de cumplimiento no es un PDF: es un flujo de trabajo con dueños, plazos y métricas.

## A. ISO/IEC 27001:2022 — ruta de implementación en 8 pasos

1. **Definir alcance e contexto** (cláusulas 4): qué sistemas, datos y ubicaciones entran; partes interesadas; requisitos legales (en Panamá: Ley 81, Ley 23, Ley 478). Entregable: alcance aprobado por dirección.
2. **Liderazgo y política** (5): responsable, roles, política de seguridad. Entregable: política firmada + matriz de responsabilidades (RACI).
3. **Evaluación de riesgos** (6.1.2): activos → amenazas → vulnerabilidades → impacto/probabilidad → criterios → tratamiento. Entregable: metodología + matriz + plan de tratamiento con dueños y fechas.
4. **Declaración de aplicabilidad (SoA)** (6.1.3): para los 93 controles del Anexo A, decisión de inclusión/exclusión y justificación. Entregable: SoA revisada.
5. **Objetivos y controles** (6.2, 8): cómo se mide cada control (KPI), y procedimientos ejecutables. Entregable: procedimientos + pipeline de evidencia.
6. **Soporte y operación** (7, 8): competencias, concienciación, comunicación, documentación controlada, gestión de cambios, operación de controles. Entregable: registros de capacitación y bitácoras.
7. **Medición y mejora** (9, 10): monitoreo, auditoría interna, revisión de la dirección, no conformidades y acciones correctivas. Entregable: informe de auditoría interna + revisión direccional.
8. **Certificación (opcional)**: auditoría Stage 1 (documentación) y Stage 2 (implementación), vigilancia anual, recertificación cada 3 años.

**Controles que suelen dar más trabajo hoy:** inventario de activos, control de accesos y MFA, cifrado, gestión de parches, registro de eventos y monitoreo, respuesta a incidentes, seguridad en la nube, seguridad de proveedores, desarrollo seguro y continuidad.

**Evidencia automatizable en un repo:** `SECURITY.md`, `CODEOWNERS`, hooks y CI security, `docs/kaizen/postmortems/`, matrix de riesgos en `docs/kaizen/adr/`, métricas en `docs/kaizen/data/metrics.jsonl`.

## B. ISO 37301:2021 — sistema de gestión de cumplimiento

1. Mapa de obligaciones (leyes, regulatorias, contractuales, internas) con dueño y evidencia.
2. Matriz de riesgo de cumplimiento (probabilidad × impacto × sanción).
3. Política de cumplimiento, código de conducta y canal de reporte (protección al denunciante).
4. Controles: debida diligencia, aprobaciones, segregación de funciones, registros.
5. Capacitación y comunicación medible.
6. Monitoreo + auditoría + revisión de la dirección.
7. Mejora: acciones correctivas con issue, dueño y fecha (K-2/K-9).

**Evidencia:** matriz de obligaciones viva, actas de aprobación, resultados de capacitación, reportes de incidentes y su cierre.

## C. ISO 31000:2018 — riesgo aplicado al día a día

- Comunicación y consulta → alcance/contexto/criterios → identificación → análisis → evaluación → tratamiento → monitoreo y revisión → registro.
- Integrarlo en el flujo de issues: cada cambio relevante lleva campo "riesgo" y "rollback" en el descriptor (K-4).
- Usar la misma matriz para riesgo técnico, legal, financiero y de reputación; no crear cuatro procesos paralelos.

## D. Cómo mantenerlo vivo (lo que falla en la práctica)

| Fallo típico | Corrección |
| --- | --- |
| Documentos desactualizados | K-7: el descriptor lista los documentos tocados en el mismo commit |
| Evidencia manual y olvidada | CI genera bitácoras y métricas; hooks previenen secretos |
| Controles sin dueño | `CODEOWNERS` + RACI en el ADR |
| Auditoría como evento anual | Monitoreo continuo + retro quincenal (K-9) |
| Demasiados controles para equipo pequeño | Empezar por CIS v8.1 y mapear a ISO 27001:2022; evitar "certificar por moda" |

## E. Reglas del agente `cumplimiento-panama`

- Toda recomendación incluye: norma + versión + fecha de corte + fuente oficial + control + evidencia esperada + dueño humano.
- Si la implementación propuesta toca seguridad, pagos, datos personales o el propio sistema de gobernanza: `BLOCK` y humano (ADR-0002).
- Investigar la versión vigente con `kaizen-org research`, no asumir que la versión de este archivo sigue siendo actual.

**Fuentes:** catálogo ISO, NIST OLIR, guías oficiales; revalidar en `../SOURCES.md`.
