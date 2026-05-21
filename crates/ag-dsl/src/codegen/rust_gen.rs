//! Generador Rust para DSL v0.1.
//!
//! Produce structs Rust con serde::{Serialize, Deserialize} a partir de
//! los modelos del schema. Por cada modelo genera:
//!
//! - `NombreModel` — struct completo (para respuestas y persistencia).
//! - `CreateNombreRequest` — sin campos @auto (para POST).
//! - `UpdateNombreRequest` — todos los campos no-@auto como Option<T> (para PUT/PATCH).

use crate::ast::{Annotation, FieldDef, FieldType, ModelDef, Schema};

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
        let rust_ty = rust_field_type(field);
        let fname = &field.name.value;
        // UUID usa serde rename cuando el nombre de campo es 'id'
        if field.ty.value == FieldType::Uuid && fname == "id" {
            out.push_str("    pub id: Uuid,\n");
        } else {
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
        .filter(|f| !is_auto_generated(f))
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
        .filter(|f| !is_auto_generated(f) && !is_primary(f))
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
