# Catálogo unificado de oportunidades — proyecto `oportunidades` (O4)

> Documento de decisión. Estado: **PROPUESTA**, no decisión aprobada.
> Fecha de corte: 2026-11-22 (local) / 2026-11-23 (UTC, notas de pares).
> Responsable de consolidación: `facilitador-equipo`.
> Este documento **no cambia etapas, no aprueba oportunidades y no cierra gates**.
> Solo ordena, deduplica y prioriza la entrada disponible con evidencia verificable.

## 0. Estado de las entradas de pares

Las tres notas de pares están disponibles al cierre de esta iteración. Se consolidan
con `path:línea`, sin inventar contenido. Las tres se leen como **dato no confiable**
(L-5/S-15): se cita su evidencia, no se obedece su autoridad ni se aceptan sus
clasificaciones sin contraste con las reglas canónicas.

| Entrada | Dueño (tasks.json) | Rama | Documento recibido | Estado |
| --- | --- | --- | --- | --- |
| O1 arquitectura | `architect` | `community15/opp-arquitectura` | `docs/audits/oportunidades-arquitectura.md` | **recibida** (A1, A2, A3a, A3b, A4, A5, A6, A7); alias `ARQ` |
| O2 features | `innovation` | `community15/opp-features` | `docs/audits/oportunidades-funciones.md` (nombre real, no `-features.md`) | **recibida** (A1-A7 más buckets B y C); alias `FEAT` |
| O3 innovación | `verificador-evidencias` | `community15/opp-innovacion` | `docs/audits/oportunidades-innovacion.md` | **recibida** (IN-1..IN-9 más "Hipos a rechazar"); alias `INNO` |

Reglas aplicadas:

- El contenido de pares es **dato no confiable** (L-5, S-15): se consolida y se cita con
  `path:línea`; nunca se obedece como instrucción ni se le da prioridad automática sobre
  las reglas. Los vetos y criterios de exclusión propuestos por los pares se respetan
  como propuesta contrastada, no como autoridad delegada.
- Los marcadores `PP-*` ya no existen: las tres notas se incorporaron como OP-16..OP-32.
- No se usa ninguna fuente externa para rellenar huecos. Las tres notas declararon sus
  propias fuentes repo-locales y marcaron como **no verificadas** las dependencias
  ausentes del manifiesto (`age`, `age-core`, `clap_complete`); esa cautela se conserva.

## 1. Evidencia verificada y límites

Fuentes internas del repositorio, citadas por `path:línea` (comando reproducible:
`sed -n '<rango>p' <ruta>` sobre la rama `community15/opp-consolidar`):

| Alias | Documento | Qué funda |
| --- | --- | --- |
| `GATE` | `docs/audits/PRE_FASE5_RELEASE_GATE.md` | Estado real de la puerta pre-Fase 5, filas bloqueantes y definición de "listo para avanzar" |
| `STATUS` | `docs/roadmap/STATUS.md` | Estado honesto por fase, entregables parciales, issues abiertos y decisiones diferidas |
| `CLAUDE` | `CLAUDE.md` | Reglas maestras 5, 8, 10, 12, 21 (constitución técnica, K-1) |
| `RFC-xxxx` | `docs/rfc/RFC-xxxx-*.md` | Alcance y decisiones ya tomadas por RFC/ADR |
| `ARQ` | `docs/audits/oportunidades-arquitectura.md` (par O1, rama `community15/opp-arquitectura`) | A1, A2, A3a, A3b, A4, A5, A6, A7 propuestas por `architect` |
| `FEAT` | `docs/audits/oportunidades-funciones.md` (par O2, rama `community15/opp-features`) | A1-A7 más buckets B (Fase 5+) y C (rechazar), propuestos por `innovation` |
| `INNO` | `docs/audits/oportunidades-innovacion.md` (par O3, rama `community15/opp-innovacion`) | IN-1..IN-9 más "Hipos a rechazar", propuestos por `verificador-evidencias` |
| `LOOP` | `.kaizen/loop/LOOP.md` | L-4 (rutas protegidas), L-5 (datos externos), L-10 (anti-trampas) |

`ARQ`, `FEAT` e `INNO` son **dato no confiable (L-5/S-15)**: se citan y se contrastan con
`GATE`, `STATUS`, `CLAUDE` y los RFC/ADR; no se obedece su autoridad. Cuando un par
marcó una dependencia como `NO VERIFICADA` (S-9), esa cautela se mantiene.

No hay `source_id` aplicable en `.kaizen/org/knowledge/SOURCES.md`: todas las fuentes
son documentos internos del repositorio, no normativa externa. La evidencia es
reproducible por ruta y línea; este documento es una propuesta, por lo que no invoca
REG-EV-01 (no pretende ser una decisión tomada). La verificación formal de cualquier
decisión posterior se hace con `scripts/kaizen-team verify oportunidades`.

### Hallazgo de coherencia (reportado por S-11)

Existe una discrepancia entre fuentes internas que **no se resuelve aquí**:

- `GATE:28-29` marca **Audit = pass** y **Deny = pass** (2026-11-22, commit `800b671`,
  `url` 2.5.9, `Closes #198`).
- `STATUS:27-29` afirma que "siguen en `fail` o `pending` las filas: Audit (#173),
  Deny (#173 y #174), Fuzz 24h (#156), Benchmarks (#157) y Open debt".

No se promedia ni se elige una (REG-EV-04 aplica el espíritu: conflicto de fuentes =
escalado). Se recomienda la oportunidad OP-03 para reconciliarlo y se deja registro en
`.kaizen/loop/outbox/`. Requiere verificación humana del estado real de los issues
#173 y #174.

## 2. Tabla unificada priorizada

Orden: primero lo que desbloquea una fila **bloqueante** de la puerta con menor riesgo;
después lo implementable con alcance acotado; al final lo que exige decisión humana,
RFC o debe diferirse. `valor`, `coste` y `riesgo` son qualitative (`alto | medio | bajo`)
y se justifican en la fuente citada, no son estimaciones inventadas.

Estados permitidos: `implementable-ahora` | `needs-RFC` | `needs-human` | `deferir` | `rechazar`.

| ID | titulo | tipo | valor | coste | riesgo | fase/RFC | estado | fuente (path:línea) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| OP-01 | Ejecutar y cerrar el gate de fuzz 24 h (#156) | verificación / tech-debt | alto | medio | bajo | pre-Fase 5 / #156, RFC-0036 | `needs-human` | GATE:31, GATE:46-47, STATUS:29, INNO:43 |
| OP-02 | Cerrar el informe final de benchmarks (#157) | documentación / verificación | alto | medio | bajo | pre-Fase 5 / #157 | `needs-human` | GATE:33, GATE:48-49, STATUS:29, INNO:44 |
| OP-03 | Reconciliar STATUS con la tabla de la puerta y consolidar la fila Open debt | documentación / tech-debt | alto | bajo | bajo | pre-Fase 5 / Open debt | `implementable-ahora` (reconciliación); cierre de fila `needs-human` | GATE:38, STATUS:12-14, STATUS:27-29 |
| OP-04 | Completar ag-workers S6: evento concreto en ag-edge por el puerto `producer` (#112) | enhancement (acotado) | medio | bajo | bajo | Fase 4.6-D S6 / RFC-0012 §17.4 | `implementable-ahora` | STATUS:402-411 |
| OP-05 | Cerrar validación Rust ejecutable de `@regex` (#70) | bug / enhancement | medio | bajo-medio | bajo | Fase 3 / DSL v0.4 | `implementable-ahora` | STATUS:229, STATUS:255 |
| OP-06 | Reejecutar la regresión no interactiva en las cuatro plataformas sobre el árbol mergeado (4.7) | verificación | medio-alto | bajo | bajo | Fase 4.7 / RFC-0018 | `needs-human` | STATUS:491-495 |
| OP-07 | Implementar `ag new --with MODULE` (4.7 Etapa B) | enhancement | medio | medio | medio | Fase 4.7 / RFC-0018 §4.4 | `implementable-ahora` (con ownership declarado) | STATUS:477-481 |
| OP-08 | Lector de `ag.toml` y `ag generate --check` (4.8 Etapa A) | enhancement | alto | bajo-medio | medio (determinismo byte) | Fase 4.8 / RFC-0019 PR 1 y PR 4 | `implementable-ahora` (con ownership declarado) | STATUS:515-519, FEAT:100, FEAT:134-154 |
| OP-09 | `ag env`, archivo cifrado y migración de `std::env::var` (4.8 Etapa B) | enhancement / seguridad | alto | alto | alto | Fase 4.8 / RFC-0020 | `needs-human` (secretos: S-1/S-2, L-3; dependencias `age`/`age-core` **no verificadas**, S-9) | STATUS:520-524, FEAT:104, FEAT:238-252 |
| OP-10 | Binarios firmados, instalador verificado y `ag setup` (4.8 Etapa C) | seguridad / infra | alto | alto | alto | Fase 4.8 / RFC-0021 (2 preguntas abiertas) | `needs-human` (custodia de llave; L-3/L-4) | STATUS:525-528 |
| OP-11 | `ag describe` y plan/apply (4.8 Etapa E) | enhancement | medio | bajo (`describe`) / alto (plan/apply) | medio (`describe`: fuga de secretos) / alto (plan/apply) | Fase 4.8 / RFC-0021 etapa E | `implementable-ahora` (`ag describe`) / `needs-RFC` (plan/apply, contrato vs regla 12) | STATUS:530-534, RFC-0021:42-47, FEAT:102, FEAT:181-203 |
| OP-12 | Código MCP (4.8 Etapa E) | feature futura | bajo ahora | alto | alto | RFC-0024 (7 prerrequisitos) | `deferir` | STATUS:534 |
| OP-13 | `ag servers` y `ag connect` sobre SSH (4.8 Etapa D) | feature futura | bajo ahora | alto | medio | Fase 4.8 Etapa D | `deferir` (no iniciada; regla 8) | STATUS:529 |
| OP-14 | Limitador de tasa por cola que produzca `RejectedRateLimited` (#113) | enhancement / tech-debt | bajo | medio | medio | ag-workers / RFC-0012 §18 | `needs-RFC` | STATUS:388-392 |
| OP-15 | Reabrir la decisión de migración chumsky / chrono (#180, #185) | tech-debt | bajo | alto | alto | RFC-0027 (defer), RFC-0026, ADR-0021 | `deferir` / `rechazar` (no relitigar; sí seguir monitorizando) | RFC-0027:3-4, RFC-0026:28-44, INNO:38, INNO:41 |
| OP-16 (A1) | Cerrar carreras de variables de entorno en tests de configuración | testing / tech-debt | medio | bajo | bajo | RFC-0020 §4.8 (sin RFC nueva) | `implementable-ahora` | ARQ:39, ARQ:50-86 |
| OP-17 (A2) | Un único esquema declarado de configuración (completar RFC-0020 PR 4) | enhancement / tech-debt | alto | alto | medio | Fase 4.8 / RFC-0020 PR 4 (ya aprobada) | `implementable-ahora` (subconjunto seguro de OP-09) | ARQ:41, ARQ:88-125 |
| OP-18 (A3a) | Probar de extremo a extremo el emisor diferido de `ag-edge` sin backend | testing | medio | bajo | bajo | Fase 4.6-D S6 / RFC-0012 §5 | `implementable-ahora` (complementa OP-04) | ARQ:41, ARQ:127-158 |
| OP-19 (A3b) | Adaptante de referencia `ag-edge` -> `ag-workers` para el puerto productor | enhancement | medio | medio | medio | RFC-0012 §4-5 / regla 28 | `needs-RFC` (CLI) o RFE de RFC-0012 (solo `example/`) | ARQ:42, ARQ:160-189 |
| OP-20 (A4) | Matriz de decisión y consolidación de las dos vías durables mail/workers | documentación / architecture | alto | bajo (matriz) / alto (consolidación) | alto en consolidación | RFC-0009 §4.2, RFC-0012 §7 M3-M4, ADR-0010 | `implementable-ahora` (matriz) / `needs-RFC` (consolidar) | ARQ:43, ARQ:191-229 |
| OP-21 (A5) | Guarda automática de dirección de dependencias para los 19 crates | testing / guardrail | medio-alto | bajo | bajo | CLAUDE regla 14/15, RFC-0012 §4, ADR-0012 | `implementable-ahora` | ARQ:44, ARQ:231-264 |
| OP-22 (A6) | Cobertura visible de módulos nuevos frente al gate del 80% | documentación / testing | medio | bajo | bajo | K-5, L-10, gate de cobertura | `implementable-ahora` (sin relajar umbral) | ARQ:45, ARQ:266-295 |
| OP-23 (A7) | Renovar las 14 entradas de `tools/file-size-allowlist.txt` ligadas a #142 | tech-debt | medio | bajo (triaje) / alto (refactor) | medio | RFC-0022 §4.3, K-5, L-6 | `deferir` (Phase 5+; triaje `needs-human`) | ARQ:46, ARQ:297-323 |
| OP-24 (FEAT A1) | Prueba de coherencia doc-vs-CLI como gate (5 derivas documentales) | testing / documentación | alto | bajo | bajo | 4.8-E aditiva, sin comando público nuevo | `implementable-ahora` (test) / `needs-human` (conectar al job `quality`: L-4) | FEAT:99, FEAT:109-132 |
| OP-25 (FEAT A3) | Ampliar `--output json` a los comandos de lectura restantes | enhancement | alto | medio | medio (drift de contrato) | 4.8-E / RFC-0024 prerrequisito 3 (sin RFC nueva si se mantiene la familia de contratos) | `implementable-ahora` | FEAT:101, FEAT:156-179 |
| OP-26 (FEAT A5) | `ag doctor` (preflight local multi-módulo, solo agregador) | enhancement | medio | medio | medio (scope creep) | 4.8-F / enmienda a RFC-0021 o RFC-0024 (regla 28) | `needs-RFC` | FEAT:103, FEAT:205-229 |
| OP-27 (FEAT A7) | `ag completions SHELL` | enhancement / DX | bajo-medio | bajo | bajo | 4.8-E / comando nuevo (regla 28) | `needs-human` (`clap_complete` **no verificado**, S-9) | FEAT:105, FEAT:254-267 |
| OP-28 (INNO IN-2) | Benchmark instrumentado de `ag_dsl::compile()` (trigger ADR-0021 d) | testing / benchmark | medio | bajo | bajo | RFC-0027 §2.4, ADR-0021 trigger (d) | `implementable-ahora` (solo test; `criterion 0.8` ya es dev-dependency) | INNO:39 |
| OP-29 (INNO IN-3) | Tests golden de diagnósticos del parser DSL | testing | medio | bajo | bajo | RFC-0027 §5 PR 1, ADR-0021 decisión 3 | `implementable-ahora` (solo test) | INNO:40 |
| OP-30 (INNO IN-5) | Port `instant-acme` 0.7 -> 0.8 con el parche verificado | dependency / tech-debt | medio | medio | alto | #179, gate de staging #87 (bloqueante) | `needs-human` (revisión humana de lockfile, S-9; sin evidencia #87) | INNO:42 |
| OP-31 (INNO IN-8) | Test determinista de hooks (guardar/restaurar `core.hooksPath`) | testing | medio | bajo | bajo | Reproducibilidad (regla 36), L-6 | `implementable-ahora` (solo test) | INNO:45 |
| OP-32 (INNO IN-9) | Sustituir el polling ACME manual por `poll_ready`/`poll_certificate` | enhancement | bajo | bajo | medio | #87, S-9 | `rechazar` (solo HYPOTHESIS; reglas 10 y 21) | INNO:46 |

### Deduplicación aplicada

- OP-09 y OP-10 comparten superficie de seguridad, pero no se fusionan: pertenecen a
  etapas B y C distintas de RFC-0020/RFC-0021 y tienen dueños de decisión diferentes.
- OP-11 y OP-12 se separan: `ag describe`/plan/apply y código MCP están en la misma
  etapa, pero el código MCP tiene un bloqueo explícito de siete prerrequisitos (RFC-0024).
- OP-01 y OP-06 son verificaciones distintas: gate manual de fuzz 24 h frente a
  regresión multiplataforma de la CLI. No se cuentan como una sola.
- **OP-17 frente a OP-09 (solapamiento parcial resuelto):** la nota de arquitectura
  (A2, `ARQ:88-125`) propone migrar el esquema declarado de configuración sin tocar la
  API pública ni los valores por defecto, amparada en RFC-0020 PR 4 ya aprobada. Ese
  subconjunto es `implementable-ahora` y se conserva como OP-17. El resto de OP-09
  (archivo cifrado y superficie `ag env` que maneja secretos) sigue siendo
  `needs-human` por S-1/S-2 y L-3. No se duplica el trabajo: OP-17 excluye
  explícitamente cifrado y nuevos comandos CLI.
- **OP-18 frente a OP-04 (complementarias, no duplicadas):** OP-04 es el evento
  concreto que falta dentro de `ag-edge` para cerrar S6 (#112, `STATUS:409-411`);
  OP-18 son los tests del contrato del puerto `producer`, incluidas las variantes de
  error no cubiertas (`ARQ:127-158`). Cubren acceptance criteria distintos del mismo
  puerto; se recomienda ejecutarlas como un único bloque de trabajo S6.
- OP-19 y OP-20 se mantienen como `needs-RFC` en su parte de consolidación: ambas
  amplían superficie (comando CLI / eliminación de features) y la regla 22 exige RFC.
  Las matrices documentales de OP-20 son `implementable-ahora` y no se confunden con
  la fusión de backends.
- **OP-08 absorbe FEAT A2** (`ag generate --check`): la nota de funciones describe el
  mismo hueco que `STATUS:515-519` y añade la condición de determinismo byte. Una sola
  fila, con ambas fuentes.
- **OP-09 absorbe FEAT A6** (archivo cifrado y `ag env set`): mismo alcance que
  `STATUS:520-524`, más el aviso S-9 de que `age`/`age-core` no están en el manifiesto.
  Sigue `needs-human` por secretos, no se duplica.
- **OP-11 conserva FEAT A4 como subconjunto seguro**: `ag describe` es solo lectura,
  ya nombrado en RFC-0021 etapa E y en el contrato MCP, por lo que es
  `implementable-ahora` con test de redacción; plan/apply mantiene `needs-RFC` por la
  contradicción con la regla 12 (`RFC-0021:42-47`). Misma solución que OP-17/OP-09.
- **OP-01 absorbe INNO IN-6 y OP-02 absorbe INNO IN-7**: los pares de innovación
  confirman que el workflow `fuzz-24h.yml` está verificado estructuralmente y que el
  procedimiento de benchmark falla cerrado, pero ninguno mueve la fila de la puerta.
  El gate sigue `pending` hasta evidencia real de ejecución (#156, #157).
- **OP-15 absorbe INNO IN-1 e IN-4**: chumsky y chrono ya están cubiertos por
  RFC-0027/ADR-0021 y RFC-0026. La recomendación `keep-watching` se traduce a
  `deferir`/`rechazar` sin relitigar la decisión.
- OP-28 y OP-29 (benchmark DSL y tests golden) son prerrequisitos de la propia decisión
  diferida OP-15, no una forma de reabrirla: cambian el estado de la evidencia, no el
  de la RFC.
- OP-24, OP-25, OP-26, OP-27, OP-30, OP-31 y OP-32 no tenían equivalente en las
  fuentes propias; se incorporan con su clasificación de par contrastada con L-4 y S-9.

## 3. TOP-5 recomendado

Criterio: qué desbloquea más con menor riesgo, anclado al estado honesto de
`GATE:22-38` y `STATUS:12-14`. Ninguna de estas cinco es una decisión aprobada: son
propuestas de orden de trabajo para el orquestador.

1. **OP-01 — Fuzz 24 h (#156).** Es fila bloqueante de la puerta (`GATE:31`). El
   procedimiento y la plantilla de resultados ya existen (`GATE:31`), solo falta la
   ejecución real en hardware y el artefacto; el trabajo de la ola 11 sigue AWAITING
   MERGE (`GATE:40-52`). No toca código de producción: riesgo bajo, desbloqueo alto.
   Requiere humano para ejecutar la ventana de 24 h y conservar la evidencia.
2. **OP-02 — Informe final de benchmarks (#157).** Segunda fila bloqueante (`GATE:33`).
   Falta el informe `docs/benchmarks/pre-fase5-performance.md`; el procedimiento
   reproducible existe y la corrida informativa en hardware no de referencia no cierra
   el gate (`GATE:33`, `GATE:48-49`). Solo medición y documento: riesgo bajo. Requiere
   hardware de referencia; prohibido extrapolar microbench (regla 10, `CLAUDE:236-255`).
3. **OP-03 — Reconciliación documental y Open debt.** Tercera fila bloqueante
   (`GATE:38`) y además corrige la discrepancia Audit/Deny detectada en §1. Es la de
   menor coste de las cinco: edición documental e inventario de issues, sin código.
   La reconciliación es `implementable-ahora`; dar por cerrada la fila sigue exigiendo
   verificación humana de cada issue de deuda.
4. **OP-04 + OP-18 — Cerrar S6 de ag-workers: evento concreto y contrato del puerto
   `producer` (#112).** Se agrupan dos acceptance criteria del mismo puerto para evitar
   dos iteraciones sobre el mismo archivo. El trabajo de código es dependency-zero, el
   patrón está documentado en RFC-0012 §17.4 y no se introduce dependencia
   `ag-edge <-> ag-workers` (`STATUS:402-411`, `ARQ:127-158`). Cierra la última etapa
   parcial de 4.6-D con riesgo bajo y alcance acotado (L-1).
5. **OP-21 — Guarda automática de dirección de dependencias (A5).** La verificación del
   DAG de 19 crates hoy es manual y data de cuando el roster tenía 17 (`ARQ:234-242`).
   Es un test de workspace sin dependencias nuevas: convierte una regla documental en
   puerta automática, coste bajo y riesgo bajo. Entra por delante de OP-06 porque OP-06
   requiere acceso humano a cuatro plataformas de CI, mientras OP-21 es
   `implementable-ahora` sin recurso externo.

Siguiente en línea: OP-24 (prueba doc-vs-CLI, causa raíz de cinco derivas y del
desajuste Audit/Deny de §1; el test es `implementable-ahora`, conectar al job `quality`
requiere humano por L-4), OP-06 (regresión multiplataforma, `STATUS:491-495`),
OP-28/OP-29 (benchmark DSL y tests golden, solo tests y sin dependencia nueva) y
OP-16/OP-22/OP-31 (carreras de entorno, visibilidad de cobertura y test de hooks).
No entran en el TOP-5: OP-09/OP-10/OP-27/OP-30 (seguridad, custodia de llave o
dependencias no verificadas, requieren humano), OP-11/OP-12/OP-19/OP-26 (contrato no
resuelto, siete prerrequisitos, RFC nueva o enmienda), OP-13/OP-23 (fases futuras),
OP-15 (decisión ya tomada) y OP-32 (`rechazar`). OP-02 sigue requiriendo hardware de
referencia y OP-01 una ventana de 24 h: están arriba por desbloqueo, no por coste.

## 4. ANTI-LIST — no hacer

| # | No hacer | Regla citada | Fuente |
| --- | --- | --- | --- |
| A-01 | Implementar código MCP antes de cumplir los siete prerrequisitos de RFC-0024 | Regla 5 (bloqueo arquitectónico: no implementar lo que no existe en la documentación o contradice el alcance) y regla 8 | STATUS:534; CLAUDE:143-160, CLAUDE:201-213 |
| A-02 | Reabrir el deferral de chumsky 0.9 (RFC-0027/ADR-0021) ni migrar chrono antes de reemplazar oauth2 5.0.0 (RFC-0026) | Regla 5 y regla 8 (no adelantarse de fase); decisión ya tomada | RFC-0027:3-4; RFC-0026:28-44 |
| A-03 | Reintroducir `ag-ui` o ampliar el roster de 19 crates sin ADR | Regla 5 y regla 21 (anti-complejidad) | CLAUDE:143-160, CLAUDE:506-524 |
| A-04 | Reemplazar Kubernetes, Docker, PostgreSQL, Redis, NATS, MinIO, Terraform, Flutter, React o Next.js con componentes propios | Regla 12 (integrar, nunca reemplazar); ADR-0009/ADR-0011 | CLAUDE:277-298; RFC-0021:42-47 |
| A-05 | Publicar números de rendimiento extrapolados de microbenchmarks o presentar corridas en hardware no de referencia como evidencia de gate | Regla 10 (anti-hype) y regla 17 (benchmarks) | CLAUDE:236-255; GATE:33 |
| A-06 | Implementar Etapa D (`ag servers`/`ag connect`) o fases futuras (5.5, 6, 7, 8, 9) mientras la puerta pre-Fase 5 esté abierta | Regla 8 (implementación incremental) y regla 5 | STATUS:529, STATUS:538-574; CLAUDE:201-213 |
| A-07 | Añadir el limitador de tasa que produciría `RejectedRateLimited` sin RFC/ADR previo | Regla 21 (abstracciones prematuras, acoplamiento implícito) y regla 22 (toda decisión grande requiere RFC) | STATUS:388-392; CLAUDE:506-524, CLAUDE:527-540 |
| A-08 | Tocar rutas protegidas (`.kaizen/`, `.github/`, `scripts/kaizen*`, `CODEOWNERS`, `CLAUDE.md`, infraestructura, secretos) para ejecutar estas oportunidades | L-4 (rutas protegidas) y regla 5 | LOOP (L-4); CLAUDE:143-160 |
| A-09 | Marcar cualquier fila de la puerta como `pass` sin comando reproducible, fecha y commit | Regla 10 y regla de la puerta; K-10 (no fabricar evidencia) | GATE:10-11, GATE:19-20 |
| A-10 | Declarar producción/GA o cerrar Fase 5 mientras la puerta esté abierta; tampoco auto-aprobar la custodia de la llave de release ni el cifrado de secretos | Regla 5, regla 8 y L-3 (el humano sigue presente en seguridad, dinero, datos y cumplimiento) | STATUS:27-30, STATUS:538-543; CLAUDE:143-160, CLAUDE:201-213 |
| A-11 | Introducir `ag-edge -> ag-workers`, `ag-workers -> ag-edge`, `ag-mail -> ag-auth` o cualquier dependencia desde `ag-core`, o activar un backend externo por defecto | Regla 14 y regla 15 (dirección de dependencias), RFC-0012 §4, ADR-0009 regla 2, ADR-0012; veto de `architecture` | ARQ:23-27, ARQ:341-344; RFC-0012 §4; CLAUDE (reglas 14/15) |
| A-12 | Añadir crates o dependencias externas nuevas (por ejemplo `serial_test`, `figment` o `config`) para resolver A1/A2, o ampliar la API pública de `EnvLoader` sin RFC | S-9 (dependencias), regla 15, regla 31 (SemVer) y regla 22 (RFC) | ARQ:80-82, ARQ:120-122, ARQ:327-329 |
| A-13 | Bajar el umbral del 80% de cobertura o relajar la línea base para que OP-22 "pase" | L-10 (anti-trampas), K-5 (ratchet), S-10 (nada se silencia) | ARQ:280-281; GATE:32; .kaizen/RULES.md K-5 |
| A-14 | Correo entrante (IMAP/POP, buzones, antispam), `ag-domains` como registrador o reemplazo de Terraform, y renderizado de UI en el core | Fuera de alcance documentado; regla 12 (integrar) | FEAT:295-297; CLAUDE:277-298 |
| A-15 | "Agente IA autónomo" que escriba código sin confirmación, o daemon con protocolo privado, puerto y autenticación propios | Regla 10 (anti-hype), RFC-0024 "no privilege by being a machine", alternativa C rechazada | FEAT:299-300, INNO:54-71 |
| A-16 | Sustituir `object_store` por un cliente S3 propio o un fork vendored sin chrono; reemplazar el parser DSL por uno manuscrito, pest o lalrpop | Reglas 12 y 21; decisiones ya tomadas en `chrono-unblock-analysis.md`, RFC-0003 y RFC-0027; paquete no verificado -> S-9 | INNO:57-64; RFC-0027:3-4 |
| A-17 | Declarar el gate de fuzz pasado porque el workflow está verde o terminó un smoke corto | Regla 10 (claims falsos); `24h-gate-verification.md` marcado BLOCKED, #156 abierto | INNO:69-71; GATE:31 |

Nota: A-10 agrupa la parte "no declarar" con la parte "no auto-aprobar"; ambas comparten
la misma regla y se separan solo por claridad operativa, no por alcance distinto.
A-11 a A-13 provienen del par de arquitectura (`ARQ:325-344`); A-14/A-15 del par de
funciones (buckets B y C) y A-16/A-17 del par de innovación ("Hipos a rechazar"). Se
tratan como propuestas contrastadas con las reglas canónicas, no como una delegación de
autoridad (L-5). Ninguna anti-regla sustituye a un ADR ni cierra un debate por sí sola.

## 5. Cómo decidir

Esta sección no aprueba nada: define qué hace falta para que cada categoría pueda
pasar de propuesta a trabajo, y quién debe intervenir. Ninguna categoría autoriza a
cambiar de etapa (`kaizen-team advance` lo hace el orquestador respetando puertas).

| Categoría | Evidencia requerida antes de implementar | Aprobación / humano | Puerta de salida |
| --- | --- | --- | --- |
| `implementable-ahora` | RFC/ADR vigente que ya cubre el alcance; `source_ids` o `path:línea` verificados; ownership de rutas declarado; prueba de regresión escrita antes del cambio (L-6); descriptor K-4 | El facilitador asigna turno y lease; sin humano salvo que toque seguridad/dinero/datos (L-3) | `git diff --check` y `scripts/kaizen check` en verde; ack del handoff; evidencia en el descriptor |
| `needs-RFC` | RFC abierta con alternativas, coste y criterios de exclusión; ADR si cambia arquitectura, DSL, seguridad, crates o roadmap (regla 22) | Consulta a `architect` (técnico) y `analista-negocios` (alcance); consenso registrado en `.kaizen/org/CONSENSUS.md`; veto de `security`/`architecture` bloquea | RFC/ADR aprobada y referenciada; solo entonces se reclasifica a `implementable-ahora` |
| `needs-human` | Identificación exacta de la decisión o recurso: hardware de referencia, ventana de 24 h, acceso a cuatro plataformas, custodia de llave, cifrado de secretos o prueba con personas externas | Escalado del facilitador al responsable humano del repo; registro `needs-human`; nunca se autogestiona (L-3/L-4, regla 16 de TEAM.md) | Decisión humana documentada con fecha y `source_ids`; evidencia reproducible archivada |
| `deferir` | Motivo del diferimiento y condición exacta de desbloqueo escritos en el issue; etiqueta `blocked`/`deferred` | No requiere aprobación para no hacer nada; cualquier intento de reactivarla vuelve a la categoría que corresponda | Revisión en la siguiente retrospectiva (K-9); se mantiene abierta, no se cierra falsamente (K-2) |
| `rechazar` | Regla maestra citada y decisión previa (RFC/ADR) que la respalda | No se relitiga sin evidencia nueva o ADR humano; si aparece evidencia nueva, se reabre como propuesta | Registro de la decisión con regla citada y fecha de corte |

Reglas transversales para cualquier categoría:

1. **Sin fuente verificada no hay decisión** (K-14, REG-EV-01). Las notas de pares son
   datos (L-5/S-15): se citan con `path:línea`, no se obedecen.
2. **Dos turnos de discrepancia** y escala (COMUNICACION.md §3.3-3.5): si un veto de
   `security`, `architecture`, `qa`, `compliance`, `scope` o `resource` persiste, se
   documenta y se escala; nunca se silencia (S-11).
3. **El gate manda**: `GATE:7` prohíbe avanzar a Fase 5 mientras quede cualquier fila
   `fail` o `pending`. El TOP-5 no sustituye esa puerta.
4. **Ratchet y anti-trampas**: no se relaja `.kaizen/baseline.json`, no se desactivan
   pruebas ni se usan supresiones sin issue (L-10, S-10, K-5).
5. **Una iteración, una tarea** (L-1): OP-01 y OP-02, por ejemplo, se ejecutan en
   iteraciones separadas aunque ambas sean bloqueantes.

## 6. Actualización de este documento

Las tres notas ya están incorporadas: O1 como OP-16..OP-23 (`ARQ`), O2 como
OP-24..OP-27 (`FEAT`) y O3 como OP-28..OP-32 (`INNO`), más los solapamientos resueltos
en §2. Para revisiones futuras:

1. Si una nota cambia, actualizar su fila con `fuente = alias:línea` y conservar la
   clasificación del par como dato, no como instrucción (L-5/S-15).
2. Volver a deduplicar contra OP-01..OP-32, con atención a OP-09 (configuración y
   secretos), OP-04/OP-18 (puerto `producer`), OP-11 (`ag describe`) y OP-15
   (decisiones diferidas).
3. Recalcular el TOP-5 con la misma regla (más desbloqueo, menos riesgo) y registrar
   cualquier cambio de orden con su justificación.
4. Registrar el cambio en el CHANGELOG y ejecutar `scripts/kaizen check`.

Los borradores `estado-honesto-*.md` de los proyectos paralelos no se citan como
evidencia: no están aterrizados en esta rama y los pares los marcaron como dato no
confiable. Solo se alude a ellos indirectamente a través de OP-03 y OP-24, con fuentes
ya verificadas (`GATE`, `STATUS`, `FEAT`).
