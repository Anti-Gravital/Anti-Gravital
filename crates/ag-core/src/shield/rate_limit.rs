//! Capa de rate limiting por IP con governor (token bucket).
//!
//! Mantiene un keyed rate limiter en memoria con clave `IpAddr` del
//! cliente. Cuando un cliente excede `per_ip_rps` peticiones por
//! segundo (con permiso de pico hasta `burst`), las peticiones
//! adicionales se rechazan con `AgError::RateLimit` (status 429).
//!
//! La IP de origen se determina por el `SocketAddr` reportado por
//! Axum a traves del extractor `ConnectInfo`. Cuando el server vive
//! detras de un proxy de confianza que setea cabeceras como
//! `X-Forwarded-For`, la resolucion correcta de la IP real se entrega
//! por configuracion en una PR posterior.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::connect_info::ConnectInfo;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use governor::clock::DefaultClock;
#[cfg(test)]
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::config::RateLimitConfig;
use crate::error::AgError;

type IpLimiter =
    RateLimiter<IpAddr, governor::state::keyed::DashMapStateStore<IpAddr>, DefaultClock>;

#[cfg(test)]
type DirectLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Tower layer de rate limiting por IP.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<IpLimiter>,
}

impl std::fmt::Debug for RateLimitLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimitLayer").finish_non_exhaustive()
    }
}

impl RateLimitLayer {
    /// Construye una capa con los parametros indicados.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Config` si `per_ip_rps` o `burst` son 0.
    pub fn new(config: &RateLimitConfig) -> Result<Self, AgError> {
        let rps = NonZeroU32::new(config.per_ip_rps)
            .ok_or_else(|| AgError::Config("rate_limit.per_ip_rps must be > 0".to_owned()))?;
        let burst = NonZeroU32::new(config.burst)
            .ok_or_else(|| AgError::Config("rate_limit.burst must be > 0".to_owned()))?;

        let quota = Quota::per_second(rps).allow_burst(burst);
        let limiter = RateLimiter::dashmap(quota);
        Ok(Self {
            limiter: Arc::new(limiter),
        })
    }

    /// Acceso al limiter interno (uso interno y tests).
    #[cfg(test)]
    fn limiter(&self) -> Arc<IpLimiter> {
        Arc::clone(&self.limiter)
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: Arc::clone(&self.limiter),
        }
    }
}

/// Servicio Tower que aplica el rate limit.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<IpLimiter>,
}

impl<S> std::fmt::Debug for RateLimitService<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimitService").finish_non_exhaustive()
    }
}

fn client_ip(req: &Request) -> Option<IpAddr> {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
}

impl<S> Service<Request> for RateLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = RateLimitFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let Some(ip) = client_ip(&req) else {
            // Sin informacion de IP no aplicamos rate limit: en este caso
            // la pipeline va detras de un transporte que no expone IP
            // (tests con TcpListener sin ConnectInfo, por ejemplo).
            return RateLimitFuture {
                state: RateLimitState::Inner {
                    future: self.inner.call(req),
                },
            };
        };

        match self.limiter.check_key(&ip) {
            Ok(()) => RateLimitFuture {
                state: RateLimitState::Inner {
                    future: self.inner.call(req),
                },
            },
            Err(_) => RateLimitFuture {
                state: RateLimitState::Reject {
                    response: Some(AgError::RateLimit.into_response()),
                },
            },
        }
    }
}

pin_project! {
    /// Future del servicio rate limit.
    #[derive(Debug)]
    pub struct RateLimitFuture<F> {
        #[pin]
        state: RateLimitState<F>,
    }
}

pin_project! {
    #[project = RateLimitStateProj]
    #[derive(Debug)]
    enum RateLimitState<F> {
        Inner { #[pin] future: F },
        Reject { response: Option<Response> },
    }
}

impl<F, E> Future for RateLimitFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.state.project() {
            RateLimitStateProj::Inner { future } => future.poll(cx),
            RateLimitStateProj::Reject { response } => {
                let resp = response
                    .take()
                    .expect("polled RateLimitFuture after completion");
                Poll::Ready(Ok(resp))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn cfg(rps: u32, burst: u32) -> RateLimitConfig {
        RateLimitConfig {
            enabled: true,
            per_ip_rps: rps,
            burst,
        }
    }

    #[test]
    fn zero_rps_returns_config_error() {
        let err = RateLimitLayer::new(&cfg(0, 10)).unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn zero_burst_returns_config_error() {
        let err = RateLimitLayer::new(&cfg(10, 0)).unwrap_err();
        assert_eq!(err.code(), "config_error");
    }

    #[test]
    fn valid_config_builds_layer() {
        assert!(RateLimitLayer::new(&cfg(100, 200)).is_ok());
    }

    #[test]
    fn limiter_allows_then_blocks_per_key() {
        let layer = RateLimitLayer::new(&cfg(1, 2)).unwrap();
        let limiter = layer.limiter();
        let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        // Burst de 2: las dos primeras pasan, la tercera rebota.
        assert!(limiter.check_key(&ip).is_ok());
        assert!(limiter.check_key(&ip).is_ok());
        assert!(limiter.check_key(&ip).is_err());
    }

    #[test]
    fn limiter_isolates_ips() {
        let layer = RateLimitLayer::new(&cfg(1, 1)).unwrap();
        let limiter = layer.limiter();
        let ip_a: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ip_b: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert!(limiter.check_key(&ip_a).is_ok());
        // ip_a agotada, ip_b sigue con cupo.
        assert!(limiter.check_key(&ip_a).is_err());
        assert!(limiter.check_key(&ip_b).is_ok());
    }

    #[test]
    fn rejects_a_well_known_direct_limiter_too() {
        // Sanity: comportamiento basico de governor sin nuestro wrapper.
        let quota =
            Quota::per_second(NonZeroU32::new(1).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
        let limiter: DirectLimiter = RateLimiter::direct(quota);
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_err());
    }
}
