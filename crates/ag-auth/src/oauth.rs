//! Cliente OAuth2 Authorization Code + PKCE para Google y GitHub.
//!
//! No usa la feature reqwest de oauth2 para evitar conflicto de versiones.
//! El intercambio de tokens se implementa directamente con reqwest 0.12.

use crate::config::AuthConfig;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use std::borrow::Cow;

/// Cliente OAuth2 con auth URL y token URL configurados.
type ConfiguredClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Proveedor OAuth2 soportado.
#[derive(Debug, Clone, Copy)]
pub enum OAuthProvider {
    /// Google Identity Platform.
    Google,
    /// GitHub OAuth Apps.
    GitHub,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "Google"),
            OAuthProvider::GitHub => write!(f, "GitHub"),
        }
    }
}

/// Informacion del usuario obtenida del proveedor tras la autenticacion.
#[derive(Debug, Clone)]
pub struct OAuthUser {
    /// ID unico del usuario en el proveedor.
    pub id: String,
    /// Email del usuario (puede ser None si el proveedor no lo devuelve).
    pub email: Option<String>,
    /// Nombre del usuario.
    pub name: Option<String>,
    /// Proveedor de donde proviene el usuario.
    pub provider: OAuthProvider,
}

/// Error del cliente OAuth2.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    /// Proveedor no configurado.
    #[error("proveedor {0} no configurado")]
    ProviderNotConfigured(OAuthProvider),
    /// Error de red.
    #[error("error de red: {0}")]
    Http(String),
    /// Respuesta invalida del proveedor.
    #[error("respuesta invalida del proveedor: {0}")]
    InvalidResponse(String),
    /// Error OAuth2.
    #[error("error OAuth2: {0}")]
    OAuth(String),
}

/// Credenciales de un proveedor OAuth2.
struct ProviderCredentials {
    client_id: String,
    client_secret: String,
}

/// Cliente OAuth2 para Google y GitHub.
///
/// Construir con [`OAuthClient::from_config`]. Cada provider se habilita
/// independientemente mediante las variables de entorno correspondientes.
pub struct OAuthClient {
    google: Option<ConfiguredClient>,
    google_creds: Option<ProviderCredentials>,
    github: Option<ConfiguredClient>,
    github_creds: Option<ProviderCredentials>,
    http: reqwest::Client,
}

impl OAuthClient {
    /// Construye el cliente a partir de la configuracion del modulo de auth.
    pub fn from_config(config: &AuthConfig, http: reqwest::Client) -> Self {
        let (google, google_creds) = match (
            config.oauth_google_client_id.as_ref(),
            config.oauth_google_client_secret.as_ref(),
        ) {
            (Some(id), Some(secret)) => {
                let client = BasicClient::new(ClientId::new(id.clone()))
                    .set_client_secret(ClientSecret::new(secret.clone()))
                    .set_auth_uri(
                        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".into())
                            .expect("Google auth URL invalida"),
                    )
                    .set_token_uri(
                        TokenUrl::new("https://oauth2.googleapis.com/token".into())
                            .expect("Google token URL invalida"),
                    );
                let creds = ProviderCredentials {
                    client_id: id.clone(),
                    client_secret: secret.clone(),
                };
                (Some(client), Some(creds))
            }
            _ => (None, None),
        };

        let (github, github_creds) = match (
            config.oauth_github_client_id.as_ref(),
            config.oauth_github_client_secret.as_ref(),
        ) {
            (Some(id), Some(secret)) => {
                let client = BasicClient::new(ClientId::new(id.clone()))
                    .set_client_secret(ClientSecret::new(secret.clone()))
                    .set_auth_uri(
                        AuthUrl::new("https://github.com/login/oauth/authorize".into())
                            .expect("GitHub auth URL invalida"),
                    )
                    .set_token_uri(
                        TokenUrl::new("https://github.com/login/oauth/access_token".into())
                            .expect("GitHub token URL invalida"),
                    );
                let creds = ProviderCredentials {
                    client_id: id.clone(),
                    client_secret: secret.clone(),
                };
                (Some(client), Some(creds))
            }
            _ => (None, None),
        };

        Self {
            google,
            google_creds,
            github,
            github_creds,
            http,
        }
    }

    /// Genera la URL de autorizacion con PKCE y state CSRF.
    ///
    /// Retorna `(url, state, pkce_verifier)`. El llamador debe:
    /// 1. Redirigir al usuario a `url`.
    /// 2. Persistir `state` y `pkce_verifier` en sesion para el callback.
    pub fn authorization_url(
        &self,
        provider: OAuthProvider,
        redirect_uri: &str,
    ) -> Result<(url::Url, CsrfToken, PkceCodeVerifier), OAuthError> {
        let client = self.client_for(provider)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let redirect = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| OAuthError::OAuth(e.to_string()))?;

        let scopes = match provider {
            OAuthProvider::Google => vec![
                Scope::new("openid".into()),
                Scope::new("email".into()),
                Scope::new("profile".into()),
            ],
            OAuthProvider::GitHub => vec![
                Scope::new("read:user".into()),
                Scope::new("user:email".into()),
            ],
        };

        let (url, state) = client
            .authorize_url(CsrfToken::new_random)
            .set_redirect_uri(Cow::Owned(redirect))
            .set_pkce_challenge(pkce_challenge)
            .add_scopes(scopes)
            .url();

        Ok((url, state, pkce_verifier))
    }

    /// Intercambia un codigo de autorizacion por la informacion del usuario.
    ///
    /// El llamador debe pasar el mismo `redirect_uri` y el `pkce_verifier` guardados en sesion.
    pub async fn exchange_code(
        &self,
        provider: OAuthProvider,
        code: &str,
        verifier: PkceCodeVerifier,
        redirect_uri: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let (token_url, client_id, client_secret) = match provider {
            OAuthProvider::Google => {
                let creds = self
                    .google_creds
                    .as_ref()
                    .ok_or(OAuthError::ProviderNotConfigured(provider))?;
                (
                    "https://oauth2.googleapis.com/token",
                    creds.client_id.as_str(),
                    creds.client_secret.as_str(),
                )
            }
            OAuthProvider::GitHub => {
                let creds = self
                    .github_creds
                    .as_ref()
                    .ok_or(OAuthError::ProviderNotConfigured(provider))?;
                (
                    "https://github.com/login/oauth/access_token",
                    creds.client_id.as_str(),
                    creds.client_secret.as_str(),
                )
            }
        };

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier.secret()),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let response = self
            .http
            .post(token_url)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(OAuthError::InvalidResponse(format!(
                "token exchange: {body}"
            )));
        }

        let token_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OAuthError::InvalidResponse(e.to_string()))?;

        let access_token = token_body["access_token"]
            .as_str()
            .ok_or_else(|| OAuthError::InvalidResponse("access_token ausente".into()))?
            .to_string();

        self.fetch_user_info(provider, &access_token).await
    }

    // ---------------------------------------------------------------------------
    // Privados
    // ---------------------------------------------------------------------------

    fn client_for(&self, provider: OAuthProvider) -> Result<&ConfiguredClient, OAuthError> {
        match provider {
            OAuthProvider::Google => self.google.as_ref(),
            OAuthProvider::GitHub => self.github.as_ref(),
        }
        .ok_or(OAuthError::ProviderNotConfigured(provider))
    }

    async fn fetch_user_info(
        &self,
        provider: OAuthProvider,
        access_token: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let (url, user_agent) = match provider {
            OAuthProvider::Google => ("https://www.googleapis.com/oauth2/v2/userinfo", None),
            OAuthProvider::GitHub => ("https://api.github.com/user", Some("anti-gravital")),
        };

        let mut req = self.http.get(url).bearer_auth(access_token);
        if let Some(ua) = user_agent {
            req = req.header("User-Agent", ua);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OAuthError::InvalidResponse(format!("user info: {body}")));
        }

        let info: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| OAuthError::InvalidResponse(e.to_string()))?;

        let id = info["id"]
            .as_i64()
            .map(|n| n.to_string())
            .or_else(|| info["id"].as_str().map(|s| s.to_string()))
            .ok_or_else(|| OAuthError::InvalidResponse("id ausente".into()))?;

        Ok(OAuthUser {
            id,
            email: info["email"].as_str().map(|s| s.to_string()),
            name: info["name"].as_str().map(|s| s.to_string()),
            provider,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn config_google() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "x".into(),
            jwt_public_key_pem: "y".into(),
            webauthn_rp_id: "localhost".into(),
            webauthn_origin: "http://localhost".into(),
            oauth_google_client_id: Some("google-client-id".into()),
            oauth_google_client_secret: Some("google-secret".into()),
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    fn config_empty() -> AuthConfig {
        AuthConfig {
            jwt_private_key_pem: "x".into(),
            jwt_public_key_pem: "y".into(),
            webauthn_rp_id: "localhost".into(),
            webauthn_origin: "http://localhost".into(),
            oauth_google_client_id: None,
            oauth_google_client_secret: None,
            oauth_github_client_id: None,
            oauth_github_client_secret: None,
        }
    }

    #[test]
    fn authorization_url_google_contains_accounts_google() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_google(), http);
        let (url, _state, _verifier) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/callback")
            .expect("debe generar URL para Google");
        assert!(
            url.host_str().unwrap_or("").contains("google.com"),
            "URL debe apuntar a google.com: {url}"
        );
    }

    #[test]
    fn authorization_url_unconfigured_returns_error() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_empty(), http);
        let result =
            client.authorization_url(OAuthProvider::Google, "http://localhost/callback");
        assert!(matches!(result, Err(OAuthError::ProviderNotConfigured(_))));
    }

    #[test]
    fn authorization_url_github_contains_github() {
        let mut cfg = config_google();
        cfg.oauth_github_client_id = Some("github-id".into());
        cfg.oauth_github_client_secret = Some("github-secret".into());
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&cfg, http);
        let (url, _state, _verifier) = client
            .authorization_url(OAuthProvider::GitHub, "http://localhost/callback")
            .expect("debe generar URL para GitHub");
        assert!(
            url.host_str().unwrap_or("").contains("github.com"),
            "URL debe apuntar a github.com: {url}"
        );
    }

    #[test]
    fn pkce_verifier_is_different_each_call() {
        let http = reqwest::Client::new();
        let client = OAuthClient::from_config(&config_google(), http);
        let (_, _, v1) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/c")
            .unwrap();
        let (_, _, v2) = client
            .authorization_url(OAuthProvider::Google, "http://localhost/c")
            .unwrap();
        assert_ne!(v1.secret(), v2.secret(), "verifiers deben ser unicos");
    }
}
