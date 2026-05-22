//! Generador OpenAPI 3.1 para DSL v0.1–v0.5.
//!
//! Produce `components/schemas` (v0.1), `paths` con operaciones (v0.2) y
//! `securitySchemes` + `security` por endpoint (v0.5).
//! La salida JSON es la version canonica; el helper YAML se usa en el pipeline de generacion.

use serde_json::{json, Value};

use crate::ast::{Annotation, AuthMode, EndpointDef, FieldDef, HttpMethod, ModelDef, Schema};

/// Genera el documento OpenAPI 3.1 completo.
///
/// Retorna JSON formateado con indentacion de dos espacios.
pub fn generate_openapi(schema: &Schema) -> String {
    let mut schemas = serde_json::Map::new();

    // Schemas de modelos (v0.1)
    for model in &schema.models {
        let name = &model.name.value;
        schemas.insert(name.clone(), model_schema(model, false));
        schemas.insert(format!("Create{name}Request"), model_schema(model, true));
    }

    // Schemas de request/response types (v0.2)
    for req in &schema.requests {
        schemas.insert(req.name.value.clone(), simple_type_schema(&req.fields));
    }
    for resp in &schema.responses {
        schemas.insert(resp.name.value.clone(), simple_type_schema(&resp.fields));
    }

    let project_name = schema
        .config
        .as_ref()
        .and_then(|c| c.project_name.as_deref())
        .unwrap_or("anti-gravital-api");

    // Paths (v0.2)
    let paths = if schema.endpoints.is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        build_paths(schema)
    };

    let doc = json!({
        "openapi": "3.1.0",
        "info": {
            "title": project_name,
            "version": "0.1.0",
            "description": "API generada por Anti-Gravital ag-dsl v0.2"
        },
        "paths": paths,
        "components": {
            "schemas": schemas
        }
    });

    serde_json::to_string_pretty(&doc).expect("serde_json serialization is infallible")
}

/// Genera el documento OpenAPI 3.1 como YAML con soporte de seguridad v0.5.
///
/// Si algun endpoint tiene `auth != None`, agrega `securitySchemes` en `components`
/// y el campo `security` en la operacion correspondiente.
pub fn generate_openapi_yaml(schema: &Schema) -> String {
    let has_auth = schema
        .endpoints
        .iter()
        .any(|ep| ep.auth != AuthMode::None);

    let project_name = schema
        .config
        .as_ref()
        .and_then(|c| c.project_name.as_deref())
        .unwrap_or("anti-gravital-api");

    let mut out = String::new();
    out.push_str("openapi: '3.1.0'\n");
    out.push_str(&format!(
        "info:\n  title: {}\n  version: '0.1.0'\n",
        project_name
    ));

    // Paths
    if !schema.endpoints.is_empty() {
        out.push_str("paths:\n");
        let mut by_path: std::collections::BTreeMap<&str, Vec<&EndpointDef>> =
            std::collections::BTreeMap::new();
        for ep in &schema.endpoints {
            by_path.entry(ep.path.value.as_str()).or_default().push(ep);
        }
        for (path, endpoints) in &by_path {
            out.push_str(&format!("  {}:\n", path));
            for ep in endpoints {
                let method = ep.method.value.as_str().to_lowercase();
                out.push_str(&format!("    {}:\n", method));
                out.push_str(&format!("      operationId: {}\n", ep.name.value));
                if ep.auth != AuthMode::None {
                    out.push_str("      security:\n");
                    out.push_str("        - BearerAuth: []\n");
                }
                if let Some(resp) = &ep.response {
                    out.push_str("      responses:\n");
                    let code = match ep.method.value {
                        HttpMethod::Post => "201",
                        HttpMethod::Delete => "204",
                        _ => "200",
                    };
                    out.push_str(&format!("        '{}':\n", code));
                    out.push_str("          description: Respuesta exitosa\n");
                    out.push_str(&format!(
                        "          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/{}'\n",
                        resp.value
                    ));
                } else {
                    out.push_str("      responses:\n        '200':\n          description: Exito\n");
                }
            }
        }
    } else {
        out.push_str("paths: {}\n");
    }

    // Components
    out.push_str("components:\n");
    if has_auth {
        out.push_str("  securitySchemes:\n");
        out.push_str("    BearerAuth:\n");
        out.push_str("      type: http\n");
        out.push_str("      scheme: bearer\n");
        out.push_str("      bearerFormat: JWT\n");
    }
    out.push_str("  schemas: {}\n");

    out
}

/// Construye el objeto `paths` para todos los endpoints del schema.
fn build_paths(schema: &Schema) -> Value {
    let mut paths: serde_json::Map<String, Value> = serde_json::Map::new();

    // Agrupa endpoints por path
    let mut by_path: std::collections::BTreeMap<&str, Vec<&EndpointDef>> =
        std::collections::BTreeMap::new();
    for ep in &schema.endpoints {
        by_path.entry(ep.path.value.as_str()).or_default().push(ep);
    }

    for (path, endpoints) in by_path {
        let mut path_item = serde_json::Map::new();
        for ep in endpoints {
            let method_key = ep.method.value.as_str().to_lowercase();
            let operation = build_operation(ep, schema);
            path_item.insert(method_key, operation);
        }
        // Convierte path DSL {id} a OpenAPI {id} (misma sintaxis)
        paths.insert(path.to_owned(), Value::Object(path_item));
    }

    Value::Object(paths)
}

fn build_operation(ep: &EndpointDef, schema: &Schema) -> Value {
    let mut op = serde_json::Map::new();
    op.insert("operationId".to_owned(), json!(ep.name.value));

    // Request body
    if let Some(body) = &ep.body {
        let rb = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": { "$ref": format!("#/components/schemas/{}", body.value) }
                }
            }
        });
        op.insert("requestBody".to_owned(), rb);
    }

    // Responses
    let mut responses = serde_json::Map::new();
    let success_code = match ep.method.value {
        HttpMethod::Post => "201",
        HttpMethod::Delete => "204",
        _ => "200",
    };
    if let Some(resp) = &ep.response {
        responses.insert(
            success_code.to_owned(),
            json!({
                "description": "Respuesta exitosa",
                "content": {
                    "application/json": {
                        "schema": { "$ref": format!("#/components/schemas/{}", resp.value) }
                    }
                }
            }),
        );
    } else {
        responses.insert(success_code.to_owned(), json!({ "description": "Exito" }));
    }

    // Error responses desde ErrorDef del schema
    for err_ref in &ep.errors {
        if let Some(err_def) = schema.errors.iter().find(|e| e.name.value == err_ref.value) {
            let code = err_def.status.value.to_string();
            responses.insert(
                code,
                json!({
                    "description": err_def.message.value
                }),
            );
        }
    }
    op.insert("responses".to_owned(), Value::Object(responses));

    Value::Object(op)
}

/// Schema para campos de request/response types simples (sin @auto).
fn simple_type_schema(fields: &[FieldDef]) -> Value {
    field_list_schema(fields)
}

/// Construye un schema object a partir de una lista de campos.
fn field_list_schema(fields: &[FieldDef]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required: Vec<Value> = Vec::new();

    for field in fields {
        let fname = &field.name.value;
        let (ts_type, format) = field.ty.value.openapi_type();
        let mut prop = serde_json::Map::new();
        if field.optional {
            prop.insert("type".to_owned(), json!([ts_type, "null"]));
        } else {
            prop.insert("type".to_owned(), json!(ts_type));
        }
        if let Some(fmt) = format {
            prop.insert("format".to_owned(), json!(fmt));
        }
        // v0.3 — añade constraints de validacion
        apply_validation_constraints(&mut prop, field);
        properties.insert(fname.clone(), Value::Object(prop));
        if !field.optional {
            required.push(json!(fname));
        }
    }

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_owned(), json!("object"));
    if !required.is_empty() {
        schema.insert("required".to_owned(), Value::Array(required));
    }
    schema.insert("properties".to_owned(), Value::Object(properties));
    Value::Object(schema)
}

/// Añade campos de validacion OpenAPI 3.1 derivados de las anotaciones v0.3.
fn apply_validation_constraints(prop: &mut serde_json::Map<String, Value>, field: &FieldDef) {
    use crate::ast::{Annotation, FieldType};

    let is_string = field.ty.value == FieldType::String;
    let is_numeric = matches!(
        field.ty.value,
        FieldType::Int | FieldType::Float | FieldType::Decimal
    );

    for ann in &field.annotations {
        match &ann.value {
            Annotation::Min(n) if is_string => {
                prop.insert("minLength".to_owned(), json!(n));
            }
            Annotation::Max(n) if is_string => {
                prop.insert("maxLength".to_owned(), json!(n));
            }
            Annotation::Min(n) if is_numeric => {
                prop.insert("minimum".to_owned(), json!(n));
            }
            Annotation::Max(n) if is_numeric => {
                prop.insert("maximum".to_owned(), json!(n));
            }
            Annotation::Email => {
                prop.insert("format".to_owned(), json!("email"));
            }
            Annotation::Regex(pattern) => {
                prop.insert("pattern".to_owned(), json!(pattern));
            }
            Annotation::Length(n) => {
                prop.insert("minLength".to_owned(), json!(n));
                prop.insert("maxLength".to_owned(), json!(n));
            }
            _ => {}
        }
    }
}

/// Genera el schema JSON Schema de un modelo.
///
/// Si `create_only` es true, omite los campos @auto para el CreateRequest.
fn model_schema(model: &ModelDef, create_only: bool) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required: Vec<Value> = Vec::new();

    for field in &model.fields {
        // Omitir campos @auto en el schema de creacion
        if create_only && is_auto_generated(field) {
            continue;
        }

        let fname = &field.name.value;

        // v0.4: campos virtuales usan $ref en lugar de type+format
        if field.virtual_field {
            let prop = match &field.ty.value {
                crate::ast::FieldType::ModelRef(m) => {
                    json!({ "$ref": format!("#/components/schemas/{m}") })
                }
                crate::ast::FieldType::ModelRefList(m) => {
                    json!({
                        "type": "array",
                        "items": { "$ref": format!("#/components/schemas/{m}") }
                    })
                }
                _ => continue,
            };
            properties.insert(fname.clone(), prop);
            // Campos virtuales son opcionales (pueden no estar cargados)
            continue;
        }

        let (ts_type, format) = field.ty.value.openapi_type();

        let mut prop = serde_json::Map::new();
        prop.insert("type".to_owned(), json!(ts_type));
        if let Some(fmt) = format {
            prop.insert("format".to_owned(), json!(fmt));
        }
        if field.optional {
            // OpenAPI 3.1 usa nullable via type array
            prop.insert("type".to_owned(), json!([ts_type, "null"]));
        }
        // v0.3 — constraints de validacion
        apply_validation_constraints(&mut prop, field);

        properties.insert(fname.clone(), Value::Object(prop));

        if !(field.optional || (create_only && is_auto_generated(field))) {
            required.push(json!(fname));
        }
    }

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_owned(), json!("object"));
    if !required.is_empty() {
        schema.insert("required".to_owned(), Value::Array(required));
    }
    schema.insert("properties".to_owned(), Value::Object(properties));

    Value::Object(schema)
}

fn is_auto_generated(field: &FieldDef) -> bool {
    field
        .annotations
        .iter()
        .any(|a| matches!(a.value, Annotation::Auto | Annotation::AutoUpdate))
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
    fn openapi_gen_adds_security_for_auth_required() {
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
        let openapi = files.files.iter()
            .find(|(p, _)| {
                let s = p.to_str().unwrap_or("");
                s.ends_with(".yaml") || s.ends_with(".yml")
            })
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        assert!(
            openapi.contains("BearerAuth") || openapi.contains("bearerAuth"),
            "debe incluir securityScheme BearerAuth, got:\n{}", openapi
        );
        assert!(openapi.contains("security"), "debe incluir campo security, got:\n{}", openapi);
    }

    #[test]
    fn generates_openapi_json() {
        let schema = schema_from(
            r#"
config {
    project_name "test-api"
    database "postgres"
}

model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#,
        );
        let json_str = generate_openapi(&schema);
        let doc: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");

        assert_eq!(doc["openapi"], "3.1.0");
        assert_eq!(doc["info"]["title"], "test-api");
        assert!(doc["components"]["schemas"]["User"].is_object());
        assert!(doc["components"]["schemas"]["CreateUserRequest"].is_object());
    }

    #[test]
    fn uuid_has_format() {
        let schema = schema_from(
            r#"
model Item {
    id UUID @primary @auto
    v  Int
}
"#,
        );
        let json_str = generate_openapi(&schema);
        let doc: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let id_prop = &doc["components"]["schemas"]["Item"]["properties"]["id"];
        assert_eq!(id_prop["format"], "uuid");
    }

    #[test]
    fn auto_field_excluded_from_create_schema() {
        let schema = schema_from(
            r#"
model Post {
    id    UUID @primary @auto
    title String
}
"#,
        );
        let json_str = generate_openapi(&schema);
        let doc: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let create = &doc["components"]["schemas"]["CreatePostRequest"]["properties"];
        assert!(
            create["id"].is_null(),
            "id should be excluded from CreateRequest"
        );
        assert!(create["title"].is_object());
    }

    // ---- Tests DSL v0.4 ----

    #[test]
    fn v04_openapi_ref_schemas() {
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
        let json_str = generate_openapi(&schema);
        let doc: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");

        let author_prop = &doc["components"]["schemas"]["Post"]["properties"]["author"];
        assert_eq!(
            author_prop["$ref"], "#/components/schemas/User",
            "N:1 field should use $ref to User"
        );

        let posts_prop = &doc["components"]["schemas"]["User"]["properties"]["posts"];
        assert_eq!(
            posts_prop["type"], "array",
            "1:N field should be array type"
        );
        assert_eq!(
            posts_prop["items"]["$ref"], "#/components/schemas/Post",
            "1:N items should use $ref to Post"
        );

        let author_id_prop = &doc["components"]["schemas"]["Post"]["properties"]["author_id"];
        assert_eq!(author_id_prop["type"], "string");
        assert_eq!(author_id_prop["format"], "uuid");
    }

    #[test]
    fn required_fields_listed() {
        let schema = schema_from(
            r#"
model Product {
    id   UUID   @primary @auto
    name String
    desc String?
}
"#,
        );
        let json_str = generate_openapi(&schema);
        let doc: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let required = &doc["components"]["schemas"]["Product"]["required"];
        let required_list: Vec<&str> = required
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required_list.contains(&"id"));
        assert!(required_list.contains(&"name"));
        assert!(
            !required_list.contains(&"desc"),
            "optional field should not be required"
        );
    }
}
