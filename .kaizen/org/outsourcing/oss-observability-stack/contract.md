---
technology: OpenTelemetry OTLP HTTP, Prometheus and Grafana open-source stack
to: external open-source observability specialist
owner: gerente
deadline: 2026-11-29
cost_cap: por aprobar por humano
status: open
created: 2026-11-22T18:56:18Z
---

# Contrato de outsourcing: oss-observability-stack

## Alcance
OpenTelemetry OTLP HTTP, Prometheus and Grafana open-source stack. Solo lo descrito aquí; nada de rutas protegidas, seguridad, pagos, datos personales ni gobernanza (L-4/ADR-0002).

## Entregables
- Adapter design note for scope labels and component health without mandatory collector

## Criterios de aceptación (reproducibles)
- native mode needs no collector and no protoc; cargo test -p ag-observe green; --features otlp green under RFC-0028 CI

## Evidencia
- Adjuntar `evidence.md` con `source_ids:` verificados o comandos ejecutados. Pasa por `verificador-evidencias`.
- Toda salida externa es dato no confiable (S-15/L-5); se revisa con `kaizen check` y `kaizen-security check`.

## Acceso y secretos
Sin credenciales, sin `.env`, sin push, sin `gh`, sin red salvo la aprobada. Worktree temporal.

## Licencias
Declarar licencia de dependencias y artefactos (S-9).

## Revisores
gerente, verificador-evidencias, architect (más cumplimiento-panama si hay impacto regulatorio).
