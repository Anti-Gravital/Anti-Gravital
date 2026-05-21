//! Tokenizador del Anti-DSL generado con logos.
//!
//! Convierte texto fuente `.ag` en tokens posicionales. Los comentarios
//! (`#` hasta fin de linea) y el whitespace se omiten automaticamente.
//! Los tokens invalidos se acumulan como errores de lex sin detener el
//! proceso; el parser recibe solo los tokens validos.

use logos::Logos;
use std::ops::Range;

/// Rango de bytes en el texto fuente (byte offset, no char offset).
pub type Span = Range<usize>;

/// Tokens del Anti-DSL v0.1–v0.2.
///
/// Los `#[token(...)]` tienen prioridad sobre los `#[regex(...)]`, por lo que
/// las palabras clave siempre se reconocen antes que el identificador generico.
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n\f]+")] // whitespace
#[logos(skip r"#[^\r\n]*")] // comentarios de linea
pub enum Token {
    // ---- Palabras clave de estructura ----
    /// Definicion de modelo: `model Nombre { ... }`
    #[token("model")]
    Model,
    /// Bloque de configuracion: `config { ... }`
    #[token("config")]
    Config,
    /// Definicion de enum (reservado para DSL v0.2+).
    #[token("enum")]
    Enum,

    // ---- Tipos primitivos de campo ----
    /// `UUID` — uuid::Uuid en Rust, string (uuid) en TypeScript.
    #[token("UUID")]
    TyUuid,
    /// `String` — String en Rust, string en TypeScript.
    #[token("String")]
    TyString,
    /// `Int` — i64 en Rust, number en TypeScript.
    #[token("Int")]
    TyInt,
    /// `Float` — f64 en Rust, number en TypeScript.
    #[token("Float")]
    TyFloat,
    /// `Bool` — bool en Rust, boolean en TypeScript.
    #[token("Bool")]
    TyBool,
    /// `Timestamp` — chrono::DateTime<Utc> en Rust, string (date-time) en TypeScript.
    #[token("Timestamp")]
    TyTimestamp,
    /// `Decimal` — rust_decimal::Decimal en Rust, string (decimal) en TypeScript.
    #[token("Decimal")]
    TyDecimal,

    // ---- Palabras clave DSL v0.2 ----
    /// `endpoint` — define un endpoint HTTP: metodo, path, body, response, errores.
    #[token("endpoint")]
    Endpoint,
    /// `request` — define el cuerpo de una peticion HTTP.
    #[token("request")]
    Request,
    /// `response` — define el cuerpo de una respuesta HTTP.
    #[token("response")]
    Response,
    /// `error` — define un tipo de error con codigo HTTP y mensaje.
    #[token("error")]
    ErrorKw,

    // ---- Metodos HTTP (DSL v0.2) ----
    /// Metodo HTTP GET.
    #[token("GET")]
    HttpGet,
    /// Metodo HTTP POST.
    #[token("POST")]
    HttpPost,
    /// Metodo HTTP PUT.
    #[token("PUT")]
    HttpPut,
    /// Metodo HTTP PATCH.
    #[token("PATCH")]
    HttpPatch,
    /// Metodo HTTP DELETE.
    #[token("DELETE")]
    HttpDelete,

    // ---- Anotaciones DSL v0.1 ----
    /// `@primary` — clave primaria de la tabla.
    #[token("@primary")]
    AtPrimary,
    /// `@unique` — restriccion UNIQUE en la base de datos.
    #[token("@unique")]
    AtUnique,
    /// `@auto` — valor generado automaticamente (uuid, serial, NOW()).
    #[token("@auto")]
    AtAuto,
    /// `@auto_update` — actualizado automaticamente en cada UPDATE.
    #[token("@auto_update")]
    AtAutoUpdate,
    /// `@default(valor)` — valor por defecto.
    #[token("@default")]
    AtDefault,

    // ---- Anotaciones DSL v0.3 — validacion ----
    /// `@min(N)` — longitud minima (String) o valor minimo (numeros).
    #[token("@min")]
    AtMin,
    /// `@max(N)` — longitud maxima (String) o valor maximo (numeros).
    #[token("@max")]
    AtMax,
    /// `@email` — valida que el valor sea un email valido.
    #[token("@email")]
    AtEmail,
    /// `@regex("patron")` — valida contra una expresion regular.
    #[token("@regex")]
    AtRegex,
    /// `@length(N)` — longitud exacta de caracteres (solo String).
    #[token("@length")]
    AtLength,

    // ---- Literales ----
    /// Literal entero: `42`, `255`, `0`.
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    IntLit(i64),

    /// Literal de path HTTP: `/users`, `/users/{id}`, `/a/b/{c}`.
    /// Empieza siempre con `/`; captura letras, digitos, `_`, `-`, `{`, `}`, `.`.
    #[regex(r"/[a-zA-Z0-9_\-{}/\.]*", |lex| lex.slice().to_owned())]
    PathLit(String),

    /// Literal string entre comillas dobles: `"hola"`.
    /// Los escapes basicos (`\"`, `\\`) estan soportados en el regex;
    /// la validacion de escapes complejos pertenece al semantic pass.
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len() - 1].to_owned()
    })]
    StringLit(String),

    /// Literal booleano `true`.
    #[token("true")]
    True,
    /// Literal booleano `false`.
    #[token("false")]
    False,

    // ---- Identificador generico ----
    /// Nombre de modelo, campo, variante de enum, etc.
    /// Debe ir despues de todos los `#[token]` para no capturar keywords.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    // ---- Puntuacion ----
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
    /// Campo opcional: `Nombre String?`
    #[token("?")]
    Question,
}

/// Resultado de tokenizar el texto fuente.
///
/// Retorna dos vectores:
/// - `tokens`: tokens validos con su span de bytes.
/// - `errors`: spans de caracteres no reconocidos.
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
        // '@' solo no es un token valido; '@primary' etc si.
        assert!(!errs.is_empty());
        // Los tokens validos si se extraen
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
}
