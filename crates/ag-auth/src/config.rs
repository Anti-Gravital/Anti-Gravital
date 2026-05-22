//! Configuracion de autenticacion leida desde variables de entorno.

/// Configuracion del modulo de autenticacion.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Clave privada Ed25519 en formato PEM para firmar JWTs.
    /// Variable: `JWT_PRIVATE_KEY`
    pub jwt_private_key_pem: String,
    /// Clave publica Ed25519 en formato PEM para verificar JWTs.
    /// Variable: `JWT_PUBLIC_KEY`
    pub jwt_public_key_pem: String,
    /// Identificador del relying party para WebAuthn (ej: `"example.com"`).
    /// Variable: `WEBAUTHN_RP_ID`
    pub webauthn_rp_id: String,
    /// URL de origen para WebAuthn (ej: `"https://example.com"`).
    /// Variable: `WEBAUTHN_ORIGIN`
    pub webauthn_origin: String,
    /// Client ID de OAuth2 Google. `None` deshabilita el provider.
    pub oauth_google_client_id: Option<String>,
    /// Client secret de OAuth2 Google.
    pub oauth_google_client_secret: Option<String>,
    /// Client ID de OAuth2 GitHub. `None` deshabilita el provider.
    pub oauth_github_client_id: Option<String>,
    /// Client secret de OAuth2 GitHub.
    pub oauth_github_client_secret: Option<String>,
}

impl AuthConfig {
    /// Lee la configuracion desde variables de entorno.
    ///
    /// Retorna un error si las claves JWT obligatorias no estan definidas.
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

/// Error de configuracion de autenticacion.
#[derive(Debug)]
pub enum AuthConfigError {
    /// Variable de entorno obligatoria no definida.
    MissingVar(&'static str),
}

impl std::fmt::Display for AuthConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthConfigError::MissingVar(v) => {
                write!(f, "variable de entorno requerida no definida: {v}")
            }
        }
    }
}

impl std::error::Error for AuthConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_fails_without_jwt_keys() {
        // Eliminar variables para asegurar que el test sea determinista.
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_PUBLIC_KEY");
        assert!(AuthConfig::from_env().is_err());
    }

    #[test]
    fn from_env_reads_jwt_keys() {
        std::env::set_var("JWT_PRIVATE_KEY", "fake-private");
        std::env::set_var("JWT_PUBLIC_KEY", "fake-public");
        let config = AuthConfig::from_env()
            .expect("debe construirse con JWT_PRIVATE_KEY y JWT_PUBLIC_KEY definidas");
        assert_eq!(config.jwt_private_key_pem, "fake-private");
        assert_eq!(config.jwt_public_key_pem, "fake-public");
        // Limpiar variables de entorno para no contaminar otros tests.
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_PUBLIC_KEY");
    }
}
