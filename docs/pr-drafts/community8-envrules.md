# Resumen

Reglas de entorno y visibilidad deterministas para el circuito dominio-despliegue: modelo local/dev/staging/prod, private/vpn-only/public, deny-by-default y manifiesto de ejemplo validado.

## Fase afectada

Ola 8 (community), circuito dominio-despliegue. Cambio aditivo de diseno y documentacion; no altera arquitectura, DSL, crates ni CLI.

## Tipo de cambio

- [x] Documentacion normativa de diseno (`docs/dns/ENVIRONMENT_RULES.md`)
- [x] Manifiesto de ejemplo machine-readable (`docs/dns/examples/domain-manifest.yaml`)
- [x] Indice de documentacion y CHANGELOG
- [x] Evidencia de verificacion con parser real
- [ ] Codigo de producto
- [ ] Cambio de CLI, DSL, crates o CI

## Documentos relacionados

- `docs/dns/ENVIRONMENT_RULES.md` (contrato, reglas ER-01 a ER-11)
- `docs/dns/examples/domain-manifest.yaml` (ejemplo con tres entornos)
- `docs/rfc/RFC-0020-environment-and-secrets.md` (precedencia, sin selector de entorno en runtime)
- `docs/templates/TEMPLATE_DESIGN.md` (secciones 2, 3, 4, 6, 7 y 10)
- `RFC-0033` y `docs/dns/DOMAIN_LOOP.md` (en preparacion por el role dnsarch; no se inventa su contenido)
- Reglas de CLAUDE.md 12, 15, 16, 19, 22, 26 y 31; ADR-0008, ADR-0009, ADR-0011

## Plan de prueba

Comandos ejecutados en el worktree de la rama (sin push ni merge):

```bash
python3 --version
python3 -c "import yaml; print(yaml.__version__)"
python3 -c "import yaml; d=yaml.safe_load(open('docs/dns/examples/domain-manifest.yaml')); print(list(d), list(d['environments']))"
python3 /tmp/opencode/validate-env-manifest.py
git diff --check
```

Resultados reales: Python 3.14.4, PyYAML 6.0.3. El parser leyó las claves
`['schema_version', 'project', 'default_visibility', 'environments']` y los
entornos `['dev', 'staging', 'prod']`. El validador de una sola ejecucion
(comprobaciones ER-01 a ER-10, claves estrictas, nombres unicos, TLS en hosts
ligados y ausencia de secretos) dio **77 PASS, 0 FAIL**, salida 0. ER-11 es un
contrato de comando y se comprobo por inspeccion, no por parsing.

La primera ejecucion del validador dio 5 FAIL por usar comparacion exacta de
hosts en lugar de la coincidencia con wildcard definida en ER-10; tras corregir
el validador, los 77 checks pasan. El manifiesto no requirio cambios. El
validador es una comprobacion puntual fuera del repositorio: el script de
validacion del circuito pertenece al role dnsqa.

No se creo ningun registro DNS, certificado, contenedor ni despliegue en cluster.

## Criterios de salida que avanza

- Vocabulario unico de entornos y visibilidad para que el circuito domino-despliegue no invente valores por objetivo de despliegue.
- Reglas verificables con identificadores estables (ER-01 a ER-11) consumibles por dnsapi y dnsqa.
- Deny-by-default explicito: sin visibilidad declarada el servicio es privado y no se expone.
- Sin variable de entorno selector inventada; la seleccion es argumento explicito `--env` o artefacto de despliegue.
- Manifestacion de la compatibilidad con herramientas dominantes (Ingress, compose) sin exigir proveedor ni cluster para el camino local.

## Cierre de issues

Closes: none

## Checklist

- [x] Cambio minimo y aditivo; sin codigo de CLI ni archivos compartidos de otros roles
- [x] Documentacion tecnica en ingles canonico (ADR-0008) e indice/CHANGELOG actualizados en el mismo cambio
- [x] Verificacion real ejecutada y registrada, sin resultados inventados
- [x] Sin evidencia de herramientas IA, sin emojis, sin nombres de marcas prohibidas
- [x] Sin push ni merge; unicamente commits en `community8/envrules`
