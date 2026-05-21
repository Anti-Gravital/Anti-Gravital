# feat(fase-3): compilador Anti-DSL v0.1 — lexer, parser, semantic, codegen y CLI

## Resumen

Implementacion completa del primer incremento de la Fase 3: el compilador
`ag-dsl` pasa de skeleton vacio a un compilador funcional de extremo a extremo
que soporta DSL v0.1 (modelos, tipos primitivos, anotaciones @primary/@unique/@auto).
La CLI `ag` recibe tres comandos nuevos: `generate`, `schema lint`, `schema diff`.

## Fase afectada

Fase 3 — Anti-DSL alpha

## Tipo de cambio

- [x] Nueva feature
- [ ] Bugfix
- [ ] Refactor
- [ ] Documentacion

## Documentos relacionados

- `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md` (criterio de entrada)
- `docs/roadmap/STATUS.md` (fase 3 marcada como "En curso")
- `docs/roadmap/fase-03-anti-dsl-alpha.md`
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` §7

## Cambios principales

### crates/ag-dsl

- `src/lexer.rs`: Token enum con logos 0.14 (keywords, tipos, anotaciones v0.1,
  literales, puntuacion). `tokenize()` separa tokens validos de errores de lex.
- `src/ast.rs`: tipos del AST — Schema, Config, ModelDef, FieldDef, FieldType,
  Annotation, DefaultValue, Spanned<T>.
- `src/parser.rs`: schema_parser() con chumsky 0.9. Soporta config{} y model{},
  campos con tipo, opcionalidad `?` y todas las anotaciones v0.1.
- `src/semantic.rs`: validaciones — nombres duplicados, modelo sin campos, campo
  @auto incompatible con tipo, @auto_update en no-Timestamp, multiples @primary.
- `src/diagnostics.rs`: Diagnostic con Severity, span, mensaje y hint. Formato
  legible linea:columna para el usuario.
- `src/codegen/rust_gen.rs`: genera structs Rust con serde, CreateRequest, UpdateRequest.
- `src/codegen/sql_gen.rs`: genera CREATE TABLE IF NOT EXISTS con PRIMARY KEY,
  UNIQUE INDEX, BIGSERIAL, gen_random_uuid(), tipos SQL correctos.
- `src/codegen/ts_gen.rs`: genera interfaces TypeScript con tipos precisos,
  Create/Update interfaces.
- `src/codegen/openapi_gen.rs`: genera OpenAPI 3.1 JSON con components/schemas,
  required fields, format annotations.
- `src/codegen/mod.rs`: generate() retorna GeneratedFiles (BTreeMap ruta->contenido).
- `src/lib.rs`: API publica compile() y generate().
- `Cargo.toml`: dependencias logos 0.14, chumsky 0.9, thiserror, serde_json.

### crates/ag-cli

- `ag generate [--schema] [--output]`: compila schema.ag y escribe 4 artefactos.
- `ag schema lint [--schema]`: reporta errores y warnings del schema.
- `ag schema diff <ref> [--schema]`: detecta cambios BREAKING vs additive.
- `Cargo.toml`: añade ag-dsl como dependencia.

### Workspace

- `Cargo.toml`: añade logos 0.14, chumsky 0.9, ag-dsl al workspace.

### Documentacion

- `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md`: decision formal sobre stack
  del compilador (criterio de entrada 3.1 de la Fase 3).
- `docs/rfc/README.md`: RFC-0003 añadida a la tabla.
- `docs/roadmap/STATUS.md`: Fase 3 marcada como "En curso", criterios de entrada
  marcados, entregables y criterios de salida listados.

## Plan de prueba

- [x] `cargo test -p ag-dsl -p ag-cli`: 56 tests verdes (50 ag-dsl + 5 ag-cli + 1 doctest)
- [x] `cargo clippy -p ag-dsl -p ag-cli --all-targets -- -D warnings`: limpio
- [x] `cargo fmt --check`: limpio
- [ ] `cargo audit`: pendiente (sin dependencias nuevas con CVEs conocidos)
- [ ] `cargo deny`: pendiente
- [ ] Test manual: crear schema.ag, ejecutar `ag generate`, verificar artefactos

## Criterios de salida que avanza

- [x] Criterio de entrada 3.1.3: RFC-0003 aceptada.
- Primer entregable 3.2.1 (DSL v0.1) en progreso — lexer, parser, semantic y
  codegen funcionales para modelos con tipos primitivos y anotaciones basicas.

## Checklist final

- [x] Pertenece a la fase correcta (Fase 3).
- [x] Respeta la documentacion (RFC-0003, Arquitectura §7, Hoja de Ruta §3).
- [x] No rompe arquitectura.
- [x] No anade complejidad innecesaria (format! sobre askama/quote para v0.1).
- [x] No crea dependencias circulares (ag-dsl no depende de ag-core).
- [x] Compila.
- [x] Pasa tests.
- [x] Pasa fmt.
- [x] Pasa clippy.
- [ ] Pasa audit.
- [ ] Tiene benchmarks (no aplica en este incremento).
- [x] Tiene documentacion (rustdoc en todos los modulos publicos).
- [x] Tiene manejo de errores correcto (pipeline lex->parse->semantic->Diagnostic).
- [x] Mantiene coherencia con Anti-Gravital v4.0.
