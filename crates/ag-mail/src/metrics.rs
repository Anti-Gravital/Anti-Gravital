//! `ag-mail` metrics exported to `ag-observe`.
//!
//! Counters: `ag_mail_sent_total`, `ag_mail_retry_total`.
//! Histogram: `ag_mail_send_latency_seconds`.
//! The `provider` and `result` labels allow breaking down by adapter and success/failure.

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};

/// Records a completed send attempt.
///
/// `provider` is the adapter name (e.g., `"smtp"`, `"resend"`).
/// `success` distinguishes success from failure.
pub fn record_send(provider: &str, success: bool) {
    #[cfg(feature = "metrics")]
    {
        let result = if success { "ok" } else { "error" };
        counter!(
            "ag_mail_sent_total",
            "provider" => provider.to_owned(),
            "result" => result,
        )
        .increment(1);
    }
    #[cfg(not(feature = "metrics"))]
    let _ = (provider, success);
}

/// Records a send retry.
pub fn record_retry(provider: &str) {
    #[cfg(feature = "metrics")]
    counter!(
        "ag_mail_retry_total",
        "provider" => provider.to_owned(),
    )
    .increment(1);
    #[cfg(not(feature = "metrics"))]
    let _ = provider;
}

/// Records the latency of a successful send in seconds.
pub fn record_send_latency(provider: &str, seconds: f64) {
    #[cfg(feature = "metrics")]
    histogram!(
        "ag_mail_send_latency_seconds",
        "provider" => provider.to_owned(),
    )
    .record(seconds);
    #[cfg(not(feature = "metrics"))]
    let _ = (provider, seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_do_not_panic() {
        record_send("smtp", true);
        record_send("resend", false);
        record_retry("smtp");
        record_send_latency("resend", 0.42);
    }
}
