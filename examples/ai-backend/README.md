# ai-backend

Multi-provider AI SSE streaming server. It detects the API keys available in the
environment and exposes only the configured providers. Adding a new provider =
implementing the `AiProvider` trait.

## Running

```bash
# Claude (Anthropic)
ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend

# Gemini (Google)
GEMINI_API_KEY=AIza... cargo run -p ai-backend

# OpenAI
OPENAI_API_KEY=sk-... cargo run -p ai-backend

# Ollama (OpenAI-compatible local, default model llama3)
OPENAI_API_KEY=ollama \
OPENAI_BASE_URL=http://localhost:11434 \
cargo run -p ai-backend

# Multiple providers at once
ANTHROPIC_API_KEY=... GEMINI_API_KEY=... cargo run -p ai-backend
```

## Environment variables

| Variable              | Default                    | Description                       |
|-----------------------|----------------------------|-----------------------------------|
| `ANTHROPIC_API_KEY`   | —                          | Enables the claude provider       |
| `GEMINI_API_KEY`      | —                          | Enables the gemini provider       |
| `OPENAI_API_KEY`      | —                          | Enables the openai provider       |
| `OPENAI_BASE_URL`     | `https://api.openai.com`   | OpenAI-compatible endpoint        |
| `AI_DEFAULT_PROVIDER` | first registered           | Default provider                  |
| `PORT`                | `3001`                     | Server port                       |
| `LOG_FORMAT`          | `pretty`                   | `pretty` or `json`                |

## API

| Method | Path         | Description                          |
|--------|--------------|--------------------------------------|
| POST   | `/chat`      | SSE token stream                     |
| GET    | `/providers` | List available providers             |
| GET    | `/health`    | Health check                         |

### List available providers

```bash
curl http://localhost:3001/providers
```

### Chat with the default provider

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explain Rust in three sentences"}'
```

### Chat specifying provider and model

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "hello", "provider": "gemini", "model": "gemini-1.5-pro"}'
```

## Adding a new provider

1. Create `src/provider/my_provider.rs` implementing the `AiProvider` trait.
2. Register it in `ProviderRegistry::from_env()` based on its environment key.
3. Re-export it in `provider/mod.rs`.

## Crates demonstrated

- `ag-observe`: logging and tracing
