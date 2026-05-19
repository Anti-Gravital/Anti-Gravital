//! Capa CORS de Anti-Gravital.
//!
//! Envuelve `tower_http::cors::CorsLayer` con defaults seguros y una
//! configuracion declarativa desde `ShieldConfig`. Por defecto la capa
//! se construye en modo restrictivo: sin origenes permitidos. Para
//! habilitar CORS hay que declarar `[shield.cors]` con `enabled = true`
//! y al menos un origen.

use axum::http::header::{HeaderName, HeaderValue};
use axum::http::Method;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::CorsConfig;
use crate::error::AgError;

/// Construye una `CorsLayer` desde la configuracion.
///
/// # Errores
///
/// Devuelve `AgError::Cors` si la configuracion contiene origenes,
/// metodos o headers invalidos.
pub fn build_layer(config: &CorsConfig) -> Result<CorsLayer, AgError> {
    let mut layer = CorsLayer::new();

    if !config.enabled {
        return Ok(layer);
    }

    let origins = config
        .allow_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).map_err(|e| AgError::Cors(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    layer = layer.allow_origin(AllowOrigin::list(origins));

    let methods = config
        .allow_methods
        .iter()
        .map(|m| {
            m.parse::<Method>()
                .map_err(|e| AgError::Cors(e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    layer = layer.allow_methods(methods);

    let headers = config
        .allow_headers
        .iter()
        .map(|h| HeaderName::from_bytes(h.as_bytes()).map_err(|e| AgError::Cors(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    layer = layer.allow_headers(headers);

    layer = layer.allow_credentials(config.allow_credentials);

    Ok(layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_cors_builds_empty_layer() {
        let cfg = CorsConfig::default();
        let layer = build_layer(&cfg);
        assert!(layer.is_ok());
    }

    #[test]
    fn enabled_cors_with_valid_origins_builds() {
        let cfg = CorsConfig {
            enabled: true,
            allow_origins: vec!["https://example.com".into()],
            allow_methods: vec!["GET".into(), "POST".into()],
            allow_headers: vec!["content-type".into()],
            allow_credentials: false,
        };
        assert!(build_layer(&cfg).is_ok());
    }

    #[test]
    fn enabled_cors_with_invalid_method_errors() {
        let cfg = CorsConfig {
            enabled: true,
            allow_origins: vec!["https://example.com".into()],
            allow_methods: vec!["NOT A METHOD".into()],
            allow_headers: vec![],
            allow_credentials: false,
        };
        let err = build_layer(&cfg).unwrap_err();
        assert_eq!(err.code(), "cors_error");
    }

    #[test]
    fn enabled_cors_with_invalid_header_errors() {
        let cfg = CorsConfig {
            enabled: true,
            allow_origins: vec!["https://example.com".into()],
            allow_methods: vec!["GET".into()],
            allow_headers: vec!["invalid header name with spaces".into()],
            allow_credentials: false,
        };
        let err = build_layer(&cfg).unwrap_err();
        assert_eq!(err.code(), "cors_error");
    }
}
