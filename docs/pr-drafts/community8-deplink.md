# Resumen

Enlace verificable entre un proyecto generado y una URL: manifiesto de dominio canonico -> overlay de Kustomize -> host/TLS del Ingress, mas acceso local por host personalizado y un ejemplo offline de tres entornos.

## Fase afectada

Ola 8 (community), ciclo dominio-despliegue. Additivo; no cambia el contrato arquitectonico, el DSL ni los crates. Sin RFC nueva: consume el contrato normativo `docs/dns/ENVIRONMENT_RULES.md` (ER-01..ER-11) propuesto por envrules, sin editar sus archivos.

## Tipo de cambio

- [x] Artefactos de despliegue (nuevo `deploy/link/`)
- [x] Ejemplo offline (nuevo `examples/domain-link/`)
- [x] Documentacion de auditoria con evidencia real
- [x] `docs/INDEX.md` y `CHANGELOG.md`
- [ ] Codigo de producto, CLI, DSL, crates o CI

## Documentos relacionados

- `docs/dns/ENVIRONMENT_RULES.md` y `docs/dns/examples/domain-manifest.yaml` (contrato canonico de nombres y reglas ER-01..ER-11; solo lectura).
- `deploy/k8s/optional/ingress/` y `deploy/k8s/overlays/api/` (artefactos de la ola 7 que se parchean).
- `deploy/docker/compose/api.yml` y `deploy/docker/README.md` (camino Compose opcional).
- `templates/CATALOG.md` (familias de plantillas objetivo).
- `docs/audits/domain-link-validation.md` (evidencia de esta rama).

## Plan de prueba

Comandos ejecutados desde el worktree de la rama (rustc/cargo 1.99.1, Python 3.14.4 con PyYAML 6.0.3, kustomize v5.5.0, kubectl cliente v1.37.1, Docker Compose v5.5.1, curl 8.18.0):

```bash
python3 -m py_compile deploy/link/render_ingress.py
bash -n deploy/link/link.sh
bash -n examples/domain-link/mock-hosts.sh
python3 - <<'PY'
from pathlib import Path
import yaml
files = sorted(p for r in (Path('deploy/link'), Path('examples/domain-link'))
               for p in r.rglob('*') if p.suffix in {'.yaml', '.yml'})
for p in files:
    list(yaml.safe_load_all(p.read_text(encoding='utf-8')))
PY
for spec in "local api" "dev api" "staging api" "prod api" "prod metrics"; do
  set -- $spec
  bash deploy/link/link.sh --manifest examples/domain-link/domain.yaml \
    --env "$1" --service "$2" --out "/tmp/ag-domain-link/$1-$2"
done
diff -q <(kustomize build /tmp/ag-domain-link/prod-api) \
        <(kubectl kustomize /tmp/ag-domain-link/prod-api)
bash examples/domain-link/mock-hosts.sh
API_PORT=127.0.0.1:8080 POSTGRES_PASSWORD=local-only-placeholder \
  docker compose -f deploy/docker/compose/api.yml \
                 -f deploy/link/compose/local-host.yml config
git diff --check
```

Resultados reales: sintaxis OK; 2/2 YAML parseados; 5/5 planes renderizados con `HOST_OK`/`TLS_OK` o `INGRESS_OK`; salida de kustomize e kubectl byte-identica; `mock-hosts.sh` PASS=11 FAIL=0 SKIP=0; compose config confirma `host_ip: 127.0.0.1` y las variables PUBLIC_BASE_URL/AG_PUBLIC_HOST. Casos negativos rechazados con codigo 2: clave antigua `schemaVersion` (ER-01), `visibility` desconocida (ER-04), servicio private con `allow_sources` (ER-08), host fuera de allowlist (ER-10) y entrada `local` (ER-02a). SKIP honesto de kustomize verificado con PATH reducido.

SKIP explicitos: registro DNS real, apply en cluster, escritura de /etc/hosts y build/up de Docker (sin demonio). No se invento ningun resultado.

## Criterios de salida que avanza

- Linea de proyecto a URL reproducible offline para local, dev, staging y prod.
- Nombres y reglas alineados con el contrato canonico de envrules: `schema_version`, `prod`, `tls.required`, visibilidad explicita.
- Host nunca infiere visibilidad publica (ER-08); ausencia de visibilidad cierra a private (ER-04).
- Evidencia con versiones de herramientas, commit y fecha en `docs/audits/domain-link-validation.md`.

## Cierre de issues

Closes: none

## Checklist

- [x] Cambio minimo y additivo; sin editar archivos CLI compartidos ni directorios de otros roles
- [x] Documentacion actualizada en la misma rama
- [x] Verificacion real ejecutada y registrada, con SKIP honestos
- [x] Sin evidencia de herramientas IA, sin emojis, sin marcas prohibidas
- [x] Sin push ni merge; unicamente commits en `community8/deplink`
