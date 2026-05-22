//! Benchmark de rendimiento Anti-Gravital contra Neon PostgreSQL.
//!
//! Simula un servicio de facturacion electronica con operaciones CRUD reales.
//! Ejecuta 7 fases de carga (cold start → pico 120 workers → cooldown)
//! durante 2 horas y genera un reporte Markdown con metricas detalladas.
//!
//! Uso:
//!   DATABASE_URL="postgresql://..." neon-bench [--duration 7200] [--port 8899]

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
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Tipos compartidos
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    db: PgPool,
    seed: SeedData,
}

#[derive(Clone, Debug)]
struct SeedData {
    client_ids: Vec<String>,
    user_ids: Vec<String>,
    customer_ids: Vec<String>,
    product_ids: Vec<String>,
    /// IDs de facturas creadas durante el benchmark (para GETs y PATCHes)
    invoice_ids: Arc<tokio::sync::RwLock<Vec<String>>>,
}

/// Registro de una operacion completada que el worker envia al colector.
#[derive(Debug)]
struct RequestRecord {
    op: Op,
    duration_ms: f64,
    success: bool,
    phase: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Op {
    CreateInvoice,
    GetInvoice,
    ListInvoices,
    UpdateStatus,
    HealthCheck,
}

impl Op {
    fn label(self) -> &'static str {
        match self {
            Op::CreateInvoice => "POST /invoices",
            Op::GetInvoice => "GET  /invoices/:id",
            Op::ListInvoices => "GET  /invoices",
            Op::UpdateStatus => "PATCH /invoices/:id/status",
            Op::HealthCheck => "GET  /health",
        }
    }
}

// ---------------------------------------------------------------------------
// DDL + seed
// ---------------------------------------------------------------------------

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS ag_bench_clients (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    slug        TEXT        NOT NULL UNIQUE,
    name        TEXT        NOT NULL,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ag_bench_users (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    email       TEXT        NOT NULL UNIQUE,
    name        TEXT        NOT NULL,
    client_id   TEXT        NOT NULL REFERENCES ag_bench_clients(id) ON DELETE CASCADE,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ag_bench_customers (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    client_id   TEXT        NOT NULL REFERENCES ag_bench_clients(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    ruc         TEXT        NOT NULL,
    dv          TEXT        NOT NULL,
    email       TEXT,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ag_bench_products (
    id              TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    client_id       TEXT        NOT NULL REFERENCES ag_bench_clients(id) ON DELETE CASCADE,
    sku             TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    sale_price      NUMERIC(12,2) NOT NULL DEFAULT 0,
    tax_rate        NUMERIC(5,2)  NOT NULL DEFAULT 7,
    current_stock   NUMERIC(12,4) NOT NULL DEFAULT 100,
    is_active       BOOLEAN     NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ag_bench_invoices (
    id                  TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    client_id           TEXT        NOT NULL REFERENCES ag_bench_clients(id) ON DELETE CASCADE,
    created_by          TEXT        NOT NULL REFERENCES ag_bench_users(id),
    client_reference_id TEXT        NOT NULL UNIQUE DEFAULT gen_random_uuid()::text,
    receiver_name       TEXT        NOT NULL,
    receiver_ruc        TEXT,
    subtotal            NUMERIC(12,2) NOT NULL DEFAULT 0,
    itbms               NUMERIC(12,2) NOT NULL DEFAULT 0,
    total               NUMERIC(12,2) NOT NULL DEFAULT 0,
    status              TEXT        NOT NULL DEFAULT 'DRAFT',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ag_bench_invoice_items (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    invoice_id  TEXT        NOT NULL REFERENCES ag_bench_invoices(id) ON DELETE CASCADE,
    product_id  TEXT        REFERENCES ag_bench_products(id) ON DELETE SET NULL,
    line_number INT         NOT NULL,
    description TEXT        NOT NULL,
    quantity    NUMERIC(12,4) NOT NULL,
    unit_price  NUMERIC(12,2) NOT NULL,
    tax_rate    NUMERIC(5,2)  NOT NULL DEFAULT 7,
    tax_amount  NUMERIC(12,2) NOT NULL,
    subtotal    NUMERIC(12,2) NOT NULL,
    total       NUMERIC(12,2) NOT NULL
);

CREATE TABLE IF NOT EXISTS ag_bench_invoice_logs (
    id          TEXT        PRIMARY KEY DEFAULT gen_random_uuid()::text,
    invoice_id  TEXT        NOT NULL REFERENCES ag_bench_invoices(id) ON DELETE CASCADE,
    action      TEXT        NOT NULL,
    message     TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ag_bench_invoices_client_status
    ON ag_bench_invoices(client_id, status);
CREATE INDEX IF NOT EXISTS idx_ag_bench_invoices_created
    ON ag_bench_invoices(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ag_bench_items_invoice
    ON ag_bench_invoice_items(invoice_id);
CREATE INDEX IF NOT EXISTS idx_ag_bench_logs_invoice
    ON ag_bench_invoice_logs(invoice_id);
CREATE INDEX IF NOT EXISTS idx_ag_bench_users_client
    ON ag_bench_users(client_id);
CREATE INDEX IF NOT EXISTS idx_ag_bench_products_client
    ON ag_bench_products(client_id);
"#;

async fn setup_schema(pool: &PgPool) -> anyhow::Result<()> {
    for stmt in DDL.split(';') {
        let s = stmt.trim();
        if !s.is_empty() {
            sqlx::query(s).execute(pool).await?;
        }
    }
    info!("Schema listo.");
    Ok(())
}

async fn seed(pool: &PgPool) -> anyhow::Result<SeedData> {
    info!("Sembrando datos de prueba...");

    // 3 clientes
    let mut client_ids = Vec::new();
    for i in 0..3usize {
        let id: (String,) = sqlx::query_as(
            "INSERT INTO ag_bench_clients (slug, name) VALUES ($1, $2)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(format!("bench-client-{i}"))
        .bind(format!("Empresa Benchmark {i}"))
        .fetch_one(pool)
        .await?;
        client_ids.push(id.0);
    }

    // 5 usuarios por cliente
    let mut user_ids = Vec::new();
    for (ci, cid) in client_ids.iter().enumerate() {
        for ui in 0..5usize {
            let email = format!("bench.user.{ci}.{ui}@test.ag");
            let id: (String,) = sqlx::query_as(
                "INSERT INTO ag_bench_users (email, name, client_id) VALUES ($1, $2, $3)
                 ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
                 RETURNING id",
            )
            .bind(&email)
            .bind(format!("Usuario {ci}-{ui}"))
            .bind(cid)
            .fetch_one(pool)
            .await?;
            user_ids.push(id.0);
        }
    }

    // 10 clientes (customers) por tenant
    let mut customer_ids = Vec::new();
    for (ci, cid) in client_ids.iter().enumerate() {
        for ki in 0..10usize {
            let id: (String,) = sqlx::query_as(
                "INSERT INTO ag_bench_customers (client_id, name, ruc, dv, email)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT DO NOTHING
                 RETURNING id",
            )
            .bind(cid)
            .bind(format!("Cliente Fiscal {ci}-{ki}"))
            .bind(format!("{:08}", (ci * 100 + ki) as u32))
            .bind("1")
            .bind(format!("fiscal.{ci}.{ki}@ejemplo.com"))
            .fetch_optional(pool)
            .await?
            .unwrap_or_else(|| {
                (format!("placeholder-{ci}-{ki}"),)
            });
            if !id.0.starts_with("placeholder") {
                customer_ids.push(id.0);
            }
        }
    }
    // Si hubo conflictos, re-fetch los IDs
    if customer_ids.is_empty() {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT id FROM ag_bench_customers LIMIT 30")
                .fetch_all(pool)
                .await?;
        customer_ids = rows.into_iter().map(|r| r.0).collect();
    }

    // 20 productos por tenant
    let mut product_ids = Vec::new();
    for (ci, cid) in client_ids.iter().enumerate() {
        for pi in 0..20usize {
            let id: (String,) = sqlx::query_as(
                "INSERT INTO ag_bench_products (client_id, sku, name, sale_price, current_stock)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT DO NOTHING
                 RETURNING id",
            )
            .bind(cid)
            .bind(format!("SKU-{ci}-{pi:04}"))
            .bind(format!("Producto Benchmark {ci}-{pi}"))
            .bind(10.0 + pi as f64 * 2.5)
            .bind(500.0_f64)
            .fetch_optional(pool)
            .await?
            .unwrap_or_else(|| (format!("placeholder-{ci}-{pi}"),));
            if !id.0.starts_with("placeholder") {
                product_ids.push(id.0);
            }
        }
    }
    if product_ids.is_empty() {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT id FROM ag_bench_products LIMIT 60")
                .fetch_all(pool)
                .await?;
        product_ids = rows.into_iter().map(|r| r.0).collect();
    }

    info!(
        "Seed completo: {} clientes, {} usuarios, {} customers, {} productos",
        client_ids.len(),
        user_ids.len(),
        customer_ids.len(),
        product_ids.len()
    );

    Ok(SeedData {
        client_ids,
        user_ids,
        customer_ids,
        product_ids,
        invoice_ids: Arc::new(tokio::sync::RwLock::new(Vec::new())),
    })
}

// ---------------------------------------------------------------------------
// Handlers HTTP
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct CreateInvoiceReq {
    client_id: String,
    created_by: String,
    receiver_name: String,
    receiver_ruc: Option<String>,
    items: Vec<CreateItemReq>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CreateItemReq {
    product_id: Option<String>,
    description: String,
    quantity: f64,
    unit_price: f64,
}

#[derive(Serialize)]
struct InvoiceResp {
    id: String,
    client_id: String,
    receiver_name: String,
    subtotal: f64,
    itbms: f64,
    total: f64,
    status: String,
    items: Vec<ItemResp>,
}

#[derive(Serialize)]
struct ItemResp {
    id: String,
    line_number: i32,
    description: String,
    quantity: f64,
    unit_price: f64,
    total: f64,
}

#[derive(Deserialize)]
struct ListParams {
    client_id: String,
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    20
}

#[derive(Deserialize)]
struct UpdateStatusReq {
    status: String,
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn create_invoice(
    State(s): State<AppState>,
    Json(req): Json<CreateInvoiceReq>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let mut tx = s.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tax_rate = 0.07_f64;
    let subtotal: f64 = req.items.iter().map(|i| i.quantity * i.unit_price).sum();
    let itbms = subtotal * tax_rate;
    let total = subtotal + itbms;

    let invoice_id: (String,) = sqlx::query_as(
        "INSERT INTO ag_bench_invoices
         (client_id, created_by, receiver_name, receiver_ruc, subtotal, itbms, total)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(&req.client_id)
    .bind(&req.created_by)
    .bind(&req.receiver_name)
    .bind(&req.receiver_ruc)
    .bind(subtotal)
    .bind(itbms)
    .bind(total)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inv_id = invoice_id.0.clone();

    for (i, item) in req.items.iter().enumerate() {
        let item_sub = item.quantity * item.unit_price;
        let item_tax = item_sub * tax_rate;
        let item_total = item_sub + item_tax;
        sqlx::query(
            "INSERT INTO ag_bench_invoice_items
             (invoice_id, product_id, line_number, description, quantity, unit_price,
              tax_rate, tax_amount, subtotal, total)
             VALUES ($1, $2, $3, $4, $5, $6, 7.00, $7, $8, $9)",
        )
        .bind(&inv_id)
        .bind(&item.product_id)
        .bind(i as i32 + 1)
        .bind(&item.description)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(item_tax)
        .bind(item_sub)
        .bind(item_total)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    sqlx::query(
        "INSERT INTO ag_bench_invoice_logs (invoice_id, action, message) VALUES ($1, 'CREATED', 'Factura creada')",
    )
    .bind(&inv_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Registrar ID para usar en GETs posteriores
    s.seed.invoice_ids.write().await.push(inv_id.clone());

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": inv_id, "total": total, "status": "DRAFT" })),
    ))
}

async fn get_invoice(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: Option<(String, String, String, f64, f64, f64, String)> = sqlx::query_as(
        "SELECT id, client_id, receiver_name,
                subtotal::float8, itbms::float8, total::float8, status
         FROM ag_bench_invoices WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&s.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inv = row.ok_or(StatusCode::NOT_FOUND)?;

    let items: Vec<(String, i32, String, f64, f64, f64)> = sqlx::query_as(
        "SELECT id, line_number, description, quantity::float8, unit_price::float8, total::float8
         FROM ag_bench_invoice_items WHERE invoice_id = $1 ORDER BY line_number",
    )
    .bind(&id)
    .fetch_all(&s.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = serde_json::json!({
        "id": inv.0,
        "client_id": inv.1,
        "receiver_name": inv.2,
        "subtotal": inv.3,
        "itbms": inv.4,
        "total": inv.5,
        "status": inv.6,
        "items": items.into_iter().map(|it| serde_json::json!({
            "id": it.0,
            "line_number": it.1,
            "description": it.2,
            "quantity": it.3,
            "unit_price": it.4,
            "total": it.5,
        })).collect::<Vec<_>>(),
    });

    Ok(Json(resp))
}

async fn list_invoices(
    State(s): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = params.status.as_deref().unwrap_or("DRAFT");
    let rows: Vec<(String, String, f64, String)> = sqlx::query_as(
        "SELECT id, receiver_name, total::float8, status
         FROM ag_bench_invoices
         WHERE client_id = $1 AND status = $2
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(&params.client_id)
    .bind(status)
    .bind(params.limit)
    .fetch_all(&s.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| serde_json::json!({ "id": r.0, "receiver_name": r.1, "total": r.2, "status": r.3 }))
        .collect();

    Ok(Json(serde_json::json!({ "data": data, "count": data.len() })))
}

async fn update_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStatusReq>,
) -> Result<StatusCode, StatusCode> {
    let mut tx = s.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let affected = sqlx::query(
        "UPDATE ag_bench_invoices SET status = $1, updated_at = now() WHERE id = $2",
    )
    .bind(&req.status)
    .bind(&id)
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .rows_affected();

    if affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    sqlx::query(
        "INSERT INTO ag_bench_invoice_logs (invoice_id, action, message) VALUES ($1, 'UPDATED', $2)",
    )
    .bind(&id)
    .bind(format!("Estado cambiado a {}", req.status))
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Metricas
// ---------------------------------------------------------------------------

#[derive(Default)]
struct PhaseStats {
    counts: HashMap<Op, u64>,
    errors: HashMap<Op, u64>,
    latencies: HashMap<Op, Vec<f64>>,
}

impl PhaseStats {
    fn record(&mut self, r: &RequestRecord) {
        *self.counts.entry(r.op).or_insert(0) += 1;
        if !r.success {
            *self.errors.entry(r.op).or_insert(0) += 1;
        }
        self.latencies.entry(r.op).or_default().push(r.duration_ms);
    }

    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn summary(&self) -> Vec<OpSummary> {
        let mut out = Vec::new();
        for (op, count) in &self.counts {
            let errs = *self.errors.get(op).unwrap_or(&0);
            let mut lats = self.latencies.get(op).cloned().unwrap_or_default();
            lats.sort_by(|a, b| a.partial_cmp(b).unwrap());
            out.push(OpSummary {
                op: *op,
                total: *count,
                errors: errs,
                p50: Self::percentile(&lats, 50.0),
                p95: Self::percentile(&lats, 95.0),
                p99: Self::percentile(&lats, 99.0),
                max: lats.last().copied().unwrap_or(0.0),
                mean: if lats.is_empty() {
                    0.0
                } else {
                    lats.iter().sum::<f64>() / lats.len() as f64
                },
            });
        }
        out.sort_by(|a, b| a.op.label().cmp(b.op.label()));
        out
    }
}

#[derive(Debug)]
struct OpSummary {
    op: Op,
    total: u64,
    errors: u64,
    p50: f64,
    p95: f64,
    p99: f64,
    max: f64,
    mean: f64,
}

#[derive(Debug)]
struct PhaseSummary {
    name: &'static str,
    workers: usize,
    duration_s: u64,
    ops: Vec<OpSummary>,
    total_req: u64,
    total_err: u64,
    req_per_sec: f64,
}

// ---------------------------------------------------------------------------
// Fases del benchmark
// ---------------------------------------------------------------------------

struct Phase {
    name: &'static str,
    workers: usize,
    duration_s: u64,
}

fn phases() -> Vec<Phase> {
    vec![
        Phase { name: "cold-start",     workers: 5,   duration_s: 300  }, // 0-5 min
        Phase { name: "low-load",       workers: 20,  duration_s: 900  }, // 5-20 min
        Phase { name: "normal",         workers: 40,  duration_s: 1800 }, // 20-50 min
        Phase { name: "peak",           workers: 80,  duration_s: 1800 }, // 50-80 min
        Phase { name: "max-stress",     workers: 120, duration_s: 900  }, // 80-95 min
        Phase { name: "cooldown",       workers: 25,  duration_s: 900  }, // 95-110 min
        Phase { name: "sustained",      workers: 50,  duration_s: 600  }, // 110-120 min
    ]
}

// ---------------------------------------------------------------------------
// Worker
// ---------------------------------------------------------------------------

async fn run_worker(
    worker_id: usize,
    phase_id: u8,
    seed: SeedData,
    base_url: String,
    tx: mpsc::Sender<RequestRecord>,
    stop: tokio::sync::watch::Receiver<bool>,
) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("reqwest client");

    while !*stop.borrow() {
        // Generar valores aleatorios en contexto sincrono (sin await)
        let (op, params) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let nc = seed.client_ids.len();
            let nu = seed.user_ids.len();
            let np = seed.product_ids.len();
            let op_roll: u8 = rng.gen_range(0..100);
            let op = if op_roll < 35 { Op::CreateInvoice }
                else if op_roll < 65 { Op::GetInvoice }
                else if op_roll < 85 { Op::ListInvoices }
                else { Op::UpdateStatus };
            let params = WorkerParams {
                client_idx: rng.gen_range(0..nc),
                user_idx: rng.gen_range(0..nu),
                prod_idx: [rng.gen_range(0..np), rng.gen_range(0..np), rng.gen_range(0..np)],
                recv_num: rng.gen_range(1000u32..9999),
                ruc_num: rng.gen_range(10000000u32..99999999u32),
                qty: [rng.gen_range(1u32..10), 1, rng.gen_range(1u32..5)],
                price: [
                    rng.gen_range(100u32..5000) as f64 / 100.0,
                    rng.gen_range(5000u32..50000) as f64 / 100.0,
                    rng.gen_range(2000u32..10000) as f64 / 100.0,
                ],
                status_idx: rng.gen_range(0usize..4),
                list_status_idx: rng.gen_range(0usize..4),
            };
            (op, params)
        };

        let start = Instant::now();
        let success = match op {
            Op::CreateInvoice => do_create_invoice(&client, &base_url, &seed, &params).await,
            Op::GetInvoice => do_get_invoice(&client, &base_url, &seed).await,
            Op::ListInvoices => do_list_invoices(&client, &base_url, &seed, &params).await,
            Op::UpdateStatus => do_update_status(&client, &base_url, &seed, &params).await,
            Op::HealthCheck => unreachable!(),
        };

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let _ = tx
            .send(RequestRecord { op, duration_ms, success, phase: phase_id })
            .await;

        if worker_id < 3 && phase_id == 0 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

struct WorkerParams {
    client_idx: usize,
    user_idx: usize,
    prod_idx: [usize; 3],
    recv_num: u32,
    ruc_num: u32,
    qty: [u32; 3],
    price: [f64; 3],
    status_idx: usize,
    list_status_idx: usize,
}

async fn do_create_invoice(
    client: &reqwest::Client,
    base_url: &str,
    seed: &SeedData,
    p: &WorkerParams,
) -> bool {
    let body = serde_json::json!({
        "client_id": seed.client_ids[p.client_idx],
        "created_by": seed.user_ids[p.user_idx],
        "receiver_name": format!("Cliente Test {}", p.recv_num),
        "receiver_ruc": format!("{:08}", p.ruc_num),
        "items": [
            {
                "product_id": seed.product_ids[p.prod_idx[0]],
                "description": "Servicio de facturacion electronica",
                "quantity": p.qty[0] as f64,
                "unit_price": p.price[0]
            },
            {
                "product_id": seed.product_ids[p.prod_idx[1]],
                "description": "Licencia anual de software",
                "quantity": p.qty[1] as f64,
                "unit_price": p.price[1]
            },
            {
                "product_id": seed.product_ids[p.prod_idx[2]],
                "description": "Soporte tecnico",
                "quantity": p.qty[2] as f64,
                "unit_price": p.price[2]
            }
        ]
    });

    client
        .post(format!("{base_url}/invoices"))
        .json(&body)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn do_get_invoice(
    client: &reqwest::Client,
    base_url: &str,
    seed: &SeedData,
) -> bool {
    let ids = seed.invoice_ids.read().await;
    if ids.is_empty() {
        drop(ids);
        return client
            .get(format!("{base_url}/health"))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
    }
    let idx = rand::thread_rng().gen_range(0..ids.len());
    let id = ids[idx].clone();
    drop(ids);

    client
        .get(format!("{base_url}/invoices/{id}"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn do_list_invoices(
    client: &reqwest::Client,
    base_url: &str,
    seed: &SeedData,
    p: &WorkerParams,
) -> bool {
    let statuses = ["DRAFT", "QUEUED", "EMITTED", "DRAFT"];
    let status = statuses[p.list_status_idx % statuses.len()];
    client
        .get(format!(
            "{base_url}/invoices?client_id={}&status={}&limit=20",
            seed.client_ids[p.client_idx], status
        ))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn do_update_status(
    client: &reqwest::Client,
    base_url: &str,
    seed: &SeedData,
    p: &WorkerParams,
) -> bool {
    let ids = seed.invoice_ids.read().await;
    if ids.is_empty() {
        drop(ids);
        return true;
    }
    let idx = p.status_idx % ids.len();
    let id = ids[idx].clone();
    drop(ids);

    let statuses = ["QUEUED", "PROCESSING", "EMITTED", "DRAFT"];
    let status = statuses[p.status_idx % statuses.len()];
    client
        .patch(format!("{base_url}/invoices/{id}/status"))
        .json(&serde_json::json!({ "status": status }))
        .send()
        .await
        .map(|r| r.status().is_success() || r.status() == reqwest::StatusCode::NO_CONTENT)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Reporte
// ---------------------------------------------------------------------------

fn generate_report(
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    phase_summaries: &[PhaseSummary],
    all_stats: &PhaseStats,
    db_url_masked: &str,
) -> String {
    let total_secs = (end_time - start_time).num_seconds();
    let total_req: u64 = phase_summaries.iter().map(|p| p.total_req).sum();
    let total_err: u64 = phase_summaries.iter().map(|p| p.total_err).sum();
    let overall_rps = total_req as f64 / total_secs as f64;

    let host = hostname();
    let rust_version = rust_version();

    let mut out = String::new();
    out.push_str(&format!(
        "# Benchmark Anti-Gravital — Neon PostgreSQL Real\n\n"
    ));
    out.push_str(&format!(
        "**Fecha:** {}\n\n",
        start_time.format("%Y-%m-%d %H:%M UTC")
    ));
    out.push_str("## Hardware y entorno\n\n");
    out.push_str(&format!("- **Hostname:** {host}\n"));
    out.push_str(&format!("- **OS:** {}\n", std::env::consts::OS));
    out.push_str(&format!("- **Arch:** {}\n", std::env::consts::ARCH));
    out.push_str(&format!("- **Rust:** {rust_version}\n"));
    out.push_str(&format!("- **Commit:** {}\n", git_commit()));
    out.push_str(&format!("- **Base de datos:** Neon PostgreSQL (pooler) — {db_url_masked}\n"));
    out.push_str(&format!(
        "- **Duracion total:** {} min {} s\n\n",
        total_secs / 60,
        total_secs % 60
    ));

    out.push_str("## Metodologia\n\n");
    out.push_str("Servicio HTTP Anti-Gravital (axum 0.7 + sqlx 0.8 + tokio rt-multi-thread) \
        ejecutando 7 fases de carga contra Neon PostgreSQL serverless via pooler. \
        Cada fase tiene un numero fijo de workers async concurrentes. \
        Mix de operaciones: 35% POST (crear factura con 3 items + log en transaccion), \
        30% GET (factura + items, 2 queries), 20% GET list (filtrado por client_id+status), \
        15% PATCH (actualizar estado + insertar log en transaccion). \
        Workers sin think time salvo cold-start (50ms). \
        Pool de conexiones DB: 20 (Neon free tier).\n\n");

    out.push_str("## Resumen global\n\n");
    out.push_str(&format!("| Metrica | Valor |\n|---|---|\n"));
    out.push_str(&format!("| Total requests | {total_req} |\n"));
    out.push_str(&format!("| Total errores | {total_err} |\n"));
    out.push_str(&format!(
        "| Error rate | {:.2}% |\n",
        if total_req > 0 {
            total_err as f64 / total_req as f64 * 100.0
        } else {
            0.0
        }
    ));
    out.push_str(&format!(
        "| Throughput promedio | {:.1} req/s |\n",
        overall_rps
    ));

    // Peak rps
    let peak_rps = phase_summaries
        .iter()
        .map(|p| p.req_per_sec)
        .fold(0.0_f64, f64::max);
    out.push_str(&format!("| Throughput pico | {peak_rps:.1} req/s |\n\n"));

    // Per-operation global
    out.push_str("### Por tipo de operacion (global)\n\n");
    out.push_str("| Operacion | Requests | Errores | Mean ms | p50 ms | p95 ms | p99 ms | Max ms |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for s in all_stats.summary() {
        out.push_str(&format!(
            "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            s.op.label(),
            s.total,
            s.errors,
            s.mean,
            s.p50,
            s.p95,
            s.p99,
            s.max
        ));
    }
    out.push('\n');

    out.push_str("## Resultados por fase\n\n");
    for p in phase_summaries {
        out.push_str(&format!(
            "### Fase: {} ({} workers, {} s)\n\n",
            p.name, p.workers, p.duration_s
        ));
        out.push_str(&format!(
            "- Requests: {}  |  Errores: {}  |  Throughput: {:.1} req/s\n\n",
            p.total_req, p.total_err, p.req_per_sec
        ));
        if !p.ops.is_empty() {
            out.push_str("| Operacion | Req | Err | Mean ms | p50 | p95 | p99 | Max |\n");
            out.push_str("|---|---|---|---|---|---|---|---|\n");
            for s in &p.ops {
                out.push_str(&format!(
                    "| {} | {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
                    s.op.label(),
                    s.total,
                    s.errors,
                    s.mean,
                    s.p50,
                    s.p95,
                    s.p99,
                    s.max
                ));
            }
            out.push('\n');
        }
    }

    out.push_str("## Criterio Fase 3: CRUD DSL vs manual\n\n");
    out.push_str("Este benchmark mide el stack Anti-Gravital (axum + sqlx) \
        contra una base de datos PostgreSQL real (Neon serverless). \
        Los handlers fueron escritos manualmente (equivalente al codigo generado por ag-dsl v0.4). \
        La comparativa directa DSL-generado vs manual requiere ejecutar \
        `cargo bench -p todo-api` con el mismo schema, pendiente en gate de Fase 4.\n\n");

    out.push_str("## Notas\n\n");
    out.push_str("- Neon PostgreSQL serverless en us-east-1 (AWS). Latencia de red incluida.\n");
    out.push_str("- Pool de 20 conexiones compartidas entre todos los workers.\n");
    out.push_str("- Operaciones de escritura (POST, PATCH) usan transacciones explicitas.\n");
    out.push_str("- Tablas con prefijo `ag_bench_` creadas para el benchmark.\n");

    out
}

fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn rust_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn mask_url(url: &str) -> String {
    if let Some(at) = url.rfind('@') {
        let host = &url[at + 1..];
        format!("postgresql://***:***@{host}")
    } else {
        "postgresql://***".to_string()
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL no definida. Ejecucion: DATABASE_URL='postgresql://...' neon-bench");

    let port: u16 = std::env::var("BENCH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8899);

    let db_url_masked = mask_url(&database_url);
    info!("Conectando a: {db_url_masked}");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await?;

    info!("Conexion establecida. Configurando schema...");
    setup_schema(&pool).await?;

    info!("Sembrando datos...");
    let seed = seed(&pool).await?;

    let state = AppState {
        db: pool.clone(),
        seed: seed.clone(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/invoices", post(create_invoice))
        .route("/invoices", get(list_invoices))
        .route("/invoices/:id", get(get_invoice))
        .route("/invoices/:id/status", patch(update_status))
        .with_state(state);

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Servidor HTTP escuchando en http://{addr}");

    // Canal de metricas
    let (tx, mut rx) = mpsc::channel::<RequestRecord>(50_000);

    // Servidor en background
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let base_url = format!("http://127.0.0.1:{port}");

    // Warm up: esperar que el servidor arranque
    tokio::time::sleep(Duration::from_millis(500)).await;

    info!("=== Iniciando benchmark 2 horas con 7 fases ===");
    let bench_start = Instant::now();
    let bench_start_utc = chrono::Utc::now();

    let phases = phases();
    let mut phase_summaries: Vec<PhaseSummary> = Vec::new();
    let mut all_stats = PhaseStats::default();

    for (phase_idx, phase) in phases.iter().enumerate() {
        let phase_id = phase_idx as u8;
        info!(
            "--- Fase {} '{}'  ({} workers, {} s) ---",
            phase_idx + 1,
            phase.name,
            phase.workers,
            phase.duration_s
        );

        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let phase_start = Instant::now();

        // Lanzar workers
        let mut handles = Vec::new();
        for w in 0..phase.workers {
            let stop = stop_rx.clone();
            let tx2 = tx.clone();
            let seed2 = seed.clone();
            let url2 = base_url.clone();
            handles.push(tokio::spawn(async move {
                run_worker(w, phase_id, seed2, url2, tx2, stop).await;
            }));
        }

        // Metricas cada 60s durante la fase
        let mut phase_stats = PhaseStats::default();
        let phase_duration = Duration::from_secs(phase.duration_s);
        let mut last_report = Instant::now();
        let mut phase_total = 0u64;
        let mut phase_errs = 0u64;

        loop {
            // Recolectar sin bloquear demasiado
            let elapsed = phase_start.elapsed();
            if elapsed >= phase_duration {
                break;
            }

            // Drain mensajes disponibles
            let deadline = tokio::time::Instant::now() + Duration::from_millis(200);
            loop {
                match tokio::time::timeout_at(
                    deadline,
                    rx.recv(),
                ).await {
                    Ok(Some(rec)) => {
                        if rec.phase == phase_id {
                            phase_stats.record(&rec);
                            all_stats.record(&rec);
                            phase_total += 1;
                            if !rec.success {
                                phase_errs += 1;
                            }
                        }
                    }
                    _ => break,
                }
            }

            // Log cada 60s
            if last_report.elapsed() >= Duration::from_secs(60) {
                let elapsed_s = phase_start.elapsed().as_secs_f64();
                let rps = phase_total as f64 / elapsed_s;
                let err_pct = if phase_total > 0 {
                    phase_errs as f64 / phase_total as f64 * 100.0
                } else {
                    0.0
                };
                info!(
                    "[{}/{} s] req={} rps={:.0} err={:.1}%",
                    elapsed.as_secs(),
                    phase.duration_s,
                    phase_total,
                    rps,
                    err_pct
                );
                last_report = Instant::now();
            }
        }

        // Detener workers
        let _ = stop_tx.send(true);
        for h in handles {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }

        // Drenar cola restante
        while let Ok(rec) = rx.try_recv() {
            if rec.phase == phase_id {
                phase_stats.record(&rec);
                all_stats.record(&rec);
                phase_total += 1;
                if !rec.success { phase_errs += 1; }
            }
        }

        let phase_elapsed_s = phase_start.elapsed().as_secs();
        let rps = if phase_elapsed_s > 0 {
            phase_total as f64 / phase_elapsed_s as f64
        } else {
            0.0
        };

        info!(
            "Fase '{}' completa: {} req, {} err, {:.1} req/s",
            phase.name, phase_total, phase_errs, rps
        );

        phase_summaries.push(PhaseSummary {
            name: phase.name,
            workers: phase.workers,
            duration_s: phase_elapsed_s,
            ops: phase_stats.summary(),
            total_req: phase_total,
            total_err: phase_errs,
            req_per_sec: rps,
        });
    }

    let bench_end_utc = chrono::Utc::now();
    let bench_total_s = bench_start.elapsed().as_secs();

    info!("=== Benchmark completado en {} min {} s ===", bench_total_s / 60, bench_total_s % 60);

    // Drenar todo lo restante
    while let Ok(rec) = rx.try_recv() {
        all_stats.record(&rec);
    }

    // Generar reporte
    let report = generate_report(
        bench_start_utc,
        bench_end_utc,
        &phase_summaries,
        &all_stats,
        &db_url_masked,
    );

    let date_str = bench_start_utc.format("%Y-%m-%d").to_string();
    let report_path = format!(
        "/home/dev-sago-one/Documents/angelnereira/Anti-Gravital/docs/benchmarks/measurement-{date_str}-neon-real.md"
    );
    std::fs::write(&report_path, &report)?;
    info!("Reporte guardado: {report_path}");

    println!("\n{report}");

    server.abort();
    Ok(())
}
