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

## Versionado de las dependencias `ag-*`

Mientras los crates `ag-*` no se publiquen en crates.io, las
dependencias `ag-*` de cada template se fijan de forma **determinista**
a un commit concreto del repositorio mediante `rev = "<sha>"`. Un
proyecto generado con `ag new` compila siempre contra ese commit fijo,
no contra el HEAD movil de la rama por defecto, de modo que un cambio
incompatible fusionado aguas arriba no rompe los proyectos recien
creados (regla 31: no romper usuarios; regla 36: reproducibilidad).

`ag new` copia el manifiesto verbatim, por lo que el `rev` se conserva
tal cual en el proyecto generado.

Procedimiento para actualizar el pin (bump):

1. Elegir un commit estable de la rama por defecto verificado en verde.
2. Reemplazar el `rev` en los cuatro manifiestos
   (`rest`, `realtime`, `fullstack`) por ese SHA completo.
3. Regenerar un proyecto de cada template y ejecutar `cargo build`
   antes de fusionar.

Cuando comience la publicacion en crates.io, cada `{ git = ..., rev = ... }`
se sustituye por un requisito de version SemVer (`ag-core = "x.y"`).

## Estado

Tres templates entregados (`rest`, `realtime`, `fullstack`),
consumidos por `ag new` via `include_str!`. Las dependencias `ag-*`
estan fijadas por `rev` (ver seccion anterior). Templates adicionales
(por ejemplo un backend para clientes Flutter) se anadiran en fases
posteriores cuando los crates que requieren esten disponibles.
