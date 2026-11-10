# RFC-0019 - Layered architecture of generated projects and interoperability scope

- Status: proposed
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: additive Phase 4.8, stage A; interoperability targets land in
  the phases named in section 4.6
- Affected modules/crates: `ag-cli` (scaffolder), `templates/`, `ag-dsl`
  (generator output paths), `ag-ui` (scope decision)
- Predecessor RFC: RFC-0018 (the creation wizard configures what this RFC
  defines)
- Comment period: seven calendar days, or waived by BDFL decision. Merging the
  pull request that carries this RFC records the decision in section 9.

## 1. Motivation

Anti-Gravital has a precise architecture for itself (crate tiers, dependency
rules, Shield and Core) and none for the projects it generates. `ag new`
produces a `Cargo.toml`, one `src/main.rs`, a `config.toml` and, for one
template, a `migrations/` directory. Where the domain model lives, where
database code lives, where an integration with an external service lives, and
which of them may depend on which, is left to each user.

A tool whose purpose is to make backend construction sustainable has to hand
the user a structure that stays maintainable at fifty endpoints, not only at
one. The structure also has to be explicit enough that a person, a reviewer or
an automated agent can tell where a change belongs without reading the whole
project.

This RFC also fixes the interoperability scope the project commits to:
strictly backend, integrating with the dominant infrastructure tools, with
generated clients for the user-interface side.

## 2. Problem

1. Generated projects have no layer boundaries. Handler code can call `sqlx`
   directly, and nothing prevents domain types from importing HTTP types.
2. Generated code (`ag generate` writes to `generated/`) and hand-written code
   have no defined relationship inside a project.
3. There is no project manifest. The tool cannot answer "which modules does
   this project use" without parsing `Cargo.toml` and guessing.
4. The roster contains `ag-ui` (server-side rendering with HTMX), a
   user-interface capability, while the product direction is backend only with
   Flutter and WebAssembly clients on the interface side.
5. gRPC and Go are named as interoperability goals but appear in no master
   document, so nothing can be built for them (rule 5).

## 3. Alternatives considered

**A. Leave structure to the user.** No cost. Rejected: it is the status quo
described in section 2.

**B. Document a recommended layout only.** Cheap. Rejected: an unenforced
convention erodes with the first deadline; this is the degradation the project
wants to prevent.

**C. One crate with layer modules, enforced by a custom lint (`ag check`).**
Fast builds, single manifest. Rejected as the default: it requires writing and
maintaining an import analyzer, and its guarantees are weaker than the
compiler's.

**D. A Cargo workspace per project, one crate per layer, enforced by Cargo
(chosen).** A layer cannot use what its `Cargo.toml` does not list. The
enforcement is the compiler, costs no custom tooling and is understood by
every Rust developer.

## 4. Proposed design

### 4.1 Layers and the dependency rule

A generated project is a Cargo workspace with four layer crates. Dependencies
point inward only.

```
            api  ----------->  app  ----------->  domain
             |                  ^
             v                  |
           infra  --------------+   (implements the ports declared in app)

   bin (composition root): builds infra, injects it into app, serves api
```

| Layer | Crate | Contains | May depend on | Must not depend on |
| --- | --- | --- | --- | --- |
| Domain | `<name>-domain` | Entities, value objects, domain errors, invariants. Pure Rust. | `serde`, `uuid`, `chrono`, `thiserror` | Any `ag-*` crate, `sqlx`, `axum`, `tokio`, anything that performs I/O |
| Application | `<name>-app` | Use cases and the **ports**: traits that state what the application needs (repositories, mailer, clock, queue) | `domain`, `async-trait` | `sqlx`, `axum`, any concrete adapter |
| Infrastructure | `<name>-infra` | **Adapters** that implement the ports: database (`ag-data`), cache (`ag-cache`), storage, mail, workers, external services | `app`, `domain`, `ag-data`, `ag-cache`, `ag-storage`, `ag-mail`, `ag-workers` | `api` |
| Interface | `<name>-api` | Transport: HTTP routes and extractors (`ag-core`), realtime (`ag-realtime`), and later gRPC services. Maps transport types to use cases. | `app`, `domain`, `ag-core`, `ag-auth`, `ag-realtime` | `infra`, `sqlx` |
| Composition root | `<name>` (binary) | `main.rs`: load configuration, build adapters, wire use cases, start the server | all of the above, `ag-observe` | - |

Infrastructure is internally divided by concern, one module per external
system, so the database layer is its own clearly bounded unit:
`infra/src/db/` (repositories, queries, row mapping), `infra/src/cache/`,
`infra/src/storage/`, `infra/src/mail/`, `infra/src/jobs/`,
`infra/src/clients/` (one submodule per external service).

### 4.2 Project tree

```
my-api/
|-- ag.toml                 project manifest (4.3)
|-- schema.ag               Anti-DSL source of truth
|-- Cargo.toml              workspace
|-- crates/
|   |-- domain/
|   |-- app/
|   |-- infra/
|   |   `-- migrations/     SQL migrations, owned by the database adapter
|   `-- api/
|-- src/main.rs             composition root
|-- config/                 non-secret configuration per environment (RFC-0020)
|-- deploy/                 generated deployment artifacts (Phase 5)
`-- clients/                generated client SDKs (TypeScript, Dart, Go)
```

### 4.3 Project manifest `ag.toml`

One file answers what the project is. It is read by every `ag` command and by
the TUI; it replaces guessing from `Cargo.toml`.

```toml
[project]
name = "my-api"
layout = "layered"          # or "compact" (4.5)

[modules]                   # which Anti-Gravital modules are enabled
data = true
auth = true
cache = false

[generate]                  # where each generator writes
rust = "crates"             # generated code goes to the layer it belongs to
openapi = "clients/openapi.yaml"
typescript = "clients/typescript"
```

Environment variables and secrets are declared in the same file by `RFC-0020`.

### 4.4 Generated versus hand-written code

`ag generate` writes each artifact into the layer it belongs to, always under
a `generated/` module that is never edited by hand and is fully rewritten on
each run:

| Artifact | Destination |
| --- | --- |
| Model structs and validation | `crates/domain/src/generated/` |
| Repository traits (ports) | `crates/app/src/generated/` |
| SQL migrations and `sqlx` repository implementations | `crates/infra/migrations/`, `crates/infra/src/db/generated/` |
| Route definitions and request/response types | `crates/api/src/generated/` |

Hand-written code lives beside `generated/` and extends it. A CI-usable
command, `ag generate --check`, exits non-zero when the committed generated
code does not match `schema.ag`.

### 4.5 Compact layout

The `rest` template for a first contact keeps a single crate with the same
four names as modules (`domain`, `app`, `infra`, `api`). `ag.toml` records
`layout = "compact"`. A later command can promote a compact project to the
layered workspace; that command is out of scope here and needs its own RFC.
Layer names and responsibilities are identical in both layouts, so
documentation is written once.

### 4.6 Interoperability scope (normative)

Anti-Gravital is a backend tool. It integrates with the tools below; it does
not replace them (rule 12) and it does not build user interfaces.

| Technology | Role in Anti-Gravital | Mechanism | Phase |
| --- | --- | --- | --- |
| Docker | Packaging | Generated multi-stage Dockerfile and compose file | 5 |
| Kubernetes | Orchestration | Generated standard manifests, consumable by existing tooling | 5 |
| Terraform | Infrastructure provisioning | Generated variable definitions and outputs for the resources a project needs (database, cache, bucket, DNS); the user's Terraform owns the resources | 5 (new deliverable, needs the Phase 5 target RFC) |
| gRPC | Service-to-service transport | `.proto` generated from `schema.ag`; a gRPC adapter in the interface layer behind a Cargo feature; HTTP stays the default | New DSL target, own RFC before code |
| Go | Consumer language | Generated Go client from the same contract. No Go runtime inside Anti-Gravital | With the gRPC RFC |
| Rust | Implementation language | - | - |
| WebAssembly | Extension and client | WASI plugins (Phase 9); a client SDK usable from WebAssembly front-ends | 9 |
| Flutter | User-interface side | Generated Dart SDK (`ag-mobile`) | 8 |

### 4.7 `ag-ui`

`ag-ui` (server-side rendering with HTMX) is the only crate whose purpose is
to render a user interface. It is an empty skeleton. This RFC proposes
removing it from the roster so that the scope statement of 4.6 is true without
exceptions. Removing a crate from the roster changes rule 14 and the three
master documents; it is listed as an explicit decision in section 9.

## 5. Implementation plan (only after approval)

1. PR 1 - `ag.toml` reader in `ag-cli`; existing commands keep working without
   it.
2. PR 2 - compact layout applied to the `rest` template.
3. PR 3 - layered workspace for `fullstack` and `realtime`; each template
   builds and runs in CI from a fresh `ag new`.
4. PR 4 - generator output routed per layer; `ag generate --check`.
5. PR 5 - master documents (architecture chapter on project structure) and the
   first-API manual chapter.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Four crates feel heavy for a small API | High | Medium | Compact layout for the entry template; identical vocabulary. |
| Existing generated projects no longer match the templates | Medium | Low | No published release exists yet; templates are pinned by commit. Documented as a pre-0.5 breaking change. |
| Ports added for every trivial call (ceremony) | Medium | Medium | The generator emits the repository ports; documentation states that a port exists only for I/O or for a second implementation. |
| Longer incremental builds | Medium | Low | Measure before and after on the `fullstack` template; record the result. |

## 7. Impact

- **Scope:** defines project structure; adds no runtime capability.
- **Roadmap:** Phase 4.8 stage A. Adds a Terraform deliverable to Phase 5 and
  opens the gRPC/Go target for a future RFC.
- **Public APIs:** generator output paths change (pre-release).
- **Documentation:** new architecture chapter; templates README; manual.

## 8. Rollback

Templates are data. Reverting the template and generator-path changes restores
the previous single-file scaffold. `ag.toml` is additive and can remain.

## 9. Decision

To be completed after the comment period.

- Decider: BDFL.
- Decision date: pending.
- Result: pending.

Open questions for the decision:

1. **Default layout.** Layered workspace as the default for `fullstack` and
   `realtime`, compact for `rest` (proposed), or compact everywhere with the
   workspace as an option.
2. **`ag-ui`.** Remove it from the roster (proposed), or keep it as optional.
3. **gRPC and Go.** Confirm both enter the scope, to be designed in their own
   RFC, or leave them out.

## 10. References

- `CLAUDE.md` rules 9, 12, 14, 15, 20, 21, 32.
- `RFC-0018`, `RFC-0020`, `RFC-0021`.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`, sections 5 and 8.7.
- `templates/README.md`.
