# Capitulo 5. Arquitectura del ecosistema: modulos y responsabilidades

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 5
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [04-estado-del-arte.md](./04-estado-del-arte.md)
> Siguiente: [06-nucleo-shield-y-core.md](./06-nucleo-shield-y-core.md)

## 5. Arquitectura del ecosistema: módulos y responsabilidades

La decisión arquitectónica más importante derivada del análisis crítico del v3.0 fue separar el núcleo de los ecosistemas. El v3.0 intentaba ser simultáneamente framework backend, motor SSR, plataforma DevOps, orquestador AI, capa de observabilidad, framework móvil y sistema de plugins. Esto es inmanejable. La v4.0 reorganiza el proyecto como un ecosistema de crates Rust independientes, cada uno con un dominio propio, un mantenedor responsable, versionado semántico independiente y una superficie de API mínima.

### 5.1 Mapa del ecosistema

| Crate              | Dominio                                                          | Estado de criticidad |
|--------------------|------------------------------------------------------------------|----------------------|
| `ag-core`          | Runtime HTTP, router, extractores, error types, Shield/Core      | Núcleo               |
| `ag-dsl`           | Lexer, parser, AST, análisis semántico y codegen del Anti-DSL    | Núcleo               |
| `ag-cli`           | Binario `ag`: new, generate, dev, build, deploy, migrate         | Núcleo               |
| `ag-lsp`           | Language Server del Anti-DSL (diagnostics, autocompletado, hover) para archivos `.ag` | Núcleo      |
| `ag-auth`          | WebAuthn, JWT Ed25519, OAuth2, RBAC, rate limiting               | Estándar             |
| `ag-data`          | sqlx con verificación compile-time, migraciones, ORM tipado      | Estándar             |
| `ag-realtime`      | WebSocket, SSE, NATS embebido, pub/sub                           | Estándar             |
| `ag-cache`         | moka en memoria, adaptador Redis, invalidación por evento        | Estándar             |
| `ag-storage`       | S3, MinIO, filesystem local, URLs firmadas, procesamiento imagen | Estándar             |
| `ag-observe`       | tracing, OpenTelemetry, Prometheus, dashboards Grafana           | Estándar             |
| `ag-mail`          | SMTP outbound, templates tipados, colas de envío con reintentos, relay SMTP nativo, helpers SPF/DKIM/DMARC | Estándar diferido |
| `ag-workers`       | Motor de ejecución en segundo plano: jobs tipados, reintentos, DLQ, scheduling, worker pools | Estándar diferido |
| `ag-ui`            | SSR con askama, hidratación selectiva, integración HTMX          | Opcional             |
| `ag-cloud`         | Orquestación de despliegue Railway-like, Dockerfile gen          | Opcional             |
| `ag-domains`       | Gestión DNS vía trait `DnsProvider`, adapters (Cloudflare), certificados ACME, dominios de despliegue | Opcional infra |
| `ag-edge`          | Plano de datos edge en tiempo de request: routing por hostname, selección de certificado por SNI, política canónica/redirect | Opcional infra |
| `ag-ai`            | Doc generation, schema suggestions, knowledge graph              | Opcional             |
| `ag-mobile`        | Generación SDK Dart, auth nativo Flutter, offline sync           | Opcional             |
| `ag-migrate`       | Importadores OpenAPI, Prisma, Django, FastAPI, Sequelize         | Opcional             |
| `ag-wasm-host`     | Runtime de plugins WASI sobre wasmtime                           | Núcleo               |

La distinción entre **núcleo**, **estándar**, **estándar diferido** y **opcional** es importante. El núcleo es el conjunto mínimo que define lo que es Anti-Gravital. Los módulos estándar cubren el 90% de las necesidades de producción de cualquier servicio backend y se instalan por defecto en los templates oficiales. Un módulo **estándar diferido** (introducido por `ADR-0007`) tiene la madurez y el alcance de un estándar pero NO se instala por defecto en los templates: se incorpora cuando el proyecto lo necesita explícitamente. `ag-mail` es estándar diferido porque la mayoría de los backends acaba enviando correo transaccional (verificación, recuperación, magic links vía `ag-auth`), pero no todo proyecto lo usa desde el minuto cero. `ag-workers` (introducido por `RFC-0012`/`ADR-0013` en la Fase 4.6-D) es el segundo crate estándar diferido: la mayoría de los backends acaba necesitando ejecución en segundo plano (jobs, reintentos, scheduling), pero no todo proyecto la usa desde el primer día, así que tiene madurez de estándar sin instalarse por defecto en los templates. Los módulos opcionales se añaden cuando el proyecto los necesita; `ag-domains` es opcional de infraestructura (lo consume `ag-cloud` durante el despliegue) y `ag-cloud → ag-domains` es una dependencia documentada en la sección 5.3. El ecosistema arrancó en **17 crates** con la introducción de la Fase 4.5 y ha crecido de forma aditiva hasta **20** con `ag-lsp` (tooling DSL de la Fase 3), `ag-edge` (`ADR-0012`) y `ag-workers` (`ADR-0013`); el conteo canónico vive en `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`.

### 5.2 Diagrama del ecosistema

```
┌──────────────────────────────────────────────────────────────────┐
│                       Anti-Gravital Ecosystem                    │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌───────────────────────┐    ┌───────────────────────┐         │
│   │       ag-cli          │    │       ag-dsl          │         │
│   │  new · generate · dev │◄──►│  lexer · parser · AST │         │
│   │  build · deploy       │    │  semantic · codegen   │         │
│   └───────────┬───────────┘    └───────────┬───────────┘         │
│               │                            │                     │
│               ▼                            ▼                     │
│   ┌──────────────────────────────────────────────────┐           │
│   │                    ag-core                       │           │
│   │  Shield (Tower middleware) + Core (Axum router)  │           │
│   │  Extractores · Error types · Runtime Tokio       │           │
│   └────────┬───────────────────────┬─────────────────┘           │
│            │                       │                             │
│   ┌────────▼─────────┐    ┌────────▼─────────┐                   │
│   │  Módulos estándar │    │ ag-wasm-host    │                   │
│   │  ag-auth ────────►│    │ wasmtime + WASI │                   │
│   │  ag-data         │    │ plugin lifecycle│                   │
│   │  ag-realtime     │    └─────────────────┘                   │
│   │  ag-cache        │                                          │
│   │  ag-storage      │                                          │
│   │  ag-observe      │                                          │
│   └────────┬─────────┘                                          │
│            │                                                    │
│   ┌────────▼─────────────────┐                                  │
│   │  Estándar diferido       │                                  │
│   │  ag-mail (◄── ag-auth)   │ ──► cooperación SPF/DKIM/DMARC   │
│   │  outbound + adapters     │                                  │
│   │  (relay SMTP nativo)     │                                  │
│   └────────┬─────────────────┘                                  │
│            │                                                    │
│   ┌────────▼──────────────────────────────────────────┐         │
│   │              Módulos opcionales                   │         │
│   │  ag-ui    ag-cloud ─► ag-domains    ag-ai         │         │
│   │  ag-mobile    ag-migrate                          │         │
│   │                                                   │         │
│   │  ag-domains: DnsProvider + ACME + adapters        │         │
│   └───────────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 Reglas de dependencia entre crates

Para mantener el ecosistema sano se aplican reglas estrictas de dependencia:

Primera regla: `ag-core` no depende de ningún otro crate del ecosistema Anti-Gravital. Es la base sobre la que todo lo demás se construye. Cualquier funcionalidad considerada genérica suficientemente que necesite otro módulo debe extraerse a `ag-core` o convertirse en un trait que el módulo implementa.

Segunda regla: los módulos estándar pueden depender de `ag-core` y de otros módulos estándar siempre que no haya ciclos. Por ejemplo, `ag-auth` puede depender de `ag-data` para persistencia de sesiones, pero `ag-data` no puede depender de `ag-auth`.

Tercera regla: los módulos opcionales pueden depender de cualquier crate núcleo o estándar. No pueden depender entre sí salvo casos explícitamente justificados (por ejemplo, `ag-mobile` puede depender de `ag-ai` para generación de código asistida).

Cuarta regla: `ag-cli` depende de todos los demás crates (es el orquestador), pero solo a través de features Cargo, de modo que el binario `ag` puede compilarse con un subconjunto reducido.

Quinta regla: todos los crates publican versiones semánticas independientes. Una breaking change en `ag-cache` no fuerza a `ag-core` a subir mayor. Esto es esencial para la sostenibilidad de un proyecto open source.

Sexta regla (introducida por `ADR-0007`, Fase 4.5): la dirección de la
dependencia `ag-auth ↔ ag-mail` es estrictamente unidireccional. `ag-auth`
**consume** `ag-mail` para enviar correos de verificación, recuperación de
contraseña y magic links, definiendo un trait pequeño que `ag-auth` invoca.
`ag-mail` **NO** depende de `ag-auth`. Esta direccionalidad preserva la
segunda regla (no ciclos) y mantiene a `ag-mail` reusable de forma aislada
en cualquier proyecto Rust. La cooperación `ag-mail ↔ ag-domains` (para
materializar SPF/DKIM/DMARC) es opcional, vía feature de Cargo: si un
proyecto usa `ag-mail` con un proveedor externo (via SMTP) y no administra
DNS propio, `ag-domains` no es necesario.

Séptima regla (introducida por `ADR-0007`, Fase 4.5): el módulo opcional
`ag-cloud` **consume** `ag-domains` durante `ag deploy` para configurar DNS
y TLS, sin que la dependencia sea rígida en todos los targets. Si el
proyecto no declara dominios en su `schema.ag`, el flujo se omite.
`ag-domains` puede usarse de forma independiente desde la CLI sin `ag-cloud`.

### 5.4 Estructura del monorepo

```
anti-gravital/
├── Cargo.toml                  # Workspace root
├── LICENSE                     # Apache 2.0
├── README.md                   # Español + Inglés
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md                 # Política de divulgación responsable
├── crates/
│   ├── ag-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── shield/         # Middleware Tower
│   │       │   ├── tls.rs
│   │       │   ├── auth.rs
│   │       │   ├── rate_limit.rs
│   │       │   ├── validation.rs
│   │       │   ├── rbac.rs
│   │       │   └── cors.rs
│   │       ├── core/           # Router y handlers
│   │       │   ├── router.rs
│   │       │   ├── extractors.rs
│   │       │   ├── error.rs
│   │       │   └── state.rs
│   │       └── runtime/        # Configuración Tokio
│   │           └── mod.rs
│   ├── ag-dsl/
│   │   └── src/
│   │       ├── lexer.rs
│   │       ├── parser.rs
│   │       ├── ast.rs
│   │       ├── semantic.rs
│   │       ├── diagnostics.rs
│   │       └── codegen/
│   │           ├── rust_gen.rs
│   │           ├── ts_gen.rs
│   │           ├── dart_gen.rs
│   │           ├── openapi_gen.rs
│   │           └── sql_gen.rs
│   ├── ag-cli/
│   ├── ag-auth/
│   ├── ag-data/
│   ├── ag-realtime/
│   ├── ag-cache/
│   ├── ag-storage/
│   ├── ag-observe/
│   ├── ag-mail/                # Fase 4.5 — estándar diferido
│   ├── ag-ui/
│   ├── ag-cloud/
│   ├── ag-domains/             # Fase 4.5 — opcional infra
│   ├── ag-ai/
│   ├── ag-mobile/
│   ├── ag-migrate/
│   └── ag-wasm-host/
├── docs/                       # Documentación bilingüe
│   ├── es/
│   └── en/
├── examples/
│   ├── todo-api/
│   ├── ecommerce-api/
│   ├── realtime-chat/
│   ├── ai-backend/
│   └── flutter-fullstack/
├── templates/                  # Templates de `ag new`
│   ├── rest/
│   ├── realtime/
│   ├── fullstack/
│   └── mobile-backend/
├── plugins/                    # Plugins WASM oficiales
│   ├── prometheus-exporter/
│   ├── datadog-exporter/
│   └── sentry/
└── benchmarks/                 # Suite TechEmpower + comparaciones
    ├── hello-world/
    ├── json-crud/
    └── plaintext/
```

---

