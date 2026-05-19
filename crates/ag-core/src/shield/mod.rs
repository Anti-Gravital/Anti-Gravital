//! Capa Shield: pipeline Tower de middleware que protege el Core.
//!
//! En Fase 1 la pipeline crece capa por capa segun el orden de
//! `docs/rfc/RFC-0002-diseno-shield-mvp.md`. La API publica es
//! `Shield::apply(router)` que recibe un `axum::Router` y devuelve el
//! mismo router con todas las capas configuradas activas.

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

/// Pipeline Shield configurable.
///
/// La construccion valida la configuracion (por ejemplo, formato de
/// origenes CORS). Si la configuracion es invalida, `try_new` devuelve
/// el error correspondiente y `new` entra en panic. Por convencion el
/// arranque del proceso usa `try_new` para fallar rapido y limpio.
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
    /// Construye un Shield validando la configuracion declarativa.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError` si alguna seccion de la configuracion es
    /// invalida en el momento del arranque.
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

    /// Sirve la aplicacion sobre el listener dado. Si TLS esta
    /// activado en la configuracion, envuelve cada conexion con
    /// rustls; en caso contrario delega en `axum::serve`.
    ///
    /// # Errores
    ///
    /// Propaga errores de I/O y de TLS. Las desconexiones individuales
    /// se logean y no detienen el accept loop.
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

    /// Variante de `serve` cuando la feature `tls` no esta activa.
    #[cfg(not(feature = "tls"))]
    pub async fn serve(&self, listener: tokio::net::TcpListener, router: Router) -> AgResult<()> {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .map_err(|e| crate::error::AgError::Other(format!("axum::serve failed: {e}")))
    }

    /// Construye un Shield sin verificar la configuracion.
    ///
    /// # Panicos
    ///
    /// Entra en panic si la configuracion es invalida. Use `try_new`
    /// si necesita manejar el error de forma estructurada.
    #[must_use]
    pub fn new(config: ShieldConfig) -> Self {
        Self::try_new(config).expect("invalid shield configuration")
    }

    /// Construye un Shield con la configuracion por defecto.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ShieldConfig::default())
    }

    /// Devuelve la configuracion en uso.
    #[must_use]
    pub fn config(&self) -> &ShieldConfig {
        &self.config
    }

    /// Aplica todas las capas configuradas al router.
    ///
    /// El orden de aplicacion es relevante: la primera capa en
    /// agregarse es la mas interna; la ultima es la mas externa, es
    /// decir la primera en ver el request entrante. La capa de
    /// logging es la mas externa para que todo request quede
    /// trazado, incluso si es rechazado por otra capa.
    pub fn apply(&self, router: Router) -> Router {
        let mut router = router;

        // Orden de adicion (de mas interno a mas externo): la primera
        // capa anadida envuelve al handler; la ultima ve el request
        // primero. Por seguridad, rate-limit y auth vienen antes que
        // las capas de proteccion semantica (CORS, CSRF), y logging
        // queda al borde para trazar absolutamente todo.

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
            // Inyecta ConnectInfo en cada request para que rate-limit
            // y cualquier otra capa que requiera la IP del cliente
            // funcione tambien sobre TLS. peer_addr viene del
            // TcpStream original aceptado antes del handshake.
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
