//! Attachment persistence.
//!
//! The store keeps domain attachments and tombstones. Per ADR-0009 the default
//! is native and self-hostable: an in-memory store and a local JSON-file store.
//! A SQL-backed store (`sql-store` feature) is an optional, multi-node option
//! and is never required to attach a domain.
//!
//! The trait is async and uses interior mutability (`&self`), so a single store
//! handle can be shared across async tasks behind an `Arc` without an external
//! mutex. The native stores keep their state behind a `std::sync::Mutex` held
//! only for the duration of a synchronous critical section (never across an
//! await). Tombstones (RFC-0011 section 7) prevent another party from
//! immediately re-claiming a detached hostname (subdomain-takeover prevention).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::attachment::{AttachmentLifecycle, DomainAttachment, RoutingStatus, TlsStatus};

/// Default tombstone duration (30 days), per blueprint section 15.2.
pub const DEFAULT_TOMBSTONE_SECS: u64 = 30 * 24 * 60 * 60;

/// Current Unix epoch seconds.
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A tombstone for a detached hostname.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tombstone {
    /// Canonical hostname identity.
    pub hostname_ascii: String,
    /// Id of the last attachment that owned this hostname.
    pub last_attachment_id: String,
    /// Tombstone expiry (Unix epoch seconds).
    pub tombstone_until: u64,
    /// Reason the hostname was tombstoned.
    pub reason: String,
}

/// Errors produced by an [`AttachmentStore`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
    /// An active attachment already exists for this hostname.
    #[error("hostname already attached: {0}")]
    AlreadyExists(String),
    /// No attachment found.
    #[error("attachment not found: {0}")]
    NotFound(String),
    /// The hostname is tombstoned and cannot be re-claimed yet.
    #[error("hostname is tombstoned until {until} (epoch s): {hostname}")]
    Tombstoned {
        /// Hostname identity.
        hostname: String,
        /// Tombstone expiry.
        until: u64,
    },
    /// Filesystem error (JSON-file store).
    #[error("store I/O error: {0}")]
    Io(String),
    /// (De)serialization error.
    #[error("store serialization error: {0}")]
    Serde(String),
    /// Backend error (SQL store).
    #[error("store backend error: {0}")]
    Backend(String),
}

/// Attachment persistence contract.
///
/// Async and `&self` (interior mutability): a store can be shared as
/// `Arc<dyn AttachmentStore>` across tasks. The native stores execute
/// synchronously; the SQL store awaits its pool.
#[async_trait]
pub trait AttachmentStore: Send + Sync {
    /// Creates a new attachment. Fails if an active attachment exists or the
    /// hostname is tombstoned.
    async fn create(&self, attachment: DomainAttachment) -> Result<(), StoreError>;
    /// Fetches an attachment by canonical hostname.
    async fn get(&self, hostname_ascii: &str) -> Result<Option<DomainAttachment>, StoreError>;
    /// Fetches an attachment by id.
    async fn get_by_id(&self, id: &str) -> Result<Option<DomainAttachment>, StoreError>;
    /// Replaces an existing attachment (matched by hostname).
    async fn update(&self, attachment: DomainAttachment) -> Result<(), StoreError>;
    /// Lists all attachments.
    async fn list(&self) -> Result<Vec<DomainAttachment>, StoreError>;
    /// Detaches a hostname: marks it detached, stops routing/TLS, and writes a
    /// tombstone valid for `tombstone_secs`.
    async fn detach(&self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError>;
    /// Whether a hostname is currently tombstoned.
    async fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> Result<bool, StoreError>;
}

/// Serializable snapshot of attachments and tombstones.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Snapshot {
    attachments: Vec<DomainAttachment>,
    tombstones: Vec<Tombstone>,
}

#[derive(Default)]
struct State {
    attachments: HashMap<String, DomainAttachment>,
    tombstones: HashMap<String, Tombstone>,
}

impl State {
    fn create(&mut self, attachment: DomainAttachment) -> Result<(), StoreError> {
        let key = attachment.hostname.ascii().to_owned();
        if self.attachments.contains_key(&key) {
            return Err(StoreError::AlreadyExists(key));
        }
        if let Some(t) = self.tombstones.get(&key) {
            if t.tombstone_until > now_unix() {
                return Err(StoreError::Tombstoned {
                    hostname: key,
                    until: t.tombstone_until,
                });
            }
        }
        self.attachments.insert(key, attachment);
        Ok(())
    }

    fn update(&mut self, attachment: DomainAttachment) -> Result<(), StoreError> {
        let key = attachment.hostname.ascii().to_owned();
        if !self.attachments.contains_key(&key) {
            return Err(StoreError::NotFound(key));
        }
        self.attachments.insert(key, attachment);
        Ok(())
    }

    fn detach(&mut self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError> {
        let mut attachment = self
            .attachments
            .remove(hostname_ascii)
            .ok_or_else(|| StoreError::NotFound(hostname_ascii.to_owned()))?;

        attachment.routing_status = RoutingStatus::Disabled;
        if attachment.tls_status == TlsStatus::Active {
            attachment.tls_status = TlsStatus::Retired;
        }
        attachment.lifecycle = AttachmentLifecycle::Detached;

        self.tombstones.insert(
            hostname_ascii.to_owned(),
            Tombstone {
                hostname_ascii: hostname_ascii.to_owned(),
                last_attachment_id: attachment.id.clone(),
                tombstone_until: now_unix() + tombstone_secs,
                reason: "detached by operator".to_owned(),
            },
        );
        Ok(())
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            attachments: self.attachments.values().cloned().collect(),
            tombstones: self.tombstones.values().cloned().collect(),
        }
    }
}

/// In-memory attachment store (native default; non-persistent).
#[derive(Default)]
pub struct InMemoryStore {
    state: Mutex<State>,
}

impl std::fmt::Debug for InMemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryStore").finish_non_exhaustive()
    }
}

impl InMemoryStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.expect_lock()
    }
}

// Small helper trait to keep lock-poison handling in one place.
trait ExpectLock<T> {
    fn expect_lock(&self) -> std::sync::MutexGuard<'_, T>;
}
impl<T> ExpectLock<T> for Mutex<T> {
    fn expect_lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().expect("attachment store lock poisoned")
    }
}

#[async_trait]
impl AttachmentStore for InMemoryStore {
    async fn create(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.lock().create(attachment)
    }

    async fn get(&self, hostname_ascii: &str) -> Result<Option<DomainAttachment>, StoreError> {
        Ok(self.lock().attachments.get(hostname_ascii).cloned())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<DomainAttachment>, StoreError> {
        Ok(self
            .lock()
            .attachments
            .values()
            .find(|a| a.id == id)
            .cloned())
    }

    async fn update(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.lock().update(attachment)
    }

    async fn list(&self) -> Result<Vec<DomainAttachment>, StoreError> {
        let mut v: Vec<_> = self.lock().attachments.values().cloned().collect();
        v.sort_by(|a, b| a.hostname.ascii().cmp(b.hostname.ascii()));
        Ok(v)
    }

    async fn detach(&self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError> {
        self.lock().detach(hostname_ascii, tombstone_secs)
    }

    async fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> Result<bool, StoreError> {
        Ok(self
            .lock()
            .tombstones
            .get(hostname_ascii)
            .map(|t| t.tombstone_until > now)
            .unwrap_or(false))
    }
}

/// JSON-file backed store (native, self-hostable, persistent).
///
/// Loads on construction and writes the whole snapshot after each mutation.
/// Suitable for single-node operation and the CLI; the SQL store is the
/// multi-node option.
#[derive(Debug)]
pub struct JsonFileStore {
    path: PathBuf,
    inner: InMemoryStore,
}

impl JsonFileStore {
    /// Opens (or initializes) a JSON-file store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let inner = InMemoryStore::new();
        if path.exists() {
            let bytes = std::fs::read(&path).map_err(|e| StoreError::Io(e.to_string()))?;
            let snapshot: Snapshot =
                serde_json::from_slice(&bytes).map_err(|e| StoreError::Serde(e.to_string()))?;
            let mut state = inner.lock();
            for a in snapshot.attachments {
                state.attachments.insert(a.hostname.ascii().to_owned(), a);
            }
            for t in snapshot.tombstones {
                state.tombstones.insert(t.hostname_ascii.clone(), t);
            }
        }
        Ok(Self { path, inner })
    }

    fn persist(&self) -> Result<(), StoreError> {
        let snapshot = self.inner.lock().snapshot();
        let bytes =
            serde_json::to_vec_pretty(&snapshot).map_err(|e| StoreError::Serde(e.to_string()))?;
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| StoreError::Io(e.to_string()))?;
            }
        }
        std::fs::write(&self.path, bytes).map_err(|e| StoreError::Io(e.to_string()))
    }
}

#[async_trait]
impl AttachmentStore for JsonFileStore {
    async fn create(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.inner.create(attachment).await?;
        self.persist()
    }

    async fn get(&self, hostname_ascii: &str) -> Result<Option<DomainAttachment>, StoreError> {
        self.inner.get(hostname_ascii).await
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<DomainAttachment>, StoreError> {
        self.inner.get_by_id(id).await
    }

    async fn update(&self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.inner.update(attachment).await?;
        self.persist()
    }

    async fn list(&self) -> Result<Vec<DomainAttachment>, StoreError> {
        self.inner.list().await
    }

    async fn detach(&self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError> {
        self.inner.detach(hostname_ascii, tombstone_secs).await?;
        self.persist()
    }

    async fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> Result<bool, StoreError> {
        self.inner.is_tombstoned(hostname_ascii, now).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment::{
        DnsMode, DnsStatus, OwnershipMethod, OwnershipStatus, TargetKind, TlsMode,
    };
    use crate::hostname::Hostname;

    fn attachment(host: &str) -> DomainAttachment {
        DomainAttachment {
            id: format!("dom_{host}"),
            hostname: Hostname::parse(host).unwrap(),
            project: "site".to_owned(),
            environment: "production".to_owned(),
            target_kind: TargetKind::Service,
            target_ref: "svc".to_owned(),
            dns_mode: DnsMode::Manual,
            tls_mode: TlsMode::ManagedHttp01,
            ownership_method: OwnershipMethod::Txt,
            ownership_name: "_ag-domain.example.com".to_owned(),
            ownership_value: "ag-verification=x".to_owned(),
            ownership_status: OwnershipStatus::Pending,
            dns_status: DnsStatus::Pending,
            tls_status: TlsStatus::Pending,
            routing_status: RoutingStatus::Disabled,
            lifecycle: AttachmentLifecycle::PendingOwnership,
            created_at: now_unix(),
        }
    }

    #[tokio::test]
    async fn create_and_get() {
        let s = InMemoryStore::new();
        s.create(attachment("api.example.com")).await.unwrap();
        assert!(s.get("api.example.com").await.unwrap().is_some());
        assert!(s.get_by_id("dom_api.example.com").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn duplicate_hostname_rejected() {
        let s = InMemoryStore::new();
        s.create(attachment("api.example.com")).await.unwrap();
        let err = s.create(attachment("api.example.com")).await.unwrap_err();
        assert!(matches!(err, StoreError::AlreadyExists(_)));
    }

    #[tokio::test]
    async fn detach_creates_tombstone_and_blocks_reclaim() {
        let s = InMemoryStore::new();
        s.create(attachment("api.example.com")).await.unwrap();
        s.detach("api.example.com", DEFAULT_TOMBSTONE_SECS)
            .await
            .unwrap();
        assert!(s
            .is_tombstoned("api.example.com", now_unix())
            .await
            .unwrap());
        let err = s.create(attachment("api.example.com")).await.unwrap_err();
        assert!(matches!(err, StoreError::Tombstoned { .. }));
    }

    #[tokio::test]
    async fn json_store_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ag-domains-test-{}", uuid::Uuid::new_v4()));
        let path = dir.join("attachments.json");
        {
            let s = JsonFileStore::open(&path).unwrap();
            s.create(attachment("api.example.com")).await.unwrap();
        }
        let s2 = JsonFileStore::open(&path).unwrap();
        assert!(s2.get("api.example.com").await.unwrap().is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
