//! Generador Rust para DSL v0.1–v0.2.
//!
//! Produce structs Rust con serde a partir de los modelos del schema (v0.1),
//! y tipos API + handler stubs + router a partir de endpoints (v0.2).

use crate::ast::{
    extract_path_params, to_snake_case, Annotation, AuthMode, EndpointDef, FieldDef, FieldType,
    HttpMethod, ModelDef, RequestDef, ResponseDef, Schema,
};

/// Genera el contenido del archivo `src/models.rs` para el schema dado.
pub fn generate_models(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("//! Modelos generados por Anti-Gravital ag-dsl v0.1.\n");
    out.push_str("//! NO editar manualmente. Regenerar con `ag generate`.\n\n");
    out.push_str("#![allow(dead_code)]\n\n");

    // Imports base
    let needs_uuid = schema
        .models
        .iter()
        .any(|m| m.fields.iter().any(|f| f.ty.value == FieldType::Uuid));
    let needs_chrono = schema
        .models
        .iter()
        .any(|m| m.fields.iter().any(|f| f.ty.value == FieldType::Timestamp));
    let needs_decimal = schema
        .models
        .iter()
        .any(|m| m.fields.iter().any(|f| f.ty.value == FieldType::Decimal));

    out.push_str("use serde::{Deserialize, Serialize};\n");
    if needs_uuid {
        out.push_str("use uuid::Uuid;\n");
    }
    if needs_chrono {
        out.push_str("use chrono::{DateTime, Utc};\n");
    }
    if needs_decimal {
        out.push_str("use rust_decimal::Decimal;\n");
    }
    out.push('\n');

    for model in &schema.models {
        out.push_str(&generate_model_struct(model));
        out.push('\n');
        out.push_str(&generate_create_request(model));
        out.push('\n');
        out.push_str(&generate_update_request(model));
        out.push('\n');
    }

    out
}

/// Struct completo del modelo (todos los campos).
fn generate_model_struct(model: &ModelDef) -> String {
    let name = &model.name.value;
    let mut out = String::new();

    out.push_str("/// Modelo completo (incluye campos generados automaticamente).\n");
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {name} {{\n"));

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

    out.push_str("}\n");
    out
}

/// Request de creacion: excluye campos @auto y @auto_update.
fn generate_create_request(model: &ModelDef) -> String {
    let name = &model.name.value;
    let create_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !is_auto_generated(f) && !f.virtual_field)
        .collect();

    if create_fields.is_empty() {
        // Todos los campos son auto; no tiene sentido un create request.
        return format!(
            "/// Todos los campos de '{name}' son autogenerados.\n\
             /// No se necesita CreateRequest.\n\n"
        );
    }

    let mut out = String::new();
    out.push_str("/// Cuerpo de la peticion POST para crear un nuevo registro.\n");
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct Create{name}Request {{\n"));

    for field in create_fields {
        let rust_ty = rust_field_type(field);
        let fname = &field.name.value;
        out.push_str(&format!("    pub {fname}: {rust_ty},\n"));
    }

    out.push_str("}\n");
    out
}

/// Request de actualizacion: todos los campos no-auto son Option<T>.
fn generate_update_request(model: &ModelDef) -> String {
    let name = &model.name.value;
    let update_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !is_auto_generated(f) && !is_primary(f) && !f.virtual_field)
        .collect();

    if update_fields.is_empty() {
        return format!("/// '{name}' no tiene campos actualizables.\n\n");
    }

    let mut out = String::new();
    out.push_str("/// Cuerpo de la peticion PUT/PATCH para actualizar un registro.\n");
    out.push_str("/// Los campos son opcionales: solo se actualizan los presentes.\n");
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize, Default)]\n");
    out.push_str(&format!("pub struct Update{name}Request {{\n"));

    for field in update_fields {
        let base_ty = field.ty.value.rust_type(false);
        let fname = &field.name.value;
        out.push_str(&format!("    pub {fname}: Option<{base_ty}>,\n"));
    }

    out.push_str("}\n");
    out
}

/// Retorna el tipo Rust del campo, considerando opcionalidad.
fn rust_field_type(field: &FieldDef) -> String {
    // Campos @auto o @auto_update son siempre presentes en la DB,
    // pero en los structs de respuesta se incluyen como requeridos.
    field.ty.value.rust_type(field.optional)
}

/// True si el campo tiene @auto o @auto_update.
fn is_auto_generated(field: &FieldDef) -> bool {
    field
        .annotations
        .iter()
        .any(|a| matches!(a.value, Annotation::Auto | Annotation::AutoUpdate))
}

/// True si el campo tiene @primary.
fn is_primary(field: &FieldDef) -> bool {
    field
        .annotations
        .iter()
        .any(|a| a.value == Annotation::Primary)
}

// ---- Generadores DSL v0.2 -------------------------------------------

/// Genera el contenido de `src/types.rs`: structs para request, response y error.
///
/// Retorna cadena vacia si el schema no define ningun tipo API.
pub fn generate_types(schema: &Schema) -> String {
    if schema.requests.is_empty() && schema.responses.is_empty() && schema.errors.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("//! Tipos de API generados por Anti-Gravital ag-dsl v0.2.\n");
    out.push_str("//! NO editar manualmente. Regenerar con `ag generate`.\n\n");
    out.push_str("#![allow(dead_code)]\n\n");
    out.push_str("use serde::{Deserialize, Serialize};\n");

    let needs_uuid = schema
        .requests
        .iter()
        .any(|r| r.fields.iter().any(|f| f.ty.value == FieldType::Uuid))
        || schema
            .responses
            .iter()
            .any(|r| r.fields.iter().any(|f| f.ty.value == FieldType::Uuid));
    if needs_uuid {
        out.push_str("use uuid::Uuid;\n");
    }
    out.push('\n');

    for req in &schema.requests {
        out.push_str(&generate_request_struct(req));
        out.push('\n');
    }
    for resp in &schema.responses {
        out.push_str(&generate_response_struct(resp));
        out.push('\n');
    }
    for err in &schema.errors {
        out.push_str(&format!(
            "/// Error HTTP {}: {}\n",
            err.status.value, err.message.value
        ));
        out.push_str(&format!(
            "pub const {}_STATUS: u16 = {};\n",
            to_snake_case(&err.name.value).to_uppercase(),
            err.status.value
        ));
        out.push_str(&format!(
            "pub const {}_MESSAGE: &str = \"{}\";\n\n",
            to_snake_case(&err.name.value).to_uppercase(),
            err.message.value
        ));
    }
    out
}

fn generate_request_struct(req: &RequestDef) -> String {
    let name = &req.name.value;
    let mut out = String::new();
    out.push_str(&format!("/// Cuerpo de peticion para `{name}`.\n"));
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {name} {{\n"));
    for field in &req.fields {
        let rust_ty = field.ty.value.rust_type(field.optional);
        let fname = &field.name.value;
        out.push_str(&format!("    pub {fname}: {rust_ty},\n"));
    }
    out.push_str("}\n");
    out.push_str(&generate_validate_impl(name, &req.fields));
    out
}

fn generate_response_struct(resp: &ResponseDef) -> String {
    let name = &resp.name.value;
    let mut out = String::new();
    out.push_str(&format!("/// Cuerpo de respuesta para `{name}`.\n"));
    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {name} {{\n"));
    for field in &resp.fields {
        let rust_ty = field.ty.value.rust_type(field.optional);
        let fname = &field.name.value;
        out.push_str(&format!("    pub {fname}: {rust_ty},\n"));
    }
    out.push_str("}\n");
    out
}

/// Genera el contenido de `src/handlers.rs`: stubs de handlers Axum.
pub fn generate_handlers(schema: &Schema) -> String {
    if schema.endpoints.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("//! Handler stubs generados por Anti-Gravital ag-dsl v0.2.\n");
    out.push_str("//! Implementa el cuerpo de cada funcion para completar la API.\n\n");
    out.push_str("#![allow(unused_variables, clippy::unused_async)]\n\n");
    out.push_str("use axum::{extract::{Path, State}, Json};\n\n");
    for ep in &schema.endpoints {
        out.push_str(&generate_handler_stub(ep));
        out.push('\n');
    }
    out
}

fn generate_handler_stub(ep: &EndpointDef) -> String {
    let fn_name = to_snake_case(&ep.name.value);
    let method_str = ep.method.value.as_str();
    let path_str = &ep.path.value;
    let path_params = extract_path_params(path_str);

    let mut params = vec!["State(state): State<AppState>".to_owned()];
    for param in &path_params {
        params.push(format!("Path({param}): Path<String>"));
    }
    // v0.5 — Claims extractor cuando el endpoint requiere autenticacion
    if ep.auth != AuthMode::None {
        params.push("Claims(claims): Claims<serde_json::Value>".to_owned());
    }
    if let Some(body) = &ep.body {
        params.push(format!("Json(body): Json<{}>", body.value));
    }

    let return_ty = if let Some(resp) = &ep.response {
        format!("Result<Json<{}>, axum::http::StatusCode>", resp.value)
    } else {
        "axum::http::StatusCode".to_owned()
    };

    let mut out = String::new();
    out.push_str(&format!(
        "/// {method_str} {path_str} — {}\n",
        ep.name.value
    ));
    out.push_str(&format!(
        "pub async fn {fn_name}(\n    {}\n) -> {return_ty} {{\n",
        params.join(",\n    ")
    ));
    // v0.6 — stubs para eventos declarados en el endpoint
    for event in &ep.emits {
        out.push_str(&format!(
            "    // TODO: state.events.emit(\"{}\", &payload).await?;\n",
            event.value
        ));
    }
    out.push_str("    todo!()\n}\n");
    out
}

/// Genera el contenido de `src/router.rs`: registro de rutas Axum.
pub fn generate_router(schema: &Schema) -> String {
    if schema.endpoints.is_empty() {
        return String::new();
    }
    use crate::ast::dsl_path_to_axum;

    let mut out = String::new();
    out.push_str("//! Router generado por Anti-Gravital ag-dsl v0.2.\n\n");
    out.push_str("use axum::routing;\n");
    out.push_str("use super::handlers;\n\n");
    out.push_str("/// Placeholder para el estado de la aplicacion.\n");
    out.push_str("pub type AppState = ();\n\n");
    out.push_str("/// Crea el router Axum con todas las rutas definidas en el schema.\n");
    out.push_str("pub fn api_router() -> axum::Router<AppState> {\n");
    out.push_str("    axum::Router::new()\n");

    for ep in &schema.endpoints {
        let fn_name = to_snake_case(&ep.name.value);
        let axum_path = dsl_path_to_axum(&ep.path.value);
        let axum_method = match ep.method.value {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
        };
        out.push_str(&format!(
            "        .route(\"{axum_path}\", routing::{axum_method}(handlers::{fn_name}))\n"
        ));
    }
    out.push_str("}\n");
    out
}

// ---- Validacion v0.3 — metodo validate() ----------------------------

/// Genera un bloque `impl NombreStruct { pub fn validate(...) }` con
/// todas las comprobaciones derivadas de las anotaciones de validacion.
/// Retorna cadena vacia si no hay anotaciones de validacion en los campos.
fn generate_validate_impl(struct_name: &str, fields: &[FieldDef]) -> String {
    use crate::ast::{Annotation, FieldType};

    let mut checks: Vec<String> = Vec::new();

    for field in fields {
        let fname = &field.name.value;
        let is_string = field.ty.value == FieldType::String;
        let is_numeric = matches!(
            field.ty.value,
            FieldType::Int | FieldType::Float | FieldType::Decimal
        );

        for ann in &field.annotations {
            match &ann.value {
                Annotation::Min(n) if is_string => checks.push(format!(
                    "    if self.{fname}.len() < {n} {{\n\
                             errors.push(format!(\"{fname}: longitud minima es {n}, encontrado {{}} caracteres\", self.{fname}.len()));\n\
                     }}"
                )),
                Annotation::Max(n) if is_string => checks.push(format!(
                    "    if self.{fname}.len() > {n} {{\n\
                             errors.push(format!(\"{fname}: longitud maxima es {n}, encontrado {{}} caracteres\", self.{fname}.len()));\n\
                     }}"
                )),
                Annotation::Min(n) if is_numeric => checks.push(format!(
                    "    if (self.{fname} as i64) < {n} {{\n\
                             errors.push(\"{fname}: valor minimo es {n}\".to_owned());\n\
                     }}"
                )),
                Annotation::Max(n) if is_numeric => checks.push(format!(
                    "    if (self.{fname} as i64) > {n} {{\n\
                             errors.push(\"{fname}: valor maximo es {n}\".to_owned());\n\
                     }}"
                )),
                Annotation::Email => checks.push(format!(
                    "    {{\n\
                             let s = &self.{fname};\n\
                             let valid = s.contains('@') && s.rfind('@')\n\
                                 .map(|i| s[i+1..].contains('.'))\n\
                                 .unwrap_or(false);\n\
                             if !valid {{\n\
                                 errors.push(\"{fname}: formato de email invalido\".to_owned());\n\
                             }}\n\
                     }}"
                )),
                Annotation::Length(n) => checks.push(format!(
                    "    if self.{fname}.len() != {n} {{\n\
                             errors.push(format!(\"{fname}: longitud exacta requerida es {n}, encontrado {{}} caracteres\", self.{fname}.len()));\n\
                     }}"
                )),
                Annotation::Regex(pattern) => checks.push(format!(
                    "    // TODO: validar {fname} contra regex: {pattern:?}\n\
                     // Añade la crate `regex` y verifica: Regex::new({pattern:?}).unwrap().is_match(&self.{fname})"
                )),
                _ => {}
            }
        }
    }

    if checks.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\nimpl {struct_name} {{\n\
         /// Valida todas las restricciones declaradas en el schema DSL.\n\
         /// Retorna la lista de errores; vacio significa valido.\n\
         pub fn validate(&self) -> Vec<String> {{\n\
             let mut errors: Vec<String> = Vec::new();\n"
    ));
    for check in checks {
        out.push_str(&check);
        out.push('\n');
    }
    out.push_str("    errors\n}\n}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_tokens;

    fn schema_from(src: &str) -> Schema {
        let (tokens, _) = tokenize(src);
        let (ast, _) = parse_tokens(tokens, src.len());
        ast.expect("valid schema")
    }

    #[test]
    fn rust_gen_includes_claims_when_auth_required() {
        let src = r#"
endpoint GetProfile {
    method GET
    path /profile
    auth required
    response UserResponse
}
response UserResponse { id UUID }
"#;
        let schema = crate::compile(src).unwrap();
        let files = crate::generate(&schema);
        let handlers = files.files.iter()
            .find(|(p, _)| p.to_str().unwrap_or("").contains("handlers"))
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        assert!(handlers.contains("Claims"), "debe incluir Claims cuando auth required, got:\n{}", handlers);
    }

    #[test]
    fn rust_gen_includes_event_stub_in_handler() {
        let src = r#"
event user.created { payload UserResponse }
endpoint CreateUser {
    method POST
    path /users
    auth required
    events [user.created]
}
response UserResponse { id UUID }
"#;
        let schema = crate::compile(src).unwrap();
        let files = crate::generate(&schema);
        let handlers = files.files.iter()
            .find(|(p, _)| p.to_str().unwrap_or("").contains("handlers"))
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        assert!(handlers.contains("user.created"), "debe incluir stub de evento, got:\n{}", handlers);
    }

    #[test]
    fn generates_valid_struct() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#,
        );
        let out = generate_models(&schema);
        assert!(out.contains("pub struct User {"));
        assert!(out.contains("pub struct CreateUserRequest {"));
        assert!(out.contains("pub struct UpdateUserRequest {"));
        // El campo id (@auto) no debe estar en CreateRequest
        assert!(!out.contains("pub struct CreateUserRequest {\n    pub id:"));
    }

    #[test]
    fn optional_field_in_update_request() {
        let schema = schema_from(
            r#"
model Post {
    id      UUID   @primary @auto
    title   String
    content String?
}
"#,
        );
        let out = generate_models(&schema);
        assert!(out.contains("pub title: Option<String>,"));
        assert!(out.contains("pub content: Option<String>,"));
    }

    #[test]
    fn uuid_import_included() {
        let schema = schema_from(
            r#"
model Item {
    id UUID @primary @auto
    v  Int
}
"#,
        );
        let out = generate_models(&schema);
        assert!(out.contains("use uuid::Uuid;"));
    }

    // ---- Tests DSL v0.4 ----

    #[test]
    fn v04_model_ref_generates_option() {
        let schema = schema_from(
            r#"
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
"#,
        );
        let out = generate_models(&schema);
        assert!(
            out.contains("pub author: Option<User>"),
            "N:1 virtual field should be Option<User>. Output:\n{out}"
        );
        assert!(
            out.contains("pub author_id:"),
            "FK field must be in struct. Output:\n{out}"
        );
    }

    #[test]
    fn v04_model_ref_list_generates_vec() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
}
"#,
        );
        let out = generate_models(&schema);
        assert!(
            out.contains("pub posts: Vec<Post>"),
            "1:N virtual field should be Vec<Post>. Output:\n{out}"
        );
    }

    #[test]
    fn v04_virtual_field_excluded_from_create_request() {
        let schema = schema_from(
            r#"
model Post {
    id        UUID @primary @auto
    title     String
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
model User {
    id UUID @primary @auto
}
"#,
        );
        let out = generate_models(&schema);
        let create_start = out.find("pub struct CreatePostRequest").unwrap_or(0);
        let create_end = out[create_start..]
            .find('}')
            .unwrap_or(out.len() - create_start)
            + create_start;
        let create_body = &out[create_start..create_end];
        assert!(
            !create_body.contains("author:"),
            "virtual field should not appear in CreateRequest. Body:\n{create_body}"
        );
    }

    #[test]
    fn no_uuid_import_when_not_needed() {
        let schema = schema_from(
            r#"
model Counter {
    id    Int    @primary @auto
    value Int
}
"#,
        );
        let out = generate_models(&schema);
        assert!(!out.contains("use uuid::Uuid;"));
    }
}
