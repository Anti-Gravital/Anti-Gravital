# Digest de conocimiento: capacidades-negocio

- **Capacidades tecnología, finanzas y mercadeo** — `.kaizen/org/knowledge/business/CAPACIDADES.md` (sha256 941d710367cb)
- **Estándares internacionales** — `.kaizen/org/knowledge/international/ESTANDARES.md` (sha256 e49876e1783a)
- **Guía implementación ISO 2026** — `.kaizen/org/knowledge/international/IMPLEMENTACION-ISO.md` (sha256 6c6caff1c2ba)
- **Previsión de capacidades** — `docs/kaizen/org/FORECAST.md` (sha256 b1626ba47bb3)
- **Contextos de negocio y tecnología (Panamá)** — `.kaizen/org/knowledge/panama-compliance/CONTEXTOS.md` (sha256 475ced9df573)

## CAPACIDADES.md
# Capacidades especializadas: tecnología, finanzas y mercadeo

Responsable: `capacidades-negocio` | Fecha de corte: 2026-11-21

> Regla de oro: toda recomendación lleva **supuestos explícitos, métrica base, fórmula y forma de verificación**. Los números de ejemplo son ilustrativos, nunca evidencia (K-10).

## 1. Tecnología

- **Arquitectura**: límites de módulo, contratos versionados, ADR por decisión estructural, bajo acoplamiento. Anti-patrón: acoplar dominio a infraestructura.
- **APIs e integraciones**: contratos antes de código, idempotencia, versionado, errores tipados, timeouts y reintentos con backoff; nunca exponer secretos.
- **Confiabilidad**: SLO/SLI, error budget, DORA (frecuencia de despliegue, lead time, tasa de fallo, MTTR), benchmarks y postmortem sin culpables.
- **Seguridad**: defensa en capas, mínimo privilegio, fallo cerrado, OWASP ASVS/Top 10, CIS v8.1, NIST CSF 2.0, ISO 27001:2022.
- **Datos**: modelado, índices, migraciones reversibles, calidad de datos, linaje, privacidad por diseño (L-12).
- **IA aplicada**: evaluaciones reproducibles, guardrails, fallback determinista, medición de latencia y coste por inferencia; prompts versionados como archivos.
- **FinOps**: etiquetado de coste, presupuesto por entorno, apagado de recursos ociosos, coste por transacción.

**Checklist de propuesta técnica:** ¿contrato? ¿prueba? ¿rollback? ¿métrica? ¿coste? ¿riesgo? ¿dueño?

## 2. Finanzas

- **Estados financieros (IFRS/NIIF)**: estado de situación, resultado, flujo de efectivo y notas. IFRS 15 (reconocimiento de ingresos), IFRS 9 (instrumentos), IFRS 16 (arrendamientos). Verificar adopción local.
- **Liquidez**: `runway_meses = efectivo / burn_rate_mensual`; `burn_rate = gastos - ingresos` en periodo. Señal de alerta: runway < 6 meses sin financiamiento comprometido.
- **Economía unitaria**: `margen_contribucion = precio - coste_variable`; `LTV = ARPU × margen bruto / churn`; `ratio LTV:CAC` referencia ≈ 3:1; `payback_meses = CAC / margen_contribucion_mensual`.
- **Capital**: NPV = Σ(FCF_t/(1+r)^t) − inversión; TIR (IRR) con cautela en flujos no convencionales; CAPEX vs OPEX.
- **Precios**: coste + margen, valor percibido, competencia, segmentación; probar elasticidad antes de subir.
- **Panamá**: moneda balboa (PAB) en paridad 1:1 con USD; ITBMS general 7% (verificar tasas especiales y vigencia en DGI); precios de transferencia y reportes según norma aplicable.
- **Riesgo**: FX, crédito, liquidez, concentración de clientes; escenarios base/estrés.

**Regla:** ningún modelo financiero se entrega sin supuestos, sensibilidad y fuente de los datos.

## 3. Mercadeo y negocio

- **Embudo (AARRR)**: adquisición → activación → retención → referido → ingresos. Medir cada etapa, no solo "visitas".
- **Segmentación y posicionamiento**: ICP, personas, jobs-to-be-done, propuesta de valor verificable con entrevistas.
- **Canales**: orgánico, pago, socios, contenido, email/eventos; elegir por CAC y calidad, no por moda.
- **Atribución**: first-touch, last-touch, multi-touch; ninguna es verdad absoluta; combinar con experimentos incrementales.
- **Experimentos**: hipótesis, métrica primaria, muestra mínima, significancia estadística, duración fija, decisión pre-registrada. No mirar resultados a diario y decidir a ojo.
- **Retención**: cohortes, churn, RFM, encuestas; retener suele ser más barato que adquirir.
- **Marketing responsable**: consentimiento y privacidad (Ley 81/2019 si aplica), mensajes no engañosos, exclusiones y preferencias del usuario, construcción de marcas sin spam.
- **KPIs**: CAC, LTV, ROAS, CTR, CVR, ARPU, churn, NPS; cada uno con definición y fuente.

## ESTANDARES.md
# Mapa de estándares internacionales y su aplicación

Fecha de corte: 2026-11-21 | Verificar siempre la versión vigente en el organismo emisor (`.kaizen/org/RESEARCH.md`).

> No todos los estándares aplican a toda organización. El agente debe justificar aplicabilidad y no recomendar certificación sin un caso de negocio.

## Seguridad de la información y ciberseguridad

| Estándar / marco | Versión de referencia | Para qué sirve | Evidencia típica |
| --- | --- | --- | --- |
| ISO/IEC 27001 | 2022 | Sistema de gestión de seguridad de la información (ISMS); Anexo A con 93 controles | Alcance, política, declaración de aplicabilidad, evaluación de riesgos, auditoría interna |
| ISO/IEC 27002 | 2022 | Guía de implementación de los controles del Anexo A | Procedimientos y evidencia de control |
| NIST Cybersecurity Framework | 2.0 (2024) | Funciones GOVERN, IDENTIFY, PROTECT, DETECT, RESPOND, RECOVER | Perfil actual/deseado, hoja de ruta de brechas |
| NIST OLIR 27001↔CSF 2.0 | 1.0.0 | Mapeo informativo oficial entre ISO 27001:2022 y CSF 2.0 | Crosswalk documentado |
| CIS Controls | v8.1 | Controles priorizados, implementación práctica | Inventario, MFA, backups, logs |
| SOC 2 (AICPA TSC) | TSC 2017 | Informe de controles para clientes (seguridad, disponibilidad, confidencialidad, privacidad, integridad) | Descripción del sistema, pruebas de control, informe del auditor |
| PCI DSS | v4.0.1 | Protección de datos de tarjeta; **evitar almacenarlos** reduce alcance | SAQ/AOC, segmentación, scanners |
| OWASP ASVS / Top 10 / SAMM | ASVS 4.0, Top 10 2021 | Seguridad de aplicaciones y madurez | Resultados de pruebas, amenazas modeladas |

## Cumplimiento, riesgo y calidad

| Estándar | Versión | Para qué sirve | Evidencia |
| --- | --- | --- | --- |
| ISO 37301 | 2021 | Sistema de gestión de cumplimiento (compliance) | Política, matriz de obligaciones, reporte, auditoría |
| ISO 31000 | 2018 | Gestión de riesgos (principios y proceso) | Matriz de riesgos, criterios, monitoreo |
| ISO 9001 | 2015 | Gestión de calidad | Procesos, indicadores, acciones correctivas |
| ISO/IEC 20000-1 | 2018 | Gestión de servicios de TI | Catálogo, SLA, gestión de incidentes y cambios |
| ISO 22301 | 2019 | Continuidad del negocio | BIA, planes, ejercicios |
| COBIT | 2019 | Gobierno de TI empresarial | Objetivos de gobernanza, métricas |
| ITIL | 4 | Prácticas de gestión de servicios | Procesos incident/change/problem |

## Privacidad, finanzas y mercados

| Marco | Aplicación típica | Nota |
| --- | --- | --- |
| GDPR (Reglamento UE 2016/679) | Tratamiento de datos de residentes en UE | No asumir aplicabilidad universal; analizar alcance |
| FATF 40 Recomendaciones | Referencia internacional de BC/FT | Se refleja en la Ley 23 de Panamá (ver mapa nacional) |
| IFRS / NIIF | Informes financieros | IFRS 15 (ingresos), IFRS 9 (instrumentos), IFRS 16 (arrendamientos) — verificar versión local |
| Basilea III | Banca | Requisitos de capital y liquidez; supervisor local SBP |
| OCDE (precios de transferencia) | Operaciones vinculadas | Documentación y método; verificar norma panameña aplicable |

## IMPLEMENTACION-ISO.md
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

## FORECAST.md
# Previsión realista de capacidades y evolución de la organización de agentes

Fecha: 2026-11-21 | Estado: vivo, se revisa cada trimestre con ADR (K-1/K-9)

## 1. Qué pueden hacer bien los modelos hoy (2026)

Con **roles acotados, reglas explícitas, contratos escritos y evidencia verificable**, los modelos actuales son fiables para: tareas cortas y reversibles, lectura/escritura de archivos con límites, pruebas reproducibles, revisión con checklist, propuestas con evidencia y resolución de disputas en 2 turnos con árbitro. Es lo que implementa este repo.

## 2. Qué NO se les puede delegar hoy (aunque la autonomía sea 3)

- Planificar a largo plazo sin revalidación: la memoria en archivos (L-2) es más fiable que la "intuición" de una sesión.
- Decisiones sin evidencia reproducible: la palabra del agente no cuenta como prueba (K-10/L-10).
- Seguridad, pagos, autenticación, migraciones destructivas, borrado de datos y modificar el propio loop: siempre humano (L-3/L-4).
- Confianza ciega en telemetría, issues, web o buzones: son datos no confiables (L-5/S-15).
- "Organizaciones autónomas" tipo fictional: un modelo no sostiene por sí solo continuidad, responsabilidad ni criterio de negocio sin frenos externos.

## 3. Por qué esta guía está diseñada así

Ante esa realidad, el diseño no es "un superagente que dirige todo", sino **muchos agentes estrechos + orquestador + puertas**: 17 roles técnicos (`.kaizen/swarm/ROSTER.md`) y los puestos de negocio del inventario (`.kaizen/org/`), incluidos `cumplimiento-panama` y `capacidades-negocio`. Cada uno tiene propósito, entradas, salidas, KPI y veto limitado. La proactividad es **proponer con evidencia**, no actuar sin control. La investigación en internet se gestiona con `.kaizen/org/RESEARCH.md` (fuentes oficiales, contraste y fecha de corte), porque un modelo sin cita ni fecha no es fiable para normativa ni estándares.

## 4. Camino de mejora continua de la IA (pronóstico acotado)

| Horizonte | Supuesto observable | Cambio previsto en esta guía |
| --- | --- | --- |
| Corto (2026-2027) | Tool-use más fiable, contextos más largos, coste por token a la baja | Más rondas por issue, más documentación alimentada por agente, pulsos proactivos más frecuentes. Mismo conjunto de frenos. |
| Medio (2027-2028) | Agentes sostienen tareas de varias horas con menos deriva; verificación automática más precisa | Autonomía `scoped` para más puestos, previo ADR y KPI en verde. El veto humano se mantiene en seguridad/dinero. |
| Largo | Modelos con memoria persistente fiable y calibración demostrable | Revisar si el orquestador puede reducirse; nunca eliminar evidencia, ratchet ni CODEOWNERS. |

Este pronóstico es **falsable**: si los KPI (tasa de revert, ruido de issues, coste por cambio, tiempo a PR verificado) empeoran, se pausa y se ajusta el diseño mediante PR humano, no subiendo la autonomía (ver señales de daño en KAIZEN-LOOP.md §11).

## 5. Cómo se refinan los agentes (el inventario es el mecanismo)

1. `kaizen-org list` → inventario vivo de puestos, departamentos, autonomía y KPI.
2. `kaizen-org copy <src> <dst> --name ...` → **copiar** un puesto y especializarlo (marketing general → marketing visual; finanzas → finanzas panameñas) sin tocar el original.
3. `kaizen-org edit <id>` / `save <id>` → **editar y guardar** con subida de versión, diario y ratchet.
4. `kaizen-org feed <id> <documento>` → **alimentar conocimiento**; el agente lo consume en su próxima iteración y el `digest` lo resume saneado.
5. Retro quincenal: hasta 3 mejoras a definiciones, con dueño, issue y fecha (K-9).

## 6. Límites honestos


## CONTEXTOS.md
# Contextos de aplicación: negocio y tecnología (Panamá)

Fecha de corte: 2026-11-21 | Complemento de `MAPA-NORMATIVO.md` | Responsable: `cumplimiento-panama`

> No es asesoría legal, fiscal ni contable. Cada fila exige verificar la norma vigente en `../SOURCES.md` y firma humana antes de aplicarla.

## 1. Contexto de negocio

| Tema | Marco verificado | Obligación / control derivado | Evidencia esperada |
| --- | --- | --- | --- |
| Constitución y gobierno societario | Ley 23/2015 (definición de beneficiario final); Registro Público; normativa sectorial | Identificar beneficiario final, documentar estructura de control, mantener registros actualizados | Matriz de control societario, actas, registro de beneficiario final |
| BC/FT/FPADM | Ley 23/2015; DE 35/2022; Ley 129/2020; UAF; SBP/SSNF/SMV | KYC, debida diligencia, perfil de riesgo, monitoreo, reporte de operación sospechosa, conservación de registros | Política BC/FT, expedientes de cliente, bitácora de monitoreo, capacitación |
| Contratos y firma electrónica | Ley 51 de 22 jul 2008 (documentos y firmas electrónicas, almacenamiento tecnológico y comercio electrónico); Ley 82 de 2012 (Registro Público como autoridad de certificación); DE 40 de 2009; DE 684 de 2013 | Validez de documentos/firmas, registro de prestadores cuando aplique, integridad y no repudio, conservación | Certificado o prueba de firma, hash de documento, registro del prestador |
| Protección al consumidor y competencia | Ley 45 de 31 oct 2007; Autoridad de Protección al Consumidor y Defensa de la Competencia (APCDC) | Información clara, contratos de adhesión sin cláusulas abusivas, publicidad no engañosa, garantías, competencia leal | Avisos, términos y condiciones, política de devoluciones, revisión de campañas |
| Fiscalidad | DGI: ITBMS general 7% (tasas especiales a verificar); Formulario 430 / e-Tax 2.0; SFEP | Facturación, declaración, soporte de exenciones, trazabilidad de operaciones | Facturas, declaraciones, conciliación, configuración fiscal del sistema |
| Laboral | MITRADEL / Código de Trabajo (texto consolidado) | Contratos, jornada, seguridad social, políticas internas | Expedientes, política de RRHH, registros |
| Operaciones reguladas | SBP (banca), SMV (valores), Superintendencia de Seguros, SSNF (no financieros), ASEP | Licencias, reportes periódicos, controles sectoriales | Matriz de regulator + obligación + evidencia + plazo |

## 2. Contexto tecnológico

| Tema | Marco verificado | Obligación / control derivado | Evidencia esperada |
| --- | --- | --- | --- |
| Datos personales | Ley 81/2019; DE 285/2021; ANTAI | Inventario de tratamientos, consentimiento, aviso de privacidad, derechos ARCO, transferencias internacionales, medidas de seguridad, plazos de conservación | Registro de tratamientos, plantilla de consentimiento, procedimiento ARCO, matriz de transferencias |
| Violación de seguridad | Ley 81 + DE 285 (plazo exacto **a verificar en texto oficial**); Ley 478/2025; CSIRT Panamá; SBP GRT para sector financiero | Detección, contención, evaluación, notificación al regulador o al titular cuando aplique, postmortem sin culpables | Plan de respuesta, bitácora de incidente, registro de notificación, postmortem |
| Cibercrimen | Ley 478 de 4 ago 2025 (Código Penal, Código Procesal Penal y Ley 11/2015) | Delitos digitales tipificados; cooperación internacional; resguardo de evidencia digital | Política de respuesta, cadena de custodia, contacto con autoridad |
| Seguridad de la información | ISO/IEC 27001:2022 + 27002:2022; NIST CSF 2.0; CIS v8.1 | ISMS, SoA, evaluación de riesgos, controles, monitoreo, mejora continua | Alcance, SoA, matriz de riesgos, evidencia de controles, auditoría interna |
| Desarrollo seguro | OWASP ASVS 4.0 / Top 10 2021; SAMM | Revisión de código, pruebas de seguridad, amenaza modelada, secretos fuera del repo | CI security, escaneo de dependencias, `SECURITY.md`, resultados |
| Disponibilidad / continuidad | ISO/IEC 20000-1:2018; ISO 22301:2019 | Gestión de incidentes/cambios, BIA, planes y ejercicios de restauración | SLA, bitácoras, backups probados, ejercicio de recuperación |
| IA y datos | Marca de cumplimiento de Ley 81 si la IA trata datos personales | Minimización, base legal, eval de privacidad, fallback determinista, sin PII en prompts | Registro de modelos/datos, evaluaciones, redacción de logs |

## 3. Regla de cruce entre contextos

Cuando un cambio toca **ambos** contextos (por ejemplo, un sistema que factura con datos personales y firma electrónica):

1. El agente `cumplimiento-panama` aplica `REGLAS-CUMPLIMIENTO.md` y emite dictamen con veto si falta fuente oficial o control.
2. `seguridad` revisa la capa tecnológica; `finanzas-panamas` revisa el impacto fiscal; `arquitecto` revisa contratos.
3. Se registra una decisión con `kaizen-org decide`, dueño humano y fecha de revisión (`.kaizen/org/CONSENSUS.md`).
4. Nunca se auto-aprueba: incluso en nivel 3, este cruce escala a humano (L-3/L-4, ADR-0002).

