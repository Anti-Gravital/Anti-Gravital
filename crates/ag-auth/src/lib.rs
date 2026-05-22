//! Autenticacion y autorizacion para el ecosistema Anti-Gravital.
//!
//! Soporta JWT Ed25519, Passkeys/WebAuthn, OAuth2 (Google, GitHub),
//! API keys con hash BLAKE3 y refresh tokens con rotacion.
//!
//! # Uso minimo (JWT + API keys)
//!
//! ```no_run
//! use ag_auth::{AgAuth, AuthConfig};
//!
//! # fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AuthConfig::from_env()?;
//! let auth = AgAuth::new(config)?;
//!
//! // API keys
//! let (raw_key, key_hash) = auth.create_api_key("sk");
//! let valid = auth.verify_api_key(&raw_key, &key_hash);
//! assert!(valid);
//! # Ok(())
//! # }
//! ```

pub mod api_keys;
pub mod config;
pub mod jwt;

// TECH-DEBT:
// motivo: WebAuthn (FIDO2) requiere integracion con webauthn-rs que necesita
//         estado de ceremonia serializable y tests con mocks de autenticador.
//         La complejidad supera el alcance de esta iteracion.
// impacto: Passkeys no disponibles hasta que se implemente este modulo.
// eliminacion esperada: rama fase-4/ag-auth segunda iteracion.
// issue: https://github.com/anti-gravital/anti-gravital/issues/TBD

// TECH-DEBT:
// motivo: OAuth2 (Google, GitHub) requiere intercambio de codigos HTTP y
//         fetch de user info — necesita un cliente HTTP (reqwest) y
//         credenciales de prueba para tests. Se pospone para siguiente iteracion.
// impacto: Login social no disponible hasta que se implemente este modulo.
// eliminacion esperada: rama fase-4/ag-auth segunda iteracion.
// issue: https://github.com/anti-gravital/anti-gravital/issues/TBD

// TECH-DEBT:
// motivo: Refresh tokens con rotacion (SessionStore) requiere testcontainers
//         PostgreSQL y la feature persistent activada. Se pospone para que
//         no bloquee el path critico JWT + API keys.
// impacto: Solo JWT stateless disponible hasta que se implemente SessionStore.
// eliminacion esperada: rama fase-4/ag-auth segunda iteracion (Task 7 del plan).
// issue: https://github.com/anti-gravital/anti-gravital/issues/TBD

pub use api_keys::{generate as generate_api_key, verify as verify_api_key};
pub use config::{AuthConfig, AuthConfigError};
pub use jwt::{Claims, JwtError, JwtSigner};

/// Fachada principal del modulo de autenticacion.
pub struct AgAuth {
    /// Firmador/verificador de JWTs.
    pub jwt: JwtSigner,
}

impl AgAuth {
    /// Crea una nueva instancia de `AgAuth` con la configuracion dada.
    pub fn new(config: AuthConfig) -> Result<Self, AuthConfigError> {
        let jwt = JwtSigner::new(config.jwt_private_key_pem, config.jwt_public_key_pem);
        Ok(Self { jwt })
    }

    /// Genera una nueva API key y su hash BLAKE3.
    ///
    /// Solo el hash debe almacenarse. La raw key se entrega al usuario una unica vez.
    pub fn create_api_key(&self, prefix: &str) -> (String, String) {
        api_keys::generate(prefix)
    }

    /// Verifica una API key contra su hash almacenado.
    pub fn verify_api_key(&self, raw_key: &str, stored_hash: &str) -> bool {
        api_keys::verify(raw_key, stored_hash)
    }
}
