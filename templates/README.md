# templates/

Templates de proyecto consumidos por `ag new`. Cada template es un
arbol completo que el binario `ag` copia y renombra al crear un nuevo
proyecto.

## Templates disponibles

Valores validos de `ag new -t <template>`:

- `rest`: API REST minima con `ag-core` (`Shield` + axum). Una ruta `/`
  de ejemplo.
- `realtime`: servidor WebSocket con `ag-core` y el soporte `ws` de
  axum (ruta `/ws`).
- `fullstack`: REST con persistencia PostgreSQL via `ag-data`
  (conexion + migraciones en `migrations/`).

Si no se pasa `-t` en una sesion no interactiva, se usa `rest`.

## Estado

Tres templates entregados (`rest`, `realtime`, `fullstack`),
consumidos por `ag new` via `include_str!`. Templates adicionales
(por ejemplo un backend para clientes Flutter) se anadiran en fases
posteriores cuando los crates que requieren esten disponibles.
