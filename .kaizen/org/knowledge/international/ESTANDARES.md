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

## Regla de decisión

1. ¿Existe obligación legal o contractual? → priorizar la norma.
2. ¿Es una buena práctica con evidencia de beneficio? → adoptar con ADR y KPI.
3. ¿Es moda sin evidencia? → descartar y documentar el porqué (rol `innovation`).

**Fuentes base:** ISO (catálogo), NIST (CSRC/OLIR), AICPA, PCI SSC, OWASP, FATF, IFRS Foundation, UE (EUR-Lex). Revisar en `SOURCES.md` y revalidar trimestralmente.
