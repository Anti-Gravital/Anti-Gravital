# wasi-plugin

WASI plugin host skeleton: a self-contained example of the host-side contract
that a WebAssembly plugin runtime needs, without executing any module. It
builds and runs offline.

## What it provides

- `PluginManifest`: name, module path and requested capabilities.
- `Capability`: an allow-listed permission (`fs_read`, `fs_write`, `net_out`,
  `env_read`).
- `PluginRegistry`: register (with duplicate and capability validation), get,
  list and unload.
- `WasiHost`: load/unload transitions over the registry, with a single marked
  execution hook.
- A demo that registers a plugin and, if a module path is supplied, walks the
  load -> unload lifecycle.

## Run

```sh
ag new my-host --template wasi-plugin --no-interactive
cd my-host
cargo run
```

With no external input the host registers the demo plugin and reports that the
load transition was skipped:

```sh
cargo run
# -> plugin registry ready; WASI_MODULE_PATH is unset; skipping the load transition
```

Supplying a path exercises load and unload (the module is not executed):

```sh
WASI_MODULE_PATH=./plugins/demo.wasm cargo run
```

## Configuration

`config.toml` documents the bind address (informational for this CLI skeleton)
and the host capability policy. The current scaffold uses the compiled-in
policy `fs_read` and `env_read`.

## External requirements

- None for the default build and runtime.
- `WASI_MODULE_PATH` is optional. It only enables the load/unload demo and is
  read from the environment, never hardcoded. This is declared in
  `template.toml` metadata.

## Integration with platform crates

`ag-wasm-host` is linked as the reserved WASI runtime boundary. At the pinned
revision it is a Phase 0 skeleton with no public API, so module discovery,
validation, sandboxing and execution are implemented as a local lifecycle
model. Replace the marked `EXECUTION HOOK` with the `ag-wasm-host` API when it
becomes available; the registry and capability policy remain unchanged.

## Limitations

- No WebAssembly is compiled, parsed or executed. The host is a lifecycle
  skeleton, not a runtime.
- Capability enforcement is a check at registration time only; it does not
  sandbox syscall access.
- No persistent registry, module integrity checks or signature verification.
- The first build resolves the pinned git revision from the local cargo cache
  when building offline.

## Status

Template, experimental. Builds and runs offline with `cargo check --offline`.
