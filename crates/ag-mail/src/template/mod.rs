//! Template system for email.
//!
//! `MailTemplate` is the trait that any template must implement.
//! `StringTemplate` is the general-purpose implementation: it substitutes
//! `{{var}}` with values from the variable map.
//!
//! The build-time variable validation (for the `ag-dsl` compiler)
//! is in the `validate` submodule.

#[cfg(feature = "minijinja")]
pub mod jinja;
pub mod validate;

use std::collections::HashMap;

use crate::error::AgMailError;

/// Abstraction over an email template.
///
/// Projects can implement `MailTemplate` with any template engine
/// (askama, minijinja, Handlebars, etc.) or use `StringTemplate`
/// for simple templates.
pub trait MailTemplate: Send + Sync {
    /// Renders the email subject with the given variables.
    fn render_subject(&self, vars: &HashMap<String, String>) -> Result<String, AgMailError>;

    /// Renders the HTML body with the given variables.
    fn render_html(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError>;

    /// Renders the plain text body with the given variables.
    fn render_text(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError>;
}

/// Template based on `{{var}}` substitution in strings.
///
/// Suitable for simple templates. For complex templates with loops
/// or conditionals, use an external engine and implement `MailTemplate`.
#[derive(Debug, Clone)]
pub struct StringTemplate {
    /// Subject template (may contain `{{var}}`).
    pub subject_tpl: String,
    /// HTML template (may contain `{{var}}`).
    pub html_tpl: Option<String>,
    /// Plain text template (may contain `{{var}}`).
    pub text_tpl: Option<String>,
}

impl StringTemplate {
    /// Creates a template with only an HTML body.
    pub fn html(subject: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: Some(html.into()),
            text_tpl: None,
        }
    }

    /// Creates a template with HTML body and plain text.
    pub fn html_and_text(
        subject: impl Into<String>,
        html: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: Some(html.into()),
            text_tpl: Some(text.into()),
        }
    }

    /// Creates a template with only plain text.
    pub fn text(subject: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: None,
            text_tpl: Some(text.into()),
        }
    }
}

impl MailTemplate for StringTemplate {
    fn render_subject(&self, vars: &HashMap<String, String>) -> Result<String, AgMailError> {
        render_string(&self.subject_tpl, vars)
    }

    fn render_html(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError> {
        self.html_tpl
            .as_deref()
            .map(|tpl| render_string(tpl, vars))
            .transpose()
    }

    fn render_text(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError> {
        self.text_tpl
            .as_deref()
            .map(|tpl| render_string(tpl, vars))
            .transpose()
    }
}

/// Substitutes `{{var}}` with the value from the map.
///
/// If a template variable is not in the map, returns
/// `AgMailError::Template` with the name of the missing variable.
fn render_string(template: &str, vars: &HashMap<String, String>) -> Result<String, AgMailError> {
    let used = validate::extract_vars(template);
    let mut output = template.to_owned();

    for var in &used {
        let value = vars.get(var).ok_or_else(|| {
            AgMailError::Template(format!(
                "variable '{{{{{}}}}}' no encontrada en el mapa",
                var
            ))
        })?;
        output = output.replace(&format!("{{{{{var}}}}}"), value);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn render_string_basic() {
        let result = render_string("Hola {{nombre}}!", &vars(&[("nombre", "Angel")])).unwrap();
        assert_eq!(result, "Hola Angel!");
    }

    #[test]
    fn render_string_multiple_vars() {
        let result = render_string(
            "{{saludo}} {{nombre}}, tu token es {{token}}.",
            &vars(&[("saludo", "Hola"), ("nombre", "Angel"), ("token", "XYZ")]),
        )
        .unwrap();
        assert_eq!(result, "Hola Angel, tu token es XYZ.");
    }

    #[test]
    fn render_string_missing_var_err() {
        let err = render_string("Hola {{nombre}}", &HashMap::new()).unwrap_err();
        assert!(matches!(err, AgMailError::Template(_)));
    }

    #[test]
    fn string_template_html_renders() {
        let tpl = StringTemplate::html("Bienvenido {{nombre}}", "<h1>Hola {{nombre}}</h1>");
        let vs = vars(&[("nombre", "Angel")]);

        assert_eq!(tpl.render_subject(&vs).unwrap(), "Bienvenido Angel");
        assert_eq!(
            tpl.render_html(&vs).unwrap(),
            Some("<h1>Hola Angel</h1>".to_owned())
        );
        assert_eq!(tpl.render_text(&vs).unwrap(), None);
    }

    #[test]
    fn string_template_text_renders() {
        let tpl = StringTemplate::text("Asunto {{x}}", "Texto {{x}}");
        let vs = vars(&[("x", "42")]);
        assert_eq!(tpl.render_subject(&vs).unwrap(), "Asunto 42");
        assert_eq!(tpl.render_text(&vs).unwrap(), Some("Texto 42".to_owned()));
        assert_eq!(tpl.render_html(&vs).unwrap(), None);
    }

    #[test]
    fn string_template_html_and_text_renders() {
        let tpl = StringTemplate::html_and_text("S {{n}}", "<p>{{n}}</p>", "plain {{n}}");
        let vs = vars(&[("n", "ok")]);
        assert_eq!(tpl.render_subject(&vs).unwrap(), "S ok");
        assert_eq!(tpl.render_html(&vs).unwrap(), Some("<p>ok</p>".to_owned()));
        assert_eq!(tpl.render_text(&vs).unwrap(), Some("plain ok".to_owned()));
    }
}
