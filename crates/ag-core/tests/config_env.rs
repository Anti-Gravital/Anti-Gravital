//! Opt-in environment validation wired into config loading (RFC-0020 stage B).
//!
//! These tests exercise only the public API: `ShieldConfig::from_path_with_env`
//! and `ShieldConfig::from_toml_str_with_env` validate the process environment
//! before returning, while plain `from_path` keeps reading no variables.

use std::path::PathBuf;

use ag_core::{EnvDeclaration, EnvLoader, EnvType, ShieldConfig};

const OK_KEY: &str = "AG_CORE_CONFIG_ENV_TEST_OK";
const BAD_KEY: &str = "AG_CORE_CONFIG_ENV_TEST_BAD";
const UNSET_KEY: &str = "AG_CORE_CONFIG_ENV_TEST_UNSET";

struct EnvGuard(&'static str);

impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.0);
    }
}

fn example_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.example.toml")
}

#[test]
fn from_path_does_not_read_the_environment_by_default() {
    // The plain loader stays non-validating even when a declared-style
    // variable is malformed. This locks in the "do not change defaults"
    // requirement of RFC-0020 stage B.
    std::env::set_var(BAD_KEY, "definitely-not-an-integer");
    let _guard = EnvGuard(BAD_KEY);
    let config = ShieldConfig::from_path(example_path()).expect("plain load ignores env");
    assert!(!config.cors.enabled);
}

#[test]
fn from_path_with_env_returns_config_and_validated_env() {
    std::env::set_var(OK_KEY, "8443");
    let _guard = EnvGuard(OK_KEY);
    let loader = EnvLoader::new(vec![EnvDeclaration::new(OK_KEY, EnvType::Int)]);

    let (config, env) = ShieldConfig::from_path_with_env(example_path(), &loader)
        .expect("declared value resolves and config parses");
    assert!(!config.cors.enabled);
    assert_eq!(env.int(OK_KEY), Some(8443));
}

#[test]
fn from_path_with_env_fails_closed_on_a_missing_declared_variable() {
    std::env::remove_var(UNSET_KEY);
    let loader = EnvLoader::new(vec![EnvDeclaration::new(UNSET_KEY, EnvType::Url)]);

    let error = ShieldConfig::from_path_with_env(example_path(), &loader)
        .expect_err("missing required variable fails startup");
    assert_eq!(error.code(), "config_error");
    let rendered = error.to_string();
    assert!(rendered.contains(UNSET_KEY));
    assert!(rendered.contains("environment validation failed"));
    assert!(!rendered.contains("8443"));
}

#[test]
fn from_path_with_env_reports_malformed_value_without_leaking_it() {
    const MALFORMED: &str = "secretish-not-an-integer";
    std::env::set_var(BAD_KEY, MALFORMED);
    let _guard = EnvGuard(BAD_KEY);
    let loader = EnvLoader::new(vec![EnvDeclaration::new(BAD_KEY, EnvType::Int)]);

    let error = ShieldConfig::from_path_with_env(example_path(), &loader)
        .expect_err("malformed declared value fails startup");
    let rendered = error.to_string();
    assert!(!rendered.contains(MALFORMED), "error leaked the raw value");
    assert!(rendered.contains(BAD_KEY));
    assert!(rendered.contains("expected int"));
}

#[test]
fn from_toml_str_with_env_validates_and_returns_both() {
    std::env::set_var(OK_KEY, "true");
    let _guard = EnvGuard(OK_KEY);
    let loader = EnvLoader::new(vec![EnvDeclaration::new(OK_KEY, EnvType::Bool).secret(true)]);

    let (config, env) = ShieldConfig::from_toml_str_with_env(r#"bind = "127.0.0.1:9001""#, &loader)
        .expect("valid config and env");
    assert_eq!(config.bind.port(), 9001);
    let secret = env.secret(OK_KEY).expect("secret value");
    assert_eq!(secret.expose(), "true");
    assert_eq!(format!("{secret:?}"), "[redacted]");
}
