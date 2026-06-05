//! Domain attachment state machine and record.
//!
//! An attachment is the canonical binding between a hostname and an
//! Anti-Gravital target (project + environment + service). It tracks four
//! independent readiness dimensions (ownership, DNS, TLS, routing) plus a
//! derived lifecycle state, following RFC-0009 section 4 and the blueprint
//! state machine. A single boolean `verified` is deliberately avoided.

use serde::{Deserialize, Serialize};

use crate::hostname::Hostname;

/// What kind of target a hostname binds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// A specific service inside a project/environment.
    Service,
    /// A whole project (default service resolution applies downstream).
    Project,
}

/// How DNS records are applied for this attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsMode {
    /// The operator pastes records manually (native default).
    Manual,
    /// A provider adapter applies records automatically (feature-gated).
    Provider,
}

/// Managed TLS mode for this attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsMode {
    /// No managed certificate.
    Disabled,
    /// ACME HTTP-01 challenge (exact hostnames).
    ManagedHttp01,
    /// ACME DNS-01 challenge (required for wildcards).
    ManagedDns01,
}

/// Ownership proof method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipMethod {
    /// TXT record under `_ag-domain` (default).
    Txt,
}

/// Ownership readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipStatus {
    /// Proof not yet observed.
    Pending,
    /// Control of the domain proven.
    Verified,
    /// Proof attempt failed.
    Failed,
    /// A previously verified proof expired.
    Expired,
}

/// DNS routing readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsStatus {
    /// Not yet observed.
    Pending,
    /// Records point at the Anti-Gravital edge.
    Routable,
    /// Records point somewhere else.
    WrongTarget,
    /// Conflicting records detected (e.g. CNAME + A at apex).
    ConflictingRecords,
    /// Could not determine.
    Unknown,
    /// Lookup failed.
    Failed,
}

/// Managed certificate readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsStatus {
    /// TLS not managed for this attachment.
    Disabled,
    /// Certificate order in progress.
    Pending,
    /// Certificate issued and served.
    Active,
    /// Within the renewal window.
    RenewalDue,
    /// Issuance or renewal failed.
    Failed,
    /// Certificate expired.
    Expired,
    /// Certificate retired after detach.
    Retired,
}

/// Routing readiness at the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStatus {
    /// Routing not enabled.
    Disabled,
    /// Resolved in shadow mode only (logged, not served).
    Shadow,
    /// Ready to serve once activated.
    Ready,
    /// Actively serving traffic.
    Active,
    /// Suspended by an operator.
    Suspended,
}

/// Derived lifecycle state of an attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentLifecycle {
    /// Created, no proof requested yet.
    Draft,
    /// Awaiting ownership proof.
    PendingOwnership,
    /// Ownership proven, awaiting routing DNS.
    PendingDns,
    /// DNS routable, awaiting certificate.
    PendingTls,
    /// All readiness dimensions satisfied.
    Active,
    /// One or more dimensions reported a failure/conflict.
    Misconfigured,
    /// Detached; routing removed, awaiting tombstone expiry.
    Detached,
}

/// A domain attachment record.
///
/// Timestamps are Unix epoch seconds to avoid coupling the core record to the
/// optional `chrono` dependency (which is gated behind the `acme` feature).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainAttachment {
    /// Opaque attachment id (`dom_<uuid-simple>`).
    pub id: String,
    /// Validated hostname (carries ASCII identity + Unicode display + kind).
    pub hostname: Hostname,
    /// Project identifier this hostname binds to.
    pub project: String,
    /// Environment identifier (e.g. `production`).
    pub environment: String,
    /// Target kind.
    pub target_kind: TargetKind,
    /// Target reference (service id or project id depending on `target_kind`).
    pub target_ref: String,
    /// DNS application mode.
    pub dns_mode: DnsMode,
    /// Managed TLS mode.
    pub tls_mode: TlsMode,
    /// Ownership proof method.
    pub ownership_method: OwnershipMethod,
    /// Ownership record name (e.g. `_ag-domain.example.com`).
    pub ownership_name: String,
    /// Expected ownership record value (e.g. `ag-verification=...`).
    pub ownership_value: String,
    /// Ownership readiness.
    pub ownership_status: OwnershipStatus,
    /// DNS readiness.
    pub dns_status: DnsStatus,
    /// TLS readiness.
    pub tls_status: TlsStatus,
    /// Routing readiness.
    pub routing_status: RoutingStatus,
    /// Derived lifecycle state.
    pub lifecycle: AttachmentLifecycle,
    /// Creation time (Unix epoch seconds).
    pub created_at: u64,
}

impl DomainAttachment {
    /// Activation rule (RFC-0009 section 6.3): a domain is active only when all
    /// four readiness dimensions are satisfied.
    pub fn is_active(&self) -> bool {
        self.ownership_status == OwnershipStatus::Verified
            && self.dns_status == DnsStatus::Routable
            && (self.tls_status == TlsStatus::Active || self.tls_mode == TlsMode::Disabled)
            && self.routing_status == RoutingStatus::Ready
    }

    /// Recomputes `lifecycle` from the current readiness dimensions.
    ///
    /// Detached attachments keep their `Detached` lifecycle (it is terminal
    /// until a fresh attachment is created), so this is a no-op for them.
    pub fn recompute_lifecycle(&mut self) {
        if self.lifecycle == AttachmentLifecycle::Detached {
            return;
        }
        self.lifecycle = self.derive_lifecycle();
    }

    fn derive_lifecycle(&self) -> AttachmentLifecycle {
        use AttachmentLifecycle as L;
        let failed = matches!(
            self.ownership_status,
            OwnershipStatus::Failed | OwnershipStatus::Expired
        ) || matches!(
            self.dns_status,
            DnsStatus::WrongTarget | DnsStatus::ConflictingRecords | DnsStatus::Failed
        ) || matches!(self.tls_status, TlsStatus::Failed | TlsStatus::Expired);
        if failed {
            return L::Misconfigured;
        }
        if self.is_active() {
            return L::Active;
        }
        if self.ownership_status != OwnershipStatus::Verified {
            return L::PendingOwnership;
        }
        if self.dns_status != DnsStatus::Routable {
            return L::PendingDns;
        }
        if self.tls_mode != TlsMode::Disabled && self.tls_status != TlsStatus::Active {
            return L::PendingTls;
        }
        L::PendingOwnership
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hostname::Hostname;

    fn sample() -> DomainAttachment {
        DomainAttachment {
            id: "dom_test".to_owned(),
            hostname: Hostname::parse("example.com").unwrap(),
            project: "site".to_owned(),
            environment: "production".to_owned(),
            target_kind: TargetKind::Service,
            target_ref: "svc_web".to_owned(),
            dns_mode: DnsMode::Manual,
            tls_mode: TlsMode::ManagedHttp01,
            ownership_method: OwnershipMethod::Txt,
            ownership_name: "_ag-domain.example.com".to_owned(),
            ownership_value: "ag-verification=dom_test_x".to_owned(),
            ownership_status: OwnershipStatus::Pending,
            dns_status: DnsStatus::Pending,
            tls_status: TlsStatus::Pending,
            routing_status: RoutingStatus::Disabled,
            lifecycle: AttachmentLifecycle::Draft,
            created_at: 0,
        }
    }

    #[test]
    fn starts_pending_ownership() {
        let mut a = sample();
        a.recompute_lifecycle();
        assert_eq!(a.lifecycle, AttachmentLifecycle::PendingOwnership);
        assert!(!a.is_active());
    }

    #[test]
    fn progresses_to_pending_dns() {
        let mut a = sample();
        a.ownership_status = OwnershipStatus::Verified;
        a.recompute_lifecycle();
        assert_eq!(a.lifecycle, AttachmentLifecycle::PendingDns);
    }

    #[test]
    fn progresses_to_pending_tls() {
        let mut a = sample();
        a.ownership_status = OwnershipStatus::Verified;
        a.dns_status = DnsStatus::Routable;
        a.recompute_lifecycle();
        assert_eq!(a.lifecycle, AttachmentLifecycle::PendingTls);
    }

    #[test]
    fn full_activation() {
        let mut a = sample();
        a.ownership_status = OwnershipStatus::Verified;
        a.dns_status = DnsStatus::Routable;
        a.tls_status = TlsStatus::Active;
        a.routing_status = RoutingStatus::Ready;
        a.recompute_lifecycle();
        assert!(a.is_active());
        assert_eq!(a.lifecycle, AttachmentLifecycle::Active);
    }

    #[test]
    fn wrong_target_is_misconfigured() {
        let mut a = sample();
        a.ownership_status = OwnershipStatus::Verified;
        a.dns_status = DnsStatus::WrongTarget;
        a.recompute_lifecycle();
        assert_eq!(a.lifecycle, AttachmentLifecycle::Misconfigured);
    }

    #[test]
    fn tls_disabled_can_activate_without_cert() {
        let mut a = sample();
        a.tls_mode = TlsMode::Disabled;
        a.tls_status = TlsStatus::Disabled;
        a.ownership_status = OwnershipStatus::Verified;
        a.dns_status = DnsStatus::Routable;
        a.routing_status = RoutingStatus::Ready;
        a.recompute_lifecycle();
        assert!(a.is_active());
    }

    #[test]
    fn detached_is_terminal() {
        let mut a = sample();
        a.lifecycle = AttachmentLifecycle::Detached;
        a.ownership_status = OwnershipStatus::Verified;
        a.dns_status = DnsStatus::Routable;
        a.tls_status = TlsStatus::Active;
        a.routing_status = RoutingStatus::Ready;
        a.recompute_lifecycle();
        assert_eq!(a.lifecycle, AttachmentLifecycle::Detached);
    }
}
