# feat(ag-mail): MTA outbound nativo (Fases 4.6-A/B/C) + retiro de marcas (ADR-0011)

## Fase afectada

Gobernanza (ADR-0010/RFC-0009 y ADR-0011/RFC-0010) entre Fase 4.5 y Fase 4.6,
**mas** la implementacion del MTA outbound nativo (Fases 4.6-A/B y parte de
4.6-C), el motor de plantillas `minijinja`, y el retiro de los adaptadores de
correo con nombre de marca comercial.

## Tipo de cambio

Gobernanza (`docs`) + implementacion (`feat`), todo **aditivo** tras features
opt-in apagadas por defecto (`mta`, `api`, `minijinja`): el build y el
comportamiento por defecto no cambian. El retiro de los adaptadores de marca es
un cambio de superficie en `ag-mail` (sin releases; `ag-auth` es trait-based).

## Resumen de lo implementado (cerrado en `docs/DEBT.md`)

- **Gobernanza**: ADR-0010 (pivot a MTA nativo), RFC-0009 (plan), ADR-0011 +
  RFC-0010 (politica de marcas), con scan de marcas en CI.
- **MTA nativo (feature `mta`)**: resolucion MX + `site_name` rollup; egress
  pools (SWRR para IP warming); DKIM Ed25519 **y RSA-SHA256**; clasificacion de
  bounces; traffic shaping (rate token-bucket + cap de conexiones); cola de dos
  niveles con retry/backoff/max-age/`max_ready`, worker y `DeliveryBackend`;
  suppression list automatica; intake asincrono DSN (RFC 3464)/ARF (RFC 5965);
  metricas y job de CI `mail-mta`. (DEBT-017/018/019/020.)
- **API (feature `api`)**: webhooks firmados HMAC-SHA256 (`whsec_`, `v1,`,
  multi-firma, anti-replay, verify en tiempo constante). (Avanza DEBT-021.)
- **Plantillas (feature `minijinja`)**: `MinijinjaTemplate` via trait
  `MailTemplate` (loops/condicionales/filtros). (DEBT-003.)
- **CI**: `cargo doc` y cobertura tarpaulin (>=80%) verdes; modulos opt-in
  excluidos del gate de cobertura por defecto (se prueban en `mail-mta`).

## Pendiente (deuda tecnica, marcada en `docs/DEBT.md`)

Trabajo no implementado todavia, no bloqueado por el entorno. PostgreSQL,
NATS/JetStream auto-alojado y un sink SMTP local son levantables como servicios
efimeros aqui y en CI; el unico limite real es la entrega a un MX publico por el
puerto 25 (bloqueado en sandbox/CI, queda como gate manual).

- **DEBT-021** (resto): rutas REST + modelo de datos PostgreSQL + marketing.
  Probable contra un PostgreSQL local/efimero.
- **DEBT-022**: test de entrega en vivo. El camino de protocolo
  (ESMTP/STARTTLS/DKIM) es probable contra un sink SMTP local; la entrega real a
  un MX externo por puerto 25 queda como gate manual.
- **DEBT-023**: spool durable del queue (JetStream/PostgreSQL). Probable contra
  PostgreSQL local o un NATS/JetStream auto-alojado.

## Politica de marcas (ADR-0011)

No se usan marcas comerciales de terceros para nombrar componentes propios. Los
adaptadores de correo con nombre de marca se retiran: para usar un proveedor
externo se apunta el `SmtpSender` nativo a su endpoint SMTP, y la via sin
terceros es el `MtaSender` nativo. Se conservan `CloudflareProvider`
(`ag-domains`) y `S3Store` (`ag-storage`) como etiquetas legitimas de adaptador
para ese tercero (sin equivalente nativo). El DSL `mail.provider` acepta solo
`smtp`/`mta`. Un job de CI verifica la ausencia de las marcas retiradas.

## Documentos relacionados

- `docs/adr/0010-ag-mail-native-mta-pivot.md`, `docs/rfc/RFC-0009-ag-mail-native-mta.md`
- `docs/adr/0011-politica-marcas-comerciales.md`,
  `docs/rfc/RFC-0010-ag-mail-superficie-sin-marcas.md`
- `CLAUDE.md` (politica de marcas), `.github/workflows/docs.yml` (scan)
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
  `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/master/VERSION.md` (hashes)

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
  datos PostgreSQL, superficie REST drop-in, webhooks firmados HMAC-SHA256, plan por
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

- `cargo build --workspace` verde.
- `cargo test -p ag-mail --features mta` (58, 1 ignorado), `ag-dsl` (155, con
  tests de regresion: `provider resend` rechazado, `provider mta` valido),
  `ag-lsp` (15), `ag-cli`, `ag-auth` (32): verdes.
- `cargo clippy --all-targets -- -D warnings` (default y `mta`) sin warnings;
  `cargo fmt --check` limpio.
- `sha256sum` de los maestros coincide con `VERSION.md` y `docs.yml`.
- `prohibited content scan`: sin emojis, sin evidencia IA, sin marcas
  comerciales retiradas (nuevo step ADR-0011).
- `cargo audit` / `cargo deny`: se ejecutan en CI (no instalados localmente).

## Criterios de salida avanzados

- Decision de pivot registrada en ADR + RFC antes del codigo (reglas 5, 22, 28).
- Fase 4.6-A: `MtaSender` entrega via MX con STARTTLS y firma DKIM Ed25519;
  clasificacion de bounces; todo tras la feature `mta` opt-in.
- MTA aditivo: build y comportamiento por defecto sin cambios.
- Sin marcas comerciales de terceros en `ag-mail` (codigo, features, docs,
  comentarios); politica fijada en `CLAUDE.md`/`ADR-0011` y verificada en CI.
- Documentacion de `ag-mail` coherente entre maestros, derivados, modulo,
  crate README, `lib.rs`, `ag-lsp` y `docs/DEBT.md`.
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
