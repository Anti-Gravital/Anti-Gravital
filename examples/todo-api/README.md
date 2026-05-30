# todo-api

CRUD completo de tareas sobre PostgreSQL. Demuestra el roundtrip
`Request -> Shield -> Core -> Handler -> Respuesta` usando `ag-core`
(capa HTTP con `Shield`) y `ag-data` (pool de conexiones + migraciones).

## Requisitos externos

- Una base de datos PostgreSQL accesible.

Las migraciones de `migrations/` se aplican automaticamente al arrancar.

## Ejecucion

```bash
export DATABASE_URL="postgresql://usuario:clave@localhost/todos"
cargo run -p todo-api
```

Si no se define `DATABASE_URL`, se usa
`postgresql://postgres:postgres@localhost/todos`.

## Variables de entorno

| Variable                   | Default                                        | Descripcion                  |
|----------------------------|------------------------------------------------|------------------------------|
| `DATABASE_URL`             | `postgresql://postgres:postgres@localhost/todos` | Cadena de conexion           |
| `DATABASE_MAX_CONNECTIONS` | `10`                                           | Tamano maximo del pool       |
| `BIND`                     | `0.0.0.0:8080`                                 | Direccion de escucha         |
| `RUST_LOG`                 | `info,sqlx=warn`                               | Filtro de tracing            |

## API

| Metodo | Ruta         | Descripcion              |
|--------|--------------|--------------------------|
| GET    | `/todos`     | Lista todas las tareas   |
| POST   | `/todos`     | Crea una tarea nueva     |
| GET    | `/todos/:id` | Obtiene una tarea        |
| PUT    | `/todos/:id` | Actualiza una tarea      |
| DELETE | `/todos/:id` | Elimina una tarea        |
| GET    | `/health`    | Verificacion de estado   |

### Crear una tarea

```bash
curl -X POST http://localhost:8080/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"comprar pan","done":false}'
```

## Docker

Imagen de produccion (binario estatico MUSL sobre `scratch`):

```bash
docker build -t todo-api -f examples/todo-api/Dockerfile .
docker run -e DATABASE_URL=postgresql://... -p 8080:8080 todo-api
```

## Crates demostrados

- `ag-core`: capa HTTP con `Shield` (defaults seguros).
- `ag-data`: pool de PostgreSQL y ejecucion de migraciones.
