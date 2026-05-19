# Anti-Gravital — Hoja de Ruta y Puertas de Verificación

**Versión:** 4.0 — Mayo 2026
**Organización:** Gravital Labs — Nereira Technology and Business Solutions
**Origen:** República de Panamá
**Estado:** Documento vivo. Se actualiza con cada release.

---

## Cómo leer este documento

Este documento define la secuencia de fases por las que debe pasar el proyecto Anti-Gravital desde su inicio hasta convertirse en una versión 1.0 estable, lista para mercado, con la promesa cumplida.

Cada fase contiene cuatro bloques:

1. **Criterios de entrada**: condiciones que deben cumplirse antes de que la fase pueda comenzar. Estos vienen de la fase anterior.
2. **Entregables**: artefactos concretos que la fase debe producir.
3. **Criterios de salida (puerta)**: condiciones que deben cumplirse antes de pasar a la siguiente fase. Funcionan como puertas bloqueantes: si no se cumplen, no se avanza. Esto es no negociable.
4. **Riesgos específicos de la fase y mitigaciones**.

Los entregables y criterios de salida se expresan como casillas marcables. Este documento se mantiene en el repositorio y se actualiza tachando lo cumplido. Sirve como tablero de mando público del proyecto.

La regla principal es: **una fase no se da por concluida hasta que todas sus casillas de criterio de salida están marcadas**. El proyecto puede pausarse temporalmente entre fases, pero no puede saltarse pasos por presión externa o por urgencia.

---

## Resumen de fases

| Fase | Nombre                                     | Duración estimada | Estado    |
|------|--------------------------------------------|-------------------|-----------|
| 0    | Fundaciones y gobernanza                   | 1–2 meses         | Pendiente |
| 1    | The Shield MVP                             | 2–3 meses         | Pendiente |
| 2    | The Core MVP + roundtrip                   | 2 meses           | Pendiente |
| 3    | Anti-DSL alpha (v0.1–v0.4)                 | 3 meses           | Pendiente |
| 4    | Módulos estándar (auth, data, realtime)    | 3 meses           | Pendiente |
| 5    | `ag-cloud` — despliegue simplificado       | 2 meses           | Pendiente |
| 6    | `ag-ai` y Knowledge Graph                  | 2 meses           | Pendiente |
| 7    | `ag-migrate` — importadores                | 2 meses           | Pendiente |
| 8    | `ag-mobile` — Flutter bridge               | 2 meses           | Pendiente |
| 9    | Sistema de plugins WASI                    | 2 meses           | Pendiente |
| 10   | Endurecimiento y hito 1.0                  | 3 meses           | Pendiente |

**Duración total estimada:** 24–28 meses desde el inicio.
**Hito de versión beta pública (0.5):** final de fase 5 (~14 meses).
**Hito de versión 1.0 estable:** final de fase 10 (~28 meses).

---

## Fase 0 — Fundaciones y gobernanza

**Objetivo.** Crear las bases del proyecto: repositorio, licencia, documentación de gobernanza, CI, contribuyentes, comunicación con la comunidad. Sin código todavía. El producto de esta fase es un proyecto open source apto para recibir colaboradores.

### 0.1 Criterios de entrada

- [ ] Decisión final de comenzar Anti-Gravital como proyecto formal de Gravital Labs.
- [ ] Aprobación de licencia Apache 2.0 sin restricciones.
- [ ] Compromiso público de Ángel Nereira como mantenedor inicial.

### 0.2 Entregables

- [ ] Repositorio `github.com/gravital-labs/anti-gravital` creado y público.
- [ ] Archivo `LICENSE` con texto completo Apache 2.0.
- [ ] Archivo `README.md` bilingüe (español + inglés) con propuesta de valor.
- [ ] Archivo `CONTRIBUTING.md` con guía de contribución, convenciones de código, proceso de pull request.
- [ ] Archivo `CODE_OF_CONDUCT.md` adoptando Contributor Covenant 2.1.
- [ ] Archivo `SECURITY.md` con política de divulgación responsable y dirección `security@gravital.io`.
- [ ] Archivo `GOVERNANCE.md` describiendo modelo BDFL inicial y plan de transición.
- [ ] Configuración de CI con GitHub Actions: build en Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Plantillas de issue (bug report, feature request, RFC) y plantilla de pull request.
- [ ] Branding básico: logo, paleta de colores, tipografía. Aplicado al README.
- [ ] Discord oficial del proyecto con canales `#español`, `#english`, `#announcements`, `#help`.
- [ ] Cuenta del proyecto en X/Bluesky para anuncios.
- [ ] Dominio `antigravital.dev` registrado y apuntando a una landing page mínima.
- [ ] Email institucional `hello@antigravital.dev` operativo.
- [ ] Calendario público de releases publicado.

### 0.3 Criterios de salida (puerta antes de Fase 1)

- [ ] El repositorio recibe su primer star externo no solicitado.
- [ ] Al menos cinco personas externas se han unido al Discord.
- [ ] La estructura de carpetas del monorepo está definida y commitada (aunque sin contenido funcional).
- [ ] El workspace Cargo está inicializado con los crates vacíos: `ag-core`, `ag-dsl`, `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- [ ] El CI construye exitosamente el workspace vacío en las cuatro plataformas objetivo.
- [ ] La landing page describe en un párrafo qué es el proyecto, qué no es, y dónde está en el roadmap.

### 0.4 Riesgos de la fase

El principal riesgo es la procrastinación por perfeccionismo. La fase 0 no produce código que se ejecute, lo que tienta a postergarla. La mitigación es un timebox estricto: 8 semanas máximo. Si al término no están todos los entregables, se concluye con lo que haya y se documenta lo pendiente como deuda técnica de fase 0 a resolver durante la fase 1.

---

## Fase 1 — The Shield MVP

**Objetivo.** Implementar la capa Shield del núcleo: una pipeline de middleware Tower que valida, autentica básicamente, aplica rate limiting y entrega requests a un handler placeholder. Sin Core completo todavía. Sin DSL todavía. El producto es un binario que responde HTTP con seguridad básica y benchmark publicable.

### 1.1 Criterios de entrada

- [ ] Fase 0 completada con todos sus criterios de salida marcados.
- [ ] Al menos un contribuidor adicional al mantenedor principal está activo en el repositorio.

### 1.2 Entregables

- [ ] Crate `ag-core` con módulo `shield` operativo.
- [ ] Soporte de HTTP/1.1 y HTTP/2 vía Axum + Tokio.
- [ ] Terminación TLS 1.3 con rustls.
- [ ] Middleware de validación de payload básico (deserialización con serde y restricciones simples).
- [ ] Middleware de autenticación JWT con verificación Ed25519.
- [ ] Middleware de rate limiting con governor (token bucket por IP).
- [ ] Middleware CORS y CSRF con defaults seguros.
- [ ] Middleware de logging estructurado con `tracing`.
- [ ] Configuración mínima desde archivo TOML.
- [ ] Tests unitarios con cobertura ≥ 80% del crate `ag-core`.
- [ ] Tests de integración end-to-end del pipeline Shield.
- [ ] Benchmark Hello World ejecutable: `cargo bench` produce cifras reproducibles.
- [ ] Documentación API del crate generada con `cargo doc`, publicada en `docs.rs`.
- [ ] Capítulo del manual de usuario explicando cómo usar la Shield directamente como librería.

### 1.3 Criterios de salida (puerta antes de Fase 2)

- [ ] Benchmark Hello World alcanza ≥ 300 K req/s en hardware de referencia documentado.
- [ ] Latencia p99 del pipeline Shield ≤ 1 ms a 100 K req/s.
- [ ] Memoria del proceso idle ≤ 15 MB.
- [ ] Tiempo de arranque ≤ 100 ms.
- [ ] CI pasa en las cuatro plataformas objetivo.
- [ ] Análisis estático con `clippy` sin warnings.
- [ ] Análisis de dependencias con `cargo-audit` sin vulnerabilidades conocidas.
- [ ] Cero bloques `unsafe` no documentados.
- [ ] Al menos un blog post técnico publicado sobre la arquitectura de la Shield.
- [ ] Al menos diez stars en el repositorio.

### 1.4 Riesgos de la fase

El riesgo principal es underestimar la complejidad de TLS y rate limiting en producción. La mitigación es usar exclusivamente crates probados (rustls, governor) y no rodar implementaciones propias. El riesgo secundario es que las cifras de benchmark no alcancen el objetivo; la mitigación es publicar lo que se mide con honestidad y documentar el déficit.

---

## Fase 2 — The Core MVP y roundtrip completo

**Objetivo.** Completar el núcleo con la capa Core: router Axum, extractores tipados, sistema de errores, estado compartido. Implementar el roundtrip completo Request → Shield → Core → Handler → Respuesta. Conectar a PostgreSQL real para un CRUD mínimo. El producto es un binario que sirve una API real, aunque escrita manualmente sin DSL.

### 2.1 Criterios de entrada

- [ ] Fase 1 completada con todos sus criterios de salida marcados.
- [ ] El crate `ag-data` ha sido iniciado con sqlx como dependencia.

### 2.2 Entregables

- [ ] Crate `ag-core` con módulo `core` operativo.
- [ ] Router Axum integrado con la Shield.
- [ ] Extractores: `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`, `Query<T>`.
- [ ] Sistema de errores `AgError` con conversión automática a respuesta HTTP.
- [ ] Sistema de respuestas: JSON, plaintext, streams.
- [ ] Crate `ag-data` con pool de conexiones PostgreSQL vía sqlx.
- [ ] Sistema de migraciones embebido con `sqlx::migrate!`.
- [ ] Example app `todo-api` en `examples/` con CRUD completo.
- [ ] Benchmark CRUD + DB ejecutable.
- [ ] Crate `ag-cli` con comandos `new` (crea proyecto desde template), `dev` (arranca servidor con hot reload vía `cargo-watch`), `build` (compila release).
- [ ] Tres templates: `rest`, `realtime`, `fullstack`.

### 2.3 Criterios de salida (puerta antes de Fase 3)

- [ ] Benchmark CRUD + PostgreSQL alcanza ≥ 40 K req/s en hardware de referencia.
- [ ] Latencia p99 del CRUD ≤ 5 ms.
- [ ] La app `todo-api` corre exitosamente con `ag new` + `ag dev`.
- [ ] La app `todo-api` se despliega como binario único (`FROM scratch` Docker).
- [ ] El binario release del `todo-api` ocupa ≤ 20 MB.
- [ ] Documentación: "Tu primera API con Anti-Gravital" publicada.
- [ ] Al menos 50 stars en el repositorio.
- [ ] Al menos tres contribuidores externos con PRs merged.

### 2.4 Riesgos de la fase

El riesgo principal es la deriva de scope: querer añadir features no estrictamente necesarias para el MVP del Core. La mitigación es una declaración explícita de scope en el ticket de la fase: el Core de esta fase no incluye autorización RBAC compleja, no incluye eventos, no incluye caché, no incluye observabilidad completa. Esos llegan en fases posteriores.

---

## Fase 3 — Anti-DSL alpha (versiones 0.1 a 0.4 del DSL)

**Objetivo.** Construir el compilador del DSL con un subconjunto entregable de la gramática. Esta fase entrega el primer codegen funcional: modelos, endpoints básicos, validaciones y relaciones. Sin auth declarativa todavía, sin eventos todavía. El producto es la primera versión del flujo "definir → generar → implementar".

### 3.1 Criterios de entrada

- [ ] Fase 2 completada con todos sus criterios de salida marcados.
- [ ] Crate `ag-dsl` iniciado.
- [ ] Decisión final sobre librerías base del compilador (logos para lexer, chumsky para parser, askama y quote para codegen). Documentada en RFC.

### 3.2 Entregables

- [ ] DSL versión 0.1: modelos básicos con anotaciones primitivas (`@primary`, `@unique`, `@auto`).
- [ ] DSL versión 0.2: endpoints (método, path, body, response).
- [ ] DSL versión 0.3: validaciones (`@min`, `@max`, `@email`, `@regex`, `@length`).
- [ ] DSL versión 0.4: relaciones entre modelos (`1:1`, `1:N`, `N:M`).
- [ ] Generador Rust: structs con serde, validators, query builders sqlx.
- [ ] Generador SQL: migraciones idempotentes.
- [ ] Generador TypeScript: tipos y cliente HTTP.
- [ ] Generador OpenAPI 3.1.
- [ ] Comando `ag generate` que lee `schema.ag` y produce todos los artefactos.
- [ ] Comando `ag schema lint` que reporta warnings de mejores prácticas.
- [ ] Comando `ag schema diff <ref>` que reporta cambios breaking vs no-breaking.
- [ ] Diagnostics legibles para errores comunes del DSL (modelo no encontrado, tipo desconocido, anotación inválida).
- [ ] Servidor LSP básico (`ag-lsp`) con autocompletado y diagnostics.
- [ ] Plugin VS Code publicado en el marketplace.
- [ ] Suite de tests del compilador con cobertura ≥ 85%.
- [ ] Fuzzing del parser con `cargo-fuzz`: 24 horas sin crashes.
- [ ] Documentación de referencia del DSL versión por versión.

### 3.3 Criterios de salida (puerta antes de Fase 4)

- [ ] Un proyecto completo se puede crear, definir en `schema.ag`, generar, y ejecutar usando solo la CLI.
- [ ] El example `ecommerce-api` se reescribe completamente con DSL y funciona.
- [ ] Los benchmarks se mantienen: CRUD generado por DSL no es más lento que CRUD escrito a mano.
- [ ] El plugin VS Code tiene ≥ 100 instalaciones.
- [ ] Al menos un colaborador externo ha contribuido al compilador.
- [ ] La documentación del DSL es completa y revisada por al menos dos personas.
- [ ] Al menos 200 stars en el repositorio.

### 3.4 Riesgos de la fase

El compilador del DSL es el componente técnicamente más complejo del proyecto. El riesgo principal es subestimar el esfuerzo y exceder el cronograma. La mitigación es la implementación incremental por subversiones: si la fase corre largo, la subversión 0.4 (relaciones) puede postergarse a la fase 4 sin bloquear el avance.

El riesgo secundario son los mensajes de error del compilador. Un compilador con mensajes incomprensibles arruina la experiencia. La mitigación es priorizar diagnostics legibles desde el día uno, con tests específicos que verifiquen que los mensajes son útiles.

---

## Fase 4 — Módulos estándar

**Objetivo.** Completar los módulos batteries-included: auth, realtime, cache, storage, observe. Cada uno como crate independiente, con tests, documentación y ejemplos.

### 4.1 Criterios de entrada

- [ ] Fase 3 completada.
- [ ] DSL versión 0.5 (auth y políticas) iniciada.

### 4.2 Entregables

- [ ] DSL versión 0.5: declaración de auth y políticas RBAC.
- [ ] DSL versión 0.6: declaración de eventos.
- [ ] Crate `ag-auth` completo: WebAuthn, JWT Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens con rotación.
- [ ] Crate `ag-realtime` completo: WebSocket binario, SSE fallback, NATS embebido para casos pequeños, cliente NATS externo para producción.
- [ ] Crate `ag-cache` completo: moka L1 + Redis L2 con fred, invalidación por evento.
- [ ] Crate `ag-storage` completo: adaptadores S3, MinIO, filesystem local. URLs firmadas. Procesamiento de imágenes.
- [ ] Crate `ag-observe` completo: tracing, OpenTelemetry exporter, métricas Prometheus, dashboards Grafana JSON incluidos.
- [ ] Integración de tokio-console en modo dev.
- [ ] Example `realtime-chat` en `examples/`.
- [ ] Example `ai-backend` en `examples/` que demuestra streaming SSE.
- [ ] Tests de integración cross-module.

### 4.3 Criterios de salida (puerta antes de Fase 5)

- [ ] Los cinco módulos publicados en crates.io con sus respectivos releases independientes.
- [ ] Cobertura de tests ≥ 80% en cada módulo.
- [ ] Documentación cada módulo: README, guía de uso, referencia de API.
- [ ] Performance: el módulo `ag-realtime` sostiene 50 K conexiones WebSocket en una instancia 2 vCPU sin degradación.
- [ ] Performance: el módulo `ag-cache` muestra ≥ 1 M ops/segundo en L1.
- [ ] Al menos cinco issues bug reports cerrados por la comunidad.
- [ ] Al menos 500 stars en el repositorio.

### 4.4 Riesgos de la fase

El riesgo principal es la fragmentación del esfuerzo entre cinco módulos paralelos. La mitigación es secuenciar la implementación: primero auth (bloquea muchos casos de uso), luego data avanzado, luego realtime, luego cache, luego storage, luego observe.

---

## Fase 5 — `ag-cloud` despliegue simplificado

**Objetivo.** Construir el subsistema de despliegue al estilo Railway/Fly.io. Soporte para los cuatro targets: docker-compose, fly, railway, k8s. Este es el hito de **versión beta pública (0.5)**.

### 5.1 Criterios de entrada

- [ ] Fase 4 completada.
- [ ] Decisión RFC sobre los targets de despliegue soportados en la 1.0.

### 5.2 Entregables

- [ ] Crate `ag-cloud` con módulos para cada target.
- [ ] Especificación del archivo `deploy.ag`.
- [ ] Generador de Dockerfile multi-stage optimizado para imagen mínima.
- [ ] Target docker-compose: generación completa de stack con Caddy como reverse proxy y TLS automático.
- [ ] Target fly: integración con flyctl.
- [ ] Target railway: integración con su API.
- [ ] Target k8s: generación de manifests estándar.
- [ ] Comando `ag deploy`.
- [ ] Comando `ag rollback`.
- [ ] Pipeline de migraciones de base de datos integrado al despliegue.
- [ ] Documentación: "Desde cero a producción en 15 minutos" con cada target.

### 5.3 Criterios de salida (puerta antes de Fase 6 y versión 0.5)

- [ ] El example `todo-api` se despliega exitosamente a Fly.io con `ag deploy`.
- [ ] El example `ecommerce-api` se despliega exitosamente con docker-compose a un VPS y se accede vía dominio con TLS.
- [ ] El example `realtime-chat` se despliega exitosamente a Railway.
- [ ] Versión 0.5 (beta pública) liberada en GitHub Releases.
- [ ] Anuncio público en Hacker News, Reddit `/r/rust`, Twitter/X, Bluesky, LinkedIn.
- [ ] Al menos diez proyectos externos reportan que han desplegado Anti-Gravital en producción o staging.
- [ ] Al menos 1 500 stars en el repositorio.

### 5.4 Riesgos de la fase

El riesgo principal es la dependencia de APIs externas (Fly, Railway) que pueden cambiar. La mitigación es estructurar cada target como un módulo desacoplado con tests de contrato.

---

## Fase 6 — `ag-ai` y Knowledge Graph

**Objetivo.** Construir el módulo de IA con el knowledge graph y las capacidades asistidas.

### 6.1 Criterios de entrada

- [ ] Versión 0.5 (beta pública) liberada.
- [ ] Retroalimentación de los primeros usuarios incorporada en backlog.

### 6.2 Entregables

- [ ] Generador del knowledge graph desde el AST del DSL.
- [ ] Persistencia del graph en `.ag/knowledge-graph.json`.
- [ ] Generador de documentación arquitectónica Markdown desde el graph.
- [ ] Generador de diagramas C4 (Context, Container, Component) en Mermaid.
- [ ] Dashboard interactivo del graph en el dev server (`ag dev`).
- [ ] Comando `ag ai suggest-schema` con integración a proveedor configurable.
- [ ] Comando `ag ai review-migration`.
- [ ] Comando `ag ai analyze-architecture`.
- [ ] Soporte para proveedores: Anthropic Claude, OpenAI, Ollama local, vLLM local.
- [ ] Modo offline donde las funciones AI están deshabilitadas pero el framework funciona.
- [ ] Documentación: "Anti-Gravital + agentes IA: el flujo schema-first" con ejemplos completos.

### 6.3 Criterios de salida (puerta antes de Fase 7)

- [ ] El knowledge graph se regenera correctamente con cada `ag generate`.
- [ ] La documentación arquitectónica generada es legible y útil (revisada por tres personas externas al equipo).
- [ ] Al menos una organización usuaria reporta que ha integrado `ag ai` en su flujo de trabajo.
- [ ] Al menos 2 500 stars en el repositorio.

### 6.4 Riesgos de la fase

El riesgo principal es la dependencia de proveedores externos de IA. La mitigación es la abstracción del proveedor y el modo offline.

---

## Fase 7 — `ag-migrate` importadores

**Objetivo.** Construir los importadores de migración desde frameworks legacy. Es probablemente la fase con mayor impacto en adopción real.

### 7.1 Criterios de entrada

- [ ] Fase 6 completada.
- [ ] Investigación de muestras reales: al menos diez schemas/proyectos de cada framework objetivo recolectados como corpus de testing.

### 7.2 Entregables

- [ ] Crate `ag-migrate` con cinco importadores:
  - [ ] Importador OpenAPI 3.0 y 3.1.
  - [ ] Importador Prisma.
  - [ ] Importador Django.
  - [ ] Importador FastAPI.
  - [ ] Importador Sequelize.
  - [ ] Importador GraphQL SDL.
- [ ] Comando `ag migrate from <framework> <ruta>`.
- [ ] Guías oficiales de migración por framework con ejemplos completos.
- [ ] Estudio de caso documentado: migración real de una aplicación FastAPI mediana.

### 7.3 Criterios de salida (puerta antes de Fase 8)

- [ ] Cada importador tiene cobertura de tests ≥ 80% sobre el corpus de proyectos reales.
- [ ] La guía de migración FastAPI ha sido validada por al menos un equipo externo que migró su aplicación.
- [ ] Al menos 3 500 stars en el repositorio.

### 7.4 Riesgos de la fase

Los importadores cubren la traducción del contrato, no la lógica de negocio. El riesgo es generar expectativas exageradas. La mitigación es documentación honesta sobre lo que se importa y lo que no.

---

## Fase 8 — `ag-mobile` Flutter bridge

**Objetivo.** Construir la integración con Flutter como objetivo prioritario móvil. Generación de SDK Dart completo, auth nativo, realtime.

### 8.1 Criterios de entrada

- [ ] Fase 7 completada.
- [ ] Al menos un colaborador con experiencia significativa en Flutter se ha unido al proyecto.

### 8.2 Entregables

- [ ] Crate `ag-mobile` con generador Dart.
- [ ] Paquete pub `anti_gravital` publicado en pub.dev:
  - [ ] Tipos generados con freezed.
  - [ ] Cliente HTTP con dio + interceptores.
  - [ ] Cliente WebSocket.
  - [ ] Cliente SSE.
  - [ ] Mocks para tests.
- [ ] Widgets de autenticación: registro y login con WebAuthn nativo (Android Credential Manager, iOS Passkeys), OAuth2.
- [ ] Example `flutter-fullstack` en `examples/`: app Flutter completa con backend Anti-Gravital.
- [ ] Documentación: guía de usuario Flutter.

### 8.3 Criterios de salida (puerta antes de Fase 9)

- [ ] El paquete `anti_gravital` en pub.dev tiene al menos 50 likes.
- [ ] El example `flutter-fullstack` corre en Android, iOS y web.
- [ ] Al menos una aplicación Flutter externa usa Anti-Gravital en staging o producción.
- [ ] Al menos 4 500 stars en el repositorio.

### 8.4 Riesgos de la fase

El riesgo principal es que el cambio de contexto Rust → Dart tenga fricciones imprevistas. La mitigación es comenzar con el caso más simple (CRUD) y construir incrementalmente.

---

## Fase 9 — Sistema de plugins WASI

**Objetivo.** Construir el sistema de plugins WASI con wasmtime, definir la ABI estable, publicar los plugins oficiales, y arrancar el registro público.

### 9.1 Criterios de entrada

- [ ] Fase 8 completada.
- [ ] Decisión RFC sobre el alcance de la ABI 1.0 de plugins. Aprobada por el comité técnico (formado en fase 4 o anterior).

### 9.2 Entregables

- [ ] Crate `ag-wasm-host` operativo sobre wasmtime.
- [ ] Definición de interfaces WIT (WebAssembly Interface Types) para el host.
- [ ] Especificación de `plugin.toml`.
- [ ] Implementación del ciclo de vida de plugin (descubrimiento, validación, carga, activación, descarga).
- [ ] Sandbox con límites de memoria, fuel y timeout.
- [ ] Plugins oficiales: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`.
- [ ] Comando `ag plugin add/remove/list`.
- [ ] Registro público en `plugins.antigravital.dev`.
- [ ] Guía: "Cómo escribir un plugin para Anti-Gravital" con ejemplos en Rust, Go (TinyGo) y AssemblyScript.

### 9.3 Criterios de salida (puerta antes de Fase 10)

- [ ] El registro publica al menos los seis plugins oficiales.
- [ ] Al menos tres plugins externos de terceros publicados en el registro.
- [ ] El benchmark muestra overhead de plugin ≤ 1% sobre handler nativo equivalente.
- [ ] Al menos 6 000 stars en el repositorio.

### 9.4 Riesgos de la fase

El riesgo principal es la complejidad del component model de WebAssembly, que sigue evolucionando. La mitigación es pinneo conservador de la versión soportada y compromiso temprano con la comunidad wasmtime.

---

## Fase 10 — Endurecimiento y hito 1.0

**Objetivo.** Llevar el proyecto a versión 1.0 estable. Es la fase de auditorías, hardening, optimización final, y declaración pública de estabilidad.

### 10.1 Criterios de entrada

- [ ] Fase 9 completada.
- [ ] DSL versión 1.0 (gramática estable) lista para freeze.
- [ ] El comité técnico está activo y operativo.

### 10.2 Entregables

- [ ] DSL versión 1.0 (gramática estable, congelada).
- [ ] Cobertura de tests ≥ 85% en todos los crates del workspace.
- [ ] Fuzzing de 72 horas sobre el parser DSL sin crashes.
- [ ] Fuzzing de 72 horas sobre el parser HTTP sin crashes.
- [ ] Auditoría externa de seguridad del componente Shield, contratada con empresa especializada (Trail of Bits, NCC Group o equivalente). Reporte público.
- [ ] Resolución de todos los findings críticos y altos de la auditoría.
- [ ] Load test: 500 K req/s sostenidos por 30 minutos con degradación ≤ 5%.
- [ ] Memory leak test: 24 horas de carga continua sin crecimiento de memoria detectable.
- [ ] Compilación verificada en: Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Compilación a `wasm32-wasi` para servir Anti-Gravital en edge functions.
- [ ] Manual oficial publicado: "The Anti-Gravital Book" en español e inglés.
- [ ] Curso de introducción al framework en YouTube (mínimo seis videos).
- [ ] Posición en TechEmpower Framework Benchmarks: top 10 en categorías Plaintext y JSON Serialization.

### 10.3 Criterios de salida (versión 1.0)

- [ ] Al menos tres proyectos externos usando Anti-Gravital en producción por al menos 30 días sin incidentes críticos.
- [ ] Al menos un servicio interno de Gravital Cloud usando Anti-Gravital en producción por 30 días sin incidentes críticos.
- [ ] Anuncio público de versión 1.0 con changelog completo.
- [ ] Compromiso de semver estricto desde la 1.0.
- [ ] Anuncio del calendario de versiones LTS.
- [ ] Charla en al menos una conferencia internacional (RustConf, EuroRust, RustNation o equivalente).
- [ ] Al menos 10 000 stars en el repositorio.
- [ ] El comité técnico ratifica la promoción a versión 1.0 por unanimidad.

### 10.4 Riesgos de la fase

El riesgo principal es la presión por liberar 1.0 antes de tiempo. La mitigación es la regla más estricta del proyecto: los criterios de salida son no negociables. Si no se cumplen, no se libera 1.0. Se libera 0.9.5, 0.9.6, hasta que se cumplen.

---

## Más allá de la 1.0: hojas de ruta futuras

Una vez liberada la 1.0, el proyecto entra en modo de mantenimiento estable con releases minor cada 3 meses. Los temas candidatos para versiones futuras incluyen:

- Versión 1.x: optimizaciones de rendimiento adicionales, soporte de protocolos adicionales (HTTP/3 vía QUIC).
- Versión 2.x: refactorización de la ABI de plugins si la comunidad WebAssembly hace cambios mayores. Soporte de nuevos targets de despliegue.
- Generador Swift para iOS nativo.
- Generador Kotlin Multiplatform para Android nativo y casos cross-platform.
- Soporte multi-tenant más sofisticado con federación de instancias.

Esta hoja de ruta extendida no es un compromiso. Se documenta para señalar dirección, pero se reservará a RFCs específicos cuando llegue el momento.

---

## Reglas de oro del proceso

A modo de cierre, las cinco reglas que rigen este proceso de extremo a extremo:

**Primera regla.** Una fase no se considera concluida hasta que todas sus casillas de criterio de salida están marcadas. Sin excepciones.

**Segunda regla.** Si una fase requiere más tiempo del estimado, se extiende. Si el alcance original no es alcanzable, se reduce con un RFC público, no se relajan los criterios de calidad.

**Tercera regla.** Toda decisión arquitectónica significativa requiere un RFC. La velocidad de iteración no justifica saltar el proceso.

**Cuarta regla.** El proyecto se libera cuando está listo, no cuando lo exige una fecha externa. La credibilidad técnica es el activo más valioso del proyecto.

**Quinta regla.** Toda promesa pública (benchmark, feature, fecha) se documenta con evidencia. Si no hay evidencia, no se promete.

Estas reglas existen por una razón. Anti-Gravital se propone competir con frameworks que han madurado durante décadas. La única manera de ser tomado en serio es construir con la misma seriedad.

---

**Fin del documento de Hoja de Ruta.**
Documento complementario: *Arquitectura Técnica e Implementación.*
Versión PDF unificada: *Anti-Gravital Blueprint v4.0 — Documento Maestro.*
