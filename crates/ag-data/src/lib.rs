//! Capa de datos de Anti-Gravital.
//!
//! Provee el pool de conexiones PostgreSQL, la configuracion declarativa
//! y el helper para ejecutar migraciones embebidas. En Fase 3+ el DSL
//! genera query builders tipados que usan este pool como backend.
//!
//! # Ejemplo minimo
//!
//! ```no_run
//! use ag_data::{DataConfig, connect};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = DataConfig {
//!     url: "postgresql://localhost/mi_app".into(),
//!     ..DataConfig::default()
//! };
//!
//! let pool = connect(&config).await?;
//! # let _ = pool;
//! # Ok(())
//! # }
//! ```
//!
//! # Migraciones
//!
//! Las migraciones se embeben en el binario con el macro `sqlx::migrate!`
//! y se aplican en el arranque con [`run_migrations`]:
//!
//! ```ignore
//! use ag_data::{DataConfig, DbPool, connect, run_migrations};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool = connect(&DataConfig::default()).await?;
//! // El macro lee los archivos SQL del crate consumidor, no de ag-data.
//! // La ruta es relativa al Cargo.toml del proyecto que llama a migrate!.
//! static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
//! run_migrations(&pool, &MIGRATOR).await?;
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

/// Pool de conexiones PostgreSQL compartido entre handlers.
///
/// Es un alias de [`sqlx::PgPool`], que ya implementa [`Clone`] con
/// costo O(1) (comparte el pool interno por `Arc`).
pub type DbPool = sqlx::PgPool;

/// Error de la capa de datos.
#[derive(Debug, Error)]
pub enum DataError {
    /// Error originado en sqlx (conexion, query, protocolo).
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Error durante la ejecucion de migraciones.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

impl From<DataError> for ag_core::AgError {
    fn from(err: DataError) -> Self {
        ag_core::AgError::Database(err.to_string())
    }
}

/// Configuracion del pool de conexiones.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DataConfig {
    /// URL de conexion PostgreSQL.
    ///
    /// Formato: `postgresql://usuario:contrasena@host:puerto/basedatos`
    pub url: String,

    /// Maximo de conexiones simultaneas en el pool.
    #[serde(default = "DataConfig::default_max_connections")]
    pub max_connections: u32,

    /// Segundos maximos para adquirir una conexion del pool.
    #[serde(default = "DataConfig::default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
}

impl DataConfig {
    fn default_max_connections() -> u32 {
        10
    }
    fn default_acquire_timeout_secs() -> u64 {
        30
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/app".into(),
            max_connections: Self::default_max_connections(),
            acquire_timeout_secs: Self::default_acquire_timeout_secs(),
        }
    }
}

/// Conecta a PostgreSQL y devuelve un pool listo para usar.
///
/// Aplica la configuracion del pool declarada en [`DataConfig`].
/// El pool se inicializa de forma lazy: las conexiones reales se
/// abren cuando se necesitan, hasta el maximo declarado.
///
/// # Errores
///
/// Devuelve [`DataError::Sqlx`] si la URL es invalida o si no se
/// puede establecer la conexion inicial de prueba.
pub async fn connect(config: &DataConfig) -> Result<DbPool, DataError> {
    tracing::debug!(url = %sanitize_url(&config.url), "conectando al pool PostgreSQL");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .connect(&config.url)
        .await?;

    tracing::info!(
        max_connections = config.max_connections,
        "pool PostgreSQL listo"
    );
    Ok(pool)
}

/// Ejecuta las migraciones embebidas contra el pool.
///
/// El `migrator` se construye con el macro `sqlx::migrate!("./migrations")`
/// en el crate que contiene los archivos SQL. Este helper aplica todas
/// las migraciones pendientes en orden y es idempotente.
///
/// # Errores
///
/// Devuelve [`DataError::Migration`] si una migracion falla.
pub async fn run_migrations(
    pool: &DbPool,
    migrator: &sqlx::migrate::Migrator,
) -> Result<(), DataError> {
    tracing::info!("aplicando migraciones pendientes");
    migrator.run(pool).await?;
    tracing::info!("migraciones completadas");
    Ok(())
}

/// Elimina credenciales de la URL para logging seguro.
fn sanitize_url(url: &str) -> String {
    // Oculta usuario:contrasena@ para no filtrar credenciales en logs.
    if let Some(at) = url.find('@') {
        if let Some(proto) = url.find("://") {
            return format!("{}://<redacted>@{}", &url[..proto], &url[at + 1..]);
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_config_default_is_sensible() {
        let cfg = DataConfig::default();
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.acquire_timeout_secs, 30);
        assert!(cfg.url.starts_with("postgresql://"));
    }

    #[test]
    fn sanitize_url_removes_credentials() {
        let url = "postgresql://user:secret@localhost:5432/db";
        let safe = sanitize_url(url);
        assert!(!safe.contains("secret"));
        assert!(safe.contains("localhost:5432/db"));
    }

    #[test]
    fn sanitize_url_without_credentials_is_unchanged() {
        let url = "postgresql://localhost/db";
        let safe = sanitize_url(url);
        assert_eq!(safe, url);
    }

    #[test]
    fn data_error_converts_to_ag_error() {
        use ag_core::AgError;
        let data_err = DataError::Sqlx(sqlx::Error::RowNotFound);
        let ag_err = AgError::from(data_err);
        assert_eq!(ag_err.code(), "database_error");
    }

    #[tokio::test]
    #[ignore = "requiere DATABASE_URL con PostgreSQL accesible"]
    async fn connect_to_real_db() {
        let url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL debe estar definida para este test");
        let config = DataConfig {
            url,
            ..DataConfig::default()
        };
        let pool = connect(&config).await.expect("conexion fallida");
        let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }
}
