//! Versioned machine-readable output for read-only CLI commands.
//!
//! RFC-0021 section 4.3 requires `--output json` to produce versioned shapes:
//! prose may change between releases, JSON follows SemVer. This module owns
//! the envelope, the version constant and the per-command data shapes. The
//! versioned contract is published under `docs/mcp/`.
//!
//! Prose output is never touched here. When the flag is absent the commands
//! keep their existing `println!` behaviour. Mutating commands are refused
//! before any executor runs (see `main::run_json_command`).
//!
//! A new read shape may be defined here ahead of its CLI dispatch wiring. Such
//! a shape is published in `docs/mcp/` as a contract fixture and is covered by
//! a serialization test, but it is not emitted until the dispatch layer in
//! `cli.rs` and `main.rs` maps the command to it. The contract document states
//! that boundary explicitly so no consumer mistakes a fixture for live output.

use serde::Serialize;
use std::io::{self, Write};

/// Version of the JSON contract emitted by `--output json`.
///
/// Shapes are unstable until Anti-Gravital 0.5 (RFC-0021 risk table). Any
/// change to a field name, type, presence or value set bumps this version.
pub(crate) const CONTRACT_VERSION: &str = "0.2.0";

/// Value of the global `--output` flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum OutputFormat {
    /// Default human-readable prose.
    Prose,
    /// Versioned machine-readable JSON.
    Json,
}

/// Result wrapper added to every JSON document.
#[derive(Debug, Serialize)]
pub(crate) struct Envelope<T: Serialize> {
    pub(crate) contract_version: &'static str,
    pub(crate) command: &'static str,
    #[serde(flatten)]
    pub(crate) outcome: Outcome<T>,
}

/// Exactly one of `data` (status `ok`) or `error` (status `error`) is emitted.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub(crate) enum Outcome<T: Serialize> {
    /// The command produced a structured result.
    Ok { data: T },
    /// The command could not produce a result.
    Error { error: ErrorBody },
}

/// Stable machine-readable error description.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

/// One diagnostic in a `schema.lint` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DiagnosticView {
    pub(crate) severity: &'static str,
    pub(crate) message: String,
    pub(crate) hint: Option<String>,
    pub(crate) span_start: usize,
    pub(crate) span_end: usize,
}

/// Data of `schema.lint`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SchemaLintData {
    /// Schema path as displayed to the user.
    pub(crate) schema: String,
    /// `clean`, `warnings` or `errors`. Non-empty errors imply exit code 1.
    pub(crate) result: &'static str,
    pub(crate) error_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) diagnostics: Vec<DiagnosticView>,
}

/// One change in a `schema.diff` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SchemaChangeView {
    /// `breaking` or `additive`.
    pub(crate) severity: &'static str,
    pub(crate) message: String,
}

/// Data of `schema.diff`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SchemaDiffData {
    pub(crate) reference: String,
    pub(crate) schema: String,
    pub(crate) changes: Vec<SchemaChangeView>,
}

/// One declared worker in a `workers.list` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WorkerView {
    pub(crate) name: String,
    pub(crate) queue: String,
    pub(crate) mode: String,
    pub(crate) concurrency: Option<u32>,
    pub(crate) input: Vec<String>,
    pub(crate) retry_max_attempts: Option<u32>,
}

/// Data of `workers.list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WorkersListData {
    pub(crate) schema: String,
    pub(crate) workers: Vec<WorkerView>,
}

/// One artifact summary in a `deploy.plan` result (RFC-0034).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DeployFileView {
    /// Destination under `deploy/generated/`.
    pub(crate) path: String,
    /// Authored source path under the project.
    pub(crate) source: String,
    /// Size of the authored file in bytes.
    pub(crate) bytes: u64,
    /// 16-hex-digit FNV-1a fingerprint of the source bytes.
    pub(crate) fingerprint: String,
}

/// Data of `deploy.plan` (read-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DeployPlanData {
    pub(crate) target: &'static str,
    pub(crate) project: String,
    pub(crate) plan_id: String,
    pub(crate) files: Vec<DeployFileView>,
}

/// Data of `deploy.status` (read-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DeployStatusData {
    pub(crate) generated: bool,
    pub(crate) target: Option<String>,
    pub(crate) plan_id: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) files: Vec<String>,
    pub(crate) current_plan_id: Option<String>,
    pub(crate) in_sync: bool,
    pub(crate) sources_available: bool,
}

impl DeployStatusData {
    /// Neutral view returned before any `ag deploy apply` has run.
    pub(crate) fn empty() -> Self {
        Self {
            generated: false,
            target: None,
            plan_id: None,
            project: None,
            files: Vec::new(),
            current_plan_id: None,
            in_sync: false,
            sources_available: false,
        }
    }
}

/// Writes one compact, deterministic JSON line to stdout.
fn write_json<T: Serialize>(value: &T) {
    let text = serde_json::to_string(value).expect("versioned JSON shapes are serializable");
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "{text}");
    let _ = stdout.flush();
}

/// Emits a successful versioned result.
pub(crate) fn emit<T: Serialize>(command: &'static str, data: T) {
    write_json(&Envelope {
        contract_version: CONTRACT_VERSION,
        command,
        outcome: Outcome::Ok { data },
    });
}

/// Emits a versioned error result. `code` is from the closed set documented in
/// `docs/mcp/cli-json-contract-0.1.0.md`.
pub(crate) fn emit_error(command: &'static str, code: &'static str, message: impl Into<String>) {
    write_json(&Envelope {
        contract_version: CONTRACT_VERSION,
        command,
        outcome: Outcome::<()>::Error {
            error: ErrorBody {
                code,
                message: message.into(),
            },
        },
    });
}

/// One validated record in `dns.plan` (RFC-0033).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DnsRecordView {
    pub(crate) record_type: String,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) ttl: u32,
    pub(crate) is_ownership: bool,
}

/// Data of `dns.plan`. Contains no credential, zone id or path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DnsPlanData {
    pub(crate) plan_id: String,
    pub(crate) domain: String,
    pub(crate) zone: String,
    pub(crate) environment: String,
    pub(crate) visibility: String,
    pub(crate) project: String,
    pub(crate) service: String,
    pub(crate) target_kind: String,
    pub(crate) tls: String,
    pub(crate) records: Vec<DnsRecordView>,
    pub(crate) notes: Vec<String>,
}

/// Data of `dns.status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DnsStatusData {
    pub(crate) environment: String,
    pub(crate) applied: bool,
    pub(crate) plan_id: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) visibility: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) record_count: usize,
    pub(crate) in_sync: bool,
}

/// Where a `config.get` value came from (RFC-0018 single config executor).
///
/// The shape is published ahead of its CLI dispatch wiring, which lives in
/// files owned by another role. It is exercised by the serialization tests in
/// this module and by the contract fixture in `tests/json_output.rs`.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ConfigValueSource {
    /// The key was present in `config.toml`.
    File,
    /// The key was absent and the compiled-in default was used.
    Default,
}

/// Data of `config.get`.
///
/// The RFC-0018 key set deliberately exposes file paths and non-secret
/// settings only; inline secret material is not addressable by a key. `key`
/// and `kind` are `'static` strings taken from closed sets, so no value
/// supplied on the command line can become a field name in this shape.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ConfigGetData {
    /// Canonical dotted `ConfigKey::as_str()`, e.g. `rate_limit.per_ip_rps`.
    pub(crate) key: &'static str,
    /// One of `bool`, `integer`, `string`, `string_list`, `bind`.
    pub(crate) kind: &'static str,
    /// The value as printed by the prose path. Always data, never a field name.
    pub(crate) value: String,
    /// Whether the value was read from the file or fell back to the default.
    pub(crate) source: ConfigValueSource,
}

impl ConfigGetData {
    /// Builds the versioned shape from the config executor's plain output.
    /// `key` and `kind` must come from the closed sets constrained by the
    /// published JSON Schema; `value` is opaque data and is never interpreted.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(
        key: &'static str,
        kind: &'static str,
        value: impl Into<String>,
        source: ConfigValueSource,
    ) -> Self {
        Self {
            key,
            kind,
            value: value.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_has_version_command_and_status() {
        let value = Envelope {
            contract_version: CONTRACT_VERSION,
            command: "workers.list",
            outcome: Outcome::Ok {
                data: WorkersListData {
                    schema: "schema.ag".to_owned(),
                    workers: vec![],
                },
            },
        };
        assert_eq!(
            serde_json::to_string(&value).expect("serialize"),
            r#"{"contract_version":"0.2.0","command":"workers.list","status":"ok","data":{"schema":"schema.ag","workers":[]}}"#
        );
    }

    #[test]
    fn error_envelope_has_no_data_field() {
        let value = Envelope {
            contract_version: CONTRACT_VERSION,
            command: "unknown",
            outcome: Outcome::<()>::Error {
                error: ErrorBody {
                    code: "unsupported_command",
                    message: "not available".to_owned(),
                },
            },
        };
        let json = serde_json::to_string(&value).expect("serialize");
        assert_eq!(
            json,
            r#"{"contract_version":"0.2.0","command":"unknown","status":"error","error":{"code":"unsupported_command","message":"not available"}}"#
        );
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn contract_matches_the_published_schema() {
        let schema = include_str!("../../../docs/mcp/cli-json-0.2.0.schema.json");
        let parsed: serde_json::Value = serde_json::from_str(schema).expect("valid schema JSON");
        assert_eq!(
            parsed["$id"].as_str(),
            Some("https://anti-gravital.dev/mcp/cli-json-0.2.0.schema.json")
        );
        assert_eq!(
            parsed["properties"]["contract_version"]["const"].as_str(),
            Some(CONTRACT_VERSION)
        );
        // The config.get shape is additive: it must be present in the same
        // versioned schema as the already emitted commands.
        let commands = parsed["properties"]["command"]["enum"]
            .as_array()
            .expect("command enum");
        assert!(
            commands
                .iter()
                .any(|value| value.as_str() == Some("config.get")),
            "schema must declare the config.get command shape"
        );
    }

    #[test]
    fn config_get_shape_serializes_to_the_versioned_envelope() {
        let value = Envelope {
            contract_version: CONTRACT_VERSION,
            command: "config.get",
            outcome: Outcome::Ok {
                data: ConfigGetData::new("bind", "bind", "127.0.0.1:8080", ConfigValueSource::File),
            },
        };
        assert_eq!(
            serde_json::to_string(&value).expect("serialize"),
            r#"{"contract_version":"0.2.0","command":"config.get","status":"ok","data":{"key":"bind","kind":"bind","value":"127.0.0.1:8080","source":"file"}}"#
        );
    }

    #[test]
    fn config_get_shape_has_a_closed_field_set() {
        let data = ConfigGetData::new(
            "tls.key_path",
            "string",
            "CANARY-SECRET-9f3a",
            ConfigValueSource::Default,
        );
        let json = serde_json::to_value(&data).expect("serialize");
        let object = json.as_object().expect("object");
        let mut fields: Vec<&str> = object.keys().map(String::as_str).collect();
        fields.sort_unstable();
        assert_eq!(fields, ["key", "kind", "source", "value"]);
        // A canary stays in the `value` data slot; it cannot become a key.
        assert_eq!(object["value"].as_str(), Some("CANARY-SECRET-9f3a"));
        assert_eq!(object["key"].as_str(), Some("tls.key_path"));
        assert_eq!(object["source"].as_str(), Some("default"));
    }
}
