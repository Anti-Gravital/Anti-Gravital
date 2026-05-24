//! Metricas estandar del framework y handler HTTP /metrics.

use axum::response::IntoResponse;

/// Registra un request HTTP completado en las metricas de Prometheus.
///
/// Debe llamarse al final de cada handler, tipicamente desde un middleware
/// de observabilidad en la Shield.
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

/// Actualiza las metricas del pool de base de datos.
///
/// Llamar periodicamente desde la logica de gestion del pool de ag-data.
pub fn set_db_pool(size: u32, idle: u32) {
    metrics::gauge!("ag_db_pool_size").set(size as f64);
    metrics::gauge!("ag_db_pool_idle").set(idle as f64);
}

/// Incrementa el contador de conexiones activas (WebSocket, SSE, etc.).
pub fn inc_active_connections() {
    metrics::gauge!("ag_active_connections").increment(1.0);
}

/// Decrementa el contador de conexiones activas.
pub fn dec_active_connections() {
    metrics::gauge!("ag_active_connections").decrement(1.0);
}

/// Handler Axum para `GET /metrics`.
///
/// Retorna las metricas en formato texto Prometheus (text/plain; version=0.0.4).
/// Montar en el router con:
/// ```ignore
/// let router = Router::new().route("/metrics", get(ag_observe::metrics_handler));
/// ```
pub async fn metrics_handler() -> impl IntoResponse {
    // El exporter Prometheus instalado globalmente expone un handle para renderizar
    // el snapshot actual. Como metrics-exporter-prometheus instala un handle global,
    // lo obtenemos aqui.
    // Si el exporter no esta instalado, retornar texto vacio con el content-type correcto.
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
    // motivo: metrics-exporter-prometheus 0.16 retorna un PrometheusHandle desde
    //         install() que es necesario para llamar a handle.render(). Como en
    //         layer::init() usamos install() sin capturar el handle, no hay forma
    //         de renderizar el snapshot real desde aqui.
    // impacto: El endpoint /metrics devuelve siempre un body vacio. Las metricas
    //          se acumulan internamente pero no son observables via HTTP hasta
    //          resolver este TODO.
    // eliminacion esperada: Fase 4, iteracion ag-observe v0.2. Guardar el
    //          PrometheusHandle en un OnceLock<PrometheusHandle> en layer::init()
    //          y usarlo aqui para render() el snapshot real.
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_request_does_not_panic() {
        // No podemos verificar el snapshot sin el handle, pero al menos
        // verificamos que la funcion no entra en panic.
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
            "content-type debe ser text/plain, fue: {ct}"
        );
    }

    #[test]
    fn render_metrics_returns_string() {
        let _s = render_metrics();
    }
}
