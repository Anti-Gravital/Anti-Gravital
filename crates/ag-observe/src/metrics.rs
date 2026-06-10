//! Standard framework metrics and HTTP /metrics handler.

use axum::{http::StatusCode, response::IntoResponse};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub(crate) fn install_prometheus_recorder() {
    if PROMETHEUS_HANDLE.get().is_some() {
        return;
    }

    if let Ok(handle) = PrometheusBuilder::new().install_recorder() {
        let _ = PROMETHEUS_HANDLE.set(handle);
    }
}

/// Records a completed HTTP request in Prometheus metrics.
///
/// Should be called at the end of each handler, typically from an
/// observability middleware in the Shield.
///
/// # Cardinality
///
/// `endpoint` becomes a metric label. Pass the **matched route pattern**
/// (e.g. `/users/:id`), never the raw request path (`/users/123`). Using the
/// raw path makes label cardinality grow without bound (one series per distinct
/// id), which exhausts memory in Prometheus and the exporter. With Axum, derive
/// it from `MatchedPath`, not `Uri::path`.
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
    let status = if PROMETHEUS_HANDLE.get().is_some() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = render_metrics().unwrap_or_else(|message| format!("{message}\n"));

    (status, prometheus_content_type(), body)
}

fn prometheus_content_type() -> [(&'static str, &'static str); 1] {
    [(
        axum::http::header::CONTENT_TYPE.as_str(),
        "text/plain; version=0.0.4; charset=utf-8",
    )]
}

fn render_metrics() -> Result<String, &'static str> {
    PROMETHEUS_HANDLE
        .get()
        .map(PrometheusHandle::render)
        .ok_or("prometheus exporter is not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_request_does_not_panic() {
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
    fn render_metrics_reports_missing_initialization() {
        if PROMETHEUS_HANDLE.get().is_none() {
            assert_eq!(
                render_metrics(),
                Err("prometheus exporter is not initialized")
            );
        }
    }

    #[test]
    fn render_metrics_includes_recorded_requests_after_init() {
        install_prometheus_recorder();
        record_request("GET", "/health", 200, 0.001);

        let rendered = render_metrics().expect("prometheus exporter should be initialized");
        assert!(
            rendered.contains("ag_requests_total"),
            "rendered metrics should include ag_requests_total, got: {rendered}"
        );
    }
}
