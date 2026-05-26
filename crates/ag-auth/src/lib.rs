//! Authentication and authorization for the Anti-Gravital ecosystem.
//!
//! Supports JWT Ed25519, Passkeys/WebAuthn, OAuth2 (Google, GitHub),
//! API keys with BLAKE3 hashing and refresh tokens with rotation.
//!
//! # Minimal usage (JWT + API keys)
//!
//! ```no_run
//! use ag_auth::{AgAuth, AuthConfig};
//!
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AuthConfig::from_env()?;
//! let auth = AgAuth::new(config, reqwest::Client::new())?;
//!
//! // API keys
//! let (raw_key, key_hash) = auth.create_api_key("sk");
//! let valid = auth.verify_api_key(&raw_key, &key_hash);
//! assert!(valid);
//! # Ok(())
//! # }
//! ```

pub mod api_keys;
pub mod config;
pub mod jwt;
#[cfg(feature = "mail")]
pub mod mailer;
pub mod oauth;
pub mod refresh;
pub mod webauthn;

pub use api_keys::{generate as generate_api_key, verify as verify_api_key};
pub use config::{AuthConfig, AuthConfigError};
pub use jwt::{Claims, JwtError, JwtSigner};
#[cfg(feature = "mail")]
pub use mailer::{AuthMailer, AuthMailerError};
pub use oauth::{OAuthClient, OAuthError, OAuthProvider, OAuthUser};
pub use refresh::RefreshBlacklist;
pub use webauthn::{
    AuthenticationChallenge, AuthenticationResponse, RegistrationChallenge, RegistrationResponse,
    StoredCredential, WebAuthnError, WebAuthnRp,
};

/// Main facade of the authentication module.
pub struct AgAuth {
    /// JWT signer/verifier.
    pub jwt: JwtSigner,
    /// WebAuthn Relying Party. None if `webauthn_rp_id` is empty.
    pub webauthn: Option<webauthn::WebAuthnRp>,
    /// OAuth2 client. None if no provider is configured.
    pub oauth: Option<oauth::OAuthClient>,
    /// Refresh token blacklist.
    pub refresh_blacklist: std::sync::Arc<refresh::RefreshBlacklist>,
    /// Mailer for verification, recovery and magic links. Requires feature `mail`.
    #[cfg(feature = "mail")]
    pub mailer: Option<std::sync::Arc<mailer::AuthMailer>>,
}

impl AgAuth {
    /// Creates a new `AgAuth` instance.
    ///
    /// - `webauthn` is initialized if `config.webauthn_rp_id` is not empty.
    /// - `oauth` is initialized if at least one provider has a client_id configured.
    /// - `http_client` is used internally for OAuth2 — the caller provides it
    ///   to allow configuration of timeouts, proxies and custom TLS.
    pub fn new(config: AuthConfig, http_client: reqwest::Client) -> Result<Self, AuthConfigError> {
        let jwt = JwtSigner::new(
            config.jwt_private_key_pem.clone(),
            config.jwt_public_key_pem.clone(),
        );

        let webauthn_rp = if !config.webauthn_rp_id.is_empty() {
            Some(webauthn::WebAuthnRp::new(
                config.webauthn_rp_id.clone(),
                config.webauthn_origin.clone(),
            ))
        } else {
            None
        };

        let has_google = config.oauth_google_client_id.is_some();
        let has_github = config.oauth_github_client_id.is_some();
        let oauth_client = if has_google || has_github {
            Some(oauth::OAuthClient::from_config(&config, http_client))
        } else {
            None
        };

        Ok(Self {
            jwt,
            webauthn: webauthn_rp,
            oauth: oauth_client,
            refresh_blacklist: std::sync::Arc::new(refresh::RefreshBlacklist::new()),
            #[cfg(feature = "mail")]
            mailer: None,
        })
    }

    /// Injects an `AuthMailer` into the instance.
    ///
    /// Available only with the `mail` feature. Returns `self` to
    /// allow chaining: `AgAuth::new(...)?.with_mail(mailer)`.
    #[cfg(feature = "mail")]
    pub fn with_mail(mut self, mailer: std::sync::Arc<mailer::AuthMailer>) -> Self {
        self.mailer = Some(mailer);
        self
    }

    /// Generates a new API key and its BLAKE3 hash.
    ///
    /// Only the hash should be stored. The raw key is given to the user only once.
    pub fn create_api_key(&self, prefix: &str) -> (String, String) {
        api_keys::generate(prefix)
    }

    /// Verifies an API key against its stored hash.
    pub fn verify_api_key(&self, raw_key: &str, stored_hash: &str) -> bool {
        api_keys::verify(raw_key, stored_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    fn fake_config() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "fake-private".to_string(),
            jwt_public_key_pem: "fake-public".to_string(),
            webauthn_rp_id: String::new(),
            webauthn_origin: String::new(),
            oauth_google_client_id: None,
            oauth_google_client_secret: None,
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    fn http() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[test]
    fn new_succeeds_with_valid_config() {
        let auth = AgAuth::new(fake_config(), http()).expect("debe construirse con config valida");
        let _ = &auth.jwt;
        assert!(auth.webauthn.is_none(), "sin rp_id, webauthn debe ser None");
        assert!(auth.oauth.is_none(), "sin providers, oauth debe ser None");
    }

    #[test]
    fn new_enables_webauthn_when_rp_id_set() {
        let mut cfg = fake_config();
        cfg.webauthn_rp_id = "example.com".into();
        cfg.webauthn_origin = "https://example.com".into();
        let auth = AgAuth::new(cfg, http()).unwrap();
        assert!(auth.webauthn.is_some());
    }

    #[test]
    fn new_enables_oauth_when_google_configured() {
        let mut cfg = fake_config();
        cfg.oauth_google_client_id = Some("gid".into());
        cfg.oauth_google_client_secret = Some("gsecret".into());
        let auth = AgAuth::new(cfg, http()).unwrap();
        assert!(auth.oauth.is_some());
    }

    #[test]
    fn create_api_key_uses_prefix() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (raw, _hash) = auth.create_api_key("sk");
        assert!(
            raw.starts_with("sk_"),
            "raw key debe iniciar con el prefijo"
        );
    }

    #[test]
    fn verify_api_key_roundtrip() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (raw, hash) = auth.create_api_key("test");
        assert!(
            auth.verify_api_key(&raw, &hash),
            "la key generada debe verificar"
        );
    }

    #[test]
    fn verify_api_key_rejects_wrong_key() {
        let auth = AgAuth::new(fake_config(), http()).unwrap();
        let (_raw, hash) = auth.create_api_key("test");
        assert!(!auth.verify_api_key("test_wrongkey", &hash));
    }
}
