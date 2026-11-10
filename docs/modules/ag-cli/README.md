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
| `ag new` | 2, 4.7-A | Scaffold a project from the `rest`, `realtime` or `fullstack` template. With missing input in an interactive terminal, a full-screen wizard asks for it (`RFC-0018`) |
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

## Source layout

| Path | Contents |
| --- | --- |
| `src/main.rs` | Dispatch and exit codes |
| `src/cli.rs` | clap definitions |
| `src/commands/` | One module per command group |
| `src/request.rs` | Typed requests shared by flags and interface |
| `src/interactive.rs` | Decision on whether a session may prompt |
| `src/tui/` | Interactive interface (`tui` Cargo feature) |

## Interactive interface (`RFC-0018`, accepted)

Stage A is implemented: the `ag new` wizard, the global `--no-interactive`
flag, automatic non-interactive mode, monochrome theme, degraded line prompts
and terminal restoration. Flow and architecture diagrams:
`docs/diagrams/cli-interactive-flow.md`.

Not implemented: stage B (`ag config`, `ag new --with MODULE`) and stage C
(home screen for bare `ag`).
