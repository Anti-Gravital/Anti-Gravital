//! Anti-DSL AST types.
//!
//! All language constructs are represented as AST nodes.
//! Key nodes carry span information to report precise errors.
//! In DSL v0.1-v0.7 the AST covers models, request/response/error types,
//! HTTP endpoints, annotations, auth/policy (v0.5), event declarations (v0.6)
//! and mail/domain blocks (v0.7).

use crate::lexer::Span;

/// Byte range in the source text attached to a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    /// Node value.
    pub value: T,
    /// Position in the source text (bytes).
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Builds a node with its span.
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Full schema: AST entry point.
///
/// Contains configuration, models (v0.1), API types and endpoints (v0.2),
/// validations (v0.3), relations (v0.4), auth/policy (v0.5), events (v0.6)
/// and mail and domain blocks (v0.7).
#[derive(Debug, Clone, Default)]
pub struct Schema {
    /// Optional `config { ... }` block.
    pub config: Option<Config>,
    /// Model definitions in order of appearance.
    pub models: Vec<ModelDef>,
    /// Request body types: `request Name { ... }`.
    pub requests: Vec<RequestDef>,
    /// Response body types: `response Name { ... }`.
    pub responses: Vec<ResponseDef>,
    /// HTTP error types: `error Name { status N message "..." }`.
    pub errors: Vec<ErrorDef>,
    /// Endpoint definitions: `endpoint Name { method path body response errors }`.
    pub endpoints: Vec<EndpointDef>,
    /// Event declarations (v0.6): `event name { payload T retain Nd }`.
    pub events: Vec<EventDef>,
    /// Transactional mail blocks (v0.7): `mail name { ... }`.
    pub mails: Vec<MailBlock>,
    /// DNS domain blocks (v0.7): `domain name { ... }`.
    pub domains: Vec<DomainBlock>,
}

/// `config { ... }` block.
///
/// Fields recognized in v0.1: `project_name` and `database`.
/// Unknown fields are reported as warnings in the semantic pass.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Project name.
    pub project_name: Option<String>,
    /// Database backend: `"postgres"`, `"sqlite"`, etc.
    pub database: Option<String>,
}

/// Model definition: `model Name { fields... }`.
#[derive(Debug, Clone)]
pub struct ModelDef {
    /// Model name with span for error reporting.
    pub name: Spanned<String>,
    /// Model fields in order of appearance.
    pub fields: Vec<FieldDef>,
    /// Span of the full block (from `model` to `}`).
    pub span: Span,
}

/// Field definition within a model.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// Field name.
    pub name: Spanned<String>,
    /// Field type.
    pub ty: Spanned<FieldType>,
    /// Whether the field is optional (`?` at the end of the type).
    pub optional: bool,
    /// List of annotations in order of appearance.
    pub annotations: Vec<Spanned<Annotation>>,
    /// true when the field is a ModelRef/ModelRefList with @relation.
    /// SQL codegen omits it; Rust/TS/OpenAPI include it as a nested type.
    pub virtual_field: bool,
}

/// Primitive types supported in DSL v0.1 and model references (v0.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// `UUID` — universally unique identifier.
    Uuid,
    /// `String` — text with no defined limit (the limit is set with @max).
    String,
    /// `Int` — signed 64-bit integer.
    Int,
    /// `Float` — 64-bit floating point.
    Float,
    /// `Bool` — boolean.
    Bool,
    /// `Timestamp` — date and time with zone (UTC by default).
    Timestamp,
    /// `Decimal` — arbitrary-precision decimal number (for money).
    Decimal,
    /// Reference to another model — virtual N:1 or 1:1 field, no SQL column.
    ModelRef(std::string::String),
    /// List of references — virtual 1:N field, no SQL column.
    ModelRefList(std::string::String),
}

impl FieldType {
    /// Rust name of the generated type.
    pub fn rust_type(&self, optional: bool) -> std::string::String {
        if let FieldType::ModelRef(m) | FieldType::ModelRefList(m) = self {
            return if optional {
                format!("Option<{m}>")
            } else {
                m.clone()
            };
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

    /// Corresponding SQL type. Empty for virtual fields (they never generate a column).
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

    /// Corresponding TypeScript type.
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

    /// OpenAPI format of the type (type/format pairs).
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

/// DSL v0.1-v0.4 annotations.
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    /// `@primary` — primary key.
    Primary,
    /// `@unique` — UNIQUE constraint.
    Unique,
    /// `@auto` — auto-generated value (UUID, SERIAL, NOW()).
    Auto,
    /// `@auto_update` — updated on every UPDATE.
    AutoUpdate,
    /// `@default(value)` — default value.
    Default(DefaultValue),
    // ---- v0.3 validations ----
    /// `@min(N)` — minimum length (String) or minimum value (Int/Float/Decimal).
    Min(i64),
    /// `@max(N)` — maximum length (String) or maximum value (Int/Float/Decimal).
    Max(i64),
    /// `@email` — validates basic RFC 5321 email format.
    Email,
    /// `@regex("pattern")` — validates against a regular expression.
    Regex(std::string::String),
    /// `@length(N)` — exact character length (String only).
    Length(i64),
    // ---- v0.4 relations ----
    /// `@references(Model.field)` — foreign key with a real SQL column.
    References {
        /// Target model name.
        model: std::string::String,
        /// Target field name (normally the primary key).
        field: std::string::String,
    },
    /// `@relation(field)` or `@relation(model.field)` — virtual field with no SQL column.
    Relation {
        /// Relation path: `field` for N:1, `model.field` for 1:N.
        path: std::string::String,
    },
}

/// Value for the `@default(...)` annotation.
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    /// Integer literal: `@default(0)`.
    Int(i64),
    /// String literal: `@default("activo")`.
    String(std::string::String),
    /// Boolean literal: `@default(true)`.
    Bool(bool),
    /// Identifier (enum variant or other reference): `@default(USER)`.
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
// DSL v0.2 — API types and endpoints
// ============================================================

/// Request body type definition: `request Name { fields... }`.
#[derive(Debug, Clone)]
pub struct RequestDef {
    /// Type name with span.
    pub name: Spanned<std::string::String>,
    /// Request body fields.
    pub fields: Vec<FieldDef>,
    /// Span of the full block.
    pub span: Span,
}

/// Response body type definition: `response Name { fields... }`.
#[derive(Debug, Clone)]
pub struct ResponseDef {
    /// Type name with span.
    pub name: Spanned<std::string::String>,
    /// Response body fields.
    pub fields: Vec<FieldDef>,
    /// Span of the full block.
    pub span: Span,
}

/// HTTP error definition: `error Name { status N message "text" }`.
#[derive(Debug, Clone)]
pub struct ErrorDef {
    /// Error type name with span.
    pub name: Spanned<std::string::String>,
    /// HTTP status code (e.g. 409, 422).
    pub status: Spanned<u16>,
    /// Client-readable message.
    pub message: Spanned<std::string::String>,
    /// Span of the full block.
    pub span: Span,
}

// ============================================================
// DSL v0.5 — auth/policy on endpoints
// ============================================================

/// Authentication mode declared on an endpoint (DSL v0.5).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AuthMode {
    /// No authentication required (default value).
    #[default]
    None,
    /// The JWT token is required; the Shield rejects requests without a token.
    Required,
    /// The JWT token is optional; the handler decides how to treat its absence.
    Optional,
}

impl AuthMode {
    /// Textual representation of the mode as it appears in the schema.
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMode::None => "none",
            AuthMode::Required => "required",
            AuthMode::Optional => "optional",
        }
    }
}

// ============================================================
// DSL v0.6 — event declaration
// ============================================================

/// Event declaration in DSL v0.6: `event name { payload T retain Nd }`.
#[derive(Debug, Clone)]
pub struct EventDef {
    /// Event name in `entity.action` format.
    pub name: Spanned<std::string::String>,
    /// Payload type name (reference to model/request/response).
    pub payload: Spanned<std::string::String>,
    /// Retention in days. `None` if not declared.
    pub retain_days: Option<u32>,
    /// Span of the full block.
    pub span: Span,
}

/// HTTP endpoint definition.
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
    /// Endpoint name with span.
    pub name: Spanned<std::string::String>,
    /// HTTP method.
    pub method: Spanned<HttpMethod>,
    /// HTTP path (e.g. `/users/{id}`).
    pub path: Spanned<std::string::String>,
    /// Request body type name (reference to `RequestDef`).
    pub body: Option<Spanned<std::string::String>>,
    /// Response type name (reference to `ResponseDef`).
    pub response: Option<Spanned<std::string::String>>,
    /// Error type names (references to `ErrorDef`).
    pub errors: Vec<Spanned<std::string::String>>,
    /// Endpoint authentication mode (v0.5). Default: None.
    pub auth: AuthMode,
    /// RBAC policy expression (v0.5). Only valid when auth != None.
    pub policy: Option<Spanned<std::string::String>>,
    /// Names of events emitted by this endpoint (v0.6).
    pub emits: Vec<Spanned<std::string::String>>,
    /// Span of the full block.
    pub span: Span,
}

/// HTTP method supported in DSL v0.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    /// HTTP GET — read.
    Get,
    /// HTTP POST — create.
    Post,
    /// HTTP PUT — full replacement.
    Put,
    /// HTTP PATCH — partial update.
    Patch,
    /// HTTP DELETE — removal.
    Delete,
}

impl HttpMethod {
    /// Returns the uppercase textual representation of the method.
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
        // TECH-DEBT:
        // motivo: char::to_lowercase() garantiza al menos 1 caracter; unwrap() es seguro pero viola
        //         la regla del proyecto de no usar unwrap() fuera de tests.
        // impacto: bajo (no puede panic en practica)
        // eliminacion esperada: PR de limpieza post-Fase 4
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

// ============================================================
// DSL v0.7 — bloques mail y domain
// ============================================================

/// Bloque de correo transaccional: `mail nombre { ... }`.
///
/// ```text
/// mail transaccional {
///     provider smtp
///     from "noreply@ejemplo.com"
///
///     template bienvenida {
///         subject "Bienvenido {{nombre}}"
///         vars [nombre, token]
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MailBlock {
    /// Nombre del bloque (identificador logico, e.g., `"transaccional"`).
    pub name: Spanned<std::string::String>,
    /// Proveedor de envio: `"smtp"`, `"resend"`, `"ses"`, `"postmark"`.
    pub provider: Option<std::string::String>,
    /// Direccion de remitente por defecto.
    pub from: Option<std::string::String>,
    /// Templates de correo definidos en este bloque.
    pub templates: Vec<MailTemplateDef>,
    /// Span del bloque completo.
    pub span: Span,
}

/// Definicion de template de correo dentro de un bloque `mail`.
///
/// ```text
/// template bienvenida {
///     subject "Bienvenido {{nombre}}"
///     vars [nombre, token]
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MailTemplateDef {
    /// Nombre del template (identificador, e.g., `"bienvenida"`).
    pub name: Spanned<std::string::String>,
    /// Asunto del correo (puede contener `{{var}}`).
    pub subject: Option<std::string::String>,
    /// Nombres de las variables que el template acepta.
    pub vars: Vec<std::string::String>,
    /// Span del bloque del template.
    pub span: Span,
}

/// Bloque de dominio DNS: `domain nombre { ... }`.
///
/// ```text
/// domain mi_dominio {
///     name "ejemplo.com"
///     provider cloudflare
///     dkim_selector "s1"
///     dmarc_policy quarantine
///     dmarc_rua "admin@ejemplo.com"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DomainBlock {
    /// Nombre logico del bloque (identificador, no el FQDN).
    pub name: Spanned<std::string::String>,
    /// FQDN del dominio (e.g., `"ejemplo.com"`).
    pub domain_name: Option<std::string::String>,
    /// Proveedor DNS: `"cloudflare"` por ahora.
    pub provider: Option<std::string::String>,
    /// Selectores DKIM activos.
    pub dkim_selectors: Vec<std::string::String>,
    /// Politica DMARC: `"none"`, `"quarantine"`, `"reject"`.
    pub dmarc_policy: Option<std::string::String>,
    /// Email destino de reportes DMARC.
    pub dmarc_rua: Option<std::string::String>,
    /// Span del bloque completo.
    pub span: Span,
}

#[cfg(test)]
mod ast_v05_v06_tests {
    use super::*;

    #[test]
    fn auth_mode_display() {
        assert_eq!(AuthMode::Required.as_str(), "required");
        assert_eq!(AuthMode::Optional.as_str(), "optional");
        assert_eq!(AuthMode::None.as_str(), "none");
    }

    #[test]
    fn event_def_fields() {
        let ev = EventDef {
            name: Spanned::new("user.created".to_string(), 0..12),
            payload: Spanned::new("UserResponse".to_string(), 0..12),
            retain_days: Some(30),
            span: 0..50,
        };
        assert_eq!(ev.name.value, "user.created");
        assert_eq!(ev.payload.value, "UserResponse");
        assert_eq!(ev.retain_days, Some(30));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
