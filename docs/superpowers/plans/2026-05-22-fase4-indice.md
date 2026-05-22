# Fase 4 — Modulos Estandar: Indice de Planes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar los cinco modulos batteries-included de Anti-Gravital (ag-auth, ag-cache, ag-realtime, ag-storage, ag-observe) junto con DSL v0.5+v0.6 y dos ejemplos completos.

**Architecture:** Rama integradora `fase-4` con 7 sub-ramas de feature. Cada sub-rama tiene su propio plan de implementacion y se mergea a `fase-4` cuando pasa sus criterios de calidad.

**Tech Stack:** Rust 1.79+, Cargo workspace, testcontainers-rs para infra en tests.

**Spec:** `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md`

---

## Orden de ejecucion (estricto)

| Orden | Rama | Plan | Dependencias |
|---|---|---|---|
| 1 | `fase-4/dsl-v05-v06` | `2026-05-22-fase4-dsl-v05-v06.md` | ninguna |
| 2 | `fase-4/ag-observe` | `2026-05-22-fase4-ag-observe.md` | ninguna |
| 3 | `fase-4/ag-auth` | `2026-05-22-fase4-ag-auth.md` | ag-observe mergeado |
| 4a | `fase-4/ag-cache` | `2026-05-22-fase4-ag-cache.md` | ag-observe mergeado |
| 4b | `fase-4/ag-realtime` | `2026-05-22-fase4-ag-realtime.md` | ag-observe mergeado |
| 5 | `fase-4/ag-storage` | `2026-05-22-fase4-ag-storage.md` | ag-observe mergeado |
| 6 | `fase-4/examples` | `2026-05-22-fase4-examples.md` | todos mergeados |

Los pasos 4a y 4b se pueden ejecutar en paralelo con subagentes.

---

## Procedimiento por sub-rama

### Antes de empezar cada sub-rama

```bash
git checkout fase-4
git checkout -b fase-4/<nombre>
```

### Gate de calidad antes de mergear a fase-4

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

Todos deben pasar sin errores. Solo entonces:

```bash
git checkout fase-4
git merge --no-ff fase-4/<nombre> -m "feat(<nombre>): implementacion completa Fase 4"
```

### PR descriptor

Crear `docs/pr-drafts/fase-4-<nombre>.md` con el formato de la plantilla del proyecto
antes de mergear cada sub-rama a `fase-4`.

---

## Checklist de cierre de Fase 4

- [ ] `fase-4/dsl-v05-v06` mergeado a `fase-4`
- [ ] `fase-4/ag-observe` mergeado a `fase-4`
- [ ] `fase-4/ag-auth` mergeado a `fase-4`
- [ ] `fase-4/ag-cache` mergeado a `fase-4`
- [ ] `fase-4/ag-realtime` mergeado a `fase-4`
- [ ] `fase-4/ag-storage` mergeado a `fase-4`
- [ ] `fase-4/examples` mergeado a `fase-4`
- [ ] `cargo test --workspace` pasa con todos los modulos
- [ ] `cargo clippy --workspace -- -D warnings` pasa
- [ ] Cobertura >= 80% en cada crate (cargo-llvm-cov)
- [ ] Benchmark ag-cache L1: >= 1M ops/seg
- [ ] Benchmark ag-realtime: 50K conexiones WebSocket en 2 vCPU
- [ ] `docs/roadmap/STATUS.md` actualizado con Fase 4 completa
- [ ] `README.md` actualizado
- [ ] PR descriptor `docs/pr-drafts/fase-4.md` creado
- [ ] `fase-4` mergeado a `main`
