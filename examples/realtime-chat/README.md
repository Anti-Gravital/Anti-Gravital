# realtime-chat

Chat en tiempo real de sala unica. Demuestra `ag-realtime` con EventBus
in-process y streaming SSE. Sin base de datos, sin autenticacion, sin
servicios externos.

## Ejecucion

```bash
cargo run -p realtime-chat
```

Abrir **http://localhost:3000** en dos ventanas del browser para
ver los mensajes en tiempo real.

## Variables de entorno

| Variable     | Default  | Descripcion           |
|--------------|----------|-----------------------|
| `PORT`       | `3000`   | Puerto del servidor   |
| `LOG_FORMAT` | `pretty` | `pretty` o `json`     |

## API

| Metodo | Ruta        | Descripcion                    |
|--------|-------------|-------------------------------|
| GET    | `/`         | UI de chat (HTML embebido)    |
| GET    | `/events`   | Stream SSE de mensajes        |
| POST   | `/messages` | Publicar un mensaje           |
| GET    | `/health`   | Health check                  |

### Publicar con curl

```bash
curl -X POST http://localhost:3000/messages \
  -H "Content-Type: application/json" \
  -d '{"user":"alice","text":"hola desde curl"}'
```

### Escuchar eventos con curl

```bash
curl -N http://localhost:3000/events
```

## Crates demostrados

- `ag-realtime`: EventBus in-process pub/sub
- `ag-observe`: logging estructurado
