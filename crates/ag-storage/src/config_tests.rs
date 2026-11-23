//! Parity and regression tests for the declared storage environment schema.

use super::*;

/// Faithful replica of the pre-migration `from_env`, reading an injected
/// map instead of the process environment. Used only as a parity oracle.
fn legacy_from_map(get: impl Fn(&str) -> Option<String>) -> StorageConfig {
    let backend = match get("STORAGE_BACKEND")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "s3" => {
            #[cfg(feature = "s3")]
            {
                StorageBackend::S3
            }
            #[cfg(not(feature = "s3"))]
            {
                StorageBackend::Native
            }
        }
        "minio" => {
            #[cfg(feature = "s3")]
            {
                StorageBackend::MinIO
            }
            #[cfg(not(feature = "s3"))]
            {
                StorageBackend::Native
            }
        }
        _ => StorageBackend::Native,
    };
    StorageConfig {
        backend,
        root_path: get("STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./ag-store-data")),
        server_mode: get("STORAGE_SERVER")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        server_bind: get("STORAGE_BIND")
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1])),
        server_port: get("STORAGE_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(4280),
        allow_insecure_public: get("STORAGE_ALLOW_INSECURE_PUBLIC")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        store_token: get("STORE_TOKEN").unwrap_or_default(),
        jwt_public_key_pem: get("STORAGE_JWT_PUBLIC_KEY").or_else(|| {
            get("STORAGE_JWT_PUBLIC_KEY_PATH").and_then(|path| std::fs::read_to_string(path).ok())
        }),
        max_object_size_mb: get("STORAGE_MAX_OBJECT_SIZE_MB")
            .and_then(|v| v.parse().ok())
            .unwrap_or(100),
        rate_limit_rps: get("STORAGE_RATE_LIMIT_RPS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(100),
        region: get("STORAGE_REGION").unwrap_or_else(|| "us-east-1".into()),
        endpoint: get("STORAGE_ENDPOINT"),
        access_key: get("AWS_ACCESS_KEY_ID"),
        secret_key: get("AWS_SECRET_ACCESS_KEY"),
        bucket: get("STORAGE_BUCKET").unwrap_or_else(|| "ag-storage".into()),
        sign_secret: get("STORAGE_SIGN_SECRET").unwrap_or_default(),
    }
}

fn assert_same(a: &StorageConfig, b: &StorageConfig) {
    assert_eq!(a.backend, b.backend);
    assert_eq!(a.root_path, b.root_path);
    assert_eq!(a.server_mode, b.server_mode);
    assert_eq!(a.server_bind, b.server_bind);
    assert_eq!(a.server_port, b.server_port);
    assert_eq!(a.allow_insecure_public, b.allow_insecure_public);
    assert_eq!(a.store_token, b.store_token);
    assert_eq!(a.jwt_public_key_pem, b.jwt_public_key_pem);
    assert_eq!(a.max_object_size_mb, b.max_object_size_mb);
    assert_eq!(a.rate_limit_rps, b.rate_limit_rps);
    assert_eq!(a.region, b.region);
    assert_eq!(a.endpoint, b.endpoint);
    assert_eq!(a.access_key, b.access_key);
    assert_eq!(a.secret_key, b.secret_key);
    assert_eq!(a.bucket, b.bucket);
    assert_eq!(a.sign_secret, b.sign_secret);
}

#[test]
fn config_defaults_are_native() {
    let cfg = StorageConfig::default();
    assert_eq!(cfg.backend, StorageBackend::Native);
    assert_eq!(cfg.root_path, PathBuf::from("./ag-store-data"));
}

#[test]
fn config_server_mode_off_by_default() {
    let cfg = StorageConfig::default();
    assert!(!cfg.server_mode);
    assert_eq!(cfg.server_port, 4280);
}

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let legacy = legacy_from_map(|_| None);
    let declared = StorageConfig::from_env_source(std::iter::empty::<(&str, &str)>());
    assert_same(&legacy, &declared);
    assert_eq!(declared.root_path, PathBuf::from("./ag-store-data"));
    assert_eq!(declared.server_port, 4280);
    assert_eq!(declared.region, "us-east-1");
    assert_eq!(declared.bucket, "ag-storage");
    assert_eq!(declared.max_object_size_mb, 100);
    assert_eq!(declared.rate_limit_rps, 100);
    assert!(declared.store_token.is_empty());
    assert!(declared.sign_secret.is_empty());
}

#[test]
fn parity_all_keys_set_to_valid_values() {
    let pairs = [
        ("STORAGE_BACKEND", "native"),
        ("STORAGE_ROOT", "/tmp/ag-store"),
        ("STORAGE_SERVER", "TRUE"),
        ("STORAGE_BIND", "0.0.0.0"),
        ("STORAGE_PORT", "9000"),
        ("STORAGE_ALLOW_INSECURE_PUBLIC", "true"),
        ("STORE_TOKEN", "synthetic-static-token"),
        ("STORAGE_JWT_PUBLIC_KEY", "synthetic-inline-key"),
        ("STORAGE_MAX_OBJECT_SIZE_MB", "256"),
        ("STORAGE_RATE_LIMIT_RPS", "42"),
        ("STORAGE_REGION", "eu-west-1"),
        ("STORAGE_ENDPOINT", "http://minio.local:9000"),
        ("AWS_ACCESS_KEY_ID", "synthetic-access-id"),
        ("AWS_SECRET_ACCESS_KEY", "synthetic-aws-secret"),
        ("STORAGE_BUCKET", "custom-bucket"),
        ("STORAGE_SIGN_SECRET", "synthetic-sign-secret"),
    ];
    let get = |key: &str| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_owned())
    };
    let legacy = legacy_from_map(get);
    let declared = StorageConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.server_port, 9000);
    assert!(declared.server_mode);
    assert_eq!(declared.region, "eu-west-1");
}

#[test]
fn parity_malformed_values_fall_back_per_field() {
    let pairs = [
        ("STORAGE_PORT", "not-a-port"),
        ("STORAGE_BIND", "999.999.999.999"),
        ("STORAGE_MAX_OBJECT_SIZE_MB", "-1"),
        ("STORAGE_RATE_LIMIT_RPS", "overflow"),
        ("STORAGE_SERVER", "yes"),
        ("STORAGE_ALLOW_INSECURE_PUBLIC", "1"),
        ("STORAGE_ROOT", ""),
        ("STORAGE_REGION", ""),
    ];
    let get = |key: &str| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_owned())
    };
    let legacy = legacy_from_map(get);
    let declared = StorageConfig::from_env_source(pairs);
    assert_same(&legacy, &declared);
    assert_eq!(declared.server_port, 4280);
    assert!(!declared.server_mode);
    assert!(!declared.allow_insecure_public);
    assert_eq!(declared.server_bind, IpAddr::from([127, 0, 0, 1]));
}

#[test]
fn parity_server_flag_accepts_only_true_literal() {
    for raw in ["true", "TRUE", "True", "1", "yes", "false", ""] {
        let legacy = legacy_from_map(|key| (key == "STORAGE_SERVER").then(|| raw.to_owned()));
        let declared = StorageConfig::from_env_source([("STORAGE_SERVER", raw)]);
        assert_same(&legacy, &declared);
        assert_eq!(declared.server_mode, raw.eq_ignore_ascii_case("true"));
    }
}

#[test]
fn parity_jwt_inline_key_and_path_fallback() {
    let file = tempfile::NamedTempFile::new().expect("temp file");
    std::fs::write(file.path(), "synthetic-key-from-file").expect("write temp file");
    let path = file.path().to_str().expect("utf-8 path").to_owned();

    let cases: [&[(&str, &str)]; 3] = [
        &[("STORAGE_JWT_PUBLIC_KEY", "synthetic-inline")],
        &[("STORAGE_JWT_PUBLIC_KEY_PATH", path.as_str())],
        &[
            ("STORAGE_JWT_PUBLIC_KEY", "synthetic-inline"),
            ("STORAGE_JWT_PUBLIC_KEY_PATH", path.as_str()),
        ],
    ];
    for pairs in cases {
        let get = |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        };
        assert_same(
            &legacy_from_map(get),
            &StorageConfig::from_env_source(pairs.iter().copied()),
        );
    }

    let from_file =
        StorageConfig::from_env_source([("STORAGE_JWT_PUBLIC_KEY_PATH", path.as_str())]);
    assert_eq!(
        from_file.jwt_public_key_pem.as_deref(),
        Some("synthetic-key-from-file")
    );
    let from_inline = StorageConfig::from_env_source([("STORAGE_JWT_PUBLIC_KEY", "inline")]);
    assert_eq!(from_inline.jwt_public_key_pem.as_deref(), Some("inline"));
}

#[test]
fn schema_marks_only_sensitive_keys_as_secret() {
    let declarations = storage_env_declarations();
    let secrets: Vec<&str> = declarations
        .iter()
        .filter(|d| d.is_secret())
        .map(EnvDeclaration::name)
        .collect();
    assert_eq!(
        secrets,
        vec![
            "STORE_TOKEN",
            "STORAGE_JWT_PUBLIC_KEY",
            "AWS_SECRET_ACCESS_KEY",
            "STORAGE_SIGN_SECRET",
        ]
    );
}

#[test]
fn schema_declares_every_expected_key_exactly_once() {
    let declarations = storage_env_declarations();
    assert_eq!(declarations.len(), 17);
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn declared_secrets_are_redacted_and_reachable_only_through_expose() {
    let env = storage_loader()
        .load_from([
            ("STORE_TOKEN", "synthetic-static-token"),
            ("AWS_SECRET_ACCESS_KEY", "synthetic-aws-secret"),
            ("STORAGE_SIGN_SECRET", "synthetic-sign-secret"),
            ("STORAGE_JWT_PUBLIC_KEY", "synthetic-inline-key"),
        ])
        .expect("all declarations are optional strings");
    let rendered = format!("{env:?}");
    for secret in [
        "synthetic-static-token",
        "synthetic-aws-secret",
        "synthetic-sign-secret",
        "synthetic-inline-key",
    ] {
        assert!(!rendered.contains(secret));
    }
    assert_eq!(
        env.secret("STORE_TOKEN").map(|v| v.expose().as_str()),
        Some("synthetic-static-token")
    );
    assert!(format!("{:?}", env.secret("STORE_TOKEN")).contains("redacted"));
}

#[test]
fn public_bind_without_token_is_rejected() {
    let config = StorageConfig {
        server_mode: true,
        server_bind: IpAddr::from([0, 0, 0, 0]),
        ..StorageConfig::default()
    };

    assert!(matches!(
        config.validate_server_security(),
        Err(StorageError::Config(_))
    ));
}

#[test]
fn loopback_bind_without_token_is_allowed() {
    let config = StorageConfig {
        server_mode: true,
        ..StorageConfig::default()
    };

    assert!(config.validate_server_security().is_ok());
}

#[test]
fn explicit_insecure_public_bind_is_allowed() {
    let config = StorageConfig {
        server_mode: true,
        server_bind: IpAddr::from([0, 0, 0, 0]),
        allow_insecure_public: true,
        ..StorageConfig::default()
    };

    assert!(config.validate_server_security().is_ok());
}

#[test]
fn public_bind_with_token_is_allowed() {
    let config = StorageConfig {
        server_mode: true,
        server_bind: IpAddr::from([0, 0, 0, 0]),
        store_token: "secret".into(),
        ..StorageConfig::default()
    };

    assert!(config.validate_server_security().is_ok());
}

#[test]
fn config_sign_secret_default_empty() {
    let cfg = StorageConfig::default();
    assert!(cfg.sign_secret.is_empty());
}
