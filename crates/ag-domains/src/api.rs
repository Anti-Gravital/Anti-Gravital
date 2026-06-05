//! REST API surface for the control plane (RFC-0009 phase C).
//!
//! An `axum` router backed by any [`crate::store::AttachmentStore`]. The native
//! store keeps this self-hostable; no external database is required. The router
//! is built by [`build_router`] and can be served with [`serve`] or mounted by
//! a host application.
//!
//! Endpoints (under `/v1/domains`):
//! - `POST   /attachments`              create an attachment
//! - `GET    /attachments`              list attachments
//! - `GET    /attachments/{id}`         fetch one
//! - `GET    /attachments/{id}/instructions` DNS instructions
//! - `GET    /attachments/{id}/status`  readiness status
//! - `POST   /attachments/{id}/detach`  detach + tombstone

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::attachment::{
    AttachmentLifecycle, DnsMode, DnsStatus, DomainAttachment, OwnershipMethod, OwnershipStatus,
    RoutingStatus, TargetKind, TlsMode, TlsStatus,
};
use crate::hostname::{Hostname, HostnameKind};
use crate::instructions::{generate_instructions, EdgeTargets};
use crate::ownership::{generate_attachment_id, generate_token, ownership_record_name};
use crate::store::{now_unix, AttachmentStore, StoreError, DEFAULT_TOMBSTONE_SECS};

/// Shared API state: a store behind a mutex plus the edge targets used to
/// generate DNS instructions.
#[derive(Clone)]
pub struct ApiState {
    store: Arc<dyn AttachmentStore>,
    edge: EdgeTargets,
}

impl ApiState {
    /// Builds API state from a shared store and edge targets.
    pub fn new(store: Arc<dyn AttachmentStore>, edge: EdgeTargets) -> Self {
        Self { store, edge }
    }
}

/// Request body to create an attachment.
#[derive(Debug, Deserialize)]
pub struct AttachRequest {
    /// Hostname to attach.
    pub hostname: String,
    /// Project identifier.
    pub project_id: String,
    /// Environment identifier (default `production`).
    #[serde(default = "default_environment")]
    pub environment_id: String,
    /// Target kind (`service` or `project`; default `service`).
    #[serde(default)]
    pub target_kind: Option<String>,
    /// Target reference (default `default`).
    #[serde(default)]
    pub target_ref: Option<String>,
    /// TLS mode (`disabled`, `managed_http01`, `managed_dns01`).
    #[serde(default)]
    pub tls_mode: Option<String>,
}

fn default_environment() -> String {
    "production".to_owned()
}

/// Ownership proof descriptor.
#[derive(Debug, Serialize)]
pub struct OwnershipDto {
    /// Proof method.
    pub method: String,
    /// Record name.
    pub name: String,
    /// Expected record value.
    pub value: String,
}

/// Attachment representation returned by the API.
#[derive(Debug, Serialize)]
pub struct AttachmentDto {
    /// Attachment id.
    pub id: String,
    /// Canonical hostname identity.
    pub hostname_ascii: String,
    /// Unicode display hostname.
    pub hostname_unicode: String,
    /// Project identifier.
    pub project: String,
    /// Environment identifier.
    pub environment: String,
    /// Derived lifecycle.
    pub lifecycle: AttachmentLifecycle,
    /// Ownership readiness.
    pub ownership_status: OwnershipStatus,
    /// DNS readiness.
    pub dns_status: DnsStatus,
    /// TLS readiness.
    pub tls_status: TlsStatus,
    /// Routing readiness.
    pub routing_status: RoutingStatus,
    /// Ownership proof.
    pub ownership: OwnershipDto,
}

impl From<&DomainAttachment> for AttachmentDto {
    fn from(a: &DomainAttachment) -> Self {
        Self {
            id: a.id.clone(),
            hostname_ascii: a.hostname.ascii().to_owned(),
            hostname_unicode: a.hostname.unicode().to_owned(),
            project: a.project.clone(),
            environment: a.environment.clone(),
            lifecycle: a.lifecycle,
            ownership_status: a.ownership_status,
            dns_status: a.dns_status,
            tls_status: a.tls_status,
            routing_status: a.routing_status,
            ownership: OwnershipDto {
                method: "txt".to_owned(),
                name: a.ownership_name.clone(),
                value: a.ownership_value.clone(),
            },
        }
    }
}

/// A single DNS record in an instructions response.
#[derive(Debug, Serialize)]
pub struct RecordDto {
    /// Record type.
    pub record_type: String,
    /// Record name.
    pub name: String,
    /// Record value.
    pub value: String,
    /// TTL seconds.
    pub ttl: u32,
    /// Whether this is the ownership record.
    pub is_ownership: bool,
}

/// DNS instructions response.
#[derive(Debug, Serialize)]
pub struct InstructionsDto {
    /// Records to publish.
    pub records: Vec<RecordDto>,
    /// Advisory notes.
    pub notes: Vec<String>,
}

/// Error body.
#[derive(Debug, Serialize)]
pub struct ErrorDto {
    /// Human-readable error message.
    pub error: String,
}

/// API error mapped to an HTTP status.
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorDto {
                error: self.message,
            }),
        )
            .into_response()
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        let status = match e {
            StoreError::AlreadyExists(_) => StatusCode::CONFLICT,
            StoreError::Tombstoned { .. } => StatusCode::CONFLICT,
            StoreError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiError::new(status, e.to_string())
    }
}

/// Builds the control-plane API router.
pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route(
            "/v1/domains/attachments",
            post(create_attachment).get(list_attachments),
        )
        .route("/v1/domains/attachments/:id", get(get_attachment))
        .route(
            "/v1/domains/attachments/:id/instructions",
            get(get_instructions),
        )
        .route("/v1/domains/attachments/:id/status", get(get_attachment))
        .route(
            "/v1/domains/attachments/:id/detach",
            post(detach_attachment),
        )
        .with_state(state)
}

/// Serves the API on the given listener until it errors.
pub async fn serve(listener: tokio::net::TcpListener, state: ApiState) -> std::io::Result<()> {
    axum::serve(listener, build_router(state)).await
}

fn parse_target_kind(s: Option<&str>) -> TargetKind {
    match s {
        Some("project") => TargetKind::Project,
        _ => TargetKind::Service,
    }
}

fn parse_tls_mode(s: Option<&str>, kind: HostnameKind) -> TlsMode {
    match s {
        Some("disabled") => TlsMode::Disabled,
        Some("managed_dns01") => TlsMode::ManagedDns01,
        Some("managed_http01") => TlsMode::ManagedHttp01,
        None => match kind {
            HostnameKind::Wildcard => TlsMode::ManagedDns01,
            _ => TlsMode::ManagedHttp01,
        },
        Some(_) => TlsMode::ManagedHttp01,
    }
}

async fn create_attachment(
    State(state): State<ApiState>,
    Json(req): Json<AttachRequest>,
) -> Result<(StatusCode, Json<AttachmentDto>), ApiError> {
    let hostname = Hostname::parse(&req.hostname)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;

    let id = generate_attachment_id();
    let ownership_value = generate_token(&id);
    let ownership_name = ownership_record_name(&hostname);
    let tls_mode = parse_tls_mode(req.tls_mode.as_deref(), hostname.kind());

    let mut attachment = DomainAttachment {
        id,
        hostname,
        project: req.project_id,
        environment: req.environment_id,
        target_kind: parse_target_kind(req.target_kind.as_deref()),
        target_ref: req.target_ref.unwrap_or_else(|| "default".to_owned()),
        dns_mode: DnsMode::Manual,
        tls_mode,
        ownership_method: OwnershipMethod::Txt,
        ownership_name,
        ownership_value,
        ownership_status: OwnershipStatus::Pending,
        dns_status: DnsStatus::Pending,
        tls_status: TlsStatus::Pending,
        routing_status: RoutingStatus::Disabled,
        lifecycle: AttachmentLifecycle::Draft,
        created_at: now_unix(),
    };
    attachment.recompute_lifecycle();

    let dto = AttachmentDto::from(&attachment);
    state.store.create(attachment).await?;
    Ok((StatusCode::CREATED, Json(dto)))
}

async fn list_attachments(
    State(state): State<ApiState>,
) -> Result<Json<Vec<AttachmentDto>>, ApiError> {
    let all = state.store.list().await?;
    Ok(Json(all.iter().map(AttachmentDto::from).collect()))
}

async fn get_attachment(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<AttachmentDto>, ApiError> {
    state
        .store
        .get_by_id(&id)
        .await?
        .map(|a| Json(AttachmentDto::from(&a)))
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no attachment {id}")))
}

async fn get_instructions(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<InstructionsDto>, ApiError> {
    let attachment = state
        .store
        .get_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no attachment {id}")))?;

    let ins = generate_instructions(
        &attachment.hostname,
        &state.edge,
        &attachment.ownership_value,
    );
    let records = ins
        .all()
        .map(|r| RecordDto {
            record_type: r.record_type.to_string(),
            name: r.name.clone(),
            value: r.value.clone(),
            ttl: r.ttl,
            is_ownership: r.is_ownership,
        })
        .collect();
    Ok(Json(InstructionsDto {
        records,
        notes: ins.notes,
    }))
}

async fn detach_attachment(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let attachment = state
        .store
        .get_by_id(&id)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, format!("no attachment {id}")))?;
    state
        .store
        .detach(attachment.hostname.ascii(), DEFAULT_TOMBSTONE_SECS)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryStore;

    fn state() -> ApiState {
        let store: Arc<dyn AttachmentStore> = Arc::new(InMemoryStore::new());
        ApiState::new(store, EdgeTargets::new("edge.example-cloud.net"))
    }

    #[test]
    fn target_kind_and_tls_defaults() {
        assert_eq!(parse_target_kind(None), TargetKind::Service);
        assert_eq!(parse_target_kind(Some("project")), TargetKind::Project);
        assert_eq!(
            parse_tls_mode(None, HostnameKind::Wildcard),
            TlsMode::ManagedDns01
        );
        assert_eq!(
            parse_tls_mode(None, HostnameKind::Apex),
            TlsMode::ManagedHttp01
        );
    }

    #[test]
    fn router_builds() {
        let _ = build_router(state());
    }
}
