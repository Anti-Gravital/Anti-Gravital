//! Anti-DSL parser using chumsky 0.9.
//!
//! Converts a token stream (produced by the lexer) into an AST.
//! Uses `parse_recovery` to report multiple errors in a single
//! pass instead of stopping at the first one.
//!
//! chumsky's `Simple<Token>` contains a HashSet with the expected tokens.
//! With a Token that has String variants, the size of the Err exceeds 128 bytes.
//! This is the expected behavior of chumsky 0.9 and cannot be reduced
//! without changing the crate's error type.
#![allow(clippy::result_large_err)]

use chumsky::prelude::*;

use crate::ast::{
    Annotation, AuthMode, Config, DefaultValue, DomainBlock, EndpointDef, ErrorDef, EventDef,
    FieldDef, FieldType, HttpMethod, MailBlock, MailTemplateDef, ModelDef, RequestDef, ResponseDef,
    Schema, Spanned,
};
use crate::diagnostics::Diagnostic;
use crate::lexer::{Span, Token};

// Internal type alias.
type ParseErr = Simple<Token>;

/// Parses a token stream into a `Schema`.
///
/// Returns the AST (possibly partial in the presence of errors) and the
/// list of accumulated parse diagnostics.
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

// ---- Parser construction --------------------------------------------

/// Main parser: reads zero or more items until EOF.
fn schema_parser() -> impl Parser<Token, Schema, Error = ParseErr> {
    let item = choice((
        config_parser().map(SchemaItem::Config),
        model_parser().map(SchemaItem::Model),
        request_parser().map(SchemaItem::Request),
        response_parser().map(SchemaItem::Response),
        error_def_parser().map(SchemaItem::Error),
        endpoint_parser().map(SchemaItem::Endpoint),
        event_parser().map(SchemaItem::Event),
        mail_parser().map(SchemaItem::Mail),
        domain_parser().map(SchemaItem::Domain),
    ))
    .recover_with(skip_then_retry_until([
        Token::Model,
        Token::Config,
        Token::Request,
        Token::Response,
        Token::ErrorKw,
        Token::Endpoint,
        Token::Event,
        Token::Mail,
        Token::Domain,
    ]));

    item.repeated().then_ignore(end()).map(|items| {
        let mut schema = Schema::default();
        for item in items {
            match item {
                SchemaItem::Config(c) => schema.config = Some(c),
                SchemaItem::Model(m) => schema.models.push(m),
                SchemaItem::Request(r) => schema.requests.push(r),
                SchemaItem::Response(r) => schema.responses.push(r),
                SchemaItem::Error(e) => schema.errors.push(e),
                SchemaItem::Endpoint(ep) => schema.endpoints.push(ep),
                SchemaItem::Event(ev) => schema.events.push(ev),
                SchemaItem::Mail(mb) => schema.mails.push(mb),
                SchemaItem::Domain(db) => schema.domains.push(db),
            }
        }
        schema
    })
}

/// Possible items in the schema (internal parser union).
enum SchemaItem {
    Config(Config),
    Model(ModelDef),
    Request(RequestDef),
    Response(ResponseDef),
    Error(ErrorDef),
    Endpoint(EndpointDef),
    Event(EventDef),
    Mail(MailBlock),
    Domain(DomainBlock),
}

/// Parser for the `config { ... }` block.
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
                    _ => {} // unknown fields ignored in parse; reported in semantic
                }
            }
            config
        })
        .labelled("bloque config")
}

/// Key-value pair in the config block.
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

/// Parser for a model definition: `model Name { fields... }`.
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

/// Parser for a field: `name Type @annotation1 @annotation2(arg) ...`
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
        // v0.4: ModelRefList before ModelRef (more specific pattern first)
        select! { Token::Ident(s) => s }
            .then_ignore(just(Token::LBracket))
            .then_ignore(just(Token::RBracket))
            .map(FieldType::ModelRefList),
        select! { Token::Ident(s) => s }.map(FieldType::ModelRef),
    ))
    .map_with_span(|t, span: Span| Spanned::new(t, span))
    .labelled("tipo de campo");

    let optional = just(Token::Question).or_not().map(|q| q.is_some());

    let annotations = annotation_parser().repeated().collect::<Vec<_>>();

    field_name
        .then(field_type)
        .then(optional)
        .then(annotations)
        .map(|(((name, ty), optional), annotations)| {
            let virtual_field = matches!(
                ty.value,
                FieldType::ModelRef(_) | FieldType::ModelRefList(_)
            );
            FieldDef {
                name,
                ty,
                optional,
                annotations,
                virtual_field,
            }
        })
}

/// Parser for individual annotations.
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

    // v0.3 — annotations with an integer argument: @min(N), @max(N), @length(N)
    let int_arg =
        || select! { Token::IntLit(n) => n }.delimited_by(just(Token::LParen), just(Token::RParen));

    let at_min = just(Token::AtMin)
        .ignore_then(int_arg())
        .map(Annotation::Min);
    let at_max = just(Token::AtMax)
        .ignore_then(int_arg())
        .map(Annotation::Max);
    let at_length = just(Token::AtLength)
        .ignore_then(int_arg())
        .map(Annotation::Length);

    // v0.3 — @regex("pattern")
    let at_regex = just(Token::AtRegex)
        .ignore_then(
            select! { Token::StringLit(s) => s }
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .labelled("patron de regex"),
        )
        .map(Annotation::Regex);

    // v0.4 — @references(Model.field)
    let at_references = just(Token::AtReferences)
        .ignore_then(
            select! { Token::Ident(s) => s }
                .then_ignore(just(Token::Dot))
                .then(select! { Token::Ident(s) => s })
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .labelled("referencia Modelo.campo"),
        )
        .map(|(model, field)| Annotation::References { model, field });

    // v0.4 — @relation(field) or @relation(model.field)
    let relation_path = select! { Token::Ident(s) => s }
        .then(
            just(Token::Dot)
                .ignore_then(select! { Token::Ident(s) => s })
                .or_not(),
        )
        .map(|(first, second)| match second {
            Some(s) => format!("{first}.{s}"),
            None => first,
        });

    let at_relation = just(Token::AtRelation)
        .ignore_then(
            relation_path
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .labelled("path de relacion"),
        )
        .map(|path| Annotation::Relation { path });

    choice((
        just(Token::AtPrimary).to(Annotation::Primary),
        just(Token::AtUnique).to(Annotation::Unique),
        just(Token::AtAutoUpdate).to(Annotation::AutoUpdate), // first: more specific
        just(Token::AtAuto).to(Annotation::Auto),
        just(Token::AtEmail).to(Annotation::Email),
        at_default,
        at_min,
        at_max,
        at_length,
        at_regex,
        at_references,
        at_relation,
    ))
    .map_with_span(|ann, span: Span| Spanned::new(ann, span))
    .labelled("anotacion")
}

// ---- DSL v0.2 parsers -----------------------------------------------

/// Parser for `request Name { fields... }`.
fn request_parser() -> impl Parser<Token, RequestDef, Error = ParseErr> {
    just(Token::Request)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del request"))
        .map_with_span(|name, span: Span| Spanned::new(name, span))
        .then(field_parser().repeated().collect::<Vec<_>>().delimited_by(
            just(Token::LBrace).labelled("{"),
            just(Token::RBrace).labelled("}"),
        ))
        .map_with_span(|(name, fields), span| RequestDef { name, fields, span })
        .labelled("definicion de request")
}

/// Parser for `response Name { fields... }`.
fn response_parser() -> impl Parser<Token, ResponseDef, Error = ParseErr> {
    just(Token::Response)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del response"))
        .map_with_span(|name, span: Span| Spanned::new(name, span))
        .then(field_parser().repeated().collect::<Vec<_>>().delimited_by(
            just(Token::LBrace).labelled("{"),
            just(Token::RBrace).labelled("}"),
        ))
        .map_with_span(|(name, fields), span| ResponseDef { name, fields, span })
        .labelled("definicion de response")
}

/// Parser for `error Name { status N message "text" }`.
fn error_def_parser() -> impl Parser<Token, ErrorDef, Error = ParseErr> {
    let error_name = just(Token::ErrorKw)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del error"))
        .map_with_span(|name, span: Span| Spanned::new(name, span));

    let status_field = just(Token::Ident("status".to_owned())).ignore_then(
        select! { Token::IntLit(n) => n as u16 }
            .map_with_span(|n, span: Span| Spanned::new(n, span))
            .labelled("codigo de estado HTTP"),
    );

    let message_field = just(Token::Ident("message".to_owned())).ignore_then(
        select! { Token::StringLit(s) => s }
            .map_with_span(|s, span: Span| Spanned::new(s, span))
            .labelled("mensaje de error"),
    );

    let body = status_field
        .then(message_field)
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    error_name
        .then(body)
        .map_with_span(|(name, (status, message)), span| ErrorDef {
            name,
            status,
            message,
            span,
        })
        .labelled("definicion de error")
}

/// Parser for a dotted compound name: `user.created` or `UserResponse`.
///
/// Tokenizes as `Ident("user") Dot Ident("created")` or just `Ident("UserResponse")`.
/// Returns the full name as `Spanned<String>`.
fn dotted_ident_parser() -> impl Parser<Token, Spanned<String>, Error = ParseErr> {
    select! { Token::Ident(s) => s }
        .map_with_span(|s, span: Span| (s, span))
        .then(
            just(Token::Dot)
                .ignore_then(select! { Token::Ident(s) => s })
                .or_not(),
        )
        .map_with_span(|((first, fspan), second), full_span: Span| {
            let name = match second {
                Some(part) => format!("{first}.{part}"),
                None => first,
            };
            // Use the span of the first segment for simple names,
            // the full span for dotted names.
            Spanned::new(
                name,
                if full_span.start < full_span.end {
                    full_span
                } else {
                    fspan
                },
            )
        })
}

/// Parser for `endpoint Name { method path [body] [response] [errors] [auth] [policy] [events] }`.
fn endpoint_parser() -> impl Parser<Token, EndpointDef, Error = ParseErr> {
    let endpoint_name = just(Token::Endpoint)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del endpoint"))
        .map_with_span(|name, span: Span| Spanned::new(name, span));

    let http_method = choice((
        just(Token::HttpGet).to(HttpMethod::Get),
        just(Token::HttpPost).to(HttpMethod::Post),
        just(Token::HttpPut).to(HttpMethod::Put),
        just(Token::HttpPatch).to(HttpMethod::Patch),
        just(Token::HttpDelete).to(HttpMethod::Delete),
    ))
    .map_with_span(|m, span: Span| Spanned::new(m, span))
    .labelled("metodo HTTP");

    let ident_ref =
        select! { Token::Ident(s) => s }.map_with_span(|s, span: Span| Spanned::new(s, span));

    let error_list = ident_ref
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .labelled("lista de errores");

    // List of event names: `[user.created, order.placed]`
    // Each element may be a dotted name (user.created).
    let emits_list = dotted_ident_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .labelled("lista de eventos");

    // auth required | auth optional
    let auth_field = just(Token::Auth)
        .ignore_then(choice((
            just(Token::Required).to(AuthMode::Required),
            just(Token::Optional).to(AuthMode::Optional),
        )))
        .map(EndpointField::Auth);

    // policy "expression"
    let policy_field = just(Token::Policy)
        .ignore_then(
            select! { Token::StringLit(s) => s }
                .map_with_span(|s, span: Span| Spanned::new(s, span))
                .labelled("expresion de policy"),
        )
        .map(EndpointField::Policy);

    // events [name.event, ...]
    let events_field = just(Token::Ident("events".to_owned()))
        .ignore_then(emits_list)
        .map(EndpointField::EmitsList);

    // Each field of the endpoint block is a key-value pair
    #[allow(clippy::type_complexity)]
    let endpoint_field = choice((
        just(Token::Ident("method".to_owned()))
            .ignore_then(http_method)
            .map(EndpointField::Method),
        just(Token::Ident("path".to_owned()))
            .ignore_then(
                select! { Token::PathLit(s) => s }
                    .map_with_span(|s, span: Span| Spanned::new(s, span))
                    .labelled("path HTTP"),
            )
            .map(EndpointField::Path),
        just(Token::Ident("body".to_owned()))
            .ignore_then(ident_ref)
            .map(EndpointField::Body),
        just(Token::Response) // 'response' is a keyword
            .ignore_then(ident_ref)
            .map(EndpointField::Response),
        just(Token::Ident("errors".to_owned()))
            .ignore_then(error_list)
            .map(EndpointField::Errors),
        auth_field,
        policy_field,
        events_field,
    ));

    endpoint_name
        .then(
            endpoint_field
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, fields), span: Span| {
            let mut method = None;
            let mut path = None;
            let mut body = None;
            let mut response = None;
            let mut errors = Vec::new();
            let mut auth = AuthMode::None;
            let mut policy = None;
            let mut emits = Vec::new();
            for f in fields {
                match f {
                    EndpointField::Method(m) => method = Some(m),
                    EndpointField::Path(p) => path = Some(p),
                    EndpointField::Body(b) => body = Some(b),
                    EndpointField::Response(r) => response = Some(r),
                    EndpointField::Errors(e) => errors = e,
                    EndpointField::Auth(a) => auth = a,
                    EndpointField::Policy(p) => policy = Some(p),
                    EndpointField::EmitsList(ev) => emits = ev,
                }
            }
            EndpointDef {
                name,
                method: method.unwrap_or_else(|| Spanned::new(HttpMethod::Get, span.clone())),
                path: path.unwrap_or_else(|| Spanned::new("/".to_owned(), span.clone())),
                body,
                response,
                errors,
                auth,
                policy,
                emits,
                span,
            }
        })
        .labelled("definicion de endpoint")
}

/// Possible fields inside an endpoint block.
enum EndpointField {
    Method(Spanned<HttpMethod>),
    Path(Spanned<String>),
    Body(Spanned<String>),
    Response(Spanned<String>),
    Errors(Vec<Spanned<String>>),
    Auth(AuthMode),
    Policy(Spanned<String>),
    EmitsList(Vec<Spanned<String>>),
}

// ---- DSL v0.6 parser — events -----------------------------------------

/// Parser for the `event name { payload T [retain N] }` block.
///
/// The name may be compound: `user.created` → `Ident("user") Dot Ident("created")`.
fn event_parser() -> impl Parser<Token, crate::ast::EventDef, Error = ParseErr> {
    use crate::ast::EventDef;

    let event_name =
        just(Token::Event).ignore_then(dotted_ident_parser().labelled("nombre del evento"));

    // payload TypeName
    let payload_field = just(Token::Ident("payload".to_owned())).ignore_then(
        select! { Token::Ident(s) => s }
            .map_with_span(|s, span: Span| Spanned::new(s, span))
            .labelled("tipo de payload"),
    );

    // retain N (days)
    let retain_field = just(Token::Retain).ignore_then(
        select! { Token::IntLit(n) => n }
            .map_with_span(|n, span: Span| Spanned::new(n, span))
            .labelled("dias de retencion"),
    );

    #[allow(clippy::type_complexity)]
    let event_field = choice((
        payload_field.map(EventField::Payload),
        retain_field.map(EventField::Retain),
    ));

    event_name
        .then(
            event_field
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, fields), span: Span| {
            let mut payload: Option<Spanned<String>> = None;
            let mut retain_days: Option<u32> = None;
            for f in fields {
                match f {
                    EventField::Payload(p) => payload = Some(p),
                    EventField::Retain(r) => retain_days = Some(r.value.unsigned_abs() as u32),
                }
            }
            let payload = payload.unwrap_or_else(|| Spanned::new(String::new(), span.clone()));
            EventDef {
                name,
                payload,
                retain_days,
                span,
            }
        })
        .labelled("declaracion de evento")
}

/// Possible fields inside an event block.
enum EventField {
    Payload(Spanned<String>),
    Retain(Spanned<i64>),
}

// ============================================================
// DSL v0.7 — mail and domain parsers
// ============================================================

/// Parser for the `template name { ... }` block inside a `mail` block.
fn template_parser() -> impl Parser<Token, MailTemplateDef, Error = ParseErr> {
    let tpl_name = just(Token::Template)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del template"))
        .map_with_span(|s, span: Span| Spanned::new(s, span));

    // Key-value pair of the template body.
    let tpl_field = select! { Token::Ident(k) => k }
        .then(choice((
            select! { Token::StringLit(v) => v },
            select! { Token::Ident(v) => v },
        )))
        .map(|(k, v)| (k, v));

    // `vars [name, token, ...]`
    let vars_field = select! { Token::Ident(k) if k == "vars" => () }.ignore_then(
        select! { Token::Ident(s) => s }
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
    );

    enum TplEntry {
        Kv(String, String),
        Vars(Vec<String>),
    }

    let entry = choice((
        vars_field.map(TplEntry::Vars),
        tpl_field.map(|(k, v)| TplEntry::Kv(k, v)),
    ));

    tpl_name
        .then(
            entry
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, entries), span: Span| {
            let mut subject = None;
            let mut vars = Vec::new();
            for entry in entries {
                match entry {
                    TplEntry::Kv(k, v) if k == "subject" => subject = Some(v),
                    TplEntry::Kv(_, _) => {}
                    TplEntry::Vars(vs) => vars = vs,
                }
            }
            MailTemplateDef {
                name,
                subject,
                vars,
                span,
            }
        })
        .labelled("bloque template")
}

/// Parser for the `mail name { ... }` block.
fn mail_parser() -> impl Parser<Token, MailBlock, Error = ParseErr> {
    let mail_name = just(Token::Mail)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del bloque mail"))
        .map_with_span(|s, span: Span| Spanned::new(s, span));

    let kv_field = select! { Token::Ident(k) => k }.then(choice((
        select! { Token::StringLit(v) => v },
        select! { Token::Ident(v) => v },
    )));

    enum MailEntry {
        Kv(String, String),
        Template(MailTemplateDef),
    }

    let entry = choice((
        template_parser().map(MailEntry::Template),
        kv_field.map(|(k, v)| MailEntry::Kv(k, v)),
    ));

    mail_name
        .then(
            entry
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, entries), span: Span| {
            let mut provider = None;
            let mut from = None;
            let mut templates = Vec::new();
            for entry in entries {
                match entry {
                    MailEntry::Kv(k, v) if k == "provider" => provider = Some(v),
                    MailEntry::Kv(k, v) if k == "from" => from = Some(v),
                    MailEntry::Kv(_, _) => {}
                    MailEntry::Template(t) => templates.push(t),
                }
            }
            MailBlock {
                name,
                provider,
                from,
                templates,
                span,
            }
        })
        .labelled("bloque mail")
}

/// Parser for the `domain name { ... }` block.
fn domain_parser() -> impl Parser<Token, DomainBlock, Error = ParseErr> {
    let domain_name = just(Token::Domain)
        .ignore_then(select! { Token::Ident(s) => s }.labelled("nombre del bloque domain"))
        .map_with_span(|s, span: Span| Spanned::new(s, span));

    let kv_field = select! { Token::Ident(k) => k }.then(choice((
        select! { Token::StringLit(v) => v },
        select! { Token::Ident(v) => v },
    )));

    domain_name
        .then(
            kv_field
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, fields), span: Span| {
            let mut domain_name_val = None;
            let mut provider = None;
            let mut dkim_selectors = Vec::new();
            let mut dmarc_policy = None;
            let mut dmarc_rua = None;
            for (k, v) in fields {
                match k.as_str() {
                    "name" => domain_name_val = Some(v),
                    "provider" => provider = Some(v),
                    "dkim_selector" => dkim_selectors.push(v),
                    "dmarc_policy" => dmarc_policy = Some(v),
                    "dmarc_rua" => dmarc_rua = Some(v),
                    _ => {}
                }
            }
            DomainBlock {
                name,
                domain_name: domain_name_val,
                provider,
                dkim_selectors,
                dmarc_policy,
                dmarc_rua,
                span,
            }
        })
        .labelled("bloque domain")
}

// ---- Conversion of chumsky errors to Diagnostic ---------------------

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

    // ---- Tests DSL v0.2 ----

    #[test]
    fn request_def_parses() {
        let src = r#"
request CreateUserRequest {
    email String
    name  String
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.requests.len(), 1);
        let req = &schema.requests[0];
        assert_eq!(req.name.value, "CreateUserRequest");
        assert_eq!(req.fields.len(), 2);
    }

    #[test]
    fn response_def_parses() {
        let src = r#"
response UserResponse {
    id    UUID
    email String
    name  String
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.responses.len(), 1);
        assert_eq!(schema.responses[0].name.value, "UserResponse");
        assert_eq!(schema.responses[0].fields.len(), 3);
    }

    #[test]
    fn error_def_parses() {
        let src = r#"error EmailTaken { status 409 message "Email already registered" }"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.errors.len(), 1);
        let err = &schema.errors[0];
        assert_eq!(err.name.value, "EmailTaken");
        assert_eq!(err.status.value, 409u16);
        assert_eq!(err.message.value, "Email already registered");
    }

    #[test]
    fn endpoint_with_all_fields_parses() {
        let src = r#"
endpoint CreateUser {
    method   POST
    path     /users
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken, WeakPassword]
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.endpoints.len(), 1);
        let ep = &schema.endpoints[0];
        assert_eq!(ep.name.value, "CreateUser");
        assert_eq!(ep.method.value, HttpMethod::Post);
        assert_eq!(ep.path.value, "/users");
        assert_eq!(
            ep.body.as_ref().map(|b| b.value.as_str()),
            Some("CreateUserRequest")
        );
        assert_eq!(
            ep.response.as_ref().map(|r| r.value.as_str()),
            Some("UserResponse")
        );
        assert_eq!(ep.errors.len(), 2);
    }

    #[test]
    fn endpoint_get_without_body_parses() {
        let src = r#"
endpoint GetUser {
    method   GET
    path     /users/{id}
    response UserResponse
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let ep = &ast.unwrap().endpoints[0];
        assert_eq!(ep.method.value, HttpMethod::Get);
        assert_eq!(ep.path.value, "/users/{id}");
        assert!(ep.body.is_none());
    }

    #[test]
    fn full_v02_schema_parses() {
        let src = r#"
config {
    project_name "users-api"
    database "postgres"
}

model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}

request CreateUserRequest {
    email String
    name  String
}

response UserResponse {
    id    UUID
    email String
    name  String
}

error EmailTaken { status 409 message "Email already taken" }

endpoint CreateUser {
    method   POST
    path     /users
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken]
}

endpoint GetUser {
    method   GET
    path     /users/{id}
    response UserResponse
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.models.len(), 1);
        assert_eq!(schema.requests.len(), 1);
        assert_eq!(schema.responses.len(), 1);
        assert_eq!(schema.errors.len(), 1);
        assert_eq!(schema.endpoints.len(), 2);
    }

    // ---- Tests DSL v0.3 ----

    #[test]
    fn v03_min_max_annotations() {
        let src = r#"
model User {
    id    UUID @primary @auto
    email String @max(255) @min(1)
    age   Int    @min(0) @max(150)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().models[0].fields[1]; // email
        let has_max = field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Max(255));
        let has_min = field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Min(1));
        assert!(has_max, "should have @max(255)");
        assert!(has_min, "should have @min(1)");
    }

    #[test]
    fn v03_email_annotation() {
        let src = r#"
model User {
    id    UUID @primary @auto
    email String @email @max(255)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let email_field = &ast.unwrap().models[0].fields[1];
        assert!(email_field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Email));
    }

    #[test]
    fn v03_regex_annotation() {
        let src = r#"
request Req {
    code String @regex("^[A-Z]{3}$")
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().requests[0].fields[0];
        assert!(field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Regex("^[A-Z]{3}$".to_owned())));
    }

    #[test]
    fn v04_parses_references_annotation() {
        let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().models[0].fields[1];
        let has_ref = field.annotations.iter().any(|a| {
            matches!(&a.value,
                Annotation::References { model, field }
                if model == "User" && field == "id")
        });
        assert!(has_ref, "should have @references(User.id)");
        assert!(!field.virtual_field, "FK field should not be virtual");
    }

    #[test]
    fn v04_parses_relation_single() {
        let src = r#"
model Post {
    id        UUID @primary @auto
    author_id UUID @references(User.id)
    author    User @relation(author_id)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().models[0].fields[2];
        assert!(
            matches!(&field.ty.value, FieldType::ModelRef(m) if m == "User"),
            "type should be ModelRef(User)"
        );
        assert!(field.virtual_field, "relation field should be virtual");
        let has_rel = field
            .annotations
            .iter()
            .any(|a| matches!(&a.value, Annotation::Relation { path } if path == "author_id"));
        assert!(has_rel, "should have @relation(author_id)");
    }

    #[test]
    fn v04_parses_relation_list() {
        let src = r#"
model User {
    id    UUID   @primary @auto
    posts Post[] @relation(post.author_id)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().models[0].fields[1];
        assert!(
            matches!(&field.ty.value, FieldType::ModelRefList(m) if m == "Post"),
            "type should be ModelRefList(Post)"
        );
        assert!(field.virtual_field, "list relation should be virtual");
        let has_rel = field
            .annotations
            .iter()
            .any(|a| matches!(&a.value, Annotation::Relation { path } if path == "post.author_id"));
        assert!(has_rel, "should have @relation(post.author_id)");
    }

    #[test]
    fn v03_length_annotation() {
        let src = r#"
model Country {
    id   UUID @primary @auto
    code String @length(3)
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let field = &ast.unwrap().models[0].fields[1];
        assert!(field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Length(3)));
    }

    // ---- Tests DSL v0.5+v0.6 ----

    #[test]
    fn parse_endpoint_with_auth_required() {
        let src = r#"
    endpoint GetProfile {
        method GET
        path /profile
        auth required
        policy "user.id == params.id"
        response UserResponse
    }
    response UserResponse { id UUID }
    "#;
        let (tokens, _) = crate::lexer::tokenize(src);
        let (ast, _diags) = parse_tokens(tokens, src.len());
        let schema = ast.expect("debe parsear");
        assert_eq!(schema.endpoints.len(), 1);
        let ep = &schema.endpoints[0];
        assert_eq!(ep.auth, crate::ast::AuthMode::Required);
        assert!(ep.policy.is_some());
    }

    #[test]
    fn parse_event_declaration() {
        let src = r#"
    event user.created {
        payload UserResponse
        retain 30
    }
    "#;
        let (tokens, _) = crate::lexer::tokenize(src);
        let (ast, _diags) = parse_tokens(tokens, src.len());
        let schema = ast.expect("debe parsear");
        assert_eq!(schema.events.len(), 1);
        assert_eq!(schema.events[0].name.value, "user.created");
        assert_eq!(schema.events[0].retain_days, Some(30));
    }

    #[test]
    fn parse_endpoint_with_events() {
        let src = r#"
    event user.created { payload UserResponse }
    endpoint CreateUser {
        method POST
        path /users
        auth required
        events [user.created]
    }
    response UserResponse { id UUID }
    "#;
        let (tokens, _) = crate::lexer::tokenize(src);
        let (ast, _) = parse_tokens(tokens, src.len());
        let schema = ast.expect("debe parsear");
        let ep = &schema.endpoints[0];
        assert_eq!(ep.emits.len(), 1);
        assert_eq!(ep.emits[0].value, "user.created");
    }

    // ---- Tests DSL v0.7 ----

    #[test]
    fn parse_mail_block_minimal() {
        let src = r#"
mail transaccional {
    provider smtp
    from "noreply@ejemplo.com"
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.mails.len(), 1);
        let mail = &schema.mails[0];
        assert_eq!(mail.name.value, "transaccional");
        assert_eq!(mail.provider.as_deref(), Some("smtp"));
        assert_eq!(mail.from.as_deref(), Some("noreply@ejemplo.com"));
        assert!(mail.templates.is_empty());
    }

    #[test]
    fn parse_mail_block_with_templates() {
        let src = r#"
mail transaccional {
    provider mta
    from "noreply@ejemplo.com"

    template bienvenida {
        subject "Bienvenido {{nombre}}"
        vars [nombre, token]
    }

    template recuperacion {
        subject "Recupera tu acceso, {{nombre}}"
        vars [nombre, link]
    }
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        let mail = &schema.mails[0];
        assert_eq!(mail.templates.len(), 2);
        assert_eq!(mail.templates[0].name.value, "bienvenida");
        assert_eq!(
            mail.templates[0].subject.as_deref(),
            Some("Bienvenido {{nombre}}")
        );
        assert_eq!(mail.templates[0].vars, vec!["nombre", "token"]);
        assert_eq!(mail.templates[1].name.value, "recuperacion");
        assert_eq!(mail.templates[1].vars, vec!["nombre", "link"]);
    }

    #[test]
    fn parse_domain_block() {
        let src = r#"
domain mi_dominio {
    name "ejemplo.com"
    provider cloudflare
    dkim_selector "s1"
    dmarc_policy quarantine
    dmarc_rua "admin@ejemplo.com"
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.domains.len(), 1);
        let domain = &schema.domains[0];
        assert_eq!(domain.name.value, "mi_dominio");
        assert_eq!(domain.domain_name.as_deref(), Some("ejemplo.com"));
        assert_eq!(domain.provider.as_deref(), Some("cloudflare"));
        assert_eq!(domain.dkim_selectors, vec!["s1"]);
        assert_eq!(domain.dmarc_policy.as_deref(), Some("quarantine"));
        assert_eq!(domain.dmarc_rua.as_deref(), Some("admin@ejemplo.com"));
    }

    #[test]
    fn parse_mail_and_domain_coexist() {
        let src = r#"
mail notif {
    provider smtp
    from "ops@ejemplo.com"
}
domain prod {
    name "ejemplo.com"
    provider cloudflare
    dkim_selector "s1"
}
"#;
        let (ast, diags) = parse(src);
        assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
        let schema = ast.unwrap();
        assert_eq!(schema.mails.len(), 1);
        assert_eq!(schema.domains.len(), 1);
    }
}
