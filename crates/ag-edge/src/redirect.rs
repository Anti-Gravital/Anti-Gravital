//! Canonical host and redirect policy.
//!
//! Supports www<->apex canonicalization and forcing a canonical hostname,
//! preserving path and query, with a configurable permanent redirect status
//! (RFC-0011 section 14.4; blueprint section 14.4). HTTP->HTTPS upgrade is a
//! listener concern handled in a later phase.

use serde::{Deserialize, Serialize};

/// Canonical host redirect policy for a set of hostnames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalPolicy {
    /// The hostname all requests should land on.
    pub canonical_hostname: String,
    /// Hostnames that redirect to the canonical one.
    pub redirect_from: Vec<String>,
    /// Permanent redirect status (typically 301 or 308).
    pub status_code: u16,
    /// Preserve the request path in the redirect target.
    pub preserve_path: bool,
    /// Preserve the query string in the redirect target.
    pub preserve_query: bool,
}

impl Default for CanonicalPolicy {
    fn default() -> Self {
        Self {
            canonical_hostname: String::new(),
            redirect_from: Vec::new(),
            status_code: 308,
            preserve_path: true,
            preserve_query: true,
        }
    }
}

/// A redirect decision for an incoming request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectDecision {
    /// No redirect; serve normally.
    None,
    /// Redirect to `location` with `status`.
    Redirect {
        /// Absolute `https://` location.
        location: String,
        /// HTTP status code.
        status: u16,
    },
}

impl CanonicalPolicy {
    /// Evaluates whether an incoming request should be redirected to the
    /// canonical hostname.
    pub fn evaluate(&self, host: &str, path: &str, query: Option<&str>) -> RedirectDecision {
        let host_norm = host.trim_end_matches('.').to_ascii_lowercase();
        if host_norm == self.canonical_hostname {
            return RedirectDecision::None;
        }
        let should_redirect = self
            .redirect_from
            .iter()
            .any(|h| h.trim_end_matches('.').eq_ignore_ascii_case(&host_norm));
        if !should_redirect {
            return RedirectDecision::None;
        }

        let mut location = format!("https://{}", self.canonical_hostname);
        if self.preserve_path && !path.is_empty() {
            location.push_str(path);
        }
        if self.preserve_query {
            if let Some(q) = query {
                if !q.is_empty() {
                    location.push('?');
                    location.push_str(q);
                }
            }
        }
        RedirectDecision::Redirect {
            location,
            status: self.status_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> CanonicalPolicy {
        CanonicalPolicy {
            canonical_hostname: "www.example.com".to_owned(),
            redirect_from: vec!["example.com".to_owned()],
            status_code: 308,
            preserve_path: true,
            preserve_query: true,
        }
    }

    #[test]
    fn canonical_host_not_redirected() {
        assert_eq!(
            policy().evaluate("www.example.com", "/x", None),
            RedirectDecision::None
        );
    }

    #[test]
    fn apex_redirects_to_www_with_path_and_query() {
        let d = policy().evaluate("example.com", "/products", Some("page=2"));
        assert_eq!(
            d,
            RedirectDecision::Redirect {
                location: "https://www.example.com/products?page=2".to_owned(),
                status: 308,
            }
        );
    }

    #[test]
    fn unrelated_host_not_redirected() {
        assert_eq!(
            policy().evaluate("other.example.com", "/", None),
            RedirectDecision::None
        );
    }

    #[test]
    fn path_query_can_be_dropped() {
        let p = CanonicalPolicy {
            preserve_path: false,
            preserve_query: false,
            ..policy()
        };
        let d = p.evaluate("example.com", "/x", Some("a=1"));
        assert_eq!(
            d,
            RedirectDecision::Redirect {
                location: "https://www.example.com".to_owned(),
                status: 308,
            }
        );
    }
}
