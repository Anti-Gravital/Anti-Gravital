# Anti-DSL — Referencia del lenguaje v0.1–v0.4

**Estado:** Implementado y verificado en `crates/ag-dsl`.
**Versiones cubiertas:** v0.1 (modelos), v0.2 (endpoints), v0.3 (validaciones), v0.4 (relaciones).
**Ultima actualizacion:** 2026-05-21.

---

## Indice

1. [Estructura de un schema](#1-estructura-de-un-schema)
2. [Tipos de campo](#2-tipos-de-campo)
3. [Anotaciones](#3-anotaciones)
4. [Modelos — DSL v0.1](#4-modelos--dsl-v01)
5. [Endpoints — DSL v0.2](#5-endpoints--dsl-v02)
6. [Validaciones — DSL v0.3](#6-validaciones--dsl-v03)
7. [Relaciones — DSL v0.4](#7-relaciones--dsl-v04)
8. [Generacion de codigo](#8-generacion-de-codigo)
9. [Diagnostics y errores](#9-diagnostics-y-errores)
10. [Ejemplo completo](#10-ejemplo-completo)

---

## 1. Estructura de un schema

Un archivo `schema.ag` contiene, en cualquier orden:

```
config { ... }        # opcional — metadatos del proyecto
model  Nombre { ... } # definicion de tabla/entidad
request  Nombre { ... }  # cuerpo de peticion HTTP
response Nombre { ... }  # cuerpo de respuesta HTTP
error    Nombre { ... }  # tipo de error HTTP
endpoint Nombre { ... }  # ruta HTTP
```

### Bloque config

```
config {
    project_name "mi-api"
    database     "postgres"
}
```

Campos reconocidos:

| Campo          | Tipo   | Descripcion                              |
|----------------|--------|------------------------------------------|
| `project_name` | string | Nombre del proyecto. Aparece en OpenAPI. |
| `database`     | string | Backend de BD. Solo `"postgres"` soportado. |

Cualquier otro campo produce un warning en el analisis semantico.

### Comentarios

El caracter `#` inicia un comentario de linea. Se puede usar en cualquier punto.

```
# esto es un comentario
model User {
    id UUID @primary @auto  # clave primaria autogenerada
}
```

---

## 2. Tipos de campo

### Tipos primitivos

| Tipo DSL    | Rust                           | SQL               | TypeScript | OpenAPI            |
|-------------|--------------------------------|-------------------|-----------|--------------------|
| `UUID`      | `uuid::Uuid`                   | `UUID`            | `string`  | `string / uuid`    |
| `String`    | `String`                       | `TEXT`            | `string`  | `string`           |
| `Int`       | `i64`                          | `BIGINT`          | `number`  | `integer / int64`  |
| `Float`     | `f64`                          | `DOUBLE PRECISION`| `number`  | `number / double`  |
| `Bool`      | `bool`                         | `BOOLEAN`         | `boolean` | `boolean`          |
| `Timestamp` | `chrono::DateTime<chrono::Utc>`| `TIMESTAMPTZ`     | `string`  | `string / date-time` |
| `Decimal`   | `rust_decimal::Decimal`        | `NUMERIC`         | `string`  | `string / decimal` |

### Tipos relacionales (v0.4)

| Sintaxis DSL   | Descripcion                    | Columna SQL | Rust           | TypeScript |
|----------------|--------------------------------|-------------|----------------|------------|
| `ModelName`    | Referencia N:1 o 1:1           | No          | `Option<M>`    | `M?`       |
| `ModelName[]`  | Lista 1:N                      | No          | `Vec<M>`       | `M[]?`     |

Los tipos relacionales requieren la anotacion `@relation`. Nunca generan columna SQL.

### Opcionalidad

Anadir `?` despues del tipo hace el campo nullable:

```
bio String?   # Option<String> en Rust, TEXT nullable en SQL, string? en TS
```

---

## 3. Anotaciones

### v0.1 — Identidad y generacion automatica

| Anotacion          | Aplica a     | SQL generado                                              | Descripcion                          |
|--------------------|-------------|-----------------------------------------------------------|--------------------------------------|
| `@primary`         | cualquier   | `PRIMARY KEY`                                             | Clave primaria. Solo una por modelo. |
| `@unique`          | cualquier   | `CREATE UNIQUE INDEX IF NOT EXISTS`                       | Restriccion de unicidad.             |
| `@auto`            | UUID/Int/Timestamp | UUID: `DEFAULT gen_random_uuid()` / Int: `BIGSERIAL` / Timestamp: `DEFAULT NOW()` | Valor autogenerado. |
| `@auto_update`     | Timestamp   | `DEFAULT NOW()` + trigger implicito                       | Se actualiza en cada UPDATE.         |
| `@default(valor)`  | cualquier   | `DEFAULT valor`                                           | Valor por defecto estatico.          |

Valores aceptados por `@default`:
- Entero: `@default(0)`
- String: `@default("activo")`
- Booleano: `@default(true)` / `@default(false)`
- Identificador: `@default(PENDING)`

Reglas semanticas:
- `@auto` solo es valido en campos `UUID`, `Int` y `Timestamp`.
- `@auto_update` requiere tipo `Timestamp`.
- Solo puede haber un `@primary` por modelo (warning si no hay ninguno).

### v0.2 — (sin anotaciones nuevas; ver endpoints)

### v0.3 — Validaciones

| Anotacion       | Aplica a               | SQL generado (CHECK)                      | Rust generado (validate())             |
|-----------------|------------------------|-------------------------------------------|----------------------------------------|
| `@min(N)`       | String, Int, Float, Decimal | `char_length(col) >= N` o `col >= N` | `len() < N` o valor < N               |
| `@max(N)`       | String, Int, Float, Decimal | `char_length(col) <= N` o `col <= N` | `len() > N` o valor > N               |
| `@length(N)`    | String                 | `char_length(col) = N`                    | `len() != N`                           |
| `@email`        | String                 | `col ~ '^[^@]+@[^@]+\.[^@]+$'`           | Verifica presencia de `@` y `.`        |
| `@regex("pat")` | String                 | `col ~ 'pat'` (regex POSIX PostgreSQL)    | Validacion ejecutable cacheada (requiere crate `regex`) |

Restricciones semanticas:
- `@min` > `@max` es error (se detecta en el mismo campo).
- `@email`, `@regex`, `@length` solo son validos en `String`.
- `@min` / `@max` no son validos en `UUID`, `Bool` ni `Timestamp`.
- `@length(N)` requiere N > 0.

### v0.4 — Relaciones

| Anotacion                   | Aplica a        | Descripcion                                              |
|-----------------------------|----------------|----------------------------------------------------------|
| `@references(Modelo.campo)` | UUID (no virtual) | Declara clave foranea. Genera FOREIGN KEY en SQL.   |
| `@relation(campo)`          | ModelRef        | Campo virtual N:1 o 1:1. `campo` es la FK en este modelo. |
| `@relation(Modelo.campo)`   | ModelRefList    | Campo virtual 1:N. `Modelo` es el modelo destino, `campo` es la FK alli. |

Reglas semanticas:
- El modelo en `@references` debe existir en el schema.
- Se emite warning si el campo referenciado no tiene `@primary`.
- Todo campo de tipo `ModelRef` o `ModelRefList` requiere `@relation`.
- `@relation(campo)` requiere que `campo` exista en el mismo modelo con `@references`.
- `@relation(Modelo.campo)` requiere que `Modelo` exista en el schema.
- Se detecta y reporta referencia circular entre FKs reales (no virtuales).

---

## 4. Modelos — DSL v0.1

### Sintaxis

```
model NombreEnPascalCase {
    nombre_campo TipoCampo [@anotacion1] [@anotacion2(arg)] ...
    ...
}
```

### Ejemplo

```
model User {
    id         UUID      @primary @auto
    email      String    @unique @email @max(255) @min(5)
    name       String    @min(2) @max(100)
    age        Int?      @min(0) @max(150)
    created_at Timestamp @auto
    updated_at Timestamp @auto_update
}
```

### Lo que genera

**SQL** (`migrations/0001_initial.sql`):
```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS "user" (
    "id"         UUID      NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    "email"      TEXT      NOT NULL,
    "name"       TEXT      NOT NULL,
    "age"        BIGINT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "chk_user_email_max"   CHECK (char_length("email") <= 255),
    CONSTRAINT "chk_user_email_min"   CHECK (char_length("email") >= 5),
    CONSTRAINT "chk_user_name_min"    CHECK (char_length("name") >= 2),
    CONSTRAINT "chk_user_name_max"    CHECK (char_length("name") <= 100),
    CONSTRAINT "chk_user_age_min"     CHECK ("age" >= 0),
    CONSTRAINT "chk_user_age_max"     CHECK ("age" <= 150)
);
CREATE UNIQUE INDEX IF NOT EXISTS "idx_user_email_unique" ON "user" ("email");
```

**Rust** (`src/models.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub age: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub age: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub age: Option<i64>,
}
```

Los campos `@auto` y `@auto_update` se excluyen de `CreateRequest` y `UpdateRequest`.

---

## 5. Endpoints — DSL v0.2

### Tipos auxiliares

#### request

Cuerpo de peticion HTTP. Los campos siguen la misma sintaxis que los modelos.

```
request CreateProductRequest {
    name  String @min(2) @max(200)
    price Decimal @min(0)
    stock Int     @min(0)
}
```

#### response

Cuerpo de respuesta HTTP.

```
response ProductResponse {
    id    UUID
    name  String
    price Decimal
    stock Int
}
```

#### error

Tipo de error HTTP. Requiere codigo de estado 4xx o 5xx y mensaje de texto.

```
error NotFound    { status 404 message "Recurso no encontrado" }
error EmailTaken  { status 409 message "El email ya esta registrado" }
```

### Endpoint

```
endpoint NombreEndpoint {
    method   GET | POST | PUT | PATCH | DELETE
    path     /ruta/{param}
    body     NombreRequest       # opcional
    response NombreResponse      # opcional
    errors   [Error1, Error2]    # opcional
}
```

Path params se declaran entre llaves: `/users/{id}`.

### Lo que genera

**Rust** (`src/handlers.rs`) — stubs Axum:
```rust
/// POST /users — CreateUser
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, axum::http::StatusCode> {
    todo!()
}
```

**Rust** (`src/router.rs`):
```rust
pub fn api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/users", routing::post(handlers::create_user))
        .route("/users/:id", routing::get(handlers::get_user))
}
```

Los paths DSL `{param}` se convierten automaticamente a `:param` (formato Axum).

**TypeScript** (`clients/typescript/client.ts`):
```typescript
/** POST /users */
export async function createUser(body: CreateUserRequest): Promise<UserResponse> {
    const resp = await fetch(`${BASE_URL}/users`, {
        method: 'post',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (!resp.ok) throw new Error(`createUser failed: ${resp.status}`);
    return resp.json();
}
```

**OpenAPI** (`openapi.json`) — operaciones con `requestBody`, `responses` y codigos de error de los `ErrorDef` referenciados.

---

## 6. Validaciones — DSL v0.3

Las validaciones se declaran como anotaciones sobre campos individuales. Se aplican tanto en modelos como en `request` y `response`.

### Ejemplo con todas las validaciones

```
request CreateAccountRequest {
    email    String  @email @max(255) @min(5)
    username String  @min(3) @max(50) @regex("^[a-zA-Z0-9_]+$")
    age      Int     @min(18) @max(120)
    currency String  @length(3)
    balance  Decimal @min(0)
}
```

### Validacion en Rust

El codegen produce un metodo `validate()` en los structs de `request`:

```rust
impl CreateAccountRequest {
    pub fn validate(&self) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        if self.email.len() > 255 {
            errors.push("email: longitud maxima es 255, encontrado N caracteres".to_owned());
        }
        // ... resto de validaciones
        errors
    }
}
```

Los campos con `@regex` emiten validacion Rust ejecutable y cachean el patron compilado con `OnceLock`. El proyecto generado debe declarar `regex = "1"`.

---

## 7. Relaciones — DSL v0.4

### Patron de declaracion

Toda relacion requiere dos partes:

1. **Campo FK real** — columna en la base de datos:
   ```
   author_id UUID @references(User.id)
   ```

2. **Campo virtual** — solo existe en el codegen, sin columna SQL:
   ```
   author User @relation(author_id)
   ```

### 1:N (un usuario tiene muchos posts)

```
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(Post.author_id)   # virtual 1:N
}

model Post {
    id        UUID   @primary @auto
    title     String @min(1) @max(200)
    author_id UUID   @references(User.id)    # FK real
    author    User   @relation(author_id)    # virtual N:1
}
```

### 1:1 (un usuario tiene un perfil)

La cardinalidad 1:1 se infiere de `@unique` + `@references` en el campo FK:

```
model User {
    id      UUID    @primary @auto
    profile Profile @relation(Profile.user_id)  # virtual 1:1
}

model Profile {
    id      UUID    @primary @auto
    user_id UUID    @unique @references(User.id)  # FK + UNIQUE = 1:1
    bio     String?
    user    User    @relation(user_id)             # virtual
}
```

### N:M mediante modelo de union explicito

```
model Post {
    id   UUID     @primary @auto
    tags PostTag[] @relation(PostTag.post_id)   # virtual N:M
}

model Tag {
    id    UUID      @primary @auto
    posts PostTag[] @relation(PostTag.tag_id)   # virtual N:M
}

model PostTag {
    id       UUID @primary @auto
    post_id  UUID @references(Post.id)
    tag_id   UUID @references(Tag.id)
}
```

### Lo que genera cada parte

| Declaracion                   | SQL                  | Rust             | TypeScript   | OpenAPI             |
|-------------------------------|----------------------|------------------|--------------|---------------------|
| `fk UUID @references(M.campo)`| FOREIGN KEY constraint | `pub fk: Uuid` | `fk: string` | `string/uuid`       |
| `m M @relation(fk)`           | (omitido)            | `pub m: Option<M>` | `m?: M`    | `$ref: '#/...M'`    |
| `ms M[] @relation(M.fk)`      | (omitido)            | `pub ms: Vec<M>` | `ms?: M[]`   | `array $ref: '#/...M'` |

Los campos virtuales se excluyen de `CreateRequest`/`UpdateRequest` en Rust y de `Create/UpdateInterface` en TypeScript.

### Convencion de nombres para FK constraints

```
fk_{tabla}_{columna_fk}_{tabla_ref}_{columna_ref}
```

Ejemplo: `fk_post_author_id_user_id`.

---

## 8. Generacion de codigo

### Comando

```bash
ag generate [--schema schema.ag] [--output ./generated]
```

Por defecto lee `schema.ag` en el directorio actual y escribe en `./generated/`.

### Artefactos producidos

| Archivo generado                     | Contenido                                           | Cuando se genera        |
|--------------------------------------|-----------------------------------------------------|-------------------------|
| `migrations/0001_initial.sql`        | CREATE TABLE, UNIQUE INDEX, FOREIGN KEY             | Siempre (si hay modelos) |
| `src/models.rs`                      | Structs con serde, Create/Update requests           | Siempre (si hay modelos) |
| `src/types.rs`                       | Structs request/response, constantes de error       | Si hay request/response/error |
| `src/handlers.rs`                    | Handler stubs Axum async                            | Si hay endpoints         |
| `src/router.rs`                      | Router Axum con todas las rutas                     | Si hay endpoints         |
| `clients/typescript/types.ts`        | Interfaces TypeScript con Create/Update variants    | Siempre (si hay modelos) |
| `clients/typescript/client.ts`       | Funciones fetch tipadas por endpoint                | Si hay endpoints         |
| `openapi.json`                       | Documento OpenAPI 3.1 completo                      | Siempre                  |

### Convencion de nombres SQL

Los nombres de modelos en PascalCase se convierten a snake_case para tablas y columnas:

| DSL              | SQL            |
|------------------|----------------|
| `User`           | `user`         |
| `BlogPost`       | `blog_post`    |
| `OrderItem`      | `order_item`   |
| `userId`         | `user_id`      |

### Operaciones de schema

```bash
ag schema lint [--schema schema.ag]
```
Reporta errores semanticos y warnings de mejores practicas.

```bash
ag schema diff <referencia> [--schema schema.ag]
```
Compara el schema actual contra un archivo de referencia y clasifica los cambios como breaking / non-breaking. Cambios breaking incluyen: eliminar modelo, eliminar campo, cambiar tipo de campo, eliminar endpoint.

---

## 9. Diagnostics y errores

El compilador reporta errores y warnings con posicion exacta (linea:columna).

### Errores de lexer

| Situacion                   | Mensaje de ejemplo                                  |
|-----------------------------|-----------------------------------------------------|
| Caracter no reconocido      | `error de lex en bytes 42..43`                      |

### Errores de parser

| Situacion                   | Mensaje de ejemplo                                  |
|-----------------------------|-----------------------------------------------------|
| Token inesperado            | `token inesperado: Ident("foo")`                    |
| Bloque sin cerrar           | `delimitador sin cerrar: LBrace`                    |
| EOF inesperado              | `fin de archivo inesperado`                         |

### Errores semanticos (v0.1)

| Situacion                        | Mensaje                                               |
|----------------------------------|-------------------------------------------------------|
| Nombre de modelo duplicado       | `nombre de modelo duplicado: 'User'`                  |
| Modelo sin campos                | `el modelo 'Empty' no tiene campos`                   |
| Campo duplicado en modelo        | `campo duplicado 'id' en el modelo 'User'`            |
| Mas de un @primary               | `el modelo 'X' tiene mas de un @primary`              |
| @auto en tipo incompatible       | `@auto en el campo 'name' no es compatible con String` |
| @auto_update en no-Timestamp     | `@auto_update en el campo 'x' requiere tipo Timestamp` |

### Warnings semanticos (v0.1)

| Situacion                   | Mensaje                                               |
|-----------------------------|-------------------------------------------------------|
| Modelo sin @primary         | `el modelo 'Tag' no tiene campo @primary`             |

### Errores semanticos (v0.2)

| Situacion                         | Mensaje                                               |
|-----------------------------------|-------------------------------------------------------|
| Nombre de request/response/error duplicado | `nombre de request duplicado: 'CreateUser'` |
| Endpoint con ruta duplicada       | `ruta duplicada: POST /users`                         |
| Body no definido en schema        | `body 'FooRequest' en endpoint 'X' no esta definido`  |
| Response no definido en schema    | `response 'FooResponse' en endpoint 'X' no esta definido` |
| Error ref no definida             | `error 'BadError' en endpoint 'X' no esta definido`   |
| Codigo HTTP invalido en error     | `codigo de estado HTTP invalido 200 en el error 'X'`  |

### Errores semanticos (v0.3)

| Situacion                         | Mensaje                                               |
|-----------------------------------|-------------------------------------------------------|
| @email en campo no-String         | `@email en 'M.campo': solo aplica a String`           |
| @regex en campo no-String         | `@regex en 'M.campo': solo aplica a String`           |
| @length en campo no-String        | `@length en 'M.campo': solo aplica a String`          |
| @min/@max en UUID/Bool/Timestamp  | `@min en 'M.campo': no aplica al tipo Uuid`           |
| @min > @max                       | `@min(10) > @max(5) en 'M.campo': el minimo supera al maximo` |
| @length(0) o negativo             | `@length(0): el valor debe ser mayor que cero`        |

### Errores semanticos (v0.4)

| Situacion                              | Mensaje                                               |
|----------------------------------------|-------------------------------------------------------|
| @references a modelo inexistente       | `el modelo 'Ghost' referenciado en @references no esta definido` |
| Campo ModelRef sin @relation           | `campo 'author' de tipo relacion requiere anotacion @relation` |
| @relation con campo FK inexistente     | `el campo 'bad_id' en @relation no existe o no tiene @references en 'Post'` |
| @relation a modelo inexistente en 1:N  | `el modelo 'Ghost' en @relation no esta definido`     |
| Referencia circular FK                 | `referencia circular entre modelos 'A' y 'B': ambos tienen FK hacia el otro` |

### Warnings semanticos (v0.4)

| Situacion                          | Mensaje                                               |
|------------------------------------|-------------------------------------------------------|
| @references a campo no @primary    | `se recomienda referenciar el campo @primary de 'User', no 'email'` |

---

## 10. Ejemplo completo

El ejemplo `examples/ecommerce-api/schema.ag` incluido en el repositorio demuestra todas las capacidades de v0.1 a v0.4:

- 5 modelos con relaciones 1:N y N:M
- Validaciones `@email`, `@min`, `@max` en campos de texto y numericos
- 4 FOREIGN KEY constraints en el SQL generado
- Tipos `Option<M>` y `Vec<M>` en Rust
- Referencias `$ref` en OpenAPI
- 8 endpoints con request/response/error types

Para regenerar los artefactos:

```bash
cd examples/ecommerce-api
ag schema lint --schema schema.ag
ag generate --schema schema.ag --output generated
```

Para el flujo completo desde cero:

```bash
ag new mi-proyecto
cd mi-proyecto
# editar schema.ag
ag schema lint
ag generate
ag dev
```

---

## Limitaciones conocidas de v0.1–v0.4

- Los campos `@regex` generan codigo ejecutable en `validate()`; cada patron se compila una vez con `OnceLock` y requiere declarar la crate `regex` en el proyecto generado.
- `ag schema diff` clasifica cambios pero no genera SQL de migracion incremental; eso pertenece a DSL v0.9.
- Los handler stubs generados contienen `todo!()` y deben implementarse manualmente.
- Los campos virtuales en structs Rust se incluyen como `Option<M>` / `Vec<M>` pero los query builders para cargarlos no se generan aun; se implementaran en Fase 4 (`ag-data`).
- El servidor LSP (`ag-lsp`) y el plugin VS Code no estan implementados en esta version.
