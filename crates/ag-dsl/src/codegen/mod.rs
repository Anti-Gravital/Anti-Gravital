//! Generadores de codigo del Anti-DSL v0.1.
//!
//! Cada sub-modulo es un generador independiente para un target distinto.
//! La funcion `generate()` invoca todos los generadores y retorna la
//! coleccion de archivos a escribir en disco.

pub mod openapi_gen;
pub mod rust_gen;
pub mod sql_gen;
pub mod ts_gen;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ast::Schema;

/// Coleccion de archivos generados, indexada por ruta relativa al proyecto.
///
/// Las claves son rutas relativas (p. ej. `src/models.rs`).
/// Los valores son el contenido del archivo como `String`.
#[derive(Debug, Default)]
pub struct GeneratedFiles {
    /// Archivos generados: ruta -> contenido.
    pub files: BTreeMap<PathBuf, String>,
}

impl GeneratedFiles {
    fn insert(&mut self, path: impl Into<PathBuf>, content: String) {
        self.files.insert(path.into(), content);
    }

    /// Numero de archivos generados.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Retorna true si no se genero ningun archivo.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Genera todos los artefactos para el schema dado.
///
/// Retorna la coleccion de archivos con sus rutas relativas y contenido.
/// El llamador (ag-cli) es responsable de escribirlos en disco.
pub fn generate(schema: &Schema) -> GeneratedFiles {
    let mut files = GeneratedFiles::default();

    // v0.1: modelos DB
    files.insert(
        PathBuf::from("src/models.rs"),
        rust_gen::generate_models(schema),
    );

    // v0.1: migracion SQL
    files.insert(
        PathBuf::from("migrations/0001_initial.sql"),
        sql_gen::generate_migration(schema),
    );

    // v0.2: tipos API (request/response/error)
    let types_rs = rust_gen::generate_types(schema);
    if !types_rs.is_empty() {
        files.insert(PathBuf::from("src/types.rs"), types_rs);
    }

    // v0.2: handler stubs
    let handlers_rs = rust_gen::generate_handlers(schema);
    if !handlers_rs.is_empty() {
        files.insert(PathBuf::from("src/handlers.rs"), handlers_rs);
    }

    // v0.2: router Axum
    let router_rs = rust_gen::generate_router(schema);
    if !router_rs.is_empty() {
        files.insert(PathBuf::from("src/router.rs"), router_rs);
    }

    // TypeScript: interfaces (v0.1+v0.2)
    files.insert(
        PathBuf::from("clients/typescript/types.ts"),
        ts_gen::generate_types(schema),
    );

    // TypeScript: cliente HTTP (v0.2)
    let client_ts = ts_gen::generate_client(schema);
    if !client_ts.is_empty() {
        files.insert(PathBuf::from("clients/typescript/client.ts"), client_ts);
    }

    // OpenAPI 3.1 completo (schemas + paths en v0.2)
    files.insert(
        PathBuf::from("openapi.json"),
        openapi_gen::generate_openapi(schema),
    );

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
    fn generates_four_files() {
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
        assert_eq!(files.len(), 4);
        assert!(files.files.contains_key(&PathBuf::from("src/models.rs")));
        assert!(files
            .files
            .contains_key(&PathBuf::from("migrations/0001_initial.sql")));
        assert!(files
            .files
            .contains_key(&PathBuf::from("clients/typescript/types.ts")));
        assert!(files.files.contains_key(&PathBuf::from("openapi.json")));
    }
}
