# Fase 4.8 - Fundaciones del proyecto: capas, entorno, instalacion y operacion remota

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-7-experiencia-interactiva-de-terminal.md](./fase-04-7-experiencia-interactiva-de-terminal.md)
> Siguiente: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)

## Phase 4.8 — Project foundations: layers, environment, installation and remote operation

**Status: accepted 2026-11-10 (`RFC-0019`, `RFC-0020`, `RFC-0021`); not
started.** Additive phase. The maintainability rules that accompany it are in
`RFC-0022` (accepted; governance, not a phase).

**Objective.** Give every generated project a defined, enforced structure;
make configuration and secrets declared, validated and safe by default; and
make the tool installable on a server with one verified command and operable
from any other device through the terminal interface of Phase 4.7, by people
and by automated agents alike. Deployment (Phase 5) builds on all three.

**Scope statement fixed by this phase (`RFC-0019`).** Anti-Gravital is a
backend tool. It integrates with Docker, Kubernetes and Terraform and does not
replace them; it offers gRPC as an optional transport and generated clients
for Go, WebAssembly and Flutter consumers; it does not render user interfaces.

**Duration:** 2–3 months.

### 4.8.1 Entry criteria

- [ ] `RFC-0019`, `RFC-0020` and `RFC-0021` accepted with their open questions
  resolved (default layout, fate of `ag-ui`, gRPC and Go in scope, default
  secret storage, remote transport, scope of `ag setup`, release key custody).
- [ ] Phase 4.7 stage A complete (the interactive foundation these flows use).

### 4.8.2 Deliverables

Stage A — project structure (`RFC-0019`):

- [ ] `ag.toml` project manifest read by every command.
- [ ] Layered workspace layout (domain, application, infrastructure,
  interface, composition root) for the `fullstack` and `realtime` templates,
  with the dependency rule enforced by Cargo.
- [ ] Compact layout with the same four layer names for the `rest` template.
- [ ] Infrastructure layer divided by concern, with the database adapter as
  its own bounded module that owns the migrations.
- [ ] `ag generate` writes each artifact into the layer it belongs to;
  `ag generate --check` for CI.
- [ ] Architecture chapter on project structure and updated first-API manual.

Stage B — environment and secrets (`RFC-0020`):

- [ ] `[env]` declaration in `ag.toml`; typed loader with fail-fast validation
  that reports every problem at once.
- [ ] `Secret<T>` type: redacted formatting, zeroed on drop, explicit access.
- [ ] Encrypted per-environment secrets file with recipient management.
- [ ] `ag env list/check/set/unset/recipients/keygen/export`.
- [ ] Modules receive typed configuration instead of reading the process
  environment directly (one crate per pull request).
- [ ] Interactive environment form with masked secret fields.

Stage C — installation (`RFC-0021`):

- [ ] Release workflow that publishes static binaries for the four targets, a
  checksum manifest and its detached signature.
- [ ] Installer that detects system and architecture, verifies signature and
  checksum, and needs no compiler. Closes issue #141.
- [ ] `ag setup --check` and `ag setup` (detect, plan, confirm, apply, verify)
  using the system package manager only.

Stage D — remote console (`RFC-0021`):

- [ ] `ag servers add/list/remove` (inventory without secrets).
- [ ] `ag connect NAME` opening the server-side interface over SSH, and
  `ag connect NAME -- COMMAND` for a single remote command.

Stage E — machine interface (`RFC-0021`):

- [ ] `--output json` on every command, with versioned schemas.
- [ ] Documented exit-code table.
- [ ] `ag describe` emitting the command tree, argument types and output
  schemas.
- [ ] `--plan` / `--apply PLAN_ID` on every command that changes state outside
  the working directory; local audit log of mutating commands.

### 4.8.3 Exit criteria

- [ ] A project created from each template builds and runs in CI from a fresh
  `ag new`, and a domain crate that imports a database or HTTP crate fails to
  compile.
- [ ] Starting an application with three invalid variables reports all three
  and opens no listener; no test can make a secret value appear in logs or
  command output.
- [ ] On a clean server of each supported distribution, the verified installer
  followed by `ag setup` produces a working installation with no manual step,
  and a tampered binary or manifest is rejected.
- [ ] From a second machine, `ag connect` reaches the server interface; the
  same session works from a phone SSH client.
- [ ] An automated client completes a create-configure-validate cycle using
  only `ag describe` and JSON output, and cannot change server state without
  an explicit plan identifier.
- [ ] Stages A and B complete before Phase 5 stage 5-A starts; stages C to E
  complete before the 0.5 release.

### 4.8.4 Phase risks

The main risk is **size**: this phase touches the scaffolder, every module's
configuration and the release process. The mitigation is that the five stages
are independent and each is delivered as several small pull requests; none
requires the others except B on A.

The second risk is **security regressions introduced in the name of
convenience**, especially in installation and remote access. The mitigation is
in the design: signature verification before anything is written, package
manager only, SSH as the only transport with no new daemon or port, and plan
before apply for every state change.

The third risk is **breaking existing users** when generator paths and module
constructors change. The mitigation is that no release has been published;
the changes land before 0.5 and are recorded as pre-release breaking changes.

---

