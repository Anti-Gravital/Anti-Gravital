//! Parity and regression tests for the declared authentication environment schema.

use super::*;

/// Faithful replica of the pre-migration `from_env`, reading an injected map
/// instead of the process environment. Used only as a parity oracle.
fn legacy_from_env(get: impl Fn(&str) -> Option<String>) -> Result<AuthConfig, AuthConfigError> {
    let jwt_private_key_pem =
        get("JWT_PRIVATE_KEY").ok_or(AuthConfigError::MissingVar("JWT_PRIVATE_KEY"))?;
    let jwt_public_key_pem =
        get("JWT_PUBLIC_KEY").ok_or(AuthConfigError::MissingVar("JWT_PUBLIC_KEY"))?;
    let webauthn_rp_id = get("WEBAUTHN_RP_ID").unwrap_or_else(|| "localhost".to_string());
    let webauthn_origin =
        get("WEBAUTHN_ORIGIN").unwrap_or_else(|| "http://localhost:8080".to_string());
    Ok(AuthConfig {
        jwt_private_key_pem,
        jwt_public_key_pem,
        webauthn_rp_id,
        webauthn_origin,
        oauth_google_client_id: get("GOOGLE_CLIENT_ID"),
        oauth_google_client_secret: get("GOOGLE_CLIENT_SECRET"),
        oauth_github_client_id: get("GITHUB_CLIENT_ID"),
        oauth_github_client_secret: get("GITHUB_CLIENT_SECRET"),
    })
}

fn getter<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_owned())
    }
}

fn assert_same(a: &AuthConfig, b: &AuthConfig) {
    assert_eq!(a.jwt_private_key_pem, b.jwt_private_key_pem);
    assert_eq!(a.jwt_public_key_pem, b.jwt_public_key_pem);
    assert_eq!(a.webauthn_rp_id, b.webauthn_rp_id);
    assert_eq!(a.webauthn_origin, b.webauthn_origin);
    assert_eq!(a.oauth_google_client_id, b.oauth_google_client_id);
    assert_eq!(a.oauth_google_client_secret, b.oauth_google_client_secret);
    assert_eq!(a.oauth_github_client_id, b.oauth_github_client_id);
    assert_eq!(a.oauth_github_client_secret, b.oauth_github_client_secret);
}

fn assert_same_error(a: &AuthConfigError, b: &AuthConfigError) {
    match (a, b) {
        (AuthConfigError::MissingVar(x), AuthConfigError::MissingVar(y)) => assert_eq!(x, y),
    }
}

#[test]
fn parity_missing_jwt_keys_errors_identically() {
    let legacy = legacy_from_env(|_| None).unwrap_err();
    let declared = AuthConfig::from_env_source(std::iter::empty::<(&str, &str)>()).unwrap_err();
    assert_same_error(&legacy, &declared);
    assert!(matches!(
        declared,
        AuthConfigError::MissingVar("JWT_PRIVATE_KEY")
    ));

    // Only the public key missing: the original reports JWT_PUBLIC_KEY after
    // the private key resolves successfully.
    let pairs = [("JWT_PRIVATE_KEY", "synthetic-private")];
    let legacy = legacy_from_env(getter(&pairs)).unwrap_err();
    let declared = AuthConfig::from_env_source(pairs).unwrap_err();
    assert_same_error(&legacy, &declared);
    assert!(matches!(
        declared,
        AuthConfigError::MissingVar("JWT_PUBLIC_KEY")
    ));
}

#[test]
fn parity_reads_jwt_keys() {
    let pairs = [
        ("JWT_PRIVATE_KEY", "synthetic-private"),
        ("JWT_PUBLIC_KEY", "synthetic-public"),
    ];
    let legacy = legacy_from_env(getter(&pairs)).expect("legacy resolves");
    let declared = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_same(&legacy, &declared);
    assert_eq!(declared.jwt_private_key_pem, "synthetic-private");
    assert_eq!(declared.jwt_public_key_pem, "synthetic-public");
}

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let pairs = [
        ("JWT_PRIVATE_KEY", "synthetic-private"),
        ("JWT_PUBLIC_KEY", "synthetic-public"),
    ];
    let legacy = legacy_from_env(getter(&pairs)).expect("legacy resolves");
    let declared = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_same(&legacy, &declared);
    assert_eq!(declared.webauthn_rp_id, "localhost");
    assert_eq!(declared.webauthn_origin, "http://localhost:8080");
    assert!(declared.oauth_google_client_id.is_none());
    assert!(declared.oauth_google_client_secret.is_none());
    assert!(declared.oauth_github_client_id.is_none());
    assert!(declared.oauth_github_client_secret.is_none());
}

#[test]
fn parity_all_keys_set_to_valid_values() {
    let pairs = [
        ("JWT_PRIVATE_KEY", "synthetic-private"),
        ("JWT_PUBLIC_KEY", "synthetic-public"),
        ("WEBAUTHN_RP_ID", "example.com"),
        ("WEBAUTHN_ORIGIN", "https://example.com"),
        ("GOOGLE_CLIENT_ID", "synthetic-google-id"),
        ("GOOGLE_CLIENT_SECRET", "synthetic-google-secret"),
        ("GITHUB_CLIENT_ID", "synthetic-github-id"),
        ("GITHUB_CLIENT_SECRET", "synthetic-github-secret"),
    ];
    let legacy = legacy_from_env(getter(&pairs)).expect("legacy resolves");
    let declared = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_same(&legacy, &declared);
    assert_eq!(declared.webauthn_rp_id, "example.com");
    assert_eq!(declared.webauthn_origin, "https://example.com");
    assert_eq!(
        declared.oauth_google_client_id.as_deref(),
        Some("synthetic-google-id")
    );
    assert_eq!(
        declared.oauth_github_client_secret.as_deref(),
        Some("synthetic-github-secret")
    );
}

#[test]
fn parity_empty_values_are_preserved() {
    // `std::env::var` returns Ok("") for a present-but-empty variable. Empty
    // JWT keys are accepted (not MissingVar) and WebAuthn defaults must NOT
    // replace an explicitly empty value.
    let pairs = [
        ("JWT_PRIVATE_KEY", ""),
        ("JWT_PUBLIC_KEY", ""),
        ("WEBAUTHN_RP_ID", ""),
        ("WEBAUTHN_ORIGIN", ""),
        ("GOOGLE_CLIENT_ID", ""),
        ("GOOGLE_CLIENT_SECRET", ""),
        ("GITHUB_CLIENT_ID", ""),
        ("GITHUB_CLIENT_SECRET", ""),
    ];
    let legacy = legacy_from_env(getter(&pairs)).expect("legacy resolves");
    let declared = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_same(&legacy, &declared);
    assert!(declared.jwt_private_key_pem.is_empty());
    assert!(declared.webauthn_rp_id.is_empty());
    assert_eq!(declared.oauth_google_client_id.as_deref(), Some(""));
    assert_eq!(declared.oauth_github_client_secret.as_deref(), Some(""));
}

#[test]
fn parity_partial_oauth_configuration() {
    // One provider fully configured, the other only with a client secret:
    // each field resolves independently with Option semantics.
    let pairs = [
        ("JWT_PRIVATE_KEY", "synthetic-private"),
        ("JWT_PUBLIC_KEY", "synthetic-public"),
        ("GOOGLE_CLIENT_ID", "synthetic-google-id"),
        ("GITHUB_CLIENT_SECRET", "synthetic-github-secret"),
    ];
    let legacy = legacy_from_env(getter(&pairs)).expect("legacy resolves");
    let declared = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_same(&legacy, &declared);
    assert_eq!(
        declared.oauth_google_client_id.as_deref(),
        Some("synthetic-google-id")
    );
    assert!(declared.oauth_google_client_secret.is_none());
    assert!(declared.oauth_github_client_id.is_none());
    assert_eq!(
        declared.oauth_github_client_secret.as_deref(),
        Some("synthetic-github-secret")
    );
}

#[test]
fn undeclared_keys_are_ignored() {
    // The loader reads only declared keys, so an unrelated key carrying a
    // secret-looking value cannot influence the config.
    let pairs = [
        ("JWT_PRIVATE_KEY", "synthetic-private"),
        ("JWT_PUBLIC_KEY", "synthetic-public"),
        ("UNRELATED_SECRET", "synthetic-canary-must-not-leak"),
    ];
    let cfg = AuthConfig::from_env_source(pairs).expect("declared resolves");
    assert_eq!(cfg.jwt_private_key_pem, "synthetic-private");
    assert!(cfg.oauth_google_client_id.is_none());
}

#[test]
fn schema_marks_only_sensitive_keys_as_secret() {
    let declarations = auth_env_declarations();
    let secrets: Vec<&str> = declarations
        .iter()
        .filter(|d| d.is_secret())
        .map(EnvDeclaration::name)
        .collect();
    assert_eq!(
        secrets,
        vec![
            "JWT_PRIVATE_KEY",
            "JWT_PUBLIC_KEY",
            "GOOGLE_CLIENT_SECRET",
            "GITHUB_CLIENT_SECRET",
        ]
    );
}

#[test]
fn schema_declares_every_expected_key_exactly_once() {
    let declarations = auth_env_declarations();
    assert_eq!(declarations.len(), 8);
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    assert_eq!(
        names,
        vec![
            "JWT_PRIVATE_KEY",
            "JWT_PUBLIC_KEY",
            "WEBAUTHN_RP_ID",
            "WEBAUTHN_ORIGIN",
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
        ]
    );
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn schema_declares_documented_defaults() {
    let declarations = auth_env_declarations();
    let rp = declarations
        .iter()
        .find(|d| d.name() == "WEBAUTHN_RP_ID")
        .expect("rp id declared");
    assert_eq!(rp.default(), Some("localhost"));
    let origin = declarations
        .iter()
        .find(|d| d.name() == "WEBAUTHN_ORIGIN")
        .expect("origin declared");
    assert_eq!(origin.default(), Some("http://localhost:8080"));
    // JWT keys must not have a default: absence is an error, not a fallback.
    for name in ["JWT_PRIVATE_KEY", "JWT_PUBLIC_KEY"] {
        let decl = declarations
            .iter()
            .find(|d| d.name() == name)
            .expect("jwt key declared");
        assert!(decl.default().is_none());
        assert!(!decl.is_required());
    }
}

#[test]
fn declared_secrets_are_redacted_and_reachable_only_through_expose() {
    const CANARY: &str = "synthetic-canary-private-key";
    let env = auth_loader()
        .load_from([("JWT_PRIVATE_KEY", CANARY)])
        .expect("all declarations are optional strings");

    let rendered = format!("{env:?}");
    assert!(!rendered.contains(CANARY));
    assert!(rendered.contains("[redacted]"));

    let secret = env.secret("JWT_PRIVATE_KEY").expect("secret value");
    assert!(!format!("{secret:?}").contains(CANARY));
    assert!(!format!("{secret}").contains(CANARY));
    assert!(format!("{secret:?}").contains("redacted"));
    assert!(format!("{secret}").contains("redacted"));

    // The only way to read the value back is an explicit expose() call.
    assert_eq!(secret.expose(), CANARY);
}

#[test]
fn errors_never_contain_a_secret_value() {
    // MissingVar carries only the variable name, never a value (S-1/S-8).
    let error = AuthConfigError::MissingVar("JWT_PRIVATE_KEY");
    let rendered = error.to_string();
    assert_eq!(
        rendered,
        "required environment variable not set: JWT_PRIVATE_KEY"
    );
    assert!(!rendered.contains("synthetic-canary"));
}
