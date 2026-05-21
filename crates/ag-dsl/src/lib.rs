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

/// Compila un texto fuente DSL y retorna el AST si no hay errores.
///
/// Los warnings (como un modelo sin @primary) no bloquean la compilacion.
/// Solo los errores de severidad `Error` producen `Err`.
///
/// # Errors
///
/// Retorna `Err(Vec<Diagnostic>)` si hay errores de lex, parse o semantic.
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
