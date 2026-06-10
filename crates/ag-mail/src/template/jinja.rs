//! External template engine adapter using `minijinja` (DEBT-003).
//!
//! `StringTemplate` only does `{{var}}` substitution. For templates with
//! loops, conditionals and filters, this adapter renders Jinja2-style sources
//! through `minijinja` while implementing the same [`MailTemplate`] trait, so
//! it is a drop-in alternative. It is behind the `minijinja` Cargo feature; the
//! built-in `StringTemplate` remains the default and needs no extra dependency.

use std::collections::HashMap;

use crate::{error::AgMailError, template::MailTemplate};

/// A `MailTemplate` backed by the `minijinja` engine.
#[derive(Debug, Clone)]
pub struct MinijinjaTemplate {
    /// Subject template source.
    pub subject_tpl: String,
    /// Optional HTML body template source.
    pub html_tpl: Option<String>,
    /// Optional plain-text body template source.
    pub text_tpl: Option<String>,
}

impl MinijinjaTemplate {
    /// Creates a template with an HTML body.
    pub fn html(subject: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: Some(html.into()),
            text_tpl: None,
        }
    }

    /// Creates a template with an HTML body and a plain-text alternative.
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

    /// Creates a plain-text-only template.
    pub fn text(subject: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: None,
            text_tpl: Some(text.into()),
        }
    }
}

/// Renders one source with the variable map as the rendering context.
fn render(source: &str, vars: &HashMap<String, String>) -> Result<String, AgMailError> {
    let env = minijinja::Environment::new();
    env.render_str(source, vars)
        .map_err(|e| AgMailError::Template(e.to_string()))
}

impl MailTemplate for MinijinjaTemplate {
    fn render_subject(&self, vars: &HashMap<String, String>) -> Result<String, AgMailError> {
        render(&self.subject_tpl, vars)
    }

    fn render_html(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError> {
        self.html_tpl
            .as_deref()
            .map(|tpl| render(tpl, vars))
            .transpose()
    }

    fn render_text(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError> {
        self.text_tpl
            .as_deref()
            .map(|tpl| render(tpl, vars))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn renders_variable_substitution() {
        let tpl = MinijinjaTemplate::html("Hi {{ name }}", "<p>Hello {{ name }}</p>");
        let v = vars(&[("name", "Angel")]);
        assert_eq!(tpl.render_subject(&v).unwrap(), "Hi Angel");
        assert_eq!(
            tpl.render_html(&v).unwrap().as_deref(),
            Some("<p>Hello Angel</p>")
        );
        assert_eq!(tpl.render_text(&v).unwrap(), None);
    }

    #[test]
    fn renders_conditionals_and_filters() {
        // Loops/conditionals/filters are exactly what StringTemplate cannot do.
        let tpl = MinijinjaTemplate::text(
            "{{ name | upper }}",
            "{% if vip %}Welcome back{% else %}Welcome{% endif %}, {{ name }}",
        );
        let v = vars(&[("name", "ana"), ("vip", "true")]);
        assert_eq!(tpl.render_subject(&v).unwrap(), "ANA");
        assert_eq!(
            tpl.render_text(&v).unwrap().as_deref(),
            Some("Welcome back, ana")
        );
    }

    #[test]
    fn missing_variable_renders_empty_not_error() {
        // minijinja treats undefined as empty by default (no hard error).
        let tpl = MinijinjaTemplate::text("s", "Hello {{ name }}");
        let out = tpl.render_text(&vars(&[])).unwrap();
        assert_eq!(out.as_deref(), Some("Hello "));
    }

    #[test]
    fn syntax_error_is_template_error() {
        let tpl = MinijinjaTemplate::text("s", "{% if %}broken");
        assert!(matches!(
            tpl.render_text(&vars(&[])),
            Err(AgMailError::Template(_))
        ));
    }

    #[test]
    fn usable_as_trait_object() {
        let tpl: Box<dyn MailTemplate> =
            Box::new(MinijinjaTemplate::html("s {{ x }}", "<b>{{ x }}</b>"));
        let v = vars(&[("x", "1")]);
        assert_eq!(tpl.render_subject(&v).unwrap(), "s 1");
    }
}
