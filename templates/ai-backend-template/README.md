# ai-backend-template

Provider-agnostic chat API built on `ag-core` (Shield + axum). The default
provider is a local echo provider, so the scaffold runs fully offline with no
API key and no network access.

## What it provides

- `ChatProvider`, a transport-independent trait for model providers.
- `EchoProvider`, a local default that echoes the most recent user message.
- `POST /chat` for conversations, `GET /providers` for the compiled-in list and
  `GET /health` for readiness.
- A single composition root whose provider is selected by an `Arc<dyn
  ChatProvider>`, so a remote provider can be added without touching handlers.

## Run

```sh
ag new my-chat --template ai-backend-template --no-interactive
cd my-chat
cargo run
```

The server listens on `127.0.0.1:8080` by default. `ag dev --bind` sets `BIND`
and overrides the address at runtime.

```sh
curl -s http://127.0.0.1:8080/health
curl -s http://127.0.0.1:8080/providers
curl -s -X POST http://127.0.0.1:8080/chat \
  -H 'content-type: application/json' \
  -d '{"messages":[{"role":"user","content":"hello"}]}'
# -> {"provider":"echo","reply":"echo: hello"}
```

An empty `messages` array returns 422 with a JSON error body.

## Configuration

`config.toml` documents the bind address and the provider kind. It is
informational: `BIND` is authoritative at runtime. The `[provider]` section is
a starting point for a future provider-selection layer; the current scaffold
always uses `EchoProvider`.

## External requirements

- None for the default build and runtime.
- A remote model provider (a future `ChatProvider` implementation) would read
  credentials from the environment, for example `AG_MODEL_API_KEY`. This value
  is declared in `template.toml` metadata and is never hardcoded in source or
  config files.

## Integration with platform crates

`ag-ai` is linked as the reserved home for AI capabilities. At the pinned
revision it is a Phase 0 skeleton with no public API, so the chat provider
logic is implemented locally in this scaffold. When `ag-ai` exposes a provider
client, move the remote implementation behind that boundary; the
`ChatProvider` trait keeps handlers unchanged.

## Limitations

- `EchoProvider` is a deterministic stand-in, not a language model. It does
  not reason, summarise or call any external service.
- No streaming, tool calling, conversation persistence or rate limiting.
- No authentication on the chat endpoint; add a Shield policy before exposing
  it beyond loopback.
- The first build resolves the pinned git revision; it requires the source to
  be available in the local cargo cache when building offline.

## Status

Template, experimental. Builds and runs offline with `cargo check --offline`.
