# PR: Reestructurar el README raiz como ventana publica honesta y navegable del proyecto

## Resumen

Reescritura completa de `README.md` para comunicar con claridad que es
Anti-Gravital, que se puede y que no se puede hacer hoy, como esta
organizado el repositorio, la arquitectura, la vision y el modelo de
fases con su estado real. Solo documentacion; sin cambios de codigo.

## Fase afectada

Ninguna fase avanza ni cambia de estado. El cambio es de presentacion:
el README ahora refleja con mas claridad el estado ya registrado en
`docs/roadmap/STATUS.md` (fases 0-4.5 implementadas con puertas
abiertas, Fase 4.6 en curso, puerta pre-Fase 5 abierta).

## Tipo de cambio

- [x] Documentacion
- [ ] Codigo
- [ ] Infraestructura o CI
- [ ] RFC nueva o actualizacion de RFC
- [ ] ADR nuevo
- [ ] Seguridad

## Documentos relacionados

- ADR: `docs/adr/` 0008 (politica de idioma: ingles canonico primero,
  espejo en espanol despues) y 0009 (estado real, sin disfrazar el
  estado declarado).
- Maestros: no se modifican. El README enlaza y resume; la fuente de
  verdad sigue siendo `docs/master/` y `docs/roadmap/STATUS.md`.
- Regla de CLAUDE.md "Sincronizacion obligatoria del README": este PR
  no responde a un cambio de estado del codigo sino a una mejora de
  claridad; todas las afirmaciones se verificaron contra STATUS.md
  (2026-06-10) y el HEAD actual el 2026-06-12.

## Detalle del cambio

### `README.md`

- Cabecera con badges (CI, quality, licencia, MSRV, estado pre-release)
  y ancla bilingue `English | Espanol` (ADR-0008).
- Tabla "Project at a glance" con version publicada (ninguna), posicion
  actual de fases, estado de la puerta pre-Fase 5 y enlace al estado
  vivo.
- Secciones explicitas "What you can do today" (con evidencia por fase)
  y "What you cannot do yet" (deploy, crates.io, API estable,
  certificacion de produccion, crates placeholder).
- Seccion "What Anti-Gravital is not" (limites de alcance e
  interoperabilidad).
- Inicio rapido, extracto real del DSL desde
  `examples/ecommerce-api/schema.ag`, y la tabla CLI corregida (fila de
  `ag domains attach|...` tenia escapes rotos).
- Diagrama Mermaid de arquitectura (workflow schema-first + niveles de
  crates) y arbol comentado del repositorio.
- Seccion de vision y principios de ingenieria derivada de CLAUDE.md y
  los maestros.
- Modelo de fases explicado: fases bloqueantes vs aditivas, distincion
  implementada vs cerrada, excepcion RFC-0001, y como se abordaran las
  fases futuras (RFC primero, puertas, sin afirmaciones de GA).
- Tabla de fases 0-10 + 4.5/4.6 con estado y que mantiene cada puerta
  abierta, incluyendo los targets de rendimiento no alcanzados de la
  Fase 2 (honestidad de evidencia).
- Limitaciones conocidas y bloqueadores de release sin rodeos.
- Mapa de documentacion con orden de lectura para humanos y agentes
  automatizados (CLAUDE.md primero).
- Espejo completo en espanol tras la seccion inglesa; la inglesa es la
  canonica.

### `CHANGELOG.md`

- Entrada bajo `[Unreleased]` describiendo la reestructuracion.

## Plan de prueba

```sh
# Render local del Markdown (sin warnings de sintaxis Mermaid;
# diagrama validado con mermaid-cli/renderer antes del commit).
# Verificacion de enlaces relativos del README:
ls docs/roadmap/STATUS.md docs/audits/PRE_FASE5_RELEASE_GATE.md \
   docs/security/INSTALLATION_INTEGRITY.md docs/manual/02-primera-api.md \
   docs/INDEX.md docs/DEBT.md crates/ag-cli/README.md \
   docs/ag-domains/reference/cli.md examples/ecommerce-api/schema.ag

# El workspace no cambia; la suite sigue verde:
cargo fmt --all -- --check
cargo test --workspace
```

## Criterios de salida que avanza

Ninguna casilla de STATUS.md cambia. Mejora el entregable de Fase 0
"README.md bilingue" ya marcado, alineandolo con ADR-0008 (ingles
canonico primero) y ADR-0009 (estado real sin disfrazar).

## Checklist

- [x] Titulo de PR de 256 caracteres o menos.
- [x] Sin emojis en ningun archivo modificado.
- [x] Sin atribuciones de herramientas IA.
- [x] Afirmaciones de estado verificadas contra `docs/roadmap/STATUS.md`
  y el codigo en HEAD.
- [x] CHANGELOG.md actualizado bajo `[Unreleased]`.
- [x] CLAUDE.md respetado: alcance limitado a documentacion; sin
  cambios de codigo funcional; sin nuevas dependencias.
- [x] Descriptor pre-rellenado existe en `docs/pr-drafts/`.
