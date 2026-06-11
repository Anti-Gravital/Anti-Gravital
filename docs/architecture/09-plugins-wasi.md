# Capitulo 9. Sistema de plugins WASI (ag-wasm-host)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 9
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [08-modulos-batteries-included.md](./08-modulos-batteries-included.md)
> Siguiente: [10-despliegue-ag-cloud.md](./10-despliegue-ag-cloud.md)

## 9. WASI plugin system (`ag-wasm-host`)

The extensibility of Anti-Gravital is built on the WebAssembly System Interface (WASI), not on the ecosystem of native Rust crates. This decision has three reasons that make it non-negotiable.

The first reason is security. Plugins are third-party code that the server operator runs. If they were native code, a malicious or defective plugin could corrupt the process memory, escape to arbitrary syscalls, or leak secrets. WASI modules run in a sandbox with explicit permissions declared in the plugin manifest; the plugin cannot access the filesystem, the network, or syscalls that are not declared.

The second reason is multi-language. A WASI plugin can be written in Rust, Go (TinyGo), C, C++, AssemblyScript, Zig, or any language that compiles to WebAssembly. This democratizes the ecosystem: a security expert who writes in Go can contribute an exporter for Datadog without having to learn Rust.

The third reason is ABI stability. The interface between the host and the plugin is defined with `wit-bindgen` and the Component Model, which allows a plugin compiled for one version of Anti-Gravital to keep working with future versions without recompilation, as long as the ABI does not change.

### 9.1 Plugin runtime

The runtime is `wasmtime`, embedded as a Rust crate. Each plugin is loaded into an isolated store with memory limits (256 MB by default, configurable), fuel limits (instruction consumption), and execution timeout.

### 9.2 Plugin lifecycle

The lifecycle of a plugin has five states. The first is **discovered**: the `.wasm` file is in the project's plugins directory and appears in the manifest. The second is **validated**: the host inspects the binary, verifies that the component model is compatible, reads the manifest, and confirms that the requested permissions are authorized. The third is **loaded**: the module is compiled ahead-of-time with Cranelift and stored in memory. The fourth is **active**: the plugin receives events and responds to invocations. The fifth is **unloaded**: the plugin is released, whether by server shutdown or by dynamic reload.

### 9.3 Plugin manifest

Each plugin brings a `plugin.toml` file with its metadata and requested permissions:

```toml
[plugin]
name = "datadog-exporter"
version = "1.2.0"
author = "Gravital Labs"
license = "Apache-2.0"
description = "Exports metrics and traces to Datadog"

[abi]
anti_gravital_version = ">= 1.0.0, < 2.0.0"
component_model = "0.5"

[permissions]
network = ["api.datadoghq.com:443", "api.datadoghq.eu:443"]
env = ["DD_API_KEY", "DD_SITE"]
filesystem = []
clock = "read"

[capabilities]
exports = ["metrics_exporter", "trace_exporter"]
imports = ["host_logger", "host_clock"]

[limits]
max_memory = "64MB"
max_execution_time = "5s"
fuel = 100_000_000
```

### 9.4 Host API exposed to plugins

The host exposes a reduced set of capabilities to plugins, defined in WIT (WebAssembly Interface Types) interfaces. The main ones are: logger (write messages to the host's tracing system), clock (get the current time and measure intervals), metrics (register additional metrics), KV (persistent key-value storage per plugin), HTTP client (with an allowlist of hosts from the manifest), and events (subscription to the internal bus).

### 9.5 Framework extension points

Plugins can extend Anti-Gravital at five points: additional middleware in the Shield (request hooks), custom handlers registered in the router, observability exporters (metrics, traces, logs), event processors (subscribers to the internal bus), and custom CLI commands (`ag <plugin-cmd>`).

### 9.6 Official plugins

The repository maintains a set of official plugins under `plugins/`, each with its own crate and release cycle: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`. The existence of official plugins serves as a technical reference and as an implementation example for third parties.

### 9.7 Plugin registry

Starting from version 1.0 of the framework, an official registry is published at `plugins.antigravital.dev`. The registry indexes plugins with verified metadata, basic security scanning, and community reviews. Installation is done with `ag plugin add <name>`. Plugins are downloaded, validated, and registered in the project manifest.

---

