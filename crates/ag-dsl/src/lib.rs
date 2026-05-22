//! Compilador del Anti-DSL: lexer, parser, AST, analisis semantico y generadores de codigo.
//!
//! # Uso
//!
//! ```rust
//! use ag_dsl::{compile, generate};
//!
//! let source = r#"
//! model User {
//!     id    UUID   @primary @auto
//!     email String @unique
//!     name  String
//! }
//! "#;
//!
//! match compile(source) {
//!     Ok(schema) => {
//!         let files = generate(&schema);
//!         for (path, content) in &files.files {
//!             println!("--- {} ---", path.display());
//!             println!("{content}");
//!         }
//!     }
//!     Err(diagnostics) => {
//!         for d in &diagnostics {
//!             eprintln!("{}", d.display(source));
//!         }
//!     }
//! }
//! ```
//!
//! # Pipeline
//!
//! ```text
//! texto .ag
//!   -> lexer (logos)      tokens + errores de lex
//!   -> parser (chumsky)   AST parcial + errores de parse
//!   -> semantic           errores/warnings semanticos
//!   -> codegen            GeneratedFiles { rust, sql, typescript, openapi }
//! ```

pub mod ast;
pub mod codegen;
pub mod diagnostics;
mod lexer;
mod parser;
mod semantic;

pub use codegen::{generate, GeneratedFiles};
pub use diagnostics::Diagnostic;

/// Compila texto fuente DSL y retorna el AST si no hay errores de lex, parse o semantica.
///
/// Los warnings (como un modelo sin @primary) no bloquean la compilacion.
/// Solo los errores de severidad `Error` producen `Err`.
///
/// # Errors
///
/// Retorna `Err(Vec<Diagnostic>)` si hay errores de lex, parse o semanticos.
pub fn compile(source: &str) -> Result<ast::Schema, Vec<Diagnostic>> {
    let (tokens, lex_spans) = lexer::tokenize(source);

    let mut all_diags: Vec<Diagnostic> = lex_spans.into_iter().map(Diagnostic::lex_error).collect();

    let (ast, parse_diags) = parser::parse_tokens(tokens, source.len());
    all_diags.extend(parse_diags);

    match ast {
        Some(schema) => {
            let semantic_diags = semantic::analyze(&schema);
            all_diags.extend(semantic_diags);

            if all_diags.iter().any(|d| d.is_error()) {
                Err(all_diags)
            } else {
                Ok(schema)
            }
        }
        None => {
            if all_diags.is_empty() {
                all_diags.push(Diagnostic::parse_error(
                    0..source.len().max(1),
                    "no se pudo construir el AST",
                ));
            }
            Err(all_diags)
        }
    }
}

/// Analiza el fuente DSL y retorna todos los diagnosticos (errores y warnings).
///
/// A diferencia de `compile()`, siempre retorna la lista completa: no descarta
/// warnings cuando no hay errores. Usar en el servidor LSP y en `ag schema lint`.
pub fn lint(source: &str) -> Vec<Diagnostic> {
    let (tokens, lex_spans) = lexer::tokenize(source);
    let mut all_diags: Vec<Diagnostic> = lex_spans.into_iter().map(Diagnostic::lex_error).collect();

    let (ast, parse_diags) = parser::parse_tokens(tokens, source.len());
    all_diags.extend(parse_diags);

    if let Some(schema) = ast {
        all_diags.extend(semantic::analyze(&schema));
    }

    all_diags
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MODEL: &str = r#"
model User {
    id    UUID   @primary @auto
    email String @unique @email @max(255)
    name  String @min(2)
}
"#;

    #[test]
    fn compile_valid_schema_returns_ok() {
        let schema = compile(VALID_MODEL).expect("valid schema should compile");
        assert_eq!(schema.models.len(), 1);
        assert_eq!(schema.models[0].name.value, "User");
    }

    #[test]
    fn compile_with_warnings_returns_ok() {
        let src = "model Tag { name String }";
        let schema = compile(src).expect("warnings should not block compilation");
        assert_eq!(schema.models[0].name.value, "Tag");
    }

    #[test]
    fn compile_semantic_error_returns_err() {
        let src = "model Bad { id UUID @primary @auto @min(1) }";
        let diags = compile(src).expect_err("@min on UUID should fail");
        assert!(diags.iter().any(|d| d.is_error()));
    }

    #[test]
    fn compile_parse_error_returns_err() {
        let src = "model Unclosed { id UUID @primary @auto";
        let diags = compile(src).expect_err("unclosed brace should fail");
        assert!(diags.iter().any(|d| d.is_error()));
    }

    #[test]
    fn compile_lex_error_returns_err() {
        let src = "model Bad { id UUID @ }";
        assert!(compile(src).is_err());
    }

    #[test]
    fn generate_produces_files_for_valid_schema() {
        let schema = compile(VALID_MODEL).unwrap();
        let files = generate(&schema);
        assert!(!files.files.is_empty());
        let paths: Vec<_> = files.files.keys().collect();
        assert!(
            paths.iter().any(|p| p.to_string_lossy().contains("models")),
            "should generate models file"
        );
        assert!(
            paths.iter().any(|p| p.to_string_lossy().contains("sql")),
            "should generate migration"
        );
    }

    #[test]
    fn generate_full_schema_produces_all_artefacts() {
        let src = r#"
config { project_name "test" database "postgres" }
model User {
    id    UUID   @primary @auto
    email String @unique @email @max(255)
}
request CreateUser { email String @email }
response UserResp  { id UUID email String }
error EmailTaken   { status 409 message "taken" }
endpoint CreateUser {
    method POST
    path   /users
    body   CreateUser
    response UserResp
    errors [EmailTaken]
}
endpoint GetUser {
    method GET
    path   /users/{id}
    response UserResp
    errors [EmailTaken]
}
"#;
        let schema = compile(src).expect("full schema should compile");
        let files = generate(&schema);
        let paths: Vec<_> = files
            .files
            .keys()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        assert!(paths.iter().any(|p| p.contains("handlers")), "handlers");
        assert!(paths.iter().any(|p| p.contains("router")), "router");
        assert!(paths.iter().any(|p| p.contains("types")), "types");
        assert!(paths.iter().any(|p| p.contains("client")), "client");
        assert!(paths.iter().any(|p| p.contains("openapi")), "openapi");
    }

    #[test]
    fn generate_with_relations_produces_fk_sql() {
        let src = r#"
model User { id UUID @primary @auto }
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#;
        let schema = compile(src).unwrap();
        let files = generate(&schema);
        let sql = files
            .files
            .iter()
            .find(|(p, _)| p.to_string_lossy().contains("sql"))
            .map(|(_, c)| c.as_str())
            .unwrap_or("");
        assert!(sql.contains("FOREIGN KEY"), "should have FK in SQL");
    }

    #[test]
    fn lint_returns_warnings_for_model_without_primary() {
        let src = "model Tag { name String }";
        let diags = lint(src);
        assert!(
            !diags.is_empty(),
            "lint debe retornar warnings aunque compile() retorne Ok"
        );
        assert!(
            diags.iter().all(|d| !d.is_error()),
            "para este schema solo deben ser warnings, no errores"
        );
    }

    #[test]
    fn lint_returns_errors_for_invalid_schema() {
        let src = "model Bad { id UUID @primary @auto @min(1) }";
        let diags = lint(src);
        assert!(diags.iter().any(|d| d.is_error()));
    }

    #[test]
    fn lint_returns_empty_for_clean_schema() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique @email @max(255)
    name  String @min(2)
}
"#;
        let diags = lint(src);
        let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "schema limpio no debe tener errores");
    }

    // ---- Tests DSL v0.5 + v0.6 ----

    #[test]
    fn policy_without_auth_is_error() {
        // auth se omite: el default es AuthMode::None.
        // policy con auth None debe producir error semantico.
        let src = r#"
endpoint GetProfile {
    method GET
    path /profile
    policy "user.id == params.id"
}
"#;
        let diags = lint(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("policy")),
            "debe haber error sobre policy sin auth: {:?}",
            diags
        );
    }

    #[test]
    fn event_in_endpoint_not_declared_is_error() {
        let src = r#"
endpoint CreateUser {
    method POST
    path /users
    auth required
    events [user.created]
}
"#;
        let diags = lint(src);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("user.created")),
            "debe haber error por evento no declarado: {:?}",
            diags
        );
    }

    #[test]
    fn event_name_without_dot_is_warning() {
        let src = r#"
event usercreated {
    payload UserResponse
}
response UserResponse { id UUID }
"#;
        let diags = lint(src);
        assert!(
            diags
                .iter()
                .any(|d| !d.is_error() && d.message.contains("usercreated")),
            "debe haber warning por nombre sin punto: {:?}",
            diags
        );
    }

    #[test]
    fn fuzz_crash_repro_tab_comment_number() {
        // Crash encontrado por cargo-fuzz: \t#\n11111111111111111111,\n#\n#
        // El compilador no debe entrar en panico con ningun input UTF-8 valido.
        let input = "\t#\n11111111111111111111,\n#\n#";
        let diags = lint(input);
        // No importa el resultado, solo que no haya panic
        let _ = diags;
        if let Ok(schema) = compile(input) {
            let _ = generate(&schema);
        }
    }
}
