//! `ag-domains` metrics exported to `ag-observe`.
//!
//! Records counters of DNS and ACME operations, and histograms of
//! propagation latency, using the `metrics` crate.

use metrics::{counter, gauge, histogram};

/// Records a DNS record upsert (creation or update).
///
/// `provider` identifies the adapter (e.g., `"cloudflare"`).
/// `record_type` is the record type (e.g., `"TXT"`, `"A"`).
/// `success` distinguishes success from failure.
pub fn record_dns_upsert(provider: &str, record_type: &str, success: bool) {
    counter!(
        "ag_domains_records_upsert_total",
        "provider" => provider.to_owned(),
        "record_type" => record_type.to_owned(),
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Records a completed or failed ACME renewal.
///
/// `domain` identifies the domain. `success` distinguishes success from failure.
pub fn record_acme_renewal(domain: &str, success: bool) {
    counter!(
        "ag_domains_acme_renewal_total",
        "domain" => domain.to_owned(),
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Reports the days remaining until the TLS certificate expires.
///
/// Called after each successful renewal so dashboards show how long
/// before the next renewal is due. `days` is 0 when expired.
pub fn set_cert_days_until_expiry(domain: &str, days: i64) {
    gauge!(
        "ag_domains_cert_days_until_expiry",
        "domain" => domain.to_owned(),
    )
    .set(days as f64);
}

/// Records the DNS propagation latency in seconds.
///
/// Called when `PropagationChecker::wait_for_txt` completes
/// (successfully or not) to have visibility into the real
/// propagation time in the production environment.
pub fn record_propagation_latency(domain: &str, seconds: f64) {
    histogram!(
        "ag_domains_propagation_latency_seconds",
        "domain" => domain.to_owned(),
    )
    .record(seconds);
}

/// Counts a created domain attachment (control plane, RFC-0011).
pub fn record_attachment_created() {
    counter!("ag_domains_attachments_total").increment(1);
}

/// Counts a detached domain attachment.
pub fn record_attachment_detached() {
    counter!("ag_domains_detached_total").increment(1);
}

/// Counts an ownership verification failure.
pub fn record_verification_failure() {
    counter!("ag_domains_verification_failures_total").increment(1);
}

/// Reports the number of currently active domain attachments (blueprint
/// section 16.1). Set from a fleet view (e.g. after a control-plane mutation
/// or list), counting attachments whose four readiness dimensions are met.
pub fn set_active_attachments(count: u64) {
    gauge!("ag_domains_attachments_active").set(count as f64);
}

/// Reports the number of certificates inside the renewal window (blueprint
/// section 16.1). Derived from attachments in the `RenewalDue` TLS state.
pub fn set_certs_expiring_soon(count: u64) {
    gauge!("ag_domains_certs_expiring_soon").set(count as f64);
}

/// Counts a TLS certificate order (blueprint section 16.1).
///
/// `success` separates completed orders from failures, covering both the
/// "orders" and "failures" dimensions on a single counter.
pub fn record_tls_order(success: bool) {
    counter!(
        "ag_domains_tls_orders_total",
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Counts a transition of an attachment into a DNS-misconfigured state
/// (blueprint section 16.1): wrong target, conflicting records or failed DNS.
pub fn record_dns_misconfigured() {
    counter!("ag_domains_dns_misconfigured_total").increment(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_do_not_panic() {
        // The calls must not panic even when no recorder is registered.
        record_dns_upsert("cloudflare", "TXT", true);
        record_dns_upsert("cloudflare", "A", false);
        record_acme_renewal("ejemplo.com", true);
        record_propagation_latency("ejemplo.com", 12.5);
        record_attachment_created();
        record_attachment_detached();
        record_verification_failure();
        set_active_attachments(3);
        set_certs_expiring_soon(1);
        record_tls_order(true);
        record_tls_order(false);
        record_dns_misconfigured();
    }
}
