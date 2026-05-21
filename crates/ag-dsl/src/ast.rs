//! Tipos del AST del Anti-DSL.
//!
//! Todas las construcciones del lenguaje se representan como nodos del AST.
//! Los nodos clave llevan informacion de span para reportar errores precisos.
//! En DSL v0.1 el AST cubre modelos, campos, tipos primitivos y anotaciones
//! basicas (@primary, @unique, @auto, @auto_update, @default).

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
/// En v0.1 contiene configuracion opcional y cero o mas modelos.
/// Las versiones futuras añadiran endpoints, enums y eventos.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    /// Bloque `config { ... }` opcional.
    pub config: Option<Config>,
    /// Definiciones de modelo en orden de aparicion.
    pub models: Vec<ModelDef>,
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

/// Anotaciones DSL v0.1.
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
