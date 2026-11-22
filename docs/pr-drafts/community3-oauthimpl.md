# Descriptor de PR: community3/oauthimpl

## Resumen

Replace oauth2 5.0.0 with a native OAuth2 Authorization Code + PKCE client in ag-auth (issue #185, RFC-0029)

## Fase afectada

Mantenimiento / higiene de dependencias (post Fase 4.6). Sin cambio de fase,
sin cambio de DSL, sin cambio de roster de crates.

## Tipo de cambio

- [x] Codigo (cliente OAuth nativo en `ag-auth`)
- [x] Documentacion (CHANGELOG, README de ag-auth, nota de seguimiento en RFC-0026)
- [x] Auditoria / evidencia de verificacion
- [x] Cambio de dependencias de Cargo (elimina `oauth2`; anade `wiremock` como dev-dependency)
- [ ] Cambio de arquitectura
- [ ] Cambio de DSL

## Documentos relacionados

- `docs/rfc/RFC-0029-oauth2-chrono-unblock.md` (diseno, seccion 4.3 contrato de compatibilidad)
- `docs/rfc/RFC-0026-chrono-replacement.md` (nota de seguimiento en 3.4)
- `CHANGELOG.md`, `crates/ag-auth/README.md`
- Issue #185 (permanece abierta: falta la migracion chrono -> time)

## Plan de prueba

Comandos ejecutados en este worktree con el target de Cargo compartido:

1. `cargo test -p ag-auth --all-features` (53 tests + 3 doc-tests, todos en verde).
2. `cargo test --workspace --all-features` (salida 0, 77 bloques `test result: ok`, 0 fallos).
3. `cargo fmt --all -- --check`
4. `cargo clippy --workspace --all-features --all-targets -- -D warnings`
5. `cargo audit` (4 avisos permitidos, incluido RUSTSEC-2026-0543; sin cambios).
6. `cargo deny --all-features check` (advisories ok, bans ok, licenses ok, sources ok;
   el ignore fechado de chrono sigue presente, a proposito).
7. `cargo tree -i oauth2` -> "package ID specification `oauth2` did not match any packages".
8. `cargo tree -i chrono` -> ya no aparece la arista `oauth2 5.0.0 -> ag-auth`.

## Criterios de salida que avanza

- RFC-0029 paso 2: eliminar el cliente oauth2 y su arista de chrono, con el
  contrato de la seccion 4.3 cubierto por tests (vector RFC 7636 Appendix B,
  golden URL, cuerpo de formulario exacto, errores del endpoint de token).
- #185 avanza: desaparece el prerrequisito bloqueante. No cierra la issue porque
  RFC-0026 todavia no ha migrado los cuatro caminos restantes a chrono.

## Cierre de issues

Refs #185
Closes: none

## Condiciones para cerrar #185 (fuera de esta PR)

1. La rama `community3/chrono` debe aterrizar: sqlx usa la feature `time`,
   ag-workers/ag-domains/examples usan `time 0.3.x`, DSL codegen actualizado y
   tests golden de serde/RFC 3339 en verde.
2. `cargo tree -i chrono` debe estar vacio en el arbol integrado.
3. El coordinador elimina el ignore `RUSTSEC-2026-0543` de `deny.toml` y
   `cargo deny --all-features check advisories` pasa sin el.
4. `cargo test --workspace --all-features` pasa en el arbol integrado.
5. Solo entonces se cierra #185 con `Closes #185` en el descriptor de integracion.

## Checklist

- [x] Pertenece a la fase correcta.
- [x] Respeta la documentacion.
- [x] No rompe arquitectura.
- [x] No anade complejidad innecesaria (cero dependencias nuevas de produccion).
- [x] No crea dependencias circulares.
- [x] Compila.
- [x] Pasa tests.
- [x] Pasa fmt.
- [x] Pasa clippy.
- [x] Pasa audit.
- [x] Pasa deny (ignore de chrono aun requerido y respetado).
- [x] Tiene documentacion.
- [x] No expone secretos en logs (Debug redactado en ambos newtypes).
- [x] Mantiene coherencia con Anti-Gravital v4.1.
