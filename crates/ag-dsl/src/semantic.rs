//! Analisis semantico del AST del Anti-DSL.
//!
//! Valida invariantes que el parser no puede verificar por ser
//! context-free: nombres duplicados, modelos sin campos, multiples
//! @primary, uso de @auto en tipos que no lo soportan, etc.
//! Produce diagnosticos de error y warning sin modificar el AST.

use std::collections::HashMap;

use crate::ast::{Annotation, FieldType, Schema};
use crate::diagnostics::Diagnostic;

/// Analiza el schema y retorna la lista de diagnosticos.
///
/// No modifica el AST. Los errores de severidad `Error` deben bloquear
/// la generacion de codigo en la capa superior.
pub fn analyze(schema: &Schema) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    check_duplicate_model_names(schema, &mut diags);

    for model in &schema.models {
        check_model_has_fields(model, &mut diags);
        check_duplicate_field_names(model, &mut diags);
        check_primary_key(model, &mut diags);
        check_auto_type_compatibility(model, &mut diags);
        check_auto_update_type_compatibility(model, &mut diags);
    }

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

fn check_duplicate_field_names(model: &crate::ast::ModelDef, diags: &mut Vec<Diagnostic>) {
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
}
