# RFC-0003 - Librerias base del compilador ag-dsl

- Estado: aceptado
- Autor: Angel Nereira (BDFL inicial)
- Fecha de borrador: 2026-05-21
- Fecha de aceptacion: 2026-05-21
- Fase objetivo: Fase 3
- Modulos o crates afectados: `ag-dsl`, `ag-cli`
- RFC predecesora: RFC-0002 (Shield MVP)
- Periodo de comentarios: omitido por decision del BDFL en modo solo

## 1. Motivacion

La Hoja de Ruta v4.0 §3.1 fija como criterio de entrada de Fase 3 la
decision formal sobre las librerias base del compilador del DSL,
documentada en RFC. Esta RFC cumple ese requisito y fija las decisiones
tecnicas que gobiernan la implementacion de `ag-dsl` desde v0.1 hasta
v0.4, y el esqueleto de la infraestructura que soportara v0.5 a v1.0.

La decision es irreversible en el corto plazo: cambiar el stack del
compilador a mitad de la Fase 3 exigiria reescribir el crate completo
y desestabilizar las herramientas derivadas (LSP, VS Code plugin).

## 2. Problema

El compilador del DSL requiere cuatro componentes con interfaces bien
definidas:

1. **Lexer**: tokeniza el texto fuente `.ag` en tokens con informacion
   de span (linea/columna). Debe ser rapido y producir mensajes de
   error utiles cuando encuentra caracteres no esperados.

2. **Parser**: toma el stream de tokens y produce un AST. Debe
   recuperarse de errores (error recovery) para reportar multiples
   problemas en una sola pasada, no solo el primero. Los mensajes de
   error deben ser legibles por un desarrollador que no conoce el
   compilador.

3. **Codegen textual** (TypeScript, SQL, OpenAPI, Dart): produce archivos
   de texto a partir del AST validado. Debe ser auditabe, predecible y
   facil de mantener. Los artefactos generados no deben sorprender al
   usuario.

4. **Codegen Rust** (structs con serde, query builders): produce codigo
   Rust idiomatico. Las opciones son: generacion de texto plano
   (misma estrategia que el resto) o generacion via AST de Rust con
   `quote`/`proc-macro2` (codigo mas idiomatico, verificable a nivel
   de tokens).

Las alternativas son: construir cada componente desde cero, o usar
librerias establecidas del ecosistema Rust.

## 3. Alternativas consideradas

### 3.1 Lexer: logos vs nom vs manual

**logos**: genera el lexer a partir de un enum con derive macros
(`#[derive(Logos)]`). Cada variante se anota con `#[token(...)]`
para literales o `#[regex(...)]` para patrones. Produce automatas
finitos deterministicos (DFA) eficientes. Es la libreria mas popular
para lexers en el ecosistema Rust moderno.

Pros: codigo declarativo, rapido, mantenido activamente, usado en
produccion por proyectos como `rome`/`biome`. El enum de tokens es
directamente legible como documentacion de la gramatica.

Contras: genera codigo en tiempo de compilacion (macro expand), lo
que ralentiza el primer build.

**nom**: parser combinator que se puede usar como lexer.
Pros: una sola dependencia para lexer y parser.
Contras: diseñado para parsing binario/texto estructurado, no para
lenguajes de programacion con keywords, indentacion y operadores.
Los errores son verbosos y dificiles de humanizar.

**Manual**: escribir el lexer a mano como una maquina de estados.
Pros: control total. Contras: cientos de lineas de codigo repetitivo,
propenso a bugs en los casos borde (escapes en strings, comentarios
anidados). No se justifica cuando logos resuelve el problema.

Se elige **logos**.

### 3.2 Parser: chumsky vs lalrpop vs pest vs manual

**chumsky 0.9**: parser combinator con soporte nativo de error recovery.
`parse_recovery()` produce un AST parcial incluso en presencia de
errores, permitiendo reportar multiples problemas en una sola pasada.
La API de combinadores (`.then()`, `.or()`, `.repeated()`) es expresiva
y el codigo resultante lee como la gramatica EBNF.

Pros: error recovery nativo, mensajes de error con `.labelled()`,
codigo legible, sin generacion de codigo externa. La feature `label`
permite etiquetar combinadores para mensajes como "se esperaba un
identificador".

Contras: curva de aprendizaje pronunciada, especialmente para parsers
recursivos. La version 0.9.x (stable) tiene API diferente a la 0.10
(alpha). La eleccion 0.9.x se justifica por estabilidad.

**lalrpop**: generador de parsers LALR(1) con gramatica en archivo
separado. Pros: gramatica como fuente de verdad formal. Contras:
genera codigo en tiempo de compilacion, los mensajes de error de
LALR son poco amigables para el usuario, la gramatica del DSL es
ambigua a nivel LALR (anotaciones opcionales al final de una linea
sin separador). Descartado.

**pest**: PEG parser con gramatica en archivo `.pest`.
Pros: gramatica formal legible. Contras: no tiene error recovery,
el CST que produce requiere una segunda pasada para construir el AST.
Descartado.

**Manual**: parser recursivo descendente escrito a mano.
Pros: maximo control, potencialmente los mejores mensajes de error.
Contras: implementar error recovery manualmente es costoso; el
mantenimiento a largo plazo es significativamente mas alto que con
una libreria establecida. Se puede reconsiderar en Fase 5+ si chumsky
muestra limitaciones concretas.

Se elige **chumsky 0.9** con la feature `label`.

### 3.3 Codegen textual: askama vs minijinja vs format! macros

**askama**: templates Jinja2 compilados en tiempo de compilacion.
Los templates son archivos `.html`/`.txt`/`.rs` que el macro de
askama incrusta y verifica en el build. Los errores de template se
detectan al compilar ag-dsl, no en runtime.

Pros: templates auditables en archivos dedicados, verificacion en
compilacion, soporte de herencia y bloques.

Contras: añade complejidad al build (requiere archivos de template
en `templates/`), la API de datos es estaticamente tipada (struct
de contexto), la curva de aprendizaje es moderada.

**minijinja**: runtime Jinja2. Pros: templates dinamicos.
Contras: errores en runtime, sin verificacion estatica.

**format! macros**: generacion de texto con macros de la stdlib.
Pros: sin dependencias adicionales, el codigo generado esta inline
en el generador, maximo control sobre el formato. Contras: para
templates grandes se vuelve verboso y dificil de leer.

**Decision para v0.1**: se usan `format!` macros para los cuatro
generadores (Rust, SQL, TypeScript, OpenAPI). Los templates son
simples en v0.1 (structs basicos, tablas con columnas primitivas).
Cuando los generadores crezcan en v0.2-v0.4 (endpoints, validaciones,
relaciones), se evalua la migracion a `askama`. Este documento
registra esa transicion pendiente como deuda tecnica planificada,
no como deuda tecnica accidental.

Se decide **format! macros para v0.1**, con transicion a **askama**
a partir de v0.2 si la complejidad de los templates justifica el cambio.
Esta decision se revisara en RFC-0004.

### 3.4 Codegen Rust: format! texto vs quote + proc-macro2

**format! texto**: el generador Rust produce el codigo fuente como
un `String`. El codigo resultante es legible, pero no hay garantia
de que sea Rust valido hasta que el usuario lo compila.

**quote + proc-macro2**: la crate `quote!` permite construir
`TokenStream` de Rust (el AST de tokens que usa el compilador).
Combinada con `prettyplease` para formateo, produce codigo Rust
idiomatico verificable a nivel de tokens.

Pros de quote: el codigo generado puede verificarse programaticamente,
no hay riesgo de generar un identificador invalido o un tipo
desconocido.

Contras de quote: `quote` es una libreria de proc-macros; su uso
fuera de un contexto de proc-macro es posible pero inusual. La curva
de aprendizaje es mayor que `format!`.

**Decision para v0.1**: se usa `format!` texto para el generador Rust.
La prioridad en v0.1 es tener un pipeline funcional de extremo a
extremo. La migracion a `quote` + `prettyplease` se programa para
v0.3 cuando el generador Rust manejara validadores, query builders y
codigo mas complejo.

## 4. Diseno propuesto

### 4.1 Stack de dependencias de ag-dsl

| Crate | Version | Feature | Proposito |
| --- | --- | --- | --- |
| `logos` | 0.14 | default | Lexer generado por derive macro |
| `chumsky` | 0.9 | default | Parser combinator con error recovery (`.labelled()` disponible sin feature extra) |
| `thiserror` | 1 | workspace | Tipos de error derivados |
| `serde_json` | 1 | workspace | Serializacion OpenAPI v0.1 |

Dependencias postergadas (se añaden cuando se justifica su uso):

| Crate | Fase estimada | Motivo |
| --- | --- | --- |
| `askama` | v0.2 codegen | Templates para generadores complejos |
| `quote` + `proc-macro2` | v0.3 codegen | Generacion de TokenStream Rust |
| `prettyplease` | v0.3 codegen | Formateo del codigo Rust generado |
| `tower-lsp` | LSP | Servidor LSP para editores |

### 4.2 Arquitectura del pipeline de compilacion

```
texto .ag
    |
    v  [lexer.rs — logos]
Vec<(Token, Span)> + Vec<LexError>
    |
    v  [parser.rs — chumsky 0.9]
Option<Schema> + Vec<ParseError>
    |
    v  [semantic.rs]
Vec<SemanticError>
    |
    v  [diagnostics.rs — unificacion de errores]
Result<Schema, Vec<Diagnostic>>
    |
    v  [codegen/*.rs — format! macros v0.1]
GeneratedFiles { rust, sql, typescript, openapi }
```

### 4.3 Versiones del DSL cubiertas en Fase 3

Esta RFC autoriza la implementacion hasta DSL v0.4.
Cada subversion extiende la anterior sin romper compatibilidad:

- v0.1: modelos, tipos primitivos, anotaciones @primary/@unique/@auto
- v0.2: endpoints, request/response, errors
- v0.3: validaciones en campos (@min, @max, @email, @regex, @length)
- v0.4: relaciones entre modelos (1:1, 1:N, N:M)

Las versiones v0.5 a v1.0 requieren RFC separadas.

### 4.4 Modulo ag-dsl: estructura de archivos

```
crates/ag-dsl/
  Cargo.toml
  src/
    lib.rs            compilar() y generar() como API publica
    lexer.rs          Token (logos), tokenize()
    ast.rs            tipos AST: Schema, ModelDef, FieldDef, etc.
    parser.rs         schema_parser() (chumsky 0.9)
    semantic.rs       analyze(): validacion semantica
    diagnostics.rs    Diagnostic, Severity
    codegen/
      mod.rs          GeneratedFiles, generate()
      rust_gen.rs     -> src/models.rs (Rust structs)
      sql_gen.rs      -> migrations/NNNN_initial.sql
      ts_gen.rs       -> clients/typescript/types.ts
      openapi_gen.rs  -> openapi.json (OpenAPI 3.1)
  tests/
    v01_models.rs     tests de integracion DSL v0.1
```

### 4.5 API publica de ag-dsl

```rust
/// Compila un texto fuente DSL. Retorna el schema si no hay errores de
/// error severity. Los warnings se incluyen en el Ok si los hay.
pub fn compile(source: &str) -> Result<Schema, Vec<Diagnostic>>;

/// Genera artefactos a partir de un schema compilado.
pub fn generate(schema: &Schema) -> GeneratedFiles;

/// Coleccion de artefactos generados indexados por ruta relativa.
pub struct GeneratedFiles {
    pub files: std::collections::BTreeMap<PathBuf, String>,
}
```

## 5. Plan de implementacion

Los PRs implementan el pipeline capa por capa. Cada PR incluye tests,
fmt y clippy. Los PRs de Fase 3 van a la rama `fase-3`.

1. RFC-0003 + inicializacion ag-dsl: Cargo.toml, estructura modulos,
   dependencias. No hay codigo funcional todavia.
2. Lexer v0.1: Token enum con logos, funcion tokenize(), tests unitarios.
3. AST v0.1: tipos Schema, ModelDef, FieldDef, anotaciones.
4. Parser v0.1: schema_parser() con chumsky, parse model definitions.
5. Semantic analysis v0.1: resolucion de nombres, tipos validos.
6. Diagnostics: Diagnostic, Severity, formateo de mensajes.
7. Codegen Rust v0.1: generate structs con serde.
8. Codegen SQL v0.1: CREATE TABLE idempotente.
9. Codegen TypeScript v0.1: interfaces tipadas.
10. Codegen OpenAPI v0.1: components/schemas.
11. Tests de integracion v0.1: golden tests con schemas reales.
12. Comando ag generate en ag-cli.
13. DSL v0.2: endpoints, request/response, errors.
14. DSL v0.3: validaciones.
15. DSL v0.4: relaciones.
16. ag schema lint.
17. ag schema diff.
18. ag-lsp basico.
19. Fuzzing con cargo-fuzz.
20. Documentacion de referencia del DSL.

## 6. Riesgos

- **chumsky 0.9 vs 0.10**: la API 0.9 es estable pero 0.10 se acerca
  a stable. Si 0.10 se publica antes de que terminemos Fase 3, se
  evaluara la migracion en RFC-0004. No migramos durante Fase 3 para
  evitar inestabilidad.

- **logos y macros de expansion**: el build inicial puede ser lento en
  maquinas con poco hardware. Mitigacion: memoizacion de build artifacts
  con sccache en CI.

- **Mensajes de error del compilador**: la calidad de los mensajes es
  critica para DX. Mitigacion: tests dedicados que verifican el texto
  exacto del mensaje de error para los casos mas comunes.

- **Complejidad del codegen en v0.4**: las relaciones (1:N, N:M) generan
  JOINs, foreign keys y tipos adicionales. Puede justificar la migracion
  a askama antes del fin de Fase 3. Se abre RFC-0004 si ocurre.

## 7. Impacto

- Sobre el alcance: ninguno; dentro del perimetro de Fase 3.
- Sobre cronograma: 3 meses estimados, consistente con la Hoja de Ruta.
- Sobre dependencias: añade logos y chumsky al workspace.
- Sobre la API publica: define la primera API del crate ag-dsl.
- Sobre la documentacion: requiere actualizacion de
  `docs/architecture/07-anti-dsl.md` a medida que avanza.

## 8. Rollback

Si logos o chumsky demuestran limitaciones concretas durante la
implementacion (por ejemplo: rendimiento del lexer inaceptable, o
mensajes de error de chumsky insuficientes para DX objetivo):

1. Se abre RFC-0004 con la alternativa propuesta.
2. Se suspende la implementacion de ag-dsl en la rama `fase-3`.
3. Se revierte al skeleton Fase 0 de ag-dsl.
4. Se implementa la alternativa en una rama separada con PR de RFC-0004.

## 9. Decision

Aceptada por el BDFL inicial en modo solo.
Fecha de decision: 2026-05-21.

Stack definitivo para Fase 3:
- Lexer: logos 0.14
- Parser: chumsky 0.9 con feature `label`
- Codegen v0.1: format! macros
- Codegen v0.2+: transicion a askama (evaluada en RFC-0004)
- Codegen Rust v0.3+: transicion a quote + prettyplease (evaluada en RFC-0004)

## 10. Referencias

- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` §3.1 (criterios de entrada Fase 3)
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` §7 (arquitectura del DSL)
- `docs/roadmap/fase-03-anti-dsl-alpha.md`
- RFC-0002 (Shield MVP)
