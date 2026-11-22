//! Native component-health registry and Axum handlers.
//!
//! No external service, poller or scheduler is involved: checks are evaluated
//! on demand when `/health` (liveness) or `/ready` (readiness) is scraped.
//! This keeps the default collector-free operation required by ADR-0009.
//!
//! Registered checks are synchronous and must be cheap and side-effect free.
//! They must never return secrets or personal data in `detail`.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

/// Result of a single component check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentHealth {
    /// Stable, human-readable component name.
    pub name: String,
    /// Whether the component is operating correctly.
    pub healthy: bool,
    /// Optional non-sensitive detail about the check result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ComponentHealth {
    /// Creates a health result without detail.
    pub fn new(name: impl Into<String>, healthy: bool) -> Self {
        Self {
            name: name.into(),
            healthy,
            detail: None,
        }
    }

    /// Attaches a non-sensitive detail message.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Aggregated health state returned by the handlers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthSnapshot {
    /// `up` when no registered component fails, `down` otherwise.
    pub status: String,
    /// Project scope configured at initialization, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Individual component results, ordered by name.
    pub components: Vec<ComponentHealth>,
}

type Check = Arc<dyn Fn() -> ComponentHealth + Send + Sync + 'static>;

/// Registry of named component checks.
///
/// Cloning shares the same underlying checks, so a registry can be mounted
/// into an Axum router with `State` without duplicating state.
#[derive(Clone, Default)]
pub struct HealthRegistry {
    checks: Arc<RwLock<BTreeMap<String, Check>>>,
    project: Option<String>,
}

impl HealthRegistry {
    /// Creates an empty registry with an optional project scope.
    pub fn new(project: Option<String>) -> Self {
        Self {
            checks: Arc::new(RwLock::new(BTreeMap::new())),
            project,
        }
    }

    /// Registers or replaces a named check.
    ///
    /// Names should be stable identifiers (for example `database`, `queue`).
    /// Re-registering the same name replaces the previous check.
    pub fn register<F>(&self, name: impl Into<String>, check: F)
    where
        F: Fn() -> ComponentHealth + Send + Sync + 'static,
    {
        let mut guards = self.checks.write().unwrap_or_else(|e| e.into_inner());
        guards.insert(name.into(), Arc::new(check));
    }

    /// Removes a check, returning whether it existed.
    pub fn unregister(&self, name: &str) -> bool {
        self.checks
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(name)
            .is_some()
    }

    /// Number of registered checks.
    pub fn len(&self) -> usize {
        self.checks.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Whether no checks are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Evaluates every check and returns the aggregate snapshot.
    pub fn snapshot(&self) -> HealthSnapshot {
        let guards = self.checks.read().unwrap_or_else(|e| e.into_inner());
        let components: Vec<ComponentHealth> = guards.values().map(|check| check()).collect();
        let healthy = components.iter().all(|c| c.healthy);
        HealthSnapshot {
            status: if healthy { "up" } else { "down" }.to_string(),
            project: self.project.clone(),
            components,
        }
    }
}

/// Liveness handler: returns 200 when the process can answer requests.
///
/// Liveness deliberately does not evaluate component checks: a failing
/// dependency should not cause the runtime to restart a process that is
/// otherwise healthy. Use [`readiness_handler`] for dependencies.
pub async fn liveness_handler() -> StatusCode {
    StatusCode::OK
}

/// Readiness handler: returns 503 when any registered component is down.
pub async fn readiness_handler(
    State(registry): State<HealthRegistry>,
) -> (StatusCode, Json<HealthSnapshot>) {
    let snapshot = registry.snapshot();
    let code = if snapshot.status == "up" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(snapshot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_is_up() {
        let registry = HealthRegistry::new(None);
        let snapshot = registry.snapshot();
        assert_eq!(snapshot.status, "up");
        assert!(snapshot.components.is_empty());
    }

    #[test]
    fn failing_component_flips_readiness() {
        let registry = HealthRegistry::new(Some("demo".to_string()));
        registry.register("database", || ComponentHealth::new("database", true));
        registry.register("queue", || {
            ComponentHealth::new("queue", false).with_detail("connection refused")
        });

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.status, "down");
        assert_eq!(snapshot.project.as_deref(), Some("demo"));
        assert_eq!(snapshot.components.len(), 2);
        assert!(!snapshot.components[1].healthy);
        assert_eq!(
            snapshot.components[1].detail.as_deref(),
            Some("connection refused")
        );
    }

    #[test]
    fn unregister_restores_readiness() {
        let registry = HealthRegistry::new(None);
        registry.register("cache", || ComponentHealth::new("cache", false));
        assert_eq!(registry.snapshot().status, "down");
        assert!(registry.unregister("cache"));
        assert_eq!(registry.snapshot().status, "up");
    }

    #[tokio::test]
    async fn readiness_handler_returns_503_when_component_is_down() {
        let registry = HealthRegistry::new(None);
        registry.register("database", || ComponentHealth::new("database", false));
        let (code, body) = readiness_handler(State(registry)).await;
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body.0.status, "down");
    }

    #[tokio::test]
    async fn liveness_handler_does_not_evaluate_components() {
        let registry = HealthRegistry::new(None);
        registry.register("database", || ComponentHealth::new("database", false));
        assert_eq!(liveness_handler().await, StatusCode::OK);
        assert_eq!(registry.snapshot().status, "down");
    }
}
