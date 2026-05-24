//! Sistema de templates para correo electronico.
//!
//! `MailTemplate` es el trait que cualquier template debe implementar.
//! `StringTemplate` es la implementacion de uso general: sustituye
//! `{{var}}` por valores del mapa de variables.
//!
//! La validacion build-time de variables (para el compilador `ag-dsl`)
//! esta en el submodulo `validate`.

pub mod validate;

use std::collections::HashMap;

use crate::error::AgMailError;

/// Abstraccion sobre un template de correo.
///
/// Los proyectos pueden implementar `MailTemplate` con cualquier motor de
/// plantillas (askama, minijinja, Handlebars, etc.) o usar `StringTemplate`
/// para templates simples.
pub trait MailTemplate: Send + Sync {
    /// Renderiza el asunto del correo con las variables dadas.
    fn render_subject(&self, vars: &HashMap<String, String>) -> Result<String, AgMailError>;

    /// Renderiza el cuerpo HTML con las variables dadas.
    fn render_html(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError>;

    /// Renderiza el cuerpo en texto plano con las variables dadas.
    fn render_text(&self, vars: &HashMap<String, String>) -> Result<Option<String>, AgMailError>;
}

/// Template basado en sustitucion de `{{var}}` en strings.
///
/// Adecuado para templates simples. Para templates complejos con loops
/// o condicionales, usar un motor externo e implementar `MailTemplate`.
#[derive(Debug, Clone)]
pub struct StringTemplate {
    /// Template del asunto (puede contener `{{var}}`).
    pub subject_tpl: String,
    /// Template HTML (puede contener `{{var}}`).
    pub html_tpl: Option<String>,
    /// Template de texto plano (puede contener `{{var}}`).
    pub text_tpl: Option<String>,
}

impl StringTemplate {
    /// Crea un template con solo cuerpo HTML.
    pub fn html(subject: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            subject_tpl: subject.into(),
            html_tpl: Some(html.into()),
            text_tpl: None,
        }
    }

    /// Crea un template con cuerpo HTML y texto plano.
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

    /// Crea un template con solo texto plano.
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

/// Sustituye `{{var}}` por el valor del mapa.
///
/// Si una variable del template no esta en el mapa, retorna
/// `AgMailError::Template` con el nombre de la variable faltante.
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
