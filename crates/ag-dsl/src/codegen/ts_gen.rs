//! Generador TypeScript para DSL v0.1–v0.2.
//!
//! Produce interfaces TypeScript para modelos, request/response types y
//! un cliente HTTP tipado por cada endpoint.

use crate::ast::{extract_path_params, Annotation, EndpointDef, FieldDef, ModelDef, Schema};

/// Genera el contenido del archivo `clients/typescript/types.ts`.
///
/// Incluye interfaces para modelos (v0.1), request/response/error types (v0.2)
/// y tipos de payload de eventos (v0.6).
pub fn generate_types(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("// Tipos generados por Anti-Gravital ag-dsl v0.1-v0.6.\n");
    out.push_str("// NO editar manualmente. Regenerar con `ag generate`.\n\n");

    for model in &schema.models {
        out.push_str(&generate_model_interface(model));
        out.push('\n');
        out.push_str(&generate_create_interface(model));
        out.push('\n');
        out.push_str(&generate_update_interface(model));
        out.push('\n');
    }

    for req in &schema.requests {
        out.push_str(&generate_simple_interface(&req.name.value, &req.fields));
        out.push('\n');
    }
    for resp in &schema.responses {
        out.push_str(&generate_simple_interface(&resp.name.value, &resp.fields));
        out.push('\n');
    }

    // v0.6 — tipos de payload de eventos
    if !schema.events.is_empty() {
        out.push_str("\n// Event payload types\n");
        for ev in &schema.events {
            // "user.created" -> "UserCreatedEvent"
            let type_name = ev
                .name
                .value
                .split('.')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                + "Event";
            out.push_str(&format!(
                "export type {} = {};\n",
                type_name, ev.payload.value
            ));
        }
    }

    out
}

/// Genera el contenido del archivo `clients/typescript/client.ts`.
///
/// Produce funciones tipadas para cada endpoint del schema.
pub fn generate_client(schema: &Schema) -> String {
    if schema.endpoints.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("// Cliente HTTP generado por Anti-Gravital ag-dsl v0.2.\n");
    out.push_str("// NO editar manualmente. Regenerar con `ag generate`.\n\n");
    out.push_str("const BASE_URL = '';\n\n");
    for ep in &schema.endpoints {
        out.push_str(&generate_api_function(ep));
        out.push('\n');
    }
    out
}

fn generate_api_function(ep: &EndpointDef) -> String {
    let fn_name = to_camel_case(&ep.name.value);
    let method = ep.method.value.as_str();
    let path = &ep.path.value;
    let path_params = extract_path_params(path);

    // Parametros de la funcion
    let mut params: Vec<String> = path_params.iter().map(|p| format!("{p}: string")).collect();
    if let Some(body) = &ep.body {
        params.push(format!("body: {}", body.value));
    }

    let return_ty = if let Some(resp) = &ep.response {
        format!("Promise<{}>", resp.value)
    } else {
        "Promise<void>".to_owned()
    };

    // Construye la URL con interpolacion de path params
    let url = if path_params.is_empty() {
        format!("`${{BASE_URL}}{path}`")
    } else {
        let mut url = format!("`${{BASE_URL}}{path}");
        // Reemplaza {param} por ${param} en la template string
        for p in &path_params {
            url = url.replace(&format!("{{{p}}}"), &format!("${{{p}}}"));
        }
        url.push('`');
        url
    };

    let needs_json = ep.body.is_some();
    let method_lower = method.to_lowercase();

    let mut out = String::new();
    out.push_str(&format!("/** {method} {path} */\n"));
    out.push_str(&format!(
        "export async function {fn_name}({}) : {return_ty} {{\n",
        params.join(", ")
    ));
    out.push_str(&format!("  const resp = await fetch({url}, {{\n"));
    out.push_str(&format!("    method: '{method_lower}',\n"));
    if needs_json {
        out.push_str("    headers: { 'Content-Type': 'application/json' },\n");
        out.push_str("    body: JSON.stringify(body),\n");
    }
    out.push_str("  });\n");
    out.push_str(&format!(
        "  if (!resp.ok) throw new Error(`{fn_name} failed: ${{resp.status}}`);\n"
    ));
    if ep.response.is_some() {
        out.push_str("  return resp.json();\n");
    }
    out.push_str("}\n");
    out
}

/// Interfaz simple sin variantes Create/Update (para request/response types).
fn generate_simple_interface(name: &str, fields: &[FieldDef]) -> String {
    let mut out = String::new();
    out.push_str(&format!("/** API type `{name}`. */\n"));
    out.push_str(&format!("export interface {name} {{\n"));
    for field in fields {
        let ts_ty = ts_field_type(field);
        let fname = &field.name.value;
        let opt = if field.optional { "?" } else { "" };
        out.push_str(&format!("  {fname}{opt}: {ts_ty};\n"));
    }
    out.push_str("}\n");
    out
}

/// Convierte PascalCase a camelCase para nombres de funcion TypeScript.
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// Interfaz completa del modelo.
fn generate_model_interface(model: &ModelDef) -> String {
    let name = &model.name.value;
    let mut out = String::new();

    out.push_str("/** Modelo completo incluyendo campos autogenerados. */\n");
    out.push_str(&format!("export interface {name} {{\n"));

    for field in &model.fields {
        let fname = &field.name.value;
        if field.virtual_field {
            let ts_ty = match &field.ty.value {
                crate::ast::FieldType::ModelRef(m) => m.clone(),
                crate::ast::FieldType::ModelRefList(m) => format!("{m}[]"),
                _ => continue,
            };
            out.push_str(&format!("  {fname}?: {ts_ty};\n"));
        } else {
            let ts_ty = ts_field_type(field);
            let optional_marker = if field.optional { "?" } else { "" };
            out.push_str(&format!("  {fname}{optional_marker}: {ts_ty};\n"));
        }
    }

    out.push_str("}\n");
    out
}

/// Interfaz de creacion: excluye campos @auto y virtuales.
fn generate_create_interface(model: &ModelDef) -> String {
    let name = &model.name.value;
    let create_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !is_auto_generated(f) && !f.virtual_field)
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
        .filter(|f| !is_auto_generated(f) && !is_primary(f) && !f.virtual_field)
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
    fn ts_gen_emits_event_payload_types() {
        let src = r#"
event user.created { payload UserResponse }
response UserResponse { id UUID }
"#;
        let schema = crate::compile(src).unwrap();
        let files = crate::generate(&schema);
        let ts = files.files.iter()
            .find(|(p, _)| p.extension().map(|e| e == "ts").unwrap_or(false))
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        assert!(
            ts.contains("UserCreatedEvent") || ts.contains("user_created"),
            "debe emitir tipo de evento, got:\n{}", ts
        );
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

    // ---- Tests DSL v0.4 ----

    #[test]
    fn v04_ts_relation_types() {
        let schema = schema_from(
            r#"
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
"#,
        );
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
