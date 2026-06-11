//! Abuse controls for the domain control plane (blueprint section 15.6).
//!
//! Three dimensions beyond the per-registered-domain issuance counter in
//! [`crate::issuance`]:
//!
//! - **Per-tenant attachment limit** — caps how many domains one tenant may
//!   attach, bounding fan-out from a single account.
//! - **Per-tenant issuance limit** — caps certificate issuances per tenant in a
//!   window, complementing the per-registered-domain CA limit.
//! - **Global ACME issuance queue** — bounds *concurrent* in-flight ACME orders
//!   across all tenants, so a burst cannot exhaust the CA's rate budget or the
//!   issuance worker pool. Admission is immediate accept-or-reject (a bounded
//!   queue), returning a RAII permit that frees the slot on drop.
//!
//! All checks are pure, deterministic and lock-based (no clock, no network), so
//! they are unit-tested directly.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configured abuse limits. A `0` limit means "unlimited" for that dimension,
/// so callers can enable only the controls they need.
#[derive(Debug, Clone, Copy)]
pub struct AbuseLimits {
    /// Maximum live attachments per tenant (`0` = unlimited).
    pub max_attachments_per_tenant: u32,
    /// Maximum certificate issuances per tenant in the window (`0` = unlimited).
    pub max_issuance_per_tenant: u32,
    /// Maximum concurrent in-flight ACME orders, globally (`0` = unlimited).
    pub max_global_acme_concurrency: usize,
}

impl Default for AbuseLimits {
    fn default() -> Self {
        // Conservative non-zero defaults; operators tune per deployment.
        Self {
            max_attachments_per_tenant: 100,
            max_issuance_per_tenant: 50,
            max_global_acme_concurrency: 16,
        }
    }
}

/// Why an abuse check rejected a request.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AbuseRejection {
    /// The tenant is at its attachment limit.
    #[error("tenant '{tenant}' attachment limit reached ({count}/{max})")]
    TenantAttachmentLimit {
        /// Tenant identifier.
        tenant: String,
        /// Current count.
        count: u32,
        /// Configured maximum.
        max: u32,
    },
    /// The tenant is at its issuance limit for the window.
    #[error("tenant '{tenant}' issuance limit reached ({count}/{max})")]
    TenantIssuanceLimit {
        /// Tenant identifier.
        tenant: String,
        /// Current count.
        count: u32,
        /// Configured maximum.
        max: u32,
    },
    /// The global ACME issuance queue is full.
    #[error("global ACME issuance queue full ({in_flight}/{max})")]
    GlobalAcmeQueueFull {
        /// Current in-flight orders.
        in_flight: usize,
        /// Configured maximum.
        max: usize,
    },
}

#[derive(Default)]
struct AbuseState {
    attachments_per_tenant: HashMap<String, u32>,
    issuance_per_tenant: HashMap<String, u32>,
    acme_in_flight: usize,
}

/// Enforces the blueprint section 15.6 abuse controls.
///
/// Cheap to clone (shared inner state), so it can live in `ApiState` and be
/// queried from multiple handlers.
#[derive(Clone)]
pub struct AbuseControls {
    limits: AbuseLimits,
    state: Arc<Mutex<AbuseState>>,
}

impl AbuseControls {
    /// Creates abuse controls with the given limits.
    pub fn new(limits: AbuseLimits) -> Self {
        Self {
            limits,
            state: Arc::new(Mutex::new(AbuseState::default())),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, AbuseState> {
        self.state.lock().expect("abuse controls lock poisoned")
    }

    /// Registers a new attachment for `tenant`, enforcing the per-tenant limit.
    ///
    /// On success the tenant's attachment count is incremented; call
    /// [`AbuseControls::release_attachment`] when the attachment is detached.
    pub fn register_attachment(&self, tenant: &str) -> Result<(), AbuseRejection> {
        let max = self.limits.max_attachments_per_tenant;
        let mut st = self.lock();
        let count = st.attachments_per_tenant.get(tenant).copied().unwrap_or(0);
        if max != 0 && count >= max {
            return Err(AbuseRejection::TenantAttachmentLimit {
                tenant: tenant.to_owned(),
                count,
                max,
            });
        }
        *st.attachments_per_tenant
            .entry(tenant.to_owned())
            .or_insert(0) += 1;
        Ok(())
    }

    /// Decrements a tenant's attachment count (on detach). Saturating at zero.
    pub fn release_attachment(&self, tenant: &str) {
        let mut st = self.lock();
        if let Some(count) = st.attachments_per_tenant.get_mut(tenant) {
            *count = count.saturating_sub(1);
        }
    }

    /// Records a successful certificate issuance for `tenant`, enforcing the
    /// per-tenant issuance limit for the window.
    pub fn record_issuance(&self, tenant: &str) -> Result<(), AbuseRejection> {
        let max = self.limits.max_issuance_per_tenant;
        let mut st = self.lock();
        let count = st.issuance_per_tenant.get(tenant).copied().unwrap_or(0);
        if max != 0 && count >= max {
            return Err(AbuseRejection::TenantIssuanceLimit {
                tenant: tenant.to_owned(),
                count,
                max,
            });
        }
        *st.issuance_per_tenant.entry(tenant.to_owned()).or_insert(0) += 1;
        Ok(())
    }

    /// Current attachment count for a tenant (diagnostics/tests).
    pub fn attachment_count(&self, tenant: &str) -> u32 {
        self.lock()
            .attachments_per_tenant
            .get(tenant)
            .copied()
            .unwrap_or(0)
    }

    /// Acquires a slot in the global ACME issuance queue, or rejects if full.
    ///
    /// The returned [`AcmePermit`] frees the slot when dropped, so the global
    /// concurrency bound is held only for the lifetime of the order.
    pub fn acquire_acme_slot(&self) -> Result<AcmePermit, AbuseRejection> {
        let max = self.limits.max_global_acme_concurrency;
        let mut st = self.lock();
        if max != 0 && st.acme_in_flight >= max {
            return Err(AbuseRejection::GlobalAcmeQueueFull {
                in_flight: st.acme_in_flight,
                max,
            });
        }
        st.acme_in_flight += 1;
        Ok(AcmePermit {
            state: Arc::clone(&self.state),
        })
    }

    /// Current number of in-flight ACME orders (diagnostics/tests).
    pub fn acme_in_flight(&self) -> usize {
        self.lock().acme_in_flight
    }
}

/// RAII permit for a slot in the global ACME issuance queue.
///
/// Dropping it returns the slot to the queue.
pub struct AcmePermit {
    state: Arc<Mutex<AbuseState>>,
}

impl std::fmt::Debug for AcmePermit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcmePermit").finish_non_exhaustive()
    }
}

impl Drop for AcmePermit {
    fn drop(&mut self) {
        if let Ok(mut st) = self.state.lock() {
            st.acme_in_flight = st.acme_in_flight.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(att: u32, iss: u32, acme: usize) -> AbuseLimits {
        AbuseLimits {
            max_attachments_per_tenant: att,
            max_issuance_per_tenant: iss,
            max_global_acme_concurrency: acme,
        }
    }

    #[test]
    fn attachment_limit_enforced_per_tenant() {
        let c = AbuseControls::new(limits(2, 0, 0));
        assert!(c.register_attachment("acme-corp").is_ok());
        assert!(c.register_attachment("acme-corp").is_ok());
        let err = c.register_attachment("acme-corp").unwrap_err();
        assert!(matches!(
            err,
            AbuseRejection::TenantAttachmentLimit {
                max: 2,
                count: 2,
                ..
            }
        ));
        // A different tenant has its own budget.
        assert!(c.register_attachment("other-corp").is_ok());
    }

    #[test]
    fn release_attachment_frees_a_slot() {
        let c = AbuseControls::new(limits(1, 0, 0));
        assert!(c.register_attachment("t").is_ok());
        assert!(c.register_attachment("t").is_err());
        c.release_attachment("t");
        assert_eq!(c.attachment_count("t"), 0);
        assert!(c.register_attachment("t").is_ok());
    }

    #[test]
    fn zero_limit_means_unlimited() {
        let c = AbuseControls::new(limits(0, 0, 0));
        for _ in 0..1000 {
            c.register_attachment("t").unwrap();
        }
        assert_eq!(c.attachment_count("t"), 1000);
    }

    #[test]
    fn issuance_limit_enforced_per_tenant() {
        let c = AbuseControls::new(limits(0, 2, 0));
        assert!(c.record_issuance("t").is_ok());
        assert!(c.record_issuance("t").is_ok());
        let err = c.record_issuance("t").unwrap_err();
        assert!(matches!(
            err,
            AbuseRejection::TenantIssuanceLimit { max: 2, .. }
        ));
    }

    #[test]
    fn global_acme_queue_bounds_concurrency() {
        let c = AbuseControls::new(limits(0, 0, 2));
        let p1 = c.acquire_acme_slot().unwrap();
        let p2 = c.acquire_acme_slot().unwrap();
        assert_eq!(c.acme_in_flight(), 2);
        // Third concurrent order is rejected.
        let err = c.acquire_acme_slot().unwrap_err();
        assert!(matches!(
            err,
            AbuseRejection::GlobalAcmeQueueFull {
                in_flight: 2,
                max: 2
            }
        ));
        // Dropping a permit frees the slot for the next order.
        drop(p1);
        assert_eq!(c.acme_in_flight(), 1);
        let _p3 = c.acquire_acme_slot().unwrap();
        assert_eq!(c.acme_in_flight(), 2);
        drop(p2);
    }

    #[test]
    fn permits_release_on_drop_even_in_bulk() {
        let c = AbuseControls::new(limits(0, 0, 3));
        {
            let _a = c.acquire_acme_slot().unwrap();
            let _b = c.acquire_acme_slot().unwrap();
            let _c = c.acquire_acme_slot().unwrap();
            assert_eq!(c.acme_in_flight(), 3);
            assert!(c.acquire_acme_slot().is_err());
        }
        assert_eq!(c.acme_in_flight(), 0, "all permits freed on scope exit");
    }
}
