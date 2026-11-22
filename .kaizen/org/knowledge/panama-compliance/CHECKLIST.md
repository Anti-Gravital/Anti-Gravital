# Checklist operativo de cumplimiento (Panamá + estándares)

Uso: el agente `cumplimiento-panama` lo aplica a cada propuesta. Cada casilla requiere **evidencia reproducible**; no basta con marcarla.

## A. Datos personales (Ley 81/2019 + DE 285/2021)

- [ ] Existe inventario de datos personales tratados y su finalidad.
- [ ] Base de tratamiento documentada con responsable, plazo de conservación y categorías.
- [ ] Aviso de privacidad claro y accesible.
- [ ] Consentimiento válido cuando aplique, con prueba de otorgamiento y revocación.
- [ ] Procedimiento ARCO con plazos y bitácora de solicitudes.
- [ ] Transferencias internacionales identificadas y con base legal.
- [ ] Medidas técnicas: cifrado, control de acceso, minimización, audit trail.
- [ ] Sin datos personales en logs, issues, buzón ni PR (S-8/L-12).

## B. BC/FT/FPADM (Ley 23/2015 + reglamentos)

- [ ] Cliente/contraparte identificado y verificado (KYC).
- [ ] Beneficiario final identificado cuando aplique.
- [ ] Perfil de riesgo documentado y revisado.
- [ ] Monitoreo transaccional y umbrales justificados.
- [ ] Procedimiento de reporte de operación sospechosa sin filtración al cliente.
- [ ] Conservación de registros y cadena de custodia.
- [ ] Capacitación periódica del equipo.

## C. Ciberseguridad (Ley 478/2025 + CSIRT + normativa sectorial)

- [ ] Inventario de activos y dueño de cada activo.
- [ ] MFA en accesos privilegiados y rotación de credenciales.
- [ ] Gestión de parches y vulnerabilidades con plazos por severidad.
- [ ] Backups probados y plan de restauración medido (RTO/RPO).
- [ ] Plan de respuesta a incidentes con contacto a CSIRT/superintendencia cuando aplique.
- [ ] Reporte de incidentes TI al regulador si el sector lo exige (SBP GRT).
- [ ] Postmortem sin culpables y acción preventiva convertida en issue (K-8).

## D. Fiscal (ITBMS / DGI)

- [ ] Tasa aplicable verificada en fuente DGI (general 7%; especiales según norma).
- [ ] Facturación y Formulario 430 según régimen vigente.
- [ ] Registro de operaciones exentas y su soporte.
- [ ] Escenarios calculados con supuestos explícitos y revisión humana.

## E. Sociedades y sectores regulados

- [ ] Tipo societario y regulador aplicable identificados.
- [ ] Beneficiario final y estructura de control documentados (cuando aplique).
- [ ] Licencias, permisos y reportes periódicos del sector listados.
- [ ] Contratos con cláusulas de cumplimiento, auditoría y terminación.

## F. Estándares internacionales (ver `../international/ESTANDARES.md`)

- [ ] ISO/IEC 27001:2022 → ISMS y Anexo A; mapeo opcional a NIST CSF 2.0.
- [ ] ISO 37301:2021 → sistema de gestión de cumplimiento.
- [ ] ISO 31000:2018 → gestión de riesgos.
- [ ] PCI DSS v4.0.1 → solo si hay datos de tarjeta (evitar almacenarlos).
- [ ] SOC 2 (AICPA TSC) → cuando lo exijan clientes.
- [ ] GDPR → si hay residentes en UE; no asumir aplicabilidad universal.

## G. Salida del dictamen

- [ ] Cada hallazgo tiene: norma fuente, fecha de corte, control propuesto, dueño humano y riesgo.
- [ ] Sin evidencia o con norma no verificada → `BLOCK` de cumplimiento + `needs-human`.
- [ ] Hogar de la evidencia: `docs/kaizen/adr/` o el descriptor del issue, nunca el buzón.
