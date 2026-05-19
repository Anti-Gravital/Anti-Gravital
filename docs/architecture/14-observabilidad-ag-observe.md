# Capitulo 14. Observabilidad (ag-observe)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 14
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [13-mobile-ag-mobile.md](./13-mobile-ag-mobile.md)
> Siguiente: [15-seguridad.md](./15-seguridad.md)

## 14. Observabilidad (`ag-observe`)

Aunque ya cubierta brevemente como módulo estándar, la observabilidad merece una sección propia porque es probablemente la diferencia más visible entre un framework juguete y un framework de producción.

### 14.1 Tres pilares

`ag-observe` cubre los tres pilares clásicos: métricas, traces y logs. El stack es OpenTelemetry como capa de abstracción, con exporters configurables.

Las métricas se exponen en `/metrics` en formato Prometheus por defecto. Incluyen latencia por endpoint (p50, p95, p99, p999), throughput, tasa de errores por código HTTP, uso de pool de base de datos, uso de pool de Redis, conexiones WebSocket activas, y métricas custom registradas por la aplicación.

Los traces se exportan vía OTLP a cualquier backend compatible (Tempo, Jaeger, Datadog, Honeycomb, Lightstep). Cada request genera un trace con spans para la Shield, los handlers, las queries SQL, las llamadas externas, y la emisión de eventos.

Los logs son estructurados (JSON por defecto) e incluyen siempre el correlation ID. Se exportan a stdout (estándar para entornos cloud-native) y opcionalmente a backends como Loki o Datadog.

### 14.2 Dashboards Grafana incluidos

El repositorio incluye dashboards Grafana pre-configurados en JSON que el operador importa directamente. Cubren: overview del servicio, latencia y throughput por endpoint, errores y excepciones, salud de la base de datos, salud del caché, y métricas de runtime Rust (uso de memoria, número de tasks Tokio, GC pauses — que siempre serán cero, pero el dashboard lo confirma).

### 14.3 Inspección en vivo con tokio-console

En modo desarrollo, `tokio-console` se habilita automáticamente. Permite al desarrollador conectarse al proceso y ver en tiempo real qué tasks están ejecutándose, cuáles están bloqueadas, dónde se están consumiendo los recursos. Es una herramienta de debugging tremendamente útil que existe solo en Rust con Tokio.

---

