//! Tipos de diagnostico del compilador Anti-DSL.
//!
//! Todos los errores y warnings del pipeline (lex, parse, semantic) se
//! unifican en `Diagnostic`. Los consumidores (ag-cli) pueden filtrar por
//! severidad y formatear los mensajes para el usuario.

use crate::lexer::Span;
use thiserror::Error;

/// Severidad del diagnostico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Error que impide la generacion de codigo.
    Error,
    /// Advertencia informativa; la generacion puede continuar.
    Warning,
}

/// Diagnostico del compilador con posicion, severidad y mensaje.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Posicion en el texto fuente.
    pub span: Span,
    /// Severidad del diagnostico.
    pub severity: Severity,
    /// Mensaje legible para el usuario.
    pub message: String,
    /// Sugerencia opcional de como corregirlo.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Error de lex: caracter no reconocido.
    pub fn lex_error(span: Span) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: "caracter inesperado".to_owned(),
            hint: None,
        }
    }

    /// Error de parse: token inesperado.
    pub fn parse_error(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    /// Error semantico generico.
    pub fn semantic_error(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    /// Error semantico con sugerencia.
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

    /// Warning semantico.
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            severity: Severity::Warning,
            message: message.into(),
            hint: None,
        }
    }

    /// Retorna true si este diagnostico es un error (bloquea generacion).
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Formatea el diagnostico como string legible para el usuario.
    ///
    /// Formato: `[error|warning] byte N-M: mensaje (sugerencia si aplica)`
    pub fn display(&self, source: &str) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };

        // Calcula linea y columna desde el byte offset.
        let (line, col) = byte_offset_to_line_col(source, self.span.start);
        let location = format!("{}:{}", line, col);

        let mut out = format!("{severity} [{location}]: {}", self.message);
        if let Some(hint) = &self.hint {
            out.push_str(&format!("\n  ayuda: {hint}"));
        }
        out
    }
}

/// Error fatal del compilador (no un diagnostico por posicion).
#[derive(Debug, Error)]
pub enum CompilerError {
    /// El texto fuente contiene errores que impiden compilar.
    #[error("el schema contiene {0} error(es)")]
    SchemaErrors(usize),
}

// Convierte byte offset a (linea, columna) 1-indexado.
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
