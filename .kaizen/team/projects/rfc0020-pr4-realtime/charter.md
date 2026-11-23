# Charter: rfc0020-pr4-realtime

## Objetivo
RFC-0020 PR4: migrar ag-realtime (config.rs) de std::env::var a EnvLoader declarado

## KPI verificable
11 claves config declaradas; lecturas #[cfg(test)] de external.rs preservadas; CI verde

## Restricciones
- Decisiones con fuentes verificadas: .kaizen/team/EVIDENCIA.md
- Vetos: security, architecture, qa, compliance, scope, resource
- Proactividad con frenos: .kaizen/team/FLUJOS.md
