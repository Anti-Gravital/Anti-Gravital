//! Dangling-DNS detection (blueprint section 15.3, subdomain-takeover hygiene).
//!
//! A hostname is *dangling* when it still points at the Anti-Gravital edge in
//! DNS but no attachment owns it any more (it was detached, or was never
//! attached). Left unaddressed, a third party could attach the same hostname and
//! receive traffic meant for the original owner. The edge already fails closed
//! for unattached hostnames (`ag-edge` route resolution never serves another
//! tenant's content), so this module adds the *detection* half: scan candidate
//! hostnames, decide which are dangling, and emit
//! [`DomainEvent::DanglingDnsDetected`] so operators can be notified.
//!
//! The decision is a pure function ([`classify`]); the worker ([`scan_dangling`])
//! feeds it from an [`AttachmentStore`] and an [`EventSink`]. Both are
//! deterministic and unit-tested without any network: the DNS observation
//! (`points_to_edge`) is supplied by the caller's resolver.

use crate::{
    attachment::AttachmentLifecycle,
    events::{DomainEvent, EventSink},
    store::{AttachmentStore, StoreError},
};

/// The result of classifying a candidate hostname.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanglingDecision {
    /// Points at the edge but is not attached: a takeover risk.
    Dangling,
    /// Either does not point at the edge, or is legitimately attached.
    Safe,
}

/// Pure decision: a hostname is dangling iff it points at the edge and is not
/// attached. This is the security invariant, isolated from any I/O.
pub fn classify(points_to_edge: bool, attached: bool) -> DanglingDecision {
    if points_to_edge && !attached {
        DanglingDecision::Dangling
    } else {
        DanglingDecision::Safe
    }
}

/// A hostname to evaluate, with the DNS observation already made.
///
/// `points_to_edge` is the resolver's verdict that the hostname's A/AAAA/CNAME
/// currently targets the Anti-Gravital edge. The worker supplies the attachment
/// side from the store, so this type carries only the DNS half.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingCandidate {
    /// Canonical ASCII hostname.
    pub hostname: String,
    /// Whether DNS currently points this hostname at the edge.
    pub points_to_edge: bool,
}

impl DanglingCandidate {
    /// Convenience constructor.
    pub fn new(hostname: impl Into<String>, points_to_edge: bool) -> Self {
        Self {
            hostname: hostname.into(),
            points_to_edge,
        }
    }
}

/// Whether the store holds a live (non-detached) attachment for `hostname`.
///
/// A detached attachment does not count as attached: the hostname is in the
/// post-detach window that subdomain takeover exploits.
async fn is_attached(store: &dyn AttachmentStore, hostname: &str) -> Result<bool, StoreError> {
    Ok(store
        .get(hostname)
        .await?
        .map(|a| a.lifecycle != AttachmentLifecycle::Detached)
        .unwrap_or(false))
}

/// Scan candidates and return the hostnames that are dangling.
///
/// For each dangling hostname, emits [`DomainEvent::DanglingDnsDetected`] to the
/// sink so operators are notified. Never serves or mutates routing; detection
/// only. The edge continues to fail closed for these hostnames.
pub async fn scan_dangling(
    store: &dyn AttachmentStore,
    sink: &dyn EventSink,
    candidates: &[DanglingCandidate],
) -> Result<Vec<String>, StoreError> {
    let mut dangling = Vec::new();
    for candidate in candidates {
        let attached = is_attached(store, &candidate.hostname).await?;
        if classify(candidate.points_to_edge, attached) == DanglingDecision::Dangling {
            sink.emit(DomainEvent::DanglingDnsDetected {
                hostname: candidate.hostname.clone(),
            })
            .await;
            dangling.push(candidate.hostname.clone());
        }
    }
    Ok(dangling)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment::{
        DnsMode, DnsStatus, DomainAttachment, OwnershipMethod, OwnershipStatus, RoutingStatus,
        TargetKind, TlsMode, TlsStatus,
    };
    use crate::events::InMemoryEventSink;
    use crate::hostname::Hostname;
    use crate::store::InMemoryStore;

    #[test]
    fn classify_truth_table() {
        assert_eq!(classify(true, false), DanglingDecision::Dangling);
        assert_eq!(classify(true, true), DanglingDecision::Safe);
        assert_eq!(classify(false, false), DanglingDecision::Safe);
        assert_eq!(classify(false, true), DanglingDecision::Safe);
    }

    fn attachment(hostname: &str) -> DomainAttachment {
        let h = Hostname::parse(hostname).unwrap();
        DomainAttachment {
            id: "dom_x".to_owned(),
            hostname: h,
            project: "p".to_owned(),
            environment: "production".to_owned(),
            target_kind: TargetKind::Project,
            target_ref: "default".to_owned(),
            dns_mode: DnsMode::Manual,
            tls_mode: TlsMode::ManagedHttp01,
            ownership_method: OwnershipMethod::Txt,
            ownership_name: "_ag-domain.example.com".to_owned(),
            ownership_value: "ag-verification=x".to_owned(),
            ownership_status: OwnershipStatus::Verified,
            dns_status: DnsStatus::Routable,
            tls_status: TlsStatus::Active,
            routing_status: RoutingStatus::Ready,
            lifecycle: AttachmentLifecycle::Active,
            created_at: 0,
        }
    }

    #[tokio::test]
    async fn points_to_edge_without_attachment_is_dangling() {
        let store = InMemoryStore::default();
        let sink = InMemoryEventSink::new();
        let candidates = vec![DanglingCandidate::new("ghost.example.com", true)];

        let dangling = scan_dangling(&store, &sink, &candidates).await.unwrap();
        assert_eq!(dangling, vec!["ghost.example.com".to_owned()]);
        assert_eq!(sink.len(), 1);
        assert_eq!(
            sink.events()[0].event_type(),
            "domain.dangling_dns_detected"
        );
    }

    #[tokio::test]
    async fn attached_hostname_is_safe() {
        let store = InMemoryStore::default();
        store.create(attachment("api.example.com")).await.unwrap();
        let sink = InMemoryEventSink::new();
        let candidates = vec![DanglingCandidate::new("api.example.com", true)];

        let dangling = scan_dangling(&store, &sink, &candidates).await.unwrap();
        assert!(dangling.is_empty());
        assert!(sink.is_empty(), "no event for a legitimately attached host");
    }

    #[tokio::test]
    async fn detached_hostname_pointing_to_edge_is_dangling() {
        let store = InMemoryStore::default();
        store.create(attachment("api.example.com")).await.unwrap();
        // Detach removes the live attachment and leaves a tombstone: exactly the
        // takeover window.
        store.detach("api.example.com", 3600).await.unwrap();
        let sink = InMemoryEventSink::new();
        let candidates = vec![DanglingCandidate::new("api.example.com", true)];

        let dangling = scan_dangling(&store, &sink, &candidates).await.unwrap();
        assert_eq!(dangling, vec!["api.example.com".to_owned()]);
        assert_eq!(sink.len(), 1);
    }

    #[tokio::test]
    async fn not_pointing_to_edge_is_safe() {
        let store = InMemoryStore::default();
        let sink = InMemoryEventSink::new();
        let candidates = vec![DanglingCandidate::new("elsewhere.example.com", false)];

        let dangling = scan_dangling(&store, &sink, &candidates).await.unwrap();
        assert!(dangling.is_empty());
        assert!(sink.is_empty());
    }
}
