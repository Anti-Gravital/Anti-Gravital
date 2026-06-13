//! Hostname normalization, classification and validation.
//!
//! Identity is the lowercase ASCII (Punycode / A-label) form. The Unicode
//! display form is preserved separately so internationalized domains render
//! correctly without breaking identity comparisons (CLAUDE.md security rule
//! 16; ADR-0012 native-first). Wildcard and apex/subdomain classification feed
//! the DNS instruction engine and the edge router.
//!
//! # Examples
//!
//! ```
//! use ag_domains::hostname::{Hostname, HostnameKind};
//!
//! let apex = Hostname::parse("Example.com").unwrap();
//! assert_eq!(apex.ascii(), "example.com");
//! assert_eq!(apex.kind(), HostnameKind::Apex);
//!
//! let sub = Hostname::parse("api.example.com").unwrap();
//! assert_eq!(sub.kind(), HostnameKind::Subdomain);
//!
//! let wild = Hostname::parse("*.example.com").unwrap();
//! assert_eq!(wild.kind(), HostnameKind::Wildcard);
//! ```

use serde::{Deserialize, Serialize};

/// Reasons a hostname input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HostnameError {
    /// Input was empty after trimming.
    Empty,
    /// Input contained a scheme, path, port, query or fragment.
    NotBareHostname(String),
    /// Input contained whitespace.
    Whitespace,
    /// The wildcard pattern is not a single leading `*.` label.
    InvalidWildcard(String),
    /// Punycode / IDNA conversion failed.
    Idna(String),
    /// The hostname has fewer than two labels (no registrable domain).
    TooFewLabels(String),
}

impl std::fmt::Display for HostnameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostnameError::Empty => write!(f, "empty hostname"),
            HostnameError::NotBareHostname(s) => {
                write!(
                    f,
                    "expected a bare hostname, got '{s}' (no scheme/path/port/query)"
                )
            }
            HostnameError::Whitespace => write!(f, "hostname contains whitespace"),
            HostnameError::InvalidWildcard(s) => {
                write!(
                    f,
                    "invalid wildcard pattern '{s}' (only a single leading '*.' is allowed)"
                )
            }
            HostnameError::Idna(s) => write!(f, "IDNA/Punycode conversion failed: {s}"),
            HostnameError::TooFewLabels(s) => {
                write!(
                    f,
                    "hostname '{s}' has no registrable domain (needs at least two labels)"
                )
            }
        }
    }
}

impl std::error::Error for HostnameError {}

/// Classification of a hostname relative to its registrable domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostnameKind {
    /// The registrable domain itself, e.g. `example.com`.
    Apex,
    /// A label under the registrable domain, e.g. `api.example.com`.
    Subdomain,
    /// A wildcard, e.g. `*.example.com`.
    Wildcard,
}

/// A validated, normalized hostname.
///
/// The ASCII form is the canonical identity; the Unicode form is for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hostname {
    ascii: String,
    unicode: String,
    kind: HostnameKind,
    registered_domain: String,
}

impl Hostname {
    /// Parses and normalizes a bare hostname.
    ///
    /// Rejects schemes, paths, ports, queries, fragments, whitespace and
    /// multi-label wildcards. Lowercases and converts to Punycode for identity.
    pub fn parse(input: &str) -> Result<Self, HostnameError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(HostnameError::Empty);
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(HostnameError::Whitespace);
        }
        if trimmed.contains("://")
            || trimmed.contains('/')
            || trimmed.contains('?')
            || trimmed.contains('#')
            || trimmed.contains(':')
        {
            return Err(HostnameError::NotBareHostname(trimmed.to_owned()));
        }

        let (is_wildcard, domain_part) = if let Some(rest) = trimmed.strip_prefix("*.") {
            // Reject any further wildcard labels such as `*.*.example.com`.
            if rest.contains('*') {
                return Err(HostnameError::InvalidWildcard(trimmed.to_owned()));
            }
            (true, rest)
        } else {
            if trimmed.contains('*') {
                return Err(HostnameError::InvalidWildcard(trimmed.to_owned()));
            }
            (false, trimmed)
        };

        let ascii_domain =
            idna::domain_to_ascii(domain_part).map_err(|e| HostnameError::Idna(e.to_string()))?;
        let (unicode_domain, _) = idna::domain_to_unicode(domain_part);

        let label_count = ascii_domain.split('.').filter(|l| !l.is_empty()).count();
        if label_count < 2 {
            return Err(HostnameError::TooFewLabels(trimmed.to_owned()));
        }

        let registered_domain = crate::registrable::registrable_domain(&ascii_domain);

        let (ascii, unicode, kind) = if is_wildcard {
            (
                format!("*.{ascii_domain}"),
                format!("*.{unicode_domain}"),
                HostnameKind::Wildcard,
            )
        } else if ascii_domain == registered_domain {
            (ascii_domain.clone(), unicode_domain, HostnameKind::Apex)
        } else {
            (
                ascii_domain.clone(),
                unicode_domain,
                HostnameKind::Subdomain,
            )
        };

        Ok(Self {
            ascii,
            unicode,
            kind,
            registered_domain,
        })
    }

    /// Canonical ASCII identity form (lowercase, Punycode).
    pub fn ascii(&self) -> &str {
        &self.ascii
    }

    /// Unicode display form.
    pub fn unicode(&self) -> &str {
        &self.unicode
    }

    /// Classification (apex / subdomain / wildcard).
    pub fn kind(&self) -> HostnameKind {
        self.kind
    }

    /// Registrable domain (eTLD+1).
    ///
    /// PSL-correct with the `psl` feature; a best-effort two-label heuristic
    /// otherwise (see [`crate::registrable`]).
    pub fn registered_domain(&self) -> &str {
        &self.registered_domain
    }

    /// The label below the registrable domain, if any.
    ///
    /// `api.example.com` -> `Some("api")`; `example.com` -> `None`;
    /// `*.example.com` -> `Some("*")`.
    pub fn leaf_label(&self) -> Option<&str> {
        match self.kind {
            HostnameKind::Apex => None,
            HostnameKind::Wildcard => Some("*"),
            HostnameKind::Subdomain => self.ascii.split('.').next(),
        }
    }
}

impl std::fmt::Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ascii)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apex_is_lowercased() {
        let h = Hostname::parse("Example.COM").unwrap();
        assert_eq!(h.ascii(), "example.com");
        assert_eq!(h.kind(), HostnameKind::Apex);
        assert_eq!(h.registered_domain(), "example.com");
        assert_eq!(h.leaf_label(), None);
    }

    #[test]
    fn subdomain_classified() {
        let h = Hostname::parse("api.example.com").unwrap();
        assert_eq!(h.kind(), HostnameKind::Subdomain);
        assert_eq!(h.registered_domain(), "example.com");
        assert_eq!(h.leaf_label(), Some("api"));
    }

    #[test]
    fn deep_subdomain_leaf_label() {
        let h = Hostname::parse("a.b.example.com").unwrap();
        assert_eq!(h.kind(), HostnameKind::Subdomain);
        assert_eq!(h.leaf_label(), Some("a"));
    }

    #[test]
    fn wildcard_classified() {
        let h = Hostname::parse("*.example.com").unwrap();
        assert_eq!(h.ascii(), "*.example.com");
        assert_eq!(h.kind(), HostnameKind::Wildcard);
        assert_eq!(h.leaf_label(), Some("*"));
    }

    #[test]
    fn idn_to_punycode() {
        let h = Hostname::parse("münchen.de").unwrap();
        assert_eq!(h.ascii(), "xn--mnchen-3ya.de");
        assert_eq!(h.unicode(), "münchen.de");
    }

    #[test]
    fn rejects_scheme_path_port() {
        assert!(matches!(
            Hostname::parse("https://example.com"),
            Err(HostnameError::NotBareHostname(_))
        ));
        assert!(matches!(
            Hostname::parse("example.com/path"),
            Err(HostnameError::NotBareHostname(_))
        ));
        assert!(matches!(
            Hostname::parse("example.com:443"),
            Err(HostnameError::NotBareHostname(_))
        ));
    }

    #[test]
    fn rejects_bad_wildcard() {
        assert!(matches!(
            Hostname::parse("*.*.example.com"),
            Err(HostnameError::InvalidWildcard(_))
        ));
        assert!(matches!(
            Hostname::parse("foo.*.example.com"),
            Err(HostnameError::InvalidWildcard(_))
        ));
    }

    #[test]
    fn rejects_whitespace_and_empty() {
        assert!(matches!(
            Hostname::parse("exa mple.com"),
            Err(HostnameError::Whitespace)
        ));
        assert!(matches!(Hostname::parse("   "), Err(HostnameError::Empty)));
    }

    #[test]
    fn rejects_single_label() {
        assert!(matches!(
            Hostname::parse("localhost"),
            Err(HostnameError::TooFewLabels(_))
        ));
    }

    #[test]
    fn mixed_case_identity_matches() {
        let a = Hostname::parse("API.Example.com").unwrap();
        let b = Hostname::parse("api.example.com").unwrap();
        assert_eq!(a.ascii(), b.ascii());
    }
}
