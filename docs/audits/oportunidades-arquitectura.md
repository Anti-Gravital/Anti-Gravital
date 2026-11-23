# Catalogo de oportunidades de arquitectura e ingenieria (O1 / oportunidades)

> Propuesta del rol `architect` para el proyecto `oportunidades`, tarea `O1-arquitectura`.
> Este documento **no implementa nada**: solo propone, con evidencia verificable, mejoras
> que reducen complejidad operativa o cierran brechas reales. No propone reescrituras,
> nuevas funcionalidades de producto ni crates adicionales.

- **Fecha de corte:** 2026-11-22
- **Rama:** `community15/opp-arquitectura`
- **Metodo:** lectura estatica de documentacion canonica y de los 19 `crates/*/Cargo.toml`;
  inspeccion de modulos de configuracion y sus tests; lectura del workflow de cobertura;
  lectura del tablero `oportunidades` (en el repositorio principal) como dato no confiable (L-5).
- **Fuentes:** exclusivamente documentos canonicos in-repo y comandos reproducibles. No se
  usan fuentes externas; no hay afirmaciones que dependan de contenido no verificado.
- **Convenciones citadas:** `CLAUDE.md` reglas 5/8/14/15/21/22/28/31, `.kaizen/RULES.md`
  K-1/K-5, `.kaizen/loop/LOOP.md` L-4, `.kaizen/security/SECURITY-RULES.md` S-9/S-10,
  RFC/ADR listados en `docs/rfc/` y `docs/adr/`.

## Restricciones que limitan toda propuesta

1. El roster es **19 crates** (`CLAUDE.md` regla 14; `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
   secciones 4 y 8). Ninguna propuesta anade un crate ni una dependencia externa nueva.
2. Sin dependencias inversas ni ciclos: `ag-core` no depende de ningun crate Anti-Gravital
   (`CLAUDE.md` regla 15); `ag-mail` no depende de `ag-auth` (regla 14, sexta regla de
   dependencias); `ag-edge` no depende de `ag-workers` y `ag-workers` nunca depende de
   `ag-edge` (`RFC-0012` seccion 4); `ag-domains` solo es consumido hacia abajo por
   `ag-cloud`/`ag-cli`/`ag-edge` (`ADR-0012`).
3. Sin abstracciones especulativas ni configuracion excesiva (`CLAUDE.md` regla 21).
4. Toda idea nueva requiere RFC antes de implementarse (`CLAUDE.md` reglas 5 y 22,
   `.kaizen/RULES.md` K-1); anadir comandos CLI o cambiar limites entre modulos tambien
   (`CLAUDE.md` regla 28).
5. No se tocan rutas protegidas (L-4): `.kaizen/`, `scripts/kaizen*`, `.github/`, agentes.
   Por eso las propuestas sobre gobernanza se entregan como recomendacion, no como cambio.

## Tabla priorizada

| ID | Prioridad | Oportunidad | Estado |
| --- | --- | --- | --- |
| A1 | P1 | Cerrar carreras de variables de entorno en tests de configuracion | `implementable now` |
| A2 | P1 | Un unico esquema declarado de configuracion (completar RFC-0020 PR 4) | `implementable now` (bajo RFC-0020 ya aprobada) |
| A3a | P2 | Probar de extremo a extremo el emisor diferido de `ag-edge` sin backend | `implementable now` |
| A3b | P2 | Adaptante de referencia `ag-edge` -> `ag-workers` para el puerto productor | `needs RFC` |
| A4 | P2 | Matriz de decision y consolidacion de las dos vias durables mail/workers | `needs RFC` |
| A5 | P3 | Guarda automatica de direccion de dependencias para los 19 crates | `implementable now` |
| A6 | P3 | Cobertura visible de modulos nuevos frente al gate del 80% | `implementable now` |
| A7 | P4 | Renovar las 14 entradas de `tools/file-size-allowlist.txt` que apuntan a #142 | `defer to Phase 5+` (requiere triaje humano de issues) |

---

## A1. Cerrar carreras de variables de entorno en tests de configuracion

- **Estado:** `implementable now`. Sin RFC nuevo: es correccion de pruebas, no cambio de
  comportamiento ni de API publica.
- **Problema que resuelve:** varios modulos de configuracion mutan el entorno del proceso
  dentro de tests que corren en paralelo. No hay una disciplina uniforme: `ag-auth` y
  `ag-realtime` serializan con un `Mutex`, pero el resto usa guardas de restauracion sin
  bloqueo. En `crates/ag-core/tests/config_env.rs` dos tests escriben la misma clave
  `BAD_KEY` (lineas 33 y 68) y solo la eliminan al hacer `drop` (lineas 16-22): si el
  primero termina mientras el segundo lee, el resultado depende del orden de hilos.
  Es una carrera estructural detectada por inspeccion; no se afirma que ya haya fallado en CI.
- **Diseno concreto:**
  1. Anadir, solo bajo `#[cfg(test)]`, un `static ENV_LOCK: Mutex<()>` por crate y
     adquirirlo al inicio de cada test que use `set_var`/`remove_var`, reutilizando el
     patron ya presente en `crates/ag-auth/src/config.rs:77-100` y
     `crates/ag-realtime/src/config.rs:123-126`. Cero dependencias nuevas.
  2. Los modulos a cubrir, segun evidencia: `ag-storage/src/config.rs:215-236`,
     `ag-observe/src/config.rs:124-133`, `ag-workers/src/config.rs:492-502`,
     `ag-cli/src/commands/env.rs:419,448`, `ag-cache/src/config.rs:78`,
     `ag-core/src/env.rs:328,484` y `ag-core/tests/config_env.rs:28-94`.
  3. Opcional y solo si se aprueba en A2: migrar el test a un `EnvLoader` con fuente
     inyectada, que elimina la mutacion global en lugar de serializarla.
- **Valor:** tests deterministas y CI fiable; elimina una fuente de fallos intermitentes
  dificiles de reproducir; no cambia binaries ni superficie publica.
- **Coste estimado:** ~10-20 LOC por modulo x ~7 modulos (70-140 LOC). 0 crates nuevos,
  0 dependencias nuevas, 0 cambios de Cargo.toml.
- **Riesgo:** bajo. El unico riesgo es olvidar algun test que comparta clave; se mitiga
  con una busqueda `grep -rnE 'set_var|remove_var' crates/ --include='*.rs'` en la misma PR.
- **RFC/ADR requerido:** ninguno. Se ampara en `RFC-0020` seccion 4.8 (testing) y en la
  regla 18 de `CLAUDE.md`.
- **Descartar si:** la solucion propuesta exigiera anadir el dev-dependency `serial_test`
  u otra crate externa (S-9, regla 15) o cambiara la API publica de `EnvLoader` (regla 31);
  en ese caso se eleva a RFC.
- **Evidencia:** `grep -rnE 'set_var|remove_var' crates/ --include='*.rs'` (salida inventariada
  arriba); `crates/ag-core/tests/config_env.rs:11-22,33,68`;
  `crates/ag-auth/src/config.rs:77-100`; `crates/ag-realtime/src/config.rs:123-126`;
  `Cargo.toml:38-39` (edition 2021, rust-version 1.99.1).

## A2. Un unico esquema declarado de configuracion (completar RFC-0020 PR 4)

- **Estado:** `implementable now` **bajo la RFC-0020 ya aprobada** (no requiere RFC nueva
  siempre que no se amplie la API publica ni se cambien valores por defecto).
- **Problema que resuelve:** `ag-core` ya expone `EnvLoader`/`EnvDeclaration` con validacion
  tipada y errores que nunca imprimen valores (`crates/ag-core/src/env.rs:1-60`), pero el
  propio modulo dice explicitamente que "migrates modules one pull request at a time
  (RFC-0020 section 5, PR 4)". Hoy conviven prefijos y lecturas ad-hoc
  (`STORAGE_*`, `NATS_*`/`RT_*`, `LOG_*`, `AG_WORKERS_*`, `JWT_*`) en al menos siete
  modulos, sin una declaracion unica auditable. No hay un solo lugar donde responder
  "que variables necesita este servicio".
- **Diseno concreto:**
  1. Mantener `EnvLoader` como unico punto de validacion. Cada crate declara sus claves
     con `EnvDeclaration` en su propio `config.rs` y las carga en la raiz de composicion
     mediante `ShieldConfig::from_path_with_env` / `EnvLoader::load_process`.
  2. Migrar de uno en uno, en este orden por riesgo decreciente y tamaño:
     `ag-storage` -> `ag-observe` -> `ag-workers` -> `ag-realtime` -> `ag-auth` ->
     `ag-cache` -> `ag-cli`. Un PR por crate, con tests de paridad de valores por defecto.
  3. Los tests usan fuente inyectada en lugar de `std::env`, lo que absorbe A1 sin
     `Mutex` globales. La ruta de proceso (`load_process`) queda intacta.
- **Valor:** un esquema declarado y auditable; fail-closed en arranque; coherencia con
  `ADR-0009`/`ADR-0019` (nunca imprimir secretos); elimina la deuda de PR 4 de RFC-0020
  y reduce la superficie de carreras de A1.
- **Coste estimado:** ~30-60 LOC por crate x 7 (~250-400 LOC), repartidas en 7 PR;
  0 crates nuevos, 0 dependencias nuevas. Es el coste mas alto del catalogo, por eso
  se recomienda estrictamente un crate por PR.
- **Riesgo:** medio. El riesgo es cambiar inadvertidamente un valor por defecto o el
  orden de resolucion (`RFC-0020` seccion 4.2: entorno de proceso, luego default).
  Se mitiga con tests de paridad por crate y sin tocar `load_process` en la misma PR.
- **RFC/ADR requerido:** `RFC-0020` seccion 5, PR 4 (ya aprobada). No se necesita RFC
  nueva salvo que se decida anadir una API publica de inyeccion (`EnvLoader::load_from`)
  o un formato externo de configuracion.
- **Descartar si:** la migracion exigiera anadir `figment`, `config` u otra crate (S-9,
  regla 15), o modificara un default documentado (regla 8: no adelantar fases; K-5:
  no empeorar la linea base).
- **Evidencia:** `docs/rfc/RFC-0020-environment-and-secrets.md:13-28,182-189`;
  `crates/ag-core/src/env.rs:1-60`; archivos `config.rs` de `ag-storage`, `ag-observe`,
  `ag-workers`, `ag-realtime`, `ag-auth`, `ag-cache` y `crates/ag-cli/src/commands/env.rs`.

## A3a. Probar de extremo a extremo el emisor diferido de `ag-edge` sin backend

- **Estado:** `implementable now`. Anadir tests no cambia comportamiento ni limites.
- **Problema que resuelve:** `ag-edge::producer` (feature `producer`, `RFC-0012` seccion 5
  S6) define `EnqueueRequest`, `DeferredJobProducer` y `emit`, pero no existe ningun
  consumidor ni adaptante dentro del workspace: la busqueda de `producer::emit` /
  `DeferredJobProducer` fuera de `crates/ag-edge/src/producer.rs` da cero resultados de
  produccion (los unicos `producer` externos son el `WorkersMailQueue` de `ag-mail`, que
  implementa un trait distinto). El unico test cubre el camino de exito
  (`producer.rs:132-140`); las variantes `Backpressure` y `Backend` de `ProducerError`
  (:64-88) y su `Display` no estan probadas. El puerto es correcto, pero su contrato
  ("el host suministra el adaptante") no se demuestra dentro del repositorio.
- **Diseno concreto:** ampliar el modulo de tests de `producer.rs` con un adaptante de
  prueba que devuelva `ProducerError::Backpressure` y otro que devuelva `Backend`,
  y afirmar: (a) `emit` propaga el error sin enriquecerlo con tipos de backend,
  (b) `payload_version` y `kind` llegan intactos al adaptante, (c) `Display` de
  `Backpressure` nombra la cola y no filtra datos del payload. Todo con `async-trait`,
  que ya es dependency opcional de la feature.
- **Valor:** cierra la brecha de verificacion del emisor sin tocar dependencias ni
  anadir superficie publica; documenta ejecutablemente el contrato que RFC-0012 S6
  atribuye a `ag-edge`.
- **Coste estimado:** ~60-100 LOC de tests en un solo archivo. 0 crates, 0 dependencias.
- **Riesgo:** bajo. Ninguno para produccion; la feature `producer` no esta en `default`.
- **RFC/ADR requerido:** ninguno. Referencia: `RFC-0012` seccion 5 (S6), seccion 4
  (direccion de dependencias).
- **Descartar si:** se decide que el puerto es intencionalmente externo y no merece
  cobertura in-repo; en ese caso basta una nota en `crates/ag-edge/README.md` y se
  descarta esta oportunidad.
- **Evidencia:** `crates/ag-edge/src/producer.rs:1-110,112-140`;
  `crates/ag-edge/src/lib.rs:19-20,35-36`;
  `crates/ag-edge/Cargo.toml:27-30`;
  `grep -rn 'producer::emit\|DeferredJobProducer' crates/ --include='*.rs'`.

## A3b. Adaptante de referencia `ag-edge` -> `ag-workers` para el puerto productor

- **Estado:** `needs RFC`.
- **Problema que resuelve:** el puerto de A3a no tiene una implementacion de referencia
  que demuestre el handoff a un backend real. Sin ella, cada host debe reimplementar
  la codificacion de `EnqueueRequest` a un job tipado de `ag-workers`, con riesgo de
  divergencia en `payload_version` o en la politica de backpressure.
- **Diseno concreto:** **no** anadir `ag-workers` como dependencia de `ag-edge`
  (prohibido por `RFC-0012` seccion 4). El adaptante vive en un crate consumidor:
  `ag-cli` ya tiene `ag-workers` opcional (`crates/ag-cli/Cargo.toml:42`), asi que una
  opcion es un nuevo comando o subcomando bajo feature; alternativamente un `example/`
  de `ag-cli` sin superficie de comando. El adaptante solo hace `enqueue`: no arrienda,
  no despacha, no hace spawn. Debe reutilizar el contrato de `ag-mail::workers` como
  referencia de codificacion sin copiar su logica de correo.
- **Valor:** reduce la complejidad operativa del despliegue edge/serverless al dar un
  unico camino probado de "emitir trabajo diferido en el edge".
- **Coste estimado:** ~120-200 LOC en un consumidor; 0 crates nuevos, 0 dependencias
  nuevas (`async-trait` y `ag-workers` ya estan en el workspace).
- **Riesgo:** medio. El riesgo principal es romper la direccion de dependencias; se
  mitiga con la guarda automatica A5. Segundo riesgo: convertir el adaptante en un
  orquestador de workflows, lo que esta fuera de alcance de `ag-workers` (regla 14).
- **RFC/ADR requerido:** RFC nueva si se expone como comando CLI (`CLAUDE.md` regla 28:
  "anadir comandos CLI"); si se limita a un `example/` con tests, basta una actualizacion
  de `RFC-0012` seccion 5 para registrar el adaptante de referencia.
- **Descartar si:** RFC-0012 cuestion 5 (S6) se interpreta como "el adaptante siempre
  externo por diseno" y el equipo decide no proveer implementacion de referencia; o
  si cualquier diseño exigiera `ag-edge -> ag-workers` (dependencia inversa prohibida).
- **Evidencia:** `crates/ag-edge/src/producer.rs:9-26`; `crates/ag-edge/Cargo.toml:27-30`;
  `crates/ag-cli/Cargo.toml:42`; `crates/ag-mail/src/workers.rs:252`;
  `docs/rfc/RFC-0012-ag-workers.md` secciones 4 y 5.

## A4. Matriz de decision y consolidacion de las dos vias durables mail/workers

- **Estado:** `needs RFC` para cualquier consolidacion; la matriz de decision, sola, es
  documentacion implementable ahora.
- **Problema que resuelve:** la familia de crates diferidos tiene dos caminos durables
  para trabajo de entrega: `queue-postgres` (spool del MTA nativo,
  `crates/ag-mail/src/sender/mta/spool.rs:10,133-136`) y `workers-postgres`
  (backend PostgreSQL de `ag-workers`, `crates/ag-mail/Cargo.toml:53`). Ambos anaden
  `sqlx`; `RFC-0012` seccion 7 M4 ya retiro la cola generica duplicada de `ag-mail`,
  pero el spool del MTA permanece con semantica propia. Hoy no existe un documento que
  diga que feature usar en cada despliegue, y el workflow de cobertura excluye ambas
  rutas por requerir infraestructura (`.github/workflows/quality.yml:199-205`).
- **Diseno concreto:**
  1. Paso 1 (documentacion, sin RFC): tabla `feature -> backend durable -> semantica ->
     suite de prueba -> quien la usa`, citando `RFC-0009` seccion 4.2 y `RFC-0012`
     seccion 7 M3-M4, y las suites `crates/ag-mail/tests/mta_spool_postgres.rs` y
     `crates/ag-mail/tests/workers_postgres.rs`.
  2. Paso 2 (RFC): evaluar enrutar tambien el spool programado del MTA a traves de
     `ag-workers`, de modo que exista un solo backend durable. La logica especifica
     de correo (SMTP/MTA/DKIM/bounce) permanece en `ag-mail`, como exige `RFC-0012` S7.
- **Valor:** menos combinaciones de features que soportar, menos superficie de
  infraestructura en CI y una respuesta clara para operaciones.
- **Coste estimado:** matriz ~80-120 LOC de documentacion. Consolidacion real:
  ~300-600 LOC removidas en `ag-mail` mas ajustes de tests; 0 crates, 0 dependencias
  nuevas (al contrario, se podria retirar `sqlx` de una via).
- **Riesgo:** alto si se remueve codigo sin paridad; el spool del MTA puede tener
  requisitos de firma DKIM o bounces no representables como job generico. Mitigacion:
  no eliminar nada antes de que la suite de paridad demuestre equivalencia.
- **RFC/ADR requerido:** RFC nueva para eliminar o fusionar features (regla 31: SemVer;
  regla 28: cambios de limites entre modulos). Referencias: `RFC-0009`, `RFC-0012`,
  `ADR-0010`.
- **Descartar si:** el spool del MTA tiene semantica no expresable como job de
  `ag-workers`, o hay evidencia de que ambos caminos estan en uso en produccion; en
  ese caso la matriz de decision ya cumple el objetivo y se descarta la fusion.
- **Evidencia:** `crates/ag-mail/Cargo.toml:37-53,75-84`;
  `crates/ag-mail/src/sender/mta/spool.rs:10,133-136`;
  `crates/ag-mail/tests/mta_spool_postgres.rs:8-13`;
  `crates/ag-mail/tests/workers_postgres.rs:85-86,140`;
  `.github/workflows/quality.yml:199-205`.

## A5. Guarda automatica de direccion de dependencias para los 19 crates

- **Estado:** `implementable now`. Test adicional, sin cambios de produccion.
- **Problema que resuelve:** la verificacion de direccion de dependencias es manual y
  data de 2026-05-29 (`docs/audits/pre-fase5-architecture.md:48-62`), cuando el roster
  era de 17 crates. Desde entonces se anadieron `ag-lsp`, `ag-edge` y `ag-workers`, con
  aristas nuevas documentadas: `ag-edge -> ag-domains` (`crates/ag-edge/Cargo.toml:36`),
  `ag-mail -> ag-workers` (`crates/ag-mail/Cargo.toml:84`), `ag-cli -> ag-workers`
  (`crates/ag-cli/Cargo.toml:42`). No existe una prueba que falle si aparece una
  dependencia inversa o una nueva dependencia externa no justificada; la unica prueba
  de direccion que hay (`crates/ag-cli/tests/wave4_regression.rs:350`) revisa los
  templates generados, no el workspace.
- **Diseno concreto:** un test de workspace (por ejemplo en `crates/ag-cli/tests/` o
  como script invocado por CI) que lea todos los `crates/*/Cargo.toml` con `std::fs`
  y afirme: (a) `ag-core` no declara ninguna dependencia `ag-*`; (b) toda arista `ag-*`
  pertenece a una lista blanca derivada de `CLAUDE.md` regla 14 y `RFC-0012` seccion 4;
  (c) ninguna dependencia externa nueva aparece sin una referencia `RFC-XXXX` en
  comentario en la misma linea. Cero dependencias nuevas.
- **Valor:** convierte una regla documental en una puerta automatica; evita que el
  proximo crate que se anada rompa el DAG sin que nadie lo note.
- **Coste estimado:** ~80-120 LOC de test, 0 crates, 0 dependencias. Requiere tocar un
  workflow de CI solo si se quiere ejecutar en PR; como test de workspace ya corre con
  `cargo test --workspace`.
- **Riesgo:** bajo. La lista blanca genera friccion deseada: cada arista nueva exige
  actualizar el test y citar su RFC, lo cual es exactamente el control buscado.
- **RFC/ADR requerido:** ninguno para el test. si se decide cambiar una regla de
  dependencias, eso si exige RFC (regla 22).
- **Descartar si:** ya existe una puerta equivalente en CI que valide el grafo del
  workspace (no se encontro; solo la de templates); en ese caso se extiende esa en
  lugar de duplicarla.
- **Evidencia:** `docs/audits/pre-fase5-architecture.md:48-62,107-115`;
  `crates/ag-edge/Cargo.toml:36`; `crates/ag-mail/Cargo.toml:84`;
  `crates/ag-cli/Cargo.toml:38-42`; `crates/ag-cli/tests/wave4_regression.rs:350`;
  `grep -rnE '^ag-[a-z-]+ *=' crates/*/Cargo.toml`.

## A6. Cobertura visible de modulos nuevos frente al gate del 80%

- **Estado:** `implementable now`. Documentacion y tests; sin cambios de produccion.
- **Problema que resuelve:** CI aplica `cargo tarpaulin --workspace --fail-under 80` con
  una lista de exclusiones (`.github/workflows/quality.yml:175-215`). El gate agregado
  es sano, pero no da visibilidad por modulo. Modulos anadidos recientemente quedan
  "dentro del 80%" sin que sea evidente que ramas nuevas estan cubiertas: `ag-edge`
  `producer.rs` tiene un solo test (A3a) y `ag-observe/src/health.rs` (201 lineas,
  commit `c1b918d`) tiene tres (`health.rs:145-176`). No es un defecto de cobertura
  global: es una brecha de observabilidad del propio gate.
- **Diseno concreto:**
  1. Anadir a este documento (o a `docs/audits/pre-fase5-...` correspondiente) una tabla
     por modulo nuevo: modulo, lineas de codigo, tests, rama no cubierta conocida, comando
     exacto de medicion (`cargo tarpaulin -p ag-edge --fail-under 80`, etc.).
  2. Cerrar las ramas triviales ya identificadas en A3a (`Backpressure`/`Backend`).
  3. No bajar ni relajar el umbral del 80% (L-10, K-5).
- **Valor:** el gate deja de ser una cifra agregada y se convierte en una herramienta
  de decision; reduce el riesgo de que un modulo nuevo entre sin pruebas "gracias" al
  promedio del workspace.
- **Coste estimado:** ~40 LOC de documentacion + ~40-60 LOC de tests (los de A3a).
  0 crates, 0 dependencias.
- **Riesgo:** bajo. Unico riesgo es que la tabla quede desactualizada; se mitiga
  generando la lista de modulos desde `git log` en cada retrospectiva.
- **RFC/ADR requerido:** ninguno. No se cambia CI ni umbrales.
- **Descartar si:** el workflow ya publica cobertura por crate con gate propio; en ese
  caso basta enlazarla en lugar de mantener una tabla manual.
- **Evidencia:** `.github/workflows/quality.yml:175-215`;
  `git show --stat c1b918d` (`crates/ag-observe/src/health.rs` 201 lineas);
  `crates/ag-observe/src/health.rs:145-176`;
  `crates/ag-edge/src/producer.rs:112-140`.

## A7. Renovar `tools/file-size-allowlist.txt` (14 entradas ligadas a #142)

- **Estado:** `defer to Phase 5+`. La renombrada de entradas requiere que un humano
  cree los issues focalizados (el orquestador, no este agente, crea issues); las
  refactorizaciones de los 14 archivos son trabajo de Phase 5+.
- **Problema que resuelve:** el archivo declara 14 excepciones de longitud (limite 600
  lineas, RFC-0022 seccion 4.3) todas referidas al issue padre `#142`, con la nota
  "Replace it with a focused refactoring issue when work on an entry starts". Mientras
  no se sustituyan, la deuda de tamano es invisible en el tablero y la lista solo
  puede encogerse (K-5): no se puede empezar ninguna refactorizacion sin un issue claro.
- **Diseno concreto:** (1) triaje humano: crear un issue por archivo o agrupaciones
  coherentes; (2) sustituir `#142` por el numero real en cada linea; (3) solo entonces
  refactorizar un archivo por PR y borrar su linea del allowlist en esa misma PR.
  No se tocan rutas protegidas: `tools/file-size-allowlist.txt` no lo es, pero la
  creacion de issues y la decision de que archivos priorizar si son competencia humana.
- **Valor:** deuda explicita y accionable (regla 29); permite que la lista de excepciones
  efectivamente disminuya.
- **Coste estimado:** triaje humano (no LOC); refactorizacion: una PR por archivo,
  fuera del alcance de esta propuesta. 0 crates, 0 dependencias.
- **Riesgo:** medio si se refactoriza antes de tener pruebas de caracterizacion; se
  mitiga con el orden "issue primero, prueba primero (L-6), refactor despues".
- **RFC/ADR requerido:** ninguna para el triaje; RFC-0022 seccion 4.3 es la norma
  aplicable. La refactorizacion de un crate no requiere RFC si no cambia limites.
- **Descartar si:** ya existe un issue focalizado por entrada (no se encontro en la
  lectura del tablero ni en las notas del allowlist).
- **Evidencia:** `tools/file-size-allowlist.txt:1-14`; `docs/rfc/RFC-0022-maintainability-guardrails.md`
  seccion 4.3; `.kaizen/RULES.md` K-5; `.kaizen/loop/LOOP.md` L-6.

## Coherencia con el roster y el veto de arquitectura

- Ninguna propuesta anade un crate: el roster sigue siendo 19.
- Ninguna propuesta anade una dependencia externa nueva (S-9); A1, A3a, A5 y A6 usan
  solo `std` y dependencias ya presentes; A2 y A4 reducen o reutilizan las existentes.
- Ninguna propuesta introduce una dependencia inversa: A3b coloca el adaptante en un
  consumidor, nunca en `ag-edge`; A2 no cruza capas; A5 lo verifica automaticamente.
- Ninguna propuesta introduce abstracciones especulativas (regla 21): A3a y A6 son
  pruebas; A1 y A5 son endurecimiento; A2 completa un plan ya aprobado; A3b y A4
  requieren RFC precisamente porque ampliarian superficie.
- Sobre la clasificacion `needs human (protected path)`: no se uso para ninguna
  oportunidad porque todas las propuestas evitan deliberadamente las rutas protegidas
  de L-4 (`.kaizen/`, `scripts/kaizen*`, `.github/`, instrucciones de agentes, CI).
  La unica accion que requiere humano es el triaje de issues de A7, y esa competencia
  es del orquestador, no una modificacion de una ruta protegida; por eso A7 se marco
  como `defer to Phase 5+` con la nota de triaje humano.
- Veto de arquitectura (si alguna de estas propuestas se implementa mal): se bloquea
  cualquier cambio que anada `ag-edge -> ag-workers`, `ag-workers -> ag-edge`,
  `ag-mail -> ag-auth`, dependencias de `ag-core` hacia otros crates, o una feature
  que active un backend externo por defecto (ADR-0009 regla 2).

## Pendiente de pares

El tablero `estado-honesto` (proyecto paralelo) aun no tiene entregables en
`docs/audits/estado-honesto-*.md` al momento de escribir esto (verificacion: `find`
en el repositorio principal y en los worktrees de los pares). Esta catalogo no contradice
sus hallazgos: cuando aterricen, conviene contrastar A2 y A7 con su inventario de deuda
antes de priorizar su ejecucion. El contenido del tablero se leyo como dato no confiable (L-5).
