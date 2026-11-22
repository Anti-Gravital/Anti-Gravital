//! Template catalog: the single source of truth for the templates offered by
//! `ag new`. It is shared by the clap value parser, the interactive selector
//! and the scaffolder so the three cannot drift apart.
//!
//! The human-readable registry, including the `template.toml` contract, lives
//! in `templates/CATALOG.md`. Keep both in sync when a template is added.

// Catalog metadata is also part of the documented contract and is consumed by
// the optional TUI. A minimal `--no-default-features` build uses only `parse`
// and `as_str`, so the rest is intentionally not referenced there.
#![allow(dead_code)]

/// Broad category of a template, mirroring the `category` field of
/// `template.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Category {
    Web,
    Api,
    Realtime,
    Infrastructure,
    Ai,
    Testing,
}

impl Category {
    /// Canonical value used in `template.toml` and `templates/CATALOG.md`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Category::Web => "web",
            Category::Api => "api",
            Category::Realtime => "realtime",
            Category::Infrastructure => "infrastructure",
            Category::Ai => "ai",
            Category::Testing => "testing",
        }
    }
}

/// Metadata of one cataloged template.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemplateEntry {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) category: Category,
    /// Environment variables or services needed for the full runtime path.
    pub(crate) external_requirements: &'static [&'static str],
}

macro_rules! define_catalog {
    ($( ($name:literal, $description:literal, $category:expr, $external:expr) ),+ $(,)?) => {
        pub(crate) const CATALOG: &[TemplateEntry] = &[
            $( TemplateEntry {
                name: $name,
                description: $description,
                category: $category,
                external_requirements: $external,
            } ),+
        ];
        /// Values accepted by the clap `--template` parser, in catalog order.
        pub(crate) const TEMPLATE_NAMES: [&str; CATALOG.len()] = [ $($name),+ ];
        static ALL_TEMPLATES: [Template; CATALOG.len()] = [ $( Template { name: $name } ),+ ];
    };
}

define_catalog![
    // Core templates (embedded by `ag new` before the catalog existed).
    (
        "rest",
        "Minimal REST API on ag-core (Shield + axum) with one example route.",
        Category::Api,
        &[]
    ),
    (
        "realtime",
        "WebSocket server on ag-core with a /ws echo route.",
        Category::Realtime,
        &[]
    ),
    (
        "fullstack",
        "REST API with PostgreSQL persistence through ag-data and SQL migrations.",
        Category::Api,
        &["DATABASE_URL"]
    ),
    (
        "layered",
        "Opt-in layered workspace (domain/app/infra/api) with an ag.toml manifest.",
        Category::Api,
        &[]
    ),
    // Web.
    (
        "static-site",
        "Static HTML/CSS/JS site served by a minimal axum handler.",
        Category::Web,
        &[]
    ),
    (
        "marketing-site",
        "Marketing landing page with hero, features and contact sections.",
        Category::Web,
        &[]
    ),
    (
        "blog",
        "Server-rendered blog with a post index and article view.",
        Category::Web,
        &[]
    ),
    (
        "docs-site",
        "Documentation site with navigation and search-ready structure.",
        Category::Web,
        &[]
    ),
    (
        "dashboard",
        "Analytics dashboard shell with layout and chart placeholders.",
        Category::Web,
        &[]
    ),
    (
        "admin-panel",
        "Admin panel with an auth guard placeholder and CRUD table layout.",
        Category::Web,
        &[]
    ),
    // API.
    (
        "database-api",
        "REST API with a database-backed repository and migrations.",
        Category::Api,
        &["DATABASE_URL"]
    ),
    (
        "auth-service",
        "Authentication service with registration, login and JWT sessions.",
        Category::Api,
        &["DATABASE_URL"]
    ),
    (
        "multi-tenant-saas",
        "Multi-tenant SaaS API with tenant isolation scaffolding.",
        Category::Api,
        &["DATABASE_URL"]
    ),
    (
        "webhook-gateway",
        "Inbound webhook receiver with signature verification and retry queue.",
        Category::Api,
        &[]
    ),
    (
        "file-upload-service",
        "Multipart file upload API with a local storage adapter.",
        Category::Api,
        &[]
    ),
    (
        "ecommerce-lite",
        "Lightweight e-commerce API: catalog, cart and checkout stubs.",
        Category::Api,
        &["DATABASE_URL"]
    ),
    // Realtime.
    (
        "realtime-chat-v2",
        "Chat server with rooms and WebSocket/SSE transport.",
        Category::Realtime,
        &[]
    ),
    (
        "websocket-service",
        "WebSocket service with typed messages and broadcast.",
        Category::Realtime,
        &[]
    ),
    (
        "worker-service",
        "Background worker host with typed jobs and an in-memory queue.",
        Category::Realtime,
        &[]
    ),
    (
        "scheduled-jobs",
        "Cron-like scheduled job runner built on ag-workers.",
        Category::Realtime,
        &[]
    ),
    (
        "iot-ingest",
        "Telemetry ingestion endpoint with batch buffering.",
        Category::Realtime,
        &["NATS_URL"]
    ),
    (
        "edge-router",
        "Edge routing policy scaffold using ag-edge hostname rules.",
        Category::Realtime,
        &[]
    ),
    // Infrastructure.
    (
        "mail-service",
        "Transactional mail service with native SMTP/MTA senders.",
        Category::Infrastructure,
        &["AG_SMTP_HOST"]
    ),
    (
        "dns-manager",
        "DNS record manager with a provider adapter and SPF/DKIM/DMARC output.",
        Category::Infrastructure,
        &["AG_CLOUDFLARE_TOKEN"]
    ),
    (
        "object-storage-api",
        "Object storage API with local and pluggable bucket adapters.",
        Category::Infrastructure,
        &["S3_BUCKET"]
    ),
    (
        "monitoring-service",
        "Metrics and health endpoint scaffold with OpenTelemetry wiring.",
        Category::Infrastructure,
        &[]
    ),
    (
        "microservices-starter",
        "Starter for a multi-service workspace with shared contracts.",
        Category::Infrastructure,
        &[]
    ),
    (
        "cli-tool",
        "Standalone CLI binary scaffold using clap and ag-core.",
        Category::Infrastructure,
        &[]
    ),
    // AI.
    (
        "ai-backend-template",
        "AI backend with /health and a provider-agnostic /chat stub.",
        Category::Ai,
        &["AI_API_KEY"]
    ),
    (
        "vector-search",
        "Vector search API with embeddings and a similarity query stub.",
        Category::Ai,
        &["DATABASE_URL", "AI_API_KEY"]
    ),
    (
        "rag-pipeline",
        "Retrieval-augmented generation pipeline scaffold.",
        Category::Ai,
        &["DATABASE_URL", "AI_API_KEY"]
    ),
    (
        "wasi-plugin",
        "WASI plugin scaffold for ag-wasm-host extensions.",
        Category::Ai,
        &[]
    ),
    // Testing.
    (
        "testing-suite",
        "Integration, fuzz and benchmark test harness scaffold.",
        Category::Testing,
        &[]
    ),
];

/// A selected template. Holds the canonical name; all other data is looked up
/// in [`CATALOG`], so a value cannot describe a template that is not cataloged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Template {
    name: &'static str,
}

impl Template {
    /// The non-interactive default and first wizard entry.
    pub(crate) const DEFAULT: Template = Template { name: "rest" };

    /// Every template, in display order.
    pub(crate) const ALL: &'static [Template] = &ALL_TEMPLATES;

    /// Canonical name accepted by `--template`.
    pub(crate) fn as_str(self) -> &'static str {
        self.name
    }

    /// Parses a `--template` value. Unknown names return `None`.
    pub(crate) fn parse(value: &str) -> Option<Template> {
        CATALOG
            .iter()
            .find(|entry| entry.name == value)
            .map(|entry| Template { name: entry.name })
    }

    /// Metadata of this template.
    pub(crate) fn entry(self) -> &'static TemplateEntry {
        CATALOG
            .iter()
            .find(|entry| entry.name == self.name)
            .expect("a Template value always names a catalog entry")
    }

    /// One-line description shown beside the name in the selector.
    pub(crate) fn description(self) -> &'static str {
        self.entry().description
    }

    /// Category of this template.
    pub(crate) fn category(self) -> Category {
        self.entry().category
    }

    /// Environment variables or services needed for the full runtime path.
    pub(crate) fn external_requirements(self) -> &'static [&'static str] {
        self.entry().external_requirements
    }

    /// Files the scaffolder writes, relative to the project directory.
    pub(crate) fn files(self) -> &'static [&'static str] {
        crate::scaffold::files_for(self.name)
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_at_least_the_documented_minimum() {
        assert!(CATALOG.len() >= 24, "catalog has {} entries", CATALOG.len());
    }

    #[test]
    fn catalog_names_are_unique() {
        for (index, entry) in CATALOG.iter().enumerate() {
            assert!(
                !CATALOG[..index]
                    .iter()
                    .any(|other| other.name == entry.name),
                "duplicate catalog name: {}",
                entry.name
            );
        }
    }

    #[test]
    fn rest_is_the_default_and_the_first_entry() {
        assert_eq!(Template::DEFAULT.as_str(), "rest");
        assert_eq!(CATALOG[0].name, "rest");
        assert_eq!(Template::ALL[0], Template::DEFAULT);
    }

    #[test]
    fn every_name_round_trips_through_parse() {
        for name in TEMPLATE_NAMES {
            let Some(template) = Template::parse(name) else {
                panic!("catalog name '{name}' does not parse");
            };
            assert_eq!(template.as_str(), name);
            assert_eq!(template.to_string(), name);
        }
    }

    #[test]
    fn parse_rejects_unknown_names() {
        assert_eq!(Template::parse("unknown"), None);
        assert_eq!(Template::parse(""), None);
        assert_eq!(Template::parse("REST"), None);
    }

    #[test]
    fn names_array_matches_the_catalog() {
        assert_eq!(TEMPLATE_NAMES.len(), CATALOG.len());
        for (name, entry) in TEMPLATE_NAMES.iter().zip(CATALOG.iter()) {
            assert_eq!(name, &entry.name);
        }
    }
}
