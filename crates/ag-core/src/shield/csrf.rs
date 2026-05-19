//! Capa CSRF de Anti-Gravital.
//!
//! Implementa proteccion CSRF apatrida via patron double-submit cookie:
//! en peticiones que mutan estado (POST, PUT, PATCH, DELETE), el
//! servidor exige que la cookie configurada y el header configurado
//! contengan exactamente el mismo valor opaco. Si faltan o no
//! coinciden, la peticion se rechaza con `AgError::Csrf` (status 403).
//!
//! Las peticiones seguras (GET, HEAD, OPTIONS, TRACE) pasan sin
//! verificacion porque por definicion no mutan estado del servidor.
//!
//! El emisor del token (cookie) es responsabilidad del proyecto: la
//! Shield no asume sesion ni storage. Una implementacion tipica usa
//! un endpoint dedicado (por ejemplo, GET `/csrf-token`) que setea
//! la cookie en la respuesta.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::Method;
use axum::response::{IntoResponse, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::config::CsrfConfig;
use crate::error::AgError;

/// Tower layer que aplica double-submit cookie CSRF.
#[derive(Debug, Clone)]
pub struct CsrfLayer {
    config: CsrfConfig,
}

impl CsrfLayer {
    /// Construye una `CsrfLayer` con la configuracion dada.
    #[must_use]
    pub fn new(config: CsrfConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Servicio que valida CSRF antes de delegar al inner.
#[derive(Debug, Clone)]
pub struct CsrfService<S> {
    inner: S,
    config: CsrfConfig,
}

fn is_safe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

fn extract_cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|kv| {
        let kv = kv.trim();
        let (k, v) = kv.split_once('=')?;
        if k.trim() == name {
            Some(v.trim())
        } else {
            None
        }
    })
}

fn validate(req: &Request, config: &CsrfConfig) -> Result<(), AgError> {
    if is_safe_method(req.method()) {
        return Ok(());
    }

    let headers = req.headers();
    let header_value = headers
        .get(config.token_header.as_str())
        .ok_or_else(|| AgError::Csrf("missing CSRF header".to_owned()))?
        .to_str()
        .map_err(|_| AgError::Csrf("CSRF header is not valid UTF-8".to_owned()))?;

    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .ok_or_else(|| AgError::Csrf("missing cookie".to_owned()))?
        .to_str()
        .map_err(|_| AgError::Csrf("cookie header is not valid UTF-8".to_owned()))?;

    let cookie_value = extract_cookie_value(cookie_header, &config.token_cookie)
        .ok_or_else(|| AgError::Csrf("missing CSRF cookie".to_owned()))?;

    if header_value == cookie_value && !header_value.is_empty() {
        Ok(())
    } else {
        Err(AgError::Csrf("CSRF token mismatch".to_owned()))
    }
}

impl<S> Service<Request> for CsrfService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = CsrfFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        match validate(&req, &self.config) {
            Ok(()) => CsrfFuture {
                state: CsrfState::Inner {
                    future: self.inner.call(req),
                },
            },
            Err(err) => CsrfFuture {
                state: CsrfState::Reject {
                    response: Some(err.into_response()),
                },
            },
        }
    }
}

pin_project! {
    /// Future del servicio CSRF.
    #[derive(Debug)]
    pub struct CsrfFuture<F> {
        #[pin]
        state: CsrfState<F>,
    }
}

pin_project! {
    #[project = CsrfStateProj]
    #[derive(Debug)]
    enum CsrfState<F> {
        Inner { #[pin] future: F },
        Reject { response: Option<Response> },
    }
}

impl<F, E> Future for CsrfFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.state.project() {
            CsrfStateProj::Inner { future } => future.poll(cx),
            CsrfStateProj::Reject { response } => {
                let resp = response.take().expect("polled CsrfFuture after completion");
                Poll::Ready(Ok(resp))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn config() -> CsrfConfig {
        CsrfConfig {
            enabled: true,
            token_header: "x-csrf-token".to_owned(),
            token_cookie: "ag_csrf".to_owned(),
        }
    }

    #[test]
    fn safe_methods_skip_validation() {
        for method in [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE] {
            assert!(is_safe_method(&method), "method {method:?} should be safe");
        }
    }

    #[test]
    fn unsafe_methods_require_validation() {
        for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
            assert!(
                !is_safe_method(&method),
                "method {method:?} should not be safe"
            );
        }
    }

    #[test]
    fn extract_cookie_value_finds_named_cookie() {
        let header = "session=abc; ag_csrf=xyz; theme=dark";
        assert_eq!(extract_cookie_value(header, "ag_csrf"), Some("xyz"));
        assert_eq!(extract_cookie_value(header, "session"), Some("abc"));
        assert_eq!(extract_cookie_value(header, "missing"), None);
    }

    #[test]
    fn validate_passes_when_header_matches_cookie() {
        let cfg = config();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/x")
            .header("x-csrf-token", HeaderValue::from_static("token-123"))
            .header(
                axum::http::header::COOKIE,
                HeaderValue::from_static("ag_csrf=token-123"),
            )
            .body(axum::body::Body::empty())
            .unwrap();
        assert!(validate(&req, &cfg).is_ok());
    }

    #[test]
    fn validate_rejects_when_header_missing() {
        let cfg = config();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/x")
            .header(
                axum::http::header::COOKIE,
                HeaderValue::from_static("ag_csrf=token-123"),
            )
            .body(axum::body::Body::empty())
            .unwrap();
        let err = validate(&req, &cfg).unwrap_err();
        assert_eq!(err.code(), "csrf_error");
    }

    #[test]
    fn validate_rejects_when_values_differ() {
        let cfg = config();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/x")
            .header("x-csrf-token", HeaderValue::from_static("aaa"))
            .header(
                axum::http::header::COOKIE,
                HeaderValue::from_static("ag_csrf=bbb"),
            )
            .body(axum::body::Body::empty())
            .unwrap();
        let err = validate(&req, &cfg).unwrap_err();
        assert_eq!(err.code(), "csrf_error");
    }

    #[test]
    fn validate_rejects_empty_token() {
        let cfg = config();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/x")
            .header("x-csrf-token", HeaderValue::from_static(""))
            .header(
                axum::http::header::COOKIE,
                HeaderValue::from_static("ag_csrf="),
            )
            .body(axum::body::Body::empty())
            .unwrap();
        let err = validate(&req, &cfg).unwrap_err();
        assert_eq!(err.code(), "csrf_error");
    }
}
