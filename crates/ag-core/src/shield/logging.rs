//! Capa de logging estructurado para la pipeline Shield.
//!
//! Emite un evento `tracing` por cada request entrante con metodo,
//! path y latencia total. Esta es la primera capa de la pipeline en
//! orden de adicion porque otras capas se beneficiarian del
//! contexto de tracing aunque fallen.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::extract::Request;
use axum::response::Response;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

/// Layer Tower que envuelve un servicio con logging estructurado.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

/// Servicio que registra cada request con `tracing`.
#[derive(Debug, Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S> Service<Request> for LoggingService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LoggingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let start = Instant::now();
        LoggingFuture {
            inner: self.inner.call(req),
            method,
            uri,
            start,
        }
    }
}

pin_project! {
    /// Future del servicio de logging.
    #[derive(Debug)]
    pub struct LoggingFuture<F> {
        #[pin]
        inner: F,
        method: axum::http::Method,
        uri: axum::http::Uri,
        start: Instant,
    }
}

impl<F, E> Future for LoggingFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let elapsed_ms = this.start.elapsed().as_secs_f64() * 1000.0;
                match &result {
                    Ok(resp) => tracing::info!(
                        method = %this.method,
                        path = %this.uri.path(),
                        status = resp.status().as_u16(),
                        latency_ms = elapsed_ms,
                        "request completed"
                    ),
                    Err(_) => tracing::error!(
                        method = %this.method,
                        path = %this.uri.path(),
                        latency_ms = elapsed_ms,
                        "request errored"
                    ),
                }
                Poll::Ready(result)
            }
        }
    }
}
