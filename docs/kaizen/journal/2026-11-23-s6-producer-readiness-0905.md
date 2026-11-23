# 2026-11-23 S6 - auditoria de preparacion del puerto producer

- Tarea: S6-workers-producer (proyecto top5-roadmap), issue #112.
- Decision: path (b). No existe emisor real en ag-edge; no se inventa uno.

## Evidencia (comandos y resultados reales)
- `cargo test -p ag-edge --features producer`: 24 tests en verde, incluye
  `producer::tests::emit_forwards_request_to_adapter`. RC=0.
- `cargo tree -p ag-edge --features producer | grep -i ag-workers`: sin salida.
- `cargo tree -p ag-edge --features producer -e features | grep -i ag-workers`: sin salida.
- `cargo tree -p ag-workers --features producer | grep -i ag-edge`: sin salida.
- `grep -rn "emit(" crates/ag-edge/src`: solo `producer.rs` (test); el unico
  emit de programa executable esta en `examples/workers-producer-edge/src/main.rs:90`.
- `grep -rn "on_demand" crates/ag-edge crates/ag-domains crates/ag-cli`: solo
  `tls.rs:68` (predicado puro, sin enqueue) y una linea de README.

## Resultado
- Anadido `docs/audits/s6-producer-readiness.md` con brecha exacta, reglas que
  prohiben el emisor sintetico (CLAUDE.md 5/8/21/28/30), contrato del emisor
  futuro y condicion exacta de cierre de #112.
- Descriptores `docs/kaizen/pr-drafts/community20-s6-workers.md` y su espejo
  `docs/pr-drafts/community20-s6-workers.md` con `Closes: none` y `Refs #112`.
- Outbox S-11 `s6-rfc-section-drift.md`: las citas "RFC-0012 seccion 17.4/32"
  son obsoletas (el RFC tiene 10 secciones).
- Sin cambios de codigo de produccion. #112 queda ABIERTO; lo cierra el
  coordinador tras emisor real y verificacion completa.

## Aprendizaje
Un puerto con test verde puede parecer "avance" de una etapa que aun no tiene
productor real. La prueba de honestidad no es que el test pase, sino que exista
un evento con dueno de producto y consumidor. Las reglas 5/8/30 exigen documentar la
brecha antes que rellenarla con codigo especulativo.
