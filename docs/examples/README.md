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
| 8 | `flutter-fullstack` | Pendiente |

## Reglas

- Cada ejemplo es minimo pero completo. No incluye codigo de
  demostracion que el framework no produciria en un proyecto real.
- Cada ejemplo trae su README con: que demuestra, como ejecutarlo, que
  requisitos externos tiene, que comandos `ag` usa.
- Los ejemplos no se publican en crates.io.
- Los ejemplos compilan en CI como parte del workspace.

## Estado

Cinco ejemplos entregados (cuatro binarios + el ejemplo DSL
`ecommerce-api`). Pendiente: `flutter-fullstack` (Fase 8). Los example
README de `examples/` estan en espanol; su migracion a ingles
(ADR-0008) se hara de forma gradual.
