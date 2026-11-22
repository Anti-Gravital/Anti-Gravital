//! Typed configuration requests and their single executor (RFC-0018 stage B).
//!
//! The flag path (`ag config get|set|validate`) and the full-screen form both
//! build a [`ConfigRequest`] and hand it to [`execute_config`]. The set of keys
//! mirrors the `ShieldConfig` schema owned by `ag-core`, and every write is
//! validated with that same loader before it touches the file (principles 1
//! and 2 of RFC-0018).

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ag_core::ShieldConfig;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

/// Default file edited by `ag config`.
pub(crate) const CONFIG_FILE: &str = "config.toml";

/// A leaf configuration key exposed by `ag config`.
///
/// The canonical spelling is the dotted TOML path understood by
/// `ag_core::ShieldConfig`. Secrets (for example an inline JWT public key PEM)
/// are deliberately not listed: the form and the flags only accept file paths
/// or environment-variable names for secret material (RFC-0018 section 4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigKey {
    Bind,
    RuntimeWorkers,
    RuntimeBlockingThreads,
    CorsEnabled,
    CorsAllowOrigins,
    CorsAllowMethods,
    CorsAllowHeaders,
    CorsAllowCredentials,
    CsrfEnabled,
    CsrfTokenHeader,
    CsrfTokenCookie,
    RateLimitEnabled,
    RateLimitPerIpRps,
    RateLimitBurst,
    AuthEnabled,
    AuthPublicKeyPath,
    AuthExpectedIssuer,
    AuthExpectedAudience,
    TlsEnabled,
    TlsCertPath,
    TlsKeyPath,
}

impl ConfigKey {
    /// Every key, in the order shown by the form.
    pub(crate) const ALL: [ConfigKey; 21] = [
        ConfigKey::Bind,
        ConfigKey::RuntimeWorkers,
        ConfigKey::RuntimeBlockingThreads,
        ConfigKey::CorsEnabled,
        ConfigKey::CorsAllowOrigins,
        ConfigKey::CorsAllowMethods,
        ConfigKey::CorsAllowHeaders,
        ConfigKey::CorsAllowCredentials,
        ConfigKey::CsrfEnabled,
        ConfigKey::CsrfTokenHeader,
        ConfigKey::CsrfTokenCookie,
        ConfigKey::RateLimitEnabled,
        ConfigKey::RateLimitPerIpRps,
        ConfigKey::RateLimitBurst,
        ConfigKey::AuthEnabled,
        ConfigKey::AuthPublicKeyPath,
        ConfigKey::AuthExpectedIssuer,
        ConfigKey::AuthExpectedAudience,
        ConfigKey::TlsEnabled,
        ConfigKey::TlsCertPath,
        ConfigKey::TlsKeyPath,
    ];

    /// Canonical dotted TOML path.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ConfigKey::Bind => "bind",
            ConfigKey::RuntimeWorkers => "runtime.workers",
            ConfigKey::RuntimeBlockingThreads => "runtime.blocking_threads",
            ConfigKey::CorsEnabled => "cors.enabled",
            ConfigKey::CorsAllowOrigins => "cors.allow_origins",
            ConfigKey::CorsAllowMethods => "cors.allow_methods",
            ConfigKey::CorsAllowHeaders => "cors.allow_headers",
            ConfigKey::CorsAllowCredentials => "cors.allow_credentials",
            ConfigKey::CsrfEnabled => "csrf.enabled",
            ConfigKey::CsrfTokenHeader => "csrf.token_header",
            ConfigKey::CsrfTokenCookie => "csrf.token_cookie",
            ConfigKey::RateLimitEnabled => "rate_limit.enabled",
            ConfigKey::RateLimitPerIpRps => "rate_limit.per_ip_rps",
            ConfigKey::RateLimitBurst => "rate_limit.burst",
            ConfigKey::AuthEnabled => "auth.enabled",
            ConfigKey::AuthPublicKeyPath => "auth.public_key_path",
            ConfigKey::AuthExpectedIssuer => "auth.expected_issuer",
            ConfigKey::AuthExpectedAudience => "auth.expected_audience",
            ConfigKey::TlsEnabled => "tls.enabled",
            ConfigKey::TlsCertPath => "tls.cert_path",
            ConfigKey::TlsKeyPath => "tls.key_path",
        }
    }

    /// Parses a `--key` value. Dotted paths are the documented spelling.
    pub(crate) fn parse(value: &str) -> Option<ConfigKey> {
        let normalized = value.trim().replace('-', "_");
        ConfigKey::ALL
            .into_iter()
            .find(|key| key.as_str() == normalized)
    }

    /// Section heading used to group fields in the form.
    #[cfg(feature = "tui")]
    pub(crate) fn section(self) -> &'static str {
        match self {
            ConfigKey::Bind => "Bind address",
            ConfigKey::RuntimeWorkers | ConfigKey::RuntimeBlockingThreads => "Runtime",
            ConfigKey::CorsEnabled
            | ConfigKey::CorsAllowOrigins
            | ConfigKey::CorsAllowMethods
            | ConfigKey::CorsAllowHeaders
            | ConfigKey::CorsAllowCredentials => "CORS",
            ConfigKey::CsrfEnabled | ConfigKey::CsrfTokenHeader | ConfigKey::CsrfTokenCookie => {
                "CSRF"
            }
            ConfigKey::RateLimitEnabled
            | ConfigKey::RateLimitPerIpRps
            | ConfigKey::RateLimitBurst => "Rate limiting",
            ConfigKey::AuthEnabled
            | ConfigKey::AuthPublicKeyPath
            | ConfigKey::AuthExpectedIssuer
            | ConfigKey::AuthExpectedAudience => "Authentication",
            ConfigKey::TlsEnabled | ConfigKey::TlsCertPath | ConfigKey::TlsKeyPath => "TLS",
        }
    }

    /// One-line description shown next to the field in the form.
    #[cfg(feature = "tui")]
    pub(crate) fn description(self) -> &'static str {
        match self {
            ConfigKey::Bind => "Listener address as host:port.",
            ConfigKey::RuntimeWorkers => "Tokio worker threads (omit for one per CPU).",
            ConfigKey::RuntimeBlockingThreads => "Maximum blocking threads.",
            ConfigKey::CorsEnabled => "Enable cross-origin requests.",
            ConfigKey::CorsAllowOrigins => "Allowed origins, comma separated.",
            ConfigKey::CorsAllowMethods => "Allowed HTTP methods, comma separated.",
            ConfigKey::CorsAllowHeaders => "Allowed request headers, comma separated.",
            ConfigKey::CorsAllowCredentials => "Allow credentials in cross-origin requests.",
            ConfigKey::CsrfEnabled => "Enable stateless double-submit CSRF protection.",
            ConfigKey::CsrfTokenHeader => "Header that carries the CSRF token.",
            ConfigKey::CsrfTokenCookie => "Cookie that carries the CSRF token.",
            ConfigKey::RateLimitEnabled => "Enable per-IP rate limiting.",
            ConfigKey::RateLimitPerIpRps => "Sustained requests per second per IP.",
            ConfigKey::RateLimitBurst => "Token-bucket capacity per IP.",
            ConfigKey::AuthEnabled => "Require a valid bearer token.",
            ConfigKey::AuthPublicKeyPath => {
                "Path to the Ed25519 public key PEM (no secrets in the form)."
            }
            ConfigKey::AuthExpectedIssuer => "Expected `iss` claim.",
            ConfigKey::AuthExpectedAudience => "Expected `aud` claim.",
            ConfigKey::TlsEnabled => "Enable TLS 1.3 termination.",
            ConfigKey::TlsCertPath => "Path to the certificate chain PEM.",
            ConfigKey::TlsKeyPath => "Path to the private key PEM.",
        }
    }

    /// Textual value type of this key, used for inline validation in the form.
    pub(crate) fn kind(self) -> ValueKind {
        match self {
            ConfigKey::Bind => ValueKind::Bind,
            ConfigKey::RuntimeWorkers
            | ConfigKey::RuntimeBlockingThreads
            | ConfigKey::RateLimitPerIpRps
            | ConfigKey::RateLimitBurst => ValueKind::Integer,
            ConfigKey::CorsEnabled
            | ConfigKey::CorsAllowCredentials
            | ConfigKey::CsrfEnabled
            | ConfigKey::RateLimitEnabled
            | ConfigKey::AuthEnabled
            | ConfigKey::TlsEnabled => ValueKind::Bool,
            ConfigKey::CorsAllowOrigins
            | ConfigKey::CorsAllowMethods
            | ConfigKey::CorsAllowHeaders => ValueKind::StringList,
            ConfigKey::CsrfTokenHeader
            | ConfigKey::CsrfTokenCookie
            | ConfigKey::AuthPublicKeyPath
            | ConfigKey::AuthExpectedIssuer
            | ConfigKey::AuthExpectedAudience
            | ConfigKey::TlsCertPath
            | ConfigKey::TlsKeyPath => ValueKind::String,
        }
    }

    /// `(table, key)` in the TOML document; `None` table means a root key.
    fn location(self) -> (Option<&'static str>, &'static str) {
        match self.as_str().split_once('.') {
            Some((table, key)) => (Some(table), key),
            None => (None, self.as_str()),
        }
    }
}

/// The type of a configuration value, used for inline and pre-write validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    Bool,
    Integer,
    String,
    StringList,
    Bind,
}

impl ValueKind {
    /// Validates the textual form of a value. Returns a user-readable problem.
    pub(crate) fn validate(self, value: &str) -> Result<(), String> {
        match self {
            ValueKind::Bool => value
                .parse::<bool>()
                .map(|_| ())
                .map_err(|_| "expected true or false".to_owned()),
            ValueKind::Integer => {
                // TOML integers are signed 64-bit. Parsing as u64 here let
                // values above i64::MAX pass validation and then silently
                // become 1 in build_value (issue #188).
                let parsed = value.parse::<i64>().map_err(|_| {
                    "expected a positive integer up to 9223372036854775807".to_owned()
                })?;
                if parsed <= 0 {
                    Err("expected a positive integer".to_owned())
                } else {
                    Ok(())
                }
            }
            ValueKind::String => Ok(()),
            ValueKind::StringList => {
                if value.trim().is_empty() {
                    return Ok(());
                }
                if value.split(',').any(|part| part.trim().is_empty()) {
                    return Err("use a comma separated list without empty items".to_owned());
                }
                Ok(())
            }
            ValueKind::Bind => SocketAddr::from_str(value)
                .map(|_| ())
                .map_err(|error| format!("expected host:port ({error})")),
        }
    }
}

/// Validates one key/value pair with a message prefixed by the key name.
#[cfg(feature = "tui")]
pub(crate) fn validate_change(key: ConfigKey, value: &str) -> Result<(), String> {
    key.kind()
        .validate(value)
        .map_err(|problem| format!("{}: {problem}", key.as_str()))
}

/// One key/value edit. `set` produces a single change, the form produces many.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigChange {
    pub(crate) key: ConfigKey,
    pub(crate) value: String,
}

impl ConfigChange {
    /// The non-interactive command that performs this exact change.
    pub(crate) fn equivalent_command(&self) -> String {
        format!(
            "ag config set {} {}",
            self.key.as_str(),
            shell_quote(&self.value)
        )
    }
}

/// Everything `ag config` can be asked to do. Both front-ends build one of
/// these; [`execute_config`] is the only place that reads or writes the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigRequest {
    Get { key: ConfigKey },
    Set { changes: Vec<ConfigChange> },
    Validate,
}

/// The result of a successful request.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConfigOutcome {
    Value(String),
    Written { changes: usize },
    Valid,
}

/// Executes a configuration request against `path`.
///
/// For writes, every value is type-checked first, then the whole document is
/// validated with `ag_core::ShieldConfig` before an atomic replace. An invalid
/// result never reaches the file (RFC-0018 section 4.4).
pub(crate) fn execute_config(
    request: &ConfigRequest,
    path: &Path,
) -> Result<ConfigOutcome, String> {
    match request {
        ConfigRequest::Validate => {
            let mut document = read_document(path)?;
            normalize_legacy_bind(&mut document);
            validate_document(&document)?;
            Ok(ConfigOutcome::Valid)
        }
        ConfigRequest::Get { key } => {
            let mut document = read_document(path)?;
            normalize_legacy_bind(&mut document);
            let value = read_value(&document, *key).unwrap_or_else(|| default_value(*key));
            Ok(ConfigOutcome::Value(value))
        }
        ConfigRequest::Set { changes } => {
            for change in changes {
                change
                    .key
                    .kind()
                    .validate(&change.value)
                    .map_err(|problem| format!("{}: {problem}", change.key.as_str()))?;
            }
            let mut document = if path.exists() {
                read_document(path)?
            } else {
                DocumentMut::new()
            };
            normalize_legacy_bind(&mut document);
            for change in changes {
                write_value(&mut document, change);
            }
            validate_document(&document)?;
            atomic_write(path, &document.to_string())?;
            Ok(ConfigOutcome::Written {
                changes: changes.len(),
            })
        }
    }
}

/// Default path of the project configuration file.
pub(crate) fn config_path() -> PathBuf {
    PathBuf::from(CONFIG_FILE)
}

/// Current value of every key, read through the same layer as the executor.
///
/// The form receives this plain data; it never touches the file system itself.
pub(crate) fn current_values(path: &Path) -> Result<Vec<(ConfigKey, String)>, String> {
    let mut document = if path.exists() {
        read_document(path)?
    } else {
        DocumentMut::new()
    };
    normalize_legacy_bind(&mut document);
    Ok(ConfigKey::ALL
        .into_iter()
        .map(|key| {
            let value = read_value(&document, key).unwrap_or_else(|| default_value(key));
            (key, value)
        })
        .collect())
}

/// Reads a document from disk with a useful error when it is absent.
fn read_document(path: &Path) -> Result<DocumentMut, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    text.parse::<DocumentMut>()
        .map_err(|error| format!("{} is not valid TOML: {error}", path.display()))
}

/// Validates a document with the loader `ag-core` itself uses.
fn validate_document(document: &DocumentMut) -> Result<ShieldConfig, String> {
    ShieldConfig::from_toml_str(&document.to_string())
        .map_err(|error| format!("invalid configuration: {error}"))
}

/// Writes to a temporary file and renames, so a crash cannot truncate the file.
fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid configuration path: {}", path.display()))?;
    let temporary = parent.join(format!(".{name}.ag-tmp-{}", std::process::id()));
    fs::write(&temporary, content)
        .map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("cannot replace {}: {error}", path.display()))?;
    Ok(())
}

/// Compatibility for the `[bind] host/port` table emitted by older project
/// templates: convert it to the canonical `bind = "host:port"` root key.
///
/// This keeps stage B usable on scaffolded projects without editing the
/// templates owned by another role in this wave. It is done in memory; the
/// change reaches disk only when a `set` request is executed.
fn normalize_legacy_bind(document: &mut DocumentMut) {
    let legacy = match document.get("bind") {
        Some(Item::Table(table)) => {
            let host = table.get("host").and_then(Item::as_str).map(str::to_owned);
            let port = table.get("port").and_then(Item::as_integer);
            match (host, port) {
                (Some(host), Some(port)) => Some(format!("{host}:{port}")),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(address) = legacy {
        document.insert("bind", Item::Value(Value::from(address)));
    }
}

/// Reads a key from a document. Missing keys return `None`.
fn read_value(document: &DocumentMut, key: ConfigKey) -> Option<String> {
    let (table, leaf) = key.location();
    let item = match table {
        None => document.get(leaf),
        Some(table) => document
            .get(table)
            .and_then(Item::as_table_like)
            .and_then(|table| table.get(leaf)),
    }?;
    let text = match key.kind() {
        ValueKind::Bool => item.as_bool()?.to_string(),
        ValueKind::Integer => item.as_integer()?.to_string(),
        ValueKind::Bind => item.as_str()?.to_owned(),
        ValueKind::String => item.as_str()?.to_owned(),
        ValueKind::StringList => {
            let array = item.as_array()?;
            array
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        }
    };
    Some(text)
}

/// Reads the default value of a key from `ShieldConfig::default()`.
fn default_value(key: ConfigKey) -> String {
    static DEFAULTS: std::sync::OnceLock<DocumentMut> = std::sync::OnceLock::new();
    let document = DEFAULTS.get_or_init(|| {
        let text = ShieldConfig::default()
            .to_toml_string()
            .expect("the default Shield configuration serializes to TOML");
        text.parse::<DocumentMut>()
            .expect("the serialized default configuration is valid TOML")
    });
    read_value(document, key).unwrap_or_default()
}

/// Builds the TOML value of a change. Inline validation has already run.
fn build_value(change: &ConfigChange) -> Value {
    match change.key.kind() {
        ValueKind::Bool => Value::from(change.value.parse::<bool>().unwrap_or(false)),
        ValueKind::Integer => Value::from(change.value.parse::<i64>().unwrap_or(1)),
        ValueKind::String | ValueKind::Bind => Value::from(change.value.clone()),
        ValueKind::StringList => {
            let mut array = Array::new();
            if !change.value.trim().is_empty() {
                for item in change.value.split(',') {
                    array.push(Value::from(item.trim().to_owned()));
                }
            }
            Value::Array(array)
        }
    }
}

/// Applies one edit to a document, preserving all other keys and comments.
fn write_value(document: &mut DocumentMut, change: &ConfigChange) {
    let value = build_value(change);
    let (table, leaf) = change.key.location();
    match table {
        None => {
            if let Some(old) = document.get_mut(leaf).and_then(Item::as_value_mut) {
                *old = apply_decor(value, old.decor().clone());
            } else {
                document.insert(leaf, Item::Value(value));
            }
        }
        Some(table_name) => {
            if document.get(table_name).and_then(Item::as_table).is_none() {
                let mut table = Table::new();
                table.set_implicit(false);
                document.insert(table_name, Item::Table(table));
            }
            if let Some(table) = document.get_mut(table_name).and_then(Item::as_table_mut) {
                if let Some(old) = table.get_mut(leaf).and_then(Item::as_value_mut) {
                    *old = apply_decor(value, old.decor().clone());
                } else {
                    table.insert(leaf, Item::Value(value));
                }
            }
        }
    }
}

/// Keeps the whitespace attached to a replaced value. The leading comment of
/// a key lives on the key itself, which is preserved by mutating in place.
fn apply_decor(mut value: Value, decor: toml_edit::Decor) -> Value {
    *value.decor_mut() = decor;
    value
}

/// Quotes a value for the printed equivalent command. Only values that contain
/// whitespace or quotes need quoting; configuration values never contain shell
/// metacharacters that could execute another command.
fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.:/,@=+*".contains(c))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_parses_from_its_canonical_spelling() {
        for key in ConfigKey::ALL {
            assert_eq!(ConfigKey::parse(key.as_str()), Some(key));
        }
        assert_eq!(ConfigKey::parse("unknown"), None);
    }

    #[test]
    fn dashed_spelling_is_accepted_too() {
        assert_eq!(
            ConfigKey::parse("rate-limit.per-ip-rps"),
            Some(ConfigKey::RateLimitPerIpRps)
        );
    }

    #[test]
    fn secret_inline_keys_are_not_exposed() {
        assert_eq!(ConfigKey::parse("auth.public_key_pem"), None);
        assert!(ConfigKey::ALL
            .iter()
            .all(|key| key.as_str() != "auth.public_key_pem"));
    }

    #[test]
    fn value_validation_rejects_wrong_types() {
        assert!(ConfigKey::TlsEnabled.kind().validate("yes").is_err());
        assert!(ConfigKey::TlsEnabled.kind().validate("true").is_ok());
        assert!(ConfigKey::RateLimitBurst.kind().validate("0").is_err());
        assert!(ConfigKey::Bind.kind().validate("not-an-address").is_err());
        assert!(ConfigKey::Bind.kind().validate("127.0.0.1:9090").is_ok());
        assert!(ConfigKey::CorsAllowMethods
            .kind()
            .validate("GET,,POST")
            .is_err());
    }

    #[test]
    fn defaults_come_from_ag_core() {
        assert_eq!(default_value(ConfigKey::Bind), "0.0.0.0:8080");
        assert_eq!(default_value(ConfigKey::TlsEnabled), "false");
        assert_eq!(default_value(ConfigKey::RateLimitPerIpRps), "100");
    }

    #[test]
    fn get_falls_back_to_the_default_when_a_key_is_absent() {
        let dir = std::env::temp_dir().join(format!("ag-config-get-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE);
        fs::write(&path, "bind = \"127.0.0.1:7000\"\n").expect("write");
        let outcome = execute_config(
            &ConfigRequest::Get {
                key: ConfigKey::Bind,
            },
            &path,
        );
        assert_eq!(outcome, Ok(ConfigOutcome::Value("127.0.0.1:7000".into())));
        let outcome = execute_config(
            &ConfigRequest::Get {
                key: ConfigKey::RateLimitBurst,
            },
            &path,
        );
        assert_eq!(outcome, Ok(ConfigOutcome::Value("200".into())));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_rejects_an_invalid_document() {
        let dir = std::env::temp_dir().join(format!("ag-config-validate-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE);
        fs::write(&path, "[cors]\nunknown_key = true\n").expect("write");
        assert!(execute_config(&ConfigRequest::Validate, &path).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_bind_form_is_normalized_in_memory() {
        let mut legacy = DocumentMut::new();
        let mut table = Table::new();
        table.insert("host", Item::Value(Value::from("0.0.0.0")));
        table.insert("port", Item::Value(Value::from(8080)));
        legacy.insert("bind", Item::Table(table));
        normalize_legacy_bind(&mut legacy);
        assert_eq!(
            legacy.get("bind").and_then(Item::as_str),
            Some("0.0.0.0:8080")
        );
        assert!(validate_document(&legacy).is_ok());
    }

    #[test]
    fn invalid_write_is_rejected_before_the_file_changes() {
        let dir = std::env::temp_dir().join(format!("ag-config-write-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE);
        let original = "bind = \"127.0.0.1:8080\"\n";
        fs::write(&path, original).expect("write");
        let request = ConfigRequest::Set {
            changes: vec![ConfigChange {
                key: ConfigKey::RateLimitPerIpRps,
                value: "0".into(),
            }],
        };
        assert!(execute_config(&request, &path).is_err());
        assert_eq!(fs::read_to_string(&path).expect("read"), original);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_valid_write_preserves_comments_and_other_keys() {
        let dir = std::env::temp_dir().join(format!("ag-config-ok-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE);
        let original = "# keep me\nbind = \"127.0.0.1:8080\"\n\n[runtime]\nblocking_threads = 8\n";
        fs::write(&path, original).expect("write");
        let request = ConfigRequest::Set {
            changes: vec![ConfigChange {
                key: ConfigKey::Bind,
                value: "0.0.0.0:9000".into(),
            }],
        };
        assert_eq!(
            execute_config(&request, &path),
            Ok(ConfigOutcome::Written { changes: 1 })
        );
        let written = fs::read_to_string(&path).expect("read");
        assert!(written.contains("# keep me"), "{written}");
        assert!(written.contains("bind = \"0.0.0.0:9000\""), "{written}");
        assert!(written.contains("blocking_threads = 8"), "{written}");
        let document = fs::read_to_string(&path)
            .expect("read")
            .parse::<DocumentMut>()
            .expect("parse");
        assert!(validate_document(&document).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn multiple_changes_are_applied_in_one_write() {
        let dir = std::env::temp_dir().join(format!("ag-config-batch-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(CONFIG_FILE);
        let request = ConfigRequest::Set {
            changes: vec![
                ConfigChange {
                    key: ConfigKey::Bind,
                    value: "127.0.0.1:7000".into(),
                },
                ConfigChange {
                    key: ConfigKey::RateLimitEnabled,
                    value: "true".into(),
                },
                ConfigChange {
                    key: ConfigKey::RateLimitPerIpRps,
                    value: "50".into(),
                },
            ],
        };
        assert_eq!(
            execute_config(&request, &path),
            Ok(ConfigOutcome::Written { changes: 3 })
        );
        let written = fs::read_to_string(&path).expect("read");
        assert!(written.contains("enabled = true"), "{written}");
        assert!(written.contains("per_ip_rps = 50"), "{written}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// Minimal single-quote aware splitter for parity tests of printed
    /// commands (the shell does the same before `ag` sees argv).
    fn split_command_line(command: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut quoted = false;
        for c in command.chars() {
            match c {
                '\'' => quoted = !quoted,
                ' ' if !quoted && !current.is_empty() => {
                    args.push(std::mem::take(&mut current));
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            args.push(current);
        }
        args
    }

    #[test]
    fn integer_validation_rejects_values_beyond_toml_i64_range() {
        assert!(ValueKind::Integer.validate("18446744073709551615").is_err());
        assert!(ValueKind::Integer.validate("0").is_err());
        assert!(ValueKind::Integer.validate("-1").is_err());
        assert!(ValueKind::Integer.validate("1").is_ok());
        assert!(ValueKind::Integer.validate("512").is_ok());
        assert!(ValueKind::Integer.validate("9223372036854775807").is_ok());
    }

    #[test]
    fn equivalent_command_round_trips_through_the_real_parser() {
        use crate::cli::{Cli, Commands, ConfigCommands};
        use clap::Parser;

        for change in [
            ConfigChange {
                key: ConfigKey::Bind,
                value: "127.0.0.1:7000".into(),
            },
            ConfigChange {
                key: ConfigKey::CorsAllowOrigins,
                value: "https://a.example, https://b.example".into(),
            },
        ] {
            let command = change.equivalent_command();
            let cli = Cli::try_parse_from(split_command_line(&command)).expect("parse");
            let Commands::Config {
                command: Some(ConfigCommands::Set { key, value }),
                ..
            } = cli.command.expect("config command")
            else {
                panic!("not a set command: {command}");
            };
            assert_eq!(ConfigKey::parse(&key), Some(change.key));
            assert_eq!(value, change.value);
        }
    }
}
