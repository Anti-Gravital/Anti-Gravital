//! Anti-DSL compiler diagnostic types.
//!
//! All pipeline errors and warnings (lex, parse, semantic) are
//! unified into `Diagnostic`. Consumers (ag-cli) can filter by
//! severity and format the messages for the user.

use crate::lexer::Span;
use thiserror::Error;

/// Diagnostic severity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Error that prevents code generation.
    Error,
    /// Informational warning; generation can continue.
    Warning,
}

/// Compiler diagnostic with position, severity and message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Position in the source text.
    pub span: Span,
    /// Diagnostic severity.
    pub severity: Severity,
    /// User-readable message.
    pub message: String,
    /// Optional hint on how to fix it.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Lex error: unrecognized character.
    pub fn lex_error(span: Span) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: "caracter inesperado".to_owned(),
            hint: None,
        }
    }

    /// Parse error: unexpected token.
    pub fn parse_error(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    /// Generic semantic error.
    pub fn semantic_error(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    /// Semantic error with a hint.
    pub fn semantic_error_with_hint(
        span: Span,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }

    /// Semantic warning.
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Warning,
            message: message.into(),
            hint: None,
        }
    }

    /// Returns true if this diagnostic is an error (blocks generation).
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Formats the diagnostic as a user-readable string.
    ///
    /// Format: `[error|warning] byte N-M: message (hint if applicable)`
    pub fn display(&self, source: &str) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        // Compute line and column from the byte offset.
        let (line, col) = byte_offset_to_line_col(source, self.span.start);
        let location = format!("{}:{}", line, col);

        let mut out = format!("{severity} [{location}]: {}", self.message);
        if let Some(hint) = &self.hint {
            out.push_str(&format!("\n  ayuda: {hint}"));
        }
        out
    }
}

/// Fatal compiler error (not a per-position diagnostic).
#[derive(Debug, Error)]
pub enum CompilerError {
    /// The source text contains errors that prevent compilation.
    #[error("el schema contiene {0} error(es)")]
    SchemaErrors(usize),
}

// Converts a byte offset to a 1-indexed (line, column).
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset) + 1;
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_error_is_error_severity() {
        let d = Diagnostic::lex_error(0..1);
        assert!(d.is_error());
    }

    #[test]
    fn warning_is_not_error() {
        let d = Diagnostic::warning(0..1, "esto es un aviso");
        assert!(!d.is_error());
    }

    #[test]
    fn display_shows_line_col() {
        let source = "model\nUser";
        let d = Diagnostic::parse_error(6..10, "token inesperado");
        let out = d.display(source);
        assert!(out.contains("2:1"), "should show line 2 col 1, got: {out}");
        assert!(out.contains("token inesperado"));
    }

    #[test]
    fn display_with_hint() {
        let source = "model User {}";
        let d = Diagnostic::semantic_error_with_hint(
            0..5,
            "sin campos",
            "anade al menos un campo al modelo",
        );
        let out = d.display(source);
        assert!(out.contains("ayuda:"));
    }
}
