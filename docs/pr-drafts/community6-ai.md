# Descriptor de PR: community6/ai

## Resumen

Wave 6: anadir cuatro plantillas de IA y datos (ai-backend-template, vector-search, rag-pipeline, wasi-plugin) verificadas con cargo check --offline

## Fase afectada

Fase 6 del catalogo de plantillas de la comunidad (catalogo de plantillas
preconstruidas). No altera fases bloqueantes del roadmap ni el alcance del
Blueprint v4.1. No modifica crates de produccion.

## Tipo de cambio

- [x] Nueva funcionalidad (cuatro directorios de plantilla bajo `templates/`)
- [x] Documentacion (README por plantilla y entrada en CHANGELOG)
- [ ] Correccion de bug
- [ ] Cambio de arquitectura / RFC / ADR
- [ ] Cambio de CLI compartida (fuera de este rol: solo `catalog`)

## Documentos relacionados

- `CLAUDE.md` (reglas maestras, secciones 14, 15, 17, 19, 26 y 29).
- `templates/README.md` y `docs/pr-drafts/README.md` (convencion de descriptors).
- Blueprint canonico `ANTI-GRAVITAL-Blueprint-v4.1.md` (alcance sin cambios).

## Cambios entregados

- `templates/ai-backend-template/`: `template.toml`, `README.md`, `Cargo.toml`,
  `config.toml`, `src/main.rs`. Chat provider-agnostico con `EchoProvider`
  local por defecto; `ag-ai` enlazado como limite reservado.
- `templates/vector-search/`: `VectorRepository` + `InMemoryVectorRepository`,
  embeddings locales y similitud coseno; `ag-data` como backend duradero
  reservado.
- `templates/rag-pipeline/`: flujo ingest -> retrieve -> answer con
  `AnswerGenerator` local; `ag-ai` enlazado como limite reservado.
- `templates/wasi-plugin/`: esqueleto de host WASI con manifiesto, politica de
  capacidades, registro y ciclo load/unload; ejecucion reservada a
  `ag-wasm-host`.
- `CHANGELOG.md`: entrada bajo `[Unreleased]` con cambios y verificacion.

## Plan de prueba

Comandos ejecutados en el worktree, con el directorio de destino compartido:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
# Scaffold de cada plantilla a /tmp con sustitucion de {{name}}, luego:
cargo check --offline      # en cada proyecto generado
```

Resultados reales (registro en `/tmp/opencode/agents/w6-ai-verify.log`):

```
RESULT ai-backend-template exit=0
RESULT vector-search exit=0
RESULT rag-pipeline exit=0
RESULT wasi-plugin exit=0
OVERALL exit=0
```

Smoke tests en vivo (registro en `/tmp/opencode/agents/w6-ai-smoke.log`):

- `ai-backend-template`: `/health` y `/providers` 200; `POST /chat` devuelve
  `{"provider":"echo","reply":"echo: hello"}`; `messages` vacio devuelve 422.
- `vector-search`: dos `POST /documents` y `GET /search?q=memory safety` rankea
  correctamente (`a` 0.866, `b` 0.408); `/health` informa 2 documentos.
- `rag-pipeline`: `POST /ask` antes de ingest devuelve 422; tras `POST /ingest`
  devuelve el contexto extraido; `/health` informa 1 chunk.
- `wasi-plugin`: termina con exit 0 con y sin `WASI_MODULE_PATH`; la politica
  rechaza la capacidad `net_out` y el ciclo load/unload se completa.

## Criterios de salida que avanza

- Catalogo de al menos 20 plantillas utilizables: se anaden 4 plantillas
  funcionales (total del rol: 4).
- Cada plantilla tiene `template.toml`, README con comandos y limitaciones, y
  compila con `cargo check --offline`.
- No se hardcoded secretos; las necesidades externas se declaran en metadata.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (catalogo de plantillas, wave 6).
- [x] Respeta la documentacion y no altera el alcance del Blueprint v4.1.
- [x] No anade dependencias circulares ni modifica crates compartidos.
- [x] Compila y pasa `cargo check --offline` en las cuatro plantillas.
- [x] Tiene documentacion (README por plantilla, CHANGELOG y este descriptor).
