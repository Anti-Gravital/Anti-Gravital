# Fase 9 - Sistema de plugins WASI

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-08-ag-mobile.md](./fase-08-ag-mobile.md)
> Siguiente: [fase-10-endurecimiento-y-1.0.md](./fase-10-endurecimiento-y-1.0.md)

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
