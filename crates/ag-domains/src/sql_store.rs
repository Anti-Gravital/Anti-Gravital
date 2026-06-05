//! SQL-backed attachment store (Postgres), RFC-0009 phase D.
//!
//! Optional, behind the `sql-store` feature. The native in-memory / JSON store
//! remains the default (ADR-0009): this is a multi-node accelerator, never a
//! requirement. The attachment is stored as JSONB plus a few indexed columns
//! (id, hostname, lifecycle) so queries and the takeover-tombstone logic stay
//! first-class while the row stays faithful to [`DomainAttachment`].
//!
//! Runtime queries (`sqlx::query`) are used rather than the compile-time macros
//! so the crate builds without a live `DATABASE_URL`. Integration tests are
//! `#[ignore]` and require a Postgres instance, matching the repo convention
//! for service-backed tests.

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::attachment::DomainAttachment;
use crate::store::{now_unix, AttachmentStore, StoreError};

/// Embedded schema, applied by [`SqlAttachmentStore::migrate`].
pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS domain_attachment (
    id             TEXT PRIMARY KEY,
    hostname_ascii TEXT NOT NULL UNIQUE,
    lifecycle      TEXT NOT NULL,
    data           JSONB NOT NULL,
    created_at     BIGINT NOT NULL
);
CREATE TABLE IF NOT EXISTS domain_tombstone (
    hostname_ascii     TEXT PRIMARY KEY,
    last_attachment_id TEXT NOT NULL,
    tombstone_until    BIGINT NOT NULL,
    reason             TEXT NOT NULL
);
"#;

fn backend(e: impl std::fmt::Display) -> StoreError {
    StoreError::Backend(e.to_string())
}

fn lifecycle_str(a: &DomainAttachment) -> String {
    serde_json::to_value(a.lifecycle)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Postgres-backed attachment store.
#[derive(Debug, Clone)]
pub struct SqlAttachmentStore {
    pool: PgPool,
}

impl SqlAttachmentStore {
    /// Wraps an existing pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connects to `database_url` and returns a store.
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(backend)?;
        Ok(Self { pool })
    }

    /// Creates the tables if they do not exist.
    pub async fn migrate(&self) -> Result<(), StoreError> {
        sqlx::query(SCHEMA)
            .execute(&self.pool)
            .await
            .map_err(backend)?;
        Ok(())
    }

    async fn tombstone_until(&self, hostname_ascii: &str) -> Result<Option<i64>, StoreError> {
        let row =
            sqlx::query("SELECT tombstone_until FROM domain_tombstone WHERE hostname_ascii = $1")
                .bind(hostname_ascii)
                .fetch_optional(&self.pool)
                .await
                .map_err(backend)?;
        Ok(row.map(|r| r.get::<i64, _>("tombstone_until")))
    }
}

#[async_trait]
impl AttachmentStore for SqlAttachmentStore {
    async fn create(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        let key = attachment.hostname.ascii().to_owned();

        if let Some(until) = self.tombstone_until(&key).await? {
            if until as u64 > now_unix() {
                return Err(StoreError::Tombstoned {
                    hostname: key,
                    until: until as u64,
                });
            }
        }

        let result = sqlx::query(
            "INSERT INTO domain_attachment (id, hostname_ascii, lifecycle, data, created_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&attachment.id)
        .bind(&key)
        .bind(lifecycle_str(&attachment))
        .bind(sqlx::types::Json(attachment.clone()))
        .bind(attachment.created_at as i64)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                Err(StoreError::AlreadyExists(key))
            }
            Err(e) => Err(backend(e)),
        }
    }

    async fn get(&self, hostname_ascii: &str) -> Result<Option<DomainAttachment>, StoreError> {
        let row = sqlx::query("SELECT data FROM domain_attachment WHERE hostname_ascii = $1")
            .bind(hostname_ascii)
            .fetch_optional(&self.pool)
            .await
            .map_err(backend)?;
        Ok(row.map(|r| r.get::<sqlx::types::Json<DomainAttachment>, _>("data").0))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<DomainAttachment>, StoreError> {
        let row = sqlx::query("SELECT data FROM domain_attachment WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(backend)?;
        Ok(row.map(|r| r.get::<sqlx::types::Json<DomainAttachment>, _>("data").0))
    }

    async fn update(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        let key = attachment.hostname.ascii().to_owned();
        let result = sqlx::query(
            "UPDATE domain_attachment SET lifecycle = $2, data = $3 WHERE hostname_ascii = $1",
        )
        .bind(&key)
        .bind(lifecycle_str(&attachment))
        .bind(sqlx::types::Json(attachment.clone()))
        .execute(&self.pool)
        .await
        .map_err(backend)?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound(key));
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<DomainAttachment>, StoreError> {
        let rows = sqlx::query("SELECT data FROM domain_attachment ORDER BY hostname_ascii")
            .fetch_all(&self.pool)
            .await
            .map_err(backend)?;
        Ok(rows
            .into_iter()
            .map(|r| r.get::<sqlx::types::Json<DomainAttachment>, _>("data").0)
            .collect())
    }

    async fn detach(&self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError> {
        let attachment = self
            .get(hostname_ascii)
            .await?
            .ok_or_else(|| StoreError::NotFound(hostname_ascii.to_owned()))?;

        let mut tx = self.pool.begin().await.map_err(backend)?;

        sqlx::query("DELETE FROM domain_attachment WHERE hostname_ascii = $1")
            .bind(hostname_ascii)
            .execute(&mut *tx)
            .await
            .map_err(backend)?;

        sqlx::query(
            "INSERT INTO domain_tombstone (hostname_ascii, last_attachment_id, tombstone_until, reason) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (hostname_ascii) DO UPDATE SET \
             last_attachment_id = EXCLUDED.last_attachment_id, \
             tombstone_until = EXCLUDED.tombstone_until, reason = EXCLUDED.reason",
        )
        .bind(hostname_ascii)
        .bind(&attachment.id)
        .bind((now_unix() + tombstone_secs) as i64)
        .bind("detached by operator")
        .execute(&mut *tx)
        .await
        .map_err(backend)?;

        tx.commit().await.map_err(backend)?;
        Ok(())
    }

    async fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> Result<bool, StoreError> {
        Ok(self
            .tombstone_until(hostname_ascii)
            .await?
            .map(|u| u as u64 > now)
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment::DomainAttachment;
    use crate::hostname::Hostname;

    #[test]
    fn schema_mentions_both_tables() {
        assert!(SCHEMA.contains("domain_attachment"));
        assert!(SCHEMA.contains("domain_tombstone"));
    }

    #[test]
    fn lifecycle_str_is_snake_case() {
        let a = DomainAttachment {
            id: "dom_x".to_owned(),
            hostname: Hostname::parse("example.com").unwrap(),
            project: "p".to_owned(),
            environment: "production".to_owned(),
            target_kind: crate::attachment::TargetKind::Service,
            target_ref: "s".to_owned(),
            dns_mode: crate::attachment::DnsMode::Manual,
            tls_mode: crate::attachment::TlsMode::ManagedHttp01,
            ownership_method: crate::attachment::OwnershipMethod::Txt,
            ownership_name: "_ag-domain.example.com".to_owned(),
            ownership_value: "ag-verification=x".to_owned(),
            ownership_status: crate::attachment::OwnershipStatus::Pending,
            dns_status: crate::attachment::DnsStatus::Pending,
            tls_status: crate::attachment::TlsStatus::Pending,
            routing_status: crate::attachment::RoutingStatus::Disabled,
            lifecycle: crate::attachment::AttachmentLifecycle::PendingOwnership,
            created_at: 0,
        };
        assert_eq!(lifecycle_str(&a), "pending_ownership");
    }
}
