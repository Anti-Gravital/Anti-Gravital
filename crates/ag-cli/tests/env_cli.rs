//! Integration tests for the two roadmap gaps closed in wave 9:
//! `ag new --with MODULE` (RFC-0019 stage A) and `ag env list|check`
//! (RFC-0020 stage B). The real binary is spawned with no terminal so every
//! case is non-interactive.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-cli-w9-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn ag(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env("CI", "1")
        .env("AG_NO_TUI", "1")
        .env_remove("SERVICE_TOKEN")
        .env_remove("PORT")
        .env_remove("LOG_LEVEL")
        .output()
        .expect("run ag")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ---- `ag new --with MODULE` ------------------------------------------------

#[test]
fn with_records_declared_modules_in_the_generated_manifest() {
    let dir = scratch("with-modules");
    let output = ag(
        &[
            "new",
            "shop",
            "--template",
            "layered",
            "--with",
            "auth",
            "--with",
            "mail",
        ],
        &dir,
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let manifest = fs::read_to_string(dir.join("shop/ag.toml")).expect("ag.toml");
    assert!(manifest.contains("auth = true"), "{manifest}");
    assert!(manifest.contains("mail = true"), "{manifest}");
    // Defaults that were not requested are untouched.
    assert!(manifest.contains("data = true"), "{manifest}");
    assert!(manifest.contains("cache = false"), "{manifest}");
    assert!(manifest.contains("layout = \"layered\""), "{manifest}");
}

#[test]
fn without_modules_the_default_scaffold_is_unchanged() {
    let plain_dir = scratch("without-modules-a");
    let enabled_dir = scratch("without-modules-b");
    assert!(ag(&["new", "shop", "--template", "layered"], &plain_dir)
        .status
        .success());
    assert!(ag(
        &["new", "shop", "--template", "layered", "--with", "data"],
        &enabled_dir
    )
    .status
    .success());
    let plain = fs::read_to_string(plain_dir.join("shop/ag.toml")).expect("plain manifest");
    let enabled = fs::read_to_string(enabled_dir.join("shop/ag.toml")).expect("enabled manifest");
    // Enabling a module that is already true is a no-op: both files are equal,
    // and neither gained a key that was absent from the original scaffold.
    // Normalize CRLF: Windows writers emit \r\n and the assertion is about
    // content, not the platform's line-ending convention.
    assert_eq!(plain.replace("\r\n", "\n"), enabled.replace("\r\n", "\n"));
    assert!(plain.contains("auth = false"));
    assert!(!plain.contains("mail ="));
}

#[test]
fn with_on_a_template_without_a_manifest_is_a_clear_error() {
    let dir = scratch("with-no-manifest");
    let output = ag(&["new", "api", "--with", "auth"], &dir);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("--with requires a template"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.join("api").exists(), "nothing should be scaffolded");
}

#[test]
fn with_rejects_an_unknown_module_before_creating_anything() {
    let dir = scratch("with-unknown");
    let output = ag(&["new", "api", "--with", "not-a-module"], &dir);
    assert_eq!(output.status.code(), Some(2));
    assert!(!dir.join("api").exists());
}

// ---- `ag env list|check` ----------------------------------------------------

const MANIFEST: &str = "[project]\nname = \"shop\"\n\n\
    [env.SERVICE_TOKEN]\ntype = \"string\"\nrequired = true\nsecret = true\n\
    description = \"service credential\"\n\n\
    [env.PORT]\ntype = \"int\"\ndefault = 8080\n\n\
    [env.LOG_LEVEL]\ntype = \"string\"\nrequired = false\n";

fn write_manifest(dir: &Path, text: &str) -> PathBuf {
    let path = dir.join("ag.toml");
    fs::write(&path, text).expect("write manifest");
    path
}

#[test]
fn env_list_shows_declarations_and_never_prints_values() {
    let dir = scratch("env-list");
    write_manifest(&dir, MANIFEST);
    let output = ag(&["env", "list"], &dir);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("SERVICE_TOKEN"), "{text}");
    assert!(!text.contains("url"), "{text}");
    assert!(text.contains("string"), "{text}");
    assert!(text.contains("SECRET"), "{text}");
    assert!(text.contains("PORT"), "{text}");
    assert!(text.contains("PRESENT"), "{text}");
    // No secret value, no non-secret default value, no description payload.
    assert!(!text.contains("service credential"), "{text}");
    assert!(!text.contains("8080"), "{text}");
    assert!(!text.contains("postgres"), "{text}");
}

#[test]
fn env_list_reports_when_a_value_is_present_without_showing_it() {
    let dir = scratch("env-presence");
    write_manifest(&dir, MANIFEST);
    let output = Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(["env", "list"])
        .env("SERVICE_TOKEN", "super-secret-value")
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("run ag");
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(!text.contains("super-secret-value"), "{text}");
    assert!(text.contains("SERVICE_TOKEN"), "{text}");
}

#[test]
fn env_check_succeeds_with_valid_declared_values() {
    let dir = scratch("env-check-ok");
    write_manifest(&dir, MANIFEST);
    let output = Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(["env", "check"])
        .env("SERVICE_TOKEN", "a-token")
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("run ag");
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("Environment OK"));
    assert!(!stdout(&output).contains("a-token"));
}

#[test]
fn env_check_collects_every_problem_without_values() {
    let dir = scratch("env-check-fail");
    write_manifest(&dir, MANIFEST);
    let output = ag(&["env", "check"], &dir);
    assert_eq!(output.status.code(), Some(1));
    let text = format!("{}{}", stdout(&output), stderr(&output));
    assert!(text.contains("SERVICE_TOKEN"), "{text}");
    assert!(text.contains("values are never printed"), "{text}");
    assert!(!text.contains("8080"), "{text}");
}

#[test]
fn env_json_shapes_are_versioned_and_redacted() {
    let dir = scratch("env-json");
    write_manifest(&dir, MANIFEST);

    let list = ag(&["--output", "json", "env", "list"], &dir);
    assert!(list.status.success(), "{}", stderr(&list));
    let value: serde_json::Value = serde_json::from_str(&stdout(&list)).expect("list JSON");
    assert_eq!(value["contract_version"], "0.2.0");
    assert_eq!(value["command"], "env.list");
    assert_eq!(value["status"], "ok");
    assert!(value["data"]["variables"].is_array());
    assert_eq!(value["data"]["variables"][0]["type"], "string");
    assert!(!stdout(&list).contains("\"ty\""), "{}", stdout(&list));
    assert!(!stdout(&list).contains("8080"));

    let check = ag(&["env", "check", "--output", "json"], &dir);
    assert_eq!(check.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_str(&stdout(&check)).expect("check JSON");
    assert_eq!(value["command"], "env.check");
    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["ok"], false);
    assert_eq!(value["data"]["problems"].as_array().map(Vec::len), Some(1));
    assert_eq!(value["data"]["problems"][0]["type"], "string");
    assert!(!stdout(&check).contains("8080"));
}

#[test]
fn env_check_json_matches_the_published_fixture_byte_for_byte() {
    // Mirrors docs/mcp/snapshots/env.check.json (mcpdev, contract 0.2.0).
    let dir = scratch("env-check-fixture");
    write_manifest(
        &dir,
        "[env.MISSING]\ntype = \"url\"\n\n\
         [env.PRESENT1]\ntype = \"string\"\n\n\
         [env.PRESENT2]\ntype = \"int\"\ndefault = 7\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(["env", "check", "--env", "dev", "--output", "json"])
        .env_remove("MISSING")
        .env("PRESENT1", "ready")
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("run ag");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stdout(&output).trim_end(),
        concat!(
            r#"{"contract_version":"0.2.0","command":"env.check","status":"ok","#,
            r#""data":{"manifest":"ag.toml","environment":"dev","ok":false,"#,
            r#""resolved_count":2,"problems":[{"name":"MISSING","type":"url","#,
            r#""reason":"missing"}]}}"#
        )
    );
}

#[test]
fn env_reports_a_missing_manifest_clearly() {
    let dir = scratch("env-missing");
    let output = ag(&["env", "list", "--manifest", "absent.toml"], &dir);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("Declare [env.*]"),
        "{}",
        stderr(&output)
    );
}
