//! Parser del Anti-DSL usando chumsky 0.9.
//!
//! Convierte un stream de tokens (producido por el lexer) en un AST.
//! Usa `parse_recovery` para reportar multiples errores en una sola
//! pasada en lugar de detenerse en el primero.
//!
//! `Simple<Token>` de chumsky contiene un HashSet con los tokens esperados.
//! Con Token que tiene variantes String, el tamano del Err supera 128 bytes.
//! Este es el comportamiento esperado de chumsky 0.9 y no puede reducirse
//! sin cambiar el tipo de error del crate.
#![allow(clippy::result_large_err)]

use chumsky::prelude::*;

use crate::ast::{
    Annotation, Config, DefaultValue, FieldDef, FieldType, ModelDef, Schema, Spanned,
};
use crate::diagnostics::Diagnostic;
use crate::lexer::{Span, Token};

// Alias de tipo interno.
type ParseErr = Simple<Token>;

/// Parsea un stream de tokens en un `Schema`.
///
/// Retorna el AST (posiblemente parcial en presencia de errores) y la
/// lista de diagnosticos de parse acumulados.
pub fn parse_tokens(
    tokens: Vec<(Token, Span)>,
    source_len: usize,
) -> (Option<Schema>, Vec<Diagnostic>) {
    let end_span = source_len..source_len;

    let stream = chumsky::Stream::from_iter(end_span, tokens.into_iter());

    let (ast, errors) = schema_parser().parse_recovery_verbose(stream);

    let diagnostics: Vec<Diagnostic> = errors.into_iter().map(parse_error_to_diagnostic).collect();

    (ast, diagnostics)
}

// ---- Construccion del parser ----------------------------------------

/// Parser principal: lee cero o mas items (model o config) hasta EOF.
fn schema_parser() -> impl Parser<Token, Schema, Error = ParseErr> {
    let model = model_parser();
    let config = config_parser();

    let item = choice((config.map(SchemaItem::Config), model.map(SchemaItem::Model)))
        .recover_with(skip_then_retry_until([Token::Model, Token::Config]));

    item.repeated().then_ignore(end()).map(|items| {
        let mut schema = Schema::default();
        for item in items {
            match item {
                SchemaItem::Config(c) => schema.config = Some(c),
                SchemaItem::Model(m) => schema.models.push(m),
            }
        }
        schema
    })
}

/// Items posibles en el schema (union interna del parser).
enum SchemaItem {
    Config(Config),
    Model(ModelDef),
}

/// Parser de bloque `config { ... }`.
fn config_parser() -> impl Parser<Token, Config, Error = ParseErr> {
    just(Token::Config)
        .ignore_then(
            config_field_parser()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|fields| {
            let mut config = Config::default();
            for (key, value) in fields {
                match key.as_str() {
                    "project_name" => config.project_name = Some(value),
                    "database" => config.database = Some(value),
                    _ => {} // campos desconocidos ignorados en parse; reportados en semantic
                }
            }
            config
        })
        .labelled("bloque config")
}

/// Par clave-valor en el bloque config.
fn config_field_parser() -> impl Parser<Token, (String, String), Error = ParseErr> {
    let key = select! { Token::Ident(s) => s }.labelled("clave de config");

    let value = choice((
        select! { Token::StringLit(s) => s },
        select! { Token::Ident(s) => s },
        select! { Token::IntLit(n) => n.to_string() },
    ))
    .labelled("valor de config");

    key.then(value)
}

/// Parser de definicion de modelo: `model Nombre { campos... }`.
fn model_parser() -> impl Parser<Token, ModelDef, Error = ParseErr> {
    let model_name = just(Token::Model)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del modelo"));

    model_name
        .map_with_span(|name, span: Span| Spanned::new(name, span))
        .then(field_parser().repeated().collect::<Vec<_>>().delimited_by(
            just(Token::LBrace).labelled("{"),
            just(Token::RBrace).labelled("}"),
        ))
        .map_with_span(|(name, fields), span| ModelDef { name, fields, span })
        .labelled("definicion de modelo")
}

/// Parser de campo: `nombre Tipo @anotacion1 @anotacion2(arg) ...`
fn field_parser() -> impl Parser<Token, FieldDef, Error = ParseErr> {
    let field_name = select! { Token::Ident(s) => s }
        .map_with_span(|s, span: Span| Spanned::new(s, span))
        .labelled("nombre de campo");

    let field_type = choice((
        just(Token::TyUuid).to(FieldType::Uuid),
        just(Token::TyString).to(FieldType::String),
        just(Token::TyInt).to(FieldType::Int),
        just(Token::TyFloat).to(FieldType::Float),
        just(Token::TyBool).to(FieldType::Bool),
        just(Token::TyTimestamp).to(FieldType::Timestamp),
        just(Token::TyDecimal).to(FieldType::Decimal),
    ))
    .map_with_span(|t, span: Span| Spanned::new(t, span))
    .labelled("tipo de campo");

    let optional = just(Token::Question).or_not().map(|q| q.is_some());

    let annotations = annotation_parser().repeated().collect::<Vec<_>>();

    field_name
        .then(field_type)
        .then(optional)
        .then(annotations)
        .map(|(((name, ty), optional), annotations)| FieldDef {
            name,
            ty,
            optional,
            annotations,
        })
}

/// Parser de anotaciones individuales.
fn annotation_parser() -> impl Parser<Token, Spanned<Annotation>, Error = ParseErr> {
    let default_value = choice((
        select! { Token::IntLit(n) => DefaultValue::Int(n) },
        select! { Token::StringLit(s) => DefaultValue::String(s) },
        just(Token::True).to(DefaultValue::Bool(true)),
        just(Token::False).to(DefaultValue::Bool(false)),
        select! { Token::Ident(s) => DefaultValue::Ident(s) },
    ))
    .labelled("valor de @default");

    let at_default = just(Token::AtDefault)
        .ignore_then(default_value.delimited_by(just(Token::LParen), just(Token::RParen)))
        .map(Annotation::Default);

    choice((
        just(Token::AtPrimary).to(Annotation::Primary),
        just(Token::AtUnique).to(Annotation::Unique),
        just(Token::AtAutoUpdate).to(Annotation::AutoUpdate), // primero: mas especifico
        just(Token::AtAuto).to(Annotation::Auto),
        at_default,
    ))
    .map_with_span(|ann, span: Span| Spanned::new(ann, span))
    .labelled("anotacion")
}

// ---- Conversion de errores de chumsky a Diagnostic ------------------

fn parse_error_to_diagnostic(err: ParseErr) -> Diagnostic {
    let span = err.span();

    let msg = match err.reason() {
        chumsky::error::SimpleReason::Unexpected => {
            if let Some(found) = err.found() {
                format!("token inesperado: {:?}", found)
            } else {
                "fin de archivo inesperado".to_owned()
            }
        }
        chumsky::error::SimpleReason::Unclosed { span: _, delimiter } => {
            format!("delimitador sin cerrar: {:?}", delimiter)
        }
        chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
    };

    let expected_tokens: Vec<_> = err
        .expected()
        .filter_map(|e| e.as_ref())
        .map(|t| format!("{t:?}"))
        .collect();

    let hint = if !expected_tokens.is_empty() {
        Some(format!(
            "se esperaba uno de: {}",
            expected_tokens.join(", ")
        ))
    } else {
        None
    };

    let mut d = Diagnostic::parse_error(span, msg);
    d.hint = hint;
    d
}

// ---- Tests ----------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse(src: &str) -> (Option<Schema>, Vec<Diagnostic>) {
        let (tokens, lex_errors) = tokenize(src);
        let mut diagnostics: Vec<Diagnostic> =
            lex_errors.into_iter().map(Diagnostic::lex_error).collect();
        let (ast, parse_diags) = parse_tokens(tokens, src.len());
        diagnostics.extend(parse_diags);
        (ast, diagnostics)
    }

    #[test]
    fn empty_schema_parses() {
        let (ast, diags) = parse("");
        assert!(ast.is_some());
        assert!(diags.is_empty());
        assert!(ast.unwrap().models.is_empty());
    }

    #[test]
    fn simple_model_no_errors() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#;
        let (ast, diags) = parse(src);
        assert!(
            diags.iter().all(|d| !d.is_error()),
            "parse errors: {diags:?}"
        );
        let schema = ast.expect("should produce AST");
        assert_eq!(schema.models.len(), 1);
        let model = &schema.models[0];
        assert_eq!(model.name.value, "User");
        assert_eq!(model.fields.len(), 3);
        assert_eq!(model.fields[0].name.value, "id");
        assert_eq!(model.fields[0].ty.value, FieldType::Uuid);
        assert_eq!(model.fields[1].name.value, "email");
        assert_eq!(model.fields[2].name.value, "name");
    }

    #[test]
    fn annotations_parsed_correctly() {
        let src = r#"
model Item {
    id    UUID @primary @auto
    score Int  @unique @default(0)
    active Bool @default(true)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let model = &ast.unwrap().models[0];
        let id_field = &model.fields[0];
        assert!(id_field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Primary));
        assert!(id_field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Auto));

        let score_field = &model.fields[1];
        let has_default_0 = score_field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Default(DefaultValue::Int(0)));
        assert!(has_default_0);
    }

    #[test]
    fn optional_field_parsed() {
        let src = r#"
model Profile {
    id  UUID   @primary @auto
    bio String?
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let model = &ast.unwrap().models[0];
        assert!(!model.fields[0].optional);
        assert!(model.fields[1].optional);
    }

    #[test]
    fn config_block_parsed() {
        let src = r#"
config {
    project_name "mi-api"
    database "postgres"
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let config = ast.unwrap().config.expect("debe tener config");
        assert_eq!(config.project_name.as_deref(), Some("mi-api"));
        assert_eq!(config.database.as_deref(), Some("postgres"));
    }

    #[test]
    fn multiple_models() {
        let src = r#"
model User {
    id UUID @primary @auto
}

model Post {
    id UUID @primary @auto
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        assert_eq!(ast.unwrap().models.len(), 2);
    }

    #[test]
    fn missing_brace_produces_error() {
        let src = "model User { id UUID @primary @auto";
        let (_, diags) = parse(src);
        assert!(
            diags.iter().any(|d| d.is_error()),
            "should have parse error"
        );
    }
}
