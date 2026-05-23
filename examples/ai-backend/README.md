# ai-backend

Servidor de streaming SSE multi-proveedor de IA. Detecta las API keys
disponibles en el entorno y expone solo los proveedores configurados.
Agregar un proveedor nuevo = implementar el trait `AiProvider`.

## Ejecucion

```bash
# Claude (Anthropic)
ANTHROPIC_API_KEY=sk-ant-... cargo run -p ai-backend

# Gemini (Google)
GEMINI_API_KEY=AIza... cargo run -p ai-backend

# OpenAI
OPENAI_API_KEY=sk-... cargo run -p ai-backend

# Ollama (OpenAI-compatible local, modelo por defecto llama3)
OPENAI_API_KEY=ollama \
OPENAI_BASE_URL=http://localhost:11434 \
cargo run -p ai-backend

# Multiples proveedores simultaneos
ANTHROPIC_API_KEY=... GEMINI_API_KEY=... cargo run -p ai-backend
```

## Variables de entorno

| Variable              | Default                    | Descripcion                       |
|-----------------------|----------------------------|-----------------------------------|
| `ANTHROPIC_API_KEY`   | —                          | Habilita proveedor claude         |
| `GEMINI_API_KEY`      | —                          | Habilita proveedor gemini         |
| `OPENAI_API_KEY`      | —                          | Habilita proveedor openai         |
| `OPENAI_BASE_URL`     | `https://api.openai.com`   | Endpoint OpenAI-compatible        |
| `AI_DEFAULT_PROVIDER` | primer registrado          | Proveedor por defecto             |
| `PORT`                | `3001`                     | Puerto del servidor               |
| `LOG_FORMAT`          | `pretty`                   | `pretty` o `json`                 |

## API

| Metodo | Ruta         | Descripcion                          |
|--------|--------------|--------------------------------------|
| POST   | `/chat`      | Stream SSE de tokens                 |
| GET    | `/providers` | Lista proveedores disponibles        |
| GET    | `/health`    | Health check                         |

### Ver proveedores disponibles

```bash
curl http://localhost:3001/providers
```

### Chat con proveedor por defecto

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explica Rust en tres oraciones"}'
```

### Chat especificando proveedor y modelo

```bash
curl -N -X POST http://localhost:3001/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "hola", "provider": "gemini", "model": "gemini-1.5-pro"}'
```

## Agregar un nuevo proveedor

1. Crear `src/provider/mi_proveedor.rs` implementando el trait `AiProvider`
2. Registrarlo en `ProviderRegistry::from_env()` segun la key del entorno
3. Re-exportarlo en `provider/mod.rs`

## Crates demostrados

- `ag-observe`: logging y trazabilidad
