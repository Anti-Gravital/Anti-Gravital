# RFC-0021 - One-command server installation, remote console and machine interface

- Status: accepted
- Decision date: 2026-11-10
- Author: Angel Nereira (BDFL), Gravital Labs
- Draft date: 2026-11-10
- Target phase: additive Phase 4.8, stages C, D and E
- Affected modules/crates: `ag-cli`, `install.sh`, `install.ps1`, release
  tooling
- Predecessor RFC: RFC-0018 (the interface that is used remotely)
- Comment period: waived by BDFL decision.

## 1. Motivation

The intended way to operate Anti-Gravital is: install it on a server with one
command; from any other device, open a window onto that server and work in the
same guided interface, even though the server has no graphical environment;
and let an automated agent drive the same tool safely.

Three pieces are missing: an installer that does not need a compiler, a way to
reach the server-side interface from elsewhere, and a machine-readable surface
for the commands.

## 2. Problem

1. `install.sh` requires a Rust toolchain and compiles the whole workspace
   from source. On a small server this takes many minutes and can fail for
   lack of memory. It detects nothing and installs no missing prerequisite.
2. The integrity of the installer rests on a checksum stored in the same
   repository as the script (issue #141), so a compromised repository defeats
   it.
3. There is no defined way to operate a server from another machine. Users
   will open an SSH session by hand, with no knowledge on the tool's side of
   which servers exist.
4. Command output is prose for humans. A script or an agent must parse text
   that may change between releases, and cannot preview what a mutating
   command would do.

## 3. Alternatives considered

**Remote access. A. A long-running Anti-Gravital daemon on the server with its
own network protocol, authentication and client.** Full control over the
experience. Rejected: it adds a listening port, an authentication system, a
wire protocol and a key-distribution problem, all of which are security
critical and all of which SSH already solves. It contradicts rule 12 (integrate
with the dominant tool) and rule 16 (minimize attack surface).

**Remote access. B. A web console.** Rejected: needs a browser-facing service
with its own authentication and TLS; same objection, larger surface.

**Remote access. C. Run the server-side `ag` interface over SSH (chosen).**
The TUI of `RFC-0018` already runs on the server and renders in any terminal.
SSH provides transport, authentication, encryption and host verification.
Every platform, including phones and tablets, has an SSH client.

**Installer. A. Keep building from source.** Rejected for servers (problem 1);
kept as the documented path for contributors.

**Installer. B. Distribution packages only.** Good for upgrades, slow to
bootstrap across distributions. Deferred; not excluded.

**Installer. C. Prebuilt static binaries with signature verification
(chosen).**

## 4. Proposed design

### 4.1 Stage C - installation

**Release artifacts.** Each release publishes one statically linked `ag`
binary per target (Linux x86-64 and ARM64 with musl, macOS ARM64, Windows
x64), a checksum manifest, and a detached signature of the manifest.

**Trust anchor.** The manifest is signed with a release key whose public half
is published outside the repository as well as embedded in the installer. This
closes issue #141: an attacker who can modify the repository cannot produce a
valid signature.

**The installer** (`install.sh`, `install.ps1`) does only this:

1. Detects operating system, CPU architecture and C library.
2. Downloads the matching binary, the manifest and the signature.
3. Verifies the signature of the manifest and the checksum of the binary.
   Any failure aborts before anything is written outside a temporary directory.
4. Installs the binary to a user-writable location by default, or system-wide
   when asked, and prints the installed version.

It installs nothing else and needs no toolchain. `ADR-0009` rule 4 stays in
force: the documented procedure downloads the installer, verifies it, then
runs it. A single copy-paste line that performs those steps in order is
provided; piping an unverified script into a shell is not.

**`ag setup`** is the step that prepares a machine, run after installation:

1. **Detect.** Operating system and version, architecture, package manager,
   and the presence and version of each prerequisite the selected role needs
   (for example a container runtime for deployment, or a PostgreSQL client).
2. **Plan.** Print the exact actions it would take: packages to install, with
   the exact package-manager commands; files to write; services to enable.
3. **Confirm.** Nothing privileged runs before explicit confirmation.
   `--yes` exists for automation and is recorded in the output.
4. **Apply and verify.** Each action is followed by a check; the result is a
   report of what changed.

`ag setup` uses the operating system's own package manager. It never downloads
and executes third-party scripts. A prerequisite it cannot install through the
package manager is reported with instructions instead of being worked around.
`ag setup --check` performs step 1 only and is safe to run anywhere.

Roles: `server` (runs applications), `workstation` (develops and operates).
Both use the same binary.

### 4.2 Stage D - remote console

```
ag servers add prod deploy@203.0.113.10     # stored in the project or user config
ag servers list
ag connect prod                             # opens the server-side interface
ag connect prod -- domains status           # runs one command remotely
```

- `ag connect` invokes the system SSH client with a pseudo-terminal and starts
  `ag` on the server. Key handling, agent forwarding policy, jump hosts and
  host-key verification are those of the user's SSH configuration. `ag` stores
  no credentials.
- The server needs only the `ag` binary and an SSH account. No daemon, no new
  open port, no new authentication mechanism.
- If the remote `ag` is missing or its version is incompatible, `ag connect`
  says so and offers the installation command; it does not install silently.
- From a phone or tablet, any SSH client that connects to the server and runs
  `ag` gets the same interface. This is a consequence of the design, not a
  separate deliverable.
- A dropped connection never leaves a half-applied change: mutating actions
  follow the plan-and-apply model of 4.3, and long operations are designed to
  be resumed or safely re-run.

The server inventory is a plain file. Entries hold a name, a user, a host and
optional SSH options, and no secrets.

### 4.3 Stage E - machine interface

So that scripts, CI and automated agents use the same tool as people:

1. **Structured output.** Every command accepts `--output json`. The JSON
   shape of each command is versioned and documented; prose output may change,
   JSON output follows SemVer.
2. **Stable exit codes.** A documented table: success, usage error,
   validation failure, precondition not met, remote failure.
3. **Self-description.** `ag describe --output json` emits the full command
   tree with arguments, types, defaults, whether a command mutates state, and
   the schema of its output. An agent discovers the tool from the tool.
4. **Plan and apply.** Every command that changes something outside the
   working directory supports `--plan`, which outputs the intended changes and
   a plan identifier, and `--apply PLAN_ID`, which executes exactly that plan
   and fails if the state has changed since. Review, by a person or by policy,
   happens between the two.
5. **No privilege by being a machine.** The machine interface exposes exactly
   the commands and checks a person has. Confirmation is replaced by the
   explicit plan identifier, never skipped.
6. **Audit trail.** Mutating commands append a record (time, user, command,
   plan identifier, result) to a local log on the machine where they run.

A protocol server mode for agent frameworks can be layered on `ag describe`
and the JSON output later; it is out of scope here and needs its own RFC.

### 4.4 Testing

Installer: verification failure paths (bad signature, bad checksum, truncated
download) tested with fixtures; `tools/check-installers.sh` extended. `ag
setup`: detection tested against recorded fixtures per distribution; the plan
is asserted, privileged application is tested in containers in CI. Remote
console: tested against a local SSH server in CI. Machine interface: schema
snapshot tests per command; a test that every mutating command rejects
execution without `--apply` or interactive confirmation.

## 5. Implementation plan (only after approval)

1. PR 1 - release workflow producing static binaries, manifest and signature
   (CI change; covered by this RFC).
2. PR 2 - installer rewrite with verification; closes issue #141.
3. PR 3 - `ag setup --check`, then `ag setup` for the first distribution
   family, then the others one per pull request.
4. PR 4 - `--output json`, exit-code table and `ag describe`.
5. PR 5 - plan and apply for the existing mutating commands
   (`ag domains sync/attach/detach`, `ag workers dlq`).
6. PR 6 - `ag servers` and `ag connect`.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Release signing key compromised or lost | Low | Critical | Key kept offline; documented rotation with a published revocation; public key distributed out of band. |
| `ag setup` breaks a server by installing or changing packages | Medium | High | Plan before apply; package manager only; never removes or replaces existing packages; `--check` mode. |
| Variation across Linux distributions | High | Medium | Supported list stated explicitly and tested in containers; unsupported systems get the plan as instructions. |
| Remote console assumed to be a security boundary | Medium | High | Documentation states that authorization is the SSH account's; a restricted account with `ag` as its forced command is the documented hardening. |
| Structured output freezes too early | Medium | Medium | Marked unstable until 0.5; versioned from 0.5. |

## 7. Impact

- **Scope:** new commands `ag setup`, `ag servers`, `ag connect`, `ag
  describe`; global `--output`, `--plan`, `--apply`.
- **Roadmap:** Phase 4.8 stages C-E. The release workflow is a prerequisite of
  the 0.5 release in any case.
- **Operational complexity:** none added on the server (no daemon).
- **Documentation:** installation guide rewritten;
  `docs/security/INSTALLATION_INTEGRITY.md` updated; new operations chapter.

## 8. Rollback

Each stage is independent. The source-build installer stays in the repository
as the contributor path and can be restored as the default. `ag connect` is a
convenience over SSH and can be removed without affecting servers.

## 9. Decision

- Decider: BDFL (Angel Nereira).
- Decision date: 2026-11-10.
- Result: accepted. The comment period was waived by BDFL decision.

Resolution of the open questions:

1. **Remote transport.** SSH. No Anti-Gravital daemon, port or authentication
   mechanism is introduced.
2. **Scope of `ag setup`.** Not decided. To be fixed before stage C starts:
   which prerequisites it may install and the first supported distributions.
3. **Release key custody.** Not decided. To be fixed before the release
   workflow of stage C is implemented.

## 10. References

- `CLAUDE.md` rules 12, 13, 16, 36, 37; `ADR-0009` rule 4.
- Issue #141 (installer trust anchor).
- `RFC-0018`, `RFC-0019`, `RFC-0020`.
- `docs/security/INSTALLATION_INTEGRITY.md`.
