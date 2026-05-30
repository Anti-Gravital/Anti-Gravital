# ecommerce-api — ejemplo Anti-DSL v0.4

Ejemplo de referencia que demuestra el compilador Anti-DSL completo:
modelos relacionales, validaciones, endpoints y generacion de artefactos.

## Schema

`schema.ag` define cinco modelos con relaciones 1:N y N:M:

```
User  --1:N--> Order --1:N--> OrderItem <--N:1-- Product <--N:1-- Category
```

Relaciones implementadas:

| Modelo    | Campo          | Tipo       | Cardinalidad |
|-----------|----------------|------------|--------------|
| User      | orders         | Order[]    | 1:N          |
| Category  | products       | Product[]  | 1:N          |
| Product   | category       | Category   | N:1          |
| Product   | order_items    | OrderItem[]| 1:N          |
| Order     | user           | User       | N:1          |
| Order     | items          | OrderItem[]| 1:N          |
| OrderItem | order          | Order      | N:1          |
| OrderItem | product        | Product    | N:1          |

## Generacion de artefactos

```bash
ag schema lint --schema schema.ag    # verifica el schema
ag generate --schema schema.ag --output generated
```

Artefactos generados en `generated/`:

| Archivo                          | Contenido                         |
|----------------------------------|-----------------------------------|
| `migrations/0001_initial.sql`    | CREATE TABLE + FOREIGN KEY        |
| `src/models.rs`                  | Structs Rust con Option<M>/Vec<M> |
| `src/types.rs`                   | Request/response structs          |
| `src/handlers.rs`                | Handler stubs Axum                |
| `src/router.rs`                  | Router con todas las rutas        |
| `clients/typescript/types.ts`    | Interfaces TypeScript             |
| `clients/typescript/client.ts`   | Cliente HTTP tipado               |
| `openapi.json`                   | Especificacion OpenAPI 3.1 (JSON) |
| `openapi.yaml`                   | Especificacion OpenAPI 3.1 (YAML) |

## Regenerar

Edita `schema.ag` y ejecuta `ag generate` para regenerar todos los artefactos.
Los archivos en `generated/` no deben editarse manualmente.
