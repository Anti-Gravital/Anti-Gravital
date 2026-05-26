//! Ed25519 JWT authentication layer.
//!
//! Verifies the `Authorization: Bearer <token>` header with an Ed25519
//! public key loaded at startup. It validates the signature, the
//! expiration and optionally the issuer and audience declared in
//! `AuthConfig`. When verification succeeds, the claims are attached to
//! the request extensions and handlers can consume them through the
//! `Claims<T>` extractor declared at the end of the module.
//!
//! Failures produce `AgError::Auth` with status 401.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::async_trait;
use axum::extract::{FromRequestParts, Request};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tower::{Layer, Service};

use crate::config::AuthConfig;
use crate::error::AgError;

/// Authentication context attached to the request extensions.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Claims decoded as generic JSON.
    pub claims: Value,
}

struct AuthInner {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl std::fmt::Debug for AuthInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthInner")
            .field("validation", &self.validation)
            .finish_non_exhaustive()
    }
}

/// JWT authentication Tower layer.
#[derive(Clone)]
pub struct AuthLayer {
    inner: Arc<AuthInner>,
}

impl std::fmt::Debug for AuthLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthLayer").finish_non_exhaustive()
    }
}

fn load_public_key(config: &AuthConfig) -> Result<DecodingKey, AgError> {
    match (&config.public_key_pem, &config.public_key_path) {
        (Some(_), Some(_)) => Err(AgError::Config(
            "auth.public_key_pem and auth.public_key_path are mutually exclusive".to_owned(),
        )),
        (Some(pem), None) => DecodingKey::from_ed_pem(pem.as_bytes())
            .map_err(|e| AgError::Config(format!("invalid Ed25519 PEM: {e}"))),
        (None, Some(path)) => {
            let bytes = std::fs::read(path)
                .map_err(|e| AgError::Config(format!("cannot read {}: {e}", path.display())))?;
            DecodingKey::from_ed_pem(&bytes).map_err(|e| {
                AgError::Config(format!("invalid Ed25519 PEM at {}: {e}", path.display()))
            })
        }
        (None, None) => Err(AgError::Config(
            "auth requires public_key_pem or public_key_path".to_owned(),
        )),
    }
}

fn build_validation(config: &AuthConfig) -> Validation {
    let mut validation = Validation::new(Algorithm::EdDSA);
    // jsonwebtoken accepts 60 seconds of leeway for `exp` and `nbf` by
    // default. We set it to 0 for strict security: if an implementation
    // needs to tolerate clock drift, it must be added via explicit
    // configuration.
    validation.leeway = 0;
    if let Some(issuer) = &config.expected_issuer {
        validation.set_issuer(std::slice::from_ref(issuer));
    }
    if let Some(audience) = &config.expected_audience {
        validation.set_audience(std::slice::from_ref(audience));
    } else {
        // By default jsonwebtoken requires aud when declared; we leave
        // it optional when the operator does not configure it.
        validation.validate_aud = false;
    }
    validation
}

impl AuthLayer {
    /// Builds the layer by loading and validating the public key.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Config` if the key is not declared, the file
    /// does not exist, or the contents are not a valid Ed25519 PEM.
    pub fn new(config: &AuthConfig) -> Result<Self, AgError> {
        let decoding_key = load_public_key(config)?;
        let validation = build_validation(config);
        Ok(Self {
            inner: Arc::new(AuthInner {
                decoding_key,
                validation,
            }),
        })
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            shared: Arc::clone(&self.inner),
        }
    }
}

/// Tower service that validates the JWT before delegating to the inner.
#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    shared: Arc<AuthInner>,
}

impl<S> std::fmt::Debug for AuthService<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService").finish_non_exhaustive()
    }
}

fn extract_bearer(req: &Request) -> Result<&str, AgError> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or_else(|| AgError::Auth("missing Authorization header".to_owned()))?
        .to_str()
        .map_err(|_| AgError::Auth("Authorization header is not valid UTF-8".to_owned()))?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| AgError::Auth("Authorization scheme must be Bearer".to_owned()))
        .map(str::trim)
}

fn verify(req: &Request, inner: &AuthInner) -> Result<AuthContext, AgError> {
    let token = extract_bearer(req)?;
    let data = jsonwebtoken::decode::<Value>(token, &inner.decoding_key, &inner.validation)
        .map_err(|e| AgError::Auth(format!("invalid token: {e}")))?;
    Ok(AuthContext {
        claims: data.claims,
    })
}

impl<S> Service<Request> for AuthService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = AuthFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        match verify(&req, &self.shared) {
            Ok(ctx) => {
                req.extensions_mut().insert(ctx);
                AuthFuture {
                    state: AuthState::Inner {
                        future: self.inner.call(req),
                    },
                }
            }
            Err(err) => AuthFuture {
                state: AuthState::Reject {
                    response: Some(err.into_response()),
                },
            },
        }
    }
}

pin_project! {
    /// Future of the authentication service.
    #[derive(Debug)]
    pub struct AuthFuture<F> {
        #[pin]
        state: AuthState<F>,
    }
}

pin_project! {
    #[project = AuthStateProj]
    #[derive(Debug)]
    enum AuthState<F> {
        Inner { #[pin] future: F },
        Reject { response: Option<Response> },
    }
}

impl<F, E> Future for AuthFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.state.project() {
            AuthStateProj::Inner { future } => future.poll(cx),
            AuthStateProj::Reject { response } => {
                let resp = response.take().expect("polled AuthFuture after completion");
                Poll::Ready(Ok(resp))
            }
        }
    }
}

/// Typed extractor for JWT claims.
///
/// Reads the `AuthContext` injected by `AuthService` from the request
/// extensions and deserializes the claims into the type `T`. Returns
/// `AgError::Auth` if the Auth layer is not active for this route or if
/// deserialization fails.
#[derive(Debug, Clone)]
pub struct Claims<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Claims<T>
where
    T: DeserializeOwned + Send + 'static,
    S: Send + Sync,
{
    type Rejection = AgError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| AgError::Auth("AuthContext missing; is AuthLayer active?".to_owned()))?;
        let value: T = serde_json::from_value(ctx.claims.clone())
            .map_err(|e| AgError::Auth(format!("cannot deserialize claims: {e}")))?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::HeaderValue;

    #[test]
    fn missing_authorization_is_rejected() {
        let req = Request::builder().uri("/x").body(Body::empty()).unwrap();
        let err = extract_bearer(&req).unwrap_err();
        assert_eq!(err.code(), "auth_error");
    }

    #[test]
    fn non_bearer_scheme_is_rejected() {
        let req = Request::builder()
            .uri("/x")
            .header(
                AUTHORIZATION,
                HeaderValue::from_static("Basic dXNlcjpwYXNz"),
            )
            .body(Body::empty())
            .unwrap();
        let err = extract_bearer(&req).unwrap_err();
        assert_eq!(err.code(), "auth_error");
    }

    #[test]
    fn bearer_extracts_token() {
        let req = Request::builder()
            .uri("/x")
            .header(
                AUTHORIZATION,
                HeaderValue::from_static("Bearer abc.def.ghi"),
            )
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_bearer(&req).unwrap(), "abc.def.ghi");
    }

    #[test]
    fn empty_config_errors() {
        let cfg = AuthConfig::default();
        let err = AuthLayer::new(&cfg).unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn both_pem_and_path_errors() {
        let cfg = AuthConfig {
            enabled: true,
            public_key_pem: Some("pem".to_owned()),
            public_key_path: Some(std::path::PathBuf::from("/nope")),
            ..AuthConfig::default()
        };
        let err = AuthLayer::new(&cfg).unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn invalid_pem_errors() {
        let cfg = AuthConfig {
            enabled: true,
            public_key_pem: Some("not a real PEM".to_owned()),
            ..AuthConfig::default()
        };
        let err = AuthLayer::new(&cfg).unwrap_err();
        assert_eq!(err.code(), "config_error");
    }
}
