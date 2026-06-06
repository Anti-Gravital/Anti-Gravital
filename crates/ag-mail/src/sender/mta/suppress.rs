//! Automatic suppression list for the native MTA (Phase 4.6-B / DEBT-020).
//!
//! Recipients that hard-bounce, complain, or unsubscribe are recorded here so
//! the delivery queue skips them on future sends. This is the in-memory,
//! native-default store; a durable backing (PostgreSQL) is a later, optional
//! step. Addresses are normalized (trimmed, lowercased) so lookups are
//! case-insensitive.

use std::collections::HashMap;
use std::sync::RwLock;

/// Why an address is suppressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionReason {
    /// Permanent SMTP failure (5xx) or a DSN with a permanent status.
    HardBounce,
    /// Feedback-loop / ARF complaint.
    Complaint,
    /// Recipient opted out (one-click or list unsubscribe).
    Unsubscribe,
    /// Repeated transient failures crossed the soft-bounce threshold.
    SoftBounceLimit,
}

/// Thread-safe set of suppressed addresses with their reason.
#[derive(Debug, Default)]
pub struct SuppressionList {
    entries: RwLock<HashMap<String, SuppressionReason>>,
}

fn normalize(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

impl SuppressionList {
    /// Creates an empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Suppresses an address. The strongest reason wins: an existing
    /// `HardBounce`/`Complaint`/`Unsubscribe` is not downgraded to
    /// `SoftBounceLimit`.
    pub fn suppress(&self, email: &str, reason: SuppressionReason) {
        let key = normalize(email);
        let mut entries = self.entries.write().expect("suppression lock poisoned");
        entries
            .entry(key)
            .and_modify(|existing| {
                if matches!(existing, SuppressionReason::SoftBounceLimit) {
                    *existing = reason;
                }
            })
            .or_insert(reason);
    }

    /// Whether an address is suppressed.
    pub fn is_suppressed(&self, email: &str) -> bool {
        let key = normalize(email);
        self.entries
            .read()
            .expect("suppression lock poisoned")
            .contains_key(&key)
    }

    /// The reason an address is suppressed, if any.
    pub fn reason(&self, email: &str) -> Option<SuppressionReason> {
        let key = normalize(email);
        self.entries
            .read()
            .expect("suppression lock poisoned")
            .get(&key)
            .copied()
    }

    /// Number of suppressed addresses.
    pub fn len(&self) -> usize {
        self.entries
            .read()
            .expect("suppression lock poisoned")
            .len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppress_and_lookup_case_insensitive() {
        let list = SuppressionList::new();
        assert!(!list.is_suppressed("user@example.com"));
        list.suppress("User@Example.COM ", SuppressionReason::HardBounce);
        assert!(list.is_suppressed("user@example.com"));
        assert_eq!(
            list.reason("USER@example.com"),
            Some(SuppressionReason::HardBounce)
        );
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn soft_limit_does_not_downgrade_a_hard_bounce() {
        let list = SuppressionList::new();
        list.suppress("a@x.com", SuppressionReason::HardBounce);
        list.suppress("a@x.com", SuppressionReason::SoftBounceLimit);
        assert_eq!(list.reason("a@x.com"), Some(SuppressionReason::HardBounce));
    }

    #[test]
    fn soft_limit_is_upgraded_by_a_complaint() {
        let list = SuppressionList::new();
        list.suppress("a@x.com", SuppressionReason::SoftBounceLimit);
        list.suppress("a@x.com", SuppressionReason::Complaint);
        assert_eq!(list.reason("a@x.com"), Some(SuppressionReason::Complaint));
    }

    #[test]
    fn empty_by_default() {
        assert!(SuppressionList::new().is_empty());
    }
}
