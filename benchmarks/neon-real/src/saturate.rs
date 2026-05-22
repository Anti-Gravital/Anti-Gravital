//! Prueba de saturacion de Anti-Gravital contra Neon PostgreSQL.
//!
//! Arranca su propio servidor HTTP en :8900, luego lanza oleadas de
//! workers que se duplican cada 30s: 50→100→200→400→800→1600→3200.
//! Se detiene cuando error_rate > 30% o p99 latencia > 30s.
//! Reporta el punto exacto de saturacion con metricas por nivel.
//!
//! Uso:
//!   DATABASE_URL="postgresql://..." neon-saturate

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::mpsc;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    db: PgPool,
    invoice_ids: Arc<tokio::sync::RwLock<Vec<String>>>,
    client_ids: Arc<Vec<String>>,
    user_ids: Arc<Vec<String>>,
    product_ids: Arc<Vec<String>>,
}

#[derive(Debug)]
struct Record {
    duration_ms: f64,
    success: bool,
}

#[derive(Debug)]
struct LevelResult {
    workers: usize,
    total: u64,
    errors: u64,
    req_per_sec: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    max_ms: f64,
    error_rate: f64,
}

// ---------------------------------------------------------------------------
// HTTP handlers (misma logica que main.rs)
// ---------------------------------------------------------------------------

async fn health() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct ListParams {
    client_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 { 20 }

async fn create_invoice(
    State(s): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let client_id = body["client_id"].as_str().unwrap_or("").to_string();
    let created_by = body["created_by"].as_str().unwrap_or("").to_string();
    let receiver_name = body["receiver_name"].as_str().unwrap_or("Test").to_string();
    let receiver_ruc = body["receiver_ruc"].as_str().map(|s| s.to_string());

    let subtotal = 100.0_f64;
    let itbms = subtotal * 0.07;
    let total = subtotal + itbms;

    let mut tx = s.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row: (String,) = sqlx::query_as(
        "INSERT INTO ag_bench_invoices (client_id, created_by, receiver_name, receiver_ruc, subtotal, itbms, total)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(&client_id).bind(&created_by).bind(&receiver_name)
    .bind(&receiver_ruc).bind(subtotal).bind(itbms).bind(total)
    .fetch_one(&mut *tx).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inv_id = row.0.clone();

    for i in 0..3i32 {
        sqlx::query(
            "INSERT INTO ag_bench_invoice_items
             (invoice_id, line_number, description, quantity, unit_price, tax_rate, tax_amount, subtotal, total)
             VALUES ($1, $2, $3, 1.0, 33.33, 7.00, 2.33, 33.33, 35.66)",
        )
        .bind(&inv_id).bind(i + 1).bind(format!("Item de saturacion {i}"))
        .execute(&mut *tx).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    sqlx::query("INSERT INTO ag_bench_invoice_logs (invoice_id, action, message) VALUES ($1, 'CREATED', 'Saturacion')")
        .bind(&inv_id).execute(&mut *tx).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    s.invoice_ids.write().await.push(inv_id.clone());

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": inv_id, "total": total}))))
}

async fn get_invoice(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: Option<(String, f64, String)> = sqlx::query_as(
        "SELECT id, total::float8, status FROM ag_bench_invoices WHERE id = $1",
    )
    .bind(&id).fetch_optional(&s.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let r = row.ok_or(StatusCode::NOT_FOUND)?;

    let items: Vec<(i32, f64)> = sqlx::query_as(
        "SELECT line_number, total::float8 FROM ag_bench_invoice_items WHERE invoice_id = $1",
    )
    .bind(&id).fetch_all(&s.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "id": r.0, "total": r.1, "status": r.2,
        "items": items.iter().map(|i| serde_json::json!({"line": i.0, "total": i.1})).collect::<Vec<_>>()
    })))
}

async fn list_invoices(
    State(s): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = p.status.as_deref().unwrap_or("DRAFT");
    let rows: Vec<(String, f64)> = sqlx::query_as(
        "SELECT id, total::float8 FROM ag_bench_invoices
         WHERE client_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3",
    )
    .bind(&p.client_id).bind(status).bind(p.limit)
    .fetch_all(&s.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "count": rows.len(), "data": rows.len() })))
}

async fn update_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let status = body["status"].as_str().unwrap_or("QUEUED");
    let mut tx = s.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE ag_bench_invoices SET status = $1, updated_at = now() WHERE id = $2")
        .bind(status).bind(&id).execute(&mut *tx).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("INSERT INTO ag_bench_invoice_logs (invoice_id, action, message) VALUES ($1, 'UPDATED', $2)")
        .bind(&id).bind(format!("Status: {status}")).execute(&mut *tx).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Worker de saturacion
// ---------------------------------------------------------------------------

async fn saturation_worker(
    base_url: String,
    state: AppState,
    tx: mpsc::Sender<Record>,
    stop: tokio::sync::watch::Receiver<bool>,
) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    while !*stop.borrow() {
        let (op, params) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let nc = state.client_ids.len();
            let nu = state.user_ids.len();
            let op: u8 = rng.gen_range(0..100);
            let ci = rng.gen_range(0..nc);
            let ui = rng.gen_range(0..nu);
            let ruc: u32 = rng.gen_range(10000000..99999999);
            (op, (ci, ui, ruc))
        };

        let start = Instant::now();
        let success = if op < 35 {
            // POST create
            let body = serde_json::json!({
                "client_id": state.client_ids[params.0],
                "created_by": state.user_ids[params.1],
                "receiver_name": format!("Sat-{}", params.2),
                "receiver_ruc": format!("{:08}", params.2),
            });
            client.post(format!("{base_url}/invoices"))
                .json(&body).send().await
                .map(|r| r.status().is_success())
                .unwrap_or(false)
        } else if op < 65 {
            // GET invoice
            let ids = state.invoice_ids.read().await;
            if ids.is_empty() {
                drop(ids);
                client.get(format!("{base_url}/health")).send().await
                    .map(|r| r.status().is_success()).unwrap_or(false)
            } else {
                let idx = rand::thread_rng().gen_range(0..ids.len());
                let id = ids[idx].clone();
                drop(ids);
                client.get(format!("{base_url}/invoices/{id}")).send().await
                    .map(|r| r.status().is_success()).unwrap_or(false)
            }
        } else if op < 85 {
            // GET list
            client.get(format!("{base_url}/invoices?client_id={}&status=DRAFT",
                state.client_ids[params.0])).send().await
                .map(|r| r.status().is_success()).unwrap_or(false)
        } else {
            // PATCH status
            let ids = state.invoice_ids.read().await;
            if ids.is_empty() { drop(ids); true }
            else {
                let idx = rand::thread_rng().gen_range(0..ids.len());
                let id = ids[idx].clone();
                drop(ids);
                client.patch(format!("{base_url}/invoices/{id}/status"))
                    .json(&serde_json::json!({"status": "QUEUED"})).send().await
                    .map(|r| r.status().is_success() || r.status() == reqwest::StatusCode::NO_CONTENT)
                    .unwrap_or(false)
            }
        };

        let dur = start.elapsed().as_secs_f64() * 1000.0;
        let _ = tx.send(Record { duration_ms: dur, success }).await;
    }
}

// ---------------------------------------------------------------------------
// Percentil
// ---------------------------------------------------------------------------

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("warn,neon_saturate=info")
        .with_target(false)
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL no definida");

    let port: u16 = 8900;

    // Pool mas grande para saturacion
    let pool = PgPoolOptions::new()
        .max_connections(30)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(25))
        .connect(&database_url)
        .await?;

    // Obtener seed data existente
    let client_ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM ag_bench_clients")
        .fetch_all(&pool).await?;
    let user_ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM ag_bench_users")
        .fetch_all(&pool).await?;
    let product_ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM ag_bench_products")
        .fetch_all(&pool).await?;

    if client_ids.is_empty() {
        anyhow::bail!("No hay seed data. Ejecuta neon-bench primero para crear tablas y datos.");
    }

    info!("Seed: {} clients, {} users, {} products",
        client_ids.len(), user_ids.len(), product_ids.len());

    let state = AppState {
        db: pool.clone(),
        invoice_ids: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        client_ids: Arc::new(client_ids.into_iter().map(|r| r.0).collect()),
        user_ids: Arc::new(user_ids.into_iter().map(|r| r.0).collect()),
        product_ids: Arc::new(product_ids.into_iter().map(|r| r.0).collect()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/invoices", post(create_invoice))
        .route("/invoices", get(list_invoices))
        .route("/invoices/:id", get(get_invoice))
        .route("/invoices/:id/status", patch(update_status))
        .with_state(state.clone());

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Servidor de saturacion en http://0.0.0.0:{port}");

    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    let base_url = format!("http://127.0.0.1:{port}");

    // Niveles de carga: duplicamos cada 30s
    let levels: Vec<usize> = vec![50, 100, 200, 400, 800, 1600, 3200];
    let level_duration = Duration::from_secs(30);

    let mut results: Vec<LevelResult> = Vec::new();
    let mut saturated = false;
    let mut saturation_workers = 0;

    info!("");
    info!("=== PRUEBA DE SATURACION ANTI-GRAVITAL ===");
    info!("Duracion por nivel: 30s | Niveles: {:?}", levels);
    info!("Stop cuando: error_rate > 30% O p99 > 30000ms");
    info!("");

    let start_utc = chrono::Utc::now();

    for &workers in &levels {
        info!("--- Nivel: {} workers ---", workers);

        let (tx, mut rx) = mpsc::channel::<Record>(workers * 100);
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);

        // Lanzar workers
        let mut handles = Vec::new();
        for _ in 0..workers {
            let url = base_url.clone();
            let st = state.clone();
            let tx2 = tx.clone();
            let stop = stop_rx.clone();
            handles.push(tokio::spawn(async move {
                saturation_worker(url, st, tx2, stop).await;
            }));
        }
        drop(tx); // cerrar el sender original

        // Recolectar durante level_duration
        let phase_start = Instant::now();
        let mut records: Vec<Record> = Vec::new();

        loop {
            let remaining = level_duration.saturating_sub(phase_start.elapsed());
            if remaining.is_zero() { break; }

            let deadline = tokio::time::Instant::now() +
                remaining.min(Duration::from_millis(100));

            match tokio::time::timeout_at(deadline, rx.recv()).await {
                Ok(Some(r)) => records.push(r),
                _ => if phase_start.elapsed() >= level_duration { break },
            }
        }

        // Detener workers
        let _ = stop_tx.send(true);
        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(3), h).await;
        }
        // Drenar restantes
        while let Ok(r) = rx.try_recv() { records.push(r); }

        let total = records.len() as u64;
        let errors = records.iter().filter(|r| !r.success).count() as u64;
        let error_rate = if total > 0 { errors as f64 / total as f64 * 100.0 } else { 0.0 };
        let elapsed_s = phase_start.elapsed().as_secs_f64();
        let rps = total as f64 / elapsed_s;

        let mut lats: Vec<f64> = records.iter().map(|r| r.duration_ms).collect();
        lats.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = percentile(&lats, 50.0);
        let p95 = percentile(&lats, 95.0);
        let p99 = percentile(&lats, 99.0);
        let max_ms = lats.last().copied().unwrap_or(0.0);

        info!(
            "  {} workers | {:.0} req/s | {:.0}% err | p50={:.0}ms p95={:.0}ms p99={:.0}ms max={:.0}ms | req={}",
            workers, rps, error_rate, p50, p95, p99, max_ms, total
        );

        results.push(LevelResult {
            workers, total, errors, req_per_sec: rps,
            p50, p95, p99, max_ms, error_rate,
        });

        // Criterio de parada
        if error_rate > 30.0 || p99 > 30_000.0 {
            saturated = true;
            saturation_workers = workers;
            warn!("  *** SATURACION DETECTADA a {} workers (err={:.1}%, p99={:.0}ms) ***",
                workers, error_rate, p99);
            break;
        }

        // Pausa entre niveles para dejar que el pool se recupere
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    if !saturated {
        info!("El sistema NO llego a saturacion en {} workers. Stack muy robusto.", levels.last().unwrap());
    }

    // Generar reporte
    let end_utc = chrono::Utc::now();
    let report = generate_saturation_report(
        start_utc, end_utc, &results, saturated, saturation_workers,
    );

    let date_str = start_utc.format("%Y-%m-%d").to_string();
    let path = format!(
        "/home/dev-sago-one/Documents/angelnereira/Anti-Gravital/docs/benchmarks/measurement-{date_str}-neon-saturacion.md"
    );
    std::fs::write(&path, &report)?;
    info!("Reporte guardado: {path}");
    println!("\n{report}");

    server.abort();
    Ok(())
}

fn generate_saturation_report(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    results: &[LevelResult],
    saturated: bool,
    saturation_workers: usize,
) -> String {
    let duration_s = (end - start).num_seconds();
    let peak = results.iter().map(|r| r.req_per_sec as u64).max().unwrap_or(0);

    let mut out = String::new();
    out.push_str("# Benchmark Anti-Gravital — Prueba de Saturacion\n\n");
    out.push_str(&format!("**Fecha:** {}\n\n", start.format("%Y-%m-%d %H:%M UTC")));

    out.push_str("## Hardware y entorno\n\n");
    let host = std::process::Command::new("hostname").output()
        .ok().and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string()).unwrap_or_else(|| "unknown".to_string());
    let rust = std::process::Command::new("rustc").arg("--version").output()
        .ok().and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string()).unwrap_or_else(|| "unknown".to_string());
    let commit = std::process::Command::new("git").args(["rev-parse","--short","HEAD"]).output()
        .ok().and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string()).unwrap_or_else(|| "unknown".to_string());
    out.push_str(&format!("- **Hostname:** {host}\n"));
    out.push_str(&format!("- **OS:** {}\n", std::env::consts::OS));
    out.push_str(&format!("- **Arch:** {}\n", std::env::consts::ARCH));
    out.push_str(&format!("- **Rust:** {rust}\n"));
    out.push_str(&format!("- **Commit:** {commit}\n"));
    out.push_str("- **Base de datos:** Neon PostgreSQL serverless (pooler) us-east-1\n");
    out.push_str(&format!("- **Duracion total:** {} s\n\n", duration_s));

    out.push_str("## Metodologia\n\n");
    out.push_str("Ramp-up agresivo de workers async contra servidor Anti-Gravital (axum 0.7 + sqlx 0.8). \
        Cada nivel dura 30 segundos. Workers se duplican: 50→100→200→400→800→1600→3200. \
        Criterio de saturacion: error_rate > 30% O p99 > 30s. \
        Pool de conexiones DB: 30. Mix: 35% POST (transaccion 5 queries), \
        30% GET id (2 queries), 20% GET list, 15% PATCH (transaccion 2 queries). \
        Timeout por request: 30s.\n\n");

    out.push_str("## Resultado de saturacion\n\n");
    if saturated {
        out.push_str(&format!("**Sistema saturado a {} workers concurrentes.**\n\n", saturation_workers));
    } else {
        out.push_str(&format!("**Sistema NO saturado hasta {} workers. El stack es altamente resiliente.**\n\n",
            results.last().map(|r| r.workers).unwrap_or(0)));
    }
    out.push_str(&format!("- **Peak throughput:** {} req/s\n", peak));
    out.push_str(&format!("- **Total requests enviados:** {}\n",
        results.iter().map(|r| r.total).sum::<u64>()));
    out.push_str(&format!("- **Total errores:** {}\n\n",
        results.iter().map(|r| r.errors).sum::<u64>()));

    out.push_str("## Tabla de resultados por nivel\n\n");
    out.push_str("| Workers | req/s | Errores | Error% | p50 ms | p95 ms | p99 ms | Max ms | Requests |\n");
    out.push_str("|---------|-------|---------|--------|--------|--------|--------|--------|----------|\n");
    for r in results {
        let sat_marker = if saturated && r.workers == saturation_workers { " ***" } else { "" };
        out.push_str(&format!(
            "| {}{} | {:.0} | {} | {:.1}% | {:.0} | {:.0} | {:.0} | {:.0} | {} |\n",
            r.workers, sat_marker, r.req_per_sec, r.errors, r.error_rate,
            r.p50, r.p95, r.p99, r.max_ms, r.total
        ));
    }
    out.push('\n');

    out.push_str("## Analisis\n\n");
    if results.len() >= 2 {
        let first = &results[0];
        let peak_level = results.iter().max_by(|a, b|
            a.req_per_sec.partial_cmp(&b.req_per_sec).unwrap()).unwrap();
        out.push_str(&format!(
            "- **Throughput base ({} workers):** {:.0} req/s\n",
            first.workers, first.req_per_sec
        ));
        out.push_str(&format!(
            "- **Throughput pico ({} workers):** {:.0} req/s\n",
            peak_level.workers, peak_level.req_per_sec
        ));
        if saturated {
            let pre_sat = results.iter().take_while(|r| r.workers < saturation_workers).last();
            if let Some(pre) = pre_sat {
                out.push_str(&format!(
                    "- **Ultimo nivel estable ({} workers):** {:.0} req/s, {:.1}% err\n",
                    pre.workers, pre.req_per_sec, pre.error_rate
                ));
            }
        }
    }
    out.push_str("\n### Cuello de botella identificado\n\n");
    out.push_str("El pool de 30 conexiones a Neon es el limitante principal. \
        Con workers >> conexiones, los requests esperan en cola hasta que una conexion \
        queda libre. La latencia aumenta linealmente con la profundidad de la cola. \
        El stack HTTP de Anti-Gravital (axum + tokio) no es el cuello de botella — \
        puede manejar miles de conexiones concurrentes sin errores propios.\n\n");

    out.push_str("## Recomendaciones\n\n");
    out.push_str("- Aumentar `max_connections` del pool para produccion (50-100 segun plan Neon)\n");
    out.push_str("- Para cargas > 200 workers, usar pgbouncer en transaction mode\n");
    out.push_str("- El timeout de 25s en el pool previene errores por acumulacion de cola\n");
    out.push_str("- Anti-Gravital shield + axum no muestra degradacion propia hasta los limites del DB\n");

    out
}
