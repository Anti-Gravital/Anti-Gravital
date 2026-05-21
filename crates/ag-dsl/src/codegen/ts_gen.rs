//! Generador TypeScript para DSL v0.1.
//!
//! Produce interfaces TypeScript con tipos precisos para cada modelo.
//! Los campos @auto se incluyen como requeridos en la interfaz completa
//! y se excluyen en la interfaz de creacion.

use crate::ast::{Annotation, FieldDef, ModelDef, Schema};

/// Genera el contenido del archivo `clients/typescript/types.ts`.
pub fn generate_types(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("// Tipos generados por Anti-Gravital ag-dsl v0.1.\n");
    out.push_str("// NO editar manualmente. Regenerar con `ag generate`.\n\n");

    for model in &schema.models {
        out.push_str(&generate_model_interface(model));
        out.push('\n');
        out.push_str(&generate_create_interface(model));
        out.push('\n');
        out.push_str(&generate_update_interface(model));
        out.push('\n');
    }

    out
}

/// Interfaz completa del modelo.
fn generate_model_interface(model: &ModelDef) -> String {
    let name = &model.name.value;
    let mut out = String::new();

    out.push_str("/** Modelo completo incluyendo campos autogenerados. */\n");
    out.push_str(&format!("export interface {name} {{\n"));

    for field in &model.fields {
        let ts_ty = ts_field_type(field);
        let fname = &field.name.value;
        let optional_marker = if field.optional { "?" } else { "" };
        out.push_str(&format!("  {fname}{optional_marker}: {ts_ty};\n"));
    }

    out.push_str("}\n");
    out
}

/// Interfaz de creacion: excluye campos @auto.
fn generate_create_interface(model: &ModelDef) -> String {
    let name = &model.name.value;
    let create_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !is_auto_generated(f))
        .collect();

    if create_fields.is_empty() {
        return format!(
            "// Todos los campos de '{name}' son autogenerados; no hay Create interface.\n\n"
        );
    }

    let mut out = String::new();
    out.push_str("/** Cuerpo de la peticion POST para crear un nuevo registro. */\n");
    out.push_str(&format!("export interface Create{name}Request {{\n"));

    for field in create_fields {
        let ts_ty = ts_field_type(field);
        let fname = &field.name.value;
        let optional_marker = if field.optional { "?" } else { "" };
        out.push_str(&format!("  {fname}{optional_marker}: {ts_ty};\n"));
    }

    out.push_str("}\n");
    out
}

/// Interfaz de actualizacion: todos los campos no-auto son opcionales.
fn generate_update_interface(model: &ModelDef) -> String {
    let name = &model.name.value;
    let update_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !is_auto_generated(f) && !is_primary(f))
        .collect();

    if update_fields.is_empty() {
        return format!("// '{name}' no tiene campos actualizables.\n\n");
    }

    let mut out = String::new();
    out.push_str("/** Cuerpo de la peticion PUT/PATCH. Todos los campos son opcionales. */\n");
    out.push_str(&format!("export interface Update{name}Request {{\n"));

    for field in update_fields {
        let ts_ty = field.ty.value.ts_type();
        let fname = &field.name.value;
        out.push_str(&format!("  {fname}?: {ts_ty};\n"));
    }

    out.push_str("}\n");
    out
}

fn ts_field_type(field: &FieldDef) -> String {
    let base = field.ty.value.ts_type();
    if field.optional {
        format!("{base} | null")
    } else {
        base.to_owned()
    }
}

fn is_auto_generated(field: &FieldDef) -> bool {
    field
        .annotations
        .iter()
        .any(|a| matches!(a.value, Annotation::Auto | Annotation::AutoUpdate))
}

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
    fn generates_interfaces() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#,
        );
        let ts = generate_types(&schema);
        assert!(ts.contains("export interface User {"));
        assert!(ts.contains("export interface CreateUserRequest {"));
        assert!(ts.contains("export interface UpdateUserRequest {"));
        // id no debe aparecer en Create (es @auto)
        assert!(!ts.contains("export interface CreateUserRequest {\n  id:"));
    }

    #[test]
    fn optional_field_uses_null_union() {
        let schema = schema_from(
            r#"
model Profile {
    id  UUID   @primary @auto
    bio String?
}
"#,
        );
        let ts = generate_types(&schema);
        assert!(ts.contains("bio?: string | null;"));
    }

    #[test]
    fn uuid_maps_to_string() {
        let schema = schema_from(
            r#"
model Item {
    id UUID @primary @auto
}
"#,
        );
        let ts = generate_types(&schema);
        assert!(ts.contains("id: string;"));
    }
}
