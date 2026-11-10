# ag-cli

> Architecture chapter: `docs/architecture/05-ecosistema-modulos.md`.
> Crate README (authoritative command reference): `crates/ag-cli/README.md`.
> Criticality: Core.
> Implementation phases: Phase 2 through Phase 9.

## Domain

The `ag` binary. It orchestrates project creation, DSL generation, the
development server, release builds and the operational commands of the modules
that expose one.

## Allowed internal dependencies

Depends on the other crates through Cargo features, so the binary can be built
with a reduced subset.

## Applicable rules

See rules 14 and 15 of `CLAUDE.md` for the core/standard/optional separation
and the dependency policy between Anti-Gravital crates.

## State

Implemented through Phase 4.6. Eight top-level commands:

| Command | Phase | Purpose |
| --- | --- | --- |
| `ag new` | 2 | Scaffold a project from the `rest`, `realtime` or `fullstack` template |
| `ag dev` | 2 | Development server with hot reload via `cargo-watch` |
| `ag build` | 2 | Release build |
| `ag generate` | 3 | Compile `schema.ag` into SQL, Rust, TypeScript and OpenAPI |
| `ag schema lint`, `ag schema diff` | 3 | Schema diagnostics and breaking-change detection |
| `ag domains ...` | 4.5 / `RFC-0011` | `check`, `sync`, `attach`, `instructions`, `export-zone`, `status`, `list`, `verify`, `detach`, `diagnose` |
| `ag mail test` | 4.5 | Send a test message |
| `ag workers ...` | 4.6 / `RFC-0012`, `RFC-0017` | `list`, and the operational group behind the default `workers-runtime` feature (`run`, `enqueue`, `queues`, `dlq`, `doctor`) |

The public command tree is pinned by the test
`command_tree_matches_the_documented_public_surface` in
`crates/ag-cli/src/main.rs`.

## Not available

`ag deploy`, `ag rollback`, `ag migrate` and `ag plugin` do not exist. They
arrive with their roadmap phases (5, 7 and 9) and each requires an approved
RFC first.

## Proposed

`RFC-0018` proposes an interactive full-screen terminal interface for project
creation and configuration (roadmap Phase 4.7), including a new `ag config`
command. It is a proposal: no part of it is implemented.
