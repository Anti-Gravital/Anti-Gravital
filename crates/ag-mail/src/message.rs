//! Email message types.
//!
//! `Email` is the central type that all adapters consume. It is built
//! with `EmailBuilder` following the builder pattern with deferred
//! validation (validation occurs in `build()`, not in each setter).

use serde::{Deserialize, Serialize};

use crate::error::AgMailError;

/// Email address with an optional display name.
///
/// # Accepted format
///
/// - Email only: `"usuario@ejemplo.com"`
/// - With name: `"Nombre Apellido <usuario@ejemplo.com>"`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Display name (optional).
    pub name: Option<String>,
    /// Email address.
    pub email: String,
}

impl Address {
    /// Creates an address without a display name.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            name: None,
            email: email.into(),
        }
    }

    /// Creates an address with a display name.
    pub fn with_name(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: email.into(),
        }
    }

    /// Validates that the address has a basic correct format.
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

    /// Returns the representation in the format `"Name <email>"` or just `"email"`.
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

/// Email attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// File name (e.g., `"factura.pdf"`).
    pub filename: String,
    /// MIME Content-Type (e.g., `"application/pdf"`, `"image/png"`).
    pub content_type: String,
    /// Binary content of the attachment.
    pub data: Vec<u8>,
}

/// Email message ready to send.
///
/// Built with `EmailBuilder`. All public fields are immutable once built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email {
    /// Sender.
    pub from: Address,
    /// Primary recipients.
    pub to: Vec<Address>,
    /// Carbon copy (visible).
    pub cc: Vec<Address>,
    /// Blind carbon copy.
    pub bcc: Vec<Address>,
    /// Reply address (defaults to the same as `from`).
    pub reply_to: Option<Address>,
    /// Email subject.
    pub subject: String,
    /// HTML body (optional if `text_body` is present).
    pub html_body: Option<String>,
    /// Plain text body (optional if `html_body` is present).
    pub text_body: Option<String>,
    /// Attachments.
    pub attachments: Vec<Attachment>,
    /// Additional headers (e.g., `X-Custom-Header`).
    pub headers: Vec<(String, String)>,
}

/// `Email` constructor with validation in `build()`.
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
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sender.
    pub fn from(mut self, addr: Address) -> Self {
        self.from = Some(addr);
        self
    }

    /// Adds a primary recipient.
    pub fn to(mut self, addr: Address) -> Self {
        self.to.push(addr);
        self
    }

    /// Adds multiple primary recipients.
    pub fn to_many(mut self, addrs: impl IntoIterator<Item = Address>) -> Self {
        self.to.extend(addrs);
        self
    }

    /// Adds a CC recipient.
    pub fn cc(mut self, addr: Address) -> Self {
        self.cc.push(addr);
        self
    }

    /// Adds a BCC recipient.
    pub fn bcc(mut self, addr: Address) -> Self {
        self.bcc.push(addr);
        self
    }

    /// Sets the `Reply-To`.
    pub fn reply_to(mut self, addr: Address) -> Self {
        self.reply_to = Some(addr);
        self
    }

    /// Sets the subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Sets the HTML body.
    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    /// Sets the plain text body.
    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }

    /// Adds an attachment.
    pub fn attachment(mut self, att: Attachment) -> Self {
        self.attachments.push(att);
        self
    }

    /// Adds a custom header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Builds the `Email`, validating the required fields.
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

    #[test]
    fn address_validate_rejects_trailing_at_and_empty() {
        assert!(matches!(
            Address::new("user@").validate(),
            Err(AgMailError::InvalidAddress(_))
        ));
        assert!(matches!(
            Address::new("   ").validate(),
            Err(AgMailError::InvalidAddress(_))
        ));
    }

    #[test]
    fn builder_rejects_invalid_cc_and_bcc() {
        let cc_err = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .cc(Address::new("not-an-email"))
            .subject("s")
            .text_body("t")
            .build()
            .unwrap_err();
        assert!(matches!(cc_err, AgMailError::InvalidAddress(_)));

        let bcc_err = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .bcc(Address::new("@nope"))
            .subject("s")
            .text_body("t")
            .build()
            .unwrap_err();
        assert!(matches!(bcc_err, AgMailError::InvalidAddress(_)));
    }

    #[test]
    fn builder_with_attachment_and_many_recipients() {
        let attachment = Attachment {
            filename: "invoice.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            data: vec![1, 2, 3, 4],
        };
        let email = EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to_many([Address::new("a@x.com"), Address::new("b@x.com")])
            .subject("s")
            .html_body("<p>hi</p>")
            .attachment(attachment.clone())
            .build()
            .unwrap();
        assert_eq!(email.to.len(), 2);
        assert_eq!(email.attachments.len(), 1);
        assert_eq!(email.attachments[0], attachment);
        assert_eq!(email.attachments[0].filename, "invoice.pdf");
    }
}

#[cfg(test)]
mod address_prop_tests {
    //! Property-based tests for address validation.
    use super::Address;
    use proptest::prelude::*;

    proptest! {
        // Validation must never panic on arbitrary input.
        #[test]
        fn validate_never_panics(s in ".*") {
            let _ = Address::new(s).validate();
        }

        // A well-formed `local@domain.tld` always validates.
        #[test]
        fn well_formed_address_validates(
            local in "[a-z0-9]{1,20}", domain in "[a-z0-9]{1,20}", tld in "[a-z]{2,6}",
        ) {
            let email = format!("{local}@{domain}.{tld}");
            prop_assert!(Address::new(email).validate().is_ok());
        }

        // An address with no `@`, or one bounded by `@`, never validates.
        #[test]
        fn malformed_address_is_rejected(s in "[a-z0-9.]{1,30}") {
            let at_prefixed = format!("@{s}");
            let at_suffixed = format!("{s}@");
            prop_assert!(Address::new(s.clone()).validate().is_err());
            prop_assert!(Address::new(at_prefixed).validate().is_err());
            prop_assert!(Address::new(at_suffixed).validate().is_err());
        }
    }
}
