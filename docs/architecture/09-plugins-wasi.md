# Capitulo 9. Sistema de plugins WASI (ag-wasm-host)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 9
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [08-modulos-batteries-included.md](./08-modulos-batteries-included.md)
> Siguiente: [10-despliegue-ag-cloud.md](./10-despliegue-ag-cloud.md)

## 9. Sistema de plugins WASI (`ag-wasm-host`)

La extensibilidad de Anti-Gravital se construye sobre WebAssembly System Interface (WASI), no sobre el ecosistema de crates Rust nativos. Esta decisión tiene tres razones que la hacen no negociable.

La primera razón es seguridad. Los plugins son código de terceros que el operador del servidor ejecuta. Si fueran código nativo, un plugin malicioso o defectuoso podría corromper memoria del proceso, escapar a syscalls arbitrarias, o filtrar secretos. Los módulos WASI ejecutan en una sandbox con permisos explícitos declarados en el manifest del plugin; el plugin no puede acceder al filesystem, a la red, ni a syscalls que no estén declarados.

La segunda razón es multilenguaje. Un plugin WASI puede escribirse en Rust, Go (TinyGo), C, C++, AssemblyScript, Zig o cualquier lenguaje que compile a WebAssembly. Esto democratiza el ecosistema: un experto en seguridad que escribe en Go puede contribuir un exportador para Datadog sin tener que aprender Rust.

La tercera razón es estabilidad de ABI. La interfaz entre el host y el plugin se define con `wit-bindgen` y el Component Model, lo que permite que un plugin compilado para una versión de Anti-Gravital siga funcionando con versiones futuras sin recompilación, siempre que la ABI no cambie.

### 9.1 Runtime de plugins

El runtime es `wasmtime`, embebido como crate Rust. Cada plugin se carga en un store aislado con límites de memoria (256 MB por defecto, configurable), límites de fuel (consumo de instrucciones), y timeout de ejecución.

### 9.2 Ciclo de vida de un plugin

El ciclo de vida de un plugin tiene cinco estados. El primero es **descubierto**: el archivo `.wasm` está en el directorio de plugins del proyecto y aparece en el manifest. El segundo es **validado**: el host inspecciona el binario, verifica que el component model sea compatible, lee el manifest y confirma que los permisos solicitados están autorizados. El tercero es **cargado**: el módulo se compila ahead-of-time con Cranelift y se almacena en memoria. El cuarto es **activo**: el plugin recibe eventos y responde a invocaciones. El quinto es **descargado**: el plugin se libera, sea por shutdown del servidor o por reload dinámico.

### 9.3 Manifest del plugin

Cada plugin trae un archivo `plugin.toml` con sus metadatos y permisos solicitados:

```toml
[plugin]
name = "datadog-exporter"
version = "1.2.0"
author = "Gravital Labs"
license = "Apache-2.0"
description = "Exports metrics and traces to Datadog"

[abi]
anti_gravital_version = ">= 1.0.0, < 2.0.0"
component_model = "0.5"

[permissions]
network = ["api.datadoghq.com:443", "api.datadoghq.eu:443"]
env = ["DD_API_KEY", "DD_SITE"]
filesystem = []
clock = "read"

[capabilities]
exports = ["metrics_exporter", "trace_exporter"]
imports = ["host_logger", "host_clock"]

[limits]
max_memory = "64MB"
max_execution_time = "5s"
fuel = 100_000_000
```

### 9.4 API de host expuesta a plugins

El host expone un conjunto reducido de capacidades a los plugins, definidas en interfaces WIT (WebAssembly Interface Types). Las principales son: logger (escribir mensajes al sistema de tracing del host), clock (obtener tiempo actual y medir intervalos), metrics (registrar métricas adicionales), KV (almacenamiento clave-valor persistente por plugin), HTTP client (con allowlist de hosts del manifest), y events (subscripción al bus interno).

### 9.5 Puntos de extensión del framework

Los plugins pueden extender Anti-Gravital en cinco puntos: middleware adicional en la Shield (request hooks), handlers personalizados registrados en el router, exporters de observabilidad (métricas, traces, logs), processors de eventos (subscriptores al bus interno), y comandos personalizados de la CLI (`ag <plugin-cmd>`).

### 9.6 Plugins oficiales

El repositorio mantiene un conjunto de plugins oficiales bajo `plugins/`, cada uno con su propio crate y release cycle: `prometheus-exporter`, `datadog-exporter`, `sentry`, `honeycomb-exporter`, `slack-notifier`, `discord-webhook`. La existencia de plugins oficiales sirve como referencia técnica y como ejemplo de implementación para terceros.

### 9.7 Registro de plugins

A partir de la versión 1.0 del framework, se publica un registro oficial en `plugins.antigravital.dev`. El registro indexa plugins con metadatos verificados, escaneo de seguridad básico, y reviews de la comunidad. La instalación se hace con `ag plugin add <nombre>`. Los plugins se descargan, validan, y registran en el manifest del proyecto.

---

