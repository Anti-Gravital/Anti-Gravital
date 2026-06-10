//! Integration tests for the Postgres attachment store.
//!
//! These require a real Postgres instance and are `#[ignore]` by default
//! (repo convention for service-backed tests). Run with:
//!
//! ```text
//! DATABASE_URL=postgres://user:pass@localhost/ag_domains_test \
//!   cargo test -p ag-domains --features sql-store --test sql_store_pg -- --ignored
//! ```
#![cfg(feature = "sql-store")]

use ag_domains::attachment::{
    AttachmentLifecycle, DnsMode, DnsStatus, DomainAttachment, OwnershipMethod, OwnershipStatus,
    RoutingStatus, TargetKind, TlsMode, TlsStatus,
};
use ag_domains::hostname::Hostname;
use ag_domains::sql_store::SqlAttachmentStore;
use ag_domains::store::{now_unix, AttachmentStore, StoreError, DEFAULT_TOMBSTONE_SECS};

fn unique_host() -> String {
    format!("t{}.example.com", uuid::Uuid::new_v4().simple())
}

fn attachment(host: &str) -> DomainAttachment {
    DomainAttachment {
        id: format!("dom_{}", uuid::Uuid::new_v4().simple()),
        hostname: Hostname::parse(host).unwrap(),
        project: "site".to_owned(),
        environment: "production".to_owned(),
        target_kind: TargetKind::Service,
        target_ref: "svc".to_owned(),
        dns_mode: DnsMode::Manual,
        tls_mode: TlsMode::ManagedHttp01,
        ownership_method: OwnershipMethod::Txt,
        ownership_name: format!("_ag-domain.{host}"),
        ownership_value: "ag-verification=x".to_owned(),
        ownership_status: OwnershipStatus::Pending,
        dns_status: DnsStatus::Pending,
        tls_status: TlsStatus::Pending,
        routing_status: RoutingStatus::Disabled,
        lifecycle: AttachmentLifecycle::PendingOwnership,
        created_at: now_unix(),
    }
}

async fn store() -> SqlAttachmentStore {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for SQL store tests");
    let store = SqlAttachmentStore::connect(&url).await.unwrap();
    store.migrate().await.unwrap();
    store
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn create_get_update_list() {
    let store = store().await;
    let host = unique_host();
    let mut a = attachment(&host);

    store.create(a.clone()).await.unwrap();
    assert!(store.get(&host).await.unwrap().is_some());
    assert!(store.get_by_id(&a.id).await.unwrap().is_some());

    a.ownership_status = OwnershipStatus::Verified;
    a.recompute_lifecycle();
    store.update(a.clone()).await.unwrap();
    let got = store.get(&host).await.unwrap().unwrap();
    assert_eq!(got.ownership_status, OwnershipStatus::Verified);

    assert!(store
        .list()
        .await
        .unwrap()
        .iter()
        .any(|x| x.hostname.ascii() == host));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn duplicate_is_conflict() {
    let store = store().await;
    let host = unique_host();
    store.create(attachment(&host)).await.unwrap();
    let err = store.create(attachment(&host)).await.unwrap_err();
    assert!(matches!(err, StoreError::AlreadyExists(_)));
}

#[tokio::test]
#[ignore = "requires DATABASE_URL to a Postgres instance"]
async fn detach_tombstones_and_blocks_reclaim() {
    let store = store().await;
    let host = unique_host();
    store.create(attachment(&host)).await.unwrap();

    store.detach(&host, DEFAULT_TOMBSTONE_SECS).await.unwrap();
    assert!(store.get(&host).await.unwrap().is_none());
    assert!(store.is_tombstoned(&host, now_unix()).await.unwrap());

    let err = store.create(attachment(&host)).await.unwrap_err();
    assert!(matches!(err, StoreError::Tombstoned { .. }));
}
