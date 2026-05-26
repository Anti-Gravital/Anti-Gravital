# P1 — Reconciliacion documental + DEBT.md + reglas CLAUDE.md

> **For agentic workers:** Plan hijo de `2026-05-26-corrective-before-fase5-MASTER.md`.
> Ejecutar con superpowers:subagent-driven-development o executing-plans, tarea por
> tarea. Pasos con checkbox (`- [ ]`). NO hay cambios de logica: la verificacion es
> `grep` + `cargo build` (los comentarios y READMEs no afectan compilacion).
> Antes de editar cualquier archivo, leerlo completo (el contenido "antes" mostrado
> aqui es del 2026-05-26 y puede haber cambiado si docs-cierre se mergeo primero).

**Goal:** Eliminar la desalineacion documentacion<->codigo (hallazgo #1 de la
auditoria): READMEs y comentarios que dicen "skeleton"/"vacio" en modulos ya
implementados, centralizar la deuda tecnica en `docs/DEBT.md` y anadir las reglas
de gobernanza que la auditoria recomienda.

**Architecture:** Cada README de crate funcional adopta tres secciones canonicas:
**Status** (estado real), **Scope / Implemented** (lo que ya hace) y **Tech Debt**
(pendiente, enlazado a `docs/DEBT.md`). Las cabeceras `//!` de `lib.rs` describen
el estado real, no "skeleton". `docs/DEBT.md` es la unica fuente de verdad de la
deuda. CLAUDE.md gana 5 reglas nuevas respaldadas por un ADR.

**Tech Stack:** Markdown, Rust doc-comments, `cargo build`, `grep`.

**Baseline:** `corrective-before-fase-5` sobre `main` + `docs-cierre-fase-4.5`
(ingles canonico). Escribir todo el texto nuevo en **ingles** (ADR-0008).

---

## Mapa de archivos

- Modify: `crates/ag-mail/README.md`, `crates/ag-mail/src/lib.rs`
- Modify: `crates/ag-domains/README.md`, `crates/ag-domains/src/lib.rs`
- Modify: `crates/ag-data/README.md`, `crates/ag-dsl/README.md`, `crates/ag-cli/README.md`
- Modify: `crates/ag-cache/README.md`
- Create: `docs/DEBT.md`
- Modify: `CLAUDE.md`
- Create: `docs/adr/0009-gobernanza-correctiva.md`
- Modify: `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/roadmap/STATUS.md`

---

## Task 1: Reconciliar `ag-mail` README + lib.rs

**Files:**
- Modify: `crates/ag-mail/README.md:5` (linea de Status) y `:13` (askama -> StringTemplate)
- Modify: `crates/ag-mail/src/lib.rs:6-12` (cabecera `//! # Status`)

- [ ] **Step 1: Reemplazar la linea de Status del README**

Cambiar la linea 5 (`Estado: **Fase 4.5 — skeleton (Etapa 2-1)**. No implementado todavia.`) por:

```markdown
Status: **Phase 4.5 — implemented.** Native SMTP sender, Resend/SES/Postmark
adapters, in-memory retry queue, string templating and `ag-observe` metrics are
functional. Pending tech debt (persistent queue, custom SMTP headers, external
template engines) is tracked in `docs/DEBT.md`.
Decision: `docs/adr/0007-ag-mail-ag-domains.md`. Technical plan:
`docs/rfc/RFC-0006-ag-mail-alcance.md`. Module sheet: `docs/modules/ag-mail/README.md`.
```

- [ ] **Step 2: Corregir la mencion a askama (templating real)**

En la seccion "Alcance v1" / Cargo features, reemplazar las menciones a `askama`
por la realidad. Cambiar la linea 13-14 ("Templates HTML/plaintext con askama
validados en compile-time contra `schema.ag`.") por:

```markdown
- HTML/plaintext templates via the built-in `StringTemplate` engine. Compile-time
  validation against `schema.ag` and external engines (askama/minijinja) are tracked
  as tech debt in `docs/DEBT.md`.
```

Y en "Features de Cargo" (linea ~34) cambiar `templates (default): renderizado askama.`
por `templates (default): built-in StringTemplate rendering.`

- [ ] **Step 3: Reescribir la cabecera de estado en lib.rs**

Leer `crates/ag-mail/src/lib.rs`. Reemplazar el bloque `//! # Status` (que dice
"Phase 4.5 skeleton (Stage 2-1). The public APIs are declared as empty modules...")
por:

```rust
//! # Status
//!
//! Phase 4.5 — implemented. Public API: [`MailSender`] trait with `SmtpSender`
//! (default) and `ResendSender`/`SesSender`/`PostmarkSender` adapters, the
//! [`Email`]/`EmailBuilder` model, an in-memory retry queue and `ag-observe`
//! metrics. Outstanding tech debt (persistent queue, custom SMTP headers,
//! external template engines) is tracked in `docs/DEBT.md`. Governing decision:
//! `ADR-0007`; technical plan: `RFC-0006`.
```

- [ ] **Step 4: Verificar que no quedan marcas de skeleton en ag-mail**

Run: `grep -rniE "skeleton|no implementado|empty modules|sin implementar" crates/ag-mail/README.md crates/ag-mail/src/lib.rs`
Expected: sin coincidencias (exit 1).

- [ ] **Step 5: Build + commit**

Run: `cargo build -p ag-mail`
Expected: compila.

```bash
git add crates/ag-mail/README.md crates/ag-mail/src/lib.rs
git commit -m "docs(ag-mail): reconcile README and lib.rs status with real implementation"
```

---

## Task 2: Reconciliar `ag-domains` README + lib.rs

**Files:**
- Modify: `crates/ag-domains/src/lib.rs` (cabecera `//! # Status`)
- Modify: `crates/ag-domains/README.md` (alinear estado y enlazar deuda)

- [ ] **Step 1: Reescribir la cabecera de estado en lib.rs**

Leer `crates/ag-domains/src/lib.rs`. Reemplazar el bloque `//! # Status` (que dice
"Phase 4.5 skeleton (Stage 2-1)...") por:

```rust
//! # Status
//!
//! Phase 4.5 — implemented. `DnsProvider` trait with a Cloudflare adapter, the
//! declarative A/AAAA/CNAME/TXT/MX record model, SPF/DKIM/DMARC generation for
//! `ag-mail`, an ACME client (Let's Encrypt) for issuance/renewal, and DNS
//! propagation checks against public resolvers. Outstanding tech debt
//! (`notAfter` parsing for scheduled renewal, extra DNS adapters) is tracked in
//! `docs/DEBT.md`. Governing decision: `ADR-0007`; technical plan: `RFC-0007`.
```

- [ ] **Step 2: Anadir seccion Tech Debt al README**

Leer `crates/ag-domains/README.md`. Confirmar que la linea de Status dice
"implementado" (ya correcta). Anadir al final una seccion:

```markdown
## Tech Debt

- `notAfter` parsing for date-based certificate renewal (currently renews every
  cycle). Tracked in `docs/DEBT.md`.
- Additional DNS provider adapters (Namecheap, Route 53, Google Domains) — optional,
  not required. Tracked in `docs/DEBT.md`.
```

- [ ] **Step 3: Verificar**

Run: `grep -rniE "skeleton|empty modules" crates/ag-domains/src/lib.rs`
Expected: sin coincidencias (exit 1).

- [ ] **Step 4: Build + commit**

Run: `cargo build -p ag-domains`

```bash
git add crates/ag-domains/README.md crates/ag-domains/src/lib.rs
git commit -m "docs(ag-domains): align lib.rs status with README and list tech debt"
```

---

## Task 3: Reconciliar READMEs "Fase 0 - Vacio" (`ag-data`, `ag-dsl`, `ag-cli`)

Estos tres tienen implementacion real pero el README dice "Fase 0 - Vacio".

**Files:**
- Modify: `crates/ag-data/README.md:3`
- Modify: `crates/ag-dsl/README.md:3`
- Modify: `crates/ag-cli/README.md:3`

- [ ] **Step 1: ag-data — corregir Status**

Reemplazar la linea 3 (`> Estado: Fase 0 - Vacio. La implementacion comienza en Fase 2...`) por:

```markdown
> Status: Phase 2 — implemented (base layer). PostgreSQL connection pool via sqlx
> (`DataConfig`, pool, URL sanitization) and embedded migrations (`sqlx::migrate!`).
> Pending: DSL-generated typed ORM (Phase 3), row-level security and multi-tenancy
> (later phases). See `docs/DEBT.md`.
```

- [ ] **Step 2: ag-dsl — corregir Status**

Reemplazar la linea 3 por:

```markdown
> Status: Phase 3 — implemented (alpha v0.1..v0.7). Functional compiler: logos lexer,
> chumsky parser, type/semantic checks, readable diagnostics, and codegen to Rust,
> SQL, OpenAPI and TypeScript. Syntax may change between v0.x releases. See `docs/DEBT.md`.
```

- [ ] **Step 3: ag-cli — corregir Status**

Reemplazar la linea 3 por:

```markdown
> Status: Phases 2-4.5 — implemented. The `ag` binary exposes `new`, `dev`, `build`,
> `generate`, `schema lint`, `schema diff`, `domains check/sync` and `mail test`.
> `deploy`/`ai`/`migrate`/`plugin` subcommands arrive in later phases. See `docs/DEBT.md`.
```

- [ ] **Step 4: Verificar los tres**

Run: `grep -rniE "fase 0 - vac|phase 0 - empty" crates/ag-data/README.md crates/ag-dsl/README.md crates/ag-cli/README.md`
Expected: sin coincidencias (exit 1).

- [ ] **Step 5: Commit**

```bash
git add crates/ag-data/README.md crates/ag-dsl/README.md crates/ag-cli/README.md
git commit -m "docs(ag-data,ag-dsl,ag-cli): replace stale 'empty' status with real implementation state"
```

---

## Task 4: Corregir exactitud del README de `ag-cache` (L2 no funcional)

La auditoria creia que el L2 Redis funcionaba; en realidad solo emite un warning.
El README debe reflejarlo sin ambiguedad.

**Files:**
- Modify: `crates/ag-cache/README.md` (lineas de Status)

- [ ] **Step 1: Leer y corregir Status**

Leer `crates/ag-cache/README.md`. Reemplazar el bloque de Status (que dice
"L2 Redis disponible via feature `redis`") por:

```markdown
> Status: Phase 4 — L1 implemented (in-process, tag-based invalidation, moka).
> L2 is NOT functional yet: when `redis_url` is set the cache only logs a tracing
> warning (`src/lib.rs`). RFC-0005 proposes a native Anti-Gravital L2 over RESP2
> (no Redis dependency); see `docs/DEBT.md` and `docs/rfc/RFC-0005-ag-cache-native-l2.md`.
```

- [ ] **Step 2: Verificar**

Run: `grep -niE "L2 Redis disponible|L2 .*available" crates/ag-cache/README.md`
Expected: sin coincidencias (exit 1).

- [ ] **Step 3: Commit**

```bash
git add crates/ag-cache/README.md
git commit -m "docs(ag-cache): clarify L2 is not functional, point to RFC-0005"
```

---

## Task 5: Crear `docs/DEBT.md`

Fuente unica de la deuda tecnica (CLAUDE.md seccion 29: motivo, impacto, eliminacion,
issue, fecha objetivo).

**Files:**
- Create: `docs/DEBT.md`

- [ ] **Step 1: Crear el archivo con todas las deudas conocidas**

```markdown
# Technical Debt Register

Single source of truth for tracked technical debt across Anti-Gravital. Every
"skeleton"/TODO/TECH-DEBT marker in the codebase must point here. Format per
CLAUDE.md section 29.

> Convention: each entry has reason, impact, expected removal, owning plan and
> target. Dates are absolute. Close an entry only when the code and its plan agree.

## ag-mail

### DEBT-001 — Persistent queue backend
- Reason: the queue is in-memory only; messages are lost on restart.
- Impact: no delivery guarantees across restarts; no retry durability.
- Expected removal: plan P2 (`queue-persistent` feature over `ag-data`).
- Status: open. Target: before Phase 5.

### DEBT-002 — Custom SMTP headers ignored
- Reason: lettre limitations drop arbitrary custom headers in the SMTP adapter.
- Impact: custom headers set on `Email` are silently ignored over SMTP.
- Expected removal: plan P2 (review lettre API or contribute upstream).
- Status: open. Target: before Phase 5.

### DEBT-003 — External template engines
- Reason: only the built-in `StringTemplate` engine exists.
- Impact: no askama/minijinja support; no compile-time variable validation vs DSL.
- Expected removal: future plan; trait-based engine selection.
- Status: open. Target: Phase 5+.

## ag-cache

### DEBT-004 — Native L2 over RESP2 (no Redis)
- Reason: L2 is a stub that only logs a warning; Redis/fred is not wired.
- Impact: no distributed cache; vendor-lock risk if Redis is added directly.
- Expected removal: plan P5, gated on RFC-0005 approval.
- Status: open (blocked on RFC-0005). Target: before Phase 5 if RFC approved.

## ag-domains

### DEBT-005 — notAfter parsing for scheduled renewal
- Reason: `acme/renewal.rs` renews every cycle because `notAfter` is not parsed.
- Impact: unnecessary ACME calls; no date-based renewal or near-expiry alerts.
- Expected removal: plan P3.
- Status: open. Target: before Phase 5.

### DEBT-006 — Additional DNS adapters
- Reason: only Cloudflare adapter exists.
- Impact: limited provider choice (optional, not blocking).
- Expected removal: future, as opt-in adapters behind features.
- Status: open. Target: Phase 5+.

## ag-realtime

### DEBT-007 — Scalability proof (50k connections)
- Reason: no load test demonstrates the Phase 4 criterion of 50,000 connections.
- Impact: scalability claim is unverified.
- Expected removal: plan P4.
- Status: open. Target: before Phase 5.

### DEBT-008 — Event persistence buffer
- Reason: in-memory event bus loses critical events on restart.
- Impact: no durability for critical events.
- Expected removal: plan P4 (optional `event-persistence` feature).
- Status: open. Target: before Phase 5.

## ag-data

### DEBT-009 — DSL-generated typed ORM / RLS / multi-tenancy
- Reason: typed queries, row-level security and multi-tenancy come from the DSL.
- Impact: advanced data features unavailable until generated.
- Expected removal: later phases (Phase 3 ORM, later for RLS/multi-tenancy).
- Status: open. Target: Phase 5+.

## Tooling

### DEBT-010 — Coverage gate in CI
- Reason: no cargo-tarpaulin gate; roadmap requires >=80% per crate.
- Expected removal: plan P6.
- Status: open. Target: before Phase 5.

### DEBT-011 — Unified installer
- Reason: no install.sh / install.ps1.
- Expected removal: plan P6.
- Status: open. Target: before Phase 5.
```

- [ ] **Step 2: Verificar y commit**

Run: `test -f docs/DEBT.md && echo OK`
Expected: `OK`

```bash
git add docs/DEBT.md
git commit -m "docs: add technical debt register (docs/DEBT.md)"
```

---

## Task 6: Reglas nuevas en `CLAUDE.md` + ADR de gobernanza

La auditoria recomienda 5 reglas. Anadirlas a CLAUDE.md y respaldarlas con un ADR.

**Files:**
- Modify: `CLAUDE.md` (nueva subseccion en "Reglas adicionales del repositorio")
- Create: `docs/adr/0009-gobernanza-correctiva.md`

- [ ] **Step 1: Anadir las reglas a CLAUDE.md**

Leer `CLAUDE.md`. En la seccion "Reglas adicionales del repositorio (no negociables)",
anadir antes de "### Cierre" una subseccion nueva:

```markdown
### Reglas de estado real y autosuficiencia (ADR-0009)

1. Prohibido marcar un modulo, crate o API como "vacio", "skeleton", "no
   implementado" o equivalente cuando ya contiene codigo funcional y/o pruebas.
   El estado declarado en README y en la cabecera `//!` debe corresponder a la
   realidad del codigo. La deuda se enumera en `docs/DEBT.md`, no se disfraza de
   "skeleton".
2. Toda integracion con un servicio externo (Redis, NATS, S3, Cloudflare, SMTP de
   terceros, etc.) debe pasar por un adaptador detras de una feature de Cargo, y el
   crate debe conservar un modo de operacion nativo por defecto. La dependencia
   externa nunca es un requisito para usar el modulo.
3. La documentacion tecnica afectada se actualiza en la MISMA PR que el codigo y se
   revisa junto a el. Una PR que cambia comportamiento observable sin actualizar la
   documentacion correspondiente no se acepta.
4. Los scripts de instalacion (`install.sh`, `install.ps1`) deben ser auditables y
   verificar integridad/firma antes de ejecutar acciones privilegiadas. No se
   distribuye un instalador que descargue y ejecute sin verificacion.
5. Toda dependencia externa de infraestructura (bases de datos, colas, caches
   distribuidas) debe ser reemplazable por una implementacion nativa o auto-alojable,
   salvo justificacion explicita en una RFC.
```

- [ ] **Step 2: Crear el ADR-0009**

Crear `docs/adr/0009-gobernanza-correctiva.md` siguiendo `docs/adr/template.md`:

```markdown
# ADR-0009 — Reglas de gobernanza correctiva pre-Fase 5

## Contexto

Una auditoria externa de las fases 0-4.5 detecto fuerte desalineacion entre la
documentacion y el codigo: modulos funcionales marcados como "skeleton"/"vacio",
dependencias externas (Redis) tratadas como requisito en lugar de adaptador, y
documentacion que se actualizaba despues del codigo. Se requiere fijar reglas que
impidan reincidir antes de iniciar la Fase 5.

## Decision

Se incorporan a `CLAUDE.md` cinco reglas (estado real, adaptadores tras features con
modo nativo, docs en la misma PR que el codigo, instaladores auditados/firmados,
infraestructura externa reemplazable por nativa salvo RFC). Se crea `docs/DEBT.md`
como registro unico de deuda tecnica.

## Consecuencias

- Los README y cabeceras `//!` deben mantenerse fieles al codigo.
- Cada nueva integracion externa exige feature + modo nativo, reforzando la
  independencia de proveedores.
- La deuda tecnica deja de esconderse en comentarios "skeleton" dispersos.

## Alternativas

- No formalizar (rechazada: la auditoria mostro que la desalineacion reincide).
- Solo documentar sin regla en CLAUDE.md (rechazada: no es vinculante para agentes).

## Estado

Aceptada (2026-05-26). Relacionada con ADR-0007 (ag-mail/ag-domains) y RFC-0005.
```

- [ ] **Step 3: Verificar y commit**

Run: `grep -n "ADR-0009\|estado real y autosuficiencia" CLAUDE.md && test -f docs/adr/0009-gobernanza-correctiva.md && echo OK`
Expected: coincidencias + `OK`.

```bash
git add CLAUDE.md docs/adr/0009-gobernanza-correctiva.md
git commit -m "docs(gov): add ADR-0009 real-status and self-sufficiency rules to CLAUDE.md"
```

---

## Task 7: Alinear Hoja de Ruta y STATUS.md

**Files:**
- Modify: `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`
- Modify: `docs/roadmap/STATUS.md`

- [ ] **Step 1: Hoja de Ruta — referenciar DEBT.md**

Leer `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`. En la seccion de fases 0-4.5,
anadir (EN y ES, manteniendo el formato bilingue de docs-cierre) una nota:

```markdown
> Phases 0-4.5 are technically implemented. Outstanding tech debt that must be
> closed before Phase 5 is tracked in `docs/DEBT.md` (persistent mail queue,
> native cache L2, notAfter renewal, realtime scalability proof).
```

- [ ] **Step 2: STATUS.md — actualizar cabecera de fecha y estado**

Leer `docs/roadmap/STATUS.md`. Actualizar la cabecera "Ultima actualizacion" a
`2026-05-26` y anadir una linea: `Trabajo correctivo pre-Fase 5 en curso: ver
docs/DEBT.md y docs/superpowers/plans/2026-05-26-corrective-before-fase5-MASTER.md`.

- [ ] **Step 3: Verificar y commit**

Run: `grep -n "docs/DEBT.md" docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md docs/roadmap/STATUS.md`
Expected: coincidencias en ambos.

```bash
git add docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md docs/roadmap/STATUS.md
git commit -m "docs(roadmap): reference DEBT.md and mark pre-Phase-5 corrective work"
```

---

## Task 8: Verificacion final de P1

- [ ] **Step 1: No quedan mentiras de skeleton en crates implementados**

Run:
```bash
grep -rniE "skeleton|sin implementar|no implementado|empty modules|fase 0 - vac" \
  crates/ag-mail crates/ag-domains crates/ag-data crates/ag-dsl crates/ag-cli crates/ag-cache \
  --include=README.md --include=lib.rs
```
Expected: sin coincidencias (exit 1). Los crates realmente skeleton (ag-cloud, ag-ai,
ag-migrate, ag-mobile, ag-ui, ag-wasm-host) SI pueden seguir diciendo "Fase 0 - Vacio"
porque es verdad — no tocarlos.

- [ ] **Step 2: El workspace compila**

Run: `cargo build --workspace`
Expected: compila sin errores.

- [ ] **Step 3: Sin emojis ni evidencia de herramientas IA (regla del repo)**

Run: `grep -rnP "[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]" docs/DEBT.md CLAUDE.md docs/adr/0009-gobernanza-correctiva.md`
Expected: sin coincidencias (exit 1).

- [ ] **Step 4: DEBT.md cubre las 4 deudas clave del master**

Run: `grep -ncE "DEBT-00[1457]|persistent queue|native L2|notAfter|50k|scalability" docs/DEBT.md`
Expected: >= 4.

---

## Self-review (cobertura vs hallazgo #1 de la auditoria)

- READMEs desfasados (ag-mail, ag-domains, ag-data, ag-dsl, ag-cli, ag-cache) -> Tasks 1-4.
- Comentarios skeleton lib.rs (ag-mail, ag-domains) -> Tasks 1-2.
- docs/DEBT.md -> Task 5.
- 5 reglas nuevas CLAUDE.md + ADR -> Task 6.
- Hoja de Ruta / STATUS -> Task 7.
- Verificacion sin regresiones -> Task 8.

Las deudas tecnicas de codigo (cola persistente, RESP2, notAfter, 50k) NO se
implementan en P1: se REGISTRAN en DEBT.md y se cierran en P2-P5.
