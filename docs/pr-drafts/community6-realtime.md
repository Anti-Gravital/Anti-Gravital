# Resumen

Add six realtime/async project templates to the wave-6 catalog (chat rooms, websocket echo/broadcast, typed workers with DLQ, scheduled jobs, IoT batch ingest, edge hostname routing).

## Fase afectada

- Fase 6 (catalogo de plantillas de comunidad). No altera el roadmap ni las fases bloqueantes del nucleo.
- Categoria: `realtime`.

## Tipo de cambio

- [x] Nuevas plantillas (`templates/<nombre>/`).
- [x] Documentacion (README de cada plantilla y CHANGELOG).
- [ ] Cambio de codigo de crates existentes.
- [ ] Cambio de CLI compartida, DSL, RFC o ADR.
- [ ] Cambio de dependencias del workspace.

## Documentos relacionados

- `CLAUDE.md` (reglas maestras: documentacion primero, sin features fuera de alcance).
- `PROTOCOL.md` de la board (wave 6, contrato minimo de `template.toml`): la seccion
  catalogo aloga la integracion futura de estas plantillas en el registro.
- RFC-0012 / ADR-0013 (`ag-workers`), ADR-0012 / RFC-0011 (`ag-edge`), ADR-0008 (ingles canonico).
- Dependencias fijadas al rev `91a95ff61ed6ef4c9d3f1c9069357e97389d1382`, el mismo que
  usan las plantillas `rest`, `realtime`, `fullstack` y `layered` existentes.

## Plan de prueba

Cada plantilla se scaffoldo a `/tmp/ag-tpl-verify/<nombre>` con sustitucion de
`{{name}}` y se verifico con el directorio de destino compartido:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
cd /tmp/ag-tpl-verify/<nombre> && cargo check --offline
```

Evidencia (todas las comprobaciones devolvieron 0):

- `realtime-chat-v2`: `cargo check --offline` OK. Smoke: health 200, join 204,
  message 201, presence `["alice"]`.
- `websocket-service`: `cargo check --offline` OK. Smoke: health 200,
  upgrade sin handshake 400.
- `worker-service`: `cargo check --offline` OK. Run: 3 correos enviados,
  reporte reintentado una vez, 1 entrada en DLQ (`reason=requested`).
- `scheduled-jobs`: `cargo check --offline` OK. Run: cleanup=9,
  aggregate=3, ping=1.
- `iot-ingest`: `cargo check --offline` OK. Smoke: health 200, POST 202,
  stats con 2 eventos agregados.
- `edge-router`: `cargo check --offline` OK. Smoke: host exacto 200,
  wildcard 200, www 308 con Location correcto, host desconocido 404.

## Criterios de salida que avanza

- Catalogo de la Fase 6: se anaden 6 plantillas funcionales y verificadas en la
  categoria `realtime` (6 de 6 con `cargo check --offline` en verde).
- Cada plantilla incluye metadatos, documentacion de ejecucion, limitaciones y
  servicios externos opcionales declarados.

## Cierre de issues

Closes: none

## Checklist

- [x] Compila (`cargo check --offline` en las seis plantillas scaffoldadas).
- [x] Tiene documentacion (README, config.toml, CHANGELOG).
- [x] No anade dependencias circulares ni toca crates compartidos.
- [x] Sin emojis, nombres de herramientas IA ni evidencia de atribucion de IA.
- [x] sin push ni merge; la rama queda lista para integracion del coordinador.
