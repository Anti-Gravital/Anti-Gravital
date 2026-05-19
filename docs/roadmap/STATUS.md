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
- [ ] Email institucional `hello@antigravital.dev` operativo.
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

Estado: Pendiente. Todos los criterios de entrada y entregables en
`docs/roadmap/fase-01-shield-mvp.md` permanecen sin marcar.

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
