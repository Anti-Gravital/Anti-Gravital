//! Domain ownership proof via a TXT record.
//!
//! Before an attachment may serve traffic, the operator must prove control of
//! the domain by publishing a TXT record at `_ag-domain.<registered-domain>`
//! containing a per-attachment verification token (RFC-0011 section 5.3,
//! CLAUDE.md security rule 16: prove control, never trust dashboard input).

use crate::hostname::Hostname;

/// The fixed label under which ownership TXT records are published.
pub const OWNERSHIP_LABEL: &str = "_ag-domain";

/// Prefix of the ownership token value.
pub const OWNERSHIP_PREFIX: &str = "ag-verification=";

/// The fully-qualified name of the ownership TXT record for a hostname.
///
/// The record always lives at the registered domain so a single proof covers
/// the apex and all of its subdomains/wildcards.
pub fn ownership_record_name(hostname: &Hostname) -> String {
    format!("{OWNERSHIP_LABEL}.{}", hostname.registered_domain())
}

/// Generates an opaque ownership token value for an attachment id.
///
/// The token is unguessable so a third party cannot pre-publish it. Format:
/// `ag-verification=<attachment-id>-<random>`.
pub fn generate_token(attachment_id: &str) -> String {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    format!("{OWNERSHIP_PREFIX}{attachment_id}-{nonce}")
}

/// Generates a fresh attachment id (`dom_<uuid-simple>`).
pub fn generate_attachment_id() -> String {
    format!("dom_{}", uuid::Uuid::new_v4().simple())
}

/// Outcome of an ownership verification probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipProbe {
    /// Whether the expected token was observed.
    pub verified: bool,
    /// How many resolvers confirmed the token.
    pub confirmed: usize,
    /// Total resolvers queried.
    pub total: usize,
}

#[cfg(feature = "propagation")]
mod verify {
    use super::*;
    use crate::propagation::PropagationChecker;

    /// Verifies the ownership TXT record across public resolvers.
    ///
    /// Returns an [`OwnershipProbe`]; the caller decides how to fold this into
    /// the attachment's [`crate::attachment::OwnershipStatus`].
    pub async fn verify_ownership(
        checker: &PropagationChecker,
        hostname: &Hostname,
        expected_token: &str,
    ) -> OwnershipProbe {
        let name = ownership_record_name(hostname);
        let result = checker.check_txt(&name, expected_token).await;
        OwnershipProbe {
            verified: result.confirmed > 0,
            confirmed: result.confirmed,
            total: result.total,
        }
    }
}

#[cfg(feature = "propagation")]
pub use verify::verify_ownership;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_name_uses_registered_domain() {
        let h = Hostname::parse("api.example.com").unwrap();
        assert_eq!(ownership_record_name(&h), "_ag-domain.example.com");
        let w = Hostname::parse("*.example.com").unwrap();
        assert_eq!(ownership_record_name(&w), "_ag-domain.example.com");
    }

    #[test]
    fn token_has_prefix_and_id() {
        let t = generate_token("dom_abc");
        assert!(t.starts_with(OWNERSHIP_PREFIX));
        assert!(t.contains("dom_abc-"));
    }

    #[test]
    fn tokens_are_unique() {
        assert_ne!(generate_token("dom_x"), generate_token("dom_x"));
    }

    #[test]
    fn attachment_ids_unique_and_prefixed() {
        let a = generate_attachment_id();
        let b = generate_attachment_id();
        assert!(a.starts_with("dom_"));
        assert_ne!(a, b);
    }
}
