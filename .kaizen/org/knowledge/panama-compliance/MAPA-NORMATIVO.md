# Mapa normativo de cumplimiento — Panamá

Fecha de corte: 2026-11-21 | Fuentes: `../SOURCES.md` | Responsable: `cumplimiento-panama`

> **Aviso:** este mapa es apoyo informativo para agentes, **no asesoría legal, fiscal ni contable**. La normativa cambia: antes de concluir, verifica la versión vigente en la fuente oficial y exige firma de un profesional humano responsable. Ninguna afirmación sin cita verificable es válida (K-10/S-15).

## 1. Protección de datos personales

- **Ley 81 de 26 de marzo de 2019**, sobre protección de datos personales. Establece principios, derechos, obligaciones y procedimientos para el tratamiento de datos de personas naturales por personas naturales o jurídicas, públicas o privadas.
- **Decreto Ejecutivo 285 de 2021**, reglamento de la Ley 81 (publicado 2021-05-28).
- **Autoridad:** ANTAI (Autoridad Nacional de Transparencia y Acceso a la Información).
- **Derechos ARCO:** acceso, rectificación, cancelación y oposición.
- **Controles derivados para un sistema:** inventario de bases de datos, base de tratamiento con consentimiento, aviso de privacidad, procedimiento ARCO, medidas de seguridad, registro de transferencias internacionales, evaluación de impacto cuando aplique, y plazos de conservación.
- **Regla del agente:** todo dato personal en logs, issues o telemetría se considera un hallazgo de seguridad (S-8/L-12); no se copia al buzón ni al PR.

Fuentes: Gaceta Oficial 28743-A (Ley 81); ANTAI (comunicado de reglamentación, 2021-05-28); vLex Panamá (DE 285 de 2021).

## 2. Prevención de blanqueo de capitales, terrorismo y proliferación (BC/FT/FPADM)

- **Ley 23 de 27 de abril de 2015**, adopta medidas para prevenir BC/FT/FPADM y dicta otras disposiciones. Define "beneficiario final" como la persona natural que posee, controla o ejerce influencia significativa sobre la relación de cuenta, contractual o de negocios, o sobre una persona jurídica, fideicomiso u otra estructura.
- **Decreto Ejecutivo 35 de 2022**, reglamentación de la Ley 23 (verificar texto consolidado).
- **Ley 129 de 2020** y normativa conexa: marco de supervisión de sujetos obligados no financieros (SSNF).
- **Autoridades:** UAF (Unidad de Análisis Financiero) y superintendencias según sector (SBP banca, SMV valores, Superintendencia de Seguros, SSNF no financieros).
- **Controles derivados:** política BC/FT, KYC/debida diligencia del cliente, identificación de beneficiario final, perfil de riesgo, monitoreo transaccional, reporte de operaciones sospechosas, conservación de registros, capacitación y auditoría independiente.
- **Regla del agente:** jamás propone eludir controles ni "optimizar" reportes; cualquier duda se escala a humano con `needs-human`.

Fuentes: Gaceta Oficial 27768-B (Ley 23, 2015-04-27); MEF/CNB; SBP (leyes y decretos AML); MEF (Guía de Beneficiarios Finales para Panamá, 2026-02).

## 3. Ciberseguridad y cibercrimen

- **Ley 478 de 4 de agosto de 2025**: modifica y adiciona artículos al Código Penal, al Código Procesal Penal y a la Ley 11 de 2015 (asistencia jurídica internacional en materia penal) en materia de ciberdelincuencia.
- **CSIRT Panamá** (equipo nacional de respuesta a incidentes, `cert.pa`) y **SOC Gubernamental** creado por **Decreto Ejecutivo 53 de 11 de junio de 2025** (Gaceta Oficial 30-298-A) como unidad centralizada de prevención, detección y respuesta.
- **Sector financiero:** la SBP opera un sistema de reportes de eventos e incidentes de TI y seguridad de la información.
- **Controles derivados:** inventario de activos, gestión de vulnerabilidades, MFA, cifrado en tránsito y reposo, respaldo probado, plan de respuesta a incidentes con plazos de reporte, bitácoras inmutables y postmortem sin culpables (K-8).

Fuentes: Presidencia de la República (Ley 478, 2025-08-04); CSIRT Panamá (`cert.pa`); Gaceta Oficial 30524-B (2026); SBP (reporte GRT).

## 4. Fiscal y facturación

- **ITBMS** (Impuesto de Transferencia de Bienes Muebles y Servicios): la DGI describe la retención general del **7%** sobre el valor de venta, dación en pago, aporte a sociedades, cesión u otros actos que transmitan dominio en el territorio nacional.
- Fuentes secundarias 2026 reportan tasas especiales (10% en ciertos servicios de hospedaje y bebidas alcohólicas; 15% en tabaco) y declaración mensual mediante **Formulario 430** en **e-Tax 2.0** con plazo aproximado de 15 días; **verificar siempre en DGI** antes de aplicar.
- **Facturación electrónica SFEP**: obligaciones según régimen y umbrales vigentes; verificar en DGI.
- **Regla del agente:** puede calcular escenarios con supuestos explícitos, pero no afirma obligación tributaria sin fuente oficial y revisión humana.

Fuente primaria: DGI (`dgi.mef.gob.pa/itbms/Itbms`). Fuentes secundarias: guías 2026 (uso solo como pista, no como autoridad).

## 5. Societario, consumo y sectores regulados

- Sociedades mercantiles y beneficiario final: la definición de la Ley 23 es la referencia transversal; las obligaciones específicas dependen del tipo societario y del regulador (SBP, SMV, Seguros, SSNF). **Verificar norma aplicable antes de afirmar.**
- Protección al consumidor, competencia, propiedad intelectual y contratación electrónica: evaluar caso por caso con fuente oficial.
- Sectores con supervisor propio: banca (SBP), valores (SMV), seguros, servicios públicos (ASEP), salud, telecomunicaciones.

## 6. Procedimiento del agente ante una propuesta

1. Clasificar: ¿hay datos personales? ¿Hay dinero/transacciones? ¿ Hay sistema crítico o incidente? ¿Hay venta/ITBMS? ¿Hay regulador sectorial?
2. Para cada dominio aplicable, marcar el `CHECKLIST.md` y citar la fuente de `SOURCES.md` con fecha de corte.
3. Emitir dictamen con: obligación, evidencia, control propuesto, responsable humano y riesgo de no hacerlo.
4. Si la norma no está verificada o el cambio afecta seguridad/dinero/datos: `BLOCK` de cumplimiento y escalar a humano (ADR-0002).
