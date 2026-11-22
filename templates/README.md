# templates/

Project templates consumed by `ag new`. Each template is a complete tree that
the `ag` binary copies and renames when a project is created.

## Available templates

The authoritative registry of all 33 templates, their categories, purposes and
external requirements is [CATALOG.md](CATALOG.md). The four original templates
are described below; every other name in the catalog is discovered at build
time from its own `templates/<name>/` directory.

Valid values for `ag new -t <template>`:

- `rest`: minimal REST API on `ag-core` (`Shield` + axum) with one example `/`
  route. This is the default when `-t` is omitted in a non-interactive session.
- `realtime`: WebSocket server on `ag-core` with axum's `ws` support (`/ws`).
- `fullstack`: REST with PostgreSQL persistence through `ag-data` (connection
  and migrations in `migrations/`).
- `layered`: opt-in four-crate Cargo workspace with an `ag.toml` manifest
  (RFC-0019 stage A). See the dedicated section below.

The three default templates (`rest`, `realtime`, `fullstack`) are unchanged by
the addition of `layered`. `rest` remains the non-interactive default.

## Pinning of `ag-*` dependencies

While the `ag-*` crates are not published on crates.io, their dependencies in
every template are pinned deterministically to a repository commit with
`rev = "<sha>"`. A project generated with `ag new` always builds against that
fixed commit, not against a moving branch head, so an upstream incompatible
change cannot break a freshly created project (rules 31 and 36).

`ag new` copies the manifests verbatim, so the `rev` is preserved in the
generated project.

Procedure for bumping the pin:

1. Pick a stable commit on the default branch verified green in CI.
2. Replace the `rev` in the manifests of `rest`, `realtime`, `fullstack` and
   in the two `layered` manifests that reference `ag-*` crates
   (`crates/infra/Cargo.toml.tmpl`, `crates/api/Cargo.toml.tmpl`).
3. Regenerate one project per template and run `cargo build` before merging.

When crates.io publication starts, each `{ git = ..., rev = ... }` is replaced
by a SemVer requirement (`ag-core = "x.y"`).

## Layered template (RFC-0019 stage A)

`ag new <name> --template layered` generates this tree:

```
<name>/
|-- ag.toml                  project manifest (layout = "layered")
|-- Cargo.toml               workspace + binary composition root
|-- config.toml
|-- src/main.rs              composition root
`-- crates/
    |-- domain/              entities, value objects, domain errors (pure Rust)
    |-- app/                 use cases and the ports (traits) they depend on
    |-- infra/               adapters implementing the ports; owns migrations/
    `-- api/                 HTTP transport through ag-core
```

Dependencies point inward only, and Cargo enforces it:

| Crate | May depend on | Must not depend on |
| --- | --- | --- |
| `domain` | `serde`, `thiserror` | `app`, `infra`, `api`, any `ag-*`, `sqlx`, `axum`, `tokio` |
| `app` | `domain` | `infra`, `api`, `sqlx`, `axum`, any `ag-*` |
| `infra` | `app`, `domain`, `ag-data` | `api`, `sqlx` |
| `api` | `app`, `domain`, `ag-core`, axum | `infra`, `sqlx` |
| root binary | all four layer crates, `tokio`, `tracing` | - |

Sibling dependencies use Cargo aliases (`domain = { package = "<name>-domain",
path = "..." }`), so the Rust identifiers are stable for any project name.

The generated manifest follows RFC-0019 section 4.3:

```toml
[project]
name = "<name>"
layout = "layered"

[modules]
data = true
auth = false
cache = false

[generate]
rust = "crates"
openapi = "clients/openapi.yaml"
typescript = "clients/typescript"
```

`modules` values reflect the dependencies actually present in this minimal
slice; enabling a module means adding its adapter and declaring it here. The
port in `app` is synchronous; making it asynchronous is a project-level choice
and does not change the layer boundaries.

Scope boundary: stage A delivers the layout, the inward dependency rules and
the manifest. It does not add an `ag.toml` reader, layer-aware generator output
or a compact-to-layered promotion command (RFC-0019 sections 4.4, 4.5). `ag dev`
keeps working because the root package is still a binary that `cargo run` can
start.

Verified from a fresh scaffold: `cargo metadata --no-deps` resolves all five
packages and `cargo check` compiles the workspace against the pinned `rev`.

## Status

Four templates delivered: `rest`, `realtime` and `fullstack` (default set,
consumed by `ag new` via `include_str!`) plus `layered` (opt-in, RFC-0019 stage
A). All `ag-*` dependencies are pinned by `rev` as described above.
