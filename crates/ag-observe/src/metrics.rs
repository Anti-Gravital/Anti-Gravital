//! Standard framework metrics and HTTP /metrics handler.

use axum::response::IntoResponse;

/// Records a completed HTTP request in Prometheus metrics.
///
/// Should be called at the end of each handler, typically from an
/// observability middleware in the Shield.
pub fn record_request(method: &str, endpoint: &str, status: u16, duration_secs: f64) {
    metrics::counter!(
        "ag_requests_total",
        "method" => method.to_string(),
        "endpoint" => endpoint.to_string(),
        "status" => status.to_string()
    )
    .increment(1);

    metrics::histogram!(
        "ag_request_duration_seconds",
        "method" => method.to_string(),
        "endpoint" => endpoint.to_string()
    )
    .record(duration_secs);
}

/// Updates database pool metrics.
///
/// Call periodically from the pool management logic in ag-data.
pub fn set_db_pool(size: u32, idle: u32) {
    metrics::gauge!("ag_db_pool_size").set(size as f64);
    metrics::gauge!("ag_db_pool_idle").set(idle as f64);
}

/// Increments the active connections counter (WebSocket, SSE, etc.).
pub fn inc_active_connections() {
    metrics::gauge!("ag_active_connections").increment(1.0);
}

/// Decrements the active connections counter.
pub fn dec_active_connections() {
    metrics::gauge!("ag_active_connections").decrement(1.0);
}

/// Handler Axum para `GET /metrics`.
///
/// Returns metrics in Prometheus text format (text/plain; version=0.0.4).
/// Mount on the router with:
/// ```ignore
/// let router = Router::new().route("/metrics", get(ag_observe::metrics_handler));
/// ```
pub async fn metrics_handler() -> impl IntoResponse {
    // The globally installed Prometheus exporter exposes a handle for rendering
    // the current snapshot. Since metrics-exporter-prometheus installs a global handle,
    // we retrieve it here.
    // If the exporter is not installed, return empty text with the correct content-type.
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        render_metrics(),
    )
}

fn render_metrics() -> String {
    // TECH-DEBT:
    // motivo: metrics-exporter-prometheus 0.16 returns a PrometheusHandle from
    //         install() that is required to call handle.render(). Since in
    //         layer::init() we call install() without capturing the handle, there
    //         is no way to render the real snapshot from here.
    // impacto: The /metrics endpoint always returns an empty body. Metrics
    //          accumulate internally but are not observable via HTTP until
    //          this TODO is resolved.
    // eliminacion esperada: Phase 4, iteration ag-observe v0.2. Store the
    //          PrometheusHandle in a OnceLock<PrometheusHandle> in layer::init()
    //          and use it here to render() the real snapshot.
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_request_does_not_panic() {
        // We cannot verify the snapshot without the handle, but at least
        // verify that the function does not panic.
        record_request("GET", "/health", 200, 0.001);
    }

    #[test]
    fn set_db_pool_does_not_panic() {
        set_db_pool(10, 8);
    }

    #[test]
    fn connection_counters_do_not_panic() {
        inc_active_connections();
        dec_active_connections();
    }

    #[tokio::test]
    async fn metrics_handler_returns_prometheus_content_type() {
        use axum::response::IntoResponse;
        let response = metrics_handler().await.into_response();
        let ct = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/plain"),
            "content-type must be text/plain, got: {ct}"
        );
    }

    #[test]
    fn render_metrics_returns_string() {
        let _s = render_metrics();
    }
}
