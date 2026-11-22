# Descriptor de PR: community6/web

## Resumen

Add six offline web templates: static-site, marketing-site, blog, docs-site, dashboard and admin-panel

## Fase afectada

Ola 6 del catalogo de plantillas (objetivo: al menos 20 plantillas predefinidas).
No cambia el estado de ninguna fase ni de ninguna casilla de
`docs/roadmap/STATUS.md`. No toca `docs/master/`. No edita `CATALOG.md`,
`cli.rs` ni `project.rs` (propiedad de `catalog`).

## Tipo de cambio

- [x] Plantillas nuevas de proyecto (`templates/<seis-nombres>/`)
- [x] Documentacion de cada plantilla (README con comandos y limitaciones)
- [x] CHANGELOG (seccion [Unreleased])
- [x] Descriptor de PR
- [ ] Codigo de produccion fuera de plantillas
- [ ] Cambio de arquitectura, DSL, security policy o stack
- [ ] Dependencias de Cargo nuevas (se usan las ya presentes en `rest`)

## Documentos relacionados

- `PROTOCOL.md` ola 6 (contrato minimo de `template.toml` y propiedad de
  directorios).
- `/tmp/opencode/agents/w6-catalog.md` (build.rs auto-descubre `*.tmpl`,
  sustituye `{{name}}` y quita el sufijo; `template.toml` es solo metadata).
- `/tmp/opencode/agents/w6-qa.md` (el harness usa `ag new` y
  `cargo check --offline --locked`; el cableado CLI es de `catalog`).
- `CHANGELOG.md` (seccion [Unreleased]).

## Plan de prueba

Ejecutado en este worktree con el target compartido y Cargo desincremental:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
```

1. `/tmp/ag-w6-web-verify.sh`: para cada plantilla copia cada `*.tmpl` a
   `/tmp/ag-w6-web-check/<nombre>/`, sustituye `{{name}}` por `probe-<nombre>`,
   quita el sufijo y ejecuta `cargo check --offline`. Resultado:
   `OVERALL_FAIL=0`; seis `RESULT: PASS`; cero warnings. Log:
   `/tmp/ag-w6-web-verify.log`.
2. `/tmp/ag-w6-web-smoke.sh` y `/tmp/ag-w6-web-smoke2.sh`: servidor real +
   curl. blog `/` 200, `/posts/hello-world` 200, `/posts/missing` 404,
   `/api/posts` 200; docs-site `/docs/shield` 200, `/docs/missing` 404;
   dashboard `/api/metrics` 200; admin-panel sin cabecera 401, token
   equivocado 401, token correcto 200, sin token configurado 503;
   static-site y marketing-site 200 en sus rutas. Logs:
   `/tmp/ag-w6-web-smoke.log` y `/tmp/ag-w6-web-smoke2.log`.
3. Defecto detectado y corregido durante la verificacion: axum 0.7 usa
   `:param`, no `{param}`. Se cambiaron `/posts/{id}` y `/docs/{slug}` a
   `/posts/:id` y `/docs/:slug`; las tablas de rutas de los README se
   actualizaron. Tras el cambio, las rutas detail devuelven 200.
4. `git diff --check`: sin errores. Sin emojis ni evidencia de herramientas IA.

## Criterios de salida que avanza

- Ninguna casilla de `STATUS.md`. Aporta seis plantillas web al catalogo de la
  ola 6, verificadas con compilacion offline y pruebas de rutas en servidor real.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (ola 6, catalogo de plantillas).
- [x] Respeta la documentacion (README por plantilla, CHANGELOG, descriptor).
- [x] No rompe arquitectura (solo directorios nuevos bajo `templates/`).
- [x] No anade complejidad innecesaria (misma forma que `rest`/`realtime`).
- [x] No crea dependencias circulares (sin dependencias nuevas).
- [x] Compila (`cargo check --offline` en los seis scaffolds).
- [x] Pasa pruebas de rutas (smoke con servidor real).
- [x] Pasa fmt (sin codigo Rust en el workspace; plantillas con formato coherente).
- [x] Pasa clippy (sin cambios en crates; scaffolds sin warnings).
- [x] Pasa git diff --check.
- [x] Tiene documentacion (README por plantilla + CHANGELOG).
- [x] Mantiene coherencia con Anti-Gravital v4.1.
