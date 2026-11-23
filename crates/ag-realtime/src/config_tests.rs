//! Parity and regression tests for the declared realtime environment schema.
//!
//! The parity oracle below is a faithful replica of the pre-migration
//! `from_env`: it reads an injected map instead of the process environment and
//! applies the same parsing/fallback rules byte for byte. No test mutates the
//! process environment (audit A1). The three `NATS_URL` reads in
//! `external.rs::tests` are integration-test gates for a real NATS server and
//! are intentionally out of scope, exactly like the S3/MinIO gates in
//! `ag-storage`.

use super::*;

/// Faithful replica of the pre-migration `from_env`, reading an injected map
/// instead of the process environment. Used only as a parity oracle.
fn legacy_from_env(get: impl Fn(&str) -> Option<String>) -> RealtimeConfig {
    let nats_mode = match get("NATS_MODE").as_deref() {
        Some("external") => NatsMode::External,
        _ => NatsMode::InProcess,
    };
    RealtimeConfig {
        nats_mode,
        nats_url: get("NATS_URL").unwrap_or_else(|| "nats://localhost:4222".to_string()),
        broadcast_capacity: get("RT_BROADCAST_CAPACITY")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1024),
        nats_tls: get("NATS_TLS")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        nats_tls_ca_path: get("NATS_TLS_CA"),
        nats_tls_cert_path: get("NATS_TLS_CERT"),
        nats_tls_key_path: get("NATS_TLS_KEY"),
        jetstream_enabled: get("NATS_JETSTREAM")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        jetstream_stream_name: get("NATS_JS_STREAM").unwrap_or_else(|| "AG_EVENTS".to_string()),
        jetstream_max_msgs: get("NATS_JS_MAX_MSGS")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1_000_000),
        jetstream_max_bytes: get("NATS_JS_MAX_BYTES")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1_073_741_824),
    }
}

fn get_from<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key: &str| {
        pairs
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned())
    }
}

fn mode_is_external(config: &RealtimeConfig) -> bool {
    matches!(config.nats_mode, NatsMode::External)
}

fn assert_same(a: &RealtimeConfig, b: &RealtimeConfig) {
    assert_eq!(mode_is_external(a), mode_is_external(b));
    assert_eq!(a.nats_url, b.nats_url);
    assert_eq!(a.broadcast_capacity, b.broadcast_capacity);
    assert_eq!(a.nats_tls, b.nats_tls);
    assert_eq!(a.nats_tls_ca_path, b.nats_tls_ca_path);
    assert_eq!(a.nats_tls_cert_path, b.nats_tls_cert_path);
    assert_eq!(a.nats_tls_key_path, b.nats_tls_key_path);
    assert_eq!(a.jetstream_enabled, b.jetstream_enabled);
    assert_eq!(a.jetstream_stream_name, b.jetstream_stream_name);
    assert_eq!(a.jetstream_max_msgs, b.jetstream_max_msgs);
    assert_eq!(a.jetstream_max_bytes, b.jetstream_max_bytes);
}

// ---------------------------------------------------------------------------
// Pre-existing unit tests: behavior unchanged, now using the injected source.
// ---------------------------------------------------------------------------

#[test]
fn default_uses_inprocess_mode() {
    let config = RealtimeConfig::default();
    assert!(matches!(config.nats_mode, NatsMode::InProcess));
}

#[test]
fn default_capacity_is_1024() {
    let config = RealtimeConfig::default();
    assert_eq!(config.broadcast_capacity, 1024);
}

#[test]
fn default_nats_url() {
    let config = RealtimeConfig::default();
    assert_eq!(config.nats_url, "nats://localhost:4222");
}

#[test]
fn external_mode_and_custom_url() {
    let pairs = [
        ("NATS_MODE", "external"),
        ("NATS_URL", "nats://myserver:4222"),
    ];
    let config = RealtimeConfig::from_env_source(pairs);
    assert!(matches!(config.nats_mode, NatsMode::External));
    assert_eq!(config.nats_url, "nats://myserver:4222");
}

#[test]
fn unknown_mode_defaults_to_inprocess() {
    let config = RealtimeConfig::from_env_source([("NATS_MODE", "other_value")]);
    assert!(matches!(config.nats_mode, NatsMode::InProcess));
}

#[test]
fn custom_broadcast_capacity() {
    let config = RealtimeConfig::from_env_source([("RT_BROADCAST_CAPACITY", "256")]);
    assert_eq!(config.broadcast_capacity, 256);
}

#[test]
fn invalid_capacity_uses_default() {
    let config = RealtimeConfig::from_env_source([("RT_BROADCAST_CAPACITY", "not_a_number")]);
    assert_eq!(config.broadcast_capacity, 1024);
}

#[test]
fn no_vars_uses_defaults() {
    let config = RealtimeConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert!(matches!(config.nats_mode, NatsMode::InProcess));
    assert_eq!(config.nats_url, "nats://localhost:4222");
    assert_eq!(config.broadcast_capacity, 1024);
}

#[test]
fn external_mode() {
    let config = RealtimeConfig::from_env_source([("NATS_MODE", "external")]);
    assert!(matches!(config.nats_mode, NatsMode::External));
}

#[test]
fn custom_url() {
    let config = RealtimeConfig::from_env_source([("NATS_URL", "nats://my-nats:4222")]);
    assert_eq!(config.nats_url, "nats://my-nats:4222");
}

#[test]
fn custom_capacity() {
    let config = RealtimeConfig::from_env_source([("RT_BROADCAST_CAPACITY", "2048")]);
    assert_eq!(config.broadcast_capacity, 2048);
}

#[test]
fn tls_enabled() {
    let config = RealtimeConfig::from_env_source([("NATS_TLS", "true")]);
    assert!(config.nats_tls);
}

#[test]
fn jetstream_stream_name() {
    let config = RealtimeConfig::from_env_source([("NATS_JS_STREAM", "MY_STREAM")]);
    assert_eq!(config.jetstream_stream_name, "MY_STREAM");
}

#[test]
fn defaults_tls_off_jetstream_off() {
    let config = RealtimeConfig::default();
    assert!(!config.nats_tls);
    assert!(!config.jetstream_enabled);
    assert_eq!(config.jetstream_stream_name, "AG_EVENTS");
}

// ---------------------------------------------------------------------------
// Parity: declared loader vs. the pre-migration direct reads.
// ---------------------------------------------------------------------------

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let legacy = legacy_from_env(|_| None);
    let declared = RealtimeConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&legacy, &declared);
    assert!(matches!(declared.nats_mode, NatsMode::InProcess));
    assert_eq!(declared.nats_url, "nats://localhost:4222");
    assert_eq!(declared.broadcast_capacity, 1024);
    assert_eq!(declared.jetstream_stream_name, "AG_EVENTS");
    assert_eq!(declared.jetstream_max_msgs, 1_000_000);
    assert_eq!(declared.jetstream_max_bytes, 1_073_741_824);
}

#[test]
fn parity_all_keys_set_to_valid_values() {
    let pairs = [
        ("NATS_MODE", "external"),
        ("NATS_URL", "nats://nats.example:4222"),
        ("RT_BROADCAST_CAPACITY", "512"),
        ("NATS_TLS", "TRUE"),
        ("NATS_TLS_CA", "/synthetic/ca.pem"),
        ("NATS_TLS_CERT", "/synthetic/client.crt"),
        ("NATS_TLS_KEY", "/synthetic/client.key"),
        ("NATS_JETSTREAM", "true"),
        ("NATS_JS_STREAM", "CUSTOM_EVENTS"),
        ("NATS_JS_MAX_MSGS", "5000"),
        ("NATS_JS_MAX_BYTES", "104857600"),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = RealtimeConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert!(mode_is_external(&declared));
    assert!(declared.nats_tls);
    assert!(declared.jetstream_enabled);
    assert_eq!(declared.broadcast_capacity, 512);
}

#[test]
fn parity_malformed_values_fall_back_per_field() {
    let pairs = [
        ("NATS_MODE", "EXTERNAL"),
        ("RT_BROADCAST_CAPACITY", "not_a_number"),
        ("NATS_TLS", "yes"),
        ("NATS_JETSTREAM", "1"),
        ("NATS_JS_MAX_MSGS", "1.5"),
        ("NATS_JS_MAX_BYTES", "999999999999999999999999999"),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = RealtimeConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert!(matches!(declared.nats_mode, NatsMode::InProcess));
    assert_eq!(declared.broadcast_capacity, 1024);
    assert!(!declared.nats_tls);
    assert!(!declared.jetstream_enabled);
    assert_eq!(declared.jetstream_max_msgs, 1_000_000);
    assert_eq!(declared.jetstream_max_bytes, 1_073_741_824);
}

#[test]
fn parity_mode_accepts_only_exact_lowercase_external() {
    for raw in ["external", "EXTERNAL", "External", " external", "external "] {
        let legacy = legacy_from_env(|key| (key == "NATS_MODE").then(|| raw.to_owned()));
        let declared = RealtimeConfig::from_env_source([("NATS_MODE", raw)]);
        assert_same(&legacy, &declared);
        assert_eq!(mode_is_external(&declared), raw == "external");
    }
}

#[test]
fn parity_bool_flags_accept_only_true_case_insensitive() {
    for raw in ["true", "TRUE", "True", "1", "yes", "false", "0", ""] {
        let legacy = legacy_from_env(|key| {
            (key == "NATS_TLS" || key == "NATS_JETSTREAM").then(|| raw.to_owned())
        });
        let pairs = [("NATS_TLS", raw), ("NATS_JETSTREAM", raw)];
        let declared = RealtimeConfig::from_env_source(pairs);
        assert_same(&legacy, &declared);
        assert_eq!(declared.nats_tls, raw.eq_ignore_ascii_case("true"));
        assert_eq!(declared.jetstream_enabled, raw.eq_ignore_ascii_case("true"));
    }
}

#[test]
fn parity_tls_paths_are_optional_and_preserved() {
    let pairs = [
        ("NATS_TLS_CA", "/synthetic/ca.pem"),
        ("NATS_TLS_CERT", "/synthetic/client.crt"),
        ("NATS_TLS_KEY", "/synthetic/client.key"),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = RealtimeConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(
        declared.nats_tls_ca_path.as_deref(),
        Some("/synthetic/ca.pem")
    );
    assert_eq!(
        declared.nats_tls_cert_path.as_deref(),
        Some("/synthetic/client.crt")
    );
    assert_eq!(
        declared.nats_tls_key_path.as_deref(),
        Some("/synthetic/client.key")
    );

    let none = RealtimeConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert!(none.nats_tls_ca_path.is_none());
    assert!(none.nats_tls_cert_path.is_none());
    assert!(none.nats_tls_key_path.is_none());
}

#[test]
fn parity_numeric_fields_fall_back_per_field() {
    let pairs = [
        ("RT_BROADCAST_CAPACITY", "0"),
        ("NATS_JS_MAX_MSGS", "-1"),
        ("NATS_JS_MAX_BYTES", " 4096 "),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = RealtimeConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.broadcast_capacity, 0);
    assert_eq!(declared.jetstream_max_msgs, -1);
    // Rust's integer parser does not trim: the spaces make this fall back.
    assert_eq!(declared.jetstream_max_bytes, 1_073_741_824);
}

#[test]
fn parity_empty_string_values_are_preserved() {
    let pairs = [
        ("NATS_URL", ""),
        ("NATS_TLS_CA", ""),
        ("NATS_TLS_KEY", ""),
        ("NATS_JS_STREAM", ""),
        ("RT_BROADCAST_CAPACITY", ""),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = RealtimeConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.nats_url, "");
    assert_eq!(declared.nats_tls_ca_path.as_deref(), Some(""));
    assert_eq!(declared.nats_tls_key_path.as_deref(), Some(""));
    assert_eq!(declared.jetstream_stream_name, "");
    assert_eq!(declared.broadcast_capacity, 1024);
}

// ---------------------------------------------------------------------------
// Schema shape and loader isolation.
// ---------------------------------------------------------------------------

#[test]
fn schema_declares_every_expected_key_exactly_once() {
    let declarations = realtime_env_declarations();
    let expected = [
        "NATS_MODE",
        "NATS_URL",
        "RT_BROADCAST_CAPACITY",
        "NATS_TLS",
        "NATS_TLS_CA",
        "NATS_TLS_CERT",
        "NATS_TLS_KEY",
        "NATS_JETSTREAM",
        "NATS_JS_STREAM",
        "NATS_JS_MAX_MSGS",
        "NATS_JS_MAX_BYTES",
    ];
    assert_eq!(declarations.len(), expected.len());
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    assert_eq!(names, expected);
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
    assert!(declarations.iter().all(|d| d.ty() == EnvType::String));
    assert!(declarations.iter().all(|d| !d.is_required()));
}

#[test]
fn schema_marks_no_key_as_secret() {
    // TLS cert/key entries are paths, not credential values; no key in this
    // crate is secret (S-1/S-8). The loader still never prints any value.
    let declarations = realtime_env_declarations();
    assert!(declarations.iter().all(|d| !d.is_secret()));
    let secrets: Vec<&str> = declarations
        .iter()
        .filter(|d| d.is_secret())
        .map(EnvDeclaration::name)
        .collect();
    assert!(secrets.is_empty());
}

#[test]
fn declared_defaults_match_config_defaults() {
    let declarations = realtime_env_declarations();
    let with_default: Vec<(&str, &str)> = declarations
        .iter()
        .filter_map(|d| d.default().map(|value| (d.name(), value)))
        .collect();
    assert_eq!(
        with_default,
        vec![
            ("NATS_URL", "nats://localhost:4222"),
            ("NATS_JS_STREAM", "AG_EVENTS"),
        ]
    );
}

#[test]
fn loader_ignores_undeclared_keys() {
    let env = realtime_loader()
        .load_from([("UNDECLARED_REALTIME_KEY", "synthetic-ignored")])
        .expect("no required declarations");
    assert!(env.get("UNDECLARED_REALTIME_KEY").is_none());
    // The undeclared key is ignored; only the two string-valued declarations
    // resolve through their declared defaults.
    assert_eq!(env.len(), 2);
    assert_eq!(env.string("NATS_URL"), Some("nats://localhost:4222"));
    assert_eq!(env.string("NATS_JS_STREAM"), Some("AG_EVENTS"));
}

#[test]
fn injected_source_and_default_config_agree() {
    let default = RealtimeConfig::default();
    let injected = RealtimeConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&default, &injected);
}
