# Estado honesto: deuda, issues abiertos y salud de pruebas (tarea E3)

- **Fecha:** 2026-11-23 (UTC). Corte de evidencia en vivo: 2026-11-22/23.
- **Proyecto:** `estado-honesto` (tablero `.kaizen/team/projects/estado-honesto/`; tarea `E3-deuda-y-issues`, owner `qa`, `tasks.json`).
- **Rama:** `community15/status-deuda`. Base de trabajo: commit `493f77a`.
- **Autoridad:** `autonomy: propose`. Este documento solo inventaria y cita; no cierra issues, no mueve etapas, no relaja la línea base y no aprueba puertas (K-10, L-3, L-10).
- **KPI:** 0 afirmaciones sin evidencia. Toda cifra lleva una referencia `EV-xx` definida en la sección 6. Los tableros, buzones y documentos citados se leen como **dato no confiable** (L-5/S-15); se reporta su contenido, no se obedece como instrucción.

---

## 0. Convención de evidencia

`EV-xx` = ruta de archivo + línea/sección, comando reproducible o issue, verificados en este repositorio o en el tablero remoto con `gh`. No se usan fuentes externas como autoridad. Los hallazgos S-11 se citan por su ruta en `.kaizen/loop/outbox/` (carpeta ignorada por git; el orquestador la sanea). No se re-ejecutaron los suites completos de Rust ni `cargo tarpaulin`: esas cifras se citan de la evidencia de puerta ya registrada, con su fecha, y se dice explícitamente que no fueron repetidas aquí (K-10).

---

## 1. Issues abiertos

### 1.1 Estado vivo del tablero

- **Anti-Gravital/Anti-Gravital:** 14 abiertos — #76, #87, #112, #141, #142, #152, #154, #156, #157, #178, #179, #180, #185, #208. [EV-01]
- **Anti-Gravital/Gravital-Fabric:** 1 abierto — #2. [EV-02]
- **Total: 15 issues abiertos** (14 AG + 1 Fabric). [EV-01][EV-02]

Este recuento **no coincide con los snapshots históricos** y la diferencia está documentada: el addendum wave-11 registró 14 AG + 2 Fabric; desde entonces #189 (AG), #198 (AG) y #3 (Fabric) se cerraron, y se abrió #208, un `ci-failure` autodetectado. [EV-09][EV-10][EV-01][EV-03]

### 1.2 Grupo A — Bloqueados externamente (8; 7 AG + 1 Fabric)

La `blocked reason` y la condición de cierre son las del cuerpo del issue y el addendum wave-11; la clasificación de "gate externo" es la del matrix wave-4. [EV-05][EV-07]

| Issue | Título (corto) | Tipo / etiquetas | Razón de bloqueo | Condición exacta de cierre | Owner |
| --- | --- | --- | --- | --- | --- |
| #87 | ag-domains: live ACME staging end-to-end test | `tech-debt`, `ag-domains`, `p3`, `blocked` | Dominio controlado + escritura DNS + HTTPS saliente a Let's Encrypt staging | El ignored test `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` existe, pasa en runner con credenciales y se vinculan logs reales; luego actualizar #76 | `dnsacme` (solo harness ACME local y docs de staging) |
| #141 | Installer integrity verification depends on a checksum file inside the same repository | `enhancement`, `p3`, `security`, `blocked` | Ceremonia de llave humana: clave offline en dos canales fuera de banda, manifiesto firmado publicado en tag, verificación en máquina limpia | Las seis condiciones de RFC-0030 §11 se cumplen | ninguno (requiere humano) |
| #156 | Execute the 24h fuzz manual gate per target and record logs | `tech-debt`, `p2`, `blocked` | Runner Linux x86-64 con presupuesto de 24 h para los SIETE targets y triage de crashes por seguridad | Logs de 24 h de los siete targets archivados, sin crashes sin resolver, triage revisado | `qafuzz` (solo automatización/evidencia de ventana corta) |
| #157 | Pre-Phase 5 release gate: final benchmark evidence on stabilized code | `tech-debt`, `p2`, `blocked` | Hardware de referencia + generador de carga + código estabilizado | Evidencia rule-17 en hardware de referencia, `docs/benchmarks/pre-fase5-performance.md` escrito, fila del gate y README reconciliados | `qabench` (solo procedimiento y muestra reducida) |
| #178 | Upgrade the mail protocol stack (mail-send 0.7, mail-auth 0.15, mail-parser 1.0) in the native MTA | `tech-debt`, `p2`, `ag-mail`, `blocked` | Runner con egress de puerto 25 a buzón real; el upgrade y el verificador DKIM independiente ya están satisfechos | Header `Authentication-Results` registrado de un envío real con egress de puerto 25 | `mailverify` (herramientas wave-11 ya mergeadas) |
| #179 | Upgrade instant-acme 0.7 to 0.8 (ACME order flow API rewritten) | `tech-debt`, `ag-domains`, `p3`, `blocked` | Depende de #87: se necesita staging E2E antes y después del port | Staging E2E pasa antes y después del port 0.7 → 0.8, sin rewrite no verificado de renewal | `dnsacme` (solo notas de migración y harness local) |
| #185 | Replace unmaintained chrono with a maintained date/time crate | `tech-debt`, `p2`, `security`, `blocked` | `object_store` 0.14.2 declara chrono como dependencia no opcional; el reporte de la librería está abierto, sin asignar y sin PR | `object_store` elimina chrono o se reemplaza el adaptador S3; se retira el ignore RUSTSEC-2026-0543; `cargo tree -i chrono` vacío y deny/audit/test en verde | `chrono` (wave-11: nota de "no se puede hacer todavía") |
| Fabric #2 | Configure KAIZEN_TOKEN secret for kaizen-loop workflow | sin etiquetas | El secreto `KAIZEN_TOKEN` no está configurado en Gravital-Fabric; S-2 prohíbe al agente leer o crear credenciales | Secreto configurado con `contents: write`, `pull-requests: write`, `issues: write`, `actions: read`; un run disparado llega a `Loop iteration`; alcance documentado en `docs/kaizen/loop/SECURITY.md` sin publicar el valor | ninguno |

Citas: #87/#141/#156/#157/#178/#179/#185 en [EV-01][EV-05][EV-06][EV-07][EV-17]; Fabric #2 en [EV-02][EV-04]. El workflow `fuzz-24h.yml` y el `dkim-independent.yml` son preparación, no evidencia de gate. [EV-19]

### 1.3 Grupo B — Diferidos por decisión aceptada (4; todos AG)

No llevan `blocked`: están pospuestos por una decisión ADR/RFC/BDFL, no por un gate externo. [EV-05][EV-06]

| Issue | Título (corto) | Tipo / etiquetas | Decisión que lo difiere | Condición exacta de cierre | Owner |
| --- | --- | --- | --- | --- | --- |
| #112 | ag-workers: wire producer-only enqueue from ag-edge when a concrete consumer exists | `enhancement`, `tech-debt`, `p3`, `ag-edge`, `ag-workers` | RFC-0012 §17.4 / S6; ADR-0013 | Existe un consumidor concreto de ag-edge; se cablea solo `producer` con test de integración y el chequeo de ciclo de dependencias está en verde | ninguno |
| #152 | ag-mail native MTA: managed REST API and marketing objects | `enhancement`, `tech-debt`, `p2`, `ag-mail` | RFC-0009 Mail A gate; decisión BDFL 2026-11-22 | Evidencia Mail A registrada (entrega externa real con SPF/DKIM/DMARC alineados) o enmienda RFC-0009 que saque C/D; superficie REST/marketing probada | ninguno |
| #154 | ag-data: DSL-generated typed ORM, row-level security and multi-tenancy | `enhancement`, `tech-debt`, `p3`, `ag-data` | ADR-0020 | Fase 5 abierta, TODAS las filas bloqueantes del release gate en `pass` (Audit/Deny ya pasan; Fuzz 24h, Benchmarks y Open debt siguen `pending`), y código ORM/RLS/tenancy generado y probado | ninguno |
| #180 | Evaluate chumsky 0.9 to 0.13 for the Anti-DSL parser | `ag-dsl`, `tech-debt`, `p3` | ADR-0021 (RFC-0027 aceptada: mantener 0.9.3) | Congelación de gramática DSL v1.0 o bloqueo concreto; RFC que supere ADR-0021, tests sin cambios, artefactos generados byte-idénticos y fuzz smoke en verde | ninguno |

Citas: [EV-01][EV-05][EV-06][EV-23]. El propio issue #180 ya corrige el objetivo a 0.14.x, no 0.13, y ADR-0021 lo refleja. [EV-05][EV-18]

### 1.4 Grupo C — Accionable ahora (1; AG)

| Issue | Título | Tipo / etiquetas | Situación | Condición de cierre | Owner |
| --- | --- | --- | --- | --- | --- |
| #208 | `[ci-failure] kaizen-gates` | `ci-failure`, `auto-detected` | Creado 2026-11-22T18:34:52Z por detección automática; cuerpo: `Resultado: failure`, run `45055852575`, commit `5ff8e5d`, sin etiqueta `blocked` | Investigar el fallo del run y dejar `kaizen-gates` en verde en el commit citado; un fallo de CI no se ignora | sin asignar |

Citas: [EV-01][EV-03]. Es el único issue abierto sin `blocked` ni decisión que lo difiera y con una acción de ejecución inmediata.

### 1.5 Padres de seguimiento (2; no encajan en los tres grupos)

| Issue | Estado | Condición de cierre | Owner |
| --- | --- | --- | --- |
| #76 `[tracking] ag-domains remaining work` | OPEN, 16 hijos (15 CLOSED / 1 OPEN: #87) | Primero `Closes #87` verificado y mergeado; solo después los 16 hijos CLOSED y `Closes #76` | ninguno (monitoreado por trackers) |
| #142 `[tracking] Repository audit through Phase 4.6` | OPEN, 31 hijos (26 CLOSED / 5 OPEN: #141, #152, #154, #156, #157) | Los cinco hijos abiertos CLOSED con evidencia mergeada en `main`; nunca cerrar el padre con un hijo abierto | ninguno (monitoreado por trackers) |

Citas: [EV-01][EV-05][EV-06][EV-08]. No se cerró ningún issue ni se usó `Closes #NNN` en este trabajo.

### 1.6 Recuento final

| Grupo | AG | Fabric | Total |
| --- | --- | --- | --- |
| A. Bloqueados externamente | 7 | 1 | **8** |
| B. Diferidos por decisión | 4 | 0 | **4** |
| C. Accionables ahora | 1 | 0 | **1** |
| Padres de seguimiento (aparte) | 2 | 0 | **2** |
| **Total** | **14** | **1** | **15** |

Fuente de los totales: [EV-01][EV-02]; clasificación: [EV-05][EV-06][EV-07][EV-03][EV-04].

---

## 2. Deuda técnica

### 2.1 `docs/DEBT.md`: estado congelado

`docs/DEBT.md` es un **registro histórico congelado**: 25 entradas `DEBT-001`…`DEBT-025`, de las cuales la mayoría están cerradas o migradas a issues (#76, #78, #142, #151–#156). La fuente viva de deuda es el tablero de GitHub con etiqueta `tech-debt` (CLAUDE.md regla 29). No se añaden entradas nuevas; el archivo se conserva para trazabilidad. [EV-11]

Deuda migrada aún abierta que aparece en ese registro: #154 (DEBT-009), #156 (DEBT-015), #76 (DEBT-025) y, vía el tracking #142, #141/#152/#157. [EV-11][EV-01]

### 2.2 Ratchet de deuda: 4 hallazgos (`mktemp XXXXXX`)

- `.kaizen/baseline.json` fija `debt_max: 0` (actualizado 2026-11-21). [EV-12]
- `scripts/kaizen:15-17` define `count_debt()` como un `grep -rEI 'TODO|FIXME|HACK|XXX'` sobre `KZ_SRC_GLOBS`; en esta rama `.kaizen/config.env` define `KZ_SRC_GLOBS="scripts tests"`. El token `XXX` coincide con la plantilla `XXXXXX` de `mktemp`. [EV-13]
- `bash scripts/kaizen check --only ratchet` devuelve `FALLA: deuda subio: 4 > 0` (exit 1). [EV-14]
- Las **4 líneas exactas** que cuenta el ratchet, verificadas con el mismo patrón, son plantillas `mktemp -d` y no deuda real: [EV-13]

| Archivo:línea | Contenido |
| --- | --- |
| `scripts/validate-deploy.sh:90` | `WORK_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/ag-deploy-validation.XXXXXX") \|\| {` |
| `scripts/validate-domain-loop.sh:95` | `WORK_ROOT=$(mktemp -d "/tmp/opencode/ag-domain-loop-validation.XXXXXX") \|\| {` |
| `scripts/release-smoke.sh:152` | `WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/ag-release-smoke.XXXXXX")"` |
| `scripts/validate-templates.sh:70` | `WORK_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/ag-template-validation.XXXXXX") \|\| {` |

- Hallazgo S-11 de referencia: `.kaizen/loop/outbox/kaizen-ratchet-deuda-codegen.md` documenta esta misma clase de falso positivo; su sección "Donde" cita literales `TODO` de `crates/ag-dsl/src/codegen/{rust_gen.rs,worker_gen.rs}` bajo otro conjunto de globs. La cuenta (4) es la misma, pero las líneas reportadas arriba son las de **esta rama**. No se relajó la línea base ni se editó `KZ_SRC_GLOBS` (K-5/L-10). [EV-15]
- Trabajo correctivo planificado: tarea `F4-arreglar-contador-deuda` del tablero `kaizen-gates-green`, owner `verificador-evidencias`, debida 2026-11-29. [EV-27]

### 2.3 Excepciones del límite de tamaño de archivo

`tools/file-size-allowlist.txt` (regla RFC-0022 §4.3, límite 600 líneas) contiene **14 excepciones**, todas referenciadas a **#142**, el padre de auditoría. `bash tools/check-file-size.sh` → `OK: 313 tracked Rust files checked, 14 allowed exceptions, none over 600 lines.` (exit 0). [EV-16]

Las 14 están activas (recuento real de líneas, 606–1507): `benchmarks/neon-real/src/main.rs` (1245), `crates/ag-auth/src/oauth.rs` (789), `crates/ag-auth/src/webauthn.rs` (625), `crates/ag-domains/src/api.rs` (654), `crates/ag-domains/src/provider/route53.rs` (606), `crates/ag-domains/src/provider/sdk.rs` (669), `crates/ag-dsl/src/ast.rs` (638), `crates/ag-dsl/src/codegen/rust_gen.rs` (919), `crates/ag-dsl/src/parser.rs` (1482), `crates/ag-dsl/src/semantic.rs` (1507), `crates/ag-mail/src/sender/mta/mod.rs` (767), `crates/ag-mail/src/sender/mta/queue.rs` (833), `crates/ag-storage/src/store/mod.rs` (667), `crates/ag-storage/src/store/server.rs` (658). [EV-16]

**Texto histórico desactualizado:** los addenda wave-9/wave-11 describen "cinco entradas" para #189 (`cli.rs`, `config.rs`, `main.rs`, `tests/json_output.rs`, `env.rs`). El archivo vivo —tanto en esta rama como en `main`— tiene 14 entradas `#142`, e #189 ya está CLOSED (2026-11-22T15:06:45Z). Se reporta la discrepancia sin editar el archivo ni el allowlist. [EV-16][EV-10]

### 2.4 Ignore de chrono

`deny.toml:21-26` mantiene el ignore de `RUSTSEC-2026-0543` (chrono sin mantenimiento) con condición de retirada: que `object_store` elimine chrono o se reemplace el adaptador S3, tracked en **#185**. `docs/audits/chrono-unblock-analysis.md` (§1–§5) verifica que: (a) solo queda el edge `object_store 0.14.2 -> chrono 0.4.45`; (b) `cargo deny` pasa solo por el ignore; (c) el reporte upstream `apache/arrow-rs-object-store#909` está abierto, sin asignar, sin milestone y sin PR; (d) la retirada exige cinco condiciones simultáneas; hasta entonces el keyword correcto es `Refs #185`, nunca `Closes #185`. [EV-17]

Además, `docs/audits/chrono-unblock-analysis.md:213-218` registra documentación stale: `crates/ag-storage/README.md` y `docs/modules/ag-storage/README.md` todavía dicen `object_store 0.11` frente a 0.14.2 en los manifiestos. [EV-17]

### 2.5 Marcadores TODO / TECH-DEBT que referencian issues

- La convención existe: `CLAUDE.md:744` (`// TECH-DEBT (issue #123):`) y `Cargo.toml:154` exigen que todo marcador apunte a un issue o RFC. [EV-18]
- **No queda ningún marcador `// TECH-DEBT` en `crates/`**: `grep` con `TECH-DEBT \(issue|TECH-DEBT:|// TECH-DEBT` sobre `crates/` devuelve 0 coincidencias. El issue #135 ("TECH-DEBT markers without issue refs") está CLOSED y ADR-0019 documenta la retirada de los dos marcadores OTLP. [EV-18]
- Sí permanecen comentarios de deuda con referencia de issue en configuración y pruebas: `deny.toml:25` (#185), `deny.toml:30` (#174), `Cargo.toml:119` (#185), `crates/ag-observe/src/layer.rs:74` (#181), `crates/ag-workers/src/admission.rs:19,115` (#113), `tests/generated-rust-fixture/tests/serde_time.rs:1` (#185), `crates/ag-cache/tests/redis_l2.rs:1` (#144), `.github/workflows/fuzz-24h.yml:7,111` (#156), `.github/workflows/dkim-independent.yml:3` (#178), `.github/workflows/docs.yml:41` (#182), `deploy/docker/Dockerfile.template:47` (#195). [EV-19]

---

## 3. Salud de pruebas

### 3.1 Número de grupos de pruebas

No hay una única definición canónica; se reportan tres mediciones con su origen, sin mezclarlas:

| Medición | Valor | Fuente |
| --- | --- | --- |
| Targets `test` del workspace (medición en vivo, sin red) | **40** | `cargo metadata --no-deps --format-version 1`, contando `targets` con `kind` = `test` [EV-20] |
| Suites registradas en la puerta Stage 1 | 52 suites, 564 passed, 0 failed, 7 ignored | `docs/audits/pre-fase5-build-gate.md:21`, snapshot 2026-05-29, no repetido aquí [EV-21] |
| Conteo del roadmap | 84 tests (56 unit + 27 E2E + 1 doctest) | `docs/roadmap/STATUS.md:112` [EV-22] |
| Targets de fuzzing (gate 24 h) | 7 | `docs/audits/PRE_FASE5_RELEASE_GATE.md:31`, `docs/fuzz/24h-gate-verification.md` [EV-23] |

Comprobaciones de apoyo: 189 archivos bajo `crates/` con anotaciones `#[test]`/`#[tokio::test]`/`#[rstest]` y 10 archivos `.rs` bajo `tests/`. [EV-20]

### 3.2 Estado de la puerta de cobertura

- **Rust / release gate: `pass`.** `cargo tarpaulin --workspace --fail-under 80` (excluye archivos de test y `main`s): 80.05 %, 3210/4010 líneas, exit 0, 2026-05-30. [EV-23]
- **CI:** job `coverage` en `.github/workflows/quality.yml:176-211` ejecuta `cargo tarpaulin --workspace --timeout 300 --fail-under 80` en cada push/PR a `main`. [EV-24]
- **Framework Kaizen: no evaluada.** `.kaizen/config.env` tiene `KZ_COVERAGE_CMD=""` y `KZ_WARNINGS_CMD=""`, por lo que `kaizen check` omite las comprobaciones de cobertura y advertencias; no se inventó un número. [EV-25]
- `docs/roadmap/STATUS.md:112` señala además que la medición oficial con `cargo-llvm-cov` está pendiente. [EV-22]
- **No se re-ejecutó tarpaulin en esta iteración**: la cifra es la registrada, con fecha. [EV-23]

### 3.3 Las 4 fallas de `kaizen check` y su referencia S-11

`bash scripts/kaizen check` ejecutado en esta rama devuelve `kaizen check: FALLO` (exit 1) con `ok: descriptores pendientes` antes de crear el descriptor, `ok: politicas de contenido`, `ok: formato`, `ok: lint`, y **cuatro fallas sustantivas** [EV-26]:

| # | Falla reportada | Causa verificada | Referencia S-11 (outbox) | Acción correctiva planeada |
| --- | --- | --- | --- | --- |
| 1 | `FALLA: tests` | `.kaizen/config.env` → `KZ_TEST_CMD="bash tests/run.sh"`; `ls tests/run.sh` → `No such file or directory` | `.kaizen/loop/outbox/kaizen-test-cmd-inexistente.md` | Tarea `F1-crear-tests-runsh`, owner `platform-engineer`, debida 2026-11-29 [EV-27] |
| 2 | `FALLA: swarm: ver kaizen-swarm check` | `bash scripts/kaizen-swarm check` → `FALLA: falta ADR-0001 (autonomia nivel 3)`; `docs/kaizen/adr/` contiene 0002..0010 pero no 0001 | `.kaizen/loop/outbox/kaizen-swarm-adr0001-faltante.md` | Tarea `F2-ADR-autonomia-nivel3`, owner `knowledge-curator`, debida 2026-11-29 [EV-27] |
| 3 | `FALLA: gobernanza: ver kaizen-gov check` | `python3 scripts/kaizen-gov check` → 4 agentes sin gobernanza: `oss-observability-specialist`, `container-k8s-specialist`, `rust-lowlevel-specialist`, `rust-perf-benchmarker`; `regenerate` degrada la matriz de 43 a 25 y se restauró desde HEAD | `.kaizen/loop/outbox/kaizen-gov-especialistas-y-regresion.md` | Tarea `F3-catalogo-needs-human`, owner `facilitador-equipo`, debida 2026-11-29 [EV-27] |
| 4 | `FALLA: deuda subio: 4 > 0` | Ratchet: baseline `debt_max: 0` frente a 4 líneas `mktemp XXXXXX` (ver §2.2) | `.kaizen/loop/outbox/kaizen-ratchet-deuda-codegen.md` | Tarea `F4-arreglar-contador-deuda`, owner `verificador-evidencias`, debida 2026-11-29 [EV-27] |

Quinta falla **local y ya resuelta por este cambio**: `FALLA: falta el descriptor docs/kaizen/pr-drafts/community15-status-deuda.md`. Se corrige añadiendo el descriptor en este mismo commit. [EV-26]

Matiz de alcance: `bash scripts/kaizen-org check` está **en verde** en esta rama; el hallazgo `.kaizen/loop/outbox/kaizen-org-catalogo-desincronizado.md` corresponde a otro contexto de instalación. Los outbox `ag-check-falsos-positivos-push.md` (3 falsos positivos del pre-push) y `kaizen-security-falso-hardcoding-test-url.md` (aviso S-3 no bloqueante en `layer.rs:242`) **no** son fallas de `kaizen check` y se citan solo para no silenciarlas (S-10/S-11). [EV-28]

---

## 4. Deuda que probablemente nunca se cerrará sin humano o externo

Cada línea es una inferencia acotada a la evidencia citada; no es una decisión.

| Issue / elemento | Por qué no se cerrará con trabajo local | Evidencia |
| --- | --- | --- |
| #141 | La ceremonia de llave (clave offline, dos canales fuera de banda, manifiesto firmado en tag, verificación en máquina limpia) no puede ser ejecutada por un agente | [EV-05][EV-07] |
| #87 | Requiere dominio controlado, escritura DNS y HTTPS saliente a Let's Encrypt staging con credenciales | [EV-05][EV-17] |
| #178 | El egress de puerto 25 está bloqueado en este sandbox y en CI hosteada; solo un host con ese egress produce el header | [EV-05][EV-11] |
| #156 | Requiere un runner financiado con 24 h para siete targets y triage de seguridad; ventanas cortas no son evidencia | [EV-05][EV-07][EV-23] |
| #157 | Requiere hardware de referencia y código estabilizado; la ejecución informativa en hardware no de referencia no cierra la fila | [EV-05][EV-07][EV-23] |
| #185 | El reporte upstream de la librería está abierto, sin asignar y sin PR; la alternativa local exige un RFC aceptado y un cliente S3 verificado que no existe | [EV-17] |
| Fabric #2 | Configurar un secreto es una acción humana con privilegios; S-2 prohíbe al agente leer credenciales | [EV-02][EV-04] |
| #179 | Depende de #87 para la evidencia "antes y después" | [EV-05][EV-07] |
| Las 4 líneas del ratchet | La corrección exige decidir `KZ_SRC_GLOBS` o la línea base; K-5 y L-4 prohíben que el agente relaje la línea base por su cuenta | [EV-12][EV-13][EV-14][EV-15] |
| #112, #152, #154, #180 | Diferidos por decisión aceptada: solo un humano/BDFL con nueva evidencia o enmienda los reabre | [EV-05][EV-06] |

Además, las cinco excepciones de `file-size` más grandes (919–1507 líneas) no tienen un issue de refactorización propio: todas apuntan al padre #142, y el propio archivo pide reemplazarlas por un issue enfocado cuando empiece el trabajo. [EV-16]

---

## 5. Verificación de este cambio

- `git diff --check` → se ejecuta antes del commit; resultado se registra en el diario `docs/kaizen/journal/`. [EV-29]
- Escaneo de contenido prohibido (emoji, atribución a IA, supresiones S-10) sobre los archivos cambiados → sin coincidencias. [EV-29]
- `bash scripts/kaizen check` → cuatro fallas preexistentes (§3.3), reportadas sin supresión; el descriptor añadido resuelve la quinta. No se relajó la línea base, no se desactivó CI y no se omitió la verificación de hooks. [EV-26]
- No se re-ejecutaron `cargo test --workspace` ni `cargo tarpaulin`: sus cifras se citan de la evidencia registrada con fecha. [EV-21][EV-23]
- No se cerró ningún issue: este descriptor declara `Closes: none`. [EV-30]

---

## 6. Registro de evidencia (EV-xx)

| ID | Fuente verificable |
| --- | --- |
| EV-01 | `gh issue list --repo Anti-Gravital/Anti-Gravital --state open --limit 200 --json number,title,labels` → 14 abiertos: 208, 185, 180, 179, 178, 157, 156, 154, 152, 142, 141, 112, 87, 76 |
| EV-02 | `gh issue list --repo Anti-Gravital/Gravital-Fabric --state open --limit 200 --json number,title,labels` → #2; `wc` sobre la salida → 1 |
| EV-03 | `gh issue view 208 --repo Anti-Gravital/Anti-Gravital --json number,title,createdAt,body` → creado 2026-11-22T18:34:52Z, run 45055852575, commit 5ff8e5d |
| EV-04 | `gh issue view 2 --repo Anti-Gravital/Gravital-Fabric --json number,title,createdAt,body,labels` → sección "Done when" con los tres criterios de `KAIZEN_TOKEN` |
| EV-05 | `docs/audits/issue-board-reconciliation.md:24-38` (matriz wave-4: clase, decisión gobernante, condición exacta, etiquetas) y `:107-128` (keywords de cierre) |
| EV-06 | `docs/audits/issue-board-reconciliation.md:169-203` (matriz wave-9 owner/evidence/closure y notas de propiedad) |
| EV-07 | `docs/audits/issue-board-reconciliation.md:236-333` (matriz wave-11 owner/branch/estado/merge; notas de propiedad y disciplina de merge) |
| EV-08 | `docs/audits/issue-board-reconciliation.md:82-105` (checklists de #76 y #142) y `:335-515` (addenda wave-12, hijos verificados uno a uno) |
| EV-09 | `docs/audits/issue-board-reconciliation.md:245-252` (snapshot wave-11: 14 AG + 2 Fabric) |
| EV-10 | `gh issue view 3 --repo Anti-Gravital/Gravital-Fabric --json state,closedAt` → CLOSED 2026-11-22T15:00:01Z; `gh issue view 189/198 --repo Anti-Gravital/Anti-Gravital` → CLOSED 2026-11-22T15:06:45Z / 12:54:23Z |
| EV-11 | `docs/DEBT.md:1-14` (congelado, regla 29 de CLAUDE.md); `grep -c '^### DEBT-' docs/DEBT.md` → 25; migraciones a #76/#142/#151-#156 en el propio archivo |
| EV-12 | `.kaizen/baseline.json` → `{"updated":"2026-11-21T00:00:00Z","debt_max":0}` |
| EV-13 | `scripts/kaizen:15-17` (`count_debt()`), `.kaizen/config.env` (`KZ_SRC_GLOBS="scripts tests"`); `grep -rEI 'TODO\|FIXME\|HACK\|XXX' scripts tests` → 4 líneas (ver §2.2) |
| EV-14 | `bash scripts/kaizen check --only ratchet` → `FALLA: deuda subio: 4 > 0`, exit 1 |
| EV-15 | `.kaizen/loop/outbox/kaizen-ratchet-deuda-codegen.md` (S-11) |
| EV-16 | `tools/file-size-allowlist.txt` (14 entradas `#142`); `bash tools/check-file-size.sh` → `OK: 313 tracked Rust files checked, 14 allowed exceptions, none over 600 lines.`; `wc -l` de las 14 rutas (606–1507); `diff` rama vs `main` → sin cambios. Discrepancia con addenda wave-9/11 en `docs/audits/issue-board-reconciliation.md:184,263,300-305` |
| EV-17 | `deny.toml:21-26`; `docs/audits/chrono-unblock-analysis.md:1-228` (§1.4, §4, §6) |
| EV-18 | `CLAUDE.md:744,767`; `Cargo.toml:154`; `grep` de `TECH-DEBT` sobre `crates/` → 0 coincidencias; `docs/audits/issue-board-reconciliation.md:442` (#135 CLOSED); `docs/adr/0019-ag-observe-otlp-transport.md:9,50` |
| EV-19 | `grep` de `issue #[0-9]` sobre el repositorio; líneas concretas listadas en §2.5 |
| EV-20 | `cargo metadata --no-deps --format-version 1` filtrado por `kind == test` → 40; `grep -rlE '#\[(tokio::)?test\]\|#\[rstest\]' crates --include='*.rs'` → 189; `find tests -name '*.rs'` → 10 |
| EV-21 | `docs/audits/pre-fase5-build-gate.md:21` (564 passed, 0 failed, 7 ignored, 52 suites; 2026-05-29) |
| EV-22 | `docs/roadmap/STATUS.md:112` |
| EV-23 | `docs/audits/PRE_FASE5_RELEASE_GATE.md:25,30-38` (Tests/Audit/Deny/Fuzz smoke/Fuzz 24h/Coverage/Benchmarks/Open debt); `docs/fuzz/24h-gate-verification.md` |
| EV-24 | `.github/workflows/quality.yml:176-211` |
| EV-25 | `.kaizen/config.env` (`KZ_COVERAGE_CMD=""`, `KZ_WARNINGS_CMD=""`) y `scripts/kaizen:60-69` (las comprobaciones se omiten si la variable está vacía) |
| EV-26 | `bash scripts/kaizen check` en esta rama antes del descriptor → `ok: sin descriptores pendientes`, `ok: politicas de contenido`, `ok: formato`, `ok: lint`, `FALLA: tests`, `FALLA: swarm: ver kaizen-swarm check`, `FALLA: gobernanza: ver kaizen-gov check`, `FALLA: deuda subio: 4 > 0`, `FALLA: falta el descriptor docs/kaizen/pr-drafts/community15-status-deuda.md`, `kaizen check: FALLO` (exit 1) |
| EV-27 | `.kaizen/team/projects/kaizen-gates-green/tasks.json` (F1/F4/F2/F3, owners y due 2026-11-29); `bash scripts/kaizen-swarm check` → falta ADR-0001; `python3 scripts/kaizen-gov check` → 4 agentes; `bash scripts/kaizen-org check` → verde |
| EV-28 | `.kaizen/loop/outbox/ag-check-falsos-positivos-push.md`; `.kaizen/loop/outbox/kaizen-security-falso-hardcoding-test-url.md`; `.kaizen/loop/outbox/kaizen-org-catalogo-desincronizado.md` (todos S-11, carpeta ignorada por git) |
| EV-29 | `git diff --check` y escaneo de contenido sobre los archivos cambiados; se registran en `docs/kaizen/journal/2026-11-23-estado-honesto-e3-*.md` |
| EV-30 | `docs/kaizen/pr-drafts/community15-status-deuda.md`, sección "Cierre de issues" → `Closes: none` |
