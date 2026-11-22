# Descriptor de PR: community7/k8s

## Resumen

Add optional Kubernetes manifests for the five template families without making a cluster a requirement

## Fase afectada

Ola 7, diseno de despliegue. Additivo sobre las fases templates existentes; no
abre ni cierra una fase de la Hoja de Ruta, no altera targets de rendimiento.

## Tipo de cambio

- [x] Documentacion y artefactos de despliegue
- [x] Nueva funcionalidad opcional (manifiestos, sin codigo Rust)
- [ ] Cambio de codigo Rust
- [ ] Cambio de CLI, DSL, crates, CI o arquitectura

## Documentos relacionados

- `CLAUDE.md` reglas 12, 13, 16, 19, 26, 36 y 37.
- `docs/adr/0009-gobernanza-correctiva.md` (regla 2: integracion externa
  opcional con modo nativo por defecto).
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
  (modelo plan/apply de la CLI; no se implementa ni se requiere aqui).
- `docs/INDEX.md` y `CHANGELOG.md` actualizados en esta misma rama.

## Plan de prueba

Comandos ejecutados desde la raiz del repositorio, sin cluster:

```
kustomize version
kubectl version --client
python3 -c 'import yaml; print("PyYAML " + yaml.__version__)'
kustomize build deploy/k8s
kustomize build deploy/k8s/overlays/web
kustomize build deploy/k8s/overlays/api
kustomize build deploy/k8s/overlays/realtime
kustomize build deploy/k8s/overlays/infrastructure
kustomize build deploy/k8s/overlays/ai
kustomize build deploy/k8s/optional/hpa
kustomize build deploy/k8s/optional/ingress
kustomize build deploy/k8s/examples/api-with-addons
kubectl kustomize deploy/k8s/overlays/api
python3 - <parseo PyYAML y aserciones de hardening>
grep -rniE '(^|[[:space:]])(password|token|api[_-]?key|secretKey|clientSecret)[[:space:]]*:' deploy/k8s --include='*.yaml'
```

Resultado exacto, incluidos los SKIPs (kubeconform ausente, sin
`kubectl apply`), en `deploy/k8s/EVIDENCE.md`.

## Criterios de salida que avanza

- Empaquetado opcional para las cinco familias de plantillas, sin exigir
  cluster: `cargo run` y `ag dev` siguen siendo el camino por defecto.
- Manifiestos con probes, limites, endurecimiento e imagen configurable.
- Patron de secretos documentado sin valores enlinea.
- HPA e Ingress disponibles pero desactivados por defecto.
- Evidencia de verificacion honesta, sin despliegue simulado.

## Cierre de issues

Closes: none

## Checklist

- [x] Compila (no aplica: sin codigo Rust; todos los YAML renderizan)
- [x] Pasan los tests (no aplica: sin tests Rust; verificacion en EVIDENCE.md)
- [x] Pasa fmt (no aplica: sin codigo Rust)
- [x] Pasa clippy (no aplica: sin codigo Rust)
- [x] Documentacion actualizada (README, INDEX, CHANGELOG, SECRETS, EVIDENCE)
- [x] Sin evidencia de herramientas IA, sin emojis, sin prefijos prohibidos
