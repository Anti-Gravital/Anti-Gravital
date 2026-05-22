//! Backend del servidor LSP Anti-Gravital.
//!
//! Implementa el trait `LanguageServer` de tower-lsp. Cada documento `.ag`
//! abierto se mantiene en memoria para publicar diagnostics en tiempo real.

use std::collections::HashMap;

use ag_dsl::Diagnostic as AgDiag;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, MarkupContent, MarkupKind, MessageType, Position, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};

/// Estado interno del servidor LSP.
pub struct Backend {
    client: Client,
    /// Texto actual de cada documento abierto, indexado por URI.
    documents: Mutex<HashMap<Url, String>>,
}

impl Backend {
    /// Crea un nuevo Backend con el cliente LSP dado.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let lsp_diags: Vec<Diagnostic> = ag_dsl::lint(source)
            .iter()
            .map(|d| ag_diag_to_lsp(source, d))
            .collect();
        self.client.publish_diagnostics(uri, lsp_diags, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ag-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ag-lsp inicializado")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents.lock().await.insert(uri.clone(), text.clone());
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.documents.lock().await.insert(uri.clone(), text.clone());
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let source = self
            .documents
            .lock()
            .await
            .get(&uri)
            .cloned()
            .unwrap_or_default();

        let mut items = static_completion_items();
        if let Ok(schema) = ag_dsl::compile(&source) {
            for model in &schema.models {
                items.push(CompletionItem {
                    label: model.name.value.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some("Modelo AG".into()),
                    ..Default::default()
                });
            }
        }
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let source = match self.documents.lock().await.get(&uri).cloned() {
            Some(s) => s,
            None => return Ok(None),
        };

        let content = word_at_position(&source, &pos).and_then(hover_content_for_word);
        Ok(content.map(|text| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text,
            }),
            range: None,
        }))
    }
}

// ---- helpers internos ------------------------------------------------

fn ag_diag_to_lsp(source: &str, d: &AgDiag) -> Diagnostic {
    let range = span_to_range(source, d.span.start, d.span.end);
    let severity = if d.is_error() {
        DiagnosticSeverity::ERROR
    } else {
        DiagnosticSeverity::WARNING
    };
    let mut message = d.message.clone();
    if let Some(hint) = &d.hint {
        message.push_str(&format!("\nAyuda: {hint}"));
    }
    Diagnostic {
        range,
        severity: Some(severity),
        message,
        source: Some("ag-lsp".to_string()),
        ..Default::default()
    }
}

fn span_to_range(source: &str, start: usize, end: usize) -> Range {
    Range {
        start: byte_to_position(source, start),
        end: byte_to_position(source, end),
    }
}

fn byte_to_position(source: &str, offset: usize) -> Position {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let last_nl = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = before[last_nl..].chars().count() as u32;
    Position { line, character }
}

fn word_at_position<'a>(source: &'a str, pos: &Position) -> Option<&'a str> {
    let line = source.lines().nth(pos.line as usize)?;
    let ch = (pos.character as usize).min(line.len());
    let start = line[..ch]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '@')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = line[ch..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| ch + i)
        .unwrap_or(line.len());
    if start >= end {
        None
    } else {
        Some(&line[start..end])
    }
}

fn hover_content_for_word(word: &str) -> Option<String> {
    let s = match word {
        "UUID" => "**UUID** — Identificador unico universal v4. SQL: `UUID`.",
        "String" => "**String** — Cadena de texto UTF-8. SQL: `TEXT`.",
        "Int" => "**Int** — Entero de 64 bits con signo. SQL: `BIGINT`.",
        "Float" => "**Float** — Punto flotante 64 bits. SQL: `DOUBLE PRECISION`.",
        "Bool" => "**Bool** — Valor booleano. SQL: `BOOLEAN`.",
        "DateTime" => "**DateTime** — Fecha y hora UTC. SQL: `TIMESTAMPTZ`.",
        "@primary" => "**@primary** — Clave primaria de la tabla.",
        "@unique" => "**@unique** — Restriccion UNIQUE en la columna.",
        "@auto" => "**@auto** — Valor generado automaticamente (UUID o serial).",
        "@auto_update" => "**@auto_update** — Se actualiza automaticamente al modificar la fila.",
        "@default" => "**@default(valor)** — Valor por defecto de la columna.",
        "@min" => "**@min(n)** — En strings: longitud minima. En numeros: valor minimo.",
        "@max" => "**@max(n)** — En strings: longitud maxima. En numeros: valor maximo.",
        "@email" => "**@email** — Valida formato de correo electronico (RFC 5322).",
        "@regex" => "**@regex(\"patron\")** — Valida contra expresion regular.",
        "@length" => "**@length(n)** — Longitud exacta del string.",
        "@relation" => "**@relation(campo_fk)** — Relacion virtual. No genera columna SQL.",
        "@references" => "**@references(Modelo.campo)** — Clave foranea hacia otro modelo.",
        _ => return None,
    };
    Some(s.to_string())
}

fn static_completion_items() -> Vec<CompletionItem> {
    let keywords: &[(&str, &str)] = &[
        ("model", "Definicion de modelo de datos"),
        ("endpoint", "Definicion de endpoint HTTP"),
        ("request", "Tipo de cuerpo de peticion"),
        ("response", "Tipo de cuerpo de respuesta"),
        ("error", "Tipo de error HTTP"),
        ("config", "Bloque de configuracion del proyecto"),
        ("GET", "Metodo HTTP GET"),
        ("POST", "Metodo HTTP POST"),
        ("PUT", "Metodo HTTP PUT"),
        ("PATCH", "Metodo HTTP PATCH"),
        ("DELETE", "Metodo HTTP DELETE"),
    ];
    let types: &[(&str, &str)] = &[
        ("UUID", "Identificador unico universal"),
        ("String", "Cadena de texto UTF-8"),
        ("Int", "Entero de 64 bits con signo"),
        ("Float", "Punto flotante de 64 bits"),
        ("Bool", "Valor booleano"),
        ("DateTime", "Fecha y hora UTC"),
    ];
    let annotations: &[(&str, &str)] = &[
        ("@primary", "Clave primaria"),
        ("@unique", "Indice unico"),
        ("@auto", "Valor generado automaticamente"),
        ("@auto_update", "Actualizado automaticamente"),
        ("@default", "@default(valor) — valor por defecto"),
        ("@min", "@min(n) — valor minimo o longitud minima"),
        ("@max", "@max(n) — valor maximo o longitud maxima"),
        ("@email", "Valida formato de email"),
        ("@regex", "@regex(\"patron\") — expresion regular"),
        ("@length", "@length(n) — longitud exacta"),
        ("@relation", "@relation(campo_fk) — relacion virtual"),
        ("@references", "@references(Modelo.campo) — clave foranea"),
    ];

    let mut items = Vec::new();
    for (label, detail) in keywords {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    for (label, detail) in types {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    for (label, detail) in annotations {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_to_position_first_line() {
        let src = "model User {}";
        let pos = byte_to_position(src, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn byte_to_position_second_line() {
        let src = "model User {\n    id UUID\n}";
        let pos = byte_to_position(src, 14);
        assert_eq!(pos.line, 1);
    }

    #[test]
    fn word_at_position_returns_type_name() {
        let src = "    id UUID @primary";
        let pos = Position {
            line: 0,
            character: 8,
        };
        assert_eq!(word_at_position(src, &pos), Some("UUID"));
    }

    #[test]
    fn word_at_position_returns_annotation() {
        let src = "    id UUID @primary";
        let pos = Position {
            line: 0,
            character: 14,
        };
        assert_eq!(word_at_position(src, &pos), Some("@primary"));
    }

    #[test]
    fn hover_content_known_type() {
        assert!(hover_content_for_word("UUID").is_some());
        assert!(hover_content_for_word("DateTime").is_some());
    }

    #[test]
    fn hover_content_known_annotation() {
        assert!(hover_content_for_word("@primary").is_some());
        assert!(hover_content_for_word("@references").is_some());
    }

    #[test]
    fn hover_content_unknown_is_none() {
        assert!(hover_content_for_word("foobar").is_none());
    }

    #[test]
    fn static_items_include_keywords_types_annotations() {
        let items = static_completion_items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"model"));
        assert!(labels.contains(&"UUID"));
        assert!(labels.contains(&"@primary"));
        assert!(labels.contains(&"@references"));
    }

    #[test]
    fn ag_diag_error_maps_to_lsp_error() {
        let src = "model Bad { id UUID @primary @auto @min(1) }";
        let diags = ag_dsl::lint(src);
        let err = diags.iter().find(|d| d.is_error()).expect("debe haber error");
        let lsp = ag_diag_to_lsp(src, err);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn ag_diag_warning_maps_to_lsp_warning() {
        let src = "model Tag { name String }";
        let diags = ag_dsl::lint(src);
        let warn = diags
            .iter()
            .find(|d| !d.is_error())
            .expect("debe haber warning");
        let lsp = ag_diag_to_lsp(src, warn);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::WARNING));
    }
}
