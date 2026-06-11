//! Registrable domain (eTLD+1) derivation, shared by `hostname` and `issuance`.
//!
//! This is the single source of eTLD+1 truth for the crate (DRY): both
//! [`crate::hostname`] and [`crate::issuance`] derive the registrable domain
//! through [`registrable_domain`], so apex/subdomain classification and the
//! per-registered-domain issuance rate-limit key never disagree.
//!
//! With the `psl` feature it uses the Public Suffix List, which is correct for
//! multi-label public suffixes (`co.uk`, `com.br`, `gov.au`). Without it, it
//! falls back to a best-effort two-label heuristic that is correct only for
//! single-label TLDs (`example.com`) and keeps the default build offline and
//! dependency-free (ADR-0009).

/// Derives the registrable domain (eTLD+1) of a host.
///
/// A leading `*.` wildcard label is stripped first; the host is lowercased and a
/// trailing dot removed. With the `psl` feature the Public Suffix List is
/// authoritative; if it cannot classify the host (an input that is itself a
/// public suffix, or has too few labels) the two-label heuristic is used as a
/// fallback so the function always returns a value.
pub(crate) fn registrable_domain(host: &str) -> String {
    let lowered = host.trim().trim_end_matches('.').to_ascii_lowercase();
    let host = lowered.strip_prefix("*.").unwrap_or(&lowered);

    #[cfg(feature = "psl")]
    {
        if let Some(domain) = psl::domain_str(host) {
            return domain.to_owned();
        }
        // PSL could not derive a registrable domain (the host is itself a public
        // suffix or has too few labels): fall through to the heuristic.
    }

    two_label_heuristic(host)
}

/// Best-effort eTLD+1: keep the last two labels.
///
/// Correct only for single-label public suffixes; the `psl` feature supersedes it
/// for multi-label suffixes. Kept as the offline default and as the fallback when
/// the PSL cannot classify a host.
fn two_label_heuristic(host: &str) -> String {
    let labels: Vec<&str> = host.split('.').filter(|l| !l.is_empty()).collect();
    let n = labels.len();
    if n <= 2 {
        labels.join(".")
    } else {
        labels[n - 2..].join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_label_tld_agrees_in_both_modes() {
        // These cases are correct under the heuristic and the PSL alike.
        assert_eq!(registrable_domain("example.com"), "example.com");
        assert_eq!(registrable_domain("api.example.com"), "example.com");
        assert_eq!(registrable_domain("a.b.c.example.com"), "example.com");
    }

    #[test]
    fn strips_wildcard_and_normalizes() {
        assert_eq!(registrable_domain("*.example.com"), "example.com");
        assert_eq!(registrable_domain("API.Example.COM."), "example.com");
    }

    #[test]
    fn too_few_labels_returns_input() {
        assert_eq!(registrable_domain("localhost"), "localhost");
    }

    #[cfg(feature = "psl")]
    #[test]
    fn multi_label_suffix_is_psl_correct() {
        // The two-label heuristic would wrongly return `co.uk` / `com.br`.
        assert_eq!(registrable_domain("example.co.uk"), "example.co.uk");
        assert_eq!(registrable_domain("www.example.co.uk"), "example.co.uk");
        assert_eq!(registrable_domain("shop.example.com.br"), "example.com.br");
        assert_eq!(registrable_domain("*.example.co.uk"), "example.co.uk");
    }

    #[cfg(not(feature = "psl"))]
    #[test]
    fn multi_label_suffix_heuristic_is_best_effort() {
        // Documented limitation of the offline default: multi-label suffixes
        // collapse to the last two labels. The `psl` feature fixes this.
        assert_eq!(registrable_domain("example.co.uk"), "co.uk");
        assert_eq!(registrable_domain("shop.example.com.br"), "com.br");
    }
}
