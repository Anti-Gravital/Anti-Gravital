# Fuzzing del repositorio

## Smoke test continuo (CI)

El job `fuzz-smoke` en `.github/workflows/quality.yml` ejecuta cada target
por 60 segundos en cada PR y push. Verifica que el harness no este roto.

## Gate manual de 24 horas (requerido antes de cerrar Fase 3)

Ejecutar en hardware Linux x86-64 antes de mergear la rama `fase-3`:

    cd fuzz

    cargo +nightly fuzz run fuzz_lexer   -- -max_total_time=86400
    cargo +nightly fuzz run fuzz_parser  -- -max_total_time=86400
    cargo +nightly fuzz run fuzz_compile -- -max_total_time=86400

Registrar el resultado en `docs/fuzz/results/YYYY-MM-DD.md` con:

- Fecha y hora de inicio/fin
- Hardware (CPU, RAM, OS)
- Version de Rust nightly usada
- Commit del repositorio
- Resultado: sin crashes / lista de crashes encontrados

## Targets

| Target | Invariante verificado |
|--------|----------------------|
| fuzz_lexer | El lexer del Anti-DSL termina sin panic en cualquier UTF-8 |
| fuzz_parser | El parser del Anti-DSL termina sin panic o retorna Err controlado |
| fuzz_compile | El pipeline completo (lint + compile + generate) no panics |
| fuzz_resp | El parser RESP2 del servidor nativo de `ag-cache` no panics ni aloca sin limite |
| fuzz_signed_url | La verificacion de URLs firmadas de `ag-storage` no panics ni acepta falsos positivos |
| fuzz_storage_key | `validate_key` de `ag-storage` (frontera anti path-traversal) no panics y rechaza claves inseguras |
| fuzz_workers_payload | La frontera de decode de payloads de `ag-workers` (`decode`/`decode_versioned`) nunca panics: el fallo se vuelve `InvalidPayload` (dead-letter) |

## Reproducir un crash

Si el CI sube artefactos en `fuzz/artifacts/`:

    cd fuzz
    cargo +nightly fuzz run fuzz_compile artifacts/fuzz_compile/<crash-file>

## Crashes encontrados y corregidos

| Fecha | Commit fix | Descripcion |
|-------|-----------|-------------|
| 2026-05-22 | ff85c6f | lexer: IntLit .unwrap() -> .ok() — panic en enteros > i64::MAX |
