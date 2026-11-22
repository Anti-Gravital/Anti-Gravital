# Descriptor de PR

## Resumen

docs(qa): paquetes de verificacion reproducible para las puertas de fuzz 24h y benchmarks (Refs #156, #157)

## Fase afectada

Fases 3 y 6 del plan de auditoria previo a Fase 5. No avanza ninguna casilla de
`docs/roadmap/STATUS.md`: ambas puertas siguen `pending` y los issues #156 y
#157 permanecen abiertos y etiquetados `blocked`.

## Tipo de cambio

- [x] Documentacion de verificacion y metodologia
- [x] Evidencia real de smoke (no oficial) con metadata completa
- [x] Sincronizacion de `fuzz/Cargo.lock` con su manifiesto
- [ ] Nueva feature de producto
- [ ] Cambio de arquitectura, DSL, dependencias o stack
- [ ] Cambio que rompe compatibilidad

## Contexto

Los issues #156 y #157 estan bloqueados porque requieren, respectivamente, una
corrida manual de 24 horas por target de fuzz y la evidencia final de
benchmarks sobre codigo estabilizado. Ninguna de las dos cosas es ejecutable
por completo en una sesion de trabajo: 7 targets x 24 h, y hardware de
referencia con generador de carga, no estan disponibles.

En lugar de inventar resultados se entregan procedimientos reproducibles con
comandos exactos, entorno requerido, rutas de artefactos y plantillas vacias de
resultados, mas la evidencia real de un smoke corto claramente etiquetado como
no oficial. Las filas `Fuzz 24h` y `Benchmarks` del release gate siguen
`pending` y ahora apuntan a esos paquetes.

Nota de alcance: la puerta de fuzz cubre **siete** targets. Los documentos
historicos de la Fase 4 mencionan seis porque `fuzz_workers_payload` se anadio
despues (RFC-0012 / ADR-0013). El paquete nuevo exige los siete.

## Documentos relacionados

- `docs/fuzz/24h-gate-verification.md` (procedimiento y plantilla, nuevo)
- `docs/fuzz/results/2026-11-22-smoke.md` (smoke real de 30 s por target, nuevo)
- `docs/benchmarks/verification-package.md` (procedimiento y plantilla, nuevo)
- `docs/benchmarks/smoke-2026-11-22.md` (smoke real de criterion, nuevo)
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (dos filas apuntan a los paquetes)
- `docs/fuzz/README.md`, `docs/benchmarks/README.md`, `CHANGELOG.md`
- `docs/audits/pre-fase5-fuzzing.md` (seccion 4.5, gate manual)
- `docs/benchmarks/measurement-template.md` (regla 17)
- `CLAUDE.md` reglas 17, 18, 22, 26, 36

## Plan de prueba

No hay codigo Rust nuevo; solo documentacion y la regeneracion de un lockfile
de fuzz que ya estaba desactualizado frente a `fuzz/Cargo.toml`. Evidencia real
recolectada en este worktree:

```sh
cd fuzz
cargo +nightly fuzz run <target> -- -max_total_time=30   # los 7 targets, rc=0
cargo bench -p ag-core -p ag-workers --no-run                        # rc=0
cargo bench -p ag-core --bench shield_hello_world -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1 --noplot    # rc=0
cargo bench -p ag-workers --bench queue_throughput -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1 --noplot    # rc=0
git diff --check                                                     # limpio
```

El procedimiento completo (no ejecutado) esta en los dos paquetes de
verificacion. No se extrapola ningun numero del smoke a la puerta de 24 h ni a
Stage 6.

## Criterios de salida que avanza

Ninguno. No se marca ninguna casilla de `docs/roadmap/STATUS.md`. Se deja
preparada la trazabilidad para que un ejecutor con el entorno adecuado pueda
cerrar #156 y #157; no se cierra ninguno.

## Cierre de issues

Closes: none

- Refs #156 (paquete de verificacion y smoke real; faltan las siete corridas de 24 h)
- Refs #157 (paquete de verificacion y smoke real; falta la medicion final en hardware de referencia)

## Checklist

- [x] Pertenece a la fase correcta (auditorias previas a Fase 5).
- [x] Respeta la documentacion y no anade alcance, crates ni capacidades nuevas.
- [x] No anade dependencias nuevas; el lockfile solo se sincroniza.
- [x] Documentacion afectada actualizada en el mismo commit.
- [x] Sin emojis, sin atribucion de herramientas, mensaje de commit corto.
