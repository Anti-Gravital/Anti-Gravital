//! Non-interactive regression suite (RFC-0018, section 4.9).
//!
//! Runs the real `ag` binary with standard input closed and asserts that it
//! never blocks waiting for input and never emits terminal escape sequences.
//! A child process spawned by the test harness has no terminal, which is the
//! same situation as a script or a CI job.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ag-cli-test-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    dir
}

fn ag(args: &[&str], dir: &PathBuf) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ag"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CI")
        .env_remove("AG_NO_TUI")
        .env("TERM", "xterm-256color")
        .output()
        .expect("run ag")
}

fn assert_no_escape_sequences(output: &Output) {
    for (stream, bytes) in [("stdout", &output.stdout), ("stderr", &output.stderr)] {
        assert!(
            !bytes.contains(&0x1b),
            "{stream} contains a terminal escape sequence: {:?}",
            String::from_utf8_lossy(bytes)
        );
    }
}

#[test]
fn new_without_a_name_fails_with_usage_and_does_not_block() {
    let dir = scratch_dir("no-name");
    let output = ag(&["new"], &dir);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("project name is required"));
    assert_no_escape_sequences(&output);
    assert_eq!(std::fs::read_dir(&dir).expect("read").count(), 0);
}

#[test]
fn new_with_a_name_defaults_the_template_and_creates_the_project() {
    let dir = scratch_dir("default-template");
    let output = ag(&["new", "api"], &dir);
    assert!(output.status.success(), "{output:?}");
    assert_no_escape_sequences(&output);
    for file in ["Cargo.toml", "src/main.rs", "config.toml"] {
        assert!(dir.join("api").join(file).is_file(), "missing {file}");
    }
    assert!(!dir.join("api/migrations").exists());
}

#[test]
fn new_with_every_flag_creates_the_requested_template() {
    let dir = scratch_dir("fullstack");
    let output = ag(&["new", "shop", "--template", "fullstack"], &dir);
    assert!(output.status.success(), "{output:?}");
    assert_no_escape_sequences(&output);
    assert!(dir.join("shop/migrations/0001_init.sql").is_file());
    // A scripted run does not print the interactive follow-up.
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Equivalent"));
}

#[test]
fn no_interactive_flag_is_accepted_on_every_position() {
    let dir = scratch_dir("flag");
    let output = ag(&["--no-interactive", "new", "one"], &dir);
    assert!(output.status.success(), "{output:?}");
    let output = ag(&["new", "two", "--no-interactive"], &dir);
    assert!(output.status.success(), "{output:?}");
    assert_no_escape_sequences(&output);
}

#[test]
fn existing_directory_is_an_error_and_nothing_is_overwritten() {
    let dir = scratch_dir("exists");
    std::fs::create_dir(dir.join("taken")).expect("mkdir");
    let output = ag(&["new", "taken", "--template", "rest"], &dir);
    assert_eq!(output.status.code(), Some(1));
    assert_no_escape_sequences(&output);
    assert_eq!(
        std::fs::read_dir(dir.join("taken")).expect("read").count(),
        0
    );
}

#[test]
fn help_and_read_only_commands_emit_no_escape_sequences() {
    let dir = scratch_dir("help");
    for args in [
        vec!["--help"],
        vec!["new", "--help"],
        vec!["--version"],
        vec!["schema", "lint", "--schema", "missing.ag"],
    ] {
        let output = ag(&args, &dir);
        assert_no_escape_sequences(&output);
    }
}
