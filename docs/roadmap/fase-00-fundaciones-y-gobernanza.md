# Fase 0 - Fundaciones y gobernanza

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [preambulo.md](./preambulo.md)
> Siguiente: [fase-01-shield-mvp.md](./fase-01-shield-mvp.md)

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
- [ ] Archivo `SECURITY.md` con política de divulgación responsable y dirección `anti@gravitalcloud.com` (respaldo: `angelnereira@gravitalcloud.com`).
- [ ] Archivo `GOVERNANCE.md` describiendo modelo BDFL inicial y plan de transición.
- [ ] Configuración de CI con GitHub Actions: build en Linux x86-64, Linux ARM64, macOS ARM64, Windows x64.
- [ ] Plantillas de issue (bug report, feature request, RFC) y plantilla de pull request.
- [ ] Branding básico: logo, paleta de colores, tipografía. Aplicado al README.
- [ ] Discord oficial del proyecto con canales `#español`, `#english`, `#announcements`, `#help`.
- [ ] Cuenta del proyecto en X/Bluesky para anuncios.
- [ ] Dominio `antigravital.dev` registrado y apuntando a una landing page mínima.
- [ ] Email institucional `anti@gravitalcloud.com` operativo (correo raíz del proyecto).
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
