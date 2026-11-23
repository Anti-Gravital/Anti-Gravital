//! Parity, schema and regression tests for the declared workers environment.
//!
//! The parity oracle below is a faithful replica of the pre-migration
//! `apply_env`: it reads an injected map instead of the process environment and
//! applies the same parsing/fallback rules byte for byte. No test mutates the
//! process environment (audit A1).

use super::*;

/// Faithful replica of the pre-migration `apply_env`, reading an injected map
/// instead of the process environment. Used only as a parity oracle.
fn legacy_apply(
    mut cfg: WorkersConfig,
    get: impl Fn(&str) -> Option<String>,
) -> Result<WorkersConfig, ConfigError> {
    if let Some(value) = get("AG_WORKERS_ENABLED") {
        cfg.enabled = matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        );
    }
    if let Some(value) = get("AG_WORKERS_MODE") {
        cfg.mode = value.parse()?;
    }
    if let Some(value) = get("AG_WORKERS_BACKEND") {
        cfg.backend = value.parse()?;
    }
    if let Some(value) = get("AG_WORKERS_SHUTDOWN_TIMEOUT") {
        cfg.shutdown_timeout = parse_duration(&value)?;
    }
    if let Some(value) =
        get("AG_WORKERS_MAX_PAYLOAD_BYTES").and_then(|v| v.trim().parse::<usize>().ok())
    {
        cfg.max_payload_bytes = value;
    }
    if let Some(value) =
        get("AG_WORKERS_POISON_GUARD_ATTEMPTS").and_then(|v| v.trim().parse::<u32>().ok())
    {
        cfg.poison_guard_attempts = value;
    }
    // Per-queue MIN/MAX overrides for queues already declared in the config.
    let names: Vec<String> = cfg.queues.keys().cloned().collect();
    for name in names {
        let upper = name.to_ascii_uppercase();
        if let Some(value) = get(&format!("AG_WORKERS_QUEUE_{upper}_MIN"))
            .and_then(|v| v.trim().parse::<usize>().ok())
        {
            cfg.queues.get_mut(&name).unwrap().min_workers = value;
        }
        if let Some(value) = get(&format!("AG_WORKERS_QUEUE_{upper}_MAX"))
            .and_then(|v| v.trim().parse::<usize>().ok())
        {
            cfg.queues.get_mut(&name).unwrap().max_workers = value;
        }
    }
    Ok(cfg)
}

fn get_from<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key: &str| {
        pairs
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned())
    }
}

fn base_with_queues() -> WorkersConfig {
    WorkersConfig::from_toml_str(
        r#"
        [workers]
        mode = "embedded"
        max_payload_bytes = 100
        [workers.queue.mail]
        min_workers = 1
        max_workers = 4
        max_depth = 50
        [workers.queue.media]
        min_workers = 2
        max_workers = 6
        "#,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Pre-existing unit tests (behavior unchanged by the migration).
// ---------------------------------------------------------------------------

#[test]
fn default_workers_config_is_embedded_memory() {
    let cfg = WorkersConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.mode, WorkerMode::Embedded);
    assert_eq!(cfg.backend, BackendKind::Memory);
    assert_eq!(cfg.shutdown_timeout, Duration::from_secs(30));
    assert_eq!(cfg.max_payload_bytes, 262_144);
    assert!(cfg.queues.is_empty());
}

#[test]
fn empty_toml_yields_default() {
    let cfg = WorkersConfig::from_toml_str("# nothing here\n").unwrap();
    assert_eq!(cfg, WorkersConfig::default());
}

#[test]
fn full_toml_parses_all_fields() {
    let src = r#"
        [workers]
        enabled = true
        mode = "distributed"
        backend = "postgres"
        shutdown_timeout = "45s"
        max_payload_bytes = 131072
        poison_guard_attempts = 5

        [workers.queue.mail]
        min_workers = 2
        max_workers = 8
        max_depth = 5000
        lease_timeout = "90s"
        heartbeat_interval = "10s"

        [workers.queue.media]
        kind = "cpu"
        max_workers = 4
    "#;
    let cfg = WorkersConfig::from_toml_str(src).unwrap();
    assert_eq!(cfg.mode, WorkerMode::Distributed);
    assert_eq!(cfg.backend, BackendKind::Postgres);
    assert_eq!(cfg.shutdown_timeout, Duration::from_secs(45));
    assert_eq!(cfg.max_payload_bytes, 131_072);
    assert_eq!(cfg.poison_guard_attempts, 5);

    let mail = &cfg.queues["mail"];
    assert_eq!(mail.min_workers, 2);
    assert_eq!(mail.max_workers, 8);
    assert_eq!(mail.lease_timeout, Duration::from_secs(90));
    assert_eq!(mail.heartbeat_interval, Duration::from_secs(10));
    assert!(!mail.cpu_bound);

    let media = &cfg.queues["media"];
    assert!(media.cpu_bound);
    assert_eq!(media.max_workers, 4);
    assert_eq!(media.max_depth, QueueConfig::default().max_depth);
}

#[test]
fn unknown_mode_is_rejected() {
    let src = "[workers]\nmode = \"galactic\"\n";
    assert!(matches!(
        WorkersConfig::from_toml_str(src),
        Err(ConfigError::Variant { field: "mode", .. })
    ));
}

#[test]
fn parse_duration_units() {
    assert_eq!(parse_duration("250ms").unwrap(), Duration::from_millis(250));
    assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
    assert_eq!(parse_duration("12").unwrap(), Duration::from_secs(12));
    assert!(parse_duration("soon").is_err());
}

#[test]
fn parse_duration_days_and_weeks() {
    assert_eq!(
        parse_duration("2d").unwrap(),
        Duration::from_secs(2 * 86_400)
    );
    assert_eq!(parse_duration("1w").unwrap(), Duration::from_secs(604_800));
    assert!(parse_duration("5x").is_err());
}

#[test]
fn runtime_config_carries_poison_guard() {
    let cfg = WorkersConfig {
        poison_guard_attempts: 7,
        ..WorkersConfig::default()
    };
    assert_eq!(cfg.runtime_config().poison_guard_attempts, 7);
}

#[test]
fn worker_mode_from_str_all_variants() {
    assert_eq!(
        "embedded".parse::<WorkerMode>().unwrap(),
        WorkerMode::Embedded
    );
    assert_eq!(
        "  Standalone ".parse::<WorkerMode>().unwrap(),
        WorkerMode::Standalone
    );
    assert_eq!(
        "DISTRIBUTED".parse::<WorkerMode>().unwrap(),
        WorkerMode::Distributed
    );
    assert_eq!(
        "producer".parse::<WorkerMode>().unwrap(),
        WorkerMode::Producer
    );
    assert!("galactic".parse::<WorkerMode>().is_err());
}

#[test]
fn backend_kind_from_str_all_variants() {
    assert_eq!(
        "memory".parse::<BackendKind>().unwrap(),
        BackendKind::Memory
    );
    for pg in ["postgres", "postgresql", "PG"] {
        assert_eq!(pg.parse::<BackendKind>().unwrap(), BackendKind::Postgres);
    }
    assert!("sqlite".parse::<BackendKind>().is_err());
}

#[test]
fn env_overrides_win_over_toml() {
    // Previously used process set_var; the injected source keeps the same
    // assertion without global-environment races (audit A1).
    let src = "[workers]\nmode = \"embedded\"\nmax_payload_bytes = 1\n[workers.queue.mail]\nmax_workers = 8\n";
    let base = WorkersConfig::from_toml_str(src).unwrap();
    let cfg = base
        .apply_env_source([
            ("AG_WORKERS_MODE", "standalone"),
            ("AG_WORKERS_MAX_PAYLOAD_BYTES", "999"),
            ("AG_WORKERS_QUEUE_MAIL_MAX", "16"),
        ])
        .unwrap();
    assert_eq!(cfg.mode, WorkerMode::Standalone);
    assert_eq!(cfg.max_payload_bytes, 999);
    assert_eq!(cfg.queues["mail"].max_workers, 16);
}

// ---------------------------------------------------------------------------
// Parity tests: declared loader vs. the pre-migration direct reads.
// ---------------------------------------------------------------------------

#[test]
fn parity_defaults_when_every_key_is_unset() {
    let base = WorkersConfig::default();
    let legacy = legacy_apply(base.clone(), |_| None).unwrap();
    let declared = base
        .apply_env_source(std::iter::empty::<(&str, &str)>())
        .unwrap();
    assert_eq!(legacy, declared);
    assert_eq!(declared, WorkersConfig::default());
}

#[test]
fn parity_all_fixed_keys_set_to_valid_values() {
    let pairs: [(&str, &str); 6] = [
        ("AG_WORKERS_ENABLED", "false"),
        ("AG_WORKERS_MODE", "standalone"),
        ("AG_WORKERS_BACKEND", "postgres"),
        ("AG_WORKERS_SHUTDOWN_TIMEOUT", "45s"),
        ("AG_WORKERS_MAX_PAYLOAD_BYTES", "999"),
        ("AG_WORKERS_POISON_GUARD_ATTEMPTS", "5"),
    ];
    let base = WorkersConfig::default();
    let legacy = legacy_apply(base.clone(), get_from(&pairs)).unwrap();
    let declared = base.apply_env_source(pairs).unwrap();
    assert_eq!(legacy, declared);
    assert!(!declared.enabled);
    assert_eq!(declared.mode, WorkerMode::Standalone);
    assert_eq!(declared.backend, BackendKind::Postgres);
    assert_eq!(declared.shutdown_timeout, Duration::from_secs(45));
    assert_eq!(declared.max_payload_bytes, 999);
    assert_eq!(declared.poison_guard_attempts, 5);
}

#[test]
fn parity_mode_accepts_only_known_variants() {
    for raw in [
        "embedded",
        "STANDALONE",
        " distributed ",
        "producer",
        "galactic",
        "",
        "1",
    ] {
        let base = WorkersConfig::default();
        let legacy = legacy_apply(base.clone(), |key| {
            (key == "AG_WORKERS_MODE").then(|| raw.to_owned())
        });
        let declared = base.apply_env_source([("AG_WORKERS_MODE", raw)]);
        assert_eq!(legacy.is_ok(), declared.is_ok(), "raw={raw}");
        match (legacy, declared) {
            (Ok(l), Ok(d)) => assert_eq!(l, d, "raw={raw}"),
            (l, d) => {
                assert!(matches!(
                    l.unwrap_err(),
                    ConfigError::Variant { field: "mode", .. }
                ));
                assert!(matches!(
                    d.unwrap_err(),
                    ConfigError::Variant { field: "mode", .. }
                ));
            }
        }
    }
}

#[test]
fn parity_backend_accepts_known_variants() {
    for raw in ["memory", "postgres", "POSTGRESQL", "pg", "sqlite", ""] {
        let base = WorkersConfig::default();
        let legacy = legacy_apply(base.clone(), |key| {
            (key == "AG_WORKERS_BACKEND").then(|| raw.to_owned())
        });
        let declared = base.apply_env_source([("AG_WORKERS_BACKEND", raw)]);
        assert_eq!(legacy.is_ok(), declared.is_ok(), "raw={raw}");
        match (legacy, declared) {
            (Ok(l), Ok(d)) => assert_eq!(l, d, "raw={raw}"),
            (l, d) => {
                assert!(matches!(
                    l.unwrap_err(),
                    ConfigError::Variant {
                        field: "backend",
                        ..
                    }
                ));
                assert!(matches!(
                    d.unwrap_err(),
                    ConfigError::Variant {
                        field: "backend",
                        ..
                    }
                ));
            }
        }
    }
}

#[test]
fn parity_shutdown_timeout_uses_crate_duration_parser() {
    for raw in [
        "250ms", "30s", "5m", "1h", "2d", "1w", "12", "", "soon", "-1s",
    ] {
        let base = WorkersConfig::default();
        let legacy = legacy_apply(base.clone(), |key| {
            (key == "AG_WORKERS_SHUTDOWN_TIMEOUT").then(|| raw.to_owned())
        });
        let declared = base.apply_env_source([("AG_WORKERS_SHUTDOWN_TIMEOUT", raw)]);
        assert_eq!(legacy.is_ok(), declared.is_ok(), "raw={raw}");
        match (legacy, declared) {
            (Ok(l), Ok(d)) => assert_eq!(l, d, "raw={raw}"),
            (l, d) => {
                assert!(matches!(l.unwrap_err(), ConfigError::Duration { .. }));
                assert!(matches!(d.unwrap_err(), ConfigError::Duration { .. }));
            }
        }
    }
}

#[test]
fn parity_bool_accepts_one_true_yes_case_insensitive() {
    for raw in [
        "1", "true", "TRUE", "Yes", "yes", "0", "false", "no", "", " true ",
    ] {
        let base = WorkersConfig::default();
        let legacy = legacy_apply(base.clone(), |key| {
            (key == "AG_WORKERS_ENABLED").then(|| raw.to_owned())
        })
        .unwrap();
        let declared = base
            .apply_env_source([("AG_WORKERS_ENABLED", raw)])
            .unwrap();
        assert_eq!(legacy, declared, "raw={raw}");
        assert_eq!(
            declared.enabled,
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        );
    }
}

#[test]
fn parity_numeric_fields_fall_back_per_field() {
    let pairs: [(&str, &str); 4] = [
        ("AG_WORKERS_MAX_PAYLOAD_BYTES", "not-a-number"),
        ("AG_WORKERS_POISON_GUARD_ATTEMPTS", " 7 "),
        ("AG_WORKERS_ENABLED", "yes"),
        ("AG_WORKERS_MAX_PAYLOAD_BYTES_EXTRA", "ignored"),
    ];
    let base = WorkersConfig::default();
    let legacy = legacy_apply(base.clone(), get_from(&pairs)).unwrap();
    let declared = base.apply_env_source(pairs).unwrap();
    assert_eq!(legacy, declared);
    assert_eq!(
        declared.max_payload_bytes,
        WorkersConfig::default().max_payload_bytes
    );
    assert_eq!(declared.poison_guard_attempts, 7);
    assert!(declared.enabled);
}

#[test]
fn parity_per_queue_min_max_overrides() {
    let pairs: [(&str, &str); 5] = [
        ("AG_WORKERS_QUEUE_MAIL_MIN", "3"),
        ("AG_WORKERS_QUEUE_MAIL_MAX", "12"),
        ("AG_WORKERS_QUEUE_MEDIA_MIN", "bad"),
        ("AG_WORKERS_QUEUE_MEDIA_MAX", "9"),
        ("AG_WORKERS_QUEUE_GHOST_MIN", "99"),
    ];
    let base = base_with_queues();
    let legacy = legacy_apply(base.clone(), get_from(&pairs)).unwrap();
    let declared = base.apply_env_source(pairs).unwrap();
    assert_eq!(legacy, declared);
    assert_eq!(declared.queues["mail"].min_workers, 3);
    assert_eq!(declared.queues["mail"].max_workers, 12);
    // A malformed override keeps the TOML value (2), not QueueConfig's default.
    assert_eq!(declared.queues["media"].min_workers, 2);
    assert_eq!(declared.queues["media"].max_workers, 9);
}

// ---------------------------------------------------------------------------
// Schema / isolation tests.
// ---------------------------------------------------------------------------

#[test]
fn reads_only_declared_keys() {
    let declared = WorkersConfig::default()
        .apply_env_source([
            ("AG_WORKERS_MODE", "producer"),
            ("RUST_LOG", "trace"),
            ("UNDECLARED_KEY", "synthetic-value"),
        ])
        .unwrap();
    assert_eq!(declared.mode, WorkerMode::Producer);
    assert_eq!(workers_env_declarations().len(), 6);
}

#[test]
fn schema_declares_every_fixed_key_once() {
    let declarations = workers_env_declarations();
    let names: Vec<&str> = declarations.iter().map(EnvDeclaration::name).collect();
    assert_eq!(
        names,
        vec![
            "AG_WORKERS_ENABLED",
            "AG_WORKERS_MODE",
            "AG_WORKERS_BACKEND",
            "AG_WORKERS_SHUTDOWN_TIMEOUT",
            "AG_WORKERS_MAX_PAYLOAD_BYTES",
            "AG_WORKERS_POISON_GUARD_ATTEMPTS",
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
            && !declaration.is_secret()
    }));
}

#[test]
fn loader_adds_per_queue_declarations() {
    let loader = workers_loader(["mail", "media"].into_iter());
    assert_eq!(loader.declarations().len(), 6 + 4);
    let names: Vec<&str> = loader
        .declarations()
        .iter()
        .map(EnvDeclaration::name)
        .collect();
    assert!(names.contains(&"AG_WORKERS_QUEUE_MAIL_MIN"));
    assert!(names.contains(&"AG_WORKERS_QUEUE_MAIL_MAX"));
    assert!(names.contains(&"AG_WORKERS_QUEUE_MEDIA_MIN"));
    assert!(names.contains(&"AG_WORKERS_QUEUE_MEDIA_MAX"));
}
