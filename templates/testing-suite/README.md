# testing-suite template

A starting point for an Anti-Gravital project whose main purpose is testing.
It demonstrates four test layers with dependencies that are already part of
the workspace development set, so a fresh project checks and tests offline.

## What is generated

```
<name>/
|-- Cargo.toml
|-- README.md
|-- benches/benchmark.rs   criterion benchmark example
|-- src/lib.rs             library helpers + #[cfg(test)] unit tests
|-- src/main.rs            minimal binary so cargo run is useful
|-- tests/integration.rs   external-crate API tests using tempfile
`-- tests/property.rs      proptest invariants over generated inputs
```

The library target is named `testing_suite_lib` regardless of the project
name, so the test files compile for every `<name>` accepted by `ag new`.

## Create and run

```
ag new my-tests --template testing-suite --no-interactive
cd my-tests
cargo test --offline
cargo run --offline
cargo bench --offline
```

`cargo test` runs the unit tests in `src/lib.rs`, the two integration test
binaries under `tests/`, and the property tests driven by `proptest`.
`cargo bench` runs the criterion benchmark in `benches/`.

## Verification used by the harness

```
cargo check --offline --locked
```

The template ships `Cargo.lock.tmpl` with versions already present in the
shared offline cache, so the generated project checks without network access.
The harness only reads template sources; every write happens in a temporary
directory.

Set `DEEP_CHECK=1` when invoking `scripts/validate-templates.sh` to add:

```
cargo check --offline --locked --all-targets
```

That extra pass compiles the unit, integration, property and benchmark
targets, which plain `cargo check` does not verify by default.

## Dev dependencies

- `proptest` 1.x: property-based tests in `tests/property.rs`.
- `tempfile` 3.x: isolated temporary directories in integration tests.
- `criterion` 0.8.x with `cargo_bench_support`: benchmark example.
  Default features are disabled to avoid pulling a plotting toolchain.

No runtime dependency is declared. The helpers are pure Rust so the project
builds with only the offline crate cache.

## Limitations

- The examples are intentionally small; they teach test structure, not
  coverage targets or CI policy.
- `cargo bench` needs the criterion crate available in the offline cache.
  If it is missing, remove the `[[bench]]` block and `benches/` directory;
  unit, integration and property tests remain unchanged.
- `proptest` uses a deterministic seed only when `PROPTEST_RNG_SEED` is set;
  without it a failing case is reproduced from the printed seed, not from a
  fixed value.
- No coverage instrumentation, mutation testing or fuzz target is included.
  Those belong to separate dedicated templates or crate-level tooling.
- Offline operation assumes the dev dependencies have already been fetched
  into the shared cargo cache. On a cold machine run one online `cargo fetch`
  in the Anti-Gravital workspace first.
