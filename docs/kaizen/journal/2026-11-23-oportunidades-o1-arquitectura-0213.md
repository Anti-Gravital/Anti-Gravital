# 2026-11-23 O1 arquitectura (proyecto oportunidades)

- **Tarea:** O1-arquitectura. Producir un catalogo de oportunidades de arquitectura e
  ingenieria, sin scope creep, en la rama `community15/opp-arquitectura`.
- **Decision:** entregar solo propuestas con evidencia verificable. Se priorizaron 7
  oportunidades: A1/A2 (determinismo de tests de entorno y esquema unico de config,
  ambos bajo RFC-0020 PR 4 ya aprobada), A3a/A3b (emisor diferido de `ag-edge`:
  prueba in-repo ahora, adaptante de referencia con RFC), A4 (dos vias durables
  mail/workers, con RFC para fusionar), A5 (guarda automatica de direccion de
  dependencias), A6 (cobertura visible de modulos nuevos) y A7 (allowlist de tamano,
  diferido a Phase 5+ con triaje humano). Se descarto proponer reescrituras, crates
  nuevos o features de producto.
- **Evidencia (comandos y resultados reales):**
  - `grep -rnE 'set_var|remove_var' crates/ --include='*.rs'`: ~50 mutaciones de entorno
    en tests; solo `ag-auth/config.rs` y `ag-realtime/config.rs` usan `ENV_LOCK: Mutex`.
    `ag-core/tests/config_env.rs` comparte `BAD_KEY` entre los tests de las lineas 33 y 68.
  - `grep -rnE '^ag-[a-z-]+ *=' crates/*/Cargo.toml`: 19 crates; aristas nuevas
    `ag-edge -> ag-domains`, `ag-mail -> ag-workers`, `ag-cli -> ag-workers`.
  - `grep -rn 'producer::emit|DeferredJobProducer' crates/ --include='*.rs'`: el puerto
    de `ag-edge` solo se usa en su propio test; sin adaptante in-repo.
  - `sed -n '175,215p' .github/workflows/quality.yml`: tarpaulin workspace, gate 80%.
  - `cat tools/file-size-allowlist.txt`: 14 entradas referidas a #142.
  - `find` en repo principal y worktrees de pares: sin `docs/audits/estado-honesto-*.md`.
- **Resultado:** `docs/audits/oportunidades-arquitectura.md`, descriptor
  `docs/kaizen/pr-drafts/community15-opp-arquitectura.md` (`Closes: none`, sin issue
  de GitHub para la tarea del tablero) y entrada de CHANGELOG. Commit local, sin push.
- **Verificacion real de `scripts/kaizen check` (K-4/AGENTS.md):** `descriptores`,
  `politicas de contenido`, `formato` y `lint` en verde; `tests`, `swarm`,
  `gobernanza` y `ratchet` en rojo. Se reproduce el MISMO resultado en un worktree
  limpio del commit base `493f77a`: `tests` falla porque `KZ_TEST_CMD="bash tests/run.sh"`
  apunta a un script que no existe en el arbol; `gobernanza` marca 4 especialistas
  nuevos sin entrada en `governance.json`; `swarm` hereda ese fallo y ademas reclama
  ADR-0001; `ratchet` marca deuda 4 > baseline 0 (marcadores preexistentes en
  `crates/ag-dsl/src/codegen/`). Ninguno es causado por este cambio, que solo toca
  `CHANGELOG.md` y `docs/`. No se relajo la linea base ni se silencio ningun gate
  (L-10/S-10); se reporta el estado real al orquestador.
- **Aprendizaje:** la auditoria publicada en `pre-fase5-architecture.md` es de la epoca
  de 17 crates y su verificacion de dependencias es manual; conviene convertirla en una
  prueba automatica (A5) antes de que el roster crezca de nuevo.
