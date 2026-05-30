# examples/

Proyectos de ejemplo ejecutables que demuestran el uso de
Anti-Gravital. Cada ejemplo es un crate independiente.

## Ejemplos disponibles

| Ejemplo          | Tipo            | Crates / DSL                | Requisitos externos |
|------------------|-----------------|-----------------------------|---------------------|
| `todo-api`       | binario         | `ag-core`, `ag-data`        | PostgreSQL          |
| `auth-mail-demo` | binario         | `ag-auth`, `ag-mail`        | ninguno             |
| `realtime-chat`  | binario         | `ag-realtime`, `ag-observe` | ninguno             |
| `ai-backend`     | binario         | `ag-observe`                | API key de IA       |
| `ecommerce-api`  | DSL (`schema.ag`) | compilador `ag-dsl`       | ninguno             |

Los cuatro binarios son miembros del workspace y compilan en CI.
`ecommerce-api` no es un crate: contiene `schema.ag` y los artefactos
generados por `ag generate` en `generated/`.

## Reglas

- Cada ejemplo es minimo y completo.
- Cada ejemplo trae su README con instrucciones de ejecucion.
- Los ejemplos no se publican en crates.io.
- Vease `docs/examples/README.md` para el catalogo y la hoja de ruta
  de ejemplos.
