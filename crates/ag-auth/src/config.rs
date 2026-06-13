//! Authentication configuration read from environment variables.

/// Configuration of the authentication module.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Ed25519 private key in PEM format for signing JWTs.
    /// Variable: `JWT_PRIVATE_KEY`
    pub jwt_private_key_pem: String,
    /// Ed25519 public key in PEM format for verifying JWTs.
    /// Variable: `JWT_PUBLIC_KEY`
    pub jwt_public_key_pem: String,
    /// Relying party identifier for WebAuthn (e.g.: `"example.com"`).
    /// Variable: `WEBAUTHN_RP_ID`
    pub webauthn_rp_id: String,
    /// Origin URL for WebAuthn (e.g.: `"https://example.com"`).
    /// Variable: `WEBAUTHN_ORIGIN`
    pub webauthn_origin: String,
    /// Google OAuth2 client ID. `None` disables the provider.
    pub oauth_google_client_id: Option<String>,
    /// Google OAuth2 client secret.
    pub oauth_google_client_secret: Option<String>,
    /// GitHub OAuth2 client ID. `None` disables the provider.
    pub oauth_github_client_id: Option<String>,
    /// GitHub OAuth2 client secret.
    pub oauth_github_client_secret: Option<String>,
}

impl AuthConfig {
    /// Reads the configuration from environment variables.
    ///
    /// Returns an error if the required JWT keys are not defined.
    pub fn from_env() -> Result<Self, AuthConfigError> {
        let jwt_private_key_pem = std::env::var("JWT_PRIVATE_KEY")
            .map_err(|_| AuthConfigError::MissingVar("JWT_PRIVATE_KEY"))?;
        let jwt_public_key_pem = std::env::var("JWT_PUBLIC_KEY")
            .map_err(|_| AuthConfigError::MissingVar("JWT_PUBLIC_KEY"))?;
        let webauthn_rp_id =
            std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Ok(Self {
            jwt_private_key_pem,
            jwt_public_key_pem,
            webauthn_rp_id,
            webauthn_origin,
            oauth_google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            oauth_google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            oauth_github_client_id: std::env::var("GITHUB_CLIENT_ID").ok(),
            oauth_github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").ok(),
        })
    }
}

/// Authentication configuration error.
#[derive(Debug)]
#[non_exhaustive]
pub enum AuthConfigError {
    /// Required environment variable not defined.
    MissingVar(&'static str),
}

impl std::fmt::Display for AuthConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthConfigError::MissingVar(v) => {
                write!(f, "required environment variable not set: {v}")
            }
        }
    }
}

impl std::error::Error for AuthConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serializes access to JWT_PRIVATE_KEY / JWT_PUBLIC_KEY across parallel tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn from_env_fails_without_jwt_keys() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_PUBLIC_KEY");
        assert!(AuthConfig::from_env().is_err());
    }

    #[test]
    fn from_env_reads_jwt_keys() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("JWT_PRIVATE_KEY", "fake-private");
        std::env::set_var("JWT_PUBLIC_KEY", "fake-public");
        let config =
            AuthConfig::from_env().expect("must build with JWT_PRIVATE_KEY and JWT_PUBLIC_KEY set");
        assert_eq!(config.jwt_private_key_pem, "fake-private");
        assert_eq!(config.jwt_public_key_pem, "fake-public");
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_PUBLIC_KEY");
    }
}
