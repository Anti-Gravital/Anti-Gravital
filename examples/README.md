# examples/

Runnable example projects demonstrating Anti-Gravital. Each example is an
independent crate (except `ecommerce-api`, which is a DSL schema plus its
generated artifacts).

## Available examples

| Example                    | Type              | Crates / DSL                   | External requirements |
|----------------------------|-------------------|--------------------------------|-----------------------|
| `todo-api`                 | binary            | `ag-core`, `ag-data`           | PostgreSQL            |
| `auth-mail-demo`           | binary            | `ag-auth`, `ag-mail`           | none                  |
| `realtime-chat`            | binary            | `ag-realtime`, `ag-observe`    | none                  |
| `ai-backend`               | binary            | `ag-observe`                   | AI API key            |
| `ecommerce-api`            | DSL (`schema.ag`) | `ag-dsl` compiler              | none                  |
| `workers-basic`            | binary            | `ag-workers`                   | none                  |
| `workers-scheduled`        | binary            | `ag-workers` (`scheduler`)     | none                  |
| `workers-producer-edge`    | binary            | `ag-workers` (`producer`)      | none                  |
| `workers-mail-integration` | binary            | `ag-mail` (`workers`)          | none                  |
| `workers-postgres`         | binary            | `ag-workers` (`postgres`)      | PostgreSQL (`DATABASE_URL`) |

The binaries are workspace members and compile in CI. `ecommerce-api` is not a
crate: it contains `schema.ag` and the artifacts produced by `ag generate`
under `generated/`. `workers-postgres` exits cleanly when `DATABASE_URL` is
not set; the other workers examples need no external service (native-first,
ADR-0009).

## Rules

- Each example is minimal and complete.
- Each example ships its own README with run instructions.
- Examples are not published to crates.io.
- See `docs/examples/README.md` for the catalog and the examples roadmap.
