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

    // v0.3 validaciones de anotaciones
    for model in &schema.models {
        for field in &model.fields {
            check_validation_annotations(
                &field.name.value,
                &model.name.value,
                &field.ty.value,
                &field.annotations,
                &mut diags,
            );
        }
    }
    for req in &schema.requests {
        for field in &req.fields {
            check_validation_annotations(
                &field.name.value,
                &req.name.value,
                &field.ty.value,
                &field.annotations,
                &mut diags,
            );
        }
    }
    for resp in &schema.responses {
        for field in &resp.fields {
            check_validation_annotations(
                &field.name.value,
                &resp.name.value,
                &field.ty.value,
                &field.annotations,
                &mut diags,
            );
        }
    }

    // v0.4 validaciones de relaciones
    check_references_model_exists(schema, &mut diags);
    check_references_to_primary(schema, &mut diags);
    check_model_ref_has_relation(schema, &mut diags);
    check_relation_fk_field_exists(schema, &mut diags);
    check_circular_fk(schema, &mut diags);

    // v0.5 validaciones — policy requiere auth != None
    for ep in &schema.endpoints {
        if let Some(ref policy_spanned) = ep.policy {
            if ep.auth == crate::ast::AuthMode::None {
                diags.push(Diagnostic::semantic_error_with_hint(
                    policy_spanned.span.clone(),
                    format!(
                        "policy '{}' en endpoint '{}' solo es valida cuando auth es 'required' u 'optional'",
                        policy_spanned.value, ep.name.value
                    ),
                    "agrega 'auth required' al endpoint o elimina la clausula policy",
                ));
            }
        }
    }

    // v0.6 validaciones — eventos referenciados deben estar declarados
    let declared_event_names: HashSet<&str> =
        schema.events.iter().map(|e| e.name.value.as_str()).collect();

    for ep in &schema.endpoints {
        for ev_ref in &ep.emits {
            if !declared_event_names.contains(ev_ref.value.as_str()) {
                diags.push(Diagnostic::semantic_error_with_hint(
                    ev_ref.span.clone(),
                    format!(
                        "evento '{}' referenciado en endpoint '{}' no esta declarado en el schema",
                        ev_ref.value, ep.name.value
                    ),
                    format!(
                        "agrega 'event {} {{ payload T }}' al schema",
                        ev_ref.value
                    ),
                ));
            }
        }
    }

    // v0.6 validaciones — nombre de evento debe contener punto (entidad.accion)
    for ev in &schema.events {
        if !ev.name.value.contains('.') {
            diags.push(Diagnostic::warning(
                ev.name.span.clone(),
                format!(
                    "el nombre de evento '{}' no sigue el patron entidad.accion",
                    ev.name.value
                ),
            ));
        }
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

// ---- Validaciones DSL v0.3 -------------------------------------------

/// Verifica compatibilidad de las anotaciones de validacion con el tipo del campo.
fn check_validation_annotations(
    field_name: &str,
    parent_name: &str,
    field_ty: &FieldType,
    annotations: &[crate::ast::Spanned<Annotation>],
    diags: &mut Vec<Diagnostic>,
) {
    use FieldType::{Bool, Decimal, Float, Int, String as Str, Timestamp, Uuid};

    let string_only = [&Annotation::Email];
    let string_and_numeric = [Str, Int, Float, Decimal];

    let mut min_val: Option<(i64, crate::lexer::Span)> = None;
    let mut max_val: Option<(i64, crate::lexer::Span)> = None;

    for ann in annotations {
        match &ann.value {
            Annotation::Email | Annotation::Regex(_) | Annotation::Length(_) => {
                if *field_ty != Str {
                    diags.push(Diagnostic::semantic_error_with_hint(
                        ann.span.clone(),
                        format!(
                            "{} en '{}.{}': solo aplica a campos de tipo String",
                            annotation_name(&ann.value),
                            parent_name,
                            field_name
                        ),
                        "cambia el tipo del campo a String".to_owned(),
                    ));
                }
                if let Annotation::Length(n) = ann.value {
                    if n <= 0 {
                        diags.push(Diagnostic::semantic_error_with_hint(
                            ann.span.clone(),
                            format!(
                                "@length({n}) en '{}.{}': el valor debe ser mayor que cero",
                                parent_name, field_name
                            ),
                            "usa @length con un entero positivo",
                        ));
                    }
                }
            }
            Annotation::Min(n) => {
                if !string_and_numeric.contains(field_ty) {
                    diags.push(Diagnostic::semantic_error_with_hint(
                        ann.span.clone(),
                        format!(
                            "@min en '{}.{}': no aplica al tipo {:?}",
                            parent_name, field_name, field_ty
                        ),
                        "@min es valido en String, Int, Float y Decimal",
                    ));
                }
                min_val = Some((*n, ann.span.clone()));
            }
            Annotation::Max(n) => {
                if !string_and_numeric.contains(field_ty) {
                    diags.push(Diagnostic::semantic_error_with_hint(
                        ann.span.clone(),
                        format!(
                            "@max en '{}.{}': no aplica al tipo {:?}",
                            parent_name, field_name, field_ty
                        ),
                        "@max es valido en String, Int, Float y Decimal",
                    ));
                }
                max_val = Some((*n, ann.span.clone()));
            }
            _ => {}
        }
    }

    // @min debe ser <= @max cuando ambos estan presentes
    if let (Some((min_n, _)), Some((max_n, max_span))) = (min_val, max_val) {
        if min_n > max_n {
            diags.push(Diagnostic::semantic_error_with_hint(
                max_span,
                format!(
                    "@min({min_n}) > @max({max_n}) en '{}.{}': el minimo supera al maximo",
                    parent_name, field_name
                ),
                format!("usa @min({min_n}) @max(N) con N >= {min_n}"),
            ));
        }
    }

    // Suprimir advertencias de unused variables
    let _ = (string_only, Bool, Timestamp, Uuid);
}

// ---- Validaciones DSL v0.4 — relaciones ---------------------------------

fn check_references_model_exists(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let model_names: HashSet<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::References {
                    model: ref_model, ..
                } = &ann.value
                {
                    if !model_names.contains(ref_model.as_str()) {
                        diags.push(Diagnostic::semantic_error_with_hint(
                            ann.span.clone(),
                            format!(
                                "el modelo '{}' referenciado en @references no esta definido",
                                ref_model
                            ),
                            format!("define 'model {} {{ ... }}' en el schema", ref_model),
                        ));
                    }
                }
            }
        }
    }
}

fn check_references_to_primary(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let primary_fields: HashMap<&str, HashSet<&str>> = schema
        .models
        .iter()
        .map(|m| {
            let primaries = m
                .fields
                .iter()
                .filter(|f| f.annotations.iter().any(|a| a.value == Annotation::Primary))
                .map(|f| f.name.value.as_str())
                .collect();
            (m.name.value.as_str(), primaries)
        })
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::References {
                    model: ref_model,
                    field: ref_field,
                } = &ann.value
                {
                    if let Some(primaries) = primary_fields.get(ref_model.as_str()) {
                        if !primaries.contains(ref_field.as_str()) {
                            diags.push(Diagnostic::warning(
                                ann.span.clone(),
                                format!(
                                    "se recomienda referenciar el campo @primary de '{}', no '{}'",
                                    ref_model, ref_field
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn check_model_ref_has_relation(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    for model in &schema.models {
        for field in &model.fields {
            let is_model_ref = matches!(
                field.ty.value,
                FieldType::ModelRef(_) | FieldType::ModelRefList(_)
            );
            if is_model_ref {
                let has_relation = field
                    .annotations
                    .iter()
                    .any(|a| matches!(a.value, Annotation::Relation { .. }));
                if !has_relation {
                    diags.push(Diagnostic::semantic_error_with_hint(
                        field.name.span.clone(),
                        format!(
                            "campo '{}' de tipo relacion requiere anotacion @relation",
                            field.name.value
                        ),
                        "agrega @relation(campo_fk) para campos de tipo modelo",
                    ));
                }
            }
        }
    }
}

fn check_relation_fk_field_exists(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let model_fk_fields: HashMap<&str, HashSet<&str>> = schema
        .models
        .iter()
        .map(|m| {
            let fks = m
                .fields
                .iter()
                .filter(|f| {
                    !f.virtual_field
                        && f.annotations
                            .iter()
                            .any(|a| matches!(a.value, Annotation::References { .. }))
                })
                .map(|f| f.name.value.as_str())
                .collect();
            (m.name.value.as_str(), fks)
        })
        .collect();

    let all_model_names: HashSet<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    for model in &schema.models {
        for field in &model.fields {
            for ann in &field.annotations {
                if let Annotation::Relation { path } = &ann.value {
                    if !path.contains('.') {
                        let fks = model_fk_fields
                            .get(model.name.value.as_str())
                            .cloned()
                            .unwrap_or_default();
                        if !fks.contains(path.as_str()) {
                            diags.push(Diagnostic::semantic_error_with_hint(
                                ann.span.clone(),
                                format!(
                                    "el campo '{}' en @relation no existe o no tiene @references en el modelo '{}'",
                                    path, model.name.value
                                ),
                                format!(
                                    "agrega '{path} UUID @references(Modelo.id)' al modelo '{}'",
                                    model.name.value
                                ),
                            ));
                        }
                    } else {
                        let ref_model = path.split('.').next().unwrap_or("");
                        if !ref_model.is_empty() && !all_model_names.contains(ref_model) {
                            diags.push(Diagnostic::semantic_error_with_hint(
                                ann.span.clone(),
                                format!("el modelo '{}' en @relation no esta definido", ref_model),
                                format!("define 'model {} {{ ... }}' en el schema", ref_model),
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn check_circular_fk(schema: &Schema, diags: &mut Vec<Diagnostic>) {
    let mut fk_targets: HashMap<&str, HashSet<&str>> = HashMap::new();

    for model in &schema.models {
        let entry = fk_targets.entry(model.name.value.as_str()).or_default();
        for field in &model.fields {
            if field.virtual_field {
                continue;
            }
            for ann in &field.annotations {
                if let Annotation::References {
                    model: ref_model, ..
                } = &ann.value
                {
                    entry.insert(ref_model.as_str());
                }
            }
        }
    }

    let model_names: Vec<&str> = schema
        .models
        .iter()
        .map(|m| m.name.value.as_str())
        .collect();

    let mut reported: HashSet<(&str, &str)> = HashSet::new();

    for &a in &model_names {
        for &b in &model_names {
            if a == b {
                continue;
            }
            let a_to_b = fk_targets.get(a).is_some_and(|s| s.contains(b));
            let b_to_a = fk_targets.get(b).is_some_and(|s| s.contains(a));

            if a_to_b && b_to_a && !reported.contains(&(b, a)) {
                reported.insert((a, b));

                let span = schema
                    .models
                    .iter()
                    .find(|m| m.name.value == a)
                    .and_then(|m| {
                        m.fields.iter().find(|f| {
                            !f.virtual_field
                                && f.annotations.iter().any(|ann| {
                                    matches!(&ann.value,
                                        Annotation::References { model, .. } if model == b)
                                })
                        })
                    })
                    .and_then(|f| {
                        f.annotations.iter().find(|ann| {
                            matches!(&ann.value,
                                Annotation::References { model, .. } if model == b)
                        })
                    })
                    .map(|ann| ann.span.clone())
                    .unwrap_or(0..0);

                diags.push(Diagnostic::semantic_error_with_hint(
                    span,
                    format!(
                        "referencia circular entre modelos '{}' y '{}': ambos tienen FK hacia el otro",
                        a, b
                    ),
                    "elimina una de las FKs o convierte una en campo virtual con @relation",
                ));
            }
        }
    }
}

fn annotation_name(ann: &Annotation) -> &'static str {
    match ann {
        Annotation::Email => "@email",
        Annotation::Regex(_) => "@regex",
        Annotation::Length(_) => "@length",
        Annotation::Min(_) => "@min",
        Annotation::Max(_) => "@max",
        _ => "@annotation",
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

    // ---- Tests DSL v0.3 ----

    #[test]
    fn v03_valid_validations_no_errors() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    email String @email @max(255) @min(1)
    age   Int    @min(0) @max(150)
    code  String @length(3)
}
"#;
        let (_, diags) = compile(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn v03_email_on_non_string_is_error() {
        let src = r#"
model Bad {
    id  UUID @primary @auto
    age Int  @email
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("@email")),
            "should error: @email on Int"
        );
    }

    #[test]
    fn v03_regex_on_non_string_is_error() {
        let src = r#"
model Bad {
    id    UUID @primary @auto
    score Int  @regex("^[0-9]+$")
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("@regex")));
    }

    #[test]
    fn v03_length_on_non_string_is_error() {
        let src = r#"
model Bad {
    id  UUID @primary @auto
    val Bool @length(3)
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("@length")));
    }

    // ---- Tests DSL v0.4 ----

    #[test]
    fn v04_references_to_undefined_model_is_error() {
        let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(Ghost.id)
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("Ghost")),
            "should error: model Ghost not defined. Got: {diags:?}"
        );
    }

    #[test]
    fn v04_references_to_non_primary_field_is_warning() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique
}
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.email)
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| !d.is_error() && d.message.contains("@primary")),
            "should warn: email is not @primary. Got: {diags:?}"
        );
    }

    #[test]
    fn v04_model_ref_without_relation_is_error() {
        let src = r#"
model User {
    id UUID @primary @auto
}
model Post {
    id     UUID @primary @auto
    author User
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("@relation")),
            "should error: ModelRef without @relation. Got: {diags:?}"
        );
    }

    #[test]
    fn v04_relation_with_missing_fk_field_is_error() {
        let src = r#"
model User {
    id UUID @primary @auto
}
model Post {
    id     UUID @primary @auto
    author User @relation(nonexistent_id)
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("nonexistent_id")),
            "should error: FK field nonexistent_id does not exist. Got: {diags:?}"
        );
    }

    #[test]
    fn v04_circular_fk_is_error() {
        let src = r#"
model A {
    id UUID @primary @auto
    b_id UUID @references(B.id)
}
model B {
    id UUID @primary @auto
    a_id UUID @references(A.id)
}
"#;
        let (_, diags) = compile(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("circular")),
            "should error: circular FK between A and B. Got: {diags:?}"
        );
    }

    #[test]
    fn v03_min_greater_than_max_is_error() {
        let src = r#"
model Bad {
    id   UUID   @primary @auto
    name String @min(100) @max(10)
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("minimo supera al maximo")),);
    }

    #[test]
    fn v03_min_on_uuid_is_error() {
        let src = r#"
model Bad {
    id UUID @primary @auto @min(1)
}
"#;
        let (_, diags) = compile(src);
        assert!(diags
            .iter()
            .any(|d| d.is_error() && d.message.contains("@min")));
    }

    #[test]
    fn v03_request_field_validations() {
        let src = r#"
request CreateUser {
    email String @email @max(255)
    name  String @min(2) @max(100)
    age   Int    @min(18) @max(120)
}
"#;
        let (_, diags) = compile(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(
            errors.is_empty(),
            "unexpected errors in request: {errors:?}"
        );
    }
}
