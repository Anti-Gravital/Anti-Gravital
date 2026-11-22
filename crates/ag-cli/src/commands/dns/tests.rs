//! Unit tests for the local domain planning and state machinery.

use std::fs;
use std::path::{Path, PathBuf};

use ag_domains::record::RecordType;

use super::model::default_visibility;
use super::*;

fn manifest(binding: &str) -> String {
    format!(
        "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"api.example.com\"\nenvironment = \
         \"staging\"\nvisibility = \"private\"\ntls = \"disabled\"\ntarget = {{ kind = \
         \"service\", ref = \"api\" }}\n\n{binding}"
    )
}

fn write_manifest(dir: &Path, text: &str) -> PathBuf {
    let path = dir.join("ag.toml");
    fs::write(&path, text).expect("write manifest");
    path
}

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-dns-unit-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn record_set(entries: &str) -> String {
    format!("[domain.dns]\n{entries}")
}

#[test]
fn environment_aliases_are_canonicalized() {
    assert_eq!(Environment::parse("dev").unwrap(), Environment::Dev);
    assert_eq!(Environment::parse("DEVELOPMENT").unwrap(), Environment::Dev);
    assert_eq!(Environment::parse("stage").unwrap(), Environment::Staging);
    assert_eq!(Environment::parse("prod").unwrap(), Environment::Production);
    assert_eq!(
        Environment::parse("production").unwrap(),
        Environment::Production
    );
    assert_eq!(Environment::parse("local").unwrap(), Environment::Local);
    assert!(Environment::parse("qa").is_err());
}

#[test]
fn visibility_defaults_to_private_and_parses_explicitly() {
    assert_eq!(Visibility::parse("private").unwrap(), Visibility::Private);
    assert_eq!(Visibility::parse("PUBLIC").unwrap(), Visibility::Public);
    assert!(Visibility::parse("internal").is_err());
    assert_eq!(default_visibility(), "private");
}

#[test]
fn plan_id_is_deterministic() {
    let dir = scratch("deterministic");
    let path = write_manifest(
        &dir,
        &manifest(&record_set(
            "TXT = [{ name = \"_app\", value = \"v=1\" }]\n",
        )),
    );
    let first = load_plan(&path, None).expect("first plan");
    let second = load_plan(&path, None).expect("second plan");
    assert_eq!(first.plan_id, second.plan_id);
    assert!(first.plan_id.starts_with("dns-staging-"));
    assert_eq!(first.plan_id.len(), "dns-staging-".len() + 16);
}

#[test]
fn ownership_record_is_deterministic_and_not_a_secret() {
    let dir = scratch("ownership");
    let path = write_manifest(&dir, &manifest(""));
    let plan = load_plan(&path, None).expect("plan");
    let ownership: Vec<_> = plan
        .records
        .iter()
        .filter(|record| record.is_ownership)
        .collect();
    assert_eq!(ownership.len(), 1);
    assert_eq!(ownership[0].name, "_ag-domain.api.example.com");
    assert!(ownership[0].value.starts_with("ag-verification="));
}

#[test]
fn records_are_sorted_and_deduplicated() {
    let dir = scratch("sorting");
    let path = write_manifest(
        &dir,
        &manifest(&record_set(
            "TXT = [{ name = \"a\", value = \"z\" }, { name = \"a\", value = \"z\" }]\n",
        )),
    );
    let plan = load_plan(&path, None).expect("plan");
    let explicit: Vec<_> = plan.records.iter().filter(|r| !r.is_ownership).collect();
    assert_eq!(explicit.len(), 1);
    assert_eq!(explicit[0].name, "a.example.com");
}

#[test]
fn wildcard_records_are_rejected_by_default() {
    let dir = scratch("wildcard");
    let path = write_manifest(
        &dir,
        &manifest(&record_set(
            "CNAME = [{ name = \"*.example.com\", value = \"edge.example.net\" }]\n",
        )),
    );
    let error = load_plan(&path, None).expect_err("wildcard must be rejected");
    assert!(error.contains("allow_wildcard"), "{error}");
}

#[test]
fn wildcard_domain_requires_dns01_opt_in() {
    let dir = scratch("wildcard-domain");
    let base = "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"*.example.com\"\n\
                    environment = \"prod\"\nallow_wildcard = true\n";
    let path = write_manifest(&dir, base);
    let error = load_plan(&path, None).expect_err("DNS-01 required");
    assert!(error.contains("managed_dns01"), "{error}");
    fs::write(&path, format!("{base}tls = \"managed_dns01\"\n")).expect("rewrite");
    assert!(load_plan(&path, None).is_ok());
}

#[test]
fn private_binding_drops_public_routing_records() {
    let dir = scratch("visibility");
    let path = write_manifest(
        &dir,
        &manifest(&record_set(
            "A = [{ name = \"@\", value = \"192.0.2.10\" }]\nTXT = [{ name = \"note\", \
             value = \"v=ag-private\" }]\n",
        )),
    );
    let plan = load_plan(&path, None).expect("plan");
    assert!(!plan
        .records
        .iter()
        .any(|record| record.record_type == RecordType::A));
    assert!(plan
        .records
        .iter()
        .any(|record| record.record_type == RecordType::Txt && !record.is_ownership));
    assert!(plan.notes.iter().any(|note| note.contains("Private")));
}

#[test]
fn public_binding_keeps_routing_records() {
    let dir = scratch("public");
    let text = manifest(&record_set(
        "A = [{ name = \"@\", value = \"192.0.2.10\" }]\n",
    ))
    .replace("visibility = \"private\"", "visibility = \"public\"");
    let path = write_manifest(&dir, &text);
    let plan = load_plan(&path, None).expect("plan");
    assert!(plan
        .records
        .iter()
        .any(|record| record.record_type == RecordType::A));
}

#[test]
fn environment_override_is_applied() {
    let dir = scratch("override");
    let text = format!(
        "{}\n[domain.environments.production]\nvisibility = \"public\"\ndomain = \
         \"app.example.com\"\n",
        manifest("")
    );
    let path = write_manifest(&dir, &text);
    let staging = load_plan(&path, None).expect("staging");
    assert_eq!(staging.domain, "api.example.com");
    let production = load_plan(&path, Some("prod")).expect("production");
    assert_eq!(production.domain, "app.example.com");
    assert_eq!(production.environment, Environment::Production);
    assert_eq!(production.visibility, Visibility::Public);
}

#[test]
fn local_environment_produces_an_empty_domain_plan() {
    let dir = scratch("local");
    let path = write_manifest(&dir, &manifest(""));
    let plan = load_plan(&path, Some("local")).expect("local plan");
    assert!(plan.domain.is_empty());
    assert!(plan.records.is_empty());
    assert!(plan.notes.iter().any(|note| note.contains("Local")));
}

#[test]
fn ttl_bounds_are_enforced() {
    let dir = scratch("ttl");
    let text = manifest("").replace("tls = \"disabled\"", "tls = \"disabled\"\nttl = 30");
    let path = write_manifest(&dir, &text);
    assert!(load_plan(&path, None)
        .expect_err("ttl too low")
        .contains("between 60 and 86400"));
}

#[test]
fn invalid_ips_types_and_unknown_fields_fail_clearly() {
    let dir = scratch("invalid");
    let path = write_manifest(
        &dir,
        &manifest(&record_set(
            "A = [{ name = \"@\", value = \"not-an-ip\" }]\n",
        )),
    );
    assert!(load_plan(&path, None)
        .expect_err("bad IP")
        .contains("not a valid IP address"));

    let path = write_manifest(
        &dir,
        &manifest(&record_set("NS = [{ name = \"@\", value = \"x\" }]\n")),
    );
    assert!(load_plan(&path, None)
        .expect_err("bad type")
        .contains("unsupported record type"));

    let path = write_manifest(
        &dir,
        "[project]\nname = \"demo\"\n\n[domain]\ndomain = \"example.com\"\nzone_id = \
         \"CANARY-ZONE\"\ntoken = \"CANARY-TOKEN\"\n",
    );
    assert!(load_plan(&path, None)
        .expect_err("unknown fields")
        .contains("unknown field"));
}

#[test]
fn missing_project_or_domain_is_an_error_not_a_guess() {
    let dir = scratch("missing");
    let path = write_manifest(&dir, "[domain]\ndomain = \"example.com\"\n");
    assert!(load_plan(&path, None)
        .expect_err("no project")
        .contains("missing [project].name"));
    let path = write_manifest(&dir, "[project]\nname = \"demo\"\n");
    assert!(load_plan(&path, None)
        .expect_err("no domain table")
        .contains("missing the [domain] table"));
}
