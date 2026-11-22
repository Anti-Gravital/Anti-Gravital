# RFC-0029: unblock issue #185 with a native OAuth2 Authorization Code + PKCE client (design only)

## Resumen

Add RFC-0029: replace oauth2 5.0.0 with a small native PKCE client in ag-auth; documentation only, issue #185 stays open.

## Fase afectada

Mantenimiento / higiene de dependencias (post Fase 4.6). No cambia fases, roadmap ni el roster de crates.

## Tipo de cambio

- [x] RFC nueva (diseno de decision de dependencias, sin implementacion)
- [x] Documentacion (docs/rfc/RFC-0029, indice de RFC, CHANGELOG)
- [x] Evaluacion con evidencia de crates.io, manifiestos publicados y upstream
- [ ] Codigo de produccion (no autorizado por esta RFC)
- [ ] Manifiestos Cargo o Cargo.lock (sin cambios)
- [ ] Cambio de API publica (solo se propone para un PR posterior)
- [ ] Cambio de DSL, maestros, ADR o diagramas

## Documentos relacionados

- Issue #185 (chaos: eliminar chrono) y sus tres comentarios de 2026-11-22
- RFC-0026 (sustitucion de chrono, diferida por este bloqueo)
- RFC-0025 y `docs/modules/ag-data/data-layer-rls-tenancy.md` (decision de tipo de fecha/hora)
- ADR-0008 (ingles canonico), ADR-0018 (cliente HTTP aportado por el llamador)
- RUSTSEC-2026-0543
- crates.io oauth2 5.0.0; upstream ramosbugs/oauth2-rs issues #351 y PR #355

## Cambios concretos

- `docs/rfc/RFC-0029-oauth2-chrono-unblock.md`: nueva RFC con opciones evaluadas,
  recomendacion (cliente nativo), contrato de compatibilidad (token endpoint,
  PKCE S256, state CSRF, refresh), pasos de migracion, riesgos, impacto,
  rollback y decision pendiente.
- `docs/rfc/README.md`: se anade la fila que faltaba de RFC-0026 y la de RFC-0029.
- `CHANGELOG.md`: entrada bajo `[Unreleased]` indicando que es solo diseno y que
  #185 permanece abierta.
- `docs/pr-drafts/community2-oauth.md`: este descriptor.

Sin cambios en `crates/`, manifiestos, `Cargo.lock`, `deny.toml` ni codigo de OAuth.

## Evidencia principal

- `oauth2 5.0.0` declara chrono 0.4.31 no opcional (`default-features = false`
  no lo elimina); es la version mas reciente segun crates.io.
- La rama `main` de upstream (oauth2 5.1.0) sigue dependiendo de chrono; PR #355
  esta abierta sin merge y el mantenedor confirmo que implica un bump mayor.
- `ag-auth` solo usa oauth2 para PKCE, state y construccion de URLs; el token
  exchange y el userinfo ya son reqwest directo. No hay refresh de proveedor.
- Ningun reemplazo libre evaluado cumple: openidconnect y oauth2-reqwest arrastran
  chrono; oauth-client es passively-maintained y sin PKCE; oauth2-client 0.2.0
  no se actualiza desde 2023; oidc-rs/axum-login/yup-oauth2 no encajan.

## Plan de prueba

```
git diff --check
cargo fmt --all -- --check
grep -R -nE 'claude-|gpt-|copilot-|ai-' docs/rfc/RFC-0029-oauth2-chrono-unblock.md docs/pr-drafts/community2-oauth.md
```

Verificacion local realizada en la rama: `git diff --check` limpio y
`cargo fmt --all -- --check` limpio. No se ejecuta `cargo test` porque el cambio
no toca codigo; la suite completa se ejecutara en el PR de implementacion.

## Criterios de salida que avanza

- Issue #185, paso 1 (evaluar reemplazos): AVANZA con evidencia verificable.
- RFC-0026, seccion 3.4 (bloqueo oauth2): AVANZA con una decision propuesta.
- Issue #185, "Done when" (sin crono en el workspace, deny sin ignore, tests):
  NO CUMPLIDO; requiere ademas ejecutar RFC-0026 completa.

## Bloqueo documentado (por que no se cierra #185)

Esta RFC elige y documenta el camino, pero no cambia codigo ni manifiestos.
`cargo tree -i chrono` sigue mostrando oauth2 5.0.0 y las aristas de
ag-domains, ag-workers y sqlx. El ignore de RUSTSEC-2026-0543 permanece.
#185 solo se cierra cuando todas las aristas desaparezcan y
`cargo deny --all-features check` pase sin el ignore.

## Cierre de issues

Refs #185
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (mantenimiento post 4.6)
- [x] Respeta la documentacion (RFC-0026 y ADR-0008/0018)
- [x] No anade complejidad ni dependencias nuevas; recomendacion deliberadamente minima
- [x] No crea dependencias circulares ni cambia el roster de crates
- [x] No modifica codigo de produccion; fmt y diff check en verde
- [x] Documentacion actualizada en el mismo commit (RFC, indice, CHANGELOG)
- [x] No se alega cierre de #185: se usa `Refs #185`
- [x] Sin emojis, sin atribucion de herramientas IA, mensajes <=256 caracteres
