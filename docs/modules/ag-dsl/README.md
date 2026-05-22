# ag-dsl

> Capitulo de arquitectura: `docs/architecture/07-anti-dsl.md`.
> Referencia del DSL: `docs/dsl/referencia-v01-v04.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 3 a Fase 10.

## Dominio

Lexer, parser, AST, analisis semantico y codegen del Anti-DSL.
Pipeline: texto `.ag` → lexer (logos 0.14) → parser (chumsky 0.9) →
AST → analisis semantico → diagnostics → codegen multi-target.

## Dependencias internas permitidas

No depende de ningun crate Anti-Gravital.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md`.

## Estado

Fase 3 completado. Compilador operativo con DSL v0.1 a v0.4.
119 tests verdes. Cobertura lineas 95.26%. API publica estable.

## Versiones implementadas

| Version | Capacidad | Commit |
|---------|-----------|--------|
| v0.1 | Modelos, tipos primitivos, @primary/@unique/@auto/@auto_update/@default | 5aa442d |
| v0.2 | Endpoints, HTTP methods, PathLit, handlers/router/client codegen | 7d41930 |
| v0.3 | @min, @max, @email, @regex, @length; SQL CHECK; Rust validate(); OpenAPI constraints | 47ae623 |
| v0.4 | @references/@relation, relaciones 1:1/1:N/N:M, FOREIGN KEY SQL, Option<M>/Vec<M> Rust, $ref OpenAPI | 9a541cd..451f740 |

## API publica

```rust
// Compila el schema. Retorna error si hay diagnostics de severidad Error.
pub fn compile(source: &str) -> Result<Schema, Vec<Diagnostic>>;

// Genera todos los artefactos para un schema compilado.
pub fn generate(schema: &Schema) -> GeneratedFiles;

// Retorna TODOS los diagnostics (errores y warnings) sin bloquear en errores.
// Usar para LSP y ag schema lint.
pub fn lint(source: &str) -> Vec<Diagnostic>;
```

## Artefactos generados por `generate()`

| Artefacto | Descripcion |
|-----------|-------------|
| `migrations/NNNN_init.sql` | CREATE TABLE, UNIQUE INDEX, FOREIGN KEY (ALTER TABLE) |
| `src/models.rs` | Structs Rust con serde, Option<M>/Vec<M> para relaciones |
| `src/types.rs` | Request/Response/Error structs con validate() |
| `src/handlers.rs` | Handlers Axum por endpoint |
| `src/router.rs` | Router Axum completo |
| `src/client.rs` | Cliente HTTP tipado |
| `openapi.json` | OpenAPI 3.1 con schemas, paths, $ref para relaciones |
| `types.ts` | Interfaces TypeScript con tipos opcionales para relaciones |

## Generadores de codigo

- `codegen/sql_gen.rs` — SQL PostgreSQL con FOREIGN KEY via ALTER TABLE
- `codegen/rust_gen.rs` — Rust + serde; Option<M>/Vec<M> para relaciones virtuales
- `codegen/ts_gen.rs` — TypeScript; tipos opcionales para relaciones
- `codegen/openapi_gen.rs` — OpenAPI 3.1; $ref y array+$ref para relaciones

## Validaciones semanticas implementadas

### v0.1–v0.3
- Modelos duplicados, campos duplicados, modelo sin campos
- @auto incompatible con tipos no-UUID/Int
- @auto_update solo en Timestamp
- Multiples @primary por modelo
- @min <= @max, @length > 0
- @min/@max/@email/@regex solo en tipos compatibles

### v0.4 (relaciones)
- Modelo referenciado en @references existe en el schema
- El campo referenciado es @primary (warning si no lo es)
- ModelRef/ModelRefList requieren @relation
- El campo FK indicado en @relation existe y tiene @references
- Referencia circular entre modelos (A -> B -> A via FK reales)

## Fuzzing

Harness cargo-fuzz con 3 targets en `fuzz/`:
- `fuzz_lexer` — lexer no panics con UTF-8 arbitrario
- `fuzz_parser` — parser no panics con entrada arbitraria
- `fuzz_compile` — pipeline completo sin panics

Crash encontrado y corregido en Fase 3: `IntLit` usaba `.unwrap()` en
parse de enteros; enteros > i64::MAX causaban panic. Corregido con
`.ok()` (commit ff85c6f). CI smoke test: 60s por target en cada PR.

## Tests

119 tests distribuidos en los modulos:
- `lexer`: 14 tests (tokens v0.1–v0.4)
- `parser`: 28 tests (modelos, endpoints, anotaciones, relaciones)
- `semantic`: 21 tests (validaciones v0.1–v0.4)
- `codegen/sql_gen`: 11 tests
- `codegen/rust_gen`: 14 tests
- `codegen/ts_gen`: 8 tests
- `codegen/openapi_gen`: 6 tests
- `lib` (integracion): 11 tests incluyendo regression fuzz crash
