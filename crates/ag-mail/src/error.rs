//! Error type for the `ag-mail` crate.
//!
//! Conversions from provider errors (SMTP, adapter HTTP) are added when
//! the corresponding modules are implemented. For now the enum covers the
//! most common variants so the rest of the skeleton can reference them.

use thiserror::Error;

/// Errors produced by `ag-mail` operations.
#[derive(Debug, Error)]
pub enum AgMailError {
    /// The email could not be sent after exhausting the retries.
    #[error("mail send exhausted after retries: {0}")]
    SendExhausted(String),

    /// The source or destination address is not valid.
    #[error("invalid mail address: {0}")]
    InvalidAddress(String),

    /// The template does not exist or could not be rendered.
    #[error("template error: {0}")]
    Template(String),

    /// The variables declared in the DSL `mail` block do not match the ones
    /// in the template HTML.
    #[error("mismatch between declared variables and template: {0}")]
    VarMismatch(String),

    /// Generic configuration error (missing environment variable, invalid
    /// credential, etc).
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Generic provider error (native SMTP or adapter).
    #[error("mail provider error ({provider}): {message}")]
    Provider {
        /// Name of the provider reporting the failure.
        provider: &'static str,
        /// Message propagated by the provider.
        message: String,
    },

    /// Queue storage error (persistent backend).
    #[error("queue error: {0}")]
    Queue(String),

    /// MX resolution failure for a destination domain (native MTA).
    #[error("DNS/MX resolution error: {0}")]
    Dns(String),

    /// The destination domain exposes no usable mail host (native MTA).
    #[error("destination domain exposes no mail host: {0}")]
    NoMailHost(String),

    /// DKIM signing key or signer construction failure (native MTA).
    #[error("DKIM signing error: {0}")]
    Dkim(String),
}

#[cfg(test)]
mod tests {
    use super::AgMailError;

    #[test]
    fn every_variant_displays_its_payload() {
        let cases = [
            AgMailError::SendExhausted("x".into()),
            AgMailError::InvalidAddress("x".into()),
            AgMailError::Template("x".into()),
            AgMailError::VarMismatch("x".into()),
            AgMailError::Config("x".into()),
            AgMailError::Provider {
                provider: "smtp",
                message: "x".into(),
            },
            AgMailError::Queue("x".into()),
            AgMailError::Dns("x".into()),
            AgMailError::NoMailHost("x".into()),
            AgMailError::Dkim("x".into()),
        ];
        for err in &cases {
            let rendered = err.to_string();
            assert!(!rendered.is_empty(), "{err:?} renders empty");
        }
    }

    #[test]
    fn native_mta_variants_render_expected_text() {
        assert!(AgMailError::Dns("mx".into()).to_string().contains("DNS/MX"));
        assert!(AgMailError::NoMailHost("d".into())
            .to_string()
            .contains("mail host"));
        assert!(AgMailError::Dkim("k".into())
            .to_string()
            .contains("DKIM signing"));
        assert!(AgMailError::Provider {
            provider: "mta",
            message: "boom".into(),
        }
        .to_string()
        .contains("mta"));
    }
}
