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

Implemented through Phase 4.7. Nine top-level commands (plus bare `ag`):

| Command | Phase | Purpose |
| --- | --- | --- |
| `ag new` | 2, 4.7-A | Scaffold a project from the `rest`, `realtime` or `fullstack` template. With missing input in an interactive terminal, a full-screen wizard asks for it (`RFC-0018`) |
| `ag dev` | 2 | Development server with hot reload via `cargo-watch` |
| `ag build` | 2 | Release build |
| `ag config` | 4.7-B | Form for `config.toml` in a terminal; `get`, `set` and `validate` provide non-interactive parity through one shared executor (`RFC-0018`) |
| bare `ag` | 4.7-C | Home screen that launches existing commands with keyboard navigation; prints help outside a terminal (`RFC-0018`) |
| `ag generate` | 3 | Compile `schema.ag` into SQL, Rust, TypeScript and OpenAPI |
| `ag schema lint`, `ag schema diff` | 3 | Schema diagnostics and breaking-change detection |
| `ag dns ...` | additive 4.8 / `RFC-0033` | `plan` (deterministic, offline), `apply` (plan-id gated, adapter opt-in), `status` (read-only); reads the `[domain]` table of `ag.toml` |
| `ag domains ...` | 4.5 / `RFC-0011` | `check`, `sync`, `attach`, `instructions`, `export-zone`, `status`, `list`, `verify`, `detach`, `diagnose`; `attach` also accepts `--manifest ag.toml` |
| `ag mail test` | 4.5 | Send a test message |
| `ag workers ...` | 4.6 / `RFC-0012`, `RFC-0017` | `list`, and the operational group behind the default `workers-runtime` feature (`run`, `enqueue`, `queues`, `dlq`, `doctor`) |
| `ag deploy plan\|apply\|status` | wave 7 / `RFC-0034` | Local, plan-first renderer of authored Docker/Kubernetes artifacts into `deploy/generated/`. No cluster, runtime or network access; not the Phase 5 cloud deploy |

The public command tree is pinned by the test
`command_tree_matches_the_documented_public_surface` in
`crates/ag-cli/src/main.rs`.

## Not available

`ag rollback`, `ag migrate` and `ag plugin` do not exist. They arrive with
their roadmap phases (5, 7 and 9) and each requires an approved RFC first.
`ag deploy` exists only as the local artifact renderer of `RFC-0034`; the
Phase 5 cloud deploy with registry push and cluster apply is still not
implemented.

## Source layout

| Path | Contents |
| --- | --- |
| `src/main.rs` | Dispatch and exit codes |
| `src/cli.rs` | clap definitions, including the global `--output` selector |
| `src/commands/` | One module per command group |
| `src/output.rs` | Versioned JSON envelope and read-command data shapes (`docs/mcp/`) |
| `src/request.rs` | Typed requests shared by flags and interface |
| `src/config.rs` | Config keys, typed config requests and the single validated executor |
| `src/interactive.rs` | Decision on whether a session may prompt |
| `src/tui/` | Interactive interface (`tui` Cargo feature) |

## Interactive interface (`RFC-0018`, accepted)

Stages A, B and C are implemented:

- Stage A: the `ag new` wizard, the global `--no-interactive` flag, automatic
  non-interactive mode, monochrome theme, degraded line prompts and terminal
  restoration.
- Stage B: `ag config get|set|validate` and the full-screen configuration form.
  Both build a typed `ConfigRequest` handled by the single `execute_config`
  executor. Keys mirror `ag_core::ShieldConfig`; writes are validated with that
  loader before an atomic, format-preserving replace. Secret material is not
  exposed. `ag new --with MODULE` (RFC-0018 section 4.4, module selection) is
  still not implemented; it requires tested scaffold composition first.
- Stage C: the bare-`ag` home screen, a launcher that returns a command line
  for `main` to re-parse and run.

Flow and architecture diagrams: `docs/diagrams/cli-interactive-flow.md`.
