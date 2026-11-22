# Resumen

RFC-0033 domain-to-deploy loop contract and end-to-end documentation (wave 8, design only)

# Fase afectada

Fase 4.8, circuito dominio-despliegue. Solo diseno: no avanza ninguna casilla
de implementacion de `docs/roadmap/STATUS.md`.

# Tipo de cambio

- [ ] Codigo
- [ ] CLI
- [x] Documentacion (RFC, guia de extremo a extremo, indices, CHANGELOG)
- [ ] Infraestructura
- [ ] Tests

# Documentos relacionados

- `docs/rfc/RFC-0033-domain-to-deploy-loop.md` (propuesta, nueva RFC)
- `docs/dns/DOMAIN_LOOP.md` (diagrama Mermaid y ejemplo)
- `docs/INDEX.md`, `docs/rfc/README.md`, `CHANGELOG.md`
- Predecesoras: RFC-0007, RFC-0011, RFC-0016, RFC-0019, RFC-0020, RFC-0021,
  RFC-0024. ADR-0008, ADR-0009, ADR-0011.

# Plan de prueba

- `git diff --check`
- `grep -n "RFC-0033" docs/INDEX.md docs/rfc/README.md CHANGELOG.md`
- `grep -n "^flowchart TD" docs/dns/DOMAIN_LOOP.md`
- Revision manual de los campos del manifiesto y del flujo plan/apply frente a
  `crates/ag-domains/src/{api,attachment,instructions,propagation}.rs`.
- No se ejecutan comandos de proveedor, cluster ni red: el cambio es diseno.

# Criterios de salida que avanza

- Contrato publicado para que dnsapi, deplink y envrules implementen sin
  improvisar: campos `[domain]`, resolucion por entorno, visibilidad, plan JSON
  versionado, fragmentos Ingress/compose y reglas de seguridad.
- Documento de extremo a extremo con ejemplo marcado como requiriendo proveedor
  real, sin hacer promesas de ejecucion local.

## Cierre de issues

Closes: none

# Checklist

- [x] Pertenece a la fase correcta (diseno previo a la implementacion)
- [x] Respeta la documentacion y no introduce features fuera de alcance
- [x] No rompe arquitectura ni anade dependencias o crates
- [x] No anade complejidad innecesaria ni credenciales
- [x] No crea dependencias circulares
- [x] No requiere compilacion (solo documentos Markdown)
- [x] Tiene documentacion (es el propio cambio)
- [x] Sin emojis, sin evidencia de herramientas IA, titulo <= 256 caracteres
