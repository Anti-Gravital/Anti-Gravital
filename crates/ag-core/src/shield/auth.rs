//! Capa de autenticacion JWT Ed25519.
//!
//! Verifica el header `Authorization: Bearer <token>` con una clave
//! publica Ed25519 cargada al arranque. Valida la firma, la expiracion
//! y opcionalmente el issuer y la audience declarados en
//! `AuthConfig`. Cuando la verificacion tiene exito, los claims se
//! adjuntan a las extensiones del request y los handlers pueden
//! consumirlos via el extractor `Claims<T>` declarado al final del
//! modulo.
//!
//! Fallos producen `AgError::Auth` con status 401.

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

/// Contexto de autenticacion adjunto a las extensiones del request.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Claims decodificados como JSON generico.
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

/// Tower layer de autenticacion JWT.
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
    // jsonwebtoken acepta 60 segundos de margen para `exp` y `nbf` por
    // defecto. Lo dejamos en 0 por seguridad estricta: si una
    // implementacion requiere tolerar deriva de reloj, hace falta
    // anadirlo via configuracion explicita.
    validation.leeway = 0;
    if let Some(issuer) = &config.expected_issuer {
        validation.set_issuer(std::slice::from_ref(issuer));
    }
    if let Some(audience) = &config.expected_audience {
        validation.set_audience(std::slice::from_ref(audience));
    } else {
        // Por defecto jsonwebtoken exige aud cuando esta declarado; lo
        // dejamos opcional cuando el operador no la configura.
        validation.validate_aud = false;
    }
    validation
}

impl AuthLayer {
    /// Construye la capa cargando y validando la clave publica.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` si la clave no esta declarada, no
    /// existe el archivo, o el contenido no es un PEM Ed25519 valido.
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

/// Servicio Tower que valida el JWT antes de delegar al inner.
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
    /// Future del servicio de autenticacion.
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

/// Extractor tipado de claims del JWT.
///
/// Lee el `AuthContext` inyectado por `AuthService` desde las
/// extensiones del request y deserializa los claims al tipo `T`.
/// Devuelve `AgError::Auth` si la capa Auth no esta activa para esta
/// ruta o si la deserializacion falla.
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
