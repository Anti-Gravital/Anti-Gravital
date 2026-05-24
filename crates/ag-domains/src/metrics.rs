//! Metricas de `ag-domains` exportadas a `ag-observe`.
//!
//! Registra contadores de operaciones DNS y ACME, e histogramas de
//! latencia de propagacion, usando el crate `metrics`.

use metrics::{counter, histogram};

/// Registra un upsert de registro DNS (creacion o actualizacion).
///
/// `provider` identifica el adapter (e.g., `"cloudflare"`).
/// `record_type` es el tipo de registro (e.g., `"TXT"`, `"A"`).
/// `success` distingue exito de fallo.
pub fn record_dns_upsert(provider: &str, record_type: &str, success: bool) {
    counter!(
        "ag_domains_records_upsert_total",
        "provider" => provider.to_owned(),
        "record_type" => record_type.to_owned(),
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Registra una renovacion ACME completada o fallida.
///
/// `domain` identifica el dominio. `success` distingue exito de fallo.
pub fn record_acme_renewal(domain: &str, success: bool) {
    counter!(
        "ag_domains_acme_renewal_total",
        "domain" => domain.to_owned(),
        "success" => success.to_string(),
    )
    .increment(1);
}

/// Registra la latencia de propagacion DNS en segundos.
///
/// Se llama cuando `PropagationChecker::wait_for_txt` completa
/// (exitosamente o no) para tener visibilidad del tiempo real de
/// propagacion en el entorno de produccion.
pub fn record_propagation_latency(domain: &str, seconds: f64) {
    histogram!(
        "ag_domains_propagation_latency_seconds",
        "domain" => domain.to_owned(),
    )
    .record(seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_do_not_panic() {
        // Las llamadas no deben entrar en panico aunque no haya recorder registrado.
        record_dns_upsert("cloudflare", "TXT", true);
        record_dns_upsert("cloudflare", "A", false);
        record_acme_renewal("ejemplo.com", true);
        record_propagation_latency("ejemplo.com", 12.5);
    }
}
