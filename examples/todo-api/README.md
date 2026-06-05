# todo-api

Full task CRUD over PostgreSQL. Demonstrates the
`Request -> Shield -> Core -> Handler -> Response` round-trip using `ag-core`
(HTTP layer with `Shield`) and `ag-data` (connection pool + migrations).

## External requirements

- A reachable PostgreSQL database.

The migrations in `migrations/` are applied automatically on startup.

## Running

```bash
export DATABASE_URL="postgresql://user:password@localhost/todos"
cargo run -p todo-api
```

If `DATABASE_URL` is not set, `postgresql://postgres:postgres@localhost/todos`
is used.

## Environment variables

| Variable                   | Default                                          | Description                  |
|----------------------------|--------------------------------------------------|------------------------------|
| `DATABASE_URL`             | `postgresql://postgres:postgres@localhost/todos` | Connection string            |
| `DATABASE_MAX_CONNECTIONS` | `10`                                             | Maximum pool size            |
| `BIND`                     | `0.0.0.0:8080`                                   | Listen address               |
| `RUST_LOG`                 | `info,sqlx=warn`                                 | Tracing filter               |

## API

| Method | Path         | Description              |
|--------|--------------|--------------------------|
| GET    | `/todos`     | List all tasks           |
| POST   | `/todos`     | Create a new task        |
| GET    | `/todos/:id` | Get a task               |
| PUT    | `/todos/:id` | Update a task            |
| DELETE | `/todos/:id` | Delete a task            |
| GET    | `/health`    | Health check             |

### Create a task

```bash
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"buy bread","done":false}'
```

## Docker

Production image (static MUSL binary on `scratch`):

```bash
docker build -t todo-api -f examples/todo-api/Dockerfile .
docker run -e DATABASE_URL=postgresql://... -p 8080:8080 todo-api
```

## Crates demonstrated

- `ag-core`: HTTP layer with `Shield` (secure defaults).
- `ag-data`: PostgreSQL pool and migration execution.
