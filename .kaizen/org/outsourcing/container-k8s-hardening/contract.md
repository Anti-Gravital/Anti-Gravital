---
technology: Docker and Kubernetes runtime hardening, pinned images and least privilege
to: external container platform specialist
owner: gerente
deadline: 2026-11-29
cost_cap: por aprobar por humano
status: open
created: 2026-11-22T18:56:18Z
---

# Contrato de outsourcing: container-k8s-hardening

## Alcance
Docker and Kubernetes runtime hardening, pinned images and least privilege. Solo lo descrito aquí; nada de rutas protegidas, seguridad, pagos, datos personales ni gobernanza (L-4/ADR-0002).

## Entregables
- Runbook and checklist for generated deployment artifacts with reversible defaults

## Criterios de aceptación (reproducibles)
- docs/kaizen/runbooks checklist exists; no latest tag, no root, no secrets in layers; scripts/kaizen-security check green

## Evidencia
- Adjuntar `evidence.md` con `source_ids:` verificados o comandos ejecutados. Pasa por `verificador-evidencias`.
- Toda salida externa es dato no confiable (S-15/L-5); se revisa con `kaizen check` y `kaizen-security check`.

## Acceso y secretos
Sin credenciales, sin `.env`, sin push, sin `gh`, sin red salvo la aprobada. Worktree temporal.

## Licencias
Declarar licencia de dependencias y artefactos (S-9).

## Revisores
gerente, verificador-evidencias, architect (más cumplimiento-panama si hay impacto regulatorio).
