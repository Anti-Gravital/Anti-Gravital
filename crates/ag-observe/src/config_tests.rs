//! Parity, schema and regression tests for the declared observe environment.

use super::*;

/// Faithful replica of the pre-migration `from_env`, reading an injected map
/// instead of the process environment. Used only as a parity oracle.
fn legacy_from_map(get: impl Fn(&str) -> Option<String>) -> ObserveConfig {
    let otlp_endpoint = get("OTEL_EXPORTER_OTLP_ENDPOINT");
    let prometheus_port = get("PROMETHEUS_PORT")
        .and_then(|value| value.parse().ok())
        .unwrap_or(9090);
    let log_format = match get("LOG_FORMAT").as_deref() {
        Some("json") => LogFormat::Json,
        _ => LogFormat::Pretty,
    };
    let dev_console = get("AG_DEV_CONSOLE")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);
    let project_id = get("AG_PROJECT_ID");
    ObserveConfig {
        otlp_endpoint,
        prometheus_port,
        log_format,
        dev_console,
        project_id,
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

fn assert_same(a: &ObserveConfig, b: &ObserveConfig) {
    assert_eq!(a.otlp_endpoint, b.otlp_endpoint);
    assert_eq!(a.prometheus_port, b.prometheus_port);
    let a_format = matches!(a.log_format, LogFormat::Json);
    let b_format = matches!(b.log_format, LogFormat::Json);
    assert_eq!(a_format, b_format, "log format must match the oracle");
    assert_eq!(a.dev_console, b.dev_console);
    assert_eq!(a.project_id, b.project_id);
}

#[test]
fn config_defaults_are_sensible() {
    let cfg = ObserveConfig::default();
    assert!(cfg.otlp_endpoint.is_none());
    assert_eq!(cfg.prometheus_port, 9090);
    assert!(matches!(cfg.log_format, LogFormat::Pretty));
    assert!(!cfg.dev_console);
    assert!(cfg.project_id.is_none());
}

#[test]
fn project_id_validation_accepts_bounded_labels() {
    assert!(is_valid_project_id("proj-1"));
    assert!(is_valid_project_id("a.b_c-9"));
    assert!(is_valid_project_id(&"x".repeat(64)));
}

#[test]
fn project_id_validation_rejects_unbounded_or_unsafe_labels() {
    assert!(!is_valid_project_id(""));
    assert!(!is_valid_project_id(&"x".repeat(65)));
    assert!(!is_valid_project_id("with space"));
    assert!(!is_valid_project_id("a/b"));
    assert!(!is_valid_project_id("a=b"));
}

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let legacy = legacy_from_map(|_| None);
    let declared = ObserveConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&legacy, &declared);
    assert_eq!(declared.prometheus_port, 9090);
    assert!(declared.otlp_endpoint.is_none());
    assert!(matches!(declared.log_format, LogFormat::Pretty));
    assert!(!declared.dev_console);
    assert!(declared.project_id.is_none());
}

#[test]
fn parity_all_keys_set_to_valid_values() {
    let pairs: [(&str, &str); 5] = [
        (
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "http://127.0.0.1:4318/v1/traces",
        ),
        ("PROMETHEUS_PORT", "9000"),
        ("LOG_FORMAT", "json"),
        ("AG_DEV_CONSOLE", "true"),
        ("AG_PROJECT_ID", "demo-project.1"),
    ];
    let legacy = legacy_from_map(get_from(&pairs));
    let declared = ObserveConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.prometheus_port, 9000);
    assert!(matches!(declared.log_format, LogFormat::Json));
    assert!(declared.dev_console);
    assert_eq!(declared.project_id.as_deref(), Some("demo-project.1"));
}

#[test]
fn parity_malformed_values_fall_back_per_field() {
    let pairs: [(&str, &str); 6] = [
        ("PROMETHEUS_PORT", "not-a-port"),
        ("PROMETHEUS_PORT_EXTRA", "ignored"),
        ("LOG_FORMAT", "JSON"),
        ("AG_DEV_CONSOLE", "yes"),
        ("AG_PROJECT_ID", ""),
        ("OTEL_EXPORTER_OTLP_ENDPOINT", ""),
    ];
    let legacy = legacy_from_map(get_from(&pairs));
    let declared = ObserveConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.prometheus_port, 9090);
    assert!(matches!(declared.log_format, LogFormat::Pretty));
    assert!(!declared.dev_console);
    // An explicitly empty value is present for the loader, exactly as the
    // old `var(...).ok()` was: the empty string is preserved, not turned
    // into None. Validation of AG_PROJECT_ID still happens in `init`.
    assert_eq!(declared.project_id.as_deref(), Some(""));
    assert_eq!(declared.otlp_endpoint.as_deref(), Some(""));
}

#[test]
fn parity_log_format_accepts_only_lowercase_json() {
    for raw in ["json", "JSON", "Json", "pretty", "", " json"] {
        let legacy = legacy_from_map(|key| (key == "LOG_FORMAT").then(|| raw.to_owned()));
        let declared = ObserveConfig::from_env_source([("LOG_FORMAT", raw)]);
        assert_same(&legacy, &declared);
        assert_eq!(
            matches!(declared.log_format, LogFormat::Json),
            raw == "json"
        );
    }
}

#[test]
fn parity_dev_console_accepts_true_or_one_only() {
    for raw in ["true", "1", "TRUE", "True", "yes", "false", "0", ""] {
        let legacy = legacy_from_map(|key| (key == "AG_DEV_CONSOLE").then(|| raw.to_owned()));
        let declared = ObserveConfig::from_env_source([("AG_DEV_CONSOLE", raw)]);
        assert_same(&legacy, &declared);
        assert_eq!(declared.dev_console, raw == "true" || raw == "1");
    }
}

#[test]
fn parity_prometheus_port_accepts_any_parseable_u16() {
    for raw in ["0", "1", "9090", "65535", "65536", "-1", " 9090", ""] {
        let legacy = legacy_from_map(|key| (key == "PROMETHEUS_PORT").then(|| raw.to_owned()));
        let declared = ObserveConfig::from_env_source([("PROMETHEUS_PORT", raw)]);
        assert_same(&legacy, &declared);
    }
}

#[test]
fn reads_only_declared_keys() {
    let declared = ObserveConfig::from_env_source([
        ("LOG_FORMAT", "json"),
        ("RUST_LOG", "trace"),
        ("UNDECLARED_KEY", "synthetic-value"),
    ]);
    assert!(matches!(declared.log_format, LogFormat::Json));
    assert_eq!(
        observe_env_declarations().len(),
        6,
        "exactly the six audited keys are declared"
    );
}

#[test]
fn schema_declares_every_expected_key_exactly_once() {
    let declarations = observe_env_declarations();
    assert_eq!(declarations.len(), 6);
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    assert_eq!(
        names,
        vec![
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "PROMETHEUS_PORT",
            "LOG_FORMAT",
            "AG_DEV_CONSOLE",
            "AG_PROJECT_ID",
            "SERVICE_NAME",
        ]
    );
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
    assert!(
        declarations
            .iter()
            .all(|declaration| !declaration.is_secret()),
        "no observability variable is a secret"
    );
}

#[test]
fn service_name_default_matches_the_layer_fallback() {
    let declaration = observe_env_declarations()
        .into_iter()
        .find(|declaration| declaration.name() == "SERVICE_NAME")
        .expect("SERVICE_NAME is declared");
    assert_eq!(declaration.default(), Some("ag-service"));
    assert_eq!(declaration.ty(), EnvType::String);
}
