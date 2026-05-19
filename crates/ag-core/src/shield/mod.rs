//! Capa Shield: pipeline Tower de middleware que protege el Core.
//!
//! En el bootstrap de Fase 1 (este PR) la pipeline solo aplica la
//! capa de logging estructurado. Las capas restantes (TLS, JWT, rate
//! limiting, validacion, CORS, CSRF) se anaden capa por capa en PRs
//! posteriores. Vease `docs/rfc/RFC-0002-diseno-shield-mvp.md`.

use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;

use crate::config::ShieldConfig;

mod logging;
#[cfg(feature = "validation")]
pub mod validation;

#[cfg(feature = "validation")]
pub use validation::{FieldError, Validate, ValidatedJson, ValidationErrors};

/// Pipeline Shield configurable.
#[derive(Debug, Clone)]
pub struct Shield {
    config: ShieldConfig,
}

impl Shield {
    /// Construye un Shield desde la configuracion declarativa.
    #[must_use]
    pub fn new(config: ShieldConfig) -> Self {
        Self { config }
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

    /// Devuelve la capa Tower que se compone con un router Axum.
    ///
    /// La capa actual incluye unicamente logging estructurado. Las
    /// siguientes capas se agregaran en orden definido por RFC-0002.
    #[must_use]
    pub fn layer(&self) -> Stack<logging::LoggingLayer, Identity> {
        ServiceBuilder::new()
            .layer(logging::LoggingLayer)
            .into_inner()
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
    fn shield_layer_is_constructible() {
        let shield = Shield::with_defaults();
        let _layer = shield.layer();
    }
}
