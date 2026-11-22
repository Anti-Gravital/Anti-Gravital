# Resumen

Validador de artefactos de despliegue con evidencia real: YAML, Dockerfiles y comportamiento de `ag deploy`.

## Fase afectada

Ola 7 (community), diseno de despliegue. Additivo; no cambia el contrato arquitectonico ni el DSL. Sin RFC nueva: es tooling de verificacion, no una decision de arquitectura.

## Tipo de cambio

- [x] Herramienta de verificacion (nuevo script)
- [x] Documentacion de auditoria con evidencia real
- [x] CHANGELOG
- [ ] Codigo de producto
- [ ] Cambio de CLI, DSL, crates o CI

## Documentos relacionados

- `docs/audits/deployment-validation.md` (evidencia de las ejecuciones reales)
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md` (modelo plan/apply, seccion 4.3)
- `docs/rfc/RFC-0033-local-deploy-artifact-rendering.md` (contrato del role deploy, en otra rama)
- Reglas del repositorio de integracion con herramientas dominantes, benchmarks honestos y testing obligatorio

## Plan de prueba

Comandos exactos ejecutados:

```bash
cd /home/angel-nereira/Github-local-angelnereira/worktrees7/deployqa
bash -n scripts/validate-deploy.sh
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 ./scripts/validate-deploy.sh
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
DEPLOY_ROOT=/tmp/ag-deploy-merged/deploy ./scripts/validate-deploy.sh
git diff --check
```

Ejecucion A (sin artefactos): PASS=0 FAIL=8 SKIP=3, salida 1. Ejecucion B (artefactos en curso, CLI aun no compilada): PASS=5 FAIL=3 SKIP=3, salida 1. Tras compilar `ag deploy` con `cargo build --locked --offline`, ejecucion C (definitiva): PASS=9 FAIL=0 SKIP=3, salida 0. Las ejecuciones A y B se conservan como historial honesto; los 3 SKIP finales requieren daemon Docker, cluster Kubernetes y kubeconform, no disponibles aqui.

## Criterios de salida que avanza

- Verificacion reproducible de artefactos de despliegue con parser YAML real (PyYAML), sin inventar resultados.
- Distincion explicita entre artefacto ausente (FAIL) y herramienta no disponible (SKIP).
- Evidencia auditable con host, versiones, commit y fecha para la ola 7.

## Cierre de issues

Closes: none

## Checklist

- [x] Cambio minimo y additivo
- [x] Documentacion actualizada en la misma PR
- [x] Verificacion real ejecutada y registrada
- [x] Sin evidencia de herramientas IA, sin emojis, sin marcas prohibidas
- [x] Sin push ni merge; unicamente commits en `community7/deployqa`
