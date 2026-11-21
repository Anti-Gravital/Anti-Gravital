# MCP layer flow

> Diagram for RFC-0024. Last reviewed: 2026-11-21.

## Logical flow

```mermaid
flowchart LR
    Agent[Connected agent] -->|stdio| Contract[Versioned MCP contract]
    Contract --> Server[ag mcp serve]
    Server --> Caps{Capability manifest}
    Caps -->|read| ReadTools[project.list, schema.lint, config.get]
    Caps -->|write grant| MutTools[project.create, workers.enqueue]
    MutTools --> Plan[Plan identifier]
    Plan --> Apply[Explicit apply call]
    ReadTools --> Executor[Shared request executors]
    Apply --> Executor
    Executor --> Audit[Local audit log]
    Server -->|AG_MCP_DEBUG=1| Debug[debug.state, debug.diagnostics, debug.trace]
    Debug --> Redact[Secret redaction]
    Agent -.->|client adapters| External[External MCP servers]
```

## Boundary with Gravital Fabric

```mermaid
flowchart TB
    subgraph AG[Anti-Gravital - open source]
        Cli[ag CLI + MCP server]
        Fw[Framework crates]
    end
    subgraph Contract[Versioned open contracts]
        Proto[Protobuf / OpenAPI / JSON Schema]
    end
    subgraph Fabric[Gravital Fabric - separate repository]
        Control[Control plane]
        Engines[Motors: compute, storage, network, migration]
    end
    Cli --> Proto
    Control --> Proto
    Fw -.->|optional adapter, no crate dependency| Engines
```

No `ag-*` crate imports a `fabric-*` crate, and no `fabric-*` crate imports an
`ag-*` crate. The arrows that cross the boundary are data contracts, not Rust
dependencies (ADR-0016).
