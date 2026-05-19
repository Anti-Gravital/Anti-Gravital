# Capitulo 7. El lenguaje Anti-DSL (ag-dsl)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [06-nucleo-shield-y-core.md](./06-nucleo-shield-y-core.md)
> Siguiente: [08-modulos-batteries-included.md](./08-modulos-batteries-included.md)

## 7. El lenguaje Anti-DSL (`ag-dsl`): especificación e implementación incremental

El compilador del DSL es, junto con el runtime, el componente técnicamente más exigente del proyecto. Es esencialmente un compilador completo: lexer, parser, análisis semántico, sistema de tipos, generación de código a múltiples targets, formatter, linter, y servidor LSP. Esta sección define la especificación del lenguaje y la estrategia de implementación incremental.

### 7.1 Filosofía del lenguaje

El Anti-DSL es declarativo, no imperativo. Describe contratos, no flujos. La premisa es que la mayor parte del valor de un framework backend reside en la consistencia de su contrato externo: qué modelos existen, qué endpoints los exponen, qué reglas se aplican, qué errores se devuelven. La lógica interna queda en Rust puro.

El lenguaje se inspira en Prisma para la sintaxis de modelos, en GraphQL SDL para la claridad de las definiciones, en sqlc para la integración con SQL y en protobuf para el codegen multi-target. No es un lenguaje Turing-completo y no pretende serlo.

### 7.2 Implementación incremental por versiones del DSL

Probablemente la decisión más importante para que el compilador sea viable es admitir que no se puede entregar el lenguaje completo en la primera versión. La especificación se entrega en fases incrementales, cada una con una gramática estable que no rompe la anterior. Las versiones del DSL son independientes de las versiones del framework y siguen su propio semver.

| Versión DSL | Capacidad gramatical                                                                                              |
|-------------|-------------------------------------------------------------------------------------------------------------------|
| v0.1        | Modelos básicos: campos, tipos primitivos, anotaciones `@primary`, `@unique`, `@auto`                              |
| v0.2        | Endpoints: método, path, body, response, errors                                                                    |
| v0.3        | Validaciones: `@min`, `@max`, `@email`, `@regex`, `@length`                                                        |
| v0.4        | Relaciones entre modelos: `1:1`, `1:N`, `N:M`, cascadas                                                           |
| v0.5        | Autenticación y autorización: `auth required`, `policy "..."`                                                      |
| v0.6        | Eventos: declaración de eventos emitidos por endpoint, suscriptores                                                |
| v0.7        | Plugins: declaración de extensiones WASI usadas por el proyecto                                                    |
| v0.8        | Multi-tenancy: schema-per-tenant, row-level security                                                              |
| v0.9        | Migración de datos: snapshots, diff, generación de migraciones SQL versionadas                                    |
| v1.0        | Gramática estable. Cualquier extensión posterior será aditiva.                                                    |

### 7.3 Ejemplo completo de schema (v1.0 target)

```ag
# schema.ag — Ejemplo completo objetivo de la v1.0

config {
    project_name "fintech-api"
    database "postgres"
    runtime { workers auto, blocking_threads 512 }
}

enum UserRole {
    ADMIN
    USER
    BANNED
}

model User {
    id           UUID       @primary @auto
    email        String     @unique @max(255) @email
    password_hash String    @max(255)
    name         String     @max(100) @min(2)
    role         UserRole   @default(USER)
    created      Timestamp  @auto
    updated      Timestamp  @auto_update

    accounts     Account[]  @relation(name: "owner")
}

model Account {
    id        UUID      @primary @auto
    owner_id  UUID      @references(User.id) @on_delete(cascade)
    balance   Decimal   @precision(18,2) @default(0)
    currency  String    @length(3)
    created   Timestamp @auto
}

request CreateUserRequest {
    email     String @email
    password  String @min(12) @max(128)
    name      String @min(2) @max(100)
}

response UserResponse {
    id        UUID
    email     String
    name      String
    role      UserRole
    created   Timestamp
}

error EmailTaken      { status 409 message "Email already registered" }
error WeakPassword    { status 422 message "Password does not meet policy" }
error InsufficientFunds { status 402 message "Insufficient balance" }

endpoint CreateUser {
    method   POST
    path     /users
    auth     optional
    body     CreateUserRequest
    response UserResponse
    errors   [EmailTaken, WeakPassword]
    events   [user.created]
    rate_limit "5/min per_ip"
}

endpoint TransferFunds {
    method   POST
    path     /accounts/{from}/transfer
    auth     required
    policy   "user.id == params.from.owner_id"
    body     TransferRequest
    response TransferReceipt
    errors   [InsufficientFunds]
    events   [account.debited, account.credited, transfer.completed]
    rate_limit "10/min per_user"
}

event user.created {
    payload UserResponse
    retain  30d
}

event account.debited {
    payload AccountDelta
    retain  7y
}
```

### 7.4 Artefactos generados desde un solo schema

Un único `schema.ag` produce, mediante `ag generate`:

| Artefacto                          | Ruta                                | Propósito                                    |
|------------------------------------|-------------------------------------|----------------------------------------------|
| Structs Rust con serde y validators | `src/models.rs`                     | Tipos del dominio                            |
| Stubs de handlers tipados          | `src/handlers/*.rs`                 | Firmas listas; el dev escribe el cuerpo      |
| Queries sqlx compile-time checked  | `src/db/queries.rs`                 | Acceso a base de datos type-safe             |
| Migraciones SQL versionadas        | `migrations/NNNN_*.sql`             | Esquema de base de datos                     |
| Tipos TypeScript                   | `clients/typescript/types.ts`       | Tipos compartidos con frontend               |
| Cliente TypeScript HTTP            | `clients/typescript/client.ts`      | SDK tipado para frontend                     |
| Tipos Dart                         | `clients/dart/lib/types.dart`       | Tipos compartidos con aplicaciones Flutter   |
| Cliente Dart con Dio               | `clients/dart/lib/client.dart`      | SDK tipado para Flutter                      |
| Documentación OpenAPI 3.1          | `openapi.yaml`                      | Documentación interactiva (Swagger UI)       |
| Especificación AsyncAPI            | `asyncapi.yaml`                     | Documentación de eventos                     |
| Grafo de conocimiento JSON         | `.ag/knowledge-graph.json`          | Insumo para `ag-ai` y dashboards             |

### 7.5 Arquitectura del compilador

El compilador del DSL se organiza en pipeline tradicional con etapas bien definidas:

La fase de **lexer** tokeniza la entrada en `.ag`. Está implementada con `logos` (crate Rust que genera tokenizers a partir de definiciones declarativas con derive macros). Produce un stream de tokens posicionales para reporte de errores con líneas y columnas.

La fase de **parser** consume tokens y produce un AST. Está implementada con `chumsky` (parser combinator library con soporte de recuperación de errores), elegida sobre `nom` por su mejor manejo de errores legibles para el usuario final.

La fase de **análisis semántico** valida el AST: comprueba que las referencias entre modelos existan, que los tipos sean consistentes, que las políticas RBAC se refieran a campos válidos, que no haya ciclos en las relaciones, que los nombres no colisionen con palabras reservadas de Rust o SQL. Produce diagnósticos estructurados con sugerencias.

La fase de **codegen** toma el AST validado y emite código a múltiples targets. Cada target (Rust, TypeScript, Dart, OpenAPI, SQL) es un módulo independiente. La emisión se hace con templates de `askama` para los outputs textuales y con `quote` para el código Rust (que se beneficia de tener un AST nativo Rust para emisión).

### 7.6 Servidor de lenguaje (LSP)

Desde la versión 0.3 del DSL se incluye un servidor LSP que ofrece autocompletado, diagnostics en vivo, go-to-definition, find-references, hover types y rename. Se distribuye como un binario `ag-lsp` y se integra con cualquier editor compatible con el protocolo (VS Code, Neovim, Helix, Zed, IntelliJ vía plugin).

El plugin oficial para VS Code se publica en el marketplace bajo el nombre `Anti-Gravital`.

### 7.7 Herramientas del DSL

La CLI ofrece tres comandos específicos para el DSL:

`ag schema lint` revisa el archivo `.ag` y reporta warnings sobre malas prácticas (modelos sin índices en campos foreign key, endpoints sin rate limit, políticas tautológicas, errores no manejados).

`ag schema diff <ref>` compara el schema actual contra una referencia (commit git, tag, archivo) y reporta cambios breaking vs no-breaking. Esencial para revisiones de pull request.

`ag schema migrate` genera la migración SQL necesaria para llevar la base de datos del estado actual al estado del schema. Incluye análisis de seguridad: detecta operaciones destructivas (drop column, drop table) y exige confirmación explícita.

---

