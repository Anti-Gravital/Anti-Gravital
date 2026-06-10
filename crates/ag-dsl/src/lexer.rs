//! Anti-DSL tokenizer generated with logos.
//!
//! Converts `.ag` source text into positional tokens. Comments
//! (`#` until end of line) and whitespace are skipped automatically.
//! Invalid tokens are accumulated as lex errors without stopping the
//! process; the parser receives only the valid tokens.

use logos::Logos;
use std::ops::Range;

/// Byte range in the source text (byte offset, not char offset).
pub type Span = Range<usize>;

/// Anti-DSL v0.1–v0.2 tokens.
///
/// The `#[token(...)]` rules take precedence over `#[regex(...)]`, so that
/// keywords are always recognized before the generic identifier.
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n\f]+")] // whitespace
#[logos(skip r"#[^\r\n]*")] // line comments
pub enum Token {
    // ---- Structure keywords ----
    /// Model definition: `model Name { ... }`
    #[token("model")]
    Model,
    /// Configuration block: `config { ... }`
    #[token("config")]
    Config,
    /// Enum definition (reserved for DSL v0.2+).
    #[token("enum")]
    Enum,

    // ---- Primitive field types ----
    /// `UUID` — uuid::Uuid in Rust, string (uuid) in TypeScript.
    #[token("UUID")]
    TyUuid,
    /// `String` — String in Rust, string in TypeScript.
    #[token("String")]
    TyString,
    /// `Int` — i64 in Rust, number in TypeScript.
    #[token("Int")]
    TyInt,
    /// `Float` — f64 in Rust, number in TypeScript.
    #[token("Float")]
    TyFloat,
    /// `Bool` — bool in Rust, boolean in TypeScript.
    #[token("Bool")]
    TyBool,
    /// `Timestamp` — chrono::DateTime<Utc> in Rust, string (date-time) in TypeScript.
    #[token("Timestamp")]
    TyTimestamp,
    /// `Decimal` — rust_decimal::Decimal in Rust, string (decimal) in TypeScript.
    #[token("Decimal")]
    TyDecimal,

    // ---- DSL v0.2 keywords ----
    /// `endpoint` — defines an HTTP endpoint: method, path, body, response, errors.
    #[token("endpoint")]
    Endpoint,
    /// `request` — defines the body of an HTTP request.
    #[token("request")]
    Request,
    /// `response` — defines the body of an HTTP response.
    #[token("response")]
    Response,
    /// `error` — defines an error type with HTTP code and message.
    #[token("error")]
    ErrorKw,

    // ---- HTTP methods (DSL v0.2) ----
    /// HTTP GET method.
    #[token("GET")]
    HttpGet,
    /// HTTP POST method.
    #[token("POST")]
    HttpPost,
    /// HTTP PUT method.
    #[token("PUT")]
    HttpPut,
    /// HTTP PATCH method.
    #[token("PATCH")]
    HttpPatch,
    /// HTTP DELETE method.
    #[token("DELETE")]
    HttpDelete,

    // ---- DSL v0.1 annotations ----
    /// `@primary` — primary key of the table.
    #[token("@primary")]
    AtPrimary,
    /// `@unique` — UNIQUE constraint in the database.
    #[token("@unique")]
    AtUnique,
    /// `@auto` — automatically generated value (uuid, serial, NOW()).
    #[token("@auto")]
    AtAuto,
    /// `@auto_update` — automatically updated on every UPDATE.
    #[token("@auto_update")]
    AtAutoUpdate,
    /// `@default(value)` — default value.
    #[token("@default")]
    AtDefault,

    // ---- DSL v0.3 annotations — validation ----
    /// `@min(N)` — minimum length (String) or minimum value (numbers).
    #[token("@min")]
    AtMin,

    /// `@max(N)` — maximum length (String) or maximum value (numbers).
    #[token("@max")]
    AtMax,
    /// `@email` — validates that the value is a valid email.
    #[token("@email")]
    AtEmail,
    /// `@regex("pattern")` — validates against a regular expression.
    #[token("@regex")]
    AtRegex,
    /// `@length(N)` — exact character length (String only).
    #[token("@length")]
    AtLength,

    // ---- DSL v0.4 annotations — relations ----
    /// `@references` — foreign key to another model.
    #[token("@references")]
    AtReferences,
    /// `@relation` — virtual relation field.
    #[token("@relation")]
    AtRelation,

    // ---- DSL v0.7 keywords — mail and domains ----
    /// `mail` — transactional mail configuration block.
    #[token("mail")]
    Mail,
    /// `domain` — DNS configuration block for a domain.
    #[token("domain")]
    Domain,
    /// `template` — mail template sub-block inside a `mail` block.
    #[token("template")]
    Template,

    // ---- DSL v0.5+v0.6 keywords — authentication and events ----
    /// `auth` — authentication/authorization block on an endpoint or model.
    #[token("auth")]
    Auth,
    /// `required` — the field or block is mandatory.
    #[token("required")]
    Required,
    /// `optional` — the field or block is optional.
    #[token("optional")]
    Optional,
    /// `policy` — defines an access control policy.
    #[token("policy")]
    Policy,
    /// `event` — defines a domain event emitted by the system.
    #[token("event")]
    Event,
    /// `retain` — indicates the data retention policy of an event.
    #[token("retain")]
    Retain,

    // ---- DSL v0.8 keyword — background workers ----
    /// `worker` — declares a background job worker (ag-workers, RFC-0012).
    #[token("worker")]
    Worker,

    // ---- Literals ----
    /// Integer literal: `42`, `255`, `0`.
    /// If the value exceeds i64::MAX, logos discards the token and emits a lex error.
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLit(i64),

    /// HTTP path literal: `/users`, `/users/{id}`, `/a/b/{c}`.
    /// Always starts with `/`; captures letters, digits, `_`, `-`, `{`, `}`, `.`.
    #[regex(r"/[a-zA-Z0-9_\-{}/\.]*", |lex| lex.slice().to_owned())]
    PathLit(String),

    /// String literal between double quotes: `"hola"`.
    /// Basic escapes (`\"`, `\\`) are supported in the regex;
    /// validation of complex escapes belongs to the semantic pass.
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len() - 1].to_owned()
    })]
    StringLit(String),

    /// Boolean literal `true`.
    #[token("true")]
    True,
    /// Boolean literal `false`.
    #[token("false")]
    False,

    // ---- Generic identifier ----
    /// Name of a model, field, enum variant, etc.
    /// Must come after all `#[token]` rules so it does not capture keywords.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    // ---- Punctuation ----
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(",")]
    Comma,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    /// Optional field: `Name String?`
    #[token("?")]
    Question,
    /// Dot: separator in `@references(Model.field)` and `@relation(model.field)`.
    #[token(".")]
    Dot,
}

/// Result of tokenizing the source text.
///
/// Returns two vectors:
/// - `tokens`: valid tokens with their byte span.
/// - `errors`: spans of unrecognized characters.
pub fn tokenize(source: &str) -> (Vec<(Token, Span)>, Vec<Span>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (result, span) in Token::lexer(source).spanned() {
        match result {
            Ok(tok) => tokens.push((tok, span)),
            Err(_) => errors.push(span),
        }
    }
    (tokens, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        let (toks, errs) = tokenize(src);
        assert!(errs.is_empty(), "lex errors: {errs:?}");
        toks.into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn keywords_not_captured_as_ident() {
        let toks = lex("model config enum");
        assert_eq!(toks, vec![Token::Model, Token::Config, Token::Enum]);
    }

    #[test]
    fn type_keywords() {
        let toks = lex("UUID String Int Float Bool Timestamp Decimal");
        assert_eq!(
            toks,
            vec![
                Token::TyUuid,
                Token::TyString,
                Token::TyInt,
                Token::TyFloat,
                Token::TyBool,
                Token::TyTimestamp,
                Token::TyDecimal,
            ]
        );
    }

    #[test]
    fn annotations_v01() {
        let toks = lex("@primary @unique @auto @auto_update @default");
        assert_eq!(
            toks,
            vec![
                Token::AtPrimary,
                Token::AtUnique,
                Token::AtAuto,
                Token::AtAutoUpdate,
                Token::AtDefault,
            ]
        );
    }

    #[test]
    fn auto_does_not_eat_auto_update() {
        let toks = lex("@auto_update @auto");
        assert_eq!(toks, vec![Token::AtAutoUpdate, Token::AtAuto]);
    }

    #[test]
    fn identifiers() {
        let toks = lex("User email_address _private CamelCase");
        assert_eq!(
            toks,
            vec![
                Token::Ident("User".to_owned()),
                Token::Ident("email_address".to_owned()),
                Token::Ident("_private".to_owned()),
                Token::Ident("CamelCase".to_owned()),
            ]
        );
    }

    #[test]
    fn integer_literals() {
        let toks = lex("0 42 255");
        assert_eq!(
            toks,
            vec![Token::IntLit(0), Token::IntLit(42), Token::IntLit(255),]
        );
    }

    #[test]
    fn string_literal() {
        let toks = lex(r#""hello world""#);
        assert_eq!(toks, vec![Token::StringLit("hello world".to_owned())]);
    }

    #[test]
    fn boolean_literals() {
        let toks = lex("true false");
        assert_eq!(toks, vec![Token::True, Token::False]);
    }

    #[test]
    fn punctuation() {
        let toks = lex("{ } ( ) , [ ] ?");
        assert_eq!(
            toks,
            vec![
                Token::LBrace,
                Token::RBrace,
                Token::LParen,
                Token::RParen,
                Token::Comma,
                Token::LBracket,
                Token::RBracket,
                Token::Question,
            ]
        );
    }

    #[test]
    fn comments_are_skipped() {
        let toks = lex("model # esto es un comentario\nUser");
        assert_eq!(toks, vec![Token::Model, Token::Ident("User".to_owned()),]);
    }

    #[test]
    fn whitespace_is_skipped() {
        let toks = lex("  model\t\nUser  ");
        assert_eq!(toks, vec![Token::Model, Token::Ident("User".to_owned()),]);
    }

    #[test]
    fn invalid_char_produces_error() {
        let (toks, errs) = tokenize("model @ User");
        // '@' alone is not a valid token; '@primary' etc. are.
        assert!(!errs.is_empty());
        // The valid tokens are still extracted
        assert!(toks.iter().any(|(t, _)| *t == Token::Model));
    }

    #[test]
    fn simple_model_tokenizes() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#;
        let (toks, errs) = tokenize(src);
        assert!(errs.is_empty());
        let kinds: Vec<_> = toks.into_iter().map(|(t, _)| t).collect();
        assert!(kinds.contains(&Token::Model));
        assert!(kinds.contains(&Token::TyUuid));
        assert!(kinds.contains(&Token::AtPrimary));
        assert!(kinds.contains(&Token::AtAuto));
        assert!(kinds.contains(&Token::AtUnique));
    }

    // ---- Tests DSL v0.2 ----

    #[test]
    fn v02_structure_keywords() {
        let toks = lex("endpoint request response error");
        assert_eq!(
            toks,
            vec![
                Token::Endpoint,
                Token::Request,
                Token::Response,
                Token::ErrorKw,
            ]
        );
    }

    #[test]
    fn v02_http_methods() {
        let toks = lex("GET POST PUT PATCH DELETE");
        assert_eq!(
            toks,
            vec![
                Token::HttpGet,
                Token::HttpPost,
                Token::HttpPut,
                Token::HttpPatch,
                Token::HttpDelete,
            ]
        );
    }

    #[test]
    fn v02_path_literals() {
        let toks = lex("/users /users/{id} /a/b/c");
        assert_eq!(
            toks,
            vec![
                Token::PathLit("/users".to_owned()),
                Token::PathLit("/users/{id}".to_owned()),
                Token::PathLit("/a/b/c".to_owned()),
            ]
        );
    }

    #[test]
    fn v02_root_path() {
        let toks = lex("/");
        assert_eq!(toks, vec![Token::PathLit("/".to_owned())]);
    }

    #[test]
    fn v02_keywords_not_captured_as_ident() {
        let (toks, errs) = tokenize("endpoint GET /users");
        assert!(errs.is_empty());
        let kinds: Vec<_> = toks.into_iter().map(|(t, _)| t).collect();
        assert!(kinds.contains(&Token::Endpoint));
        assert!(kinds.contains(&Token::HttpGet));
        assert!(kinds.contains(&Token::PathLit("/users".to_owned())));
    }

    // ---- Tests DSL v0.3 ----

    #[test]
    fn v03_validation_annotations() {
        let toks = lex("@min @max @email @regex @length");
        assert_eq!(
            toks,
            vec![
                Token::AtMin,
                Token::AtMax,
                Token::AtEmail,
                Token::AtRegex,
                Token::AtLength,
            ]
        );
    }

    #[test]
    fn v04_relation_tokens() {
        let toks = lex("@references @relation");
        assert_eq!(toks, vec![Token::AtReferences, Token::AtRelation]);
    }

    #[test]
    fn v04_dot_token() {
        let toks = lex("User.id");
        assert_eq!(
            toks,
            vec![
                Token::Ident("User".to_owned()),
                Token::Dot,
                Token::Ident("id".to_owned()),
            ]
        );
    }

    #[test]
    fn v04_list_type_brackets() {
        let toks = lex("Post[]");
        assert_eq!(
            toks,
            vec![
                Token::Ident("Post".to_owned()),
                Token::LBracket,
                Token::RBracket,
            ]
        );
    }

    #[test]
    fn v03_min_max_with_args() {
        let toks = lex("@min(2) @max(255)");
        assert_eq!(
            toks,
            vec![
                Token::AtMin,
                Token::LParen,
                Token::IntLit(2),
                Token::RParen,
                Token::AtMax,
                Token::LParen,
                Token::IntLit(255),
                Token::RParen,
            ]
        );
    }

    // ---- Tests DSL v0.5+v0.6 ----

    #[test]
    fn lex_auth_keywords() {
        let src = "auth required optional policy event retain";
        let (tokens, errs) = tokenize(src);
        assert!(errs.is_empty(), "sin errores lex: {:?}", errs);
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert!(kinds.contains(&Token::Auth));
        assert!(kinds.contains(&Token::Required));
        assert!(kinds.contains(&Token::Optional));
        assert!(kinds.contains(&Token::Policy));
        assert!(kinds.contains(&Token::Event));
        assert!(kinds.contains(&Token::Retain));
    }

    // ---- Tests DSL v0.7 ----

    #[test]
    fn v07_mail_domain_template_keywords() {
        let toks = lex("mail domain template");
        assert_eq!(toks, vec![Token::Mail, Token::Domain, Token::Template]);
    }

    #[test]
    fn v07_mail_block_tokenizes() {
        let src = r#"
mail transaccional {
    provider smtp
    from "noreply@ejemplo.com"
    template bienvenida {
        subject "Bienvenido {{nombre}}"
        vars [nombre, token]
    }
}
"#;
        let (toks, errs) = tokenize(src);
        assert!(errs.is_empty(), "sin errores lex: {:?}", errs);
        let kinds: Vec<_> = toks.iter().map(|(t, _)| t.clone()).collect();
        assert!(kinds.contains(&Token::Mail));
        assert!(kinds.contains(&Token::Template));
        assert!(kinds.contains(&Token::LBrace));
        assert!(kinds.contains(&Token::LBracket));
    }

    #[test]
    fn v07_domain_block_tokenizes() {
        let src = r#"
domain mi_dominio {
    name "ejemplo.com"
    provider cloudflare
    dkim_selector "s1"
    dmarc_policy quarantine
    dmarc_rua "admin@ejemplo.com"
}
"#;
        let (toks, errs) = tokenize(src);
        assert!(errs.is_empty(), "sin errores lex: {:?}", errs);
        let kinds: Vec<_> = toks.iter().map(|(t, _)| t.clone()).collect();
        assert!(kinds.contains(&Token::Domain));
    }

    #[test]
    fn v07_mail_not_captured_as_ident() {
        let toks = lex("mail");
        assert_eq!(toks, vec![Token::Mail]);
    }
}
