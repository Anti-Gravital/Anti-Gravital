# Especificacion de Diseno: DSL v0.4 — Relaciones entre modelos

**Fecha:** 2026-05-21
**Fase:** Fase 3 — Anti-DSL alpha
**Entregable roadmap:** DSL version 0.4: relaciones entre modelos (1:1, 1:N, N:M)
**Estado:** Aprobado por mantenedor

---

## 1. Objetivo

Extender el compilador Anti-DSL para soportar relaciones entre modelos con
tres cardinalidades: uno-a-uno (1:1), uno-a-muchos (1:N) y muchos-a-muchos
(N:M). Las relaciones N:M se expresan mediante modelos de union explicitos,
bajo control total del desarrollador.

El diseno sigue el patron **campo FK explicito + campo virtual**: el campo de
clave foranea genera columna real en la base de datos; el campo virtual existe
solo en el codegen (Rust, TypeScript, OpenAPI) y no produce columna SQL.

---

## 2. Sintaxis DSL

### 2.1 Relacion 1:N

```
model User {
    id    UUID   @primary @auto
    email String @unique @email @max(255)
    posts Post[] @relation(post.author_id)
}

model Post {
    id        UUID   @primary @auto
    title     String @min(1) @max(200)
    author_id UUID   @references(User.id)
    author    User   @relation(author_id)
}
```

### 2.2 Relacion 1:1

```
model User {
    id      UUID    @primary @auto
    email   String  @unique @email
    profile Profile @relation(profile.user_id)
}

model Profile {
    id      UUID    @primary @auto
    user_id UUID    @unique @references(User.id)
    bio     String?
    user    User    @relation(user_id)
}
```

La cardinalidad 1:1 se infiere de la combinacion `@unique + @references` en el
mismo campo FK. No se requiere sintaxis adicional.

### 2.3 Relacion N:M (modelo de union explicito)

```
model Post {
    id   UUID     @primary @auto
    tags PostTag[] @relation(post_tag.post_id)
}

model Tag {
    id    UUID      @primary @auto
    posts PostTag[] @relation(post_tag.tag_id)
}

model PostTag {
    post_id UUID @references(Post.id)
    tag_id  UUID @references(Tag.id)
}
```

### 2.4 Reglas de sintaxis

- `@references(Modelo.campo)` — declara FK; genera columna SQL y FOREIGN KEY constraint.
- `@relation(campo)` — campo virtual N:1 o 1:1; el argumento es el nombre del campo FK en el modelo actual.
- `@relation(modelo.campo)` — campo virtual 1:N; el argumento identifica el campo FK en el modelo destino.
- `Tipo[]` — tipo lista; solo valido para campos con `@relation`.
- La cardinalidad 1:1 se determina por la presencia de `@unique` en el campo FK.

---

## 3. Cambios en el AST

### 3.1 FieldType — dos variantes nuevas

```rust
pub enum FieldType {
    // Existentes: Uuid, String, Int, Float, Bool, Timestamp, Decimal
    // Nuevas:
    ModelRef(String),      // "User" — campo virtual N:1 o 1:1
    ModelRefList(String),  // "Post" en Post[] — campo virtual 1:N o N:M
}
```

### 3.2 Annotation — dos variantes nuevas

```rust
pub enum Annotation {
    // Existentes: Primary, Unique, Auto, AutoUpdate, Default, Min, Max, Email, Regex, Length
    // Nuevas:
    References { model: String, field: String },  // @references(User.id)
    Relation { path: String },                     // @relation(author_id) o @relation(post.author_id)
}
```

### 3.3 FieldDef — un campo nuevo

```rust
pub struct FieldDef {
    pub name: Spanned<String>,
    pub ty: Spanned<FieldType>,
    pub optional: bool,
    pub annotations: Vec<Spanned<Annotation>>,
    pub virtual_field: bool,   // true = sin columna SQL
}
```

`virtual_field` es `false` por defecto para todos los campos existentes.
Un campo es virtual cuando su tipo es `ModelRef` o `ModelRefList` y tiene
la anotacion `@relation`.

### 3.4 Inferencia de cardinalidad

| Tipo del campo    | Anotacion           | Cardinalidad    | Virtual |
|-------------------|---------------------|-----------------|---------|
| UUID              | @references(M.f)    | FK real         | No      |
| UUID + @unique    | @references(M.f)    | FK 1:1          | No      |
| ModelRef(M)       | @relation(campo)    | N:1 o 1:1       | Si      |
| ModelRefList(M)   | @relation(m.campo)  | 1:N             | Si      |

---

## 4. Cambios en el Lexer

Dos tokens nuevos en `lexer.rs`:

```rust
#[token("@references")]
AtReferences,

#[token("@relation")]
AtRelation,
```

El tipo lista `Post[]` no requiere token nuevo. El parser combina el `Ident`
existente con `LBracket` y `RBracket` ya presentes.

---

## 5. Cambios en el Parser

Tres reglas extendidas en `parser.rs`:

### 5.1 Tipo de campo extendido

```
field_type = primitive_type
           | Ident              -> ModelRef(ident)
           | Ident "[" "]"      -> ModelRefList(ident)
```

### 5.2 Anotacion @references

```
annotation_references = "@references" "(" Ident "." Ident ")"
```

Produce `Annotation::References { model, field }`.

### 5.3 Anotacion @relation

```
relation_path = Ident             # "author_id"
              | Ident "." Ident   # "post.author_id"

annotation_relation = "@relation" "(" relation_path ")"
```

Produce `Annotation::Relation { path }`.

El parser marca `virtual_field = true` cuando el tipo es `ModelRef` o
`ModelRefList`.

---

## 6. Analisis semantico (5 validaciones nuevas)

Las funciones existentes en `semantic.rs` no se modifican. Se agregan cinco
funciones nuevas invocadas desde `analyze()`.

### V1: @references apunta a modelo existente

- Recorre todos los campos con `Annotation::References { model, field }`.
- Error si `model` no esta en `schema.models`.
- Mensaje: `"el modelo 'X' referenciado en @references(X.Y) no esta definido"`.

### V2: @references apunta a campo @primary del modelo destino

- Warning si el campo `field` del modelo destino no tiene `@primary`.
- Mensaje: `"se recomienda referenciar el campo @primary de 'X'"`.

### V3: Campo ModelRef/ModelRefList requiere @relation

- Error si un campo con tipo `ModelRef` o `ModelRefList` no tiene `@relation`.
- Mensaje: `"campo 'X' de tipo 'Y' requiere anotacion @relation"`.

### V4: @relation(campo) apunta a FK valida en el modelo actual

- Para `@relation(campo)` (sin punto): verifica que `campo` exista en el mismo
  modelo y tenga `@references`.
- Error: `"el campo 'campo' en @relation no existe o no tiene @references en el modelo 'M'"`.
- Para `@relation(modelo.campo)`: verifica que `modelo` exista en el schema.

### V5: No se permiten FK circulares directas

- Error si el modelo A tiene un campo no-virtual con `@references(B.x)` Y el
  modelo B tiene un campo no-virtual con `@references(A.y)`.
- Mensaje: `"referencia circular entre modelos 'A' y 'B': ambos tienen FK hacia el otro"`.
- Las relaciones virtuales bidireccionales son validas y no se reportan.

---

## 7. Codegen — los 4 generadores

### 7.1 SQL (`sql_gen.rs`)

- Los campos con `virtual_field: true` se omiten de la lista de columnas.
- Por cada campo con `Annotation::References { model, field }` se agrega al
  final del `CREATE TABLE` una clausula `CONSTRAINT FOREIGN KEY`.

```sql
CREATE TABLE IF NOT EXISTS "post" (
    "id"        UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    "title"     TEXT NOT NULL,
    "author_id" UUID NOT NULL,
    CONSTRAINT "fk_post_author_id_user_id"
        FOREIGN KEY ("author_id") REFERENCES "user" ("id") ON DELETE RESTRICT
);
```

El nombre del constraint sigue el patron: `fk_{tabla}_{campo}_{tabla_destino}_{campo_destino}`.

### 7.2 Rust (`rust_gen.rs`)

- Campos virtuales `ModelRef(M)` generan `pub campo: Option<M>`.
- Campos virtuales `ModelRefList(M)` generan `pub campo: Vec<M>`.
- Los campos FK reales generan su tipo primitivo normalmente (ej. `uuid::Uuid`).

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Post {
    pub id: uuid::Uuid,
    pub title: String,
    pub author_id: uuid::Uuid,
    pub author: Option<User>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub posts: Vec<Post>,
}
```

### 7.3 TypeScript (`ts_gen.rs`)

- Campos virtuales `ModelRef(M)` generan `campo?: M`.
- Campos virtuales `ModelRefList(M)` generan `campo?: M[]`.

```typescript
export interface Post {
  id: string;
  title: string;
  author_id: string;
  author?: User;
}

export interface User {
  id: string;
  email: string;
  posts?: Post[];
}
```

### 7.4 OpenAPI (`openapi_gen.rs`)

- Campos virtuales `ModelRef(M)` generan `$ref: '#/components/schemas/M'`.
- Campos virtuales `ModelRefList(M)` generan `type: array, items: {$ref: ...}`.

```yaml
components:
  schemas:
    Post:
      type: object
      properties:
        id:
          type: string
          format: uuid
        author_id:
          type: string
          format: uuid
        author:
          $ref: '#/components/schemas/User'
    User:
      type: object
      properties:
        posts:
          type: array
          items:
            $ref: '#/components/schemas/Post'
```

---

## 8. Tests

17 tests nuevos distribuidos en los modulos afectados.

### Lexer (2 tests)
- `v04_relation_tokens`: `@references` y `@relation` tokenizan correctamente.
- `v04_list_type_tokens`: `Post[]` produce `Ident("Post") + LBracket + RBracket`.

### Parser (3 tests)
- `v04_parses_references_annotation`: campo con `@references(User.id)` parsea y produce `References { model: "User", field: "id" }`.
- `v04_parses_relation_single`: `author User @relation(author_id)` produce `virtual_field: true` y `ModelRef("User")`.
- `v04_parses_relation_list`: `posts Post[] @relation(post.author_id)` produce `ModelRefList("Post")`.

### Semantico (5 tests)
- `v04_references_to_undefined_model_is_error`
- `v04_references_to_non_primary_field_is_warning`
- `v04_model_ref_without_relation_is_error`
- `v04_relation_with_missing_fk_field_is_error`
- `v04_circular_fk_is_error`

### SQL codegen (3 tests)
- `v04_virtual_field_not_in_sql`: campo virtual no aparece como columna.
- `v04_fk_generates_constraint`: `@references` genera clausula `FOREIGN KEY`.
- `v04_one_to_one_unique_fk`: `@unique + @references` genera indice unico y FK.

### Rust codegen (2 tests)
- `v04_model_ref_generates_option`: `User @relation(...)` genera `Option<User>`.
- `v04_model_ref_list_generates_vec`: `Post[] @relation(...)` genera `Vec<Post>`.

### TypeScript codegen (1 test)
- `v04_ts_relation_types`: relaciones generan tipos opcionales correctos.

### OpenAPI codegen (1 test)
- `v04_openapi_ref_schemas`: relaciones generan `$ref` y `array + $ref`.

---

## 8.1 Nota de implementacion: metodos de FieldType

Las variantes `ModelRef(String)` y `ModelRefList(String)` deben implementar
los metodos existentes de `FieldType`. Como los campos virtuales nunca llegan
al codegen de columnas SQL, `sql_type()` puede retornar `""` (unreachable en
practica). Los metodos `rust_type()`, `ts_type()` y `openapi_type()` retornan
el nombre del modelo tal cual, ya que el codegen de relaciones los envuelve
en `Option<M>` / `Vec<M>` segun `virtual_field`:

```rust
FieldType::ModelRef(m) | FieldType::ModelRefList(m) => {
    // rust_type: el codegen de relaciones no llama a este metodo directamente;
    // retorna el nombre base para uso futuro.
    m.as_str()
}
```

Tambien se debe extender el `Default` de `FieldDef` o asegurar que toda
construccion existente en el parser inicialice `virtual_field: false`.

---

## 9. Archivos modificados

Ningun archivo nuevo. Solo extensiones en archivos existentes:

| Archivo | Cambios |
|---|---|
| `crates/ag-dsl/src/ast.rs` | Variantes `ModelRef`, `ModelRefList` en `FieldType`; variantes `References`, `Relation` en `Annotation`; campo `virtual_field` en `FieldDef` |
| `crates/ag-dsl/src/lexer.rs` | Tokens `AtReferences`, `AtRelation` |
| `crates/ag-dsl/src/parser.rs` | Reglas para tipos lista, `@references`, `@relation` |
| `crates/ag-dsl/src/semantic.rs` | 5 funciones de validacion nuevas |
| `crates/ag-dsl/src/codegen/sql_gen.rs` | Omision de campos virtuales; generacion de `FOREIGN KEY` |
| `crates/ag-dsl/src/codegen/rust_gen.rs` | Generacion de `Option<M>` y `Vec<M>` |
| `crates/ag-dsl/src/codegen/ts_gen.rs` | Generacion de tipos opcionales y listas |
| `crates/ag-dsl/src/codegen/openapi_gen.rs` | Generacion de `$ref` y `array + $ref` |

---

## 10. Criterios de aceptacion

- `cargo fmt`, `cargo clippy`, `cargo test --workspace` pasan sin errores.
- Los 17 tests nuevos estan verdes.
- Los 73 tests existentes siguen verdes (sin regresiones).
- El schema de ejemplo `ecommerce` con User, Product, Order, OrderItem y Tag
  compila sin errores y genera SQL, Rust, TS y OpenAPI correctos.
- `cargo audit` y `cargo deny` pasan sin vulnerabilidades nuevas.
