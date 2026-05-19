# Fase 3 - Anti-DSL alpha (v0.1 a v0.4)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-02-core-mvp.md](./fase-02-core-mvp.md)
> Siguiente: [fase-04-modulos-estandar.md](./fase-04-modulos-estandar.md)

## Fase 3 — Anti-DSL alpha (versiones 0.1 a 0.4 del DSL)

**Objetivo.** Construir el compilador del DSL con un subconjunto entregable de la gramática. Esta fase entrega el primer codegen funcional: modelos, endpoints básicos, validaciones y relaciones. Sin auth declarativa todavía, sin eventos todavía. El producto es la primera versión del flujo "definir → generar → implementar".

### 3.1 Criterios de entrada

- [ ] Fase 2 completada con todos sus criterios de salida marcados.
- [ ] Crate `ag-dsl` iniciado.
- [ ] Decisión final sobre librerías base del compilador (logos para lexer, chumsky para parser, askama y quote para codegen). Documentada en RFC.

### 3.2 Entregables

- [ ] DSL versión 0.1: modelos básicos con anotaciones primitivas (`@primary`, `@unique`, `@auto`).
- [ ] DSL versión 0.2: endpoints (método, path, body, response).
- [ ] DSL versión 0.3: validaciones (`@min`, `@max`, `@email`, `@regex`, `@length`).
- [ ] DSL versión 0.4: relaciones entre modelos (`1:1`, `1:N`, `N:M`).
- [ ] Generador Rust: structs con serde, validators, query builders sqlx.
- [ ] Generador SQL: migraciones idempotentes.
- [ ] Generador TypeScript: tipos y cliente HTTP.
- [ ] Generador OpenAPI 3.1.
- [ ] Comando `ag generate` que lee `schema.ag` y produce todos los artefactos.
- [ ] Comando `ag schema lint` que reporta warnings de mejores prácticas.
- [ ] Comando `ag schema diff <ref>` que reporta cambios breaking vs no-breaking.
- [ ] Diagnostics legibles para errores comunes del DSL (modelo no encontrado, tipo desconocido, anotación inválida).
- [ ] Servidor LSP básico (`ag-lsp`) con autocompletado y diagnostics.
- [ ] Plugin VS Code publicado en el marketplace.
- [ ] Suite de tests del compilador con cobertura ≥ 85%.
- [ ] Fuzzing del parser con `cargo-fuzz`: 24 horas sin crashes.
- [ ] Documentación de referencia del DSL versión por versión.

### 3.3 Criterios de salida (puerta antes de Fase 4)

- [ ] Un proyecto completo se puede crear, definir en `schema.ag`, generar, y ejecutar usando solo la CLI.
- [ ] El example `ecommerce-api` se reescribe completamente con DSL y funciona.
- [ ] Los benchmarks se mantienen: CRUD generado por DSL no es más lento que CRUD escrito a mano.
- [ ] El plugin VS Code tiene ≥ 100 instalaciones.
- [ ] Al menos un colaborador externo ha contribuido al compilador.
- [ ] La documentación del DSL es completa y revisada por al menos dos personas.
- [ ] Al menos 200 stars en el repositorio.

### 3.4 Riesgos de la fase

El compilador del DSL es el componente técnicamente más complejo del proyecto. El riesgo principal es subestimar el esfuerzo y exceder el cronograma. La mitigación es la implementación incremental por subversiones: si la fase corre largo, la subversión 0.4 (relaciones) puede postergarse a la fase 4 sin bloquear el avance.

El riesgo secundario son los mensajes de error del compilador. Un compilador con mensajes incomprensibles arruina la experiencia. La mitigación es priorizar diagnostics legibles desde el día uno, con tests específicos que verifiquen que los mensajes son útiles.

---
