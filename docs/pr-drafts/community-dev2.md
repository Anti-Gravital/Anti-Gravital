# Upgrade the mail protocol stack in ag-mail native MTA (issue #178, partial)

## Resumen

Upgrade ag-mail MTA deps to mail-send 0.7, mail-auth 0.15 and mail-parser 1.0 with explicit pure-Rust DKIM backend; independent verifier gate remains open.

## Fase afectada

Fase 4.6 (ag-mail native MTA, RFC-0009 / ADR-0010). No cambia el alcance ni el roadmap; es una actualizacion de dependencias del modulo MTA ya implementado.

## Tipo de cambio

- [x] Dependency upgrade (coordinated, three crates behind `mta`)
- [x] Bug-fix / compatibility adaptation for changed crate APIs
- [x] Test anadido (cross-version DKIM known-answer vector)
- [x] Documentacion (CHANGELOG.md)
- [ ] RFC nuevo (no requerido: la decision ya esta registrada en la issue #178 y revisada en `docs/audits/low-level-review-178-180.md` en la rama del revisor)

## Documentos relacionados

- Issue #178 (principal, permanece abierta con `Refs`)
- RFC-0009 (ag-mail native MTA), ADR-0010 (pivote MTA)
- ADR-0008 (ingles canonico en codigo/documentacion tecnica)
- issue #181 (instalacion del provider ring en `MtaSender::new`, conservada)
- Revision de bajo nivel: `docs/audits/low-level-review-178-180.md` (rama `community/bajonivel`)

## Cambios concretos

- `crates/ag-mail/Cargo.toml`: `mail-send` 0.7 (`builder`, `ring`, `tls12`), `mail-auth` 0.15 (`default-features = false`, `dns-hickory`, `rust-crypto`), `mail-parser` 1.0. `mail-send/dkim` sigue desactivado.
- `crates/ag-mail/src/sender/mta/dkim.rs`: imports a `mail_auth::crypto` / `mail_auth::headers`, canonicalizacion relaxed/relaxed fijada explicitamente y test known-answer (firma mail-auth 0.9 verificada por el verificador 0.15, y firma nueva verificada por el mismo verificador).
- `crates/ag-mail/src/sender/mta/dsn.rs`: API de mail-parser 1.0 (`message.parts()`, `part.decoded()`; el trait `MimeHeaders` desaparecio).
- `crates/ag-mail/src/sender/mta/mod.rs`: import del test de firma en `mail_auth::crypto`.
- `Cargo.lock`: resuelve mail-auth 0.15.1, mail-builder 2.0.1, mail-parser 1.0.1, mail-send 0.7.0.
- `CHANGELOG.md`: entrada bajo `[Unreleased]` con el estado parcial y los dos huecos pendientes.

## Plan de prueba

```
cargo test -p ag-mail --all-features
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy -p ag-mail --all-features --all-targets -- -D warnings
cargo audit
cargo deny --all-features check
```

Evidencia local (2026-11-21): 129 passed / 0 failed en la suite de `ag-mail` (1 ignored de DNS en vivo); workspace completo en verde; fmt limpio; clippy sin warnings; audit con 4 warnings permitidos y exit 0; deny `advisories ok, bans ok, licenses ok, sources ok` y exit 0.

## Criterios de salida que avanza

- Issue #178, "Done when" 1 (`cargo test -p ag-mail --all-features` en las versiones nuevas): CUMPLIDO.
- Issue #178, plan "Migrar call sites de DKIM y parseo": CUMPLIDO.
- Issue #178, plan "Mantener los tests sink ESMTP/STARTTLS/DKIM en verde y anadir un known-answer": CUMPLIDO.
- Issue #178, "Done when" 2 (verificador independiente): NO CUMPLIDO, bloqueado (ver abajo).
- Gate manual de buzon real / Mail A: NO CUMPLIDO (port 25 bloqueado en este entorno).

## Bloqueo documentado (por que no se cierra #178)

No existe un verificador DKIM independiente utilizable aqui:

- `dkimpy` / `opendkim-testmsg` no estan instalados y no hay `pip`.
- `mailrs-dkim` 3.2 (Apache/MIT) rechaza TANTO la firma generada por mail-auth 0.9 como la generada por 0.15: calcula el hash de las cabeceras firmadas en el orden literal de la etiqueta `h=` (bottom-up), mientras mail-auth y los servidores de correo desplegados hashean en orden de mensaje (top-down). Probado con el vector fijo: `MAILRS_NEW=Fail MAILRS_OLD=Fail`. No se anadio como dependencia para no dejar codigo roto.
- `viadkim` 0.2 es GPL-3.0-or-later y `cargo deny` (solo licencias Apache/MIT/BSD/permisivas) lo rechazaria.
- Las aserciones de substrings de los tests antiguos no son un verificador.
- `mail-auth` 0.15 es el unico verificador disponible; el test known-answer demuestra que la firma 0.9 sigue verificando con 0.15, pero el revisor senala correctamente que ambas versiones comparten linaje y no cuentan como evidencia independiente.

Siguiente paso propuesto: ejecutar la firma del vector fijo por un verificador externo (dkimpy/OpenDKIM en un entorno con herramientas) y registrar el resultado en la issue, o abrir un issue separado para incorporar un verificador independiente compatible con el orden de cabeceras de mensaje.

## Cierre de issues

Refs #178
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (4.6, modulo MTA existente)
- [x] Respeta la documentacion (RFC-0009 / ADR-0010, sin cambio de alcance)
- [x] No anade dependencias de runtime nuevas; backend cripto consistente con ring/TLS
- [x] Compila, pasa tests, fmt, clippy, audit y deny
- [x] Documentacion actualizada en el mismo commit (CHANGELOG)
- [x] No se alega cierre de #178: quedan el verificador independiente y el gate de buzon real
- [x] Sin emojis, sin atribucion de herramientas IA, mensaje <=256 caracteres
