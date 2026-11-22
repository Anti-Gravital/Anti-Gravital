//! Native OAuth2 Authorization Code + PKCE client for Google and GitHub.
//!
//! The PKCE and CSRF primitives are built here from primitives already present
//! in `ag-auth` (RFC 7636, RFC 6749 section 10.12); no third-party OAuth crate
//! is involved. Token exchange and user-info retrieval use the caller-supplied
//! `reqwest` client (ADR-0018). Secret newtypes redact their `Debug` output so
//! accidental logging cannot leak a state or verifier value.

use crate::config::AuthConfig;
use base64ct::{Base64UrlUnpadded, Encoding};
use sha2::{Digest, Sha256};

/// Random bytes behind a PKCE verifier. RFC 7636 requires 32..=96 bytes before
/// base64url encoding, which yields a 43-character verifier for 32 bytes.
const PKCE_RANDOM_BYTES: usize = 32;
/// Random bytes behind a CSRF state: 128 bits (RFC 6749 section 10.12).
const STATE_RANDOM_BYTES: usize = 16;

/// CSRF `state` value generated for an authorization request.
///
/// The caller persists this value in the session and compares it on callback;
/// the client does not silently verify it. `Debug` is redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct OAuthState(String);

impl OAuthState {
    /// Returns the raw state value. Treat it as a secret until the callback
    /// has been matched against the session copy.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for OAuthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OAuthState([REDACTED])")
    }
}

/// PKCE `code_verifier` generated for an authorization request.
///
/// The caller persists this value in the session and passes it back to
/// [`OAuthClient::exchange_code`]. Single use by convention. `Debug` is
/// redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct PkceVerifier(String);

impl PkceVerifier {
    /// Returns the raw verifier value passed to the token endpoint.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for PkceVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PkceVerifier([REDACTED])")
    }
}

/// Supported OAuth2 provider.
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

/// User information obtained from the provider after authentication.
#[derive(Debug, Clone)]
pub struct OAuthUser {
    /// Unique user ID at the provider.
    pub id: String,
    /// User email (may be None if the provider does not return it).
    pub email: Option<String>,
    /// User display name.
    pub name: Option<String>,
    /// Provider from which the user came.
    pub provider: OAuthProvider,
}

/// OAuth2 client error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OAuthError {
    /// Provider not configured.
    #[error("provider {0} not configured")]
    ProviderNotConfigured(OAuthProvider),
    /// Network error.
    #[error("network error: {0}")]
    Http(String),
    /// Invalid provider response.
    #[error("invalid provider response: {0}")]
    InvalidResponse(String),
    /// OAuth2 URL or parameter error.
    #[error("OAuth2 error: {0}")]
    OAuth(String),
    /// Failed to obtain cryptographically secure randomness.
    #[error("entropy error: {0}")]
    Entropy(String),
}

/// Credentials for an OAuth2 provider.
struct ProviderCredentials {
    client_id: String,
    client_secret: String,
}

/// All provider-specific constants. URLs are owned so tests can point the
/// client at a local mock server without changing production configuration.
struct ProviderConfig {
    auth_url: String,
    token_url: String,
    userinfo_url: String,
    user_agent: Option<&'static str>,
    scopes: &'static [&'static str],
    credentials: ProviderCredentials,
}

/// OAuth2 client for Google and GitHub.
///
/// Build with [`OAuthClient::from_config`]. Each provider is enabled
/// independently via the corresponding environment variables.
pub struct OAuthClient {
    google: Option<ProviderConfig>,
    github: Option<ProviderConfig>,
    http: reqwest::Client,
}

impl OAuthClient {
    /// Builds the client from the auth module configuration.
    pub fn from_config(config: &AuthConfig, http: reqwest::Client) -> Self {
        let google = match (
            config.oauth_google_client_id.as_ref(),
            config.oauth_google_client_secret.as_ref(),
        ) {
            (Some(id), Some(secret)) => Some(ProviderConfig {
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
                token_url: "https://oauth2.googleapis.com/token".into(),
                userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".into(),
                user_agent: None,
                scopes: &["openid", "email", "profile"],
                credentials: ProviderCredentials {
                    client_id: id.clone(),
                    client_secret: secret.clone(),
                },
            }),
            _ => None,
        };

        let github = match (
            config.oauth_github_client_id.as_ref(),
            config.oauth_github_client_secret.as_ref(),
        ) {
            (Some(id), Some(secret)) => Some(ProviderConfig {
                auth_url: "https://github.com/login/oauth/authorize".into(),
                token_url: "https://github.com/login/oauth/access_token".into(),
                userinfo_url: "https://api.github.com/user".into(),
                user_agent: Some("anti-gravital"),
                scopes: &["read:user", "user:email"],
                credentials: ProviderCredentials {
                    client_id: id.clone(),
                    client_secret: secret.clone(),
                },
            }),
            _ => None,
        };

        Self {
            google,
            github,
            http,
        }
    }

    /// Generates the authorization URL with PKCE S256 and a 128-bit CSRF state.
    ///
    /// Returns `(url, state, pkce_verifier)`. The caller must:
    /// 1. Redirect the user to `url`.
    /// 2. Persist `state` and `pkce_verifier` in the session for the callback,
    ///    where `state` must be compared with the returned value.
    pub fn authorization_url(
        &self,
        provider: OAuthProvider,
        redirect_uri: &str,
    ) -> Result<(url::Url, OAuthState, PkceVerifier), OAuthError> {
        let config = self.config_for(provider)?;
        let verifier = new_pkce_verifier()?;
        let state = new_state()?;
        let url = build_authorization_url(config, redirect_uri, &state, &verifier)?;
        Ok((url, state, verifier))
    }

    /// Exchanges an authorization code for user information.
    ///
    /// The caller must pass the same `redirect_uri` and the `pkce_verifier`
    /// saved in the session. No refresh-token grant exists: the provider's
    /// `refresh_token`, if any, is deliberately discarded.
    pub async fn exchange_code(
        &self,
        provider: OAuthProvider,
        code: &str,
        verifier: PkceVerifier,
        redirect_uri: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let config = self.config_for(provider)?;
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier.expose_secret()),
            ("client_id", config.credentials.client_id.as_str()),
            ("client_secret", config.credentials.client_secret.as_str()),
        ];

        let response = self
            .http
            .post(&config.token_url)
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
            .filter(|token| !token.is_empty())
            .ok_or_else(|| OAuthError::InvalidResponse("access_token absent".into()))?
            .to_string();

        self.fetch_user_info(config, provider, &access_token).await
    }

    // ---------------------------------------------------------------------------
    // Private
    // ---------------------------------------------------------------------------

    fn config_for(&self, provider: OAuthProvider) -> Result<&ProviderConfig, OAuthError> {
        match provider {
            OAuthProvider::Google => self.google.as_ref(),
            OAuthProvider::GitHub => self.github.as_ref(),
        }
        .ok_or(OAuthError::ProviderNotConfigured(provider))
    }

    async fn fetch_user_info(
        &self,
        config: &ProviderConfig,
        provider: OAuthProvider,
        access_token: &str,
    ) -> Result<OAuthUser, OAuthError> {
        let mut req = self
            .http
            .get(&config.userinfo_url)
            .bearer_auth(access_token);
        if let Some(ua) = config.user_agent {
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
            .ok_or_else(|| OAuthError::InvalidResponse("id absent".into()))?;

        Ok(OAuthUser {
            id,
            email: info["email"].as_str().map(|s| s.to_string()),
            name: info["name"].as_str().map(|s| s.to_string()),
            provider,
        })
    }
}

/// Builds the authorization URL with the exact query order used by oauth2 5:
/// `response_type`, `client_id`, `state`, `code_challenge`,
/// `code_challenge_method`, `redirect_uri`, `scope`.
fn build_authorization_url(
    config: &ProviderConfig,
    redirect_uri: &str,
    state: &OAuthState,
    verifier: &PkceVerifier,
) -> Result<url::Url, OAuthError> {
    let mut url =
        url::Url::parse(&config.auth_url).map_err(|e| OAuthError::OAuth(e.to_string()))?;
    let challenge = pkce_challenge_for_verifier(verifier.expose_secret());
    let scopes = config.scopes.join(" ");
    let params: Vec<(&str, &str)> = vec![
        ("response_type", "code"),
        ("client_id", config.credentials.client_id.as_str()),
        ("state", state.expose_secret()),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("redirect_uri", redirect_uri),
        ("scope", scopes.as_str()),
    ];
    url.query_pairs_mut().extend_pairs(params);
    Ok(url)
}

/// Generates a 128-bit base64url-no-pad CSRF state (22 characters).
fn new_state() -> Result<OAuthState, OAuthError> {
    Ok(OAuthState(random_b64url(STATE_RANDOM_BYTES)?))
}

/// Generates a 32-byte base64url-no-pad PKCE verifier (43 characters).
fn new_pkce_verifier() -> Result<PkceVerifier, OAuthError> {
    Ok(PkceVerifier(random_b64url(PKCE_RANDOM_BYTES)?))
}

/// CSPRNG bytes encoded as unpadded URL-safe base64.
fn random_b64url(count: usize) -> Result<String, OAuthError> {
    let mut bytes = vec![0u8; count];
    getrandom::fill(&mut bytes).map_err(|e| OAuthError::Entropy(e.to_string()))?;
    Ok(Base64UrlUnpadded::encode_string(&bytes))
}

/// `BASE64URL-ENCODE(SHA256(ASCII(code_verifier)))` from RFC 7636 section 4.2.
fn pkce_challenge_for_verifier(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    Base64UrlUnpadded::encode_string(&digest)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{
        matchers::{body_string, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const REDIRECT: &str = "http://localhost/callback";
    const RFC_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const RFC_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    fn http() -> reqwest::Client {
        crate::install_ring_provider();
        reqwest::Client::new()
    }

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

    fn google_config() -> ProviderConfig {
        ProviderConfig {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_url: "https://oauth2.googleapis.com/token".into(),
            userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".into(),
            user_agent: None,
            scopes: &["openid", "email", "profile"],
            credentials: ProviderCredentials {
                client_id: "google-client-id".into(),
                client_secret: "google-secret".into(),
            },
        }
    }

    fn github_config() -> ProviderConfig {
        ProviderConfig {
            auth_url: "https://github.com/login/oauth/authorize".into(),
            token_url: "https://github.com/login/oauth/access_token".into(),
            userinfo_url: "https://api.github.com/user".into(),
            user_agent: Some("anti-gravital"),
            scopes: &["read:user", "user:email"],
            credentials: ProviderCredentials {
                client_id: "github-client-id".into(),
                client_secret: "github-secret".into(),
            },
        }
    }

    fn client_with_endpoints(
        provider: OAuthProvider,
        http: reqwest::Client,
        token_url: String,
        userinfo_url: String,
    ) -> OAuthClient {
        let mut config = match provider {
            OAuthProvider::Google => google_config(),
            OAuthProvider::GitHub => github_config(),
        };
        config.credentials = ProviderCredentials {
            client_id: "cid".into(),
            client_secret: "csecret".into(),
        };
        config.token_url = token_url;
        config.userinfo_url = userinfo_url;
        let (google, github) = match provider {
            OAuthProvider::Google => (Some(config), None),
            OAuthProvider::GitHub => (None, Some(config)),
        };
        OAuthClient {
            google,
            github,
            http,
        }
    }

    fn assert_url_safe(value: &str) {
        assert!(
            value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "value must be unpadded URL-safe base64: {value}"
        );
    }

    fn token_form_body(code: &str) -> String {
        format!(
            "grant_type=authorization_code&code={code}&redirect_uri=http%3A%2F%2Flocalhost%2Fcallback&code_verifier={RFC_VERIFIER}&client_id=cid&client_secret=csecret"
        )
    }

    #[test]
    fn pkce_challenge_matches_rfc7636_appendix_b() {
        assert_eq!(pkce_challenge_for_verifier(RFC_VERIFIER), RFC_CHALLENGE);
    }

    #[test]
    fn csrf_state_is_128_bits_url_safe_and_unique() {
        let client = OAuthClient::from_config(&config_google(), http());
        let (_, first, _) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .unwrap();
        let (_, second, _) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .unwrap();
        assert_eq!(first.expose_secret().len(), 22);
        assert_url_safe(first.expose_secret());
        assert_ne!(first, second);
    }

    #[test]
    fn pkce_verifier_is_43_url_safe_chars() {
        let client = OAuthClient::from_config(&config_google(), http());
        let (_, _, verifier) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .unwrap();
        assert_eq!(verifier.expose_secret().len(), 43);
        assert_url_safe(verifier.expose_secret());
    }

    #[test]
    fn secret_newtypes_redact_debug_output() {
        let state = OAuthState("super-secret-state".into());
        let verifier = PkceVerifier("super-secret-verifier".into());
        let debug = format!("{state:?} {verifier:?}");
        assert!(!debug.contains("super-secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn authorization_url_google_contains_accounts_google() {
        let client = OAuthClient::from_config(&config_google(), http());
        let (url, _, _) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .expect("should generate URL for Google");
        assert!(
            url.host_str().unwrap_or("").contains("google.com"),
            "URL should point to google.com: {url}"
        );
    }

    #[test]
    fn authorization_url_unconfigured_returns_error() {
        let client = OAuthClient::from_config(&config_empty(), http());
        let result = client.authorization_url(OAuthProvider::Google, REDIRECT);
        assert!(matches!(result, Err(OAuthError::ProviderNotConfigured(_))));
    }

    #[test]
    fn authorization_url_github_contains_github_with_expected_scopes() {
        let mut cfg = config_google();
        cfg.oauth_github_client_id = Some("github-id".into());
        cfg.oauth_github_client_secret = Some("github-secret".into());
        let client = OAuthClient::from_config(&cfg, http());
        let (url, _, _) = client
            .authorization_url(OAuthProvider::GitHub, REDIRECT)
            .expect("should generate URL for GitHub");
        assert!(url.host_str().unwrap_or("").contains("github.com"));
        let scopes: Vec<String> = url
            .query_pairs()
            .filter_map(|(key, value)| (key == "scope").then(|| value.into_owned()))
            .collect();
        assert_eq!(scopes, vec!["read:user user:email".to_string()]);
    }

    #[test]
    fn pkce_verifier_is_different_each_call() {
        let client = OAuthClient::from_config(&config_google(), http());
        let (_, _, first) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .unwrap();
        let (_, _, second) = client
            .authorization_url(OAuthProvider::Google, REDIRECT)
            .unwrap();
        assert_ne!(first, second, "verifiers must be unique");
    }

    #[test]
    fn authorization_url_golden_query_matches_oauth2_5() {
        let state = OAuthState("0123456789abcdefghij".into());
        let verifier = PkceVerifier(RFC_VERIFIER.into());
        let url = build_authorization_url(&google_config(), REDIRECT, &state, &verifier).unwrap();
        assert_eq!(
            url.as_str(),
            "https://accounts.google.com/o/oauth2/v2/auth?response_type=code\
             &client_id=google-client-id&state=0123456789abcdefghij\
             &code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM\
             &code_challenge_method=S256\
             &redirect_uri=http%3A%2F%2Flocalhost%2Fcallback\
             &scope=openid+email+profile"
        );
    }

    #[test]
    fn authorization_url_form_encodes_special_characters() {
        let state = OAuthState("state".into());
        let verifier = PkceVerifier(RFC_VERIFIER.into());
        let redirect = "https://app.example/cb?next=/home page&x=1";
        let url = build_authorization_url(&google_config(), redirect, &state, &verifier).unwrap();
        let raw = url.query().unwrap();
        assert!(raw
            .contains("redirect_uri=https%3A%2F%2Fapp.example%2Fcb%3Fnext%3D%2Fhome+page%26x%3D1"));
        let decoded: Vec<String> = url
            .query_pairs()
            .filter_map(|(key, value)| (key == "redirect_uri").then(|| value.into_owned()))
            .collect();
        assert_eq!(decoded, vec![redirect.to_string()]);
    }

    #[tokio::test]
    async fn token_exchange_posts_exact_form_and_fetches_userinfo() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(header("accept", "application/json"))
            .and(body_string(token_form_body("auth-code")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "test-token", "token_type": "Bearer", "expires_in": 3600
            })))
            .expect(1)
            .mount(&mock)
            .await;
        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 42, "email": "user@example.com", "name": "Test User"
            })))
            .expect(1)
            .mount(&mock)
            .await;

        let client = client_with_endpoints(
            OAuthProvider::Google,
            http(),
            format!("{}/token", mock.uri()),
            format!("{}/userinfo", mock.uri()),
        );
        let user = client
            .exchange_code(
                OAuthProvider::Google,
                "auth-code",
                PkceVerifier(RFC_VERIFIER.into()),
                REDIRECT,
            )
            .await
            .expect("token exchange should succeed");

        assert_eq!(user.id, "42");
        assert_eq!(user.email.as_deref(), Some("user@example.com"));
        assert_eq!(user.name.as_deref(), Some("Test User"));
        assert!(matches!(user.provider, OAuthProvider::Google));
        mock.verify().await;
    }

    #[tokio::test]
    async fn token_exchange_non_success_returns_invalid_response() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid_grant"))
            .mount(&mock)
            .await;
        let client = client_with_endpoints(
            OAuthProvider::Google,
            http(),
            format!("{}/token", mock.uri()),
            format!("{}/userinfo", mock.uri()),
        );
        let error = client
            .exchange_code(
                OAuthProvider::Google,
                "auth-code",
                PkceVerifier(RFC_VERIFIER.into()),
                REDIRECT,
            )
            .await
            .expect_err("non-2xx must fail");
        match error {
            OAuthError::InvalidResponse(message) => {
                assert!(message.contains("token exchange: invalid_grant"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn token_exchange_malformed_json_returns_invalid_response() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock)
            .await;
        let client = client_with_endpoints(
            OAuthProvider::Google,
            http(),
            format!("{}/token", mock.uri()),
            format!("{}/userinfo", mock.uri()),
        );
        let error = client
            .exchange_code(
                OAuthProvider::Google,
                "auth-code",
                PkceVerifier(RFC_VERIFIER.into()),
                REDIRECT,
            )
            .await
            .expect_err("malformed JSON must fail");
        assert!(matches!(error, OAuthError::InvalidResponse(_)));
    }

    #[tokio::test]
    async fn token_exchange_missing_access_token_returns_invalid_response() {
        let mock = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "token_type": "Bearer"
            })))
            .mount(&mock)
            .await;
        let client = client_with_endpoints(
            OAuthProvider::Google,
            http(),
            format!("{}/token", mock.uri()),
            format!("{}/userinfo", mock.uri()),
        );
        let error = client
            .exchange_code(
                OAuthProvider::Google,
                "auth-code",
                PkceVerifier(RFC_VERIFIER.into()),
                REDIRECT,
            )
            .await
            .expect_err("missing access_token must fail");
        match error {
            OAuthError::InvalidResponse(message) => {
                assert_eq!(message, "access_token absent");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn token_exchange_transport_error_returns_http_error() {
        crate::install_ring_provider();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .expect("client with timeout");
        let client = client_with_endpoints(
            OAuthProvider::Google,
            http,
            "http://127.0.0.1:1/token".into(),
            "http://127.0.0.1:1/userinfo".into(),
        );
        let error = client
            .exchange_code(
                OAuthProvider::Google,
                "auth-code",
                PkceVerifier(RFC_VERIFIER.into()),
                REDIRECT,
            )
            .await
            .expect_err("closed port must fail");
        assert!(matches!(error, OAuthError::Http(_)));
    }

    #[tokio::test]
    async fn github_userinfo_keeps_user_agent_header() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user"))
            .and(header("user-agent", "anti-gravital"))
            .and(header("authorization", "Bearer gh-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "id": "gh-1", "login": "octocat" })),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let client = client_with_endpoints(
            OAuthProvider::GitHub,
            http(),
            "http://127.0.0.1:1/token".into(),
            format!("{}/user", mock.uri()),
        );
        let config = client.config_for(OAuthProvider::GitHub).unwrap();
        let user = client
            .fetch_user_info(config, OAuthProvider::GitHub, "gh-token")
            .await
            .expect("user info should succeed");
        assert_eq!(user.id, "gh-1");
        mock.verify().await;
    }
}
