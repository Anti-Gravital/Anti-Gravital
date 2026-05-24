//! Metricas de `ag-mail` exportadas a `ag-observe`.
//!
//! Contadores: `ag_mail_sent_total`, `ag_mail_retry_total`.
//! Histograma: `ag_mail_send_latency_seconds`.
//! Las etiquetas `provider` y `result` permiten desagregar por adapter y exito/fallo.

#[cfg(feature = "metrics")]
use metrics::{counter, histogram};

/// Registra un intento de envio completado.
///
/// `provider` es el nombre del adapter (e.g., `"smtp"`, `"resend"`).
/// `success` distingue exito de fallo.
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

/// Registra un reintento de envio.
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

/// Registra la latencia de un envio exitoso en segundos.
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
