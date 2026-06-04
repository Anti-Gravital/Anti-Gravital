# docs(ag-mail): pivot a MTA outbound nativo (ADR-0010, RFC-0009) y alineacion documental

## Fase afectada

Gobernanza aditiva entre Fase 4.5 (completa) y la nueva Fase 4.6 futura.
No introduce codigo funcional: registra la decision de expandir `ag-mail` a
un MTA outbound nativo y alinea la documentacion afectada. La implementacion
queda bloqueada hasta que este conjunto documental este mergeado (regla 27).

## Tipo de cambio

Documentacion y gobernanza (`docs`). Sin cambios de codigo, API ni
comportamiento.

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
transaccional ("NO es un MTA", `ADR-0007`/`RFC-0006`) hacia una replica
nativa de Resend con MTA propio. Es una reversion de alcance, no una sync de
docs, y por las reglas 5, 22 y 28 del `CLAUDE.md` exige gobernanza antes de
codigo. Este PR entrega esa capa de gobernanza y deja la documentacion
coherente con el estado real (baseline implementado) y con la direccion
acordada (MTA por fases, opt-in, sin declararlo implementado).

**Principio aditivo-only (vinculante).** La expansion solo anade capacidad
tras features opt-in; no elimina, degrada ni cambia el comportamiento de nada
del baseline. El blueprint proponia *sobrescribir* (MTA por defecto, Resend a
feature "no produccion", migrar la cola en memoria); eso se **rechaza** en
`ADR-0010`/`RFC-0009`. Permanecen sin cambios: features por defecto (`smtp`,
`templates`, `metrics`), `SmtpSender` por defecto, `MailSender`/`AgMail`/
`NullSender`, adapters Resend/SES/Postmark, colas en memoria y `ag-data`,
templates tipados, integracion `ag-auth` y CLI `ag mail test`.

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
  (SmtpSender + Resend/SES/Postmark + cola persistente + templates tipados)
  de la direccion MTA planeada; corrige el estado obsoleto "Pendiente / se
  creara".
- Maestros: nota de actualizacion de alcance en Arquitectura §8.8 (EN+ES) y
  nota futura Fase 4.6 en Hoja-de-Ruta §4.5.5 (EN+ES), con sus derivados.
- `README.md` raiz (EN+ES): el parrafo "que no es" refleja el MTA planeado
  sin declararlo implementado.
- Indices `docs/adr/README.md` y `docs/rfc/README.md` completados y con los
  estados superseded correctos; `RFC-0006` y `ADR-0007` anotados.

**Integridad de maestros:**
- `VERSION.md` y `.github/workflows/docs.yml`: SHA-256 de Arquitectura y
  Hoja-de-Ruta recalculados; entrada nueva en el historial de revisiones.

## Plan de prueba

- `sha256sum` de los dos maestros coincide con `VERSION.md` y con
  `.github/workflows/docs.yml` (job `masters integrity`).
- Job `prohibited content scan`: sin emojis ni evidencia de herramientas IA en
  el contenido.
- Revision manual: ningun documento declara el MTA como implementado (regla
  26); el baseline descrito coincide con `crates/ag-mail` real.
- Sin cambios de codigo: `cargo build/test` no afectados.

## Criterios de salida avanzados

- Decision de pivot registrada en ADR + RFC antes de cualquier codigo (reglas
  5, 22, 28).
- Documentacion de `ag-mail` coherente entre maestros, derivados, modulo y
  README raiz.
- `RFC-0006`/`ADR-0007` correctamente marcados superseded para el alcance de
  `ag-mail`, sin perder su vigencia para `ag-domains`.
- Repositorio listo para abrir la Fase 4.6-A (PoC del MTA) en una rama
  posterior.

## Checklist final

- [x] Pertenece a la fase correcta (gobernanza previa a Fase 4.6)
- [x] Respeta la documentacion (ADR + RFC para el cambio de alcance)
- [x] No rompe arquitectura (Native | Adapter y dependencias preservadas)
- [x] No anade complejidad innecesaria (sin codigo; features opt-in en el plan)
- [x] No crea dependencias circulares (`ag-mail` sigue sin depender de `ag-auth`)
- [x] Aditivo-only: no elimina ni degrada el baseline (default, adapters, colas)
- [x] No declara capacidades inexistentes (regla 26)
- [x] Maestros con hashes actualizados (VERSION.md + workflow)
- [x] Sin emojis ni evidencia de herramientas IA en el contenido
- [x] Mantiene coherencia con Anti-Gravital v4.0/v4.1
