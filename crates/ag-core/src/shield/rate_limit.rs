//! Per-IP rate limiting layer with governor (token bucket).
//!
//! Keeps an in-memory keyed rate limiter keyed by the client `IpAddr`.
//! When a client exceeds `per_ip_rps` requests per second (with a burst
//! allowance up to `burst`), the additional requests are rejected with
//! `AgError::RateLimit` (status 429).
//!
//! The source IP is determined from the `SocketAddr` reported by Axum
//! through the `ConnectInfo` extractor. When the server lives behind a
//! trusted proxy that sets headers like `X-Forwarded-For`, correct
//! resolution of the real IP is delivered via configuration in a later
//! PR.

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

/// Per-IP rate limiting Tower layer.
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
    /// Builds a layer with the given parameters.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Config` if `per_ip_rps` or `burst` is 0.
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

    /// Access to the internal limiter (internal use and tests).
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

/// Tower service that applies the rate limit.
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
            // Without IP information we do not apply rate limit: in this
            // case the pipeline runs behind a transport that does not
            // expose the IP (tests with TcpListener without ConnectInfo,
            // for example).
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
    /// Future of the rate limit service.
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
        // Burst of 2: the first two pass, the third bounces.
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
        // ip_a exhausted, ip_b still has quota.
        assert!(limiter.check_key(&ip_a).is_err());
        assert!(limiter.check_key(&ip_b).is_ok());
    }

    #[test]
    fn rejects_a_well_known_direct_limiter_too() {
        // Sanity: basic governor behavior without our wrapper.
        let quota =
            Quota::per_second(NonZeroU32::new(1).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
        let limiter: DirectLimiter = RateLimiter::direct(quota);
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_err());
    }
}
