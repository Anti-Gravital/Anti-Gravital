//! Parity, schema and redaction tests for the declared cache environment.
//!
//! The parity oracle below is a faithful replica of the pre-migration
//! `from_env`: it reads an injected map instead of the process environment and
//! applies the same parsing/fallback rules byte for byte. No test mutates the
//! process environment (audit A1).

use super::*;

/// Faithful replica of the pre-migration `from_env`, reading an injected map
/// instead of the process environment. Used only as a parity oracle.
fn legacy_from_env(get: impl Fn(&str) -> Option<String>) -> CacheConfig {
    CacheConfig {
        l1_max_capacity: get("CACHE_L1_MAX_CAPACITY")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000),
        l1_ttl_secs: get("CACHE_L1_TTL_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(300),
        redis_url: get("REDIS_URL"),
        redis_pool_size: get("REDIS_POOL_SIZE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        native_server_enabled: get("CACHE_NATIVE_SERVER")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false),
        native_server_port: get("CACHE_NATIVE_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(6379),
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

fn assert_same(a: &CacheConfig, b: &CacheConfig) {
    assert_eq!(a.l1_max_capacity, b.l1_max_capacity);
    assert_eq!(a.l1_ttl_secs, b.l1_ttl_secs);
    assert_eq!(a.redis_url, b.redis_url);
    assert_eq!(a.redis_pool_size, b.redis_pool_size);
    assert_eq!(a.native_server_enabled, b.native_server_enabled);
    assert_eq!(a.native_server_port, b.native_server_port);
}

// ---------------------------------------------------------------------------
// Pre-existing unit tests (behavior unchanged by the migration).
// ---------------------------------------------------------------------------

#[test]
fn default_has_no_redis() {
    let cfg = CacheConfig::default();
    assert!(cfg.redis_url.is_none());
    assert_eq!(cfg.l1_max_capacity, 10_000);
    assert_eq!(cfg.l1_ttl_secs, 300);
    assert_eq!(cfg.redis_pool_size, 10);
}

#[test]
fn native_server_defaults_disabled() {
    let cfg = CacheConfig::default();
    assert!(!cfg.native_server_enabled);
    assert_eq!(cfg.native_server_port, 6379);
}

// ---------------------------------------------------------------------------
// Parity tests: declared loader vs. the pre-migration direct reads.
// ---------------------------------------------------------------------------

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let legacy = legacy_from_env(|_| None);
    let declared = CacheConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&legacy, &declared);
    assert_same(&declared, &CacheConfig::default());
    assert!(declared.redis_url.is_none());
}

#[test]
fn parity_l2_disabled_when_no_redis_url() {
    // Replaces the old test that called std::env::remove_var("REDIS_URL").
    // The injected source keeps the same assertion without global races.
    let legacy = legacy_from_env(|_| None);
    let declared = CacheConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&legacy, &declared);
    assert!(declared.redis_url.is_none());
}

#[test]
fn parity_all_keys_set_to_valid_values() {
    let pairs: [(&str, &str); 6] = [
        ("CACHE_L1_MAX_CAPACITY", "2048"),
        ("CACHE_L1_TTL_SECS", "45"),
        ("REDIS_URL", "redis://cache.example.invalid:6379/0"),
        ("REDIS_POOL_SIZE", "25"),
        ("CACHE_NATIVE_SERVER", "TRUE"),
        ("CACHE_NATIVE_PORT", "6380"),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = CacheConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.l1_max_capacity, 2048);
    assert_eq!(declared.l1_ttl_secs, 45);
    assert_eq!(
        declared.redis_url.as_deref(),
        Some("redis://cache.example.invalid:6379/0")
    );
    assert_eq!(declared.redis_pool_size, 25);
    assert!(declared.native_server_enabled);
    assert_eq!(declared.native_server_port, 6380);
}

#[test]
fn parity_numeric_fields_fall_back_per_field() {
    let pairs: [(&str, &str); 6] = [
        ("CACHE_L1_MAX_CAPACITY", "not-a-number"),
        ("CACHE_L1_TTL_SECS", " 42 "),
        ("REDIS_POOL_SIZE", "-1"),
        ("CACHE_NATIVE_PORT", "70000"),
        ("CACHE_NATIVE_SERVER", "yes"),
        ("UNRELATED_KEY", "ignored"),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = CacheConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    // Each malformed value keeps its own default, not a shared one.
    assert_eq!(declared.l1_max_capacity, 10_000);
    assert_eq!(declared.l1_ttl_secs, 300);
    assert_eq!(declared.redis_pool_size, 10);
    assert_eq!(declared.native_server_port, 6379);
    // "yes" is not accepted by the original bool expression.
    assert!(!declared.native_server_enabled);
}

#[test]
fn parity_bool_accepts_one_and_true_case_insensitive() {
    for raw in [
        "1", "true", "TRUE", "True", "0", "false", "no", "", " true ",
    ] {
        let legacy = legacy_from_env(|key| (key == "CACHE_NATIVE_SERVER").then(|| raw.to_owned()));
        let declared = CacheConfig::from_env_source([("CACHE_NATIVE_SERVER", raw)]);
        assert_same(&legacy, &declared);
        assert_eq!(
            declared.native_server_enabled,
            raw == "1" || raw.to_lowercase() == "true",
            "raw={raw}"
        );
    }
}

#[test]
fn parity_empty_values_are_preserved() {
    // std::env::var returns Ok("") for a present-but-empty variable. An empty
    // REDIS_URL stays Some("") (L2 configured with an empty URL); the empty
    // numeric/bool values fall back to their defaults.
    let pairs: [(&str, &str); 6] = [
        ("CACHE_L1_MAX_CAPACITY", ""),
        ("CACHE_L1_TTL_SECS", ""),
        ("REDIS_URL", ""),
        ("REDIS_POOL_SIZE", ""),
        ("CACHE_NATIVE_SERVER", ""),
        ("CACHE_NATIVE_PORT", ""),
    ];
    let legacy = legacy_from_env(get_from(&pairs));
    let declared = CacheConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.redis_url.as_deref(), Some(""));
    assert_eq!(declared.l1_max_capacity, 10_000);
    assert_eq!(declared.l1_ttl_secs, 300);
    assert_eq!(declared.redis_pool_size, 10);
    assert_eq!(declared.native_server_port, 6379);
    assert!(!declared.native_server_enabled);
}

#[test]
fn undeclared_keys_are_ignored() {
    let pairs: [(&str, &str); 3] = [
        ("CACHE_L1_MAX_CAPACITY", "512"),
        ("RUST_LOG", "trace"),
        ("UNDECLARED_SECRET", "synthetic-canary-must-not-leak"),
    ];
    let cfg = CacheConfig::from_env_source(pairs);
    assert_eq!(cfg.l1_max_capacity, 512);
    assert!(cfg.redis_url.is_none());
}

// ---------------------------------------------------------------------------
// Schema / redaction tests.
// ---------------------------------------------------------------------------

#[test]
fn schema_declares_every_expected_key_exactly_once() {
    let declarations = cache_env_declarations();
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    assert_eq!(
        names,
        vec![
            "CACHE_L1_MAX_CAPACITY",
            "CACHE_L1_TTL_SECS",
            "REDIS_URL",
            "REDIS_POOL_SIZE",
            "CACHE_NATIVE_SERVER",
            "CACHE_NATIVE_PORT",
        ]
    );
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
    assert!(declarations.iter().all(|declaration| {
        declaration.ty() == EnvType::String
            && !declaration.is_required()
            && declaration.default().is_none()
    }));
}

#[test]
fn schema_marks_only_redis_url_as_secret() {
    let declarations = cache_env_declarations();
    let secrets: Vec<&str> = declarations
        .iter()
        .filter(|d| d.is_secret())
        .map(EnvDeclaration::name)
        .collect();
    assert_eq!(secrets, vec!["REDIS_URL"]);
    let non_secrets: Vec<&str> = declarations
        .iter()
        .filter(|d| !d.is_secret())
        .map(EnvDeclaration::name)
        .collect();
    assert_eq!(
        non_secrets,
        vec![
            "CACHE_L1_MAX_CAPACITY",
            "CACHE_L1_TTL_SECS",
            "REDIS_POOL_SIZE",
            "CACHE_NATIVE_SERVER",
            "CACHE_NATIVE_PORT",
        ]
    );
}

#[test]
fn redis_url_secret_is_redacted_by_the_loader() {
    const CANARY: &str = "synthetic-canary-redis-url-value";
    let env = cache_loader()
        .load_from([("REDIS_URL", CANARY)])
        .expect("all declarations are optional strings");

    let rendered = format!("{env:?}");
    assert!(!rendered.contains(CANARY));
    assert!(!rendered.contains("synthetic-canary"));
    assert!(rendered.contains("[redacted]"));

    let secret = env.secret("REDIS_URL").expect("secret value");
    assert!(!format!("{secret:?}").contains(CANARY));
    assert!(!format!("{secret}").contains(CANARY));
    assert!(format!("{secret:?}").contains("redacted"));
    assert!(format!("{secret}").contains("redacted"));

    // The only way to read the value back is an explicit expose() call.
    assert_eq!(secret.expose(), CANARY);
}

#[test]
fn redis_url_value_reaches_config_through_expose() {
    const CANARY: &str = "synthetic-redis-url-value";
    let cfg = CacheConfig::from_env_source([("REDIS_URL", CANARY)]);
    assert_eq!(cfg.redis_url.as_deref(), Some(CANARY));
}
