# Catalogo de ejemplos

Esta carpeta acompana al directorio `examples/` del workspace.
Mientras `examples/` contiene los proyectos Rust ejecutables, esta
carpeta documenta cada ejemplo, su proposito y la fase en la que se
entrega.

## Ejemplos por fase

| Fase | Ejemplo | Estado |
| --- | --- | --- |
| 2 | `todo-api` | Entregado |
| 3 | `ecommerce-api` | Entregado |
| 4 | `realtime-chat` | Entregado |
| 4 | `ai-backend` | Entregado |
| 4 | `auth-mail-demo` | Entregado |
| 4.6-D | `workers-basic` | Entregado |
| 4.6-D | `workers-scheduled` | Entregado |
| 4.6-D | `workers-producer-edge` | Entregado |
| 4.6-D | `workers-mail-integration` | Entregado |
| 4.6-D | `workers-postgres` | Entregado (requiere `DATABASE_URL` para el camino durable) |
| 8 | `flutter-fullstack` | Pendiente |

## Reglas

- Cada ejemplo es minimo pero completo. No incluye codigo de
  demostracion que el framework no produciria en un proyecto real.
- Cada ejemplo trae su README con: que demuestra, como ejecutarlo, que
  requisitos externos tiene, que comandos `ag` usa.
- Los ejemplos no se publican en crates.io.
- Los ejemplos compilan en CI como parte del workspace.

## Estado

Diez ejemplos entregados (nueve binarios + el ejemplo DSL
`ecommerce-api`), incluidos los cinco ejemplos de `ag-workers` de la
Fase 4.6-D (RFC-0012 seccion 38). Pendiente: `flutter-fullstack`
(Fase 8). Los README de ejemplo se migran a ingles al tocarse
(ADR-0008); los de `workers-*` y el indice `examples/README.md` ya
estan en ingles.
