# Fase 9 - Sistema de plugins WASI

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-08-ag-mobile.md](./fase-08-ag-mobile.md)
> Siguiente: [fase-10-endurecimiento-y-1.0.md](./fase-10-endurecimiento-y-1.0.md)

## Phase 9 — WASI plugin system

**Objective.** Build the WASI plugin system with wasmtime, define the stable ABI, publish the official plugins, and start the public registry.

### 9.1 Entry criteria

- [ ] Phase 8 completed.
- [ ] RFC decision on the scope of the 1.0 plugin ABI. Approved by the technical committee (formed in phase 4 or earlier).

### 9.2 Deliverables

- [ ] `ag-wasm-host` crate operational over wasmtime.
- [ ] Definition of WIT interfaces (WebAssembly Interface Types) for the host.
- [ ] Specification of `plugin.toml`.
- [ ] Implementation of the plugin life cycle (discovery, validation, loading, activation, unloading).
- [ ] Sandbox with memory, fuel and timeout limits.
- [ ] Official plugins: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`.
- [ ] `ag plugin add/remove/list` command.
- [ ] Public registry at `plugins.antigravital.dev`.
- [ ] Guide: "How to write a plugin for Anti-Gravital" with examples in Rust, Go (TinyGo) and AssemblyScript.
- [ ] Plugin installation shows the capabilities a plugin requests and asks
  for confirmation (interactive per `RFC-0018`, with flag parity).

RFC prerequisites (rule 28), in addition to the ABI RFC of 9.1:

- [ ] RFC for the `ag plugin` commands and the registry trust model
  (signing, integrity verification, revocation).

Stages:

1. **9-A, host.** WIT interfaces, `plugin.toml`, life cycle, sandbox limits.
2. **9-B, CLI.** `ag plugin add/remove/list` against local plugins.
3. **9-C, official plugins.**
4. **9-D, registry.** Public registry with integrity verification.
5. **9-E, guide and overhead benchmark.**

### 9.3 Exit criteria (gate before Phase 10)

- [ ] The registry publishes at least the six official plugins.
- [ ] At least three third-party external plugins published in the registry.
- [ ] The benchmark shows plugin overhead ≤ 1% over an equivalent native handler.
- [ ] At least 6 000 stars on the repository.

### 9.4 Phase risks

The main risk is the complexity of the WebAssembly component model, which keeps evolving. The mitigation is conservative pinning of the supported version and early commitment with the wasmtime community.

---

