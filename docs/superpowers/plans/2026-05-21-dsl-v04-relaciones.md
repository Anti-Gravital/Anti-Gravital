# DSL v0.4 — Relaciones entre modelos: Plan de Implementacion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extender el compilador Anti-DSL con soporte de relaciones 1:1, 1:N y N:M usando `@references`/`@relation`, generando SQL FOREIGN KEY, Rust Option<M>/Vec<M>, TypeScript y OpenAPI $ref en los 4 generadores existentes.

**Architecture:** Campo FK explicito con `@references(Modelo.campo)` genera columna SQL real; campo virtual con tipo `ModelRef`/`ModelRefList` y `@relation(path)` es omitido por SQL pero incluido por Rust/TS/OpenAPI como tipo anidado. La cardinalidad se infiere del tipo: `Modelo` = N:1, `Modelo[]` = 1:N, `@unique + @references` = 1:1.

**Tech Stack:** logos 0.14 (lexer), chumsky 0.9 (parser), serde_json (OpenAPI), Rust 1.82+. Sin dependencias nuevas.

---

## Mapa de archivos

| Archivo | Accion | Que cambia |
|---|---|---|
| `crates/ag-dsl/src/ast.rs` | Modificar | FieldType + Annotation nuevas variantes; FieldDef.virtual_field |
| `crates/ag-dsl/src/lexer.rs` | Modificar | Tokens AtReferences, AtRelation, Dot |
| `crates/ag-dsl/src/parser.rs` | Modificar | Tipos ModelRef/ModelRefList, anotaciones @references/@relation |
| `crates/ag-dsl/src/semantic.rs` | Modificar | 5 funciones de validacion nuevas |
| `crates/ag-dsl/src/codegen/sql_gen.rs` | Modificar | Omitir virtuales, generar FOREIGN KEY |
| `crates/ag-dsl/src/codegen/rust_gen.rs` | Modificar | Option<M> y Vec<M> para campos virtuales |
| `crates/ag-dsl/src/codegen/ts_gen.rs` | Modificar | Tipos opcionales y arrays para relaciones |
| `crates/ag-dsl/src/codegen/openapi_gen.rs` | Modificar | $ref y array+$ref para relaciones |

---

## Task 1: AST — nuevas variantes y campo virtual_field

**Files:**
- Modify: `crates/ag-dsl/src/ast.rs`
- Modify: `crates/ag-dsl/src/parser.rs` (fix de compilacion por nuevo campo)

- [ ] **Step 1.1: Escribir tests que fallen**

Al final del bloque `#[cfg(test)]` en `crates/ag-dsl/src/ast.rs`, agregar:

```rust
#[test]
fn model_ref_rust_type_single() {
    let ty = FieldType::ModelRef("User".to_owned());
    assert_eq!(ty.rust_type(false), "User");
    assert_eq!(ty.rust_type(true), "Option<User>");
}

#[test]
fn model_ref_list_rust_type() {
    let ty = FieldType::ModelRefList("Post".to_owned());
    assert_eq!(ty.rust_type(false), "Post");
}

#[test]
fn model_ref_sql_type_empty() {
    let ty = FieldType::ModelRef("User".to_owned());
    assert_eq!(ty.sql_type(), "");
}

#[test]
fn model_ref_ts_type() {
    let ty = FieldType::ModelRef("User".to_owned());
    assert_eq!(ty.ts_type(), "object");
}
```

- [ ] **Step 1.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- ast::
```

Resultado esperado: FAIL — las variantes no existen aun.

- [ ] **Step 1.3: Agregar variantes a FieldType**

En `crates/ag-dsl/src/ast.rs`, reemplazar el enum `FieldType` completo:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Uuid,
    String,
    Int,
    Float,
    Bool,
    Timestamp,
    Decimal,
    /// Referencia a otro modelo — campo virtual N:1 o 1:1, sin columna SQL.
    ModelRef(std::string::String),
    /// Lista de referencias — campo virtual 1:N, sin columna SQL.
    ModelRefList(std::string::String),
}
```

- [ ] **Step 1.4: Extender los metodos de FieldType**

Reemplazar el bloque `impl FieldType` completo:

```rust
impl FieldType {
    pub fn rust_type(&self, optional: bool) -> std::string::String {
        match self {
            FieldType::ModelRef(m) | FieldType::ModelRefList(m) => {
                return if optional {
                    format!("Option<{m}>")
                } else {
                    m.clone()
                };
            }
            _ => {}
        }
        let base = match self {
            FieldType::Uuid => "uuid::Uuid",
            FieldType::String => "String",
            FieldType::Int => "i64",
            FieldType::Float => "f64",
            FieldType::Bool => "bool",
            FieldType::Timestamp => "chrono::DateTime<chrono::Utc>",
            FieldType::Decimal => "rust_decimal::Decimal",
            FieldType::ModelRef(_) | FieldType::ModelRefList(_) => unreachable!(),
        };
        if optional {
            format!("Option<{base}>")
        } else {
            base.to_owned()
        }
    }

    pub fn sql_type(&self) -> &'static str {
        match self {
            FieldType::Uuid => "UUID",
            FieldType::String => "TEXT",
            FieldType::Int => "BIGINT",
            FieldType::Float => "DOUBLE PRECISION",
            FieldType::Bool => "BOOLEAN",
            FieldType::Timestamp => "TIMESTAMPTZ",
            FieldType::Decimal => "NUMERIC",
            FieldType::ModelRef(_) | FieldType::ModelRefList(_) => "",
        }
    }

    pub fn ts_type(&self) -> &'static str {
        match self {
            FieldType::Uuid => "string",
            FieldType::String => "string",
            FieldType::Int => "number",
            FieldType::Float => "number",
            FieldType::Bool => "boolean",
            FieldType::Timestamp => "string",
            FieldType::Decimal => "string",
            FieldType::ModelRef(_) | FieldType::ModelRefList(_) => "object",
        }
    }

    pub fn openapi_type(&self) -> (&'static str, Option<&'static str>) {
        match self {
            FieldType::Uuid => ("string", Some("uuid")),
            FieldType::String => ("string", None),
            FieldType::Int => ("integer", Some("int64")),
            FieldType::Float => ("number", Some("double")),
            FieldType::Bool => ("boolean", None),
            FieldType::Timestamp => ("string", Some("date-time")),
            FieldType::Decimal => ("string", Some("decimal")),
            FieldType::ModelRef(_) | FieldType::ModelRefList(_) => ("object", None),
        }
    }
}
```

- [ ] **Step 1.5: Agregar variantes a Annotation**

En `crates/ag-dsl/src/ast.rs`, reemplazar el enum `Annotation` completo:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    Primary,
    Unique,
    Auto,
    AutoUpdate,
    Default(DefaultValue),
    Min(i64),
    Max(i64),
    Email,
    Regex(std::string::String),
    Length(i64),
    /// `@references(Modelo.campo)` — clave foranea con columna SQL real.
    References {
        model: std::string::String,
        field: std::string::String,
    },
    /// `@relation(campo)` o `@relation(modelo.campo)` — campo virtual.
    Relation {
        path: std::string::String,
    },
}
```

- [ ] **Step 1.6: Agregar virtual_field a FieldDef**

Reemplazar la definicion de `FieldDef`:

```rust
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Spanned<String>,
    pub ty: Spanned<FieldType>,
    pub optional: bool,
    pub annotations: Vec<Spanned<Annotation>>,
    /// true cuando el campo es ModelRef/ModelRefList con @relation.
    /// SQL codegen lo omite; Rust/TS/OpenAPI lo incluyen como tipo anidado.
    pub virtual_field: bool,
}
```

- [ ] **Step 1.7: Corregir error de compilacion en parser.rs**

En `crates/ag-dsl/src/parser.rs`, en `field_parser()`, reemplazar el `.map(...)` final (linea ~166-175):

```rust
field_name
    .then(field_type)
    .then(optional)
    .then(annotations)
    .map(|(((name, ty), optional), annotations)| {
        let virtual_field = matches!(
            ty.value,
            FieldType::ModelRef(_) | FieldType::ModelRefList(_)
        );
        FieldDef {
            name,
            ty,
            optional,
            annotations,
            virtual_field,
        }
    })
```

- [ ] **Step 1.8: Verificar que compila y los tests pasan**

```
cargo test -p ag-dsl
```

Resultado esperado: Los 73 tests existentes pasan + los 4 nuevos de ast. Total: 77 tests verdes.

- [ ] **Step 1.9: Commit**

```bash
git add crates/ag-dsl/src/ast.rs crates/ag-dsl/src/parser.rs
git commit -m "feat(dsl): v0.4 — AST FieldType ModelRef/ModelRefList, Annotation References/Relation, FieldDef.virtual_field"
```

---

## Task 2: Lexer — tokens @references, @relation, Dot

**Files:**
- Modify: `crates/ag-dsl/src/lexer.rs`

- [ ] **Step 2.1: Escribir tests que fallen**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/lexer.rs`, agregar:

```rust
#[test]
fn v04_relation_tokens() {
    let toks = lex("@references @relation");
    assert_eq!(toks, vec![Token::AtReferences, Token::AtRelation]);
}

#[test]
fn v04_dot_token() {
    let toks = lex("User.id");
    assert_eq!(
        toks,
        vec![
            Token::Ident("User".to_owned()),
            Token::Dot,
            Token::Ident("id".to_owned()),
        ]
    );
}

#[test]
fn v04_list_type_brackets() {
    let toks = lex("Post[]");
    assert_eq!(
        toks,
        vec![
            Token::Ident("Post".to_owned()),
            Token::LBracket,
            Token::RBracket,
        ]
    );
}
```

- [ ] **Step 2.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- lexer::tests::v04
```

Resultado esperado: FAIL — los tokens no existen.

- [ ] **Step 2.3: Agregar los 3 tokens**

En `crates/ag-dsl/src/lexer.rs`, despues del token `AtLength` (linea ~119), agregar:

```rust
// ---- Anotaciones DSL v0.4 — relaciones ----
/// `@references` — clave foranea hacia otro modelo.
#[token("@references")]
AtReferences,
/// `@relation` — campo virtual de relacion.
#[token("@relation")]
AtRelation,
```

Despues del token `Question` (linea ~170), agregar:

```rust
/// Punto: separador en `@references(Modelo.campo)` y `@relation(modelo.campo)`.
#[token(".")]
Dot,
```

- [ ] **Step 2.4: Ejecutar tests del lexer**

```
cargo test -p ag-dsl -- lexer::tests
```

Resultado esperado: Todos los tests del lexer pasan incluyendo los 3 nuevos.

- [ ] **Step 2.5: Commit**

```bash
git add crates/ag-dsl/src/lexer.rs
git commit -m "feat(dsl): v0.4 — tokens AtReferences, AtRelation, Dot"
```

---

## Task 3: Parser — tipos ModelRef/ModelRefList, anotaciones @references/@relation

**Files:**
- Modify: `crates/ag-dsl/src/parser.rs`

- [ ] **Step 3.1: Escribir tests que fallen**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/parser.rs`, agregar:

```rust
#[test]
fn v04_parses_references_annotation() {
    let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
}
"#;
    let (ast, diags) = parse(src);
    assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
    let field = &ast.unwrap().models[0].fields[1];
    let has_ref = field.annotations.iter().any(|a| {
        matches!(&a.value,
            Annotation::References { model, field }
            if model == "User" && field == "id")
    });
    assert!(has_ref, "should have @references(User.id)");
    assert!(!field.virtual_field, "FK field should not be virtual");
}

#[test]
fn v04_parses_relation_single() {
    let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#;
    let (ast, diags) = parse(src);
    assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
    let field = &ast.unwrap().models[0].fields[2];
    assert!(
        matches!(&field.ty.value, FieldType::ModelRef(m) if m == "User"),
        "type should be ModelRef(User)"
    );
    assert!(field.virtual_field, "relation field should be virtual");
    let has_rel = field.annotations.iter().any(|a| {
        matches!(&a.value, Annotation::Relation { path } if path == "author_id")
    });
    assert!(has_rel, "should have @relation(author_id)");
}

#[test]
fn v04_parses_relation_list() {
    let src = r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
"#;
    let (ast, diags) = parse(src);
    assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
    let field = &ast.unwrap().models[0].fields[1];
    assert!(
        matches!(&field.ty.value, FieldType::ModelRefList(m) if m == "Post"),
        "type should be ModelRefList(Post)"
    );
    assert!(field.virtual_field, "list relation should be virtual");
    let has_rel = field.annotations.iter().any(|a| {
        matches!(&a.value, Annotation::Relation { path } if path == "post.author_id")
    });
    assert!(has_rel, "should have @relation(post.author_id)");
}
```

- [ ] **Step 3.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- parser::tests::v04
```

Resultado esperado: FAIL — el parser no maneja los nuevos tokens.

- [ ] **Step 3.3: Extender field_type en field_parser()**

En `crates/ag-dsl/src/parser.rs`, en `field_parser()`, reemplazar el bloque `let field_type = choice((...))`:

```rust
let field_type = choice((
    just(Token::TyUuid).to(FieldType::Uuid),
    just(Token::TyString).to(FieldType::String),
    just(Token::TyInt).to(FieldType::Int),
    just(Token::TyFloat).to(FieldType::Float),
    just(Token::TyBool).to(FieldType::Bool),
    just(Token::TyTimestamp).to(FieldType::Timestamp),
    just(Token::TyDecimal).to(FieldType::Decimal),
    // v0.4: ModelRefList debe ir antes que ModelRef (patron mas especifico primero)
    select! { Token::Ident(s) => s }
        .then_ignore(just(Token::LBracket))
        .then_ignore(just(Token::RBracket))
        .map(FieldType::ModelRefList),
    select! { Token::Ident(s) => s }.map(FieldType::ModelRef),
))
.map_with_span(|t, span: Span| Spanned::new(t, span))
.labelled("tipo de campo");
```

- [ ] **Step 3.4: Agregar @references y @relation a annotation_parser()**

En `crates/ag-dsl/src/parser.rs`, en `annotation_parser()`, agregar despues de `at_regex` y antes del `choice((...))` final:

```rust
// v0.4 — @references(Modelo.campo)
let at_references = just(Token::AtReferences)
    .ignore_then(
        select! { Token::Ident(s) => s }
            .then_ignore(just(Token::Dot))
            .then(select! { Token::Ident(s) => s })
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .labelled("referencia Modelo.campo"),
    )
    .map(|(model, field)| Annotation::References { model, field });

// v0.4 — @relation(campo) o @relation(modelo.campo)
let relation_path = select! { Token::Ident(s) => s }
    .then(
        just(Token::Dot)
            .ignore_then(select! { Token::Ident(s) => s })
            .or_not(),
    )
    .map(|(first, second)| match second {
        Some(s) => format!("{first}.{s}"),
        None => first,
    });

let at_relation = just(Token::AtRelation)
    .ignore_then(
        relation_path
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .labelled("path de relacion"),
    )
    .map(|path| Annotation::Relation { path });
```

Luego en el `choice((...))` final de `annotation_parser`, agregar `at_references` y `at_relation` a la lista de opciones.

- [ ] **Step 3.5: Ejecutar tests del parser**

```
cargo test -p ag-dsl -- parser::tests
```

Resultado esperado: Todos los tests pasan incluyendo los 3 nuevos. Total ~28 tests verdes en parser.

- [ ] **Step 3.6: Ejecutar workspace completo**

```
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

Resultado esperado: Todos los tests verdes, cero warnings de clippy.

- [ ] **Step 3.7: Commit**

```bash
git add crates/ag-dsl/src/parser.rs
git commit -m "feat(dsl): v0.4 — parser ModelRef/ModelRefList, @references, @relation"
```

---

## Task 4: Semantic — 5 validaciones de relaciones

**Files:**
- Modify: `crates/ag-dsl/src/semantic.rs`

- [ ] **Step 4.1: Escribir 5 tests que fallen**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/semantic.rs`, agregar:

```rust
#[test]
fn v04_references_to_undefined_model_is_error() {
    let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(Ghost.id)
}
"#;
    let (_, diags) = compile(src);
    assert!(
        diags.iter().any(|d| d.is_error() && d.message.contains("Ghost")),
        "should error: model Ghost not defined. Got: {diags:?}"
    );
}

#[test]
fn v04_references_to_non_primary_field_is_warning() {
    let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique
}
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.email)
}
"#;
    let (_, diags) = compile(src);
    assert!(
        diags.iter().any(|d| !d.is_error() && d.message.contains("@primary")),
        "should warn: email is not @primary. Got: {diags:?}"
    );
}

#[test]
fn v04_model_ref_without_relation_is_error() {
    let src = r#"
model User {
    id UUID @primary @auto
}
model Post {
    id     UUID @primary @auto
    author User
}
"#;
    let (_, diags) = compile(src);
    assert!(
        diags.iter().any(|d| d.is_error() && d.message.contains("@relation")),
        "should error: ModelRef without @relation. Got: {diags:?}"
    );
}

#[test]
fn v04_relation_with_missing_fk_field_is_error() {
    let src = r#"
model User {
    id UUID @primary @auto
}
model Post {
    id     UUID @primary @auto
    author User @relation(nonexistent_id)
}
"#;
    let (_, diags) = compile(src);
    assert!(
        diags.iter().any(|d| d.is_error() && d.message.contains("nonexistent_id")),
        "should error: FK field nonexistent_id does not exist. Got: {diags:?}"
    );
}

#[test]
fn v04_circular_fk_is_error() {
    let src = r#"
model A {
    id UUID @primary @auto
    b_id UUID @references(B.id)
}
model B {
    id UUID @primary @auto
    a_id UUID @references(A.id)
}
"#;
    let (_, diags) = compile(src);
    assert!(
        diags.iter().any(|d| d.is_error() && d.message.contains("circular")),
        "should error: circular FK between A and B. Got: {diags:?}"
    );
}
```

- [ ] **Step 4.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- semantic::tests::v04
```

Resultado esperado: FAIL — las 5 validaciones no existen.

- [ ] **Step 4.3: Agregar llamadas a analyze()**

En `crates/ag-dsl/src/semantic.rs`, al final de la funcion `analyze()` (antes del `diags`), agregar:

```rust
// v0.4 validaciones de relaciones
check_references_model_exists(schema, &mut diags);
check_references_to_primary(schema, &mut diags);
check_model_ref_has_relation(schema, &mut diags);
check_relation_fk_field_exists(schema, &mut diags);
check_circular_fk(schema, &mut diags);
```

- [ ] **Step 4.4: Implementar check_references_model_exists**

Agregar al final del archivo (antes de los tests):

```rust
fn check_references_model_exists(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let model_names: HashSet<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::References { model: ref_model, .. } = &ann.value {
                    if !model_names.contains(ref_model.as_str()) {
                        diags.push(Diagnostic::semantic_error_with_hint(
                            ann.span.clone(),
                            format!(
                                "el modelo '{}' referenciado en @references no esta definido",
                                ref_model
                            ),
                            format!("define 'model {} {{ ... }}' en el schema", ref_model),
                        ));
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4.5: Implementar check_references_to_primary**

```rust
fn check_references_to_primary(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let primary_fields: HashMap<&str, HashSet<&str>> = schema
        .models
        .iter()
        .map(|m| {
            let primaries = m
                .fields
                .iter()
                .filter(|f| f.annotations.iter().any(|a| a.value == Annotation::Primary))
                .map(|f| f.name.value.as_str())
                .collect();
            (m.name.value.as_str(), primaries)
        })
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::References {
                    model: ref_model,
                    field: ref_field,
                } = &ann.value
                {
                    if let Some(primaries) = primary_fields.get(ref_model.as_str()) {
                        if !primaries.contains(ref_field.as_str()) {
                            diags.push(Diagnostic::warning(
                                ann.span.clone(),
                                format!(
                                    "se recomienda referenciar el campo @primary de '{}', no '{}'",
                                    ref_model, ref_field
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4.6: Implementar check_model_ref_has_relation**

```rust
fn check_model_ref_has_relation(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    for model in &schema.models {
        for field in &model.fields {
            let is_model_ref = matches!(
                field.ty.value,
                FieldType::ModelRef(_) | FieldType::ModelRefList(_)
            );
            if is_model_ref {
                let has_relation = field
                    .annotations
                    .iter()
                    .any(|a| matches!(a.value, Annotation::Relation { .. }));
                if !has_relation {
                    diags.push(Diagnostic::semantic_error_with_hint(
                        field.name.span.clone(),
                        format!(
                            "campo '{}' de tipo relacion requiere anotacion @relation",
                            field.name.value
                        ),
                        "agrega @relation(campo_fk) para campos de tipo modelo",
                    ));
                }
            }
        }
    }
}
```

- [ ] **Step 4.7: Implementar check_relation_fk_field_exists**

```rust
fn check_relation_fk_field_exists(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let model_fk_fields: HashMap<&str, HashSet<&str>> = schema
        .models
        .iter()
        .map(|m| {
            let fks = m
                .fields
                .iter()
                .filter(|f| {
                    !f.virtual_field
                        && f.annotations
                            .iter()
                            .any(|a| matches!(a.value, Annotation::References { .. }))
                })
                .map(|f| f.name.value.as_str())
                .collect();
            (m.name.value.as_str(), fks)
        })
        .collect();

    let all_model_names: HashSet<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::Relation { path } = &ann.value {
                    if !path.contains('.') {
                        // @relation(campo) — el campo debe existir como FK en el mismo modelo
                        let fks = model_fk_fields
                            .get(model.name.value.as_str())
                            .cloned()
                            .unwrap_or_default();
                        if !fks.contains(path.as_str()) {
                            diags.push(Diagnostic::semantic_error_with_hint(
                                ann.span.clone(),
                                format!(
                                    "el campo '{}' en @relation no existe o no tiene @references en el modelo '{}'",
                                    path, model.name.value
                                ),
                                format!(
                                    "agrega '{path} UUID @references(Modelo.id)' al modelo '{}'",
                                    model.name.value
                                ),
                            ));
                        }
                    } else {
                        // @relation(modelo.campo) — el modelo debe existir
                        let ref_model = path.splitn(2, '.').next().unwrap_or("");
                        if !ref_model.is_empty() && !all_model_names.contains(ref_model) {
                            diags.push(Diagnostic::semantic_error_with_hint(
                                ann.span.clone(),
                                format!(
                                    "el modelo '{}' en @relation no esta definido",
                                    ref_model
                                ),
                                format!("define 'model {} {{ ... }}' en el schema", ref_model),
                            ));
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4.8: Implementar check_circular_fk**

```rust
fn check_circular_fk(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    // Construye mapa: nombre_modelo -> set de modelos a los que apunta via FK real
    let mut fk_targets: HashMap<&str, HashSet<&str>> = HashMap::new();

    for model in &schema.models {
        let entry = fk_targets.entry(model.name.value.as_str()).or_default();
        for field in &model.fields {
            if field.virtual_field {
                continue;
            }
            for ann in &field.annotations {
                if let Annotation::References { model: ref_model, .. } = &ann.value {
                    entry.insert(ref_model.as_str());
                }
            }
        }
    }

    let model_names: Vec<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    let mut reported: HashSet<(&str, &str)> = HashSet::new();

    for &a in &model_names {
        for &b in &model_names {
            if a == b {
                continue;
            }
            let a_to_b = fk_targets.get(a).map_or(false, |s| s.contains(b));
            let b_to_a = fk_targets.get(b).map_or(false, |s| s.contains(a));

            if a_to_b && b_to_a && !reported.contains(&(b, a)) {
                reported.insert((a, b));

                // Buscar el span del campo FK de A que apunta a B
                let span = schema
                    .models
                    .iter()
                    .find(|m| m.name.value == a)
                    .and_then(|m| {
                        m.fields.iter().find(|f| {
                            !f.virtual_field
                                && f.annotations.iter().any(|ann| {
                                    matches!(&ann.value,
                                        Annotation::References { model, .. } if model == b)
                                })
                        })
                    })
                    .and_then(|f| {
                        f.annotations.iter().find(|ann| {
                            matches!(&ann.value,
                                Annotation::References { model, .. } if model == b)
                        })
                    })
                    .map(|ann| ann.span.clone())
                    .unwrap_or(0..0);

                diags.push(Diagnostic::semantic_error_with_hint(
                    span,
                    format!(
                        "referencia circular entre modelos '{}' y '{}': ambos tienen FK hacia el otro",
                        a, b
                    ),
                    "elimina una de las FKs o convierte una en campo virtual con @relation",
                ));
            }
        }
    }
}
```

- [ ] **Step 4.9: Ejecutar tests semanticos**

```
cargo test -p ag-dsl -- semantic::tests
```

Resultado esperado: Los 5 tests nuevos pasan + los 16 existentes. Total: 21 tests en semantic.

- [ ] **Step 4.10: Ejecutar workspace completo**

```
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] **Step 4.11: Commit**

```bash
git add crates/ag-dsl/src/semantic.rs
git commit -m "feat(dsl): v0.4 — 5 validaciones semanticas de relaciones @references/@relation"
```

---

## Task 5: SQL codegen — omitir virtuales, generar FOREIGN KEY

**Files:**
- Modify: `crates/ag-dsl/src/codegen/sql_gen.rs`

- [ ] **Step 5.1: Escribir tests que fallen**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/codegen/sql_gen.rs`, agregar:

```rust
#[test]
fn v04_virtual_field_not_in_sql() {
    let schema = schema_from(r#"
model User {
    id    UUID @primary @auto
    email String @unique
}
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#);
    let sql = generate_migration(&schema);
    assert!(
        !sql.contains("\"author\" "),
        "virtual field should not be a column. SQL:\n{sql}"
    );
    assert!(sql.contains("\"author_id\""), "FK field must be present");
}

#[test]
fn v04_fk_generates_constraint() {
    let schema = schema_from(r#"
model User {
    id UUID @primary @auto
}
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
}
"#);
    let sql = generate_migration(&schema);
    assert!(
        sql.contains("FOREIGN KEY (\"author_id\") REFERENCES \"user\" (\"id\")"),
        "should generate FOREIGN KEY. SQL:\n{sql}"
    );
    assert!(
        sql.contains("fk_post_author_id_user_id"),
        "should use naming convention fk_<tabla>_<campo>_<ref_tabla>_<ref_campo>"
    );
}

#[test]
fn v04_one_to_one_unique_fk() {
    let schema = schema_from(r#"
model User {
    id UUID @primary @auto
}
model Profile {
    id      UUID @primary @auto
    user_id UUID @unique @references(User.id)
}
"#);
    let sql = generate_migration(&schema);
    assert!(
        sql.contains("idx_profile_user_id_unique"),
        "1:1 FK should have unique index"
    );
    assert!(
        sql.contains("FOREIGN KEY (\"user_id\") REFERENCES \"user\" (\"id\")"),
        "should also have FK constraint"
    );
}
```

- [ ] **Step 5.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- codegen::sql_gen::tests::v04
```

Resultado esperado: FAIL.

- [ ] **Step 5.3: Modificar generate_table() para omitir campos virtuales**

En `crates/ag-dsl/src/codegen/sql_gen.rs`, en `generate_table()`, reemplazar la seccion de calculo de `total_items` y el loop de columnas:

```rust
fn generate_table(model: &ModelDef) -> String {
    let table_name = to_snake_case(&model.name.value);
    let mut out = String::new();

    // Solo campos no-virtuales tienen columna SQL
    let real_fields: Vec<_> = model.fields.iter().filter(|f| !f.virtual_field).collect();

    let checks: Vec<String> = real_fields
        .iter()
        .flat_map(|f| field_check_constraints(&table_name, f))
        .collect();

    out.push_str(&format!("CREATE TABLE IF NOT EXISTS \"{table_name}\" (\n"));

    let total_items = real_fields.len() + checks.len();
    let mut item_idx = 0;

    for field in &real_fields {
        let col = generate_column(field);
        item_idx += 1;
        if item_idx < total_items {
            out.push_str(&format!("    {col},\n"));
        } else {
            out.push_str(&format!("    {col}\n"));
        }
    }
    for check in &checks {
        item_idx += 1;
        if item_idx < total_items {
            out.push_str(&format!("    {check},\n"));
        } else {
            out.push_str(&format!("    {check}\n"));
        }
    }

    out.push_str(");\n");

    // Indices UNIQUE (solo campos no-virtuales)
    for field in &real_fields {
        if field.annotations.iter().any(|a| a.value == Annotation::Unique)
            && !field.annotations.iter().any(|a| a.value == Annotation::Primary)
        {
            let col_name = to_snake_case(&field.name.value);
            let idx_name = format!("idx_{table_name}_{col_name}_unique");
            out.push_str(&format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS \"{idx_name}\" \
                 ON \"{table_name}\" (\"{col_name}\");\n"
            ));
        }
    }

    // FOREIGN KEY constraints (v0.4)
    out.push_str(&generate_fk_constraints(model, &table_name));

    out
}
```

- [ ] **Step 5.4: Implementar generate_fk_constraints()**

Agregar la funcion auxiliar despues de `generate_table()`:

```rust
fn generate_fk_constraints(model: &ModelDef, table_name: &str) -> String {
    let mut out = String::new();

    for field in &model.fields {
        if field.virtual_field {
            continue;
        }
        for ann in &field.annotations {
            if let Annotation::References {
                model: ref_model,
                field: ref_field,
            } = &ann.value
            {
                let col_name = to_snake_case(&field.name.value);
                let ref_table = to_snake_case(ref_model);
                let ref_col = to_snake_case(ref_field);
                let constraint_name =
                    format!("fk_{table_name}_{col_name}_{ref_table}_{ref_col}");
                out.push_str(&format!(
                    "ALTER TABLE \"{table_name}\" ADD CONSTRAINT \"{constraint_name}\" \
                     FOREIGN KEY (\"{col_name}\") REFERENCES \"{ref_table}\" (\"{ref_col}\") \
                     ON DELETE RESTRICT;\n"
                ));
            }
        }
    }
    out
}
```

Tambien agregar `Annotation` al import de `use crate::ast::...` si no esta:

```rust
use crate::ast::{Annotation, FieldDef, FieldType, ModelDef, Schema};
```

- [ ] **Step 5.5: Ejecutar tests SQL**

```
cargo test -p ag-dsl -- codegen::sql_gen::tests
```

Resultado esperado: Todos los tests SQL pasan incluyendo los 3 nuevos. Total: ~11 tests.

- [ ] **Step 5.6: Commit**

```bash
git add crates/ag-dsl/src/codegen/sql_gen.rs
git commit -m "feat(dsl): v0.4 — SQL codegen omite virtuales, genera FOREIGN KEY con ALTER TABLE"
```

---

## Task 6: Rust codegen — Option<M> y Vec<M> para campos virtuales

**Files:**
- Modify: `crates/ag-dsl/src/codegen/rust_gen.rs`

- [ ] **Step 6.1: Escribir tests que fallen**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/codegen/rust_gen.rs`, agregar:

```rust
#[test]
fn v04_model_ref_generates_option() {
    let schema = schema_from(r#"
model User {
    id    UUID @primary @auto
    email String
}
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#);
    let out = generate_models(&schema);
    assert!(
        out.contains("pub author: Option<User>"),
        "N:1 virtual field should be Option<User>. Output:\n{out}"
    );
    // author_id es UUID real, debe aparecer normalmente
    assert!(
        out.contains("pub author_id:"),
        "FK field must be in struct. Output:\n{out}"
    );
}

#[test]
fn v04_model_ref_list_generates_vec() {
    let schema = schema_from(r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
}
"#);
    let out = generate_models(&schema);
    assert!(
        out.contains("pub posts: Vec<Post>"),
        "1:N virtual field should be Vec<Post>. Output:\n{out}"
    );
}

#[test]
fn v04_virtual_field_excluded_from_create_request() {
    let schema = schema_from(r#"
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
model User {
    id UUID @primary @auto
}
"#);
    let out = generate_models(&schema);
    // CreatePostRequest no debe tener el campo virtual 'author'
    let create_section = out
        .find("pub struct CreatePostRequest")
        .map(|i| &out[i..])
        .unwrap_or("");
    let end = create_section.find('}').unwrap_or(create_section.len());
    let create_body = &create_section[..end];
    assert!(
        !create_body.contains("author:"),
        "virtual field should not appear in CreateRequest. CreateRequest body:\n{create_body}"
    );
}
```

- [ ] **Step 6.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- codegen::rust_gen::tests::v04
```

Resultado esperado: FAIL.

- [ ] **Step 6.3: Modificar generate_model_struct() para campos virtuales**

En `crates/ag-dsl/src/codegen/rust_gen.rs`, reemplazar el loop de campos en `generate_model_struct()`:

```rust
for field in &model.fields {
    let fname = &field.name.value;
    if field.virtual_field {
        let rust_ty = match &field.ty.value {
            FieldType::ModelRef(m) => format!("Option<{m}>"),
            FieldType::ModelRefList(m) => format!("Vec<{m}>"),
            _ => continue,
        };
        out.push_str(&format!("    pub {fname}: {rust_ty},\n"));
    } else if field.ty.value == FieldType::Uuid && fname == "id" {
        out.push_str("    pub id: Uuid,\n");
    } else {
        let rust_ty = rust_field_type(field);
        out.push_str(&format!("    pub {fname}: {rust_ty},\n"));
    }
}
```

Agregar `FieldType` al import de `use crate::ast::...` si no esta:

```rust
use crate::ast::{
    extract_path_params, to_snake_case, Annotation, EndpointDef, FieldDef, FieldType, HttpMethod,
    ModelDef, RequestDef, ResponseDef, Schema,
};
```

- [ ] **Step 6.4: Excluir virtuales de generate_create_request() y generate_update_request()**

En `generate_create_request()`, cambiar el filtro:

```rust
let create_fields: Vec<_> = model
    .fields
    .iter()
    .filter(|f| !is_auto_generated(f) && !f.virtual_field)
    .collect();
```

En `generate_update_request()`, cambiar el filtro:

```rust
let update_fields: Vec<_> = model
    .fields
    .iter()
    .filter(|f| !is_auto_generated(f) && !is_primary(f) && !f.virtual_field)
    .collect();
```

- [ ] **Step 6.5: Ejecutar tests Rust**

```
cargo test -p ag-dsl -- codegen::rust_gen::tests
```

Resultado esperado: Todos los tests pasan incluyendo los 3 nuevos.

- [ ] **Step 6.6: Commit**

```bash
git add crates/ag-dsl/src/codegen/rust_gen.rs
git commit -m "feat(dsl): v0.4 — Rust codegen genera Option<M> y Vec<M> para relaciones"
```

---

## Task 7: TypeScript codegen — tipos opcionales para relaciones

**Files:**
- Modify: `crates/ag-dsl/src/codegen/ts_gen.rs`

- [ ] **Step 7.1: Escribir test que falle**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/codegen/ts_gen.rs`, agregar:

```rust
#[test]
fn v04_ts_relation_types() {
    let schema = schema_from(r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#);
    let out = generate_types(&schema);
    assert!(
        out.contains("posts?: Post[];"),
        "1:N should be optional Post[]. Output:\n{out}"
    );
    assert!(
        out.contains("author?: User;"),
        "N:1 should be optional User. Output:\n{out}"
    );
    assert!(
        out.contains("author_id: string;"),
        "FK field should be string. Output:\n{out}"
    );
}
```

- [ ] **Step 7.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- codegen::ts_gen::tests::v04
```

Resultado esperado: FAIL.

- [ ] **Step 7.3: Modificar generate_model_interface() para campos virtuales**

En `crates/ag-dsl/src/codegen/ts_gen.rs`, reemplazar el loop en `generate_model_interface()`:

```rust
for field in &model.fields {
    let fname = &field.name.value;
    if field.virtual_field {
        let ts_ty = match &field.ty.value {
            crate::ast::FieldType::ModelRef(m) => format!("{m}"),
            crate::ast::FieldType::ModelRefList(m) => format!("{m}[]"),
            _ => continue,
        };
        // Los campos virtuales son siempre opcionales (pueden no estar cargados)
        out.push_str(&format!("  {fname}?: {ts_ty};\n"));
    } else {
        let ts_ty = ts_field_type(field);
        let optional_marker = if field.optional { "?" } else { "" };
        out.push_str(&format!("  {fname}{optional_marker}: {ts_ty};\n"));
    }
}
```

Tambien excluir campos virtuales de `generate_create_interface()` y `generate_update_interface()`:

En `generate_create_interface()`:
```rust
let create_fields: Vec<_> = model
    .fields
    .iter()
    .filter(|f| !is_auto_generated(f) && !f.virtual_field)
    .collect();
```

En `generate_update_interface()`:
```rust
let update_fields: Vec<_> = model
    .fields
    .iter()
    .filter(|f| !is_auto_generated(f) && !is_primary(f) && !f.virtual_field)
    .collect();
```

- [ ] **Step 7.4: Ejecutar tests TypeScript**

```
cargo test -p ag-dsl -- codegen::ts_gen::tests
```

Resultado esperado: Todos los tests pasan incluyendo el nuevo.

- [ ] **Step 7.5: Commit**

```bash
git add crates/ag-dsl/src/codegen/ts_gen.rs
git commit -m "feat(dsl): v0.4 — TypeScript codegen genera tipos de relacion opcionales"
```

---

## Task 8: OpenAPI codegen — $ref y array+$ref para relaciones

**Files:**
- Modify: `crates/ag-dsl/src/codegen/openapi_gen.rs`

- [ ] **Step 8.1: Escribir test que falle**

En el bloque `#[cfg(test)]` de `crates/ag-dsl/src/codegen/openapi_gen.rs`, agregar:

```rust
#[test]
fn v04_openapi_ref_schemas() {
    let schema = schema_from(r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#);
    let json_str = generate_openapi(&schema);
    let doc: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");

    // Post.author debe usar $ref a User
    let author_prop = &doc["components"]["schemas"]["Post"]["properties"]["author"];
    assert_eq!(
        author_prop["$ref"],
        "#/components/schemas/User",
        "N:1 field should use $ref to User"
    );

    // User.posts debe ser array con $ref a Post
    let posts_prop = &doc["components"]["schemas"]["User"]["properties"]["posts"];
    assert_eq!(posts_prop["type"], "array", "1:N field should be array type");
    assert_eq!(
        posts_prop["items"]["$ref"],
        "#/components/schemas/Post",
        "1:N items should use $ref to Post"
    );

    // author_id debe ser string/uuid normal (no virtual)
    let author_id_prop = &doc["components"]["schemas"]["Post"]["properties"]["author_id"];
    assert_eq!(author_id_prop["type"], "string");
    assert_eq!(author_id_prop["format"], "uuid");
}
```

- [ ] **Step 8.2: Ejecutar para confirmar fallo**

```
cargo test -p ag-dsl -- codegen::openapi_gen::tests::v04
```

Resultado esperado: FAIL.

- [ ] **Step 8.3: Modificar model_schema() para campos virtuales**

En `crates/ag-dsl/src/codegen/openapi_gen.rs`, en `model_schema()`, reemplazar el loop de campos:

```rust
for field in &model.fields {
    if create_only && is_auto_generated(field) {
        continue;
    }
    // v0.4: campos virtuales usan $ref en lugar de type+format
    if field.virtual_field {
        let fname = &field.name.value;
        let prop = match &field.ty.value {
            crate::ast::FieldType::ModelRef(m) => {
                json!({ "$ref": format!("#/components/schemas/{m}") })
            }
            crate::ast::FieldType::ModelRefList(m) => {
                json!({
                    "type": "array",
                    "items": { "$ref": format!("#/components/schemas/{m}") }
                })
            }
            _ => continue,
        };
        properties.insert(fname.clone(), prop);
        // Campos virtuales son opcionales en OpenAPI (pueden no estar cargados)
        continue;
    }

    let fname = &field.name.value;
    let (ts_type, format) = field.ty.value.openapi_type();

    let mut prop = serde_json::Map::new();
    prop.insert("type".to_owned(), json!(ts_type));
    if let Some(fmt) = format {
        prop.insert("format".to_owned(), json!(fmt));
    }
    if field.optional {
        prop.insert("type".to_owned(), json!([ts_type, "null"]));
    }
    apply_validation_constraints(&mut prop, field);

    properties.insert(fname.clone(), Value::Object(prop));

    if !(field.optional || (create_only && is_auto_generated(field))) {
        required.push(json!(fname));
    }
}
```

- [ ] **Step 8.4: Ejecutar tests OpenAPI**

```
cargo test -p ag-dsl -- codegen::openapi_gen::tests
```

Resultado esperado: Todos los tests pasan incluyendo el nuevo. Total: ~6 tests.

- [ ] **Step 8.5: Commit**

```bash
git add crates/ag-dsl/src/codegen/openapi_gen.rs
git commit -m "feat(dsl): v0.4 — OpenAPI codegen genera \$ref y array+\$ref para relaciones"
```

---

## Task 9: Verificacion final y cierre de documentacion

**Files:**
- Modify: `docs/roadmap/STATUS.md`
- Modify: `README.md` (si aplica segun regla de sincronizacion)

- [ ] **Step 9.1: Ejecutar suite completa**

```
cargo test --workspace
```

Resultado esperado: 90+ tests (73 existentes + ~17 nuevos), cero fallos.

- [ ] **Step 9.2: Ejecutar verificaciones de calidad**

```
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo audit
```

Resultado esperado: Cero errores en los tres comandos.

- [ ] **Step 9.3: Marcar entregable en STATUS.md**

En `docs/roadmap/STATUS.md`, en la seccion "Fase 3 - Anti-DSL alpha", cambiar:

```
- [ ] DSL version 0.4: relaciones entre modelos (1:1, 1:N, N:M).
```

Por:

```
- [x] DSL version 0.4: relaciones entre modelos (1:1, 1:N, N:M). @references/@relation,
  SQL FOREIGN KEY, Rust Option<M>/Vec<M>, TypeScript tipos opcionales, OpenAPI $ref.
```

- [ ] **Step 9.4: Commit final**

```bash
git add docs/roadmap/STATUS.md
git commit -m "docs(status): DSL v0.4 completado — relaciones @references/@relation con codegen completo"
```

- [ ] **Step 9.5: Push**

```bash
git push origin fase-3
```
