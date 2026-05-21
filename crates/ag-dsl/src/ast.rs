//! Tipos del AST del Anti-DSL.
//!
//! Todas las construcciones del lenguaje se representan como nodos del AST.
//! Los nodos clave llevan informacion de span para reportar errores precisos.
//! En DSL v0.1–v0.2 el AST cubre modelos, request/response/error types,
//! endpoints HTTP y anotaciones basicas.

use crate::lexer::Span;

/// Rango de bytes en el texto fuente adjunto a un valor.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    /// Valor del nodo.
    pub value: T,
    /// Posicion en el texto fuente (bytes).
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Construye un nodo con su span.
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Schema completo: punto de entrada del AST.
///
/// Contiene configuracion, modelos (v0.1) y tipos de API + endpoints (v0.2).
#[derive(Debug, Clone, Default)]
pub struct Schema {
    /// Bloque `config { ... }` opcional.
    pub config: Option<Config>,
    /// Definiciones de modelo en orden de aparicion.
    pub models: Vec<ModelDef>,
    /// Tipos de cuerpo de peticion: `request Nombre { ... }`.
    pub requests: Vec<RequestDef>,
    /// Tipos de cuerpo de respuesta: `response Nombre { ... }`.
    pub responses: Vec<ResponseDef>,
    /// Tipos de error HTTP: `error Nombre { status N message "..." }`.
    pub errors: Vec<ErrorDef>,
    /// Definiciones de endpoint: `endpoint Nombre { method path body response errors }`.
    pub endpoints: Vec<EndpointDef>,
}

/// Bloque `config { ... }`.
///
/// Campos reconocidos en v0.1: `project_name` y `database`.
/// Campos desconocidos se reportan como warnings en el paso semantico.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Nombre del proyecto.
    pub project_name: Option<String>,
    /// Backend de base de datos: `"postgres"`, `"sqlite"`, etc.
    pub database: Option<String>,
}

/// Definicion de modelo: `model Nombre { campos... }`.
#[derive(Debug, Clone)]
pub struct ModelDef {
    /// Nombre del modelo con span para reportar errores.
    pub name: Spanned<String>,
    /// Campos del modelo en orden de aparicion.
    pub fields: Vec<FieldDef>,
    /// Span del bloque completo (del `model` al `}`).
    pub span: Span,
}

/// Definicion de campo dentro de un modelo.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Nombre del campo.
    pub name: Spanned<String>,
    /// Tipo del campo.
    pub ty: Spanned<FieldType>,
    /// Si el campo es opcional (`?` al final del tipo).
    pub optional: bool,
    /// Lista de anotaciones en orden de aparicion.
    pub annotations: Vec<Spanned<Annotation>>,
}

/// Tipos primitivos soportados en DSL v0.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// `UUID` — identificador unico universal.
    Uuid,
    /// `String` — texto sin limite definido (el limite se pone con @max).
    String,
    /// `Int` — entero de 64 bits con signo.
    Int,
    /// `Float` — punto flotante de 64 bits.
    Float,
    /// `Bool` — booleano.
    Bool,
    /// `Timestamp` — fecha y hora con zona (UTC por defecto).
    Timestamp,
    /// `Decimal` — numero decimal de precision arbitraria (para dinero).
    Decimal,
}

impl FieldType {
    /// Nombre Rust del tipo generado.
    pub fn rust_type(&self, optional: bool) -> std::string::String {
        let base = match self {
            FieldType::Uuid => "uuid::Uuid",
            FieldType::String => "String",
            FieldType::Int => "i64",
            FieldType::Float => "f64",
            FieldType::Bool => "bool",
            FieldType::Timestamp => "chrono::DateTime<chrono::Utc>",
            FieldType::Decimal => "rust_decimal::Decimal",
        };
        if optional {
            format!("Option<{base}>")
        } else {
            base.to_owned()
        }
    }

    /// Tipo SQL correspondiente.
    pub fn sql_type(&self) -> &'static str {
        match self {
            FieldType::Uuid => "UUID",
            FieldType::String => "TEXT",
            FieldType::Int => "BIGINT",
            FieldType::Float => "DOUBLE PRECISION",
            FieldType::Bool => "BOOLEAN",
            FieldType::Timestamp => "TIMESTAMPTZ",
            FieldType::Decimal => "NUMERIC",
        }
    }

    /// Tipo TypeScript correspondiente.
    pub fn ts_type(&self) -> &'static str {
        match self {
            FieldType::Uuid => "string",
            FieldType::String => "string",
            FieldType::Int => "number",
            FieldType::Float => "number",
            FieldType::Bool => "boolean",
            FieldType::Timestamp => "string",
            FieldType::Decimal => "string",
        }
    }

    /// Formato OpenAPI del tipo (pares type/format).
    pub fn openapi_type(&self) -> (&'static str, Option<&'static str>) {
        match self {
            FieldType::Uuid => ("string", Some("uuid")),
            FieldType::String => ("string", None),
            FieldType::Int => ("integer", Some("int64")),
            FieldType::Float => ("number", Some("double")),
            FieldType::Bool => ("boolean", None),
            FieldType::Timestamp => ("string", Some("date-time")),
            FieldType::Decimal => ("string", Some("decimal")),
        }
    }
}

/// Anotaciones DSL v0.1–v0.3.
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    /// `@primary` — clave primaria.
    Primary,
    /// `@unique` — restriccion UNIQUE.
    Unique,
    /// `@auto` — valor autogenerado (UUID, SERIAL, NOW()).
    Auto,
    /// `@auto_update` — actualizado en cada UPDATE.
    AutoUpdate,
    /// `@default(valor)` — valor por defecto.
    Default(DefaultValue),
    // ---- Validaciones v0.3 ----
    /// `@min(N)` — longitud minima (String) o valor minimo (Int/Float/Decimal).
    Min(i64),
    /// `@max(N)` — longitud maxima (String) o valor maximo (Int/Float/Decimal).
    Max(i64),
    /// `@email` — valida formato de email RFC 5321 basico.
    Email,
    /// `@regex("patron")` — valida contra expresion regular.
    Regex(std::string::String),
    /// `@length(N)` — longitud exacta de caracteres (solo String).
    Length(i64),
}

/// Valor para la anotacion `@default(...)`.
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    /// Literal entero: `@default(0)`.
    Int(i64),
    /// Literal string: `@default("activo")`.
    String(std::string::String),
    /// Literal booleano: `@default(true)`.
    Bool(bool),
    /// Identificador (variante de enum u otra referencia): `@default(USER)`.
    Ident(std::string::String),
}

impl std::fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultValue::Int(n) => write!(f, "{n}"),
            DefaultValue::String(s) => write!(f, "'{s}'"),
            DefaultValue::Bool(b) => write!(f, "{b}"),
            DefaultValue::Ident(s) => write!(f, "'{s}'"),
        }
    }
}

// ============================================================
// DSL v0.2 — API types y endpoints
// ============================================================

/// Definicion de tipo de cuerpo de peticion: `request Nombre { campos... }`.
#[derive(Debug, Clone)]
pub struct RequestDef {
    /// Nombre del tipo con span.
    pub name: Spanned<std::string::String>,
    /// Campos del cuerpo de peticion.
    pub fields: Vec<FieldDef>,
    /// Span del bloque completo.
    pub span: Span,
}

/// Definicion de tipo de cuerpo de respuesta: `response Nombre { campos... }`.
#[derive(Debug, Clone)]
pub struct ResponseDef {
    /// Nombre del tipo con span.
    pub name: Spanned<std::string::String>,
    /// Campos del cuerpo de respuesta.
    pub fields: Vec<FieldDef>,
    /// Span del bloque completo.
    pub span: Span,
}

/// Definicion de error HTTP: `error Nombre { status N message "texto" }`.
#[derive(Debug, Clone)]
pub struct ErrorDef {
    /// Nombre del tipo de error con span.
    pub name: Spanned<std::string::String>,
    /// Codigo de estado HTTP (ej. 409, 422).
    pub status: Spanned<u16>,
    /// Mensaje legible para el cliente.
    pub message: Spanned<std::string::String>,
    /// Span del bloque completo.
    pub span: Span,
}

/// Definicion de endpoint HTTP.
///
/// ```text
/// endpoint CreateUser {
///     method   POST
///     path     /users
///     body     CreateUserRequest
///     response UserResponse
///     errors   [EmailTaken, WeakPassword]
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EndpointDef {
    /// Nombre del endpoint con span.
    pub name: Spanned<std::string::String>,
    /// Metodo HTTP.
    pub method: Spanned<HttpMethod>,
    /// Path HTTP (ej. `/users/{id}`).
    pub path: Spanned<std::string::String>,
    /// Nombre del tipo de cuerpo de peticion (referencia a `RequestDef`).
    pub body: Option<Spanned<std::string::String>>,
    /// Nombre del tipo de respuesta (referencia a `ResponseDef`).
    pub response: Option<Spanned<std::string::String>>,
    /// Nombres de tipos de error (referencias a `ErrorDef`).
    pub errors: Vec<Spanned<std::string::String>>,
    /// Span del bloque completo.
    pub span: Span,
}

/// Metodo HTTP soportado en DSL v0.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    /// HTTP GET — lectura.
    Get,
    /// HTTP POST — creacion.
    Post,
    /// HTTP PUT — reemplazo completo.
    Put,
    /// HTTP PATCH — actualizacion parcial.
    Patch,
    /// HTTP DELETE — eliminacion.
    Delete,
}

impl HttpMethod {
    /// Retorna la representacion textual del metodo en mayusculas.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convierte un path DSL (`/users/{id}`) a formato Axum (`/users/:id`).
pub fn dsl_path_to_axum(path: &str) -> std::string::String {
    let mut result = std::string::String::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            result.push(':');
            for inner in chars.by_ref() {
                if inner == '}' {
                    break;
                }
                result.push(inner);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extrae los nombres de los parametros de path de un path DSL.
///
/// `/users/{id}/posts/{postId}` → `["id", "postId"]`
pub fn extract_path_params(path: &str) -> Vec<std::string::String> {
    let mut params = Vec::new();
    let mut current = std::string::String::new();
    let mut inside = false;
    for ch in path.chars() {
        match ch {
            '{' => inside = true,
            '}' => {
                if !current.is_empty() {
                    params.push(current.clone());
                    current.clear();
                }
                inside = false;
            }
            c if inside => current.push(c),
            _ => {}
        }
    }
    params
}

/// Convierte PascalCase o camelCase a snake_case.
///
/// Usado para convertir nombres de endpoints/modelos a nombres de funcion Rust.
pub fn to_snake_case(s: &str) -> std::string::String {
    let mut result = std::string::String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}
