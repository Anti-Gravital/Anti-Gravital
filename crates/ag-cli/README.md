# ag-cli

> Status: Phases 2-4.7 — implemented. The `ag` binary exposes project, DSL,
> transactional-mail, DNS, local domain-attachment, worker, and interactive
> configuration workflows documented below (the `workers` operational group
> `run`/`enqueue`/`queues`/`dlq`/`doctor` sits behind the default
> `workers-runtime` feature). `deploy`, `migrate`, and `plugin` are not
> available; they require their future roadmap phases and approved RFCs.
> Criticality: Core.
> Architecture chapter: docs/architecture/05-ecosistema-modulos.md

## Subcommands

### `ag new [name] [--template NAME] [--with MODULE]...`

Creates a new project. `--with` is repeatable and enables a declared module
(`data`, `auth`, `cache`, `mail`, `workers`, `storage`, `realtime`, `observe`)
in the `[modules]` table of the generated `ag.toml` (`RFC-0019` stage A). It
requires a template that produces a manifest (for example `--template layered`);
otherwise it exits with a clear error before creating anything. The default
scaffold is unchanged and the printed equivalent command includes the flags.

With every input on the command line it runs without asking anything:

```bash
ag new my-api --template realtime
```

With an input missing, in an interactive terminal, a guided full-screen wizard
opens (RFC-0018, stage A). It asks for the name with live validation, lets you
pick a template while showing the files each one creates, and ends on a review
screen that lists exactly what will be written. Nothing is written before you
confirm there.

```bash
ag new            # wizard asks for name and template
ag new my-api     # wizard starts on the template step
```

Keys: `Enter` continues, `Esc` goes back (or cancels on the first step),
`Up`/`Down` or `j`/`k` select, `q` or `Ctrl-C` cancels. After the project is
created, `ag` prints the equivalent non-interactive command so the action can
be repeated or scripted.

The wizard is never used by surprise. `ag` does not prompt when any of these
holds: standard input or output is not a terminal, `CI` is set, `AG_NO_TUI` is
set, `TERM` is `dumb`, or `--no-interactive` is passed. In that case:

| Input | Result |
| --- | --- |
| Name and template given | Project created |
| Name given, template omitted | Template defaults to `rest` |
| Name omitted | Usage error, exit code 2, nothing written |

Other behavior:

- `NO_COLOR` selects a monochrome theme. Selection and errors are also marked
  with text, never by color alone.
- In a terminal smaller than 80x24 the same questions are asked as plain line
  prompts.
- The terminal is restored on exit, on error, on `Ctrl-C` and on panic.
- The interface is the `tui` Cargo feature, on by default. A build with
  `--no-default-features` has no interactive code and behaves as a
  non-interactive session.

### Global flag `--no-interactive`

Accepted before or after the subcommand. Forces the non-interactive path.

### `ag config` (RFC-0018, stage B)

Without a subcommand, in an interactive terminal, opens a full-screen form
for `config.toml`. Fields are grouped by section (bind address, runtime, CORS,
CSRF, rate limiting, authentication and TLS), show their current value and a
one-line description, and validate inline while you type. `r` opens a review
step showing every change and its equivalent command before anything is
written. Keys: `Up`/`Down` or `j`/`k` select, `Enter` edits, `Esc` cancels an
edit, `q` aborts.

Non-interactive parity, usable in scripts and CI:

```bash
ag config get bind                    # prints the value (or its default)
ag config set rate_limit.enabled true # validates before writing
ag config validate                    # exits non-zero on an invalid file
```

Keys are dotted TOML paths such as `bind`, `cors.allow_origins` or
`rate_limit.per_ip_rps`. Secret material (an inline JWT public key PEM) is not
exposed by the form or the flags; only file paths and claim names are editable.
Every write is validated with `ag_core::ShieldConfig`, the same loader the
runtime uses, before the file is replaced atomically. Unrelated keys and
comments are preserved. A legacy `[bind] host/port` table from older templates
is normalized in memory so scaffolded projects validate; it is rewritten to
the canonical `bind = "host:port"` form on the next `set`.

### `ag env list|check` (RFC-0020, stage B)

Read-only views over the variables declared in the `[env.*]` tables of
`ag.toml`, resolved through `ag_core::EnvLoader`:

```bash
ag env list [--manifest ag.toml] [--env NAME]
ag env check [--manifest ag.toml] [--env NAME]
```

`list` prints each declaration with its type, required/secret flags, source and
presence. `check` resolves the environment exactly as startup would and exits 1
with every problem at once on failure, so it is usable in CI. Neither command
writes a file, reads an encrypted secrets file, or prints a value, including a
non-secret default. `--env NAME` is a label only in stage B: environment-specific
sources and the `set`/`unset`/`keygen`/`recipients`/`export` commands are later
stages. Both commands support the global `--output json` flag with the versioned
`env.list` / `env.check` envelopes of the `0.2.0` contract.

### `ag describe` (RFC-0024, section 4.2)

```bash
ag describe
ag --output json describe
```

Read-only description of the current directory as a project: recognized layout
markers, artifact destinations, authored source paths, sizes and fingerprints,
and whether generated output is in sync with the sources. It writes no file and
makes no network call. All paths are relative; no secret, environment value or
absolute path is emitted. Prose and JSON share one builder, so they cannot
drift. The JSON shape is `project.describe` in contract `0.2.0`.

### `ag` with no arguments (RFC-0018, stage C)

In an interactive terminal it opens a home screen that lists the commands
available in the current context. Outside a project it offers project
creation, help and version; inside a project (detected by `config.toml` and a
`Cargo.toml` depending on `ag-core`) it offers `ag dev`, `ag generate`,
`ag schema lint`, `ag config` and the read-only status commands. Keys:
`Up`/`Down` or `j`/`k` select, `Enter` launches, `q` or `Esc` closes. The
screen performs no work: it returns a command line that `main` re-parses
through the real CLI and prints before running. Without a terminal it prints
help, exactly as in earlier releases.

### `ag dev [--bind host:port]`

Starts the server in development mode with hot reload via `cargo-watch`.
Falls back to `cargo run` if `cargo-watch` is not installed.

```bash
ag dev                    # listens on 127.0.0.1:8080 (loopback only)
ag dev --bind 0.0.0.0:8080 # explicit opt-in: all interfaces (containers, LAN)
ag dev --bind 127.0.0.1:3000
```

Generated project templates still bind `0.0.0.0:8080` because they are built
for containers, where the process must accept traffic from outside its own
network namespace. That is a deliberate, documented difference: `ag dev`
protects the laptop by default, while a generated service is ready to run in a
container. Change the listener in `src/main.rs` if you run a generated project
directly on an untrusted network.

### `ag build [--target triple]`

Compiles the project in release mode. Optionally cross-compiles.

```bash
ag build
ag build --target x86_64-unknown-linux-musl
```

### `ag generate [--schema schema.ag] [--output ./generated]`

Reads the DSL schema and writes generated Rust, SQL, TypeScript, and OpenAPI
artifacts to the output directory.

```bash
ag generate
ag generate --schema custom.ag --output out/
```

### `ag schema lint [--schema schema.ag]`

Lints the DSL schema and reports warnings and errors.

### `ag schema diff <reference> [--schema schema.ag]`

Compares the current schema against a reference file and reports changes.

### `ag workers list [--schema schema.ag]`

Lists background workers declared in the DSL schema. This command reads local
configuration only and does not require queue infrastructure.

### `ag domains check --domain example.com [--expected value] [--min-confirmed N]`

Verifies DNS propagation for a domain. Exits non-zero if fewer than
`--min-confirmed` resolvers (default 2) have the expected value.

### `ag domains sync --schema schema.ag --zone-id ZONE [--token TOKEN]`

Applies SPF/DKIM/DMARC records to the DNS provider idempotently.
Reads zone ID and token from flags or environment variables.

### Local domain loop (RFC-0033)

The `[domain]` table of `ag.toml` is the canonical binding for a project in a
given environment. `plan` is read-only and deterministic; `apply` is gated by
the exact plan id and writes only `.ag/dns/state.json` unless a provider is
explicitly requested:

```bash
ag dns plan
ag dns plan --env production
ag dns apply --plan dns-production-0123456789abcdef
ag dns apply --plan dns-production-0123456789abcdef --provider cloudflare --zone-id ZONE
ag dns status
```

Without `--provider` there is no network access. Tokens and provider zone ids
are read from the environment and never from the manifest or generated files.
`status` never contacts a provider. A private binding emits no public
A/AAAA/CNAME record; a wildcard requires both `allow_wildcard = true` and
`tls = "managed_dns01"`. `ag domains attach --manifest ag.toml` derives the
host, project and environment from the same binding.

### Local domain attachment workflow

These commands use the native JSON store at `.ag/domains.json` by default and
do not require provider credentials:

```bash
ag domains attach example.com --project my-api --edge-host edge.example.net
ag domains instructions example.com --edge-host edge.example.net
ag domains export-zone example.com --edge-host edge.example.net
ag domains status example.com
ag domains list
ag domains verify example.com
ag domains detach example.com
ag domains diagnose example.com --edge-host edge.example.net
```

Use `--state PATH` to select another store. Apex domains also require one or
more `--ip ADDRESS` values for commands that generate or diagnose records.
See the [domain CLI reference](../../docs/ag-domains/reference/cli.md) for the
full lifecycle and flags.

### `ag workers run|enqueue|queues|dlq|doctor` (feature `workers-runtime`)

Operational surface over the durable `ag-workers` backend (RFC-0012 §27),
beyond the always-available `ag workers list` documented above:

- `run` — run a standalone worker process against the configured backend.
- `enqueue KIND --payload FILE` — enqueue a job.
- `queues` — show queue depths.
- `dlq list|inspect|retry|purge` — inspect and manage the dead-letter queue.
- `doctor` — check workers configuration and backend connectivity.

These subcommands are compiled behind the `workers-runtime` Cargo feature of
`ag-cli` (enabled by default; disable with `--no-default-features` for a
lighter binary), and the ones that touch the durable backend require
`DATABASE_URL`.

### `ag mail test --to dest@example.com [options]`

Sends a test email to verify the SMTP configuration.

```bash
ag mail test --to me@example.com
ag mail test --to me@example.com --smtp-host mail.example.com --smtp-port 587
```

## Environment variables

| Variable              | Used by                  | Default       | Description                        |
|-----------------------|--------------------------|---------------|------------------------------------|
| `AG_CLOUDFLARE_TOKEN` | `ag domains sync`        | —             | Cloudflare API token               |
| `AG_DNS_ZONE_ID`      | `ag domains sync`, `ag dns apply --provider cloudflare` | — | Cloudflare zone ID |
| `AG_EDGE_HOST`        | Domain attach/instructions/export/diagnose | — | Edge CNAME target |
| `AG_SMTP_HOST`        | `ag mail test`           | `localhost`   | SMTP host                          |
| `AG_SMTP_PORT`        | `ag mail test`           | `587`         | SMTP port                          |
| `AG_SMTP_USER`        | `ag mail test`           | —             | SMTP username (optional)           |
| `AG_SMTP_PASS`        | `ag mail test`           | —             | SMTP password (optional)           |
| `AG_MAIL_FROM`        | `ag mail test`           | `test@localhost` | Sender address                  |
| `DATABASE_URL`        | `ag workers` runtime cmds | —            | PostgreSQL URL for the durable backend |

`ag dev --bind ADDRESS` passes `BIND` to the child application. Users select
the address with the flag; `BIND` is not read as CLI configuration.

## Machine-readable output

`ag --output json` (or `--output json` after a read subcommand) emits one
versioned JSON line for `ag config get`, `ag schema lint`, `ag schema diff`,
`ag workers list`, `ag deploy plan`, `ag deploy status`, `ag dns plan`,
`ag dns status`, `ag env list`, `ag env check` and `ag describe`. The flag is refused with a versioned
`unsupported_command` error before any mutating command does work. Prose output
is unchanged when the flag is absent. The contract, its JSON Schema and
byte-exact snapshots live in
[`docs/mcp/`](../../docs/mcp/); contract version `0.2.0` is unstable until
Anti-Gravital 0.5. For `ag generate`, whose directory flag owns `--output`,
the same selector is exposed as `--output-format`.

## Command availability

`ag deploy`, `ag migrate`, and `ag plugin` are intentionally unavailable.
Their roadmap phases and RFCs must be approved and implemented before those
commands can be added. The CLI does not expose placeholder commands for future
work.

## Installation

```bash
# From source (requires Rust >= 1.99):
bash install.sh          # Linux / macOS
.\install.ps1            # Windows PowerShell
```

Or directly:

```bash
cargo install --path crates/ag-cli --locked
```

## References

- Master architecture: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
- Navigation chapter: `docs/architecture/05-ecosistema-modulos.md`
- Module sheet: `docs/modules/ag-cli/README.md`
- Onboarding guide: `docs/manual/04-instalacion-y-onboarding.md`
- Governance: `CLAUDE.md`
