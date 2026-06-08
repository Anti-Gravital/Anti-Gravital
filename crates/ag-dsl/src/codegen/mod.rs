//! Anti-DSL v0.1 code generators.
//!
//! Each sub-module is an independent generator for a different target.
//! The `generate()` function invokes all generators and returns the
//! collection of files to write to disk.

pub mod async_api_gen;
pub mod openapi_gen;
pub mod rust_gen;
pub mod sql_gen;
pub mod ts_gen;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ast::Schema;

/// Collection of generated files, indexed by path relative to the project.
///
/// Keys are relative paths (e.g. `src/models.rs`).
/// Values are the file contents as `String`.
#[derive(Debug, Default)]
pub struct GeneratedFiles {
    /// Generated files: path -> content.
    pub files: BTreeMap<PathBuf, String>,
}

impl GeneratedFiles {
    fn insert(&mut self, path: impl Into<PathBuf>, content: String) {
        self.files.insert(path.into(), content);
    }

    /// Number of generated files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns true if no file was generated.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Generates all artifacts for the given schema.
///
/// Returns the collection of files with their relative paths and content.
/// The caller (ag-cli) is responsible for writing them to disk.
pub fn generate(schema: &Schema) -> GeneratedFiles {
    let mut files = GeneratedFiles::default();

    files.insert(
        PathBuf::from("src/mod.rs"),
        rust_gen::generate_module(schema),
    );

    // v0.1: DB models
    files.insert(
        PathBuf::from("src/models.rs"),
        rust_gen::generate_models(schema),
    );

    // v0.1: SQL migration
    files.insert(
        PathBuf::from("migrations/0001_initial.sql"),
        sql_gen::generate_migration(schema),
    );

    // v0.2: API types (request/response/error)
    let types_rs = rust_gen::generate_types(schema);
    if !types_rs.is_empty() {
        files.insert(PathBuf::from("src/types.rs"), types_rs);
    }

    // v0.2: handler stubs
    let handlers_rs = rust_gen::generate_handlers(schema);
    if !handlers_rs.is_empty() {
        files.insert(PathBuf::from("src/handlers.rs"), handlers_rs);
    }

    // v0.2: Axum router
    let router_rs = rust_gen::generate_router(schema);
    if !router_rs.is_empty() {
        files.insert(PathBuf::from("src/router.rs"), router_rs);
    }

    // TypeScript: interfaces (v0.1+v0.2)
    files.insert(
        PathBuf::from("clients/typescript/types.ts"),
        ts_gen::generate_types(schema),
    );

    // TypeScript: HTTP client (v0.2)
    let client_ts = ts_gen::generate_client(schema);
    if !client_ts.is_empty() {
        files.insert(PathBuf::from("clients/typescript/client.ts"), client_ts);
    }

    // Full OpenAPI 3.1 in JSON (schemas + paths v0.2)
    files.insert(
        PathBuf::from("openapi.json"),
        openapi_gen::generate_openapi(schema),
    );

    // OpenAPI 3.1 in YAML with securitySchemes v0.5
    files.insert(
        PathBuf::from("openapi.yaml"),
        openapi_gen::generate_openapi_yaml(schema),
    );

    // AsyncAPI 2.6 — only when there are declared events (v0.6)
    if let Some((path, content)) = async_api_gen::generate(schema) {
        files.insert(path, content);
    }

    files
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
    fn generate_produces_asyncapi_when_events_present() {
        let src = r#"
event user.created { payload UserResponse retain 30 }
response UserResponse { id UUID }
"#;
        let schema = crate::compile(src).unwrap();
        let files = generate(&schema);
        let has_async_api = files
            .files
            .iter()
            .any(|(p, _)| p.to_str().unwrap_or("").contains("asyncapi"));
        assert!(
            has_async_api,
            "debe generar asyncapi.yaml cuando hay eventos"
        );
    }

    #[test]
    fn generates_six_files() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#,
        );
        let files = generate(&schema);
        assert_eq!(files.len(), 6);
        assert!(files.files.contains_key(&PathBuf::from("src/mod.rs")));
        assert!(files.files.contains_key(&PathBuf::from("src/models.rs")));
        assert!(files
            .files
            .contains_key(&PathBuf::from("migrations/0001_initial.sql")));
        assert!(files
            .files
            .contains_key(&PathBuf::from("clients/typescript/types.ts")));
        assert!(files.files.contains_key(&PathBuf::from("openapi.json")));
        assert!(files.files.contains_key(&PathBuf::from("openapi.yaml")));
    }
}
