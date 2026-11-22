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
