# Template design conventions

English canonical reference for project templates and their deployment artifacts.
This document defines conventions; it changes no template and no CLI code. It is
the shared contract for the wave-7 deployment work so that Docker, Kubernetes,
the `deploy` command and the template catalog agree on the same shapes.

Status: **normative design, partially adopted**. Sections state the target
convention and, where relevant, the current adoption state. A convention is not
a claim that every template already implements it. Gaps are listed in section 12
and are follow-up work for template owners, not silent debt.

Authorities this document serves, and does not amend:

- `RFC-0019`: project layout (compact/layered), `ag.toml`, inward dependencies,
  interoperability scope (Docker and Kubernetes are peers, rule 12).
- `RFC-0020`: declared environment variables, `Secret<T>`, resolution order,
  native encrypted secrets, external stores as adapters.
- `RFC-0021`: plan/apply and machine-readable output for mutating commands.
- `ADR-0009`: honest state reporting, native-by-default with adapters behind
  Cargo features, documentation in the same change as behaviour.
- `ADR-0011`: no third-party brand names as component names; adapter labels only.
- `CLAUDE.md` rules 12, 14, 15, 16, 17, 18, 19, 20, 31 and 36.

This is a technical reference, so English is canonical (`ADR-0008`). For the
bilingual discovery index see `docs/templates/README.md` and `docs/INDEX.md`.

---

## 1. Standard project skeleton

Every server-shaped template uses the following tree. A template may omit a
directory when the omission is justified in its `README.md` (for example a
static-site has no `schema.ag` and a `cli-tool` has no listener). Omissions are
declared, never silent.

```
<name>/
|-- ag.toml                  project manifest: project, modules, generate, env
|-- schema.ag                DSL source of truth (when the template has a data
|                            model or API contract)
|-- Cargo.toml               workspace or single-crate manifest
|-- config/
|   |-- default.toml         non-secret settings, committed in clear
|   |-- <env>.toml           per-environment overrides (dev, staging, prod)
|   |-- <env>.recipients     public keys allowed to read that environment
|   `-- <env>.secrets        age-encrypted secrets, safe to commit
|-- src/main.rs              composition root (compact layout)
|-- crates/                  layered layout only: domain, app, infra, api
|   `-- infra/migrations/    SQL migrations owned by the database adapter
|-- tests/                   integration tests at the HTTP boundary
|-- deploy/                  generated Docker/Kubernetes artifacts (wave 7)
|-- clients/                 generated SDKs and OpenAPI document
|-- README.md                run commands, environment, limitations
`-- .gitignore               excludes .env, *.key and decrypted outputs
```

Layered projects follow `RFC-0019` section 4.1: `domain` (pure Rust), `app`
(use cases and ports), `infra` (adapters implementing ports), `api` (transport),
with `src/main.rs` as the composition root that injects adapters. Compact
projects keep the same four names as modules in one crate (`RFC-0019` 4.5).
The vocabulary is identical in both layouts so documentation is written once.

| Path | Status | Rule |
| --- | --- | --- |
| `template.toml` in the catalog source | Required | Four-field metadata contract of the wave-6 board; `name` equals the directory name. |
| `Cargo.toml.tmpl` | Required | `ag-*` dependencies pinned by `rev` until crates.io publication (`templates/README.md`). |
| `config.toml.tmpl` or `config/` | Required | Non-secret settings only; never a secret value. |
| `src/main.rs.tmpl` | Required for services | Loads config, builds adapters, starts transport. |
| `schema.ag` | Conditional | Required when the template exposes a model or contract; omitted with a stated reason otherwise. |
| `tests/` | Required | At least one integration test that starts the router and exercises the health contract (section 4). |
| `deploy/` | Generated | Produced by deployment tooling, not hand-edited in a scaffolded project. |
| `README.md` in the catalog source | Required | Run commands, declared variables, external requirements, limitations. |
| `.gitignore` | Required | Must exclude `.env`, `*.key`, decrypted outputs. |

**Current adoption (honest state).** The alpha templates were written before
this convention. The four default templates have `Cargo.toml.tmpl`,
`config.toml.tmpl` and `src/main.rs.tmpl`; only `layered` has `ag.toml`;
`fullstack` and `layered` have migrations; no default template has `config/`,
`schema.ag` or `tests/` in the scaffold. Wave-6 web templates add
`template.toml` and a per-template `README.md`. Bringing older templates to the
full skeleton is a separate, reviewable change per template.

## 2. Naming and environment variable conventions

### 2.1 Names

- Template directory, `template.toml` `name`, and `--template` value are the
  same lowercase `kebab-case` token: `[a-z0-9]+(-[a-z0-9]+)*`.
- The project name passed to `ag new` is also `kebab-case`. The `{{name}}`
  placeholder is substituted by the scaffolder and the `.tmpl` suffix stripped.
- Generated Rust identifiers must not depend on the project name. Cargo aliases
  keep layer crate identifiers stable (`RFC-0019` stage A); a project called
  `my-api` must not require a Rust module named `my_api`.
- Category values are exactly those of the catalog contract: `web`, `api`,
  `realtime`, `infrastructure`, `ai`, `testing`.
- Component names are chosen by the project. Third-party brand names are used
  only as labels of the adapter that integrates that specific service, inside
  that adapter module and behind its Cargo feature (`ADR-0011`).

### 2.2 Environment variables

Three classes exist. A name belongs to exactly one class.

| Class | Rule | Examples |
| --- | --- | --- |
| Framework-owned | Prefix `AG_`. Reserved for `ag` itself; templates must not read or write them. | `AG_INSTALLER_CHANNEL` |
| Conventional | Fixed names with one meaning across all templates. New templates reuse them instead of inventing synonyms. | `BIND`, `RUST_LOG` |
| Application | Declared once in `ag.toml` under `[env.<NAME>]`, `SCREAMING_SNAKE_CASE`, typed, with a description. Undeclared names are not readable through the configuration API (`RFC-0020` 4.1). | `DATABASE_URL`, `REDIS_URL`, `NATS_URL` |

Canonical conventional variables:

| Name | Type | Meaning | Default |
| --- | --- | --- | --- |
| `BIND` | `host:port` | Listen address of this process. Containers and orchestrators must set `0.0.0.0:<port>`; the loopback default is for local development only. | `127.0.0.1:8080` |
| `RUST_LOG` | tracing filter | Log filter consumed by `EnvFilter`. | `info` |

Application names already present in the codebase are preserved during the
`RFC-0020` migration so users do not have to rewrite their environment:
`DATABASE_URL`, `REDIS_URL`, `NATS_URL`, `STORAGE_SIGN_SECRET`,
`STORAGE_JWT_PUBLIC_KEY`. A new template that needs the same concept uses the
same name. The `PORT` name in `RFC-0020` section 4.1 is an illustration of a
declaration, not a second listen-address convention; generated servers use
`BIND`.

Every variable a template reads must appear in three places with the same name
and meaning: the `ag.toml` declaration, the template `README.md`, and the
deployment artifact that supplies its shape. `ag env list` is the checkable
source of truth once that command lands (`RFC-0020` 4.6).

## 3. Configuration and precedence

Normative resolution order, highest priority first (`RFC-0020` 4.2):

1. Process environment.
2. Decrypted secrets file for the selected environment
   (`config/<env>.secrets`).
3. Per-environment file `config/<env>.toml`.
4. `config/default.toml`.
5. Declared default in `ag.toml`.

All variables resolve and validate at startup, before a listener opens. On
failure the process exits non-zero and reports every problem at once: name,
expected type and where it looked. Values are never printed. The result is an
immutable typed configuration passed to the composition root; there is no
global mutable configuration.

`RFC-0020` does not standardize a runtime environment selector. Until it does,
templates and deployment artifacts use the process environment (source 1),
which works today. No template may invent a selector variable and call it a
convention.

**Current adoption.** The `ag-core` stage-B loader implements sources 1 and 5
only; file sources and the `[env]` reader are future stages. Alpha templates
read `BIND`, `RUST_LOG` and `DATABASE_URL` with direct `std::env::var` calls;
`config.toml.tmpl` documents bind settings but is not parsed by generated
`main.rs`. This is a known conflict (section 12), not a capability claim.

## 4. Health and readiness endpoint contract

Server templates expose two unauthenticated probe endpoints. They are part of
the template's public contract and are consumed by Docker health checks and
Kubernetes probes.

| Path | Purpose | Success | Failure | Probe use |
| --- | --- | --- | --- | --- |
| `/health` | Liveness: the process is alive and its event loop serves requests. | `200`, JSON `{"status":"ok","service":"<name>"}` | Connection failure or non-2xx only. Must never probe a dependency. | liveness, Docker `HEALTHCHECK` |
| `/ready` | Readiness: required dependencies are available and the service can accept work. | `200`, same JSON shape with `"status":"ok"` | `503`, JSON `{"status":"degraded","service":"<name>","reason":"<stable-code>"}` | readiness, rollout gating |

Rules:

- Both endpoints set `Content-Type: application/json` and `Cache-Control:
  no-store`. The `service` value is the project name, never a version or a
  secret.
- `reason` is a stable lowercase code such as `database_unavailable` or
  `dependency_not_ready`, not a free-form error and never a connection string.
- Optional adapters do not gate readiness. Only dependencies declared as
  required in `external_requirements` and enabled in `ag.toml` are checked.
- If Shield or another security layer protects routes by default, these two
  paths must be allowlisted for probes. Probes must not need credentials.
- `/healthz` and `/readyz` are not canonical names. An artifact may expose them
  only as aliases of the endpoints above, never with different semantics.

**Current adoption.** `fullstack`, `layered`, `dashboard` and `admin-panel`
expose `/health` (with varying bodies); `rest`, `realtime` and the other web
templates have none. No template currently exposes `/ready`. Until a template
is updated, deployment artifacts for it must use a TCP-socket probe and say so
in their documentation; they must not claim the JSON contract is implemented.

## 5. Structured logging and telemetry

Alpha baseline, already used by generated servers: `tracing` plus
`tracing-subscriber::fmt()` with `EnvFilter::try_from_default_env()` and an
`info` fallback. Logs go to stdout/stderr so a container runtime collects them.

Every new log event uses stable field names so queries survive refactors:

| Field | Meaning |
| --- | --- |
| `service` | Project name. |
| `request_id` | Inbound request identifier; echo `X-Request-Id` when supplied, otherwise generate one. |
| `method`, `path`, `status` | HTTP request outcome, for access logs. |
| `duration_ms` | Observed latency as a number, not a formatted string. |
| `error` | Error kind or code; never a serialized secret or full connection string. |

Rules:

- A `Secret<T>` value cannot be formatted or serialized (`RFC-0020` 4.3), so it
  cannot reach a log or trace layer. Never call `expose()` for logging.
- Do not log environment dumps, request bodies containing credentials, or
  signed URLs. Redact at the source, not in the collector.
- Text format is the alpha default and adds no dependency. A JSON formatter,
  `ag-observe` integration or an OTLP exporter is an opt-in adapter behind a
  Cargo feature. No template requires a collector to start.
- Telemetry beyond stdout is operator-supplied (section 10). The framework
  promises stable field names, not a monitoring backend.

Current adoption: the filter and stdout baseline are present in the default
templates; the stable field set and request identifiers are conventions for
new or updated templates, not a claim about existing code.

## 6. Secret handling (RFC-0020)

- Declare every secret in `ag.toml` with `secret = true`. It is delivered as
  `Secret<T>`: redacted in `Debug`/`Display`, no `Serialize`, zeroized on drop,
  readable only through an explicit `expose()` call.
- Non-secret settings live in committed `config/*.toml`. Secret values live in
  the age-encrypted `config/<env>.secrets` with public keys in
  `config/<env>.recipients`. Private keys never enter the repository.
- A secret value is never written to `template.toml`, `config/*.toml`,
  `Cargo.toml.tmpl`, a Dockerfile, a compose file or a Kubernetes manifest.
  Deployment artifacts emit the **shape** only: `ag env export` produces a
  manifest skeleton, compose `environment` keys or Terraform sensitive
  variables; values are printed only with an explicit flag and only to stdout.
- Secret input uses a hidden prompt or stdin, never a command-line argument
  (`RFC-0020` 4.6), because arguments are visible in process listings.
- Container and cluster deployment uses the process environment (highest
  priority in section 3) or a mounted file read through a `SecretSource`
  adapter. The native encrypted file remains the default; Kubernetes Secrets
  or a secret manager are adapters behind Cargo features, never a requirement.
- Removing a recipient must print the rotation list. Removing a key does not
  un-leak what that recipient already read.
- `ag env check` is the CI gate: it validates the environment as startup would
  without printing values.

## 7. What belongs in a template versus an adapter

| Concern | Template owns | Adapter owns (Cargo feature) |
| --- | --- | --- |
| Skeleton and layer boundaries | Directories, ports/traits in `app`, inward-only dependencies (`RFC-0019`) | Nothing |
| Configuration | Declarations, defaults, native file sources, startup validation | Reading a target-specific secret store behind `SecretSource` |
| Health/readiness | `/health` and `/ready`, including required-dependency checks | Reporting availability of its own backend to readiness |
| Persistence | Port trait and migrations location | Concrete `ag-data` repository for a database |
| Events and jobs | Port trait, in-memory default | NATS or durable PostgreSQL backend |
| Mail | Port trait, native in-process MTA default | SMTP relay endpoint |
| Storage | Port trait, filesystem/in-memory default | S3-compatible backend |
| DNS and TLS | Port trait and routing config | DNS provider and ACME integration via `ag-domains`/`ag-edge` |
| Observability | Stable log fields, stdout | JSON formatter, `ag-observe`, OTLP exporter |
| Deployment | Contract shapes in section 10 | Target-specific wiring (compose, kustomize) |

Every template must have a native default: `cargo check --offline`, `ag dev`
and the integration tests work with no database, broker, cache, bucket, relay
or cloud account. An external service is allowed only when the concept itself
requires a counterpart (for example DNS record management) and the case is
documented in `external_requirements`. Even then a native path is provided
where one exists (`ADR-0009` rules 2 and 5).

`external_requirements = []` in `template.toml` means the full documented path
needs no external service and no credential. An entry such as
`["DATABASE_URL"]` means that variable must be supplied for the full path;
the scaffold still builds and starts its native fallback offline. The gallery
uses the same vocabulary (`docs/templates/GALLERY.md`).

## 8. Composition model

Templates are building blocks, not monoliths. Four composition moments exist:

1. **Creation time.** A base template supplies the root binary and the
   composition root. Additional templates contribute packages or layer crates.
   The `ag new --with <template>` form belongs to `RFC-0018` stage B and is
   **not implemented** in the alpha; composition today is a manual, reviewable
   merge of scaffolds.
2. **Build time.** One Cargo workspace, one `ag.toml`, one `schema.ag`. Each
   selected template lands in its own package. Dependencies point inward
   (`RFC-0019`); no template crate depends on another template crate. Shared
   behaviour is reached through ports declared in `app`, not by importing a
   sibling template.
3. **Run time.** The composition root selects exactly one adapter per port and
   injects it. Cross-service communication uses explicit transport (HTTP,
   event bus), never shared memory. Each listener owns one `BIND` value; a
   composite assigns distinct ports and documents them in its deployment
   overlays. The framework does not allocate a global port registry.
4. **Deploy time.** Compose and kustomize overlays combine the artifacts of
   each template into one stack. The composite exposes one aggregated `/ready`
   that checks only required dependencies of the selected adapters.

Collision rules for any composition:

- `ag.toml` `[env]` declarations merge by name. The same name with the same
  type and meaning is shared; the same name with a different type or meaning is
  a hard error, not a silent override.
- Two templates cannot both own the root binary. Choose one base.
- Two processes cannot bind the same address in one stack. The operator or
  overlay resolves the conflict explicitly.
- Two adapters for the same port cannot both be selected in one composition
  root.
- `external_requirements` sets are unioned; the resulting README lists the
  full set and every native fallback.

This is a design contract. The mechanical part available in this wave is the
deployment overlay work of the `docker`, `k8s` and `stacks` roles; scaffolding a
composite with one command remains future work.

## 9. Upgrade and migration path

Templates are data copied at scaffold time, and `ag-*` dependencies are pinned
to a verified commit (`templates/README.md`). An upgrade is therefore explicit
and reviewable, never automatic.

1. Read the CHANGELOG and the template's migration notes for the target
   revision. Identify changed files and any breaking change to `ag.toml`,
   `BIND`, `external_requirements` or the health contract.
2. Scaffold a fresh reference project with the same template at the new pin,
   in a temporary directory, using the shared Cargo target dir.
3. Diff the reference tree against the project. Ignore user files and
   hand-written code beside `generated/`; review every difference in tracked
   scaffold files.
4. Apply changes by hand. Do not overwrite user code with generated code.
5. Update the `rev` in every template-derived `Cargo.toml`, regenerate
   `Cargo.lock`, then run `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
   `cargo audit` and `cargo deny` (or the project's documented equivalents).
6. Re-render deployment artifacts and run `ag deploy plan` before `apply`
   (`RFC-0021` 4.3); review the plan identifier.
7. Re-run `ag env check` and the offline template validator against the result.

Rules:

- No auto-migration command exists in the alpha. Compact-to-layered promotion
  needs its own RFC (`RFC-0019` 4.5) and is not implied here.
- Until crates.io publication, compatibility is "same pinned commit". After
  publication, each `ag-*` dependency becomes a SemVer requirement and the
  normal stability rules apply (`CLAUDE.md` rules 31 and 36).
- A template may not silently rename a declared environment variable, change
  the health JSON shape, or move a generated output path. Such changes require
  a CHANGELOG entry and a migration note in the same change.

## 10. Compatibility matrix: Docker and Kubernetes

Rule 12 is explicit: Anti-Gravital integrates with Docker and Kubernetes; it
does not replace either. Generated artifacts are ordinary Dockerfiles, compose
files and Kubernetes manifests, consumable by existing tooling. Neither a
container runtime nor a cluster is required to scaffold, build or run a
template locally.

The artifacts referenced below are produced by other wave-7 roles and are not
claimed here as delivered or verified at the time of writing. This section is
the contract they target.

| Concern | Framework generates | Operator supplies | Native default, no external service |
| --- | --- | --- | --- |
| Build | Multi-stage Dockerfile per template family; single static binary in the final stage; `Cargo.lock` used | Base-image digest policy, build context, registry and tags | `cargo build --offline` and `ag dev` on the host, no image build |
| Runtime identity | Non-root `USER`, no secrets baked into layers, read-only filesystem where the template allows it | UID/GFS mapping, seccomp/AppArmor profile, any writable paths | Runs as the local user; no container constraints |
| Configuration | ConfigMap / compose `environment` keys derived from declared `[env]` names; no values | Real values, environment selection, redeploy on change | `config/*.toml` plus process environment |
| Secrets | Secret skeleton or mount reference from `ag env export`; never a value | Values through the target's secret mechanism or environment injection | age-encrypted `config/<env>.secrets`, process environment override |
| Probes | `/health` liveness and `/ready` readiness per section 4, or TCP probe where a template lacks them (documented) | Initial delay, timeouts, failure thresholds | No probes needed for a local process |
| Networking | Service and container port derived from `BIND` convention (`0.0.0.0:<port>` in artifacts) | Ingress, DNS, TLS certificates, network policies | Loopback bind `127.0.0.1:8080` |
| Resources | Conservative requests/limits and compose memory limits | Right-sizing from real measurements | Host process uses normal OS resource limits |
| Scaling | Optional HPA/overlay skeleton, disabled by default | Enabling and tuning, cluster capacity | One process, one port |
| Storage | Volumes only for templates that write files; in-memory templates declare none | Storage class, persistent volumes, backup policy | Local filesystem or in-memory repository |
| Databases and brokers | `external_requirements` and opt-in adapter features | Provision the service only if that adapter is selected | In-memory or embedded default; build and tests need no service |
| Observability | stdout log contract and stable fields from section 5 | Collection, retention, dashboards, optional OTLP endpoint | Human-readable logs on stdout |
| Upgrade | Pinned `rev`, migration notes, plan/apply support | Rollout strategy and rollback decision | Replace the binary and re-run the section 9 steps |

Explicit non-claims: no generated artifact requires a managed database, broker,
cache, object store, secret manager, collector or cloud account to build or
start the template's native path. No manifest contains a secret value. No
Docker or Kubernetes dependency is added to a generated Cargo manifest. The
framework generates the shapes; the operator owns the live infrastructure.

## 11. Checklist for adding a new template

1. Need and scope: no gallery entry already covers the use case; category is
   one of the six allowed values; the change belongs to the current phase and
   needs no RFC (`CLAUDE.md` rules 5 and 22).
2. Metadata: `templates/<name>/template.toml` with `name`, `description`,
   `category`, `external_requirements`; `name` equals the directory; values are
   honest and match section 7 semantics.
3. Skeleton: section 1 tree, with every omission justified in the README;
   `ag-*` dependencies pinned by `rev`; `{{name}}` and `.tmpl` used correctly.
4. Native default: `cargo check --offline`, `ag dev` and integration tests work
   with no external service; any external counterpart is behind a Cargo feature
   and a named adapter.
5. Configuration: variables declared in `ag.toml`, named per section 2, with
   no secret in a clear-text file; resolution follows section 3.
6. Probes: server templates expose `/health` and `/ready` per section 4; the
   `README.md` states the paths and bodies; deployment artifacts use them.
7. Logging: section 5 field names; no secret or connection string in a log.
8. Tests: unit tests plus an HTTP-boundary integration test for both probes;
   `scripts/validate-templates.sh` passes, including the scaffold and offline
   build; evidence is recorded in the notes, not invented.
9. Deployment: artifacts for the family follow section 10 or explicitly defer
   to a family default; no secret in a manifest; non-root by default.
10. Documentation: per-template `README.md` with run commands, declared
    variables, external requirements and limitations; gallery row and index
    link; docs/INDEX.md and CHANGELOG updated in the same change.
11. Governance: descriptor in `docs/pr-drafts/` with a closing-keyword
    section; subject at most 256 characters; no emoji and no tool attribution
    anywhere in the change.
12. Verification: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
    `cargo audit`, `cargo deny` as applicable; documentation links resolve;
    the compatibility matrix and gallery vocabulary agree with the manifest.

## 12. Known conflicts and follow-ups

This list records honest drift found while writing this document. Nothing here
is fixed by this document; each item needs its own change by its owner.

| Conflict | Evidence | Follow-up owner |
| --- | --- | --- |
| `config.toml.tmpl` is documented but not parsed by generated `main.rs`; `BIND` and `RUST_LOG` are read directly. | Default template sources on `main`; wave-6 web templates. | Template owners, with the `RFC-0020` file-source stage |
| `/health` exists only in some templates and no template has `/ready`. | `fullstack`, `layered`, `dashboard`, `admin-panel` vs `rest`, `realtime`, `static-site`. | Template owners; docker/k8s use TCP probes until then |
| `[env]` declarations exist in RFC-0020 but not yet in template `ag.toml` files; the `ag env` commands are future stages. | `RFC-0020` section 13 implementation status. | `ag-core`/`ag-cli` stages, then templates |
| The scaffolded skeleton (section 1) is richer than the alpha templates. | `templates/README.md` and template trees. | Per-template migration PRs |
| `PORT` in the RFC-0020 example and `BIND` in templates name the same concept differently. | `RFC-0020` 4.1 vs template `main.rs.tmpl`. | Keep `BIND`; correct the example in its own doc change |
| Wave-6 catalog registry is not merged in this worktree; several `template.toml` names are documented as `confirm` in the gallery. | `community6/gallery` and `community6/web` notes and branches. | Catalog role at integration time |
| `ag new --with` (creation-time composition) is not implemented. | `RFC-0018` stage B partial; alpha README. | CLI role in a future stage |

## References

- `docs/rfc/RFC-0019-project-layered-architecture.md`
- `docs/rfc/RFC-0020-environment-and-secrets.md`
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
- `docs/adr/0009-gobernanza-correctiva.md`, `docs/adr/0011-*.md`, `docs/adr/0008-*.md`
- `templates/README.md`, `docs/templates/GALLERY.md` (wave-6 branch),
  `docs/templates/README.md` (wave-6 branch)
- `scripts/validate-templates.sh` (wave-6 QA branch)
- `CLAUDE.md` rules 12, 14, 15, 16, 17, 18, 19, 20, 31, 36
