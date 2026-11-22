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

/// One artifact summary in a `deploy.plan` result (RFC-0033).
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
    }
}
