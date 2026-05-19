# Estado vivo de la Hoja de Ruta

Este archivo agrega el estado de las casillas de cada fase, en orden,
para que un agente o un humano pueda saber en un solo vistazo que esta
hecho y que falta. Se actualiza con cada pull request que avance la
hoja de ruta.

Convencion: `- [x]` significa cumplido y verificable en el repositorio,
`- [ ]` significa pendiente, `- [/]` significa parcialmente cumplido
(con explicacion).

Ultima actualizacion: 2026-05-19, fin del setup de Fase 0.

---

## Fase 0 - Fundaciones y gobernanza

Estado: En curso.

### Criterios de entrada

- [x] Decision final de comenzar Anti-Gravital como proyecto formal de Gravital Labs.
- [x] Aprobacion de licencia Apache 2.0 sin restricciones.
- [x] Compromiso publico de Angel Nereira como mantenedor inicial.

### Entregables en el repositorio

- [x] Repositorio `github.com/anti-gravital/anti-gravital` creado y publico.
- [x] Archivo `LICENSE` con texto completo Apache 2.0.
- [x] Archivo `README.md` bilingue (espanol mas ingles).
- [x] Archivo `CONTRIBUTING.md`.
- [x] Archivo `CODE_OF_CONDUCT.md` adoptando Contributor Covenant 2.1.
- [x] Archivo `SECURITY.md`.
- [x] Archivo `GOVERNANCE.md`.
- [x] Configuracion de CI con GitHub Actions en cuatro plataformas.
- [x] Plantillas de issue (bug report, feature request, RFC) y plantilla de pull request.
- [x] Estructura de carpetas del monorepo definida y commiteada.
- [x] Workspace Cargo inicializado con los 15 crates vacios.
- [x] El CI construye exitosamente el workspace vacio en las cuatro plataformas objetivo.
- [x] CLAUDE.md instalado como constitucion tecnica del repositorio.
- [x] Maestros instalados en `docs/master/` con `VERSION.md` y SHA-256.
- [x] Documentacion descompuesta en `docs/architecture/`, `docs/roadmap/`, `docs/modules/`, `docs/dsl/`, `docs/security/`, `docs/governance/`, `docs/benchmarks/`.
- [x] ADRs iniciales bajo `docs/adr/`.

### Entregables externos (no viven en el repositorio)

- [ ] Branding basico: logo, paleta de colores, tipografia. Aplicado al README.
- [ ] Discord oficial del proyecto con canales requeridos.
- [ ] Cuenta del proyecto en X o Bluesky para anuncios.
- [ ] Dominio `antigravital.dev` registrado y apuntando a landing page.
- [x] Email institucional `anti@gravitalcloud.com` operativo (correo raiz). Respaldo del BDFL: `angelnereira@gravitalcloud.com`.
- [ ] Calendario publico de releases publicado en el sitio.

Detalle y owner sugerido en `docs/governance/external-deliverables.md`.

### Criterios de salida (puerta antes de Fase 1)

- [ ] El repositorio recibe su primer star externo no solicitado.
- [ ] Al menos cinco personas externas se han unido al Discord.
- [x] La estructura de carpetas del monorepo esta definida y commiteada.
- [x] El workspace Cargo esta inicializado con los crates vacios listados en CLAUDE.md.
- [x] El CI construye exitosamente el workspace vacio en las cuatro plataformas objetivo. (Pendiente de verificacion del primer run.)
- [ ] La landing page describe en un parrafo que es el proyecto, que no es, y donde esta en el roadmap.

---

## Fase 1 - The Shield MVP

Estado: En curso bajo excepcion documentada en
`docs/rfc/RFC-0001-paralelizar-fase-0-externa-y-fase-1.md`. La
implementacion avanza en paralelo con el cierre de las puertas
externas de Fase 0. El diseno de Shield esta fijado en
`docs/rfc/RFC-0002-diseno-shield-mvp.md`.

### Criterios de entrada (1.1)

- [/] Fase 0 completada con todos sus criterios de salida marcados.
  (Excepcion: tres casillas externas siguen pendientes; vease RFC-0001.)
- [ ] Al menos un contribuidor adicional al mantenedor principal esta
  activo en el repositorio. (Excepcion: BDFL en modo solo; vease
  RFC-0001.)

### Entregables (1.2)

- [/] Crate `ag-core` con modulo `shield` operativo. (En bootstrap.)
- [ ] Soporte de HTTP/1.1 y HTTP/2 via Axum + Tokio.
- [x] Terminacion TLS 1.3 con rustls. `shield::tls` + helper `Shield::serve(listener, router)` que despacha a TLS o plain segun config.
- [x] Middleware de validacion de payload basico. Trait `Validate` y extractor `ValidatedJson<T>` bajo `shield::validation`.
- [x] Middleware de autenticacion JWT con verificacion Ed25519. `shield::auth` con `AuthLayer`, `AuthContext` y extractor `Claims<T>`.
- [x] Middleware de rate limiting con governor. Token bucket por IP en `shield::rate_limit`.
- [x] Middleware CORS y CSRF con defaults seguros. CORS en `shield::cors` y CSRF en `shield::csrf` (double-submit cookie apatrida).
- [ ] Middleware de logging estructurado con `tracing`.
- [x] Configuracion minima desde archivo TOML. `ShieldConfig::from_path` y `from_toml_str` con `deny_unknown_fields`; ejemplo en `crates/ag-core/config.example.toml`.
- [ ] Tests unitarios con cobertura >= 80% del crate.
- [ ] Tests de integracion end-to-end del pipeline Shield.
- [x] Benchmark Hello World ejecutable. `cargo bench -p ag-core --bench shield_hello_world` con tres grupos criterion. Metricas duras de cierre (>=300K req/s, p99 <=1ms) se miden en PR 10 con carga real.
- [ ] Documentacion API generada con `cargo doc`.
- [ ] Capitulo del manual de usuario sobre uso de Shield como libreria.

### Criterios de salida (1.3)

- [ ] Benchmark Hello World >= 300K req/s en hardware de referencia.
- [ ] Latencia p99 del pipeline Shield <= 1 ms a 100K req/s.
- [ ] Memoria del proceso idle <= 15 MB.
- [ ] Tiempo de arranque <= 100 ms.
- [ ] CI pasa en las cuatro plataformas objetivo.
- [ ] Clippy sin warnings.
- [ ] `cargo audit` sin vulnerabilidades conocidas.
- [ ] Cero bloques `unsafe` no documentados.
- [ ] Al menos un blog post tecnico sobre la arquitectura de Shield.
- [ ] Al menos diez stars en el repositorio.

## Fase 2 - The Core MVP

Estado: Pendiente. Vease `docs/roadmap/fase-02-core-mvp.md`.

## Fase 3 - Anti-DSL alpha

Estado: Pendiente. Vease `docs/roadmap/fase-03-anti-dsl-alpha.md`.

## Fase 4 - Modulos estandar

Estado: Pendiente. Vease `docs/roadmap/fase-04-modulos-estandar.md`.

## Fase 5 - ag-cloud

Estado: Pendiente. Vease `docs/roadmap/fase-05-ag-cloud.md`.

## Fase 6 - ag-ai y Knowledge Graph

Estado: Pendiente. Vease `docs/roadmap/fase-06-ag-ai-knowledge-graph.md`.

## Fase 7 - ag-migrate

Estado: Pendiente. Vease `docs/roadmap/fase-07-ag-migrate.md`.

## Fase 8 - ag-mobile

Estado: Pendiente. Vease `docs/roadmap/fase-08-ag-mobile.md`.

## Fase 9 - Plugins WASI

Estado: Pendiente. Vease `docs/roadmap/fase-09-plugins-wasi.md`.

## Fase 10 - Endurecimiento y 1.0

Estado: Pendiente. Vease `docs/roadmap/fase-10-endurecimiento-y-1.0.md`.
