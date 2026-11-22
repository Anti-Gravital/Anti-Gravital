# Descriptor de PR: community5/examples

## Resumen

Add offline alpha prototype walkthrough, secure scaffold binds, and offline-path regression tests

## Fase afectada

Fase de release alpha (v0.1.0-alpha.1). No cambia el estado de ninguna fase ni
de ninguna casilla de `docs/roadmap/STATUS.md`. No toca `docs/master/`.

## Tipo de cambio

- [x] Documentacion tecnica (`docs/examples/alpha-prototype.md`, indices)
- [x] Tests de regresion (`crates/ag-cli/tests/alpha_prototype.rs`)
- [x] Correccion de seguridad en plantillas generadas (issue #191)
- [x] CHANGELOG
- [ ] Codigo de produccion fuera de plantillas
- [ ] Cambio de arquitectura, DSL, security policy o stack
- [ ] Dependencias de Cargo

## Documentos relacionados

- `docs/examples/alpha-prototype.md` (guia de esta rama)
- `docs/examples/README.md` y `docs/INDEX.md` (indices actualizados)
- `CHANGELOG.md` (seccion [Unreleased])
- `docs/pr-drafts/community5-security.md` y `docs/audits/alpha-release-security-review.md`
  (hallazgo D1 / issue #191, auditado por otra rama)

## Plan de prueba

Ejecutados en este worktree (target compartido, CARGO_INCREMENTAL=0):

1. `cargo build -p ag-cli --bin ag` (tras tocar `project.rs` para invalidar el
   include de plantillas): exit 0. El binario genera `main.rs` con `BIND` y
   `127.0.0.1:8080`.
2. `ag new demo --template rest --no-interactive`: exit 0, tres archivos, sin
   migraciones.
3. `cargo build` del proyecto generado (dep `ag-core` rev 91a95ff): exit 0.
4. Ejecutar el binario y `curl -i http://127.0.0.1:8080/`: HTTP 200,
   cuerpo `demo corriendo con Anti-Gravital`; `ss -ltnp` mostro
   `127.0.0.1:8080` (solo loopback).
5. `ag dev --bind 127.0.0.1:8080` en el proyecto generado: mismo resultado.
6. `ag generate --schema examples/ecommerce-api/schema.ag --output generated`:
   10 artefactos, incluido el SDK TypeScript, exit 0.
7. Tutoriales `workers-basic`, `workers-scheduled`, `workers-producer-edge`,
   `workers-mail-integration`, `auth-mail-demo` ejecutados sin servicios: exit 0.
   `workers-postgres` sin `DATABASE_URL`: imprime pista y exit 0.
8. `realtime-chat` y `ai-backend`: `/health` (y `/providers`) devuelven 200 sin
   claves ni servicios externos.
9. `cargo test -p ag-cli`: 100 + 4 + 12 + 3 + 11 + 6 tests, 0 fallos.
10. `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets
    --all-features -- -D warnings`: exit 0.
11. `git diff --check`: exit 0. Sin emojis ni marcas de herramientas IA.

Nota de entorno: el directorio Cargo compartido lo reconstruyen otras ramas de
la ola 5; `CARGO_BIN_EXE_ag` puede apuntar brevemente a otro binario. La
evidencia determinista se reprodujo tocando la fuente y reconstruyendo justo
antes de cada verificacion.

## Criterios de salida que avanza

- Ninguna casilla de `STATUS.md`. Esta rama hace usable y verificable el alpha
  para prototipos y cierra el defecto de enlace inseguro #191.

## Notas para el coordinador

- Las plantillas `rest`, `realtime`, `fullstack` y `layered` ahora enlazan
  `127.0.0.1:8080` por defecto y respetan `BIND`; su `config.toml` tambien
  genera `host = "127.0.0.1"`. Ningun archivo generado contiene `0.0.0.0`, asi
  que se cumplen los criterios de aceptacion y el comando de verificacion de
  #191. `ag dev --bind 0.0.0.0:8080` sigue permitiendo exponer todas las
  interfaces de forma explicita.
- El test de regresion de la ola 4 `wave4_regression.rs` tenia una unica
  asercion que fijaba el valor antiguo `0.0.0.0` en `config get bind.host`; se
  actualizo a `127.0.0.1` en la misma PR para que refleje el nuevo defaults.
  Es el unico cambio en un archivo de otra rama; se senala aqui para el
  coordinador. Sin el, #191 no puede cerrarse con el comando de verificacion
  del issue.
- No se edita README (propiedad de docs) ni `docs/master/`.

## Cierre de issues

Closes #191

## Checklist

- [x] Pertenece a la fase correcta (release alpha, sin cambio de fase).
- [x] Respeta la documentacion (guia nueva, indices, CHANGELOG).
- [x] No rompe arquitectura (solo plantillas, tests y documentacion).
- [x] No anade complejidad innecesaria.
- [x] No crea dependencias circulares (sin dependencias nuevas).
- [x] Compila.
- [x] Pasa tests.
- [x] Pasa fmt.
- [x] Pasa clippy.
- [x] Pasa git diff --check.
- [x] Tiene documentacion (guia + CHANGELOG).
- [x] Mantiene coherencia con Anti-Gravital v4.1.
