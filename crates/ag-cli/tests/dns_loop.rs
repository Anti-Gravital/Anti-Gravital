//! End-to-end checks for the local domain loop (`ag dns`, `ag domains attach`).
//!
//! Every case runs in a scratch directory with no network service and no
//! credential. The provider adapter is only reached via an explicit
//! `--provider` flag, and an incomplete configuration fails before any state
//! is written.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-dns-loop-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn write(dir: &Path, name: &str, contents: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent");
    }
    std::fs::write(path, contents).expect("fixture");
}

fn ag(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ag"));
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI")
        .env_remove("AG_CLOUDFLARE_TOKEN")
        .env_remove("AG_DNS_ZONE_ID")
        .env_remove("AG_EDGE_HOST");
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().expect("run ag")
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn manifest(environment: &str, visibility: &str, tls: &str) -> String {
    format!(
        "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\nenvironment = \
         \"{environment}\"\nvisibility = \"{visibility}\"\ntls = \"{tls}\"\ntarget = {{ kind = \
         \"service\", ref = \"api\" }}\n"
    )
}

fn plan_output(dir: &Path, manifest_text: &str) -> Output {
    write(dir, "ag.toml", manifest_text);
    ag(dir, &["dns", "plan"], &[])
}

fn plan_id(output: &Output) -> String {
    text(output)
        .lines()
        .find_map(|line| line.strip_prefix("Plan:        "))
        .expect("plan id line")
        .trim()
        .to_owned()
}

#[test]
fn plan_is_deterministic_and_offline() {
    let dir = scratch("plan");
    let source = manifest("production", "public", "managed_http01");
    let first = plan_output(&dir, &source);
    assert!(first.status.success(), "{:?}", first);
    let second = plan_output(&dir, &source);
    assert_eq!(plan_id(&first), plan_id(&second));
    assert!(plan_id(&first).starts_with("dns-production-"));
    assert!(text(&first).contains("Zone:        example.com"));
    assert!(text(&first).contains("Visibility:  public"));
    assert!(!dir.join(".ag/dns/state.json").exists());
}

#[test]
fn apply_requires_an_exact_plan_id() {
    let dir = scratch("gate");
    let source = manifest("staging", "private", "disabled");
    let plan = plan_output(&dir, &source);
    assert!(plan.status.success());
    let id = plan_id(&plan);

    let missing = ag(&dir, &["dns", "apply"], &[]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(!dir.join(".ag/dns/state.json").exists());

    let wrong = ag(
        &dir,
        &["dns", "apply", "--plan", "dns-staging-deadbeef"],
        &[],
    );
    assert_eq!(wrong.status.code(), Some(1));
    assert!(text(&wrong).is_empty());
    assert!(stderr(&wrong).contains("plan drift"), "{:?}", wrong);
    assert!(!dir.join(".ag/dns/state.json").exists());

    let applied = ag(&dir, &["dns", "apply", "--plan", &id], &[]);
    assert!(applied.status.success(), "{:?}", applied);
    assert!(text(&applied).contains("local configuration only"));
    assert!(dir.join(".ag/dns/state.json").exists());
}

#[test]
fn adapter_configuration_fails_closed_without_leaking_values() {
    let dir = scratch("adapter");
    let source = manifest("dev", "private", "disabled");
    let plan = plan_output(&dir, &source);
    assert!(plan.status.success());
    let id = plan_id(&plan);

    const TOKEN: &str = "CANARY-TOKEN-9f3a";
    const ZONE: &str = "CANARY-ZONE-42";

    let no_zone = ag(
        &dir,
        &["dns", "apply", "--plan", &id, "--provider", "cloudflare"],
        &[("AG_CLOUDFLARE_TOKEN", TOKEN)],
    );
    assert_eq!(no_zone.status.code(), Some(1));
    assert!(!dir.join(".ag/dns/state.json").exists());
    let combined = format!("{}{}", text(&no_zone), stderr(&no_zone));
    assert!(!combined.contains(TOKEN), "token leaked: {combined}");
    assert!(combined.contains("AG_DNS_ZONE_ID"), "{combined}");

    let no_token = ag(
        &dir,
        &[
            "dns",
            "apply",
            "--plan",
            &id,
            "--provider",
            "cloudflare",
            "--zone-id",
            ZONE,
        ],
        &[],
    );
    assert_eq!(no_token.status.code(), Some(1));
    assert!(!dir.join(".ag/dns/state.json").exists());
    let combined = format!("{}{}", text(&no_token), stderr(&no_token));
    assert!(!combined.contains(ZONE), "zone id leaked: {combined}");
    assert!(combined.contains("AG_CLOUDFLARE_TOKEN"), "{combined}");
}

#[test]
fn state_has_no_credentials_and_status_is_read_only() {
    let dir = scratch("state");
    let source = manifest("dev", "private", "disabled");
    let plan = plan_output(&dir, &source);
    assert!(plan.status.success());
    let id = plan_id(&plan);
    let applied = ag(&dir, &["dns", "apply", "--plan", &id], &[]);
    assert!(applied.status.success(), "{:?}", applied);
    let state = std::fs::read_to_string(dir.join(".ag/dns/state.json")).expect("state");
    assert!(!state.contains("CANARY"));
    assert!(!state.contains("zone_id"));
    assert!(state.contains(&id));
    assert!(state.contains("\"zone\": \"example.com\""));

    let status = ag(&dir, &["dns", "status"], &[]);
    assert!(status.status.success(), "{:?}", status);
    assert!(text(&status).contains("In sync:     yes"));
    assert_eq!(
        std::fs::read_to_string(dir.join(".ag/dns/state.json")).expect("state"),
        state,
        "status must not modify state"
    );
}

#[test]
fn environment_aliases_and_overrides_are_resolved() {
    let dir = scratch("env");
    let mut source = manifest("staging", "private", "disabled");
    source.push_str("\n[domain.environments.production]\nvisibility = \"public\"\ndomain = \"app.example.com\"\ntls = \"managed_http01\"\n");
    let default = plan_output(&dir, &source);
    assert!(default.status.success(), "{:?}", default);
    assert!(text(&default).contains("Domain:      api.example.com"));

    let prod = ag(&dir, &["dns", "plan", "--env", "prod"], &[]);
    assert!(prod.status.success(), "{:?}", prod);
    assert!(text(&prod).contains("Domain:      app.example.com"));
    assert!(text(&prod).contains("Environment: production"));
    assert!(text(&prod).contains("Visibility:  public"));
    assert!(plan_id(&prod).starts_with("dns-production-"));
}

#[test]
fn validation_rejects_wildcards_bad_ips_and_private_http01() {
    let dir = scratch("validation");
    let wildcard = "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"*.example.com\"\n\
                    environment = \"production\"\ntls = \"managed_http01\"\n";
    let output = plan_output(&dir, wildcard);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("managed_dns01"), "{:?}", output);

    let bad_ip = "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"example.com\"\n\
                  tls = \"disabled\"\n\n[domain.dns]\nA = [{ name = \"@\", value = \
                  \"not-an-ip\" }]\n";
    let output = plan_output(&dir, bad_ip);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("not a valid IP address"));

    let private_http = manifest("production", "private", "managed_http01");
    let output = plan_output(&dir, &private_http);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("private visibility requires"));
}

#[test]
fn domains_attach_reads_the_manifest_offline() {
    let dir = scratch("attach");
    write(&dir, "ag.toml", &manifest("staging", "private", "disabled"));
    let output = ag(
        &dir,
        &[
            "domains",
            "attach",
            "--manifest",
            "ag.toml",
            "--edge-host",
            "edge.example-cloud.net",
            "--state",
            ".ag/domains.json",
        ],
        &[],
    );
    assert!(output.status.success(), "{:?}", output);
    let stdout = text(&output);
    assert!(stdout.contains("Domain:      api.example.com"));
    assert!(stdout.contains("Environment: staging"));
    let store = std::fs::read_to_string(dir.join(".ag/domains.json")).expect("store");
    assert!(!store.contains("cloudflare"), "no provider brand in store");
}
