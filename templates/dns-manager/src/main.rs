//! DNS record manager scaffold built on the `ag-domains` `DnsProvider` trait.
//!
//! All record operations go through the provider trait. A native in-memory
//! adapter is bundled so the service builds and runs without DNS credentials;
//! swap it for a real adapter in production.

mod mail_records;

use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ag_domains::error::AgDomainsError;
use ag_domains::provider::DnsProvider;
use ag_domains::record::{DnsRecord, DnsRecordSpec};
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};

/// Native adapter implementing the ag-domains CRUD port without any external
/// service. Data lives for the lifetime of the process.
struct MemoryProvider {
    next_id: AtomicU64,
    zones: Mutex<HashMap<String, Vec<DnsRecord>>>,
}

impl MemoryProvider {
    fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            zones: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl DnsProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "memory"
    }

    async fn zone_id(&self, domain: &str) -> Result<String, AgDomainsError> {
        let mut zones = self.zones.lock().unwrap();
        zones.entry(domain.to_owned()).or_default();
        Ok(domain.to_owned())
    }

    async fn list_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, AgDomainsError> {
        let zones = self.zones.lock().unwrap();
        Ok(zones.get(zone_id).cloned().unwrap_or_default())
    }

    async fn create_record(
        &self,
        zone_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let mut zones = self.zones.lock().unwrap();
        let records = zones.entry(zone_id.to_owned()).or_default();
        let record = DnsRecord {
            id: format!("rec-{}", self.next_id.fetch_add(1, Ordering::Relaxed)),
            zone_id: zone_id.to_owned(),
            name: spec.name.clone(),
            record_type: spec.record_type.clone(),
            content: spec.content.clone(),
            ttl: spec.ttl,
            proxied: spec.proxied,
        };
        records.push(record.clone());
        Ok(record)
    }

    async fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        spec: &DnsRecordSpec,
    ) -> Result<DnsRecord, AgDomainsError> {
        let mut zones = self.zones.lock().unwrap();
        let records = zones
            .get_mut(zone_id)
            .ok_or_else(|| AgDomainsError::ZoneNotFound(zone_id.to_owned()))?;
        let record = records
            .iter_mut()
            .find(|record| record.id == record_id)
            .ok_or_else(|| AgDomainsError::RecordNotFound(record_id.to_owned()))?;
        record.name = spec.name.clone();
        record.record_type = spec.record_type.clone();
        record.content = spec.content.clone();
        record.ttl = spec.ttl;
        record.proxied = spec.proxied;
        Ok(record.clone())
    }

    async fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<(), AgDomainsError> {
        let mut zones = self.zones.lock().unwrap();
        let records = zones
            .get_mut(zone_id)
            .ok_or_else(|| AgDomainsError::ZoneNotFound(zone_id.to_owned()))?;
        let before = records.len();
        records.retain(|record| record.id != record_id);
        if records.len() == before {
            return Err(AgDomainsError::RecordNotFound(record_id.to_owned()));
        }
        Ok(())
    }
}

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn map_error(error: AgDomainsError) -> (StatusCode, String) {
    let status = match &error {
        AgDomainsError::ZoneNotFound(_) | AgDomainsError::RecordNotFound(_) => {
            StatusCode::NOT_FOUND
        }
        _ => StatusCode::BAD_REQUEST,
    };
    (status, error.to_string())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_records(
    State(provider): State<Arc<dyn DnsProvider>>,
    Path(zone): Path<String>,
) -> ApiResult<Json<Vec<DnsRecord>>> {
    let zone_id = provider.zone_id(&zone).await.map_err(map_error)?;
    let records = provider.list_records(&zone_id).await.map_err(map_error)?;
    Ok(Json(records))
}

async fn create_record(
    State(provider): State<Arc<dyn DnsProvider>>,
    Path(zone): Path<String>,
    Json(spec): Json<DnsRecordSpec>,
) -> ApiResult<Json<DnsRecord>> {
    let zone_id = provider.zone_id(&zone).await.map_err(map_error)?;
    let record = provider
        .create_record(&zone_id, &spec)
        .await
        .map_err(map_error)?;
    Ok(Json(record))
}

async fn update_record(
    State(provider): State<Arc<dyn DnsProvider>>,
    Path((zone, id)): Path<(String, String)>,
    Json(spec): Json<DnsRecordSpec>,
) -> ApiResult<Json<DnsRecord>> {
    let zone_id = provider.zone_id(&zone).await.map_err(map_error)?;
    let record = provider
        .update_record(&zone_id, &id, &spec)
        .await
        .map_err(map_error)?;
    Ok(Json(record))
}

async fn delete_record(
    State(provider): State<Arc<dyn DnsProvider>>,
    Path((zone, id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    let zone_id = provider.zone_id(&zone).await.map_err(map_error)?;
    provider
        .delete_record(&zone_id, &id)
        .await
        .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mail_record_presets() -> Json<serde_json::Value> {
    Json(mail_records::presets())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let provider: Arc<dyn DnsProvider> = Arc::new(MemoryProvider::new());
    tracing::info!(provider = provider.name(), "DNS provider ready");

    let app = Router::new()
        .route("/health", get(health))
        .route("/mail-presets", get(mail_record_presets))
        .route(
            "/zones/:zone/records",
            get(list_records).post(create_record),
        )
        .route(
            "/zones/:zone/records/:id",
            put(update_record).delete(delete_record),
        )
        .with_state(provider);

    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "{{name}} listening");
    axum::serve(listener, app).await?;
    Ok(())
}
