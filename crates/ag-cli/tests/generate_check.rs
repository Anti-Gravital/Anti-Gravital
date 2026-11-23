//! Integration tests for the read-only `ag generate --check` command
//! (RFC-0019 PR 4).
//!
//! Every test drives the real `ag` binary against an isolated scratch
//! directory. No assertion depends on the shared Cargo target directory: a
//! stale artifact there once made a green check trustworthy when it was not.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

const SCHEMA_A: &str = "model User { id UUID @primary @auto email String @unique }";
const SCHEMA_B: &str = "model Product { id UUID @primary @auto sku String @unique }";

fn scratch_dir(label: &str) -> PathBuf {
    let seq = SCRATCH_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "ag-cli-generate-check-{label}-{}-{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn write_schema(dir: &Path, source: &str) {
    std::fs::write(dir.join("schema.ag"), source).expect("write schema");
}

fn ag(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("run ag")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn generate(dir: &Path, output_dir: &str) {
    let output = ag(
        &["generate", "--schema", "schema.ag", "--output", output_dir],
        dir,
    );
    assert!(
        output.status.success(),
        "generate failed: {}",
        stdout(&output)
    );
}

fn check(dir: &Path, output_dir: &str) -> Output {
    ag(
        &[
            "generate",
            "--schema",
            "schema.ag",
            "--output",
            output_dir,
            "--check",
        ],
        dir,
    )
}

/// Full on-disk snapshot of a directory: sorted relative paths with bytes.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("read dir")
            .map(|entry| entry.expect("entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .expect("child of root")
                    .to_string_lossy()
                    .into_owned();
                out.push((rel, std::fs::read(&path).expect("read file")));
            }
        }
    }
    let mut out = Vec::new();
    if root.is_dir() {
        walk(root, root, &mut out);
    }
    out
}

#[test]
fn check_passes_for_freshly_generated_artifacts() {
    let dir = scratch_dir("match");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    let output = check(&dir, "generated");
    assert!(
        output.status.success(),
        "matching artifacts must exit 0: {}",
        stdout(&output)
    );
    let text = stdout(&output);
    assert!(text.contains("Check passed"), "missing summary: {text}");
    assert!(text.contains("artifact(s)"), "missing count: {text}");
}

#[test]
fn check_fails_and_names_the_changed_file() {
    let dir = scratch_dir("changed");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    let target = dir.join("generated/src/models.rs");
    let original = std::fs::read_to_string(&target).expect("generated models.rs");
    std::fs::write(&target, format!("{original}\n// drift\n")).expect("drift models.rs");

    let output = check(&dir, "generated");
    assert!(
        !output.status.success(),
        "drift must exit non-zero: {}",
        stdout(&output)
    );
    let text = stdout(&output);
    assert!(text.contains("Check failed"), "missing summary: {text}");
    assert!(text.contains("src/models.rs"), "file not named: {text}");
    assert!(text.contains("changed"), "status missing: {text}");
}

#[test]
fn check_fails_and_names_the_missing_file() {
    let dir = scratch_dir("missing");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    let target = dir.join("generated/openapi.json");
    assert!(target.is_file(), "fixture must generate openapi.json");
    std::fs::remove_file(&target).expect("remove openapi.json");

    let output = check(&dir, "generated");
    assert!(
        !output.status.success(),
        "missing artifact must exit non-zero: {}",
        stdout(&output)
    );
    let text = stdout(&output);
    assert!(text.contains("openapi.json"), "file not named: {text}");
    assert!(text.contains("missing"), "status missing: {text}");
}

#[test]
fn check_fails_when_the_schema_changed() {
    let dir = scratch_dir("wrong-schema");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    // The committed artifacts still describe `User`, but the schema now
    // declares `Product`: the check must detect the stale output.
    write_schema(&dir, SCHEMA_B);

    let output = check(&dir, "generated");
    assert!(
        !output.status.success(),
        "stale schema must exit non-zero: {}",
        stdout(&output)
    );
    let text = stdout(&output);
    assert!(text.contains("Check failed"), "missing summary: {text}");
    assert!(
        text.contains("src/models.rs") || text.contains("openapi.json"),
        "no differing artifact named: {text}"
    );
}

#[test]
fn check_reports_unexpected_files_without_removing_them() {
    let dir = scratch_dir("unexpected");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    let extra = dir.join("generated/UNEXPECTED.txt");
    std::fs::write(&extra, "hand edited").expect("create extra file");

    let output = check(&dir, "generated");
    assert!(
        !output.status.success(),
        "extra file must exit non-zero: {}",
        stdout(&output)
    );
    let text = stdout(&output);
    assert!(text.contains("UNEXPECTED.txt"), "file not named: {text}");
    assert!(text.contains("unexpected"), "status missing: {text}");
    assert!(extra.is_file(), "check must not delete files");
}

#[test]
fn check_never_writes_and_the_report_is_deterministic() {
    let dir = scratch_dir("readonly");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");
    let clean = snapshot(&dir.join("generated"));

    let first = check(&dir, "generated");
    assert!(first.status.success(), "{}", stdout(&first));
    assert_eq!(
        snapshot(&dir.join("generated")),
        clean,
        "check mutated the tree on the matching case"
    );
    let first_text = stdout(&first);
    assert_eq!(
        stdout(&check(&dir, "generated")),
        first_text,
        "report drifted"
    );

    // A drifted file must survive the check byte-for-byte: proof that the
    // verifier never calls the writer.
    let target = dir.join("generated/openapi.json");
    std::fs::write(&target, "drifted").expect("drift openapi.json");
    let drifted = snapshot(&dir.join("generated"));
    let bad = check(&dir, "generated");
    assert!(!bad.status.success(), "{}", stdout(&bad));
    assert_eq!(
        snapshot(&dir.join("generated")),
        drifted,
        "check overwrote drifted artifacts"
    );
}

#[test]
fn check_does_not_create_a_missing_output_directory() {
    let dir = scratch_dir("no-create");
    write_schema(&dir, SCHEMA_A);
    assert!(!dir.join("generated").exists());

    let output = check(&dir, "generated");
    assert!(
        !output.status.success(),
        "absent artifacts must exit non-zero: {}",
        stdout(&output)
    );
    assert!(stdout(&output).contains("missing"), "{}", stdout(&output));
    assert!(
        !dir.join("generated").exists(),
        "check created the output directory"
    );
}

#[test]
fn check_stays_prose_even_with_a_json_format_selector() {
    let dir = scratch_dir("json-selector");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    let output = ag(
        &[
            "generate",
            "--check",
            "--output",
            "generated",
            "--output-format",
            "json",
        ],
        &dir,
    );
    assert!(
        output.status.success(),
        "check must not fail on the format selector: {}",
        stdout(&output)
    );
    assert!(
        stdout(&output).contains("Check passed"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn check_honours_the_configured_output_directory() {
    let dir = scratch_dir("custom-output");
    write_schema(&dir, SCHEMA_A);
    generate(&dir, "generated");

    // A different output path has no committed artifacts, so it fails...
    let output = check(&dir, "custom");
    assert!(
        !output.status.success(),
        "check must read --output: {}",
        stdout(&output)
    );
    assert!(stdout(&output).contains("custom"), "{}", stdout(&output));

    // ...and passes once that directory holds generated artifacts.
    generate(&dir, "custom");
    let output = check(&dir, "custom");
    assert!(
        output.status.success(),
        "check must honour --output: {}",
        stdout(&output)
    );
}
