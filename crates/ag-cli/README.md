# ag-cli

> Status: Phases 2-4.5 — implemented. The `ag` binary exposes project, DSL,
> transactional-mail, DNS, local domain-attachment, and worker workflows documented
> below. `deploy`, `migrate`, and `plugin` are not available; they require their
> future roadmap phases and approved RFCs.
> Criticidad: Nucleo.
> Architecture chapter: docs/architecture/05-ecosistema-modulos.md

## Subcommands

### `ag new <name> [--template rest|realtime|fullstack]`

Creates a new project. When `--template` is omitted and the terminal is
interactive, you are prompted to choose. In CI / scripts the default is `rest`.

```bash
ag new my-api                        # interactive prompt
ag new my-api --template realtime    # non-interactive
```

### `ag dev [--bind host:port]`

Starts the server in development mode with hot reload via `cargo-watch`.
Falls back to `cargo run` if `cargo-watch` is not installed.

```bash
ag dev                    # listens on 0.0.0.0:8080
ag dev --bind 127.0.0.1:3000
```

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
| `AG_DNS_ZONE_ID`      | `ag domains sync`        | —             | Cloudflare zone ID                 |
| `AG_EDGE_HOST`        | Domain attach/instructions/export/diagnose | — | Edge CNAME target |
| `AG_SMTP_HOST`        | `ag mail test`           | `localhost`   | SMTP host                          |
| `AG_SMTP_PORT`        | `ag mail test`           | `587`         | SMTP port                          |
| `AG_SMTP_USER`        | `ag mail test`           | —             | SMTP username (optional)           |
| `AG_SMTP_PASS`        | `ag mail test`           | —             | SMTP password (optional)           |
| `AG_MAIL_FROM`        | `ag mail test`           | `test@localhost` | Sender address                  |

`ag dev --bind ADDRESS` passes `BIND` to the child application. Users select
the address with the flag; `BIND` is not read as CLI configuration.

## Command availability

`ag deploy`, `ag migrate`, and `ag plugin` are intentionally unavailable.
Their roadmap phases and RFCs must be approved and implemented before those
commands can be added. The CLI does not expose placeholder commands for future
work.

## Installation

```bash
# From source (requires Rust >= 1.95):
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
