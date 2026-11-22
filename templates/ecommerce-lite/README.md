# {{name}} - ecommerce lite

A compact, runnable subset of `examples/ecommerce-api`: products and orders
with stock checking and integer-cent prices. The default build needs no
database; a `postgres` feature switches persistence to `ag-data`.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP and security | `ag-core` (`Shield`), axum 0.7 |
| Persistence | `ag-data` + `sqlx` 0.8 behind the `postgres` feature |
| Money | integer cents (`i64`), never floating point |
| Ids | `uuid` v4 |

## Run offline (default)

```bash
cargo run
curl http://127.0.0.1:8080/products
curl -X POST http://127.0.0.1:8080/products \
  -H 'content-type: application/json' \
  -d '{"name":"notebook","price_cents":499,"stock":10}'
```

To order, copy an `id` from `/products`:

```bash
curl -X POST http://127.0.0.1:8080/orders \
  -H 'content-type: application/json' \
  -d '{"product_id":"<id>","quantity":2}'
```

`/health` reports `{"status":"ok","backend":"memory"}`. Stock is decremented
atomically under a mutex and orders exceeding stock return 422.

## Run with PostgreSQL

```bash
export DATABASE_URL='postgresql://postgres:postgres@localhost/{{name}}'
cargo run --features postgres
```

Startup applies `migrations/0001_init.sql` through `ag_data::run_migrations`.
Order creation runs inside an `ag_data::AgTx` transaction with
`SELECT ... FOR UPDATE`, so concurrent orders cannot oversell. Without
`DATABASE_URL` the same binary falls back to memory.

`external_requirements` declares `DATABASE_URL`; `cargo check --offline`
never opens a connection.

## DSL source

`schema.ag` is the canonical description of the models, requests and
endpoints, derived from `examples/ecommerce-api/schema.ag` and reduced to
Product and Order. It is not compiled by `cargo`; use `ag generate` to
produce typed models and migrations from it.

## Limitations

- No users, authentication, carts, payments or order lifecycle transitions.
- No product update/delete endpoints.
- The catalog is small and the in-memory store resets on restart.
- Prices are integer cents and there is no currency field.
- `list` has no pagination or filtering.
- Concurrency safety in the default build relies on `std::sync::Mutex`;
  the PostgreSQL build relies on row locking.
