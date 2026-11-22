# cli-tool

A standalone CLI binary scaffold using `clap`'s derive API and the shared
`ag_core::AgError` type from [ag-core](../../../crates/ag-core). It follows the
same explicit command/error conventions as the `ag` binary without depending
on it (ag-cli has no library target).

## Files

```
Cargo.toml      package manifest (ag-core pinned by rev)
config.toml     documented CLI defaults
src/main.rs     parser, subcommands and typed error handling
template.toml   catalog metadata
```

## Run

```sh
ag new mycli --template cli-tool
cd mycli
cargo run -- --help
cargo run -- init --output config.toml
cargo run -- inspect config.toml
cargo run -- commands
```

Without the CLI, replace `{{name}}` in the source files and run Cargo directly.
The binary name and the `#[command(name = ...)]` value both come from the
project name.

## Structure

- `Cli` is the top-level `clap::Parser` with a `Commands` subcommand enum.
- Each subcommand is a small match arm; add new variants to extend the tool.
- Errors use `AgError` (`Validation` for bad input, `Io` for filesystem
  failures), so exit codes and messages stay consistent with the ecosystem.

## External requirements

None. The scaffold has no network, database or credential requirements and
builds entirely offline.

## Limitations

- This is a command scaffold, not a finished tool: the example subcommands only
  write and inspect a text file.
- ag-cli cannot be used as a library because it exposes only a binary target;
  CLI conventions are copied, not imported.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
