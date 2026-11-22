# Fuzzing del repositorio

## Smoke test continuo (CI)

El job `fuzz-smoke` en `.github/workflows/quality.yml` ejecuta cada target
por 60 segundos en cada PR y push. Verifica que el harness no este roto.

## Gate manual de 24 horas (requerido antes de cerrar Fase 3)

El procedimiento verificable, los 7 targets obligatorios, las rutas de
artefactos y la plantilla de resultados estan en
`docs/fuzz/24h-gate-verification.md`. Una ejecucion de smoke de 30 s por target
esta registrada en `docs/fuzz/results/2026-11-22-smoke.md`. Ninguno de los dos
documentos sustituye la corrida de 24 horas; la fila `Fuzz 24h` sigue `pending`.

## Ejecucion programada en runner propio (RFC-0035)

El workflow `.github/workflows/fuzz-24h.yml` dispara el gate de forma mensual y
bajo demanda usando `scripts/run-fuzz-window.sh` con `DURATION=86400`, un target
por job (siete jobs secuenciales) porque un job no puede alojar 168 h. Solo
corre en un runner auto-alojado con la etiqueta `fuzz-dedicated`; nunca en un
runner publico de GitHub. Usa acciones fijadas por SHA, permisos minimos y
unicamente el `GITHUB_TOKEN`. La preparacion del host, el aislamiento de red,
el dimensionamiento de disco y los comandos de verificacion estan en
`self-hosted-runner.md`. Un aviso idempotente informa en la issue #156 si el
gate falla. Ni el workflow ni la preparacion del runner prueban que el gate
paso: la aceptacion sigue siendo una decision humana segun
`24h-gate-verification.md`, y la issue #156 permanece abierta.

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
