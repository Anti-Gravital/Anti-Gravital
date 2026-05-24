//! Tipos de mensaje de correo electronico.
//!
//! `Email` es el tipo central que todos los adapters consumen. Se construye
//! con `EmailBuilder` siguiendo el patron builder con validacion diferida
//! (la validacion ocurre en `build()`, no en cada setter).

use serde::{Deserialize, Serialize};

use crate::error::AgMailError;

/// Direccion de correo con nombre de visualizacion opcional.
///
/// # Formato aceptado
///
/// - Solo email: `"usuario@ejemplo.com"`
/// - Con nombre: `"Nombre Apellido <usuario@ejemplo.com>"`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Nombre de visualizacion (opcional).
    pub name: Option<String>,
    /// Direccion de correo electronico.
    pub email: String,
}

impl Address {
    /// Crea una direccion sin nombre de visualizacion.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            name: None,
            email: email.into(),
        }
    }

    /// Crea una direccion con nombre de visualizacion.
    pub fn with_name(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: email.into(),
        }
    }

    /// Valida que la direccion tenga un formato basico correcto.
    pub fn validate(&self) -> Result<(), AgMailError> {
        let email = self.email.trim();
        if email.is_empty()
            || !email.contains('@')
            || email.starts_with('@')
            || email.ends_with('@')
        {
            return Err(AgMailError::InvalidAddress(self.email.clone()));
        }
        Ok(())
    }

    /// Retorna la representacion en formato `"Nombre <email>"` o solo `"email"`.
    pub fn display(&self) -> String {
        match &self.name {
            Some(name) => format!("{name} <{}>", self.email),
            None => self.email.clone(),
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Adjunto de correo.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Nombre del archivo (e.g., `"factura.pdf"`).
    pub filename: String,
    /// Content-Type MIME (e.g., `"application/pdf"`, `"image/png"`).
    pub content_type: String,
    /// Contenido binario del adjunto.
    pub data: Vec<u8>,
}

/// Mensaje de correo listo para enviar.
///
/// Construido con `EmailBuilder`. Todos los campos publicos son inmutables
/// una vez construido.
#[derive(Debug, Clone)]
pub struct Email {
    /// Remitente.
    pub from: Address,
    /// Destinatarios principales.
    pub to: Vec<Address>,
    /// Copia de carbon (visible).
    pub cc: Vec<Address>,
    /// Copia de carbon oculta.
    pub bcc: Vec<Address>,
    /// Direccion de respuesta (por defecto igual que `from`).
    pub reply_to: Option<Address>,
    /// Asunto del correo.
    pub subject: String,
    /// Cuerpo HTML (opcional si hay `text_body`).
    pub html_body: Option<String>,
    /// Cuerpo en texto plano (opcional si hay `html_body`).
    pub text_body: Option<String>,
    /// Adjuntos.
    pub attachments: Vec<Attachment>,
    /// Cabeceras adicionales (e.g., `X-Custom-Header`).
    pub headers: Vec<(String, String)>,
}

/// Constructor de `Email` con validacion en `build()`.
#[derive(Default)]
pub struct EmailBuilder {
    from: Option<Address>,
    to: Vec<Address>,
    cc: Vec<Address>,
    bcc: Vec<Address>,
    reply_to: Option<Address>,
    subject: Option<String>,
    html_body: Option<String>,
    text_body: Option<String>,
    attachments: Vec<Attachment>,
    headers: Vec<(String, String)>,
}

impl EmailBuilder {
    /// Crea un builder vacio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el remitente.
    pub fn from(mut self, addr: Address) -> Self {
        self.from = Some(addr);
        self
    }

    /// Agrega un destinatario principal.
    pub fn to(mut self, addr: Address) -> Self {
        self.to.push(addr);
        self
    }

    /// Agrega multiples destinatarios principales.
    pub fn to_many(mut self, addrs: impl IntoIterator<Item = Address>) -> Self {
        self.to.extend(addrs);
        self
    }

    /// Agrega un destinatario en CC.
    pub fn cc(mut self, addr: Address) -> Self {
        self.cc.push(addr);
        self
    }

    /// Agrega un destinatario en BCC.
    pub fn bcc(mut self, addr: Address) -> Self {
        self.bcc.push(addr);
        self
    }

    /// Establece el `Reply-To`.
    pub fn reply_to(mut self, addr: Address) -> Self {
        self.reply_to = Some(addr);
        self
    }

    /// Establece el asunto.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Establece el cuerpo HTML.
    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    /// Establece el cuerpo en texto plano.
    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }

    /// Agrega un adjunto.
    pub fn attachment(mut self, att: Attachment) -> Self {
        self.attachments.push(att);
        self
    }

    /// Agrega una cabecera personalizada.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Construye el `Email` validando los campos obligatorios.
    pub fn build(self) -> Result<Email, AgMailError> {
        let from = self
            .from
            .ok_or_else(|| AgMailError::Config("falta campo `from`".to_owned()))?;
        from.validate()?;

        if self.to.is_empty() {
            return Err(AgMailError::Config(
                "el correo debe tener al menos un destinatario".to_owned(),
            ));
        }
        for addr in &self.to {
            addr.validate()?;
        }
        for addr in &self.cc {
            addr.validate()?;
        }
        for addr in &self.bcc {
            addr.validate()?;
        }

        let subject = self
            .subject
            .ok_or_else(|| AgMailError::Config("falta campo `subject`".to_owned()))?;

        if self.html_body.is_none() && self.text_body.is_none() {
            return Err(AgMailError::Config(
                "el correo debe tener al menos `html_body` o `text_body`".to_owned(),
            ));
        }

        Ok(Email {
            from,
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            reply_to: self.reply_to,
            subject,
            html_body: self.html_body,
            text_body: self.text_body,
            attachments: self.attachments,
            headers: self.headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_email() -> Email {
        EmailBuilder::new()
            .from(Address::new("remitente@ejemplo.com"))
            .to(Address::with_name("Receptor", "receptor@ejemplo.com"))
            .subject("Hola")
            .html_body("<p>Hola</p>")
            .build()
            .unwrap()
    }

    #[test]
    fn builder_minimal_ok() {
        let email = valid_email();
        assert_eq!(email.from.email, "remitente@ejemplo.com");
        assert_eq!(email.to[0].name.as_deref(), Some("Receptor"));
        assert_eq!(email.subject, "Hola");
    }

    #[test]
    fn builder_missing_from_err() {
        let err = EmailBuilder::new()
            .to(Address::new("x@x.com"))
            .subject("s")
            .text_body("t")
            .build()
            .unwrap_err();
        assert!(matches!(err, AgMailError::Config(_)));
    }

    #[test]
    fn builder_missing_recipients_err() {
        let err = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .subject("s")
            .html_body("<p>p</p>")
            .build()
            .unwrap_err();
        assert!(matches!(err, AgMailError::Config(_)));
    }

    #[test]
    fn builder_missing_body_err() {
        let err = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("s")
            .build()
            .unwrap_err();
        assert!(matches!(err, AgMailError::Config(_)));
    }

    #[test]
    fn address_validate_rejects_no_at() {
        let err = Address::new("invalido").validate().unwrap_err();
        assert!(matches!(err, AgMailError::InvalidAddress(_)));
    }

    #[test]
    fn address_validate_rejects_leading_at() {
        let err = Address::new("@dominio.com").validate().unwrap_err();
        assert!(matches!(err, AgMailError::InvalidAddress(_)));
    }

    #[test]
    fn address_display_with_name() {
        let addr = Address::with_name("Angel Nereira", "angel@ejemplo.com");
        assert_eq!(addr.display(), "Angel Nereira <angel@ejemplo.com>");
    }

    #[test]
    fn address_display_without_name() {
        let addr = Address::new("angel@ejemplo.com");
        assert_eq!(addr.display(), "angel@ejemplo.com");
    }

    #[test]
    fn builder_cc_bcc_headers() {
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .cc(Address::new("cc@cc.com"))
            .bcc(Address::new("bcc@bcc.com"))
            .reply_to(Address::new("reply@reply.com"))
            .subject("s")
            .text_body("t")
            .header("X-Custom", "value")
            .build()
            .unwrap();

        assert_eq!(email.cc.len(), 1);
        assert_eq!(email.bcc.len(), 1);
        assert!(email.reply_to.is_some());
        assert_eq!(email.headers[0].0, "X-Custom");
    }
}
