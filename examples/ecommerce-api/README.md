# ecommerce-api — Anti-DSL v0.4 example

Reference example that demonstrates the full Anti-DSL compiler: relational
models, validations, endpoints and artifact generation.

## Schema

`schema.ag` defines five models with 1:N and N:M relations:

```
User  --1:N--> Order --1:N--> OrderItem <--N:1-- Product <--N:1-- Category
```

Implemented relations:

| Model     | Field          | Type       | Cardinality  |
|-----------|----------------|------------|--------------|
| User      | orders         | Order[]    | 1:N          |
| Category  | products       | Product[]  | 1:N          |
| Product   | category       | Category   | N:1          |
| Product   | order_items    | OrderItem[]| 1:N          |
| Order     | user           | User       | N:1          |
| Order     | items          | OrderItem[]| 1:N          |
| OrderItem | order          | Order      | N:1          |
| OrderItem | product        | Product    | N:1          |

## Artifact generation

```bash
ag schema lint --schema schema.ag    # validate the schema
ag generate --schema schema.ag --output generated
```

Artifacts generated under `generated/`:

| File                             | Contents                          |
|----------------------------------|-----------------------------------|
| `migrations/0001_initial.sql`    | CREATE TABLE + FOREIGN KEY        |
| `src/models.rs`                  | Rust structs with Option<M>/Vec<M>|
| `src/types.rs`                   | Request/response structs          |
| `src/handlers.rs`                | Axum handler stubs                |
| `src/router.rs`                  | Router with all routes            |
| `clients/typescript/types.ts`    | TypeScript interfaces             |
| `clients/typescript/client.ts`   | Typed HTTP client                 |
| `openapi.json`                   | OpenAPI 3.1 spec (JSON)           |
| `openapi.yaml`                   | OpenAPI 3.1 spec (YAML)           |

## Regenerate

Edit `schema.ag` and run `ag generate` to regenerate all artifacts. The files in
`generated/` must not be edited by hand.
