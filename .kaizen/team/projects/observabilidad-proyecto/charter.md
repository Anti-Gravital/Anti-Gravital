# Charter: observabilidad-proyecto

## Objetivo
Anadir scopes por proyecto y HealthRegistry nativo en ag-observe sin romper ADR-0009/ADR-0019; exporter OTLP sigue opt-in

## KPI verificable
cargo test -p ag-observe sin collector ni protoc; /metrics contiene project=...; componente caido cambia readiness

## Restricciones
- Decisiones con fuentes verificadas: .kaizen/team/EVIDENCIA.md
- Vetos: security, architecture, qa, compliance, scope, resource
- Proactividad con frenos: .kaizen/team/FLUJOS.md
