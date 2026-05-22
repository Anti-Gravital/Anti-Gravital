# Fase 4 — Modulos Estandar: DSL v0.5+v0.6, ag-observe, ag-auth (iteracion 1)

## Fase afectada

Fase 4 — Modulos Estandar

## Tipo de cambio

Nuevos crates y extension del compilador DSL (`feat`)

## Documentos relacionados

- `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md`
- `docs/superpowers/plans/2026-05-22-fase4-indice.md`
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` secciones 7, 8.1, 14
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` seccion Fase 4

## Resumen

Primera iteracion de Fase 4. Incluye tres entregas completadas:

**DSL v0.5 + v0.6** (`crates/ag-dsl`):
- `AuthMode` en endpoints: `auth required | optional | none`
- `policy "expresion"` con validacion semantica (requiere auth != None)
- Bloque `event nombre { payload T retain Nd }` con `EventDef` en el AST
- `events [nombre]` en endpoints con validacion de referencias declaradas
- `rust_gen`: Claims extractor + stubs de emision de eventos
- `openapi_gen`: securitySchemes BearerAuth + security por endpoint
- `ts_gen`: tipos de payload de eventos (ej. `UserCreatedEvent`)
- `async_api_gen`: nuevo generador AsyncAPI 2.6
- 136 tests, cobertura 95.88%

**ag-observe** (`crates/ag-observe`):
- `ObserveConfig::from_env()` con LOG_FORMAT, PROMETHEUS_PORT, OTEL_EXPORTER_OTLP_ENDPOINT
- `init()`: subscriber global tracing con fmt layer (pretty/json) + Prometheus
- `record_request`, `set_db_pool`, `inc/dec_active_connections`
- Handler Axum `/metrics` en formato Prometheus
- Dashboards Grafana JSON: overview, database, runtime
- 7 tests

**ag-auth** (`crates/ag-auth`):
- `AuthConfig::from_env()` para claves JWT y config WebAuthn
- `JwtSigner`: firma/verificacion EdDSA (Ed25519) via jsonwebtoken
- `api_keys`: generacion BLAKE3 con entropia OS, verificacion en tiempo constante
- `AgAuth` facade publica
- Migraciones SQL: ag_sessions, ag_api_keys, ag_webauthn_credentials
- TECH-DEBT documentado: WebAuthn, OAuth2, SessionStore (segunda iteracion)
- 13 tests

## Plan de prueba

- `cargo test --workspace` — todos los tests pasan
- `cargo clippy --workspace -- -D warnings` — cero advertencias
- `cargo fmt --all` — sin cambios pendientes
- `cargo build --workspace` — sin errores en las cuatro plataformas CI

## Criterios de salida avanzados (parciales — Fase 4 en curso)

- DSL v0.5+v0.6 completo y mergeado
- ag-observe operativo con Prometheus y dashboards Grafana
- ag-auth con JWT Ed25519 y API keys BLAKE3 funcionales
- Pendiente: ag-cache, ag-realtime, ag-storage, examples (iteraciones siguientes)

## Checklist final

- [x] Pertenece a Fase 4 segun Hoja de Ruta
- [x] Respeta documentacion (spec y planes)
- [x] No rompe arquitectura ni modularidad
- [x] No crea dependencias circulares
- [x] Compila sin errores
- [x] Pasa todos los tests
- [x] Pasa cargo fmt
- [x] Pasa cargo clippy -D warnings
- [x] Tiene documentacion (doc comments + TECH-DEBT)
- [x] No contiene emojis
- [x] No contiene atribucion de IA
- [x] Commits individuales por componente logico
