# Capitulo 7. El lenguaje Anti-DSL (ag-dsl)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [06-nucleo-shield-y-core.md](./06-nucleo-shield-y-core.md)
> Siguiente: [08-modulos-batteries-included.md](./08-modulos-batteries-included.md)

## 7. The Anti-DSL language (`ag-dsl`): specification and incremental implementation

The DSL compiler is, together with the runtime, the technically most demanding component of the project. It is essentially a complete compiler: lexer, parser, semantic analysis, type system, code generation to multiple targets, formatter, linter, and LSP server. This section defines the language specification and the incremental implementation strategy.

### 7.1 Language philosophy

The Anti-DSL is declarative, not imperative. It describes contracts, not flows. The premise is that most of the value of a backend framework resides in the consistency of its external contract: which models exist, which endpoints expose them, which rules apply, which errors are returned. The internal logic stays in pure Rust.

The language draws inspiration from Prisma for the model syntax, from GraphQL SDL for the clarity of definitions, from sqlc for the integration with SQL, and from protobuf for the multi-target codegen. It is not a Turing-complete language and does not intend to be.

### 7.2 Incremental implementation by DSL versions

Probably the most important decision for making the compiler viable is to admit that the complete language cannot be delivered in the first version. The specification is delivered in incremental phases, each with a stable grammar that does not break the previous one. The DSL versions are independent of the framework versions and follow their own semver.

| DSL version | Grammatical capability                                                                                              | Milestone                  |
|-------------|-------------------------------------------------------------------------------------------------------------------|----------------------------|
| v0.1        | Basic models: fields, primitive types, annotations `@primary`, `@unique`, `@auto`                                  | End of Phase 3 (delivered) |
| v0.2        | Endpoints: method, path, body, response, errors                                                                    | End of Phase 3 (delivered) |
| v0.3        | Validations: `@min`, `@max`, `@email`, `@regex`, `@length`                                                         | End of Phase 3 (delivered) |
| v0.4        | Relationships between models: `1:1`, `1:N`, `N:M`, cascades                                                        | End of Phase 3 (delivered) |
| v0.5        | Authentication and authorization: `auth required`, `policy "..."`                                                  | End of Phase 4 (delivered) |
| v0.6        | Events: declaration of events emitted per endpoint, subscribers                                                    | End of Phase 4 (delivered) |
| v0.7        | Mail and declarative domains: `mail`, `domain`, `dns`, `tls`                                                       | End of Phase 4.5           |
| v0.8        | Plugin hooks (lifecycle, decorators)                                                                               | End of Phase 9             |
| v1.0        | Stable grammar, frozen under semver. Any subsequent extension will be additive.                                    | End of Phase 10            |

This table is realigned by `ADR-0007` (Phase 4.5). The multi-tenancy and data migration capabilities planned for intermediate versions of the DSL in earlier revisions are deferred: they will be specified in their own RFCs when the scope justifies it, without occupying a fixed numbered slot until then. This avoids promising features that do not have verified traction.

### 7.3 Complete schema example (v1.0 target)

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

### 7.4 Artifacts generated from a single schema

A single `schema.ag` produces, via `ag generate`:

| Artifact                           | Path                                | Purpose                                      |
|------------------------------------|-------------------------------------|----------------------------------------------|
| Rust structs with serde and validators | `src/models.rs`                 | Domain types                                 |
| Typed handler stubs                | `src/handlers/*.rs`                 | Ready signatures; the dev writes the body    |
| Compile-time checked sqlx queries  | `src/db/queries.rs`                 | Type-safe database access                    |
| Versioned SQL migrations           | `migrations/NNNN_*.sql`             | Database schema                              |
| TypeScript types                   | `clients/typescript/types.ts`       | Types shared with the frontend               |
| TypeScript HTTP client             | `clients/typescript/client.ts`      | Typed SDK for the frontend                   |
| Dart types                         | `clients/dart/lib/types.dart`       | Types shared with Flutter applications       |
| Dart client with Dio               | `clients/dart/lib/client.dart`      | Typed SDK for Flutter                        |
| OpenAPI 3.1 documentation          | `openapi.yaml`                      | Interactive documentation (Swagger UI)       |
| AsyncAPI specification             | `asyncapi.yaml`                     | Event documentation                          |
| JSON knowledge graph               | `.ag/knowledge-graph.json`          | Input for `ag-ai` and dashboards             |

### 7.5 Compiler architecture

The DSL compiler is organized in a traditional pipeline with well-defined stages:

The **lexer** phase tokenizes the `.ag` input. It is implemented with `logos` (a Rust crate that generates tokenizers from declarative definitions with derive macros). It produces a stream of positional tokens for error reporting with lines and columns.

The **parser** phase consumes tokens and produces an AST. It is implemented with `chumsky` (a parser combinator library with error recovery support), chosen over `nom` for its better handling of error messages readable by the end user.

The **semantic analysis** phase validates the AST: it checks that the references between models exist, that the types are consistent, that the RBAC policies refer to valid fields, that there are no cycles in the relationships, that the names do not collide with reserved words of Rust or SQL. It produces structured diagnostics with suggestions.

The **codegen** phase takes the validated AST and emits code to multiple targets. Each target (Rust, TypeScript, Dart, OpenAPI, SQL) is an independent module. The emission is done with `askama` templates for the textual outputs and with `quote` for the Rust code (which benefits from having a native Rust AST for emission).

### 7.6 Language server (LSP)

From version 0.3 of the DSL, an LSP server is included that offers autocompletion, live diagnostics, go-to-definition, find-references, hover types, and rename. It is distributed as an `ag-lsp` binary and integrates with any editor compatible with the protocol (VS Code, Neovim, Helix, Zed, IntelliJ via plugin).

The official plugin for VS Code is published in the marketplace under the name `Anti-Gravital`.

### 7.7 DSL tooling

The CLI offers three specific commands for the DSL:

`ag schema lint` reviews the `.ag` file and reports warnings about bad practices (models without indexes on foreign key fields, endpoints without rate limit, tautological policies, unhandled errors).

`ag schema diff <ref>` compares the current schema against a reference (git commit, tag, file) and reports breaking vs non-breaking changes. Essential for pull request reviews.

`ag schema migrate` generates the SQL migration needed to bring the database from the current state to the schema state. It includes a safety analysis: it detects destructive operations (drop column, drop table) and demands explicit confirmation.

---

