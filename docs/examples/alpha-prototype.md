# Build an offline prototype with the alpha

This guide takes you from zero to a running HTTP API and a generated SDK using
`ag`. It is written for the `0.1.0-alpha.1` toolchain and needs **no external
infrastructure**: no PostgreSQL, NATS, Redis or SMTP server is started, and no
API key is required.

Scope: this is a prototype guide, not a production deployment manual. The
generated handler bodies are deliberate stubs, and the alpha has not passed the
pre-Phase-5 release gate.

## Prerequisites

- Rust 1.99.1 or newer and Cargo.
- Git.
- `curl` (only to probe the running API).
- The `ag` binary installed (`bash install.sh`, `.\install.ps1`, or
  `cargo install --path crates/ag-cli --locked` from a checkout).

The first build of a scaffolded project resolves its pinned `ag-core` git
dependency, so network access (or a warm Cargo cache) is needed once. After the
crates are cached, the whole flow works offline. No runtime service is ever
contacted.

## Part 1 - Scaffold and run a local API

Create a REST project non-interactively. `--no-interactive` guarantees that no
wizard opens; it is equivalent to running in CI or with a closed stdin.

```bash
ag new demo --template rest --no-interactive
cd demo
```

Expected output from `ag new`:

```text
Creating project 'demo' with template 'rest'...

Project 'demo' created.

Next steps:
  cd demo
  ag dev
```

Three files are written: `Cargo.toml`, `config.toml` and `src/main.rs`. There is
no `migrations/` directory and the manifest pulls only `ag-core`, `axum`,
`serde`, `serde_json`, `tokio` and `tracing`. No database, queue, cache or mail
crate is involved.

Run the server directly:

```bash
cargo run
```

In a second terminal, probe it:

```bash
curl -i http://127.0.0.1:8080/
```

Expected response:

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 32

demo corriendo con Anti-Gravital
```

The server listens on `127.0.0.1:8080` by default (loopback only). The listener
reads the `BIND` environment variable, so you can change the address without
editing code:

```bash
BIND=127.0.0.1:9090 cargo run
```

`ag dev` sets `BIND` for you from its `--bind` flag (default
`127.0.0.1:8080`) and uses `cargo-watch` for hot reload when it is installed;
otherwise it falls back to plain `cargo run`:

```bash
ag dev                              # loopback, default port
ag dev --bind 127.0.0.1:9090        # explicit loopback address
ag dev --bind 0.0.0.0:8080          # opt-in: expose on all interfaces
```

The generated `config.toml` contains a `[bind]` section whose `host` defaults
to `127.0.0.1` as well, so nothing in the scaffold suggests all interfaces. In
the alpha that section is still informational: the generated program does not
read it yet, and the actual address is controlled by `BIND` (or the loopback
default). This is documented here so the configuration file is not mistaken for
a live control surface.

## Part 2 - Generate an SDK from a schema

The `.ag` schema is the source of truth. Use the shipped example schema or your
own:

```bash
# From the repository root, using the bundled schema:
ag generate \
  --schema examples/ecommerce-api/schema.ag \
  --output generated
```

Expected summary:

```text
Generating 10 artifacts from 'examples/ecommerce-api/schema.ag'...
  generated/clients/typescript/client.ts
  generated/clients/typescript/types.ts
  generated/migrations/0001_initial.sql
  generated/openapi.json
  generated/openapi.yaml
  generated/src/handlers.rs
  generated/src/mod.rs
  generated/src/models.rs
  generated/src/router.rs
  generated/src/types.rs
Generation completed in 'generated'.
```

The TypeScript client (`generated/clients/typescript/client.ts`) exposes one
`fetch`-based function per endpoint, for example:

```typescript
export async function listUsers() : Promise<UserResponse> {
  const resp = await fetch(`${BASE_URL}/users`, { method: 'get' });
  if (!resp.ok) throw new Error(`listUsers failed: ${resp.status}`);
  return resp.json();
}
```

`BASE_URL` is emitted empty; set it to your API origin before using the client.
Generated handler bodies are stubs that you implement in your application.

To validate a schema before generating, lint it first:

```bash
ag schema lint --schema examples/ecommerce-api/schema.ag
```

## What needs external infrastructure, and what does not

The following was checked by building and running each target with no external
service configured. "External" means a process or credential outside the local
machine.

| Target | Runs offline? | What is required for the full path |
| --- | --- | --- |
| `ag new --template rest` | yes | nothing (git dep fetch on first build only) |
| `ag new --template realtime` | yes | nothing; WebSocket echo on `:8080/ws` |
| `ag new --template layered` | yes | nothing; health route on `:8080/health` |
| `ag new --template fullstack` | no | PostgreSQL via `DATABASE_URL` (runs migrations) |
| `examples/workers-basic` | yes | nothing (in-memory queue) |
| `examples/workers-scheduled` | yes | nothing (in-memory scheduler) |
| `examples/workers-producer-edge` | yes | nothing (in-memory backend) |
| `examples/workers-mail-integration` | yes | nothing (no-op mail sender) |
| `examples/workers-postgres` | exits cleanly without `DATABASE_URL` | PostgreSQL for the durable path |
| `examples/auth-mail-demo` | yes | nothing (`NullSender` captures mail) |
| `examples/realtime-chat` | yes | nothing; HTTP+SSE on `:3000` |
| `examples/ai-backend` | partially | server and `/health`, `/providers` work offline; `/chat` needs an AI API key |
| `examples/todo-api` | no | PostgreSQL via `DATABASE_URL` |
| `examples/ecommerce-api` | yes | DSL schema only; no runtime service |

Quick check of the offline examples (all exit 0 with no service configured):

```bash
cargo run -p workers-basic
cargo run -p workers-scheduled
cargo run -p workers-producer-edge
cargo run -p workers-mail-integration
cargo run -p auth-mail-demo
env -u DATABASE_URL cargo run -p workers-postgres
```

`realtime-chat` and `ai-backend` are long-running servers; press Ctrl-C to stop
them. `ai-backend` logs a warning when no provider key is set and its `/chat`
endpoint is not usable offline.

## Known limitations of the alpha prototype path

- First build fetches the pinned `ag-core` git revision from GitHub. This is a
  build-time dependency, not a runtime service; use a warm cache or a network
  connection once.
- Generated handlers are stubs; the scaffold gives you transport and middleware,
  not business logic.
- `config.toml` is not yet wired to the generated runtime; use `BIND`.
- `cargo-watch` is optional. Without it `ag dev` runs `cargo run` without hot
  reload.
- No database, auth, cache or mail module is enabled in the `rest` template.
  Add them deliberately; do not assume they are present.
- `ag migrate`, `ag deploy`, `ag plugin` and the MCP interface are not available
  in the alpha.

## Verification evidence

Recorded on Linux x86_64, rustc/cargo 1.99.1, from `community5/examples`:

- `ag new demo --template rest --no-interactive` wrote three files, exit 0.
- `cargo build` of the scaffold completed, then the binary served
  `GET / -> 200 "demo corriendo con Anti-Gravital"`; `ss -ltnp` showed
  `127.0.0.1:8080` (loopback only).
- `ag dev --bind 127.0.0.1:8080` produced the same response and bind.
- `ag generate --schema examples/ecommerce-api/schema.ag --output generated`
  emitted 10 artifacts including the TypeScript SDK, exit 0.
- All offline examples listed above were built and executed with no external
  service; `workers-postgres` printed its `DATABASE_URL` hint and exited 0.
- `cargo test -p ag-cli` passed: 100 + 4 + 12 + 3 + 11 + 6 tests, 0 failed.
