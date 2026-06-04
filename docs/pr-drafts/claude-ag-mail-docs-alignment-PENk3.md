# feat(ag-mail): MTA outbound nativo opt-in (Fase 4.6-A) + gobernanza ADR-0010/RFC-0009

## Fase afectada

Gobernanza aditiva (ADR-0010 / RFC-0009) entre Fase 4.5 (completa) y la Fase
4.6, **mas** la implementacion de la Fase 4.6-A (nucleo del MTA nativo). Todo
es aditivo y opt-in: no degrada ni cambia el baseline existente.

## Tipo de cambio

Gobernanza (`docs`) + implementacion (`feat`) aditiva tras la feature de Cargo
`mta`, apagada por defecto. Sin cambios en la API publica existente, el sender
por defecto, los adapters ni las colas.

## Documentos relacionados

- `docs/adr/0010-ag-mail-native-mta-pivot.md` — decision (supersede el alcance
  de `ag-mail` de `ADR-0007`)
- `docs/rfc/RFC-0009-ag-mail-native-mta.md` — plan tecnico (supersede
  `RFC-0006` para el alcance de `ag-mail`)
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` §8.8 y
  `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` §4.5.5
- `docs/master/VERSION.md` — hashes recalculados

## Resumen

El research report adjunto reformula `ag-mail` desde un relay outbound
transaccional ("NO es un MTA", `ADR-0007`/`RFC-0006`) hacia un MTA outbound
nativo propio, self-hosted e independiente de proveedores. Es una reversion de
alcance, no una sync de docs, y por las reglas 5, 22 y 28 del `CLAUDE.md` exige
gobernanza antes de codigo. Este PR entrega esa capa de gobernanza y deja la
documentacion coherente con el estado real (baseline implementado) y con la
direccion acordada (MTA por fases, opt-in, sin declararlo implementado).
Cualquier plataforma de correo externa citada en la investigacion es solo
referencia de ingenieria: no es dependencia ni objetivo de integracion.

**Principio aditivo-only (vinculante).** La expansion solo anade capacidad
tras features opt-in; no elimina, degrada ni cambia el comportamiento de nada
del baseline. El blueprint proponia *sobrescribir* (MTA por defecto, degradar
los adapters de proveedor a "no produccion", migrar la cola en memoria); eso
se **rechaza** en `ADR-0010`/`RFC-0009`. Permanecen sin cambios: features por
defecto (`smtp`, `templates`, `metrics`), `SmtpSender` por defecto,
`MailSender`/`AgMail`/`NullSender`, los adapters de proveedor existentes
(opcionales, off por defecto), colas en memoria y `ag-data`, templates
tipados, integracion `ag-auth` y CLI `ag mail test`.

**Gobernanza:**
- `ADR-0010`: supersede el alcance v1 "NO es un MTA / inbound nunca" de
  `ADR-0007`; expande `ag-mail` a MTA outbound nativo (resolucion MX,
  ESMTP+STARTTLS, firma DKIM, clasificacion de bounces), por fases y opt-in
  tras features de Cargo (`mta`, `api`, `queue-jetstream`), conservando el
  patron Native | Adapter, el modo nativo por defecto (`ADR-0009`) y la
  direccionalidad `ag-auth -> ag-mail`. Inbound solo como DSN/ARF para
  bounces; buzones/IMAP/POP siguen fuera de alcance.
- `RFC-0009`: 6 subsistemas, dependencias `mail-send`/`mail-builder`/
  `mail-auth`/`mail-parser`/`hickory-resolver`, cola de dos niveles, modelo de
  datos PostgreSQL, superficie REST drop-in, webhooks estilo Svix, plan por
  fases 4.6-A..D mas endurecimiento Fase 5+, riesgos y rollback.

**Alineacion documental (fiel al codigo real):**
- `docs/modules/ag-mail/README.md` reescrito: separa el baseline implementado
  (`SmtpSender` + adapters de proveedor + cola persistente + templates tipados)
  de la direccion MTA planeada; corrige el estado obsoleto "Pendiente / se
  creara".
- Maestros: nota de actualizacion de alcance en Arquitectura §8.8 (EN+ES) y
  nota futura Fase 4.6 en Hoja-de-Ruta §4.5.5 (EN+ES), con sus derivados.
- `README.md` raiz (EN+ES): el parrafo "que no es" refleja el MTA planeado
  sin declararlo implementado.
- Indices `docs/adr/README.md` y `docs/rfc/README.md` completados y con los
  estados superseded correctos; `RFC-0006` y `ADR-0007` anotados.

**Implementacion Fase 4.6-A (feature `mta`, opt-in):**
- `crates/ag-mail/src/sender/mta/`:
  - `mod.rs` — `MtaSender` (implementa `MailSender`): entrega directa al MX,
    agrupacion por dominio, MIME via `mail-builder`, firma DKIM al final.
  - `resolve.rs` — resolucion MX (`hickory-resolver`), orden por preferencia,
    rollup `site_name`, fallback MX implicito (RFC 5321).
  - `dkim.rs` — firma DKIM Ed25519 (`mail-auth`); clave aportada por el
    llamador / `ag-domains`; `Debug` redacta el material de clave.
  - `bounce.rs` — clasificador SMTP/RFC 3463 (transitorio vs permanente), puro.
- `error.rs`: variantes `Dns`, `NoMailHost`, `Dkim`.
- `Cargo.toml`: feature `mta` + deps opcionales `mail-send`/`mail-auth`/
  `hickory-resolver` (no en build por defecto).
- Documentacion fiel al codigo: `lib.rs` `//!`, crate README, modulo README
  (seccion "Implemented — Phase 4.6-A"), `docs/DEBT.md` (DEBT-012/013/014).

**Integridad de maestros:**
- `VERSION.md` y `.github/workflows/docs.yml`: SHA-256 de Hoja-de-Ruta
  recalculado tras la nota 4.5.5; entradas nuevas en el historial.

## Plan de prueba

- `cargo build --workspace` verde (lockfile unifica `hickory-resolver 0.26`
  con `ag-domains`).
- `cargo test -p ag-mail --features mta` — 58 pasan, 1 ignorado (entrega en
  vivo, requiere red/puerto 25).
- `cargo clippy -p ag-mail --features mta --all-targets -- -D warnings` y con
  features por defecto: sin warnings. `cargo fmt --check` limpio.
- `sha256sum` de los maestros coincide con `VERSION.md` y `docs.yml`
  (job `masters integrity`).
- Job `prohibited content scan`: sin emojis ni evidencia de herramientas IA.
- `cargo audit` / `cargo deny`: se ejecutan en CI (no instalados localmente).

## Criterios de salida avanzados

- Decision de pivot registrada en ADR + RFC antes del codigo (reglas 5, 22, 28).
- Fase 4.6-A: `MtaSender` entrega via MX con STARTTLS y firma DKIM Ed25519;
  clasificacion de bounces; todo tras la feature `mta` opt-in.
- Aditivo: build y comportamiento por defecto sin cambios; baseline intacto.
- Documentacion de `ag-mail` coherente entre maestros, derivados, modulo,
  crate README, `lib.rs` y `docs/DEBT.md`.
- `RFC-0006`/`ADR-0007` marcados superseded para el alcance de `ag-mail`.

## Checklist final

- [x] Pertenece a la fase correcta (gobernanza 4.6 + implementacion 4.6-A)
- [x] Respeta la documentacion (ADR + RFC antes del codigo)
- [x] No rompe arquitectura (Native | Adapter y dependencias preservadas)
- [x] No anade complejidad innecesaria (feature opt-in, deps tras `mta`)
- [x] No crea dependencias circulares (`ag-mail` sigue sin depender de `ag-auth`)
- [x] Aditivo-only: no elimina ni degrada el baseline (default, adapters, colas)
- [x] Compila (`cargo build --workspace`)
- [x] Pasa tests (`cargo test -p ag-mail --features mta`)
- [x] Pasa fmt y clippy (`-D warnings`, default y `mta`)
- [x] No declara capacidades inexistentes (regla 26); deuda en `docs/DEBT.md`
- [x] Maestros con hashes actualizados (VERSION.md + workflow)
- [x] Sin emojis ni evidencia de herramientas IA en el contenido
- [x] Mantiene coherencia con Anti-Gravital v4.0/v4.1
