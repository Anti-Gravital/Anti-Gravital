//! Bearer token authentication middleware for the HTTP server.
//!
//! Two modes, selected by [`AuthState`]:
//!
//! - Static-token mode (default): compares the Bearer against the configured
//!   `STORE_TOKEN`. An empty token means the server accepts all requests
//!   without authentication (loopback development only).
//! - JWT mode (feature `auth`): validates the Bearer as an Ed25519 JWT via
//!   `ag-auth`, using the configured PEM public key. Selected when
//!   `jwt_public_key_pem` is set and the `auth` feature is enabled.

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Authentication state for the storage HTTP server.
#[derive(Clone)]
pub struct AuthState {
    /// Static Bearer token; empty = development mode (no authentication).
    token: Arc<String>,
    /// JWT verifier (feature `auth`): when present, the Bearer is validated as
    /// an Ed25519 JWT via `ag-auth` instead of compared as a static string.
    #[cfg(feature = "auth")]
    jwt: Option<Arc<ag_auth::JwtSigner>>,
}

impl AuthState {
    /// Static-token mode (the default). An empty token enables dev mode.
    pub fn from_static_token(token: impl Into<String>) -> Self {
        Self {
            token: Arc::new(token.into()),
            #[cfg(feature = "auth")]
            jwt: None,
        }
    }

    /// JWT mode (feature `auth`): validate the Bearer as an Ed25519 JWT with the
    /// given PEM public key.
    #[cfg(feature = "auth")]
    pub fn from_jwt_public_pem(public_key_pem: impl Into<String>) -> Self {
        // `JwtSigner::verify` only reads the public key; the private key is
        // never used on the verification path, so an empty one is safe here.
        let signer = ag_auth::JwtSigner::new(String::new(), public_key_pem.into());
        Self {
            token: Arc::new(String::new()),
            jwt: Some(Arc::new(signer)),
        }
    }

    /// Builds the auth state from the storage configuration: JWT mode when a
    /// public key is configured and the `auth` feature is on, static-token
    /// mode otherwise.
    pub fn from_config(config: &crate::StorageConfig) -> Self {
        #[cfg(feature = "auth")]
        if let Some(pem) = config
            .jwt_public_key_pem
            .as_ref()
            .filter(|pem| !pem.trim().is_empty())
        {
            return Self::from_jwt_public_pem(pem.clone());
        }
        Self::from_static_token(config.store_token.clone())
    }

    /// True when the server accepts requests without authentication.
    fn is_open(&self) -> bool {
        #[cfg(feature = "auth")]
        if self.jwt.is_some() {
            return false;
        }
        self.token.is_empty()
    }

    /// Authorizes a presented Bearer credential.
    fn authorize(&self, presented: &str) -> bool {
        #[cfg(feature = "auth")]
        if let Some(jwt) = &self.jwt {
            return jwt.verify(presented).is_ok();
        }
        !self.token.is_empty() && presented == self.token.as_str()
    }
}

/// Axum middleware that requires `Authorization: Bearer <credential>`.
///
/// Applied at `Router` level, not per individual route, so that any
/// new route inherits the protection automatically.
pub async fn bearer_auth_middleware(
    State(state): State<AuthState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if state.is_open() {
        // Dev mode: no authentication configured.
        return next.run(request).await;
    }

    let bearer = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match bearer {
        Some(credential) if state.authorize(credential) => next.run(request).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn ok_handler() -> StatusCode {
        StatusCode::OK
    }

    fn app_with_token(token: &str) -> Router {
        Router::new()
            .route("/test", get(ok_handler))
            .layer(middleware::from_fn_with_state(
                AuthState::from_static_token(token),
                bearer_auth_middleware,
            ))
    }

    #[tokio::test]
    async fn empty_token_allows_all_requests() {
        let app = app_with_token("");
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_token_allows_request() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer secret123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer wrongtoken")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn malformed_bearer_returns_401() {
        let app = app_with_token("secret123");
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Basic secret123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[cfg(feature = "auth")]
    mod jwt {
        use super::*;
        use ag_auth::{Claims, JwtSigner};

        fn keypair() -> (String, String) {
            use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
            use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
            use ed25519_dalek::SigningKey;
            use rand_core::OsRng;
            let sk = SigningKey::generate(&mut OsRng);
            let vk = sk.verifying_key();
            (
                sk.to_pkcs8_pem(LineEnding::LF).unwrap().to_string(),
                vk.to_public_key_pem(LineEnding::LF).unwrap(),
            )
        }

        fn token(private_pem: &str, public_pem: &str, exp_offset_secs: i64) -> String {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let exp = if exp_offset_secs >= 0 {
                now + exp_offset_secs as u64
            } else {
                now.saturating_sub((-exp_offset_secs) as u64)
            };
            let claims = Claims {
                sub: "user-1".into(),
                exp,
                iat: now,
                jti: "jti-1".into(),
                role: "admin".into(),
            };
            JwtSigner::new(private_pem.to_owned(), public_pem.to_owned())
                .sign(&claims)
                .unwrap()
        }

        fn app_jwt(public_pem: &str) -> Router {
            Router::new()
                .route("/test", get(ok_handler))
                .layer(middleware::from_fn_with_state(
                    AuthState::from_jwt_public_pem(public_pem.to_owned()),
                    bearer_auth_middleware,
                ))
        }

        async fn status_with_bearer(public_pem: &str, bearer: &str) -> StatusCode {
            let req = Request::builder()
                .uri("/test")
                .header("Authorization", format!("Bearer {bearer}"))
                .body(Body::empty())
                .unwrap();
            app_jwt(public_pem).oneshot(req).await.unwrap().status()
        }

        #[tokio::test]
        async fn valid_jwt_allows_request() {
            let (priv_pem, pub_pem) = keypair();
            let jwt = token(&priv_pem, &pub_pem, 3600);
            assert_eq!(status_with_bearer(&pub_pem, &jwt).await, StatusCode::OK);
        }

        #[tokio::test]
        async fn expired_jwt_returns_401() {
            let (priv_pem, pub_pem) = keypair();
            let jwt = token(&priv_pem, &pub_pem, -3600);
            assert_eq!(
                status_with_bearer(&pub_pem, &jwt).await,
                StatusCode::UNAUTHORIZED
            );
        }

        #[tokio::test]
        async fn jwt_signed_with_other_key_returns_401() {
            let (priv_pem, pub_pem) = keypair();
            let (_other_priv, other_pub) = keypair();
            let jwt = token(&priv_pem, &pub_pem, 3600);
            // Verifier configured with a different public key must reject it.
            assert_eq!(
                status_with_bearer(&other_pub, &jwt).await,
                StatusCode::UNAUTHORIZED
            );
        }

        #[tokio::test]
        async fn garbage_bearer_returns_401() {
            let (_priv_pem, pub_pem) = keypair();
            assert_eq!(
                status_with_bearer(&pub_pem, "not-a-jwt").await,
                StatusCode::UNAUTHORIZED
            );
        }
    }
}
