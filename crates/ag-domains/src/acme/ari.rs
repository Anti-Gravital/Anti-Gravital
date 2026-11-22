//! ACME Renewal Information (ARI, RFC 9773).
//!
//! ARI lets the CA tell clients *when* to renew, instead of clients guessing
//! from `notAfter`. The CA publishes a `RenewalInfo` resource with a suggested
//! renewal window; a well-behaved client schedules renewal inside that window
//! (which also lets the CA spread load and force early renewal during
//! incidents). When ARI is unavailable, the client falls back to the
//! `notAfter`-minus-threshold schedule ([`super::renewal::seconds_until_renewal`]).
//!
//! This module is the deterministic core: parsing the `RenewalInfo` JSON and
//! deciding how long to sleep. The HTTP fetch of the resource is the integration
//! boundary (it needs the live CA and a credentialed environment, exercised by
//! the staging end-to-end test, issue #87); it feeds a [`RenewalInfo`] into
//! [`next_renewal_sleep`].

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

use crate::error::AgDomainsError;

/// The CA's suggested renewal window (RFC 9773 `suggestedWindow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalWindow {
    /// Earliest time the CA suggests renewing.
    pub start: OffsetDateTime,
    /// Latest time the CA suggests renewing.
    pub end: OffsetDateTime,
}

/// A parsed ARI `RenewalInfo` resource (RFC 9773).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalInfo {
    /// The suggested renewal window.
    pub window: RenewalWindow,
    /// Optional human-readable explanation URL (e.g. an incident notice).
    pub explanation_url: Option<String>,
}

// Wire shape: timestamps are parsed as strings and validated as RFC 3339, so we
// do not depend on a date/time crate's serde integration being enabled.
#[derive(Deserialize)]
struct RawRenewalInfo {
    #[serde(rename = "suggestedWindow")]
    suggested_window: RawWindow,
    #[serde(rename = "explanationURL")]
    explanation_url: Option<String>,
}

#[derive(Deserialize)]
struct RawWindow {
    start: String,
    end: String,
}

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime, AgDomainsError> {
    // Parse the explicit offset, then normalize to UTC as the previous chrono
    // implementation did with `with_timezone(&Utc)`.
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|dt| dt.to_offset(UtcOffset::UTC))
        .map_err(|e| AgDomainsError::Acme(format!("invalid ARI timestamp '{value}': {e}")))
}

/// Parses an ARI `RenewalInfo` JSON body (RFC 9773).
pub fn parse_renewal_info(json: &str) -> Result<RenewalInfo, AgDomainsError> {
    let raw: RawRenewalInfo = serde_json::from_str(json)
        .map_err(|e| AgDomainsError::Acme(format!("invalid ARI RenewalInfo JSON: {e}")))?;
    let start = parse_rfc3339(&raw.suggested_window.start)?;
    let end = parse_rfc3339(&raw.suggested_window.end)?;
    if end < start {
        return Err(AgDomainsError::Acme(
            "ARI suggested window ends before it starts".to_owned(),
        ));
    }
    Ok(RenewalInfo {
        window: RenewalWindow { start, end },
        explanation_url: raw.explanation_url,
    })
}

/// Seconds to sleep before renewing, per the ARI suggested window.
///
/// Schedules at the window start: returns the gap until `start`, or `0` if the
/// window has already opened (renew now), which also covers an overdue window.
pub fn seconds_until_ari_renewal(window: &RenewalWindow, now: OffsetDateTime) -> u64 {
    let gap = window.start - now;
    gap.whole_seconds().max(0) as u64
}

/// Combined renewal schedule: prefer ARI when available, else fall back to the
/// `notAfter`-minus-threshold schedule.
pub fn next_renewal_sleep(
    ari: Option<&RenewalInfo>,
    not_after: OffsetDateTime,
    renew_before_days: u64,
    now: OffsetDateTime,
) -> u64 {
    match ari {
        Some(info) => seconds_until_ari_renewal(&info.window, now),
        None => super::renewal::seconds_until_renewal(not_after, renew_before_days, now),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::SignedDuration;

    const SAMPLE: &str = r#"{
        "suggestedWindow": {
            "start": "2025-01-02T04:00:00Z",
            "end": "2025-01-03T04:00:00Z"
        },
        "explanationURL": "https://acme.example/docs/renewal"
    }"#;

    #[test]
    fn parses_renewal_info() {
        let info = parse_renewal_info(SAMPLE).unwrap();
        // RFC-0026 format note: chrono's `to_rfc3339()` spelled UTC as
        // `+00:00`; time's `Rfc3339` formatter uses `Z`. Both are valid RFC 3339,
        // but the exact text is a breaking change for literal comparisons.
        assert_eq!(
            info.window.start.format(&Rfc3339).expect("valid RFC 3339"),
            "2025-01-02T04:00:00Z"
        );
        assert_eq!(
            info.window.end.format(&Rfc3339).expect("valid RFC 3339"),
            "2025-01-03T04:00:00Z"
        );
        assert_eq!(
            info.explanation_url.as_deref(),
            Some("https://acme.example/docs/renewal")
        );
    }

    #[test]
    fn explanation_url_is_optional() {
        let json =
            r#"{"suggestedWindow":{"start":"2025-01-02T04:00:00Z","end":"2025-01-03T04:00:00Z"}}"#;
        let info = parse_renewal_info(json).unwrap();
        assert!(info.explanation_url.is_none());
    }

    #[test]
    fn rejects_garbage_and_inverted_window() {
        assert!(parse_renewal_info("not json").is_err());
        let inverted =
            r#"{"suggestedWindow":{"start":"2025-01-03T04:00:00Z","end":"2025-01-02T04:00:00Z"}}"#;
        assert!(parse_renewal_info(inverted).is_err());
    }

    #[test]
    fn sleeps_until_window_start() {
        let now = OffsetDateTime::now_utc();
        let window = RenewalWindow {
            start: now + SignedDuration::hours(10),
            end: now + SignedDuration::hours(34),
        };
        let secs = seconds_until_ari_renewal(&window, now);
        assert_eq!(secs, 10 * 3600);
    }

    #[test]
    fn zero_when_window_already_open() {
        let now = OffsetDateTime::now_utc();
        let window = RenewalWindow {
            start: now - SignedDuration::hours(1),
            end: now + SignedDuration::hours(23),
        };
        assert_eq!(seconds_until_ari_renewal(&window, now), 0);
    }

    #[test]
    fn next_sleep_prefers_ari_over_not_after() {
        let now = OffsetDateTime::now_utc();
        let not_after = now + SignedDuration::days(60);
        let ari = RenewalInfo {
            window: RenewalWindow {
                start: now + SignedDuration::days(1),
                end: now + SignedDuration::days(2),
            },
            explanation_url: None,
        };
        // ARI says renew in 1 day; notAfter fallback would say ~30 days.
        assert_eq!(next_renewal_sleep(Some(&ari), not_after, 30, now), 86_400);
    }

    #[test]
    fn next_sleep_falls_back_to_not_after_without_ari() {
        let now = OffsetDateTime::now_utc();
        let not_after = now + SignedDuration::days(30);
        // 30 days left, renew 10 days before -> sleep 20 days.
        assert_eq!(next_renewal_sleep(None, not_after, 10, now), 20 * 86_400);
    }
}
