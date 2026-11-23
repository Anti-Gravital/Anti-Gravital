//! Config/env `--output json` snapshot tests (RFC-0021 section 4.3), split from
//! `json_output.rs` to stay under the 600-line guardrail. Same contract, same
//! byte-exact golden documents; no behaviour change.

mod json_output_support;
use json_output_support::*;

#[test]
fn config_get_shape_has_no_secret_or_arbitrary_field_names() {
    // Field names are fixed by the schema; command input can only reach the
    // opaque `value` slot. No config key maps to inline secret material.
    let secret_fields = ["token", "password", "secret", "key_path_pem", "value_pem"];
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/mcp/cli-json-0.2.0.schema.json"))
            .expect("schema is valid JSON");
    let properties = schema["$defs"]["configGetData"]["properties"]
        .as_object()
        .expect("configGetData properties");
    let mut names: Vec<&str> = properties.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["key", "kind", "source", "value"]);
    for name in secret_fields {
        assert!(!names.contains(&name), "unexpected field: {name}");
    }
}

#[test]
fn config_get_matches_snapshot() {
    let dir = scratch_dir("config-get");
    write(&dir, "config.toml", config_toml());
    let output = ag(&["--output", "json", "config", "get", "bind"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_snapshot(&output, "config.get");
}

#[test]
fn config_get_default_source_and_closed_shape() {
    // A config.toml exists but omits this key: the compiled-in default is used
    // and the shape still carries a closed, fixed field set. A missing file is
    // a read_failed error in both prose and JSON (existing behaviour).
    let dir = scratch_dir("config-get-default");
    write(&dir, "config.toml", config_toml());
    let output = ag(
        &["--output", "json", "config", "get", "rate_limit.per_ip_rps"],
        &dir,
        &[],
    );
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value =
        serde_json::from_str(stdout(&output).trim()).expect("valid JSON");
    assert_eq!(value["contract_version"], "0.2.0");
    assert_eq!(value["command"], "config.get");
    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["key"], "rate_limit.per_ip_rps");
    assert_eq!(value["data"]["kind"], "integer");
    assert_eq!(value["data"]["source"], "default");
    assert!(value["data"]["value"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .is_ok());
    assert!(value.get("error").is_none());
    let mut names: Vec<&str> = value["data"]
        .as_object()
        .expect("data object")
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["key", "kind", "source", "value"]);
}

#[test]
fn config_get_unknown_key_is_a_versioned_read_error() {
    let dir = scratch_dir("config-get-unknown");
    const CANARY: &str = "CANARY-SECRET-9f3a";
    let output = ag(
        &["--output", "json", "config", "get", "not.a.key"],
        &dir,
        &[("AG_CONFIG_CANARY", CANARY)],
    );
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value =
        serde_json::from_str(stdout(&output).trim()).expect("valid JSON");
    assert_eq!(value["contract_version"], "0.2.0");
    assert_eq!(value["command"], "config.get");
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["code"], "read_failed");
    assert!(
        stdout(&output).contains("bind"),
        "the closed key set must be listed"
    );
    assert!(!stdout(&output).contains(CANARY));
}

#[test]
fn config_set_refuses_json_without_writing() {
    let dir = scratch_dir("config-set-refuse");
    let output = ag(
        &[
            "--output",
            "json",
            "config",
            "set",
            "bind",
            "127.0.0.1:9000",
        ],
        &dir,
        &[],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_snapshot(&output, "unsupported_command");
    assert!(
        !dir.join("config.toml").exists(),
        "a refused config set wrote a file"
    );
}

#[test]
fn config_get_prose_is_unchanged_without_the_flag() {
    let dir = scratch_dir("config-get-prose");
    write(&dir, "config.toml", config_toml());
    let output = ag(&["config", "get", "bind"], &dir, &[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "127.0.0.1:8080\n");
    assert!(!stdout(&output).contains("contract_version"));
}

/// The env fixtures pin the exact field order so the emitted documents cannot
/// drift from the published contract.
#[test]
fn env_fixtures_match_the_published_schema() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/mcp/cli-json-0.2.0.schema.json"))
            .expect("schema is valid JSON");
    let commands = schema["properties"]["command"]["enum"]
        .as_array()
        .expect("command enum");
    for name in ["env.list", "env.check"] {
        assert!(
            commands.iter().any(|value| value.as_str() == Some(name)),
            "schema must declare {name}"
        );
    }

    let list: serde_json::Value =
        serde_json::from_str(snapshot("env.list")).expect("env.list fixture");
    assert_eq!(list["contract_version"], "0.2.0");
    assert_eq!(list["command"], "env.list");
    assert_eq!(list["status"], "ok");
    let variable = &list["data"]["variables"][0];
    for field in [
        "name",
        "type",
        "required",
        "secret",
        "has_default",
        "source",
        "present",
    ] {
        assert!(variable.get(field).is_some(), "missing field: {field}");
    }

    let check: serde_json::Value =
        serde_json::from_str(snapshot("env.check")).expect("env.check fixture");
    assert_eq!(check["contract_version"], "0.2.0");
    assert_eq!(check["command"], "env.check");
    assert_eq!(check["status"], "ok");
    let problem = &check["data"]["problems"][0];
    for field in ["name", "type", "reason"] {
        assert!(problem.get(field).is_some(), "missing field: {field}");
    }
}

#[test]
fn env_fixtures_never_carry_a_value_or_default_value() {
    // RFC-0020 forbids secret values in tool arguments and outputs. The shapes
    // expose presence and source only; no value field is part of the contract.
    for name in ["env.list", "env.check"] {
        let text = snapshot(name);
        for forbidden in ["\"value\"", "\"default_value\"", "\"raw\"", "CANARY"] {
            assert!(
                !text.contains(forbidden),
                "{name} must not contain {forbidden}"
            );
        }
    }
}

/// `config.get` is emitted (RFC-0021 section 4.3). The golden document is
/// asserted byte-for-byte by `config_get_matches_snapshot`; these checks pin
/// the same shape against the published schema so code and contract cannot
/// drift silently.
#[test]
fn config_get_fixture_matches_the_published_schema() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/mcp/cli-json-0.2.0.schema.json"))
            .expect("schema is valid JSON");
    let commands = schema["properties"]["command"]["enum"]
        .as_array()
        .expect("command enum");
    assert!(commands
        .iter()
        .any(|value| value.as_str() == Some("config.get")));
    let definitions = &schema["$defs"]["configGetData"];
    assert_eq!(definitions["additionalProperties"], false);
    let required = definitions["required"].as_array().expect("required fields");
    for field in ["key", "kind", "value", "source"] {
        assert!(required.iter().any(|value| value.as_str() == Some(field)));
    }

    let document: serde_json::Value =
        serde_json::from_str(snapshot("config.get")).expect("fixture is valid JSON");
    assert_eq!(document["contract_version"], "0.2.0");
    assert_eq!(document["command"], "config.get");
    assert_eq!(document["status"], "ok");
    let data = &document["data"];
    assert_eq!(data["key"].as_str(), Some("bind"));
    assert_eq!(data["kind"].as_str(), Some("bind"));
    assert_eq!(data["value"].as_str(), Some("127.0.0.1:8080"));
    assert_eq!(data["source"].as_str(), Some("file"));
    assert!(data.get("error").is_none());
}
