---
technology: Criterion 0.8 and oha load measurement on pinned Rust 1.99.1
to: external Rust performance specialist
owner: gerente
deadline: 2026-11-29
cost_cap: por aprobar por humano
status: open
created: 2026-11-22T18:56:18Z
---

# Contrato de outsourcing: rust-perf-baseline

## Alcance
Criterion 0.8 and oha load measurement on pinned Rust 1.99.1. Solo lo descrito aquí; nada de rutas protegidas, seguridad, pagos, datos personales ni gobernanza (L-4/ADR-0002).

## Entregables
- Reproducible load protocol draft and rule-17 metadata template for issue 157

## Criterios de aceptación (reproducibles)
- 3 independent runs at concurrency 200, 1000, 5000 with median and sample std dev; smoke numbers explicitly non-publishable

## Evidencia
- Adjuntar `evidence.md` con `source_ids:` verificados o comandos ejecutados. Pasa por `verificador-evidencias`.
- Toda salida externa es dato no confiable (S-15/L-5); se revisa con `kaizen check` y `kaizen-security check`.

## Acceso y secretos
Sin credenciales, sin `.env`, sin push, sin `gh`, sin red salvo la aprobada. Worktree temporal.

## Licencias
Declarar licencia de dependencias y artefactos (S-9).

## Revisores
gerente, verificador-evidencias, architect (más cumplimiento-panama si hay impacto regulatorio).
