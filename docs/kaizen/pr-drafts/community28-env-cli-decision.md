## Resumen
Close RFC-0020 PR 4 honestly: decide and document whether the seventh crate in the A2 migration order (`ag-cli`) belongs in PR 4. Decision: PR 4 is complete for declarable service configuration (six crates migrated); `ag-cli` is not migrated in PR 4 by design because its six direct environment reads are runtime session context, the loader's own source report, or opt-in/feature-gated credentials read where used. Documentation-only change; no production code.

## Componente afectado
Documentacion y estado: `docs/audits/rfc0020-pr4-completion.md` (nuevo), `docs/rfc/RFC-0020-environment-and-secrets.md` (nota aditiva de estado), `CHANGELOG.md`, este descriptor. Sin cambios bajo `crates/`, sin API publica nueva, sin default ni feature modificado, sin RFC/ADR nueva.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [ ] tech-debt
- [x] documentation
- [ ] refactor

## Documentos relacionados
- `docs/rfc/RFC-0020-environment-and-secrets.md` seccion 5 PR 4 y seccion 4.2/4.3 (ya aprobada; sin RFC nueva).
- `docs/audits/oportunidades-arquitectura.md` A2 (esquema declarado, orden `storage -> observe -> workers -> realtime -> auth -> cache -> cli`, criterio de descarte).
- `CLAUDE.md` reglas 5 (bloqueo arquitectonico), 8 (implementacion incremental), 21 (anti-complejidad), 28 (RFC).
- RFC-0018 principio 4 (`ag` nunca interactivo por sorpresa) para la deteccion de TUI.
- EVIDENCIA.md REG-EV-07: decision con source_ids verificados (file:line reales, reproducibles).

## Decision de diseno

Verificacion real de los seis puntos de lectura de `ag-cli`:

1. `interactive.rs:41,46,47,49` - `CI`, `AG_NO_TUI`, `NO_COLOR` via `var_os` en `Session::from_environment`: contexto de sesion para deteccion de TUI (RFC-0018). Semantica `OsString` no UTF-8 y chequeo de cadena vacia; declararlas en `ag.toml` con loader global cambiaria el comportamiento de `ag` sin proyecto. No declarable.
2. `interactive.rs:48` - `TERM == "dumb"`: mismos motivo y decision. No declarable.
3. `commands/env.rs:240` - `var_os(declaration.name())` solo comprueba presencia para informar el origen `process`/`default`/`none` en `ag env list` (PR 2, ya fusionado). Es el reporte del propio loader; migrarlo seria circular. No declarable en PR 4.
4. `commands/workers.rs:102` - `DATABASE_URL` en `workers_backend()`, tras `#[cfg(feature = "workers-runtime")]`: credencial del backend durable leida en el comando que la usa. `DATABASE_URL` esta ausente de las declaraciones de `ag-workers` (verificado). Cargarla en arranque exigiria la credencial para toda invocacion de `ag`. No declarable.
5. `commands/workers.rs:335` - `DATABASE_URL` sonda de presencia en `cmd_workers_doctor()`; la configuracion ya usa `ag_workers::WorkersConfig::apply_env()` (migrada). No declarable.
6. `commands/dns/provider.rs:32` - `AG_CLOUDFLARE_TOKEN` en `apply_provider()`, solo con `--provider cloudflare`; el codigo documenta "no credential is read from the manifest". Declararla abriria una via de fuga y un segundo esquema. No declarable.

`main.rs` no tiene loader global; `cmd_dev` toma `--bind` del argumento CLI (default `127.0.0.1:8080`) y lo pasa al hijo, sin leer `BIND`. No existe subconjunto seguro: un loader global en `main.rs` seria un cambio de arquitectura no documentado (reglas 5/8/21/28), crearia un segundo esquema de credenciales y romperia la deteccion de TUI y los comandos feature-gated. Por tanto no se toca codigo de produccion.

Evidencia de los seis crates migrados: `storage_env_declarations` (17 claves), `observe_env_declarations` (6), `workers_env_declarations` (6 fijas + 2 por cola), `realtime_env_declarations` (11), `auth_env_declarations` (8), `cache_env_declarations` (6); cada uno con `config_tests.rs` de paridad y su arista `-> core` en `scripts/check-dependency-direction.sh:64-84` (cache ya presente en :67).

## Plan de prueba

- `git diff --check` (exit 0).
- Escaneo de contenido: sin secretos reales, sin atribucion a IA, sin emojis, sin supresiones S-10 nuevas, sin cambios bajo `crates/`.
- `bash scripts/ag-check` (verde; descriptor tambien en `docs/pr-drafts/` para este gate).
- `bash scripts/kaizen check` (verde).
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` (exit 0; sin cambios de codigo, confirma que el workspace sigue documentando sin advertencias).

## Impacto en calidad (ratchet)

- Cobertura: igual (sin cambios de codigo).
- Advertencias: igual (`cargo doc -D warnings` en verde).
- Deuda (TODO/FIXME): igual (no se anade ninguna).

## Criterios de salida que avanza

- Decision documentada con tabla de clasificacion de los seis puntos de lectura y evidencia file:line reproducible.
- Los seis crates migrados quedan enumerados con su evidencia de declaration/paridad/guard.
- Declaracion exacta de cierre: PR 4 completo para configuracion de servicio declarable; `ag-cli` conserva lecturas de contexto por diseno.
- Nota de estado aditiva y fechada en RFC-0020; CHANGELOG actualizado; sin cambios de API, defaults, features ni codigo de produccion.

## Aprendizaje (opcional)

"Cerrar" no siempre significa migrar: cuando la evidencia muestra que las lecturas restantes son contexto de sesion, reporte del propio loader o credenciales opt-in leidas donde se usan, forzar una migracion degradaria la arquitectura (reglas 5/8/21/28) y romperia comportamiento documentado (RFC-0018). El cierre honesto es documentar el alcance real con source_ids verificados.

## Cierre de issues

Closes: none

Refs RFC-0020 PR 4 y audit A2. No hay numero de issue asignado a esta unidad en el tablero disponible (K-10).

## Checklist

- [x] Sin cambios de codigo de produccion
- [x] `bash scripts/ag-check` en verde
- [x] `bash scripts/kaizen check` en verde
- [x] Documentacion y CHANGELOG actualizados
- [x] Linea base no empeora
