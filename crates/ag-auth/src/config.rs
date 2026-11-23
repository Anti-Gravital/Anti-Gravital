//! Authentication configuration read from declared environment variables.

use ag_core::{EnvDeclaration, EnvLoader, EnvType, LoadedEnv};

/// Configuration of the authentication module.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Ed25519 private key in PEM format for signing JWTs.
    /// Variable: `JWT_PRIVATE_KEY`
    pub jwt_private_key_pem: String,
    /// Ed25519 public key in PEM format for verifying JWTs.
    /// Variable: `JWT_PUBLIC_KEY`
    pub jwt_public_key_pem: String,
    /// Relying party identifier for WebAuthn (e.g.: `"example.com"`).
    /// Variable: `WEBAUTHN_RP_ID`
    pub webauthn_rp_id: String,
    /// Origin URL for WebAuthn (e.g.: `"https://example.com"`).
    /// Variable: `WEBAUTHN_ORIGIN`
    pub webauthn_origin: String,
    /// Google OAuth2 client ID. `None` disables the provider.
    pub oauth_google_client_id: Option<String>,
    /// Google OAuth2 client secret.
    pub oauth_google_client_secret: Option<String>,
    /// GitHub OAuth2 client ID. `None` disables the provider.
    pub oauth_github_client_id: Option<String>,
    /// GitHub OAuth2 client secret.
    pub oauth_github_client_secret: Option<String>,
}

/// Declared environment schema for `ag-auth` (RFC-0020 PR 4, audit A2).
///
/// Every key the crate reads is listed once here: name, secret classification
/// and, where the raw variable has one, its default. This is the single
/// auditable answer to "which variables does this service need".
///
/// Four keys are secrets and are declared `secret(true)`:
/// `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEY`, `GOOGLE_CLIENT_SECRET` and
/// `GITHUB_CLIENT_SECRET`. The loader delivers them as `Secret<String>`, whose
/// `Debug`/`Display` redact and which require an explicit `expose()` call; a
/// value is never printed in an error (S-1/S-8). The public `AuthConfig` keeps
/// its original `String`/`Option<String>` fields (no public API change), so the
/// secret bytes are copied out only in `from_loaded_env` via `expose()`.
///
/// All keys are declared as [`EnvType::String`] on purpose. The previous
/// `from_env` used `std::env::var(k)` with a *per-field* fallback: an unset,
/// non-UTF-8 or (where parsed) malformed value fell back independently for
/// that field. [`EnvLoader`] validates the whole set at once and rejects the
/// load if a single value fails its declared type, so a typed declaration
/// would silently change that per-field behavior. Parsing therefore stays in
/// [`AuthConfig::from_loaded_env`], mirroring the original expressions byte for
/// byte; the loader still provides the declared list, per-field defaults,
/// secret redaction and process-environment isolation (only declared keys are
/// read). The JWT keys are declared optional with no default so that absence
/// is reported by `from_loaded_env` as the original
/// `AuthConfigError::MissingVar` error.
pub(crate) fn auth_env_declarations() -> Vec<EnvDeclaration> {
    vec![
        EnvDeclaration::new("JWT_PRIVATE_KEY", EnvType::String)
            .required(false)
            .secret(true)
            .description_text("Ed25519 private key (PEM) for signing JWTs"),
        EnvDeclaration::new("JWT_PUBLIC_KEY", EnvType::String)
            .required(false)
            .secret(true)
            .description_text("Ed25519 public key (PEM) for verifying JWTs"),
        EnvDeclaration::new("WEBAUTHN_RP_ID", EnvType::String)
            .required(false)
            .with_default("localhost"),
        EnvDeclaration::new("WEBAUTHN_ORIGIN", EnvType::String)
            .required(false)
            .with_default("http://localhost:8080"),
        EnvDeclaration::new("GOOGLE_CLIENT_ID", EnvType::String).required(false),
        EnvDeclaration::new("GOOGLE_CLIENT_SECRET", EnvType::String)
            .required(false)
            .secret(true),
        EnvDeclaration::new("GITHUB_CLIENT_ID", EnvType::String).required(false),
        EnvDeclaration::new("GITHUB_CLIENT_SECRET", EnvType::String)
            .required(false)
            .secret(true),
    ]
}

/// Reusable loader for the declared authentication schema.
pub(crate) fn auth_loader() -> EnvLoader {
    EnvLoader::new(auth_env_declarations())
}

impl AuthConfig {
    /// Reads the configuration from environment variables.
    ///
    /// Returns an error if the required JWT keys are not defined.
    ///
    /// Production path: resolves the declared schema through
    /// [`EnvLoader::load_process`], which reads only declared keys and never
    /// prints a value. A non-UTF-8 process value is the only possible load
    /// error for this all-string schema; it is handled below by replaying the
    /// declared keys leniently so that field keeps its previous fallback.
    pub fn from_env() -> Result<Self, AuthConfigError> {
        let loader = auth_loader();
        let env = match loader.load_process() {
            Ok(env) => env,
            Err(_) => {
                let vars: Vec<(String, String)> = loader
                    .declarations()
                    .iter()
                    .filter_map(|declaration| {
                        std::env::var(declaration.name())
                            .ok()
                            .map(|value| (declaration.name().to_owned(), value))
                    })
                    .collect();
                loader
                    .load_from(vars)
                    .expect("all declarations are optional strings and always resolve")
            }
        };
        Self::from_loaded_env(&env)
    }

    /// Test seam: resolves the same declared schema from an injected in-memory
    /// source instead of the process environment. Using this in tests removes
    /// the `set_var`/`remove_var` global races (audit A1) without a global
    /// mutex. Scoped to tests so the public API is unchanged.
    #[cfg(test)]
    pub(crate) fn from_env_source<I, K, V>(vars: I) -> Result<Self, AuthConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env = auth_loader()
            .load_from(vars)
            .expect("all declarations are optional strings and always resolve");
        Self::from_loaded_env(&env)
    }

    /// Builds a config from a resolved environment. Parsing, defaults and
    /// errors are identical to the pre-migration direct reads, field by field.
    fn from_loaded_env(env: &LoadedEnv) -> Result<Self, AuthConfigError> {
        let jwt_private_key_pem = env
            .secret("JWT_PRIVATE_KEY")
            .map(|value| value.expose().to_owned())
            .ok_or(AuthConfigError::MissingVar("JWT_PRIVATE_KEY"))?;
        let jwt_public_key_pem = env
            .secret("JWT_PUBLIC_KEY")
            .map(|value| value.expose().to_owned())
            .ok_or(AuthConfigError::MissingVar("JWT_PUBLIC_KEY"))?;
        let webauthn_rp_id = env
            .string("WEBAUTHN_RP_ID")
            .map(str::to_owned)
            .unwrap_or_else(|| "localhost".to_string());
        let webauthn_origin = env
            .string("WEBAUTHN_ORIGIN")
            .map(str::to_owned)
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        Ok(Self {
            jwt_private_key_pem,
            jwt_public_key_pem,
            webauthn_rp_id,
            webauthn_origin,
            oauth_google_client_id: env.string("GOOGLE_CLIENT_ID").map(str::to_owned),
            oauth_google_client_secret: env
                .secret("GOOGLE_CLIENT_SECRET")
                .map(|value| value.expose().to_owned()),
            oauth_github_client_id: env.string("GITHUB_CLIENT_ID").map(str::to_owned),
            oauth_github_client_secret: env
                .secret("GITHUB_CLIENT_SECRET")
                .map(|value| value.expose().to_owned()),
        })
    }
}

/// Authentication configuration error.
#[derive(Debug)]
#[non_exhaustive]
pub enum AuthConfigError {
    /// Required environment variable not defined.
    MissingVar(&'static str),
}

impl std::fmt::Display for AuthConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthConfigError::MissingVar(v) => {
                write!(f, "required environment variable not set: {v}")
            }
        }
    }
}

impl std::error::Error for AuthConfigError {}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
