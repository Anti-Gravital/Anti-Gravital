//! Attachment persistence.
//!
//! The store keeps domain attachments and tombstones. Per ADR-0009 the default
//! is native and self-hostable: an in-memory store and a local JSON-file store.
//! A SQL-backed store is a later, feature-gated phase and is never required to
//! attach a domain.
//!
//! Tombstones (RFC-0009 section 7; blueprint section 15.2) prevent another
//! party from immediately re-claiming a detached hostname (subdomain-takeover
//! prevention): re-attaching a tombstoned hostname requires fresh ownership.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// (De)serialization error (JSON-file store).
    #[error("store serialization error: {0}")]
    Serde(String),
}

/// Attachment persistence contract.
///
/// Synchronous by design: the native stores are local. A future SQL adapter
/// may wrap this behind its own async API without changing callers that use
/// the native default.
pub trait AttachmentStore {
    /// Creates a new attachment. Fails if an active attachment exists or the
    /// hostname is tombstoned.
    fn create(&mut self, attachment: DomainAttachment) -> Result<(), StoreError>;
    /// Fetches an attachment by canonical hostname.
    fn get(&self, hostname_ascii: &str) -> Option<DomainAttachment>;
    /// Fetches an attachment by id.
    fn get_by_id(&self, id: &str) -> Option<DomainAttachment>;
    /// Replaces an existing attachment (matched by hostname).
    fn update(&mut self, attachment: DomainAttachment) -> Result<(), StoreError>;
    /// Lists all attachments.
    fn list(&self) -> Vec<DomainAttachment>;
    /// Detaches a hostname: marks it detached, stops routing/TLS, and writes a
    /// tombstone valid for `tombstone_secs`.
    fn detach(&mut self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError>;
    /// Whether a hostname is currently tombstoned.
    fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> bool;
}

/// In-memory attachment store (native default; non-persistent).
#[derive(Debug, Default)]
pub struct InMemoryStore {
    attachments: HashMap<String, DomainAttachment>,
    tombstones: HashMap<String, Tombstone>,
}

impl InMemoryStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl AttachmentStore for InMemoryStore {
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

    fn get(&self, hostname_ascii: &str) -> Option<DomainAttachment> {
        self.attachments.get(hostname_ascii).cloned()
    }

    fn get_by_id(&self, id: &str) -> Option<DomainAttachment> {
        self.attachments.values().find(|a| a.id == id).cloned()
    }

    fn update(&mut self, attachment: DomainAttachment) -> Result<(), StoreError> {
        let key = attachment.hostname.ascii().to_owned();
        if !self.attachments.contains_key(&key) {
            return Err(StoreError::NotFound(key));
        }
        self.attachments.insert(key, attachment);
        Ok(())
    }

    fn list(&self) -> Vec<DomainAttachment> {
        let mut v: Vec<_> = self.attachments.values().cloned().collect();
        v.sort_by(|a, b| a.hostname.ascii().cmp(b.hostname.ascii()));
        v
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

    fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> bool {
        self.tombstones
            .get(hostname_ascii)
            .map(|t| t.tombstone_until > now)
            .unwrap_or(false)
    }
}

/// Serializable snapshot persisted by [`JsonFileStore`].
#[derive(Debug, Default, Serialize, Deserialize)]
struct Snapshot {
    attachments: Vec<DomainAttachment>,
    tombstones: Vec<Tombstone>,
}

/// JSON-file backed store (native, self-hostable, persistent).
///
/// Loads on construction and writes the whole snapshot after each mutation.
/// Suitable for single-node operation and the CLI; a SQL store is the
/// multi-node, later-phase option.
#[derive(Debug)]
pub struct JsonFileStore {
    path: PathBuf,
    inner: InMemoryStore,
}

impl JsonFileStore {
    /// Opens (or initializes) a JSON-file store at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let mut inner = InMemoryStore::new();
        if path.exists() {
            let bytes = std::fs::read(&path).map_err(|e| StoreError::Io(e.to_string()))?;
            let snapshot: Snapshot =
                serde_json::from_slice(&bytes).map_err(|e| StoreError::Serde(e.to_string()))?;
            for a in snapshot.attachments {
                inner.attachments.insert(a.hostname.ascii().to_owned(), a);
            }
            for t in snapshot.tombstones {
                inner.tombstones.insert(t.hostname_ascii.clone(), t);
            }
        }
        Ok(Self { path, inner })
    }

    fn persist(&self) -> Result<(), StoreError> {
        let snapshot = Snapshot {
            attachments: self.inner.attachments.values().cloned().collect(),
            tombstones: self.inner.tombstones.values().cloned().collect(),
        };
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

impl AttachmentStore for JsonFileStore {
    fn create(&mut self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.inner.create(attachment)?;
        self.persist()
    }

    fn get(&self, hostname_ascii: &str) -> Option<DomainAttachment> {
        self.inner.get(hostname_ascii)
    }

    fn get_by_id(&self, id: &str) -> Option<DomainAttachment> {
        self.inner.get_by_id(id)
    }

    fn update(&mut self, attachment: DomainAttachment) -> Result<(), StoreError> {
        self.inner.update(attachment)?;
        self.persist()
    }

    fn list(&self) -> Vec<DomainAttachment> {
        self.inner.list()
    }

    fn detach(&mut self, hostname_ascii: &str, tombstone_secs: u64) -> Result<(), StoreError> {
        self.inner.detach(hostname_ascii, tombstone_secs)?;
        self.persist()
    }

    fn is_tombstoned(&self, hostname_ascii: &str, now: u64) -> bool {
        self.inner.is_tombstoned(hostname_ascii, now)
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

    #[test]
    fn create_and_get() {
        let mut s = InMemoryStore::new();
        s.create(attachment("api.example.com")).unwrap();
        assert!(s.get("api.example.com").is_some());
        assert!(s.get_by_id("dom_api.example.com").is_some());
    }

    #[test]
    fn duplicate_hostname_rejected() {
        let mut s = InMemoryStore::new();
        s.create(attachment("api.example.com")).unwrap();
        let err = s.create(attachment("api.example.com")).unwrap_err();
        assert!(matches!(err, StoreError::AlreadyExists(_)));
    }

    #[test]
    fn detach_creates_tombstone_and_blocks_reclaim() {
        let mut s = InMemoryStore::new();
        s.create(attachment("api.example.com")).unwrap();
        s.detach("api.example.com", DEFAULT_TOMBSTONE_SECS).unwrap();
        assert!(s.is_tombstoned("api.example.com", now_unix()));
        let err = s.create(attachment("api.example.com")).unwrap_err();
        assert!(matches!(err, StoreError::Tombstoned { .. }));
    }

    #[test]
    fn json_store_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ag-domains-test-{}", uuid::Uuid::new_v4()));
        let path = dir.join("attachments.json");
        {
            let mut s = JsonFileStore::open(&path).unwrap();
            s.create(attachment("api.example.com")).unwrap();
        }
        let s2 = JsonFileStore::open(&path).unwrap();
        assert!(s2.get("api.example.com").is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
