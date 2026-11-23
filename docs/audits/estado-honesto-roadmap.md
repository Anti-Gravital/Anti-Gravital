# Auditoría: estado honesto del roadmap (tarea E1)

> Fecha de corte: **2026-11-22**. Rama: `community15/status-roadmap`.
> Alcance: medir el progreso **verificable real** de la Hoja de Ruta, fase por
> fase y global. No es una encuesta de opinión ni una proyección: es un conteo
> reproducible de casillas con evidencia mergeada.
>
> Esta auditoría **no cierra ninguna fase ni abre ninguna puerta**. Solo mide.

---

## 1. Regla de conteo (explícita y reproducible)

1. **Unidad** = una casilla markdown de criterio verificable (`- [x]`, `- [/]`,
   `- [ ]`) que aparece bajo el encabezado `## Fase ...` de la hoja de ruta.
2. **Fuente canónica por fase**:
   - Fases **0 a 4.8**: el tablero vivo `docs/roadmap/STATUS.md` (declarado
     "estado vivo del tablero" en `docs/roadmap/README.md:7-8`; convención de
     casillas en `docs/roadmap/STATUS.md:8-10`).
   - Fases **5, 5.5, 6, 7, 8, 9 y 10**: `STATUS.md` las declara "Pendiente /
     sin iniciar" **sin casillas propias** (`STATUS.md:538-574`), así que se
     cuentan las casillas de su archivo de fase (`docs/roadmap/fase-*.md`),
     todas `- [ ]` porque los archivos de fase son verbatim del maestro y
     nunca se marcan (`docs/roadmap/README.md:58-62`).
   - La Fase 4.6 se cuenta por su desglose detallado `4.6-D / ag-workers`
     (`STATUS.md:349`); el archivo `fase-04-6-*.md` es un resumen de gran
     orden del mismo conjunto y **no se cuenta dos veces**.
3. **Clasificación**:
   - `Hecho` = casilla `- [x]` **con evidencia mergeada** (commit, test/CI,
     medida fechada o artefacto versionado). `STATUS.md:8` define `[x]` como
     "cumplido y verificable en el repositorio". La evidencia vive en el
     código/CI mergeado, no se repite en el texto de cada casilla; se
     verifica por muestreo en §5.
   - `Parcial` = casilla `- [/]` (cumplimiento declarado con explicación,
     `STATUS.md:9`). **No se le da crédito de hecho.**
   - `Pendiente` = casilla `- [ ]`.
4. **Porcentaje** = `Hecho / Total`, **truncado a 1 decimal, sin redondeo al
   alza**. Un `Parcial` nunca se cuenta como `Hecho`. La aspiración (planes,
   RFCs aceptadas, "en curso", promesas de fecha) **no cuenta**.

Esta regla se puede recomputar con el comando de §9.

---

## 2. Progreso por fase

| Fase | Ítems verificables | Hecho (con evidencia) | Parcial | Pendiente | % hecho | Fuente |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 0 - Fundaciones y gobernanza | 31 | 23 | 0 | 8 | 74.1% | `STATUS.md:34-82` |
| 1 - The Shield MVP | 26 | 15 | 4 | 7 | 57.6% | `STATUS.md:85-129` |
| 2 - The Core MVP | 21 | 17 | 0 | 4 | 80.9% | `STATUS.md:131-209` |
| 3 - Anti-DSL alpha | 27 | 18 | 5 | 4 | 66.6% | `STATUS.md:211-267` |
| 4 - Módulos estándar | 25 | 18 | 2 | 5 | 72.0% | `STATUS.md:269-306` |
| 4.5 - ag-mail y ag-domains | 29 | 27 | 2 | 0 | 93.1% | `STATUS.md:308-347` |
| 4.6 - MTA / ag-workers | 21 | 20 | 1 | 0 | 95.2% | `STATUS.md:349-450` |
| 4.7 - TUI de `ag` | 10 | 5 | 2 | 3 | 50.0% | `STATUS.md:453-499` |
| 4.8 - Fundaciones del proyecto | 5 | 0 | 3 | 2 | 0.0% | `STATUS.md:503-534` |
| 5 - ag-cloud | 27 | 0 | 0 | 27 | 0.0% | `fase-05-ag-cloud.md:14+` |
| 5.5 - Funciones serverless | 19 | 0 | 0 | 19 | 0.0% | `fase-05-5-funciones-serverless.md:23+` |
| 6 - ag-ai y Knowledge Graph | 19 | 0 | 0 | 19 | 0.0% | `fase-06-ag-ai-knowledge-graph.md:14+` |
| 7 - ag-migrate | 17 | 0 | 0 | 17 | 0.0% | `fase-07-ag-migrate.md:14+` |
| 8 - ag-mobile | 16 | 0 | 0 | 16 | 0.0% | `fase-08-ag-mobile.md:14+` |
| 9 - Plugins WASI | 17 | 0 | 0 | 17 | 0.0% | `fase-09-plugins-wasi.md:14+` |
| 10 - Endurecimiento y 1.0 | 25 | 0 | 0 | 25 | 0.0% | `fase-10-endurecimiento-y-1.0.md:14+` |
| **TOTAL (0–10)** | **335** | **143** | **19** | **173** | **42.6%** | ver §1 y §9 |

Notas sobre la tabla:

- Los conteos de fases terminadas arrancan en los encabezados citados; el
  desglose exacto es reproducible (§9).
- La Fase 4.8 muestra 0 hechos aun cuando la ola 4 entregó partes de las
  etapas A, B y E: **todas sus casillas están `- [/]` o `- [ ]`**
  (`STATUS.md:515-534`), ninguna `- [x]`. Contar "trabajo parcial" como
  avance inflaría el resultado; por eso es 0.0%.
- La Fase 4.7 está al 50.0%: la Etapa B sigue `- [/]` porque
  `ag new --with MODULE` no existe en el árbol mergeado (`STATUS.md:477-481`).
- Las fases 5–10 están al 0.0% **no por falta de diseño** (hay RFCs
  aceptadas, p. ej. `STATUS.md:548-550`, `fase-05-5-...:23`), sino porque
  ninguna casilla verificable está marcada: diseño ≠ implementación.

---

## 3. Global: cifra exacta

**Progreso global del roadmap completo (Fases 0–10) = 143 / 335 = 42.6%**
(numerador 143 casillas `- [x]` con evidencia; denominador 335 casillas
verificables; 19 parciales y 173 pendientes no cuentan como hechos).

Dos vistas complementarias, para que nadie pueda acusar de sesgo:

| Vista | Numerador / denominador | % | Qué significa |
| --- | --- | ---: | --- |
| **Roadmap completo (0–10)** | 143 / 335 | **42.6%** | Progreso real del plan entero. Es la cifra principal. |
| Solo fases iniciadas (0–4.8) | 143 / 195 | 73.3% | Progreso dentro de lo que ya se ha tocado. **Excluye la mitad del plan** (fases 5–10, 140 casillas, todas pendientes). |

La diferencia entre 42.6% y 73.3% **es la noticia**: quien reporte "vamos al
73%" está midiendo solo la mitad ya empezada del roadmap. El estado honesto
del proyecto entero es **42.6%**.

**Fases formalmente completadas: 0 de 16.** Ninguna fase tiene todas sus
casillas de salida marcadas. La Regla de Oro 1 lo prohíbe
(`docs/roadmap/reglas-de-oro.md`, "First rule": *"A phase is not considered
concluded until all of its exit criteria boxes are checked. No exceptions."*).

---

## 4. Fase actual y qué significa "completo"

**Fase actual (2026-11-22):** el proyecto está terminando la consolidación de
las fases 4.x aditivas y **no ha entrado en Fase 5**. Estado declarado:

- Fase 4.6-D: "En curso" (`STATUS.md:349-356`); S6 parcial, Issue #112 abierto
  (`STATUS.md:402-411`).
- Fase 4.7: "En curso"; etapas A y C hechas, B parcial (`STATUS.md:453-460`).
- Fase 4.8: "parcialmente implementada" con la Etapa C abierta
  (`STATUS.md:503-511`).
- Fase 5 (`ag-cloud`): **Pendiente**, bloqueada por la puerta
  (`STATUS.md:538-543`).
- Fase 0 sigue "En curso" por entregables externos (`STATUS.md:34-36, 65-70`).

La **puerta pre-Fase 5 sigue ABIERTA** (`PRE_FASE5_RELEASE_GATE.md:10`).
De sus filas bloqueantes, tres siguen sin `pass`:

| Fila | Estado | Evidencia / issue | Línea |
| --- | --- | --- | --- |
| Fuzz 24h | `pending` | issue #156; solo smoke 30 s | `PRE_FASE5_RELEASE_GATE.md:31` |
| Benchmarks | `pending` | issue #157; `docs/benchmarks/pre-fase5-performance.md` no existe | `PRE_FASE5_RELEASE_GATE.md:33` |
| Open debt | `pending` | deuda `tech-debt` en el tablero de Issues | `PRE_FASE5_RELEASE_GATE.md:38` |

(Nota: las filas Audit y Deny **ya están en `pass`** desde el 2026-11-22,
commit mergeado `800b671`; `PRE_FASE5_RELEASE_GATE.md:28-29, 50-51`. Cualquier
texto que las siga llamando `fail` está desactualizado.) La lista
"Ready to advance to Phase 5" tiene **1 de 14 casillas marcada**
(`PRE_FASE5_RELEASE_GATE.md:76-94`).

### Qué significa "completo" (producto completo y consistente)

No significa "casi todas las casillas hechas". Significa, acumulativamente:

1. **Cobertura**: todas las casillas de criterios de salida de la fase en
   `- [x]` (Regla de Oro 1).
2. **Puerta**: las 15 filas bloqueantes en `pass` y las 14 casillas de
   "Ready to advance" marcadas — hoy 3 filas `pending`, 13 casillas sin marcar.
3. **Consistencia documental**: `README.md`, `STATUS.md`, `calendar.md` y el
   maestro dicen lo mismo, sin afirmaciones de "implementada" contra una
   puerta abierta. Hoy hay desajustes concretos (§6).
4. **Criterios duros medidos, no prometidos**: los objetivos de rendimiento
   se alcanzan en hardware de referencia o se documentan como no cumplidos
   sin cerrar la casilla (Regla de Oro 5).

Con esos cuatro criterios, **ninguna fase del 0 al 10 está completa hoy**, y
el producto **no es "producto completo y consistente"**: es un alpha
instalable desde fuente (`README.md:47, 49-50`) con puertas abiertas.

---

## 5. Verificación de evidencia (comandos ejecutados, sin secretos)

Se ejecutaron sobre la rama `community15/status-roadmap`:

- `git cat-file -t <hash>` para 12 commits citados en `STATUS.md` y el gate
  (`5aa442d, 7d41930, 47ae623, 9a541cd, 01f256a, c7e01a9, fd8b882, 133a390,
  ff85c6f, 635e623, 800b671, 7cbc28a`): **los 12 existen y sus mensajes
  coinciden con el criterio citado**. p. ej. `5aa442d` = "feat(fase-3):
  compilador Anti-DSL v0.1".
- Comprobación de artefactos versionados: existen `LICENSE`, `README.md`,
  `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `GOVERNANCE.md`,
  `CLAUDE.md`, `.github/workflows/ci.yml`, `docs/master/`, `docs/adr/`,
  `docs/manual/`, `docs/dsl/`, `docs/releases/0.1.0-alpha.2.md`,
  `docs/fuzz/24h-gate-verification.md`, `docs/dsl/referencia-v01-v04.md`,
  `crates/ag-core/config.example.toml` y los 19 crates `ag-*`.
- `tests/shield_full_pipeline.rs` **no está en la raíz** como dice
  `STATUS.md:113`; está en `crates/ag-core/tests/shield_full_pipeline.rs`.
  Evidencia presente, ruta citada imprecisa (hallazgo menor, no cambia el
  conteo).
- `docs/benchmarks/pre-fase5-performance.md` **no existe**, coherente con la
  fila Benchmarks `pending` (`PRE_FASE5_RELEASE_GATE.md:33`). No se cuenta
  como hecho.

**Límite de esta verificación**: la evidencia de muchas casillas `[x]` es el
código/CI mergeado, no una cadena repetida en la línea de la casilla. Un
filtro textual estricto marcaría 90 de 143 casillas como "sin evidencia en la
misma línea", lo cual **subestimaría** el avance (p. ej.
"Middleware de rate limiting ... en `shield::rate_limit`", `STATUS.md:108`,
está respaldado por el crate fusionado). Por eso el conteo adopta la
convención declarada del tablero (`STATUS.md:8`) más verificación por
muestreo, y no el filtro textual. Este informe **no afirma** que cada una de
las 143 casillas tenga un commit citado en su propia línea.

---

## 6. Lo que NO está hecho

Lista explícita de huecos verificables ( no aspiración ):

- **Criterios duros de rendimiento incumplidos** (están `- [ ]`, no se
  maquillan):
  - Shield ≥ 300K req/s y p99 ≤ 1 ms (`STATUS.md:120-121`).
  - CRUD + PostgreSQL ≥ 40K req/s y p99 ≤ 5 ms: medido 14 478 req/s y p99
    14.6 ms en Ryzen 5 2500U, **por debajo del objetivo** (`STATUS.md:177-191`).
  - ag-realtime 50K conexiones WS y ag-cache 1M ops/s: sin benchmark
    (`STATUS.md:303-304`).
- **Criterios externos de comunidad pendientes** en casi todas las fases:
  primer star, 5 personas en Discord, 10/50/200/500 stars, contribuidores
  externos, 100 instalaciones del plugin (`STATUS.md:65-70, 76-77, 97-99,
  129, 208-209, 264-267, 305-306`).
- **Fuzzing 24h** no ejecutado (issue #156); solo smoke 60 s
  (`STATUS.md:245-248`, `PRE_FASE5_RELEASE_GATE.md:31`).
- **Benchmark final pre-Fase 5** no escrito (issue #157); el informe
  `pre-fase5-performance.md` no existe (`PRE_FASE5_RELEASE_GATE.md:33`).
- **Deuda técnica abierta** en el tablero de Issues; `docs/DEBT.md` es solo
  registro histórico (`PRE_FASE5_RELEASE_GATE.md:38`).
- **Cierre de Fase 4.5**: falta reejecutar los gates sobre el commit final de
  consolidación (`STATUS.md:347`, casilla `- [/]`).
- **Fase 4.6**: S6 parcial, falta evento concreto en `ag-edge` que emita por
  el puerto (issue #112, `STATUS.md:402-411`).
- **Fase 4.7**: `ag new --with MODULE` no implementado; suite de regresión
  sin reejecutar en las 4 plataformas sobre el árbol mergeado; sin prueba con
  tres personas externas (`STATUS.md:477-481, 490-496`).
- **Fase 4.8**: sin `ag.toml` reader, sin `ag generate --check`, sin archivo
  cifrado, sin comandos `ag env`, sin binarios firmados, sin `ag setup`, sin
  `ag servers`/`ag connect`, sin `ag describe`/plan/apply, sin código MCP
  (RFC-0024 bloqueada por 7 prerrequisitos) (`STATUS.md:515-534`).
- **Fases 5 a 10 completas**: 140 casillas verificables, todas `- [ ]`,
  incluido `ag-cloud` (skeleton), ag-ai, ag-migrate, ag-mobile, plugins WASI,
  endurecimiento y 1.0 (§2).
- **Inconsistencias documentales** que impiden decir "consistente":
  - `README.md:54` afirma que `STATUS.md` tiene última actualización
    2026-11-10, pero `STATUS.md:12` dice 2026-11-22.
  - `README.md:1031` describe la Fase 4.8 como "Aceptada, sin iniciar" y
    "sin código aun", mientras `STATUS.md:505-511, 530-534` la declara
    parcialmente implementada.
  - `README.md:1030` da Etapa B de 4.7 como "hecha"; `STATUS.md:477-481` la
    mantiene `- [/]` porque falta `ag new --with`.
  - `calendar.md` y `README.md:49` sitúan el frente en 4.6, sin reflejar el
    trabajo 4.7/4.8 de `STATUS.md`.

---

## 7. Por qué el % no es mayor

El 42.6% no es bajo por pesimismo, sino por cuatro razones medibles:

1. **La mitad del roadmap no ha empezado**: las fases 5–10 suman 140 de 335
   casillas (41.8% del total) y todas están pendientes. Solo esto limita el
   techo global a ~58.2% aunque todo lo ya iniciado estuviera perfecto.
2. **Los criterios "difíciles" están sin cerrar a propósito**: rendimiento,
   fuzz 24h, benchmarks, adopción y deuda son casillas `- [ ]` o `- [/]`.
   Cerrarlas "de palabra" elevaría el número pero rompería la Regla de Oro 5
   (toda promesa necesita evidencia).
3. **Las fases aditivas 4.7 y 4.8 apenas arrancan**: 10 y 5 casillas,
   con 50.0% y 0.0% hechos. Absorben trabajo reciente sin cerrar casillas.
4. **El tablero no da crédito por "en curso"**: trabajo parcial se registra
   `- [/]` y cuenta 0 en el numerador. Eso es exactamente lo que pide una
   medición honesta.

Si mañana se cerraran los tres huecos de la puerta y se reejecutara todo,
el % subiría; hoy hacerlo supondría contar evidencia inexistente.

---

## 8. Fuentes y `source_ids`

- **No se citan `source_ids`**: `docs/org/knowledge/SOURCES.md` regula fuentes
  normativas (Panamá, ISO, NIST…). Esta auditoría mide progreso de código,
  no decide sobre cumplimiento, dinero, datos ni arquitectura; no aplica
  ninguna de esas fuentes. REG-EV-01 no se invoca porque no hay decisión con
  fuente externa que validar.
- Toda cifra se apoya en **archivos versionados (`file:line`), commits
  verificables y comandos reproducibles** (§2, §5, §9). Contenido de
  `STATUS.md`, README, calendar, gate y phase files se trata como dato
  versionado del repo, no como instrucción externa (S-15/L-5).
- Todo hallazgo menor de esta auditoría (ruta de `shield_full_pipeline.rs`
  imprecisa, desajustes README/STATUS) se reporta además por el outbox S-11
  según `.kaizen/security/SECURITY-RULES.md`.

---

## 9. Cómo recomputar (reproducible)

Desde la raíz del repo, sobre la rama `community15/status-roadmap`:

```bash
python3 - <<'PY'
import re
s=open('docs/roadmap/STATUS.md',encoding='utf-8').read().splitlines()
def st(prefix):
    c={'x':0,'/':0,' ':0}; on=False
    for l in s:
        if l.startswith('## '): on=l.strip().startswith('## '+prefix); continue
        if on:
            m=re.match(r'^- \[([x /])\]',l.strip())
            if m: c[m.group(1)]+=1
    return c
def fl(p):
    c={'x':0,'/':0,' ':0}
    for l in open(p,encoding='utf-8'):
        m=re.match(r'^- \[([x /])\]',l.strip())
        if m: c[m.group(1)]+=1
    return c
act=[('0','Fase 0 - Fundaciones y gobernanza'),('1','Fase 1 - The Shield MVP'),
     ('2','Fase 2 - The Core MVP'),('3','Fase 3 - Anti-DSL alpha'),
     ('4','Fase 4 - Modulos estandar'),('4.5','Fase 4.5 - ag-mail y ag-domains'),
     ('4.6','Fase 4.6-D - ag-workers'),('4.7','Fase 4.7 - Experiencia interactiva'),
     ('4.8','Fase 4.8 - Fundaciones del proyecto')]
pen=[('5','fase-05-ag-cloud'),('5.5','fase-05-5-funciones-serverless'),
     ('6','fase-06-ag-ai-knowledge-graph'),('7','fase-07-ag-migrate'),
     ('8','fase-08-ag-mobile'),('9','fase-09-plugins-wasi'),
     ('10','fase-10-endurecimiento-y-1.0')]
g={'x':0,'/':0,' ':0}
for _,p in act:
    c=st(p)
    for k in g: g[k]+=c[k]
for _,f in pen:
    c=fl('docs/roadmap/'+f+'.md')
    for k in g: g[k]+=c[k]
t=sum(g.values())
print(g,'total',t,'done%',int(100*g['x']/t*10)/10)
PY
# Salida esperada: {'x': 143, '/': 19, ' ': 173} total 335 done% 42.6
```

Verificación de integridad del cambio: `git diff --check` y escaneo de
contenido (§ "Verify" del descriptor).
