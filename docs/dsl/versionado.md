# Anti-DSL - Implementacion incremental por versiones

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7.2.

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
