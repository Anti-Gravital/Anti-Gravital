# Template catalog validation evidence

## Purpose

`scripts/validate-templates.sh` is the reproducibility harness for the wave 6
template catalog. It reads every `templates/<name>/template.toml`, checks the
minimal metadata contract, scaffolds a throwaway project, and compiles it
offline. Template sources are never modified: all writes go to a temporary
work root that is removed on exit.

This document records the first real run on the `community6/qa` branch. The
coordinator should rerun the same command after the other wave 6 template
branches are integrated; the harness discovers new templates automatically.

## Environment

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-11-22T04:41:52Z |
| Host | angel-nereira-HP-ProBook-440-14-inch-G9-Notebook-PC |
| Kernel | Linux 7.0.0-44-generic x86_64 |
| Rust | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| Cargo | cargo 1.99.1 (5f94df478 2026-08-27) |
| Commit | e9347ab44b58b0c288a3db433dac327da80ee9c6 |
| Branch | community6/qa |
| Shared target | /home/angel-nereira/Github-local-angelnereira/shared-cargo-target |

Environment variables exported for the runs:

```
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
CARGO_INCREMENTAL=0
CARGO_PROFILE_TEST_DEBUG=0
```

## Command

```
./scripts/validate-templates.sh
```

### Harness output (normal mode, exit 0)

```
Template catalog validation
templates dir: /home/angel-nereira/Github-local-angelnereira/worktrees6/qa/templates
ag binary:     /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
cargo target:  /home/angel-nereira/Github-local-angelnereira/shared-cargo-target

[PASS] testing-suite (fallback scaffold, offline check ok; ag new not wired yet)

Summary: 1 passed (0 via ag new, 1 via fallback), 0 failed, 0 skipped, total 1
Note: fallback scaffolds prove template usability but not CLI registry wiring.
      Re-run with STRICT_CLI=1 after the catalog registers every template.
```

### Harness output (DEEP_CHECK=1, exit 0)

```
Template catalog validation
templates dir: /home/angel-nereira/Github-local-angelnereira/worktrees6/qa/templates
ag binary:     /home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/ag
cargo target:  /home/angel-nereira/Github-local-angelnereira/shared-cargo-target

[PASS] testing-suite (fallback scaffold, offline check ok; ag new not wired yet)

Summary: 1 passed (0 via ag new, 1 via fallback), 0 failed, 0 skipped, total 1
Note: fallback scaffolds prove template usability but not CLI registry wiring.
      Re-run with STRICT_CLI=1 after the catalog registers every template.
```

`DEEP_CHECK=1` adds `cargo check --offline --locked --all-targets`, which
compiles the unit, integration, property and benchmark targets. The template
ships `Cargo.lock.tmpl` with versions present in the shared offline cache, so
both passes complete without network access.

### Test execution in the scaffolded project

The same scaffold was then run with `cargo test --offline --locked` and
`cargo run --offline --locked`. Relevant output:

```
   Compiling probe v0.1.0 (/tmp/opencode/tsrun.M9LV3t/probe)
    Finished `test` profile [unoptimized] target(s) in 11.09s
     Running unittests src/lib.rs
running 4 tests
test tests::empty_input_is_a_palindrome ... ok
test tests::partition_preserves_order_and_multiset ... ok
test tests::non_palindrome_is_rejected ... ok
test tests::palindrome_ignores_case_and_punctuation ... ok
test result: ok. 4 passed; 0 failed

     Running tests/integration.rs
running 3 tests
test public_api_is_callable_from_an_external_crate ... ok
test count_lines_reports_missing_files ... ok
test count_lines_handles_empty_and_multiline_files ... ok
test result: ok. 3 passed; 0 failed

     Running tests/property.rs
running 2 tests
test partition_keeps_all_values_and_respects_pivot ... ok
test reversing_preserves_palindrome_classification ... ok
test result: ok. 2 passed; 0 failed

     Running `/home/angel-nereira/Github-local-angelnereira/shared-cargo-target/debug/probe`
"A man, a plan, a canal: Panama" is a palindrome: true
```

Nine tests passed in total: four unit, three integration, two property.

## Interpretation

- At the time of this run only `templates/testing-suite/` declares
  `template.toml` in this worktree. The four pre-existing templates (`rest`,
  `realtime`, `fullstack`, `layered`) do not yet carry metadata; that is
  catalog work and was not modified here.
- The `ag` binary used here was built before catalog's registry wiring, so it
  accepts only the four original names. The harness therefore used its local
  scaffold fallback for `testing-suite`. The fallback applies the same two
  rules as `ag new`: strip `.tmpl` and replace `{{name}}`. It is reported
  explicitly rather than silently counted as an `ag new` success.
- Catalog's design (`build.rs` auto-discovery) will make every merged template
  directory scaffoldable without further CLI edits. After that merge, rebuild
  `ag` and rerun with `STRICT_CLI=1` to prove registry wiring: any template
  that still needs the fallback then fails the run.

## CI integration decision

A CI job was not added in this change. The harness currently depends on a
prebuilt `ag` binary, a warm offline crate cache, and template branches that
are still being integrated. Wiring those moving pieces into a required CI job
would be brittle and would fail for reasons unrelated to template correctness.
The manual command above is the documented gate for this wave. Once catalog's
registry is merged and all template branches land, a CI step can call the
same script with `STRICT_CLI=1` as a non-brittle integration check.

## Reproduce

From the repository root:

```
export CARGO_TARGET_DIR=<shared-cargo-target>
./scripts/validate-templates.sh
DEEP_CHECK=1 ./scripts/validate-templates.sh
STRICT_CLI=1 ./scripts/validate-templates.sh   # after catalog registry merges
```

A non-zero exit means at least one template failed, or that no template with
`template.toml` was found. Per-template failure logs are printed inline by the
harness. `KEEP_WORK=1` preserves the scaffolded projects for inspection.
