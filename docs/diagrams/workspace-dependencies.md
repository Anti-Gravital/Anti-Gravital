# Workspace dependency diagram

Inter-crate dependency edges between Anti-Gravital crates, derived from the
Cargo manifests (`crates/*/Cargo.toml`). Solid arrows are direct Cargo
dependencies; dotted arrows are feature-gated or deferred relationships.
Source of truth: CLAUDE.md sections 14-15 and the manifests.

```mermaid
graph TD
  subgraph Nucleo
    ag_core[ag-core]
    ag_dsl[ag-dsl]
    ag_cli[ag-cli]
    ag_lsp[ag-lsp]
    ag_wasm[ag-wasm-host]
  end
  subgraph Estandar
    ag_auth[ag-auth]
    ag_data[ag-data]
    ag_realtime[ag-realtime]
    ag_cache[ag-cache]
    ag_storage[ag-storage]
    ag_observe[ag-observe]
  end
  subgraph Estandar_diferido
    ag_mail[ag-mail]
    ag_workers[ag-workers]
  end
  subgraph Infra_opcional
    ag_domains[ag-domains]
    ag_edge[ag-edge]
  end

  ag_data --> ag_core
  ag_cache --> ag_core
  ag_domains --> ag_core
  ag_mail --> ag_core
  ag_mail --> ag_workers
  ag_workers --> ag_data
  ag_auth --> ag_data
  ag_auth --> ag_mail
  ag_storage --> ag_auth
  ag_lsp --> ag_dsl
  ag_edge --> ag_domains
  ag_cli --> ag_dsl
  ag_cli --> ag_domains
  ag_cli --> ag_mail
  ag_cli --> ag_workers
  ag_edge -. producer feature, issue 112 .-> ag_workers
  ag_cloud[ag-cloud] -. ag deploy .-> ag_domains
```

Invariants enforced by these edges (CLAUDE.md rule 15):

- `ag-core` depends on no other Anti-Gravital crate.
- No cycles.
- `ag-mail` does not depend on `ag-auth` (the reverse holds).
- `ag-workers` does not depend on `ag-edge`; the only allowed direction is
  `ag-edge -> ag-workers` behind the `producer` feature (issue #112).
