# Charter: infra-oss-observability

## Objetivo
Disenar adaptadores de contenedores/Kubernetes y stack open source de observabilidad bajo RFC-0038 aceptada, sin collector obligatorio

## KPI verificable
cargo test -p ag-observe sin collector ni protoc; --features otlp verde; runbook con imagenes fijadas y sin secretos

## Restricciones
- Decisiones con fuentes verificadas: .kaizen/team/EVIDENCIA.md
- Vetos: security, architecture, qa, compliance, scope, resource
- Proactividad con frenos: .kaizen/team/FLUJOS.md
