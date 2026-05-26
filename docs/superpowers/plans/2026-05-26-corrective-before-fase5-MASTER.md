# Corrective-Before-Fase-5 — Plan Maestro

> **For agentic workers:** Este es el plan MAESTRO (indice + analisis + secuencia).
> Cada subsistema tiene su propio plan hijo con tareas bite-sized TDD. Ejecutar
> con superpowers:subagent-driven-development (recomendado) o
> superpowers:executing-plans, plan hijo por plan hijo, respetando el orden y las
> compuertas de RFC. Los pasos usan checkbox (`- [ ]`).

**Goal:** Consolidar las fases 0-4.5 de Anti-Gravital — reconciliar documentacion
con codigo y saldar las 4 deudas tecnicas clave — para entrar a la Fase 5
(`ag-cloud`) sin arrastrar inconsistencias.

**Architecture:** Trabajo en la rama `corrective-before-fase-5`, construida encima
del cierre documental `docs-cierre-fase-4.5` (ingles canonico + ADR-0008). Se
divide en 6 planes hijos secuenciados por riesgo y dependencias: primero la
reconciliacion documental (sin logica), luego las deudas tecnicas (codigo real,
algunas con compuerta de RFC), y por ultimo el tooling de adopcion.

**Tech Stack:** Rust (workspace 18 crates), sqlx + PostgreSQL, lettre, instant-acme,
rcgen, moka, hickory-resolver, tokio, axum, criterion, cargo-tarpaulin.

**Fuente:** Auditoria externa `~/Downloads/Auditoria de proyecto GitHub.pdf`
(fases 0-4.5), reconciliada commit a commit contra el estado real de `main`
el 2026-05-26.

---

## 0. Reconciliacion auditoria <-> estado real de `main`

La auditoria es un snapshot. Cada afirmacion se verifico contra el codigo. Resultado:

### 0.1 Confirmado (trabajo real pendiente)

| Item | Evidencia en `main` |
| --- | --- |
| README `ag-mail` desfasado | `crates/ag-mail/README.md:5` = "Fase 4.5 — skeleton (Etapa 2-1). No implementado todavia" mientras el modulo tiene sender/, queue/, template/, message.rs, metrics.rs |
| README `ag-data`/`ag-dsl`/`ag-cli` desfasados | Los tres dicen "Fase 0 - Vacio" pero tienen implementacion (sqlx pool, compilador DSL, CLI con 8 subcomandos) |
| `lib.rs` "skeleton" | `ag-mail/src/lib.rs:10-11` y `ag-domains/src/lib.rs:11` afirman skeleton/modulos vacios |
| Mismatch README/lib.rs en `ag-domains` | README dice "implementado (Etapas 2-2 a 2-4)", `lib.rs:11` dice "Skeleton (Etapa 2-1)" |
| Cola persistente `ag-mail` = stub | `crates/ag-mail/src/queue/store.rs` son 4 lineas de comentario; `ag-data` NO es dependencia en `crates/ag-mail/Cargo.toml`; feature `queue-persistent = []` vacia |
| `notAfter` no implementado | `ag-domains/src/acme/renewal.rs:96-97,117` — renueva en cada ciclo, hay TECH-DEBT explicito |
| Test de carga 50k ausente | `ag-realtime` sin `tests/` de carga ni `benches/` |
| Sin `docs/DEBT.md` | No existe |
| Sin `install.sh` | No existe en raiz |
| Sin cobertura en CI | `tarpaulin`/`codecov` ausentes en `.github/workflows/` |

### 0.2 Correcciones a la auditoria (verificadas contra codigo)

| Afirmacion de la auditoria | Realidad |
| --- | --- |
| "ag-cache implementa L2 con Redis" | FALSO. `ag-cache/src/lib.rs:75` — si `redis_url` es `Some`, solo emite warning en tracing. L2 NO funciona; fred/Redis NO cableado. RFC-0005 propone L2 nativo RESP2 (no implementado). |
| "Revisar si ag-storage tiene procesamiento de imagenes" | YA existe: `ag-storage/src/lib.rs:24` `pub use image::ImageProcessor`. |
| "Crear docs/manual" | YA existe parcialmente: `docs/manual/` con `01-shield-as-library.md`, `02-primera-api.md`, `03-dominio-tls-correo.md`, `README.md`. Falta ampliarlo, no crearlo. |

### 0.3 Contexto que la auditoria no podia conocer

- `docs-cierre-fase-4.5` (17 commits, tip `d16784b`) **no esta en main**. Aporta
  ADR-0008 (ingles canonico), traduccion ES->EN de ~95 `.rs`, maestros bilingues.
- Critico: ese cierre **solo tradujo** los comentarios skeleton al ingles; NO
  corrigio la mentira semantica. Ej: `ag-mail/src/lib.rs` en `docs-cierre` dice
  "Phase 4.5 skeleton (Stage 2-1). The public APIs are declared as empty modules".
  La reconciliacion semantica sigue siendo trabajo nuevo.

---

## 1. Prerequisito y estrategia de ramas (DECISION REQUERIDA)

El trabajo correctivo toca los mismos README y comentarios que `docs-cierre`. Para
no escribir comentarios en espanol que luego se retraducen, hay que fijar el
baseline. **Recomendacion:**

1. Re-pushear `docs-cierre-fase-4.5` a `origin`
   (`https://github.com/Anti-Gravital/Anti-Gravital.git`) y abrir su PR a `main`.
   Es un cierre documental coherente y autocontenido (ADR-0008 + ingles + maestros
   bilingues). Mergearlo primero.
2. Rebasar `corrective-before-fase-5` sobre el `main` actualizado. Asi hereda el
   ingles y ADR-0008, y todo el trabajo correctivo se escribe una sola vez en ingles.
3. Todo el trabajo de los planes hijos ocurre en `corrective-before-fase-5`, que
   produce un PR final (o varios por subsistema) a `main`.

Alternativa (si no se quiere tocar main aun): mergear `docs-cierre-fase-4.5` dentro
de `corrective-before-fase-5` y emitir un unico PR grande al final. Menos limpio de
revisar.

**Hasta que el usuario apruebe el paso 1 (accion visible en repo compartido), los
planes hijos asumen baseline = `main` + `docs-cierre` (ingles canonico).**

---

## 2. Compuertas de gobernanza (CLAUDE.md secciones 22, 28, 35)

Antes de implementar codigo que cambie arquitectura, DSL, performance targets o
dependencias de infraestructura, se requiere RFC/ADR. Estado por item:

| Item del plan | Compuerta | Estado |
| --- | --- | --- |
| Reconciliacion docs + DEBT.md + reglas CLAUDE.md | Ninguna (es gobernanza/docs) | Listo para ejecutar |
| `ag-mail` cola persistente | RFC-0006 ya define el alcance y declara `queue-persistent` | Sin RFC nuevo; dentro de alcance |
| `ag-domains` notAfter | RFC-0007 + TECH-DEBT ya documentado | Sin RFC nuevo; dentro de alcance |
| `ag-realtime` tests de carga 50k | Ninguna (es verificacion de criterio de fase) | Listo |
| `ag-cache` L2 nativo RESP2 | **RFC-0005 existe y esta detallada (protocolo, comandos, diseno), pero en estado PROPUESTA** | BLOQUEADO: aprobar RFC-0005 (no requiere redaccion, solo revision+aprobacion) antes de codificar |
| Reglas nuevas en CLAUDE.md (5 de la auditoria) | Cambio de gobernanza | Requiere ADR de gobernanza ligero |
| Adaptadores DNS extra / backend storage nativo / auth gateway / multi-tenancy | Fase 5+ o RFC propio | FUERA de alcance de este plan (defer) |

---

## 3. Secuencia y planes hijos

Orden por dependencias y riesgo (de menor a mayor). Cada plan hijo es ejecutable y
verificable por si mismo.

| # | Plan hijo | Archivo | Riesgo | Compuerta |
| --- | --- | --- | --- | --- |
| P1 | Reconciliacion documental + DEBT.md + reglas CLAUDE.md | `2026-05-26-corrective-p1-docs-reconciliation.md` | Bajo | Ninguna |
| P2 | `ag-mail` cola persistente (ag-data) + headers SMTP | `2026-05-26-corrective-p2-ag-mail-queue.md` | Medio | RFC-0006 (ok) |
| P3 | `ag-domains` notAfter + renovacion programada + metricas | `2026-05-26-corrective-p3-ag-domains-notafter.md` | Medio | RFC-0007 (ok) |
| P4 | `ag-realtime` tests de carga 50k + persistencia de eventos | `2026-05-26-corrective-p4-ag-realtime-scale.md` | Medio | Ninguna |
| P5 | `ag-cache` L2 nativo RESP2 | `2026-05-26-corrective-p5-ag-cache-resp2.md` | Alto | **RFC-0005 (completar+aprobar)** |
| P6 | Tooling de adopcion: install.sh, tarpaulin CI, E2E cross-module, manual | `2026-05-26-corrective-p6-tooling-onboarding.md` | Bajo-Medio | Ninguna |

Notas de orden:
- P1 primero: desbloquea coherencia y crea `docs/DEBT.md` que P2-P5 actualizan al cerrar cada deuda.
- P2 antes que P5: la cola persistente de `ag-mail` y el L2 de `ag-cache` ambos consumen `ag-data`/almacenamiento; P2 valida el patron de dependencia opcional bajo feature primero.
- P5 al final por riesgo y por su compuerta RFC; puede ejecutarse en paralelo a P6 una vez aprobada la RFC.

---

## 4. Resumen detallado por plan hijo

### P1 — Reconciliacion documental (sin logica)
**Alcance:** corregir la mentira semantica documentada en 0.1.
- READMEs de `ag-mail`, `ag-domains`, `ag-data`, `ag-dsl`, `ag-cli`, `ag-cache`:
  reemplazar "skeleton/vacio" por seccion **Status** (real), **Scope** (implementado)
  y **Tech Debt** (pendiente, enlazada a `docs/DEBT.md`).
- `lib.rs` `//!` de `ag-mail` y `ag-domains`: quitar "skeleton/empty modules";
  describir estado real.
- Crear `docs/DEBT.md` agregando TODA la deuda tecnica (las 4 grandes + headers
  SMTP, multi-tenancy/RLS marcadas como fase posterior, etc.) con formato CLAUDE.md
  seccion 29 (motivo, impacto, eliminacion esperada, issue, fecha objetivo).
- Anadir a `CLAUDE.md` las reglas de la auditoria (ver P1 detalle): prohibir marcar
  funcional como "skeleton/vacio"; exigir adaptadores externos detras de features
  con modo nativo; docs tecnicas en la misma PR que el codigo; scripts de instalacion
  auditados/firmados; infra externa reemplazable por nativa salvo RFC. Acompanar de
  un ADR de gobernanza.
- Alinear `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` y `docs/roadmap/STATUS.md`.
**Exit:** `grep -rniE "skeleton|sin implementar|fase 0 - vac" crates/*/README.md crates/*/src/lib.rs` no devuelve falsos negativos; `docs/DEBT.md` existe y enlaza cada deuda; CLAUDE.md con reglas nuevas; `cargo build` ok (comentarios no afectan).

### P2 — ag-mail cola persistente
**Alcance:** implementar `queue-persistent` apoyado en `ag-data`.
- Anadir `ag-data` como dependencia opcional bajo feature `queue-persistent` en
  `crates/ag-mail/Cargo.toml`.
- `crates/ag-mail/src/queue/store.rs`: implementar `PersistentQueue` con tabla
  PostgreSQL (id, payload, estado enum {pending,sending,sent,failed}, intentos,
  next_retry_at, created_at, updated_at, metadata JSONB), migracion sqlx embebida,
  reintentos con backoff persistido, recuperacion tras reinicio.
- Trait comun de cola para que memoria y persistente sean intercambiables.
- Headers SMTP personalizados (`ag-mail/src/sender/smtp.rs`): implementar via API de
  lettre si la version lo permite; si no, documentar limitacion en DEBT.md y abrir issue upstream.
- Tests: unit de transiciones de estado, integration con Postgres (testcontainers o
  `#[ignore]` si CI no tiene DB), recuperacion tras reinicio.
- Metricas `ag-observe`: tiempo en cola, profundidad de cola.
**Exit:** feature `queue-persistent` compila y pasa tests; un mensaje sobrevive reinicio simulado; DEBT.md actualizado (deuda cerrada).

### P3 — ag-domains notAfter
**Alcance:** parsear `notAfter` del certificado emitido y programar renovacion real.
- `crates/ag-domains/src/acme/renewal.rs`: parsear la fecha `notAfter` del PEM del
  cert (via `x509-parser` o `rustls-pki-types` + der; elegir el mas ligero ya presente
  o justificar la dep en DEBT/RFC-0007). Calcular `check_interval` real (renovar
  cuando falten `renew_before_days`), reemplazando la cota conservadora del TECH-DEBT
  en `renewal.rs:117`.
- Metrica de expiracion proxima (`ag-observe`): dias hasta vencimiento + alerta.
- CLI `ag domains`: subcomando para listar certs y simular (`--dry-run`) renovacion.
- Tests: parseo de un cert de prueba con notAfter conocido; calculo de intervalo.
**Exit:** renovacion programada por fecha real; metrica de expiracion expuesta; TECH-DEBT removido de `renewal.rs`; DEBT.md actualizado.

### P4 — ag-realtime escalabilidad
**Alcance:** demostrar el criterio de fase 4 (50k conexiones) y persistencia opcional.
- `crates/ag-realtime/tests/load_50k.rs` (o `benches/`): test de carga que abre
  ~50.000 suscriptores al bus interno y mide throughput/latencia. Marcado `#[ignore]`
  por defecto (recurso), ejecutable en gate manual; documentar hardware/metodologia
  segun CLAUDE.md seccion 17.
- Buffer de eventos opcional en disco/DB para no perder eventos criticos al reiniciar
  (feature `event-persistence`, analogo al L2 de cache). Diseno minimo; si crece,
  abrir RFC.
- Documentar patrones pub/sub, fallback NATS->bus interno, y resultados de carga en
  `docs/modules/ag-realtime/` y `docs/benchmarks/`.
**Exit:** test de carga reproducible con resultados documentados; doc de patrones; DEBT.md actualizado.

### P5 — ag-cache L2 nativo RESP2 (COMPUERTA RFC-0005)
**Alcance:** reemplazar el L2 Redis (hoy stub) por implementacion nativa RESP2.
- **Paso 0 (bloqueante):** revisar y APROBAR `docs/rfc/RFC-0005-ag-cache-native-l2.md`.
  La RFC ya contiene el diseno completo (protocolo RESP2, tabla de comandos, esbozo de
  `NativeCacheServer`, config, limitaciones); solo esta en estado "Propuesto". No
  requiere redaccion, solo revision tecnica + aprobacion. Sin aprobacion NO se escribe
  codigo (CLAUDE.md secciones 5 y 22).
- Implementacion (tareas detalladas se escriben tras aprobar la RFC): servidor RESP2,
  cliente L2, integracion con `L1Cache`, TTL e invalidacion por tags, snapshots.
- Quitar/aislar la dependencia de Redis/fred tras feature de compatibilidad.
- Tests de concurrencia y de coherencia L1/L2.
**Exit:** RFC-0005 aprobada; L2 nativo funcional sin Redis por defecto; tests de concurrencia verdes; DEBT.md actualizado.

### P6 — Tooling de adopcion
**Alcance:** reducir friccion de adopcion (recomendaciones generales de la auditoria).
- `install.sh` multiplataforma (Linux/macOS) + `install.ps1` (Windows PowerShell):
  detecta/instala Rust, configura PostgreSQL opcional, compila el workspace, instala
  el binario `ag` en PATH. Con verificacion de firma/integridad (regla nueva CLAUDE.md).
- `cargo-tarpaulin` en CI (`.github/workflows/quality.yml`) con umbral >=80% por crate
  (criterio de la hoja de ruta).
- Tests E2E cross-module en CI: `ag-auth` enviando correo via `ag-mail` con
  `ag-domains` (SPF/DKIM/DMARC) — extender `tests/integration/`.
- CLI: prompts interactivos en `ag new` (plantilla, DB, correo, dominios) estilo
  cargo-generate; actualizar README del CLI con cada subcomando y variables de entorno
  (`AG_CLOUDFLARE_TOKEN`, `AG_SMTP_HOST`, etc.).
- Ampliar `docs/manual/` con guia end-to-end y troubleshooting; enlazar desde README raiz.
**Exit:** `install.sh`/`install.ps1` probados; cobertura en CI con umbral; E2E cross-module verde; manual ampliado y enlazado.

---

## 5. Criterio global de "listo para Fase 5" (de la conclusion de la auditoria)

- [ ] Todos los README reflejan estado real, alcance y pendientes (P1).
- [ ] `docs/DEBT.md` centraliza la deuda tecnica (P1, actualizado por P2-P5).
- [ ] Cola persistente `ag-mail` implementada (P2).
- [ ] L2 de `ag-cache` sin dependencia obligatoria de Redis (P5).
- [ ] Renovacion `notAfter` en `ag-domains` (P3).
- [ ] Tests de escalabilidad `ag-realtime` documentados (P4).
- [ ] Documentacion/manuales actualizados + instalador unificado (P6).
- [ ] `cargo fmt` / `clippy -D warnings` / `test --workspace` / `audit` / `deny` limpios.
- [ ] Reglas de gobernanza nuevas en CLAUDE.md + ADR (P1).

---

## 6. Riesgos

- **Conflictos con docs-cierre:** mitigado fijando baseline (seccion 1) antes de empezar.
- **RFC-0005 sin aprobar bloquea P5:** P5 puede no entrar en este ciclo si la RFC no
  se aprueba; el resto del plan no depende de P5.
- **Tests con PostgreSQL/red en CI:** usar `#[ignore]` + gate manual donde no haya DB,
  documentando como reproducir (CLAUDE.md seccion 36).
- **Scope creep hacia fase 5:** los items de seccion 2 marcados "defer" NO se
  implementan aqui; se registran en DEBT.md.

---

## 7. Handoff de ejecucion

Tras aprobar este maestro y el baseline (seccion 1), se generan/ejecutan los planes
hijos P1..P6. Recomendado: subagente fresco por plan hijo con revision entre planes
(superpowers:subagent-driven-development). Sonnet 4.6 codifica cada plan hijo.
