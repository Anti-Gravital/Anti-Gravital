//! Analisis semantico del AST del Anti-DSL.
//!
//! Valida invariantes que el parser no puede verificar por ser
//! context-free: nombres duplicados, modelos sin campos, multiples
//! @primary, referencias a tipos inexistentes en endpoints, etc.
//! Produce diagnosticos de error y warning sin modificar el AST.

use std::collections::{HashMap, HashSet};

use crate::ast::{Annotation, FieldType, Schema};
use crate::diagnostics::Diagnostic;

/// Analiza el schema y retorna la lista de diagnosticos.
///
/// No modifica el AST. Los errores de severidad `Error` deben bloquear
/// la generacion de codigo en la capa superior.
pub fn analyze(schema: &Schema) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // v0.1 validaciones
    check_duplicate_model_names(schema, &mut diags);
    for model in &schema.models {
        check_model_has_fields(model, &mut diags);
        check_duplicate_field_names_in_model(model, &mut diags);
        check_primary_key(model, &mut diags);
        check_auto_type_compatibility(model, &mut diags);
        check_auto_update_type_compatibility(model, &mut diags);
    }

    // v0.2 validaciones
    check_duplicate_request_names(schema, &mut diags);
    check_duplicate_response_names(schema, &mut diags);
    check_duplicate_error_names(schema, &mut diags);
    check_duplicate_endpoint_names(schema, &mut diags);
    check_error_status_codes(schema, &mut diags);
    check_endpoint_references(schema, &mut diags);

    diags
}

fn check_duplicate_model_names(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for model in &schema.models {
        let name = model.name.value.as_str();
        if let Some(first_span_start) = seen.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                model.name.span.clone(),
                format!("nombre de modelo duplicado: '{name}'"),
                format!("el primer 'model {name}' esta en el byte {first_span_start}"),
            ));
        } else {
            seen.insert(name, model.name.span.start);
        }
    }
}

fn check_model_has_fields(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
    if model.fields.is_empty() {
        diags.push(Diagnostic::semantic_error_with_hint(
            model.span.clone(),
            format!("el modelo '{}' no tiene campos", model.name.value),
            "anade al menos un campo con su tipo",
        ));
    }
}

fn check_duplicate_field_names_in_model(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for field in &model.fields {
        let name = field.name.value.as_str();
        if let Some(first_start) = seen.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                field.name.span.clone(),
                format!(
                    "campo duplicado '{}' en el modelo '{}'",
                    name, model.name.value
                ),
                format!("el primer campo '{name}' esta en el byte {first_start}"),
            ));
        } else {
            seen.insert(name, field.name.span.start);
        }
    }
}

fn check_primary_key(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
    let primary_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| f.annotations.iter().any(|a| a.value == Annotation::Primary))
        .collect();

    if primary_fields.is_empty() {
        diags.push(Diagnostic::warning(
            model.span.clone(),
            format!("el modelo '{}' no tiene campo @primary", model.name.value),
        ));
    } else if primary_fields.len() > 1 {
        for field in &primary_fields[1..] {
            diags.push(Diagnostic::semantic_error_with_hint(
                field.name.span.clone(),
                format!("el modelo '{}' tiene mas de un @primary", model.name.value),
                "solo puede haber un campo @primary por modelo",
            ));
        }
    }
}

fn check_auto_type_compatibility(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
    let auto_compatible = [FieldType::Uuid, FieldType::Int, FieldType::Timestamp];
    for field in &model.fields {
        let has_auto = field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Auto);
        if has_auto && !auto_compatible.contains(&field.ty.value) {
            diags.push(Diagnostic::semantic_error_with_hint(
                field.name.span.clone(),
                format!(
                    "@auto en el campo '{}' no es compatible con el tipo {:?}",
                    field.name.value, field.ty.value
                ),
                "@auto solo es valido en campos UUID (uuid_generate_v4()), \
                 Int (SERIAL/SEQUENCE) y Timestamp (NOW())",
            ));
        }
    }
}

fn check_auto_update_type_compatibility(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
    for field in &model.fields {
        let has_auto_update = field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::AutoUpdate);
        if has_auto_update && field.ty.value != FieldType::Timestamp {
            diags.push(Diagnostic::semantic_error_with_hint(
                field.name.span.clone(),
                format!(
                    "@auto_update en el campo '{}' requiere tipo Timestamp",
                    field.name.value
                ),
                "cambia el tipo del campo a Timestamp",
            ));
        }
    }
}

// ---- Validaciones DSL v0.2 -------------------------------------------

fn check_duplicate_request_names(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for r in &schema.requests {
        let name = r.name.value.as_str();
        if let Some(first) = seen.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                r.name.span.clone(),
                format!("nombre de request duplicado: '{name}'"),
                format!("el primer 'request {name}' esta en el byte {first}"),
            ));
        } else {
            seen.insert(name, r.name.span.start);
        }
    }
}

fn check_duplicate_response_names(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for r in &schema.responses {
        let name = r.name.value.as_str();
        if let Some(first) = seen.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                r.name.span.clone(),
                format!("nombre de response duplicado: '{name}'"),
                format!("el primer 'response {name}' esta en el byte {first}"),
            ));
        } else {
            seen.insert(name, r.name.span.start);
        }
    }
}

fn check_duplicate_error_names(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for e in &schema.errors {
        let name = e.name.value.as_str();
        if let Some(first) = seen.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                e.name.span.clone(),
                format!("nombre de error duplicado: '{name}'"),
                format!("el primer 'error {name}' esta en el byte {first}"),
            ));
        } else {
            seen.insert(name, e.name.span.start);
        }
    }
}

fn check_duplicate_endpoint_names(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut seen_names: HashMap<&str, usize> = HashMap::new();
    let mut seen_routes: HashSet<String> = HashSet::new();
    for ep in &schema.endpoints {
        let name = ep.name.value.as_str();
        if let Some(first) = seen_names.get(name) {
            diags.push(Diagnostic::semantic_error_with_hint(
                ep.name.span.clone(),
                format!("nombre de endpoint duplicado: '{name}'"),
                format!("el primer 'endpoint {name}' esta en el byte {first}"),
            ));
        } else {
            seen_names.insert(name, ep.name.span.start);
        }
        let route = format!("{} {}", ep.method.value, ep.path.value);
        if seen_routes.contains(&route) {
            diags.push(Diagnostic::semantic_error_with_hint(
                ep.path.span.clone(),
                format!("ruta duplicada: {route}"),
                "dos endpoints no pueden tener el mismo metodo y path",
            ));
        } else {
            seen_routes.insert(route);
        }
    }
}

fn check_error_status_codes(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    for e in &schema.errors {
        let code = e.status.value;
        if !(400..=599).contains(&code) {
            diags.push(Diagnostic::semantic_error_with_hint(
                e.status.span.clone(),
                format!(
                    "codigo de estado HTTP invalido {code} en el error '{}'",
                    e.name.value
                ),
                "los errores deben usar codigos HTTP 4xx o 5xx",
            ));
        }
    }
}

fn check_endpoint_references(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let request_names: HashSet<&str> = schema
        .requests
        .iter()
        .map(|r| r.name.value.as_str())
        .collect();
    let response_names: HashSet<&str> = schema
        .responses
        .iter()
        .map(|r| r.name.value.as_str())
        .collect();
    let error_names: HashSet<&str> = schema
        .errors
        .iter()
        .map(|e| e.name.value.as_str())
        .collect();

    for ep in &schema.endpoints {
        if let Some(body) = &ep.body {
            if !request_names.contains(body.value.as_str()) {
                diags.push(Diagnostic::semantic_error_with_hint(
                    body.span.clone(),
                    format!(
                        "body '{}' en endpoint '{}' no esta definido",
                        body.value, ep.name.value
                    ),
                    format!("define 'request {} {{ ... }}' en el schema", body.value),
                ));
            }
        }
        if let Some(resp) = &ep.response {
            if !response_names.contains(resp.value.as_str()) {
                diags.push(Diagnostic::semantic_error_with_hint(
                    resp.span.clone(),
                    format!(
                        "response '{}' en endpoint '{}' no esta definido",
                        resp.value, ep.name.value
                    ),
                    format!("define 'response {} {{ ... }}' en el schema", resp.value),
                ));
            }
        }
        for err_ref in &ep.errors {
            if !error_names.contains(err_ref.value.as_str()) {
                diags.push(Diagnostic::semantic_error_with_hint(
                    err_ref.span.clone(),
                    format!(
                        "error '{}' en endpoint '{}' no esta definido",
                        err_ref.value, ep.name.value
                    ),
                    format!("define 'error {} {{ ... }}' en el schema", err_ref.value),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_tokens;

    fn compile(src: &str) -> (Option<Schema>, Vec<Diagnostic>) {
        let (tokens, lex_errors) = tokenize(src);
        let mut diags: Vec<Diagnostic> =
            lex_errors.into_iter().map(Diagnostic::lex_error).collect();
        let (ast, parse_diags) = parse_tokens(tokens, src.len());
        diags.extend(parse_diags);
        if let Some(ref schema) = ast {
            diags.extend(analyze(schema));
        }
        (ast, diags)
    }

    #[test]
    fn valid_model_no_errors() {
        let src = r#"
model Product {
    id    UUID   @primary @auto
    name  String @unique
    price Int
}
"#;
        let (_, diags) = compile(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn duplicate_model_name_is_error() {
        let src = r#"
model User { id UUID @primary @auto }
model User { id UUID @primary @auto }
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("duplicado")),
            "should report duplicate model error"
        );
    }

    #[test]
    fn duplicate_field_is_error() {
        let src = r#"
model User {
    id   UUID @primary @auto
    name String
    name String
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("campo duplicado")),);
    }

    #[test]
    fn model_without_primary_is_warning() {
        let src = r#"
model Tag {
    name String
}
"#;
        let (_, diags) = compile(src);
        let has_warning = diags
            .iter()
            .any(|d| !d.is_error() && d.message.contains("@primary"));
        assert!(has_warning, "should warn about missing @primary");
    }

    #[test]
    fn multiple_primary_is_error() {
        let src = r#"
model BadModel {
    id1 UUID @primary @auto
    id2 UUID @primary @auto
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("mas de un @primary")),);
    }

    #[test]
    fn auto_on_string_is_error() {
        let src = r#"
model Bad {
    id   UUID   @primary @auto
    name String @auto
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("@auto")),
            "should error on @auto with String type"
        );
    }

    #[test]
    fn auto_update_on_non_timestamp_is_error() {
        let src = r#"
model Bad {
    id      UUID @primary @auto
    counter Int  @auto_update
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("@auto_update")),);
    }

    #[test]
    fn empty_model_is_error() {
        let src = r#"model Empty {}"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("no tiene campos")),);
    }

    // ---- Tests DSL v0.2 ----

    #[test]
    fn valid_endpoint_no_errors() {
        let src = r#"
request CreateUserRequest { email String }
response UserResponse { id UUID }
error EmailTaken { status 409 message "Email taken" }
endpoint CreateUser {
    method   POST
    path     /users
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken]
}
"#;
        let (_, diags) = compile(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn endpoint_with_undefined_body_is_error() {
        let src = r#"
response UserResponse { id UUID }
endpoint Create {
    method   POST
    path     /users
    body     NonExistentRequest
    response UserResponse
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("body")),
            "should error on undefined body reference"
        );
    }

    #[test]
    fn endpoint_with_undefined_response_is_error() {
        let src = r#"
endpoint GetUser {
    method   GET
    path     /users/{id}
    response NonExistentResponse
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("response")),);
    }

    #[test]
    fn endpoint_with_undefined_error_ref_is_error() {
        let src = r#"
endpoint Create {
    method POST
    path   /items
    errors [UndefinedError]
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("error")),);
    }

    #[test]
    fn duplicate_route_is_error() {
        let src = r#"
endpoint A { method GET path /users }
endpoint B { method GET path /users }
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("duplicada")),);
    }

    #[test]
    fn invalid_error_status_code_is_error() {
        let src = r#"error BadCode { status 200 message "ok" }"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("invalido")),);
    }
}
