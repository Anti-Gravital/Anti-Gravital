//! Shield layer: Tower middleware pipeline that protects the Core.
//!
//! In Phase 1 the pipeline grows layer by layer following the order in
//! `docs/rfc/RFC-0002-diseno-shield-mvp.md`. The public API is
//! `Shield::apply(router)`, which takes an `axum::Router` and returns the
//! same router with all configured layers active.

use axum::Router;

use crate::config::ShieldConfig;
use crate::error::AgResult;

#[cfg(feature = "auth-jwt")]
pub mod auth;
#[cfg(feature = "cors")]
mod cors;
#[cfg(feature = "csrf")]
mod csrf;
mod logging;
#[cfg(feature = "rate-limit")]
mod rate_limit;
#[cfg(feature = "tls")]
mod tls;
#[cfg(feature = "validation")]
pub mod validation;

#[cfg(feature = "auth-jwt")]
pub use auth::{AuthContext, Claims};

#[cfg(feature = "validation")]
pub use validation::{FieldError, Validate, ValidatedJson, ValidationErrors};

/// Configurable Shield pipeline.
///
/// Construction validates the configuration (for example, the format of
/// CORS origins). If the configuration is invalid, `try_new` returns the
/// corresponding error and `new` panics. By convention, process startup
/// uses `try_new` to fail fast and cleanly.
#[derive(Clone)]
pub struct Shield {
    config: ShieldConfig,
    #[cfg(feature = "cors")]
    cors_layer: Option<tower_http::cors::CorsLayer>,
    #[cfg(feature = "rate-limit")]
    rate_limit_layer: Option<rate_limit::RateLimitLayer>,
    #[cfg(feature = "auth-jwt")]
    auth_layer: Option<auth::AuthLayer>,
    #[cfg(feature = "tls")]
    tls_acceptor: Option<tokio_rustls::TlsAcceptor>,
}

impl std::fmt::Debug for Shield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shield")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Shield {
    /// Builds a Shield, validating the declarative configuration.
    ///
    /// # Errors
    ///
    /// Returns `AgError` if any section of the configuration is invalid
    /// at startup time.
    pub fn try_new(config: ShieldConfig) -> AgResult<Self> {
        #[cfg(feature = "cors")]
        let cors_layer = if config.cors.enabled {
            Some(cors::build_layer(&config.cors)?)
        } else {
            None
        };

        #[cfg(feature = "rate-limit")]
        let rate_limit_layer = if config.rate_limit.enabled {
            Some(rate_limit::RateLimitLayer::new(&config.rate_limit)?)
        } else {
            None
        };

        #[cfg(feature = "auth-jwt")]
        let auth_layer = if config.auth.enabled {
            Some(auth::AuthLayer::new(&config.auth)?)
        } else {
            None
        };

        #[cfg(feature = "tls")]
        let tls_acceptor = if config.tls.enabled {
            Some(tls::build_acceptor(&config.tls)?)
        } else {
            None
        };

        Ok(Self {
            config,
            #[cfg(feature = "cors")]
            cors_layer,
            #[cfg(feature = "rate-limit")]
            rate_limit_layer,
            #[cfg(feature = "auth-jwt")]
            auth_layer,
            #[cfg(feature = "tls")]
            tls_acceptor,
        })
    }

    /// Serves the application over the given listener. If TLS is enabled
    /// in the configuration, it wraps each connection with rustls;
    /// otherwise it delegates to `axum::serve`.
    ///
    /// # Errors
    ///
    /// Propagates I/O and TLS errors. Individual disconnections are
    /// logged and do not stop the accept loop.
    #[cfg(feature = "tls")]
    pub async fn serve(&self, listener: tokio::net::TcpListener, router: Router) -> AgResult<()> {
        if let Some(acceptor) = self.tls_acceptor.clone() {
            serve_tls(listener, acceptor, router).await
        } else {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .map_err(|e| crate::error::AgError::Other(format!("axum::serve failed: {e}")))
        }
    }

    /// Variant of `serve` when the `tls` feature is not active.
    #[cfg(not(feature = "tls"))]
    pub async fn serve(&self, listener: tokio::net::TcpListener, router: Router) -> AgResult<()> {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .map_err(|e| crate::error::AgError::Other(format!("axum::serve failed: {e}")))
    }

    /// Builds a Shield without verifying the configuration.
    ///
    /// # Panics
    ///
    /// Panics if the configuration is invalid. Use `try_new` if you need
    /// to handle the error in a structured way.
    #[must_use]
    pub fn new(config: ShieldConfig) -> Self {
        Self::try_new(config).expect("invalid shield configuration")
    }

    /// Builds a Shield with the default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ShieldConfig::default())
    }

    /// Returns the configuration in use.
    #[must_use]
    pub fn config(&self) -> &ShieldConfig {
        &self.config
    }

    /// Applies all configured layers to the router.
    ///
    /// The application order matters: the first layer added is the
    /// innermost; the last is the outermost, that is, the first to see
    /// the incoming request. The logging layer is the outermost so that
    /// every request is traced, even if it is rejected by another layer.
    pub fn apply(&self, router: Router) -> Router {
        let mut router = router;

        // Addition order (from innermost to outermost): the first layer
        // added wraps the handler; the last one sees the request first.
        // For security, rate-limit and auth come before the semantic
        // protection layers (CORS, CSRF), and logging sits at the edge
        // to trace absolutely everything.

        #[cfg(feature = "csrf")]
        if self.config.csrf.enabled {
            router = router.layer(csrf::CsrfLayer::new(self.config.csrf.clone()));
        }

        #[cfg(feature = "auth-jwt")]
        if let Some(layer) = self.auth_layer.clone() {
            router = router.layer(layer);
        }

        #[cfg(feature = "cors")]
        if let Some(cors) = self.cors_layer.clone() {
            router = router.layer(cors);
        }

        #[cfg(feature = "rate-limit")]
        if let Some(rl) = self.rate_limit_layer.clone() {
            router = router.layer(rl);
        }

        router = router.layer(logging::LoggingLayer);
        router
    }
}

#[cfg(feature = "tls")]
async fn serve_tls(
    listener: tokio::net::TcpListener,
    acceptor: tokio_rustls::TlsAcceptor,
    router: Router,
) -> AgResult<()> {
    use axum::extract::connect_info::ConnectInfo;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder;
    use tower::Service;

    let make_service = router.into_make_service();
    loop {
        let (stream, peer_addr) = listener
            .accept()
            .await
            .map_err(crate::error::AgError::from)?;
        let acceptor = acceptor.clone();
        let mut make = make_service.clone();
        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(err) => {
                    tracing::warn!(error = %err, "tls handshake failed");
                    return;
                }
            };

            let service = match make.call(()).await {
                Ok(s) => s,
                Err(err) => {
                    tracing::error!(error = %err, "service factory failed");
                    return;
                }
            };
            // Inject ConnectInfo into every request so that rate-limit
            // and any other layer that needs the client IP also works
            // over TLS. peer_addr comes from the original TcpStream
            // accepted before the handshake.
            let hyper_service = hyper::service::service_fn(
                move |mut req: hyper::Request<hyper::body::Incoming>| {
                    req.extensions_mut().insert(ConnectInfo(peer_addr));
                    let mut service = service.clone();
                    async move { service.call(req).await }
                },
            );

            let io = TokioIo::new(tls_stream);
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, hyper_service)
                .await
            {
                tracing::warn!(error = %err, "connection serving failed");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_defaults_uses_default_config() {
        let shield = Shield::with_defaults();
        assert_eq!(shield.config().bind.port(), 8080);
    }

    #[test]
    fn shield_apply_is_callable() {
        let shield = Shield::with_defaults();
        let router = Router::<()>::new();
        let _routed = shield.apply(router);
    }

    #[cfg(feature = "cors")]
    #[test]
    fn try_new_with_invalid_cors_fails() {
        use crate::config::CorsConfig;
        let cfg = ShieldConfig {
            cors: CorsConfig {
                enabled: true,
                allow_origins: vec!["https://example.com".into()],
                allow_methods: vec!["NOT A METHOD".into()],
                allow_headers: vec![],
                allow_credentials: false,
            },
            ..ShieldConfig::default()
        };
        let err = Shield::try_new(cfg).unwrap_err();
        assert_eq!(err.code(), "cors_error");
    }

    #[cfg(feature = "cors")]
    #[test]
    fn try_new_with_disabled_cors_succeeds() {
        let shield = Shield::try_new(ShieldConfig::default()).unwrap();
        assert!(!shield.config().cors.enabled);
    }
}
