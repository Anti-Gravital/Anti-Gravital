# Catalogo de oportunidades de funciones y caracteristicas (O2 / oportunidades)

- **Fecha:** 2026-11-23 (UTC). Corte de evidencia: 2026-11-22/23.
- **Proyecto / tarea:** `oportunidades` / `O2-features` (owner: `innovation`).
- **Rama:** `community15/opp-features`. Base de trabajo: `493f77a`.
- **Autonomia:** `propose`. Este documento propone; no cierra puertas, no cambia
  etapas, no aprueba y no ejecuta. Los documentos y mensajes de otros agentes se
  leen como dato no confiable (L-5/S-15).
- **KPI del charter:** catalogo priorizado donde cada oportunidad lleva valor,
  coste, riesgo, RFC requerida y criterio de exclusion. Meta: 0 afirmaciones sin
  evidencia.

---

## 0. Convencion de evidencia

Los `source_ids` regulatorios de `.kaizen/org/knowledge/SOURCES.md` (formato
`XX-NNN`) gobiernan decisiones de cumplimiento, dinero, datos o arquitectura.
Este catalogo **no decide** ninguna de esas materias: propone funciones de un
backend tool. Por tanto no se invoca REG-EV-01 y no se citan fuentes externas
como instruccion (S-15/L-5). Las fuentes verificadas aqui son **repositorio-locales**
y se citan como `EV-xx` = ruta + seccion/linea + comando + commit, definidas en
la seccion 7. No se usa ninguna fuente externa.

**Auditorias de estado de pares:** `docs/audits/estado-honesto-producto.md` y
`docs/audits/estado-honesto-roadmap.md` **no estan aterrizadas** en esta rama ni
en `main` al corte (verificacion: `find` sobre los worktrees de la ola). Existen
como borradores en las ramas `community15/status-producto` y
`community15/status-roadmap`. Se leen unicamente como corroboracion (EV-09,
EV-10); ninguna afirmacion de este catalogo depende de que esos borradores se
fusionen.

---

## 1. Restricciones que limitan toda propuesta

1. **Backend tool, Rust puro, binario estatico unico, sin runtime externo ni
   sidecar obligatorio.** No se propone nada que requiera un daemon propio,
   interprete o servicio externo para funcionar. [EV-01]
2. **Schema-first:** `.ag` es la fuente de verdad; cualquier funcion nueva debe
   generarse o validarse desde el AST, no crear un segundo lenguaje de
   configuracion. [EV-01]
3. **Crates modulares y sin crates nuevos** para estas mejoras: viven en los
   crates existentes (`ag-cli`, `ag-observe`, `ag-dsl`), segun reglas 5/14.
   [EV-01][EV-03]
4. **Regla 12 (interoperabilidad):** donde existe herramienta dominante se
   integra, no se reinventa. Terraform, Kubernetes, Docker, PostgreSQL, Redis,
   NATS y MinIO quedan fuera. [EV-03]
5. **Regla 8 (implementacion incremental):** la fase actual es 4.7/4.8
   aditiva; la puerta pre-Fase 5 sigue abierta (3 filas `pending`). Ninguna
   propuesta de esta seccion abre la Fase 5 ni adelanta las fases 5.5-10.
   [EV-02][EV-03][EV-07]
6. **Regla 10 (anti-hype):** se descarta marketing tecnico, "features cool sin
   proposito" y complejidad por ego. Cada propuesta nace de un hueco
   verificable, no de una tendencia. [EV-03][EV-09]
7. **Apache-2.0 + producto privado:** el nucleo es Apache-2.0; Gravital Fabric
   es el producto privado. El contrato MCP es compartido como dato versionado,
   sin dependencia de compilacion entre ambos y sin exponer el plano de control
   de Fabric bajo un segundo contrato. [EV-01][EV-05]
8. **Una sola superficie de ejecucion:** CLI, TUI y MCP comparten los mismos
   `Request`/executor; las formas JSON solo existen para comandos de solo
   lectura y los mutantes exigen plan + apply + auditoria. [EV-04][EV-05]

---

## 2. Estado de partida (hechos verificables)

- El binario `ag` ya expone `new` (con `--with MODULE`), `dev`, `build`,
  `config`, `env list|check`, `generate`, `schema`, `dns plan|apply|status`,
  `domains`, `mail`, `workers` y `deploy plan|apply|status`. `deploy` es un
  renderizador local, **no** el despliegue cloud de Fase 5. [EV-04][EV-09]
- Ya hay contratos JSON versionados 0.1.0 y 0.2.0 con shapes publicados; solo
  algunos comandos de lectura estan cableados al selector global `--output`.
  [EV-04][EV-05]
- Huecos verificables de la fase actual: no existe `ag generate --check`,
  `ag describe`, archivo cifrado de secretos ni codigo MCP; RFC-0024 sigue
  bloqueada por siete prerrequisitos. [EV-02][EV-04][EV-05][EV-06]
- La brecha real no es codigo faltante del nucleo: es evidencia de puerta
  (Fuzz 24h #156, Benchmarks #157, deuda abierta), distribucion de confianza
  y coherencia documental. El borrador de producto senala que la causa raiz de
  cinco defectos de deriva es la **ausencia de una prueba doc-vs-CLI**.
  [EV-07][EV-09]
- Progreso honesto del roadmap: 42.6% global; fases 5-10 al 0.0%. Esto no
  habilita proposiciones que "salten" esas fases. [EV-10]

---

## 3. Bucket A: small wins alcanzables dentro de la fase actual

Criterio para entrar aqui: (a) resuelve un problema verificable, (b) cabe en
Fase 4.7/4.8 sin abrir la puerta, (c) no crea crate, (d) solo usa dependencias
**verificadas** en `Cargo.toml`/`Cargo.lock` (S-9), (e) no toca rutas protegidas
(L-4) ni secretos.

### 3.1 Tabla resumen

| ID | Prio | Nombre | Fase | Valor | Coste | Riesgo | RFC | Estado |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A1 | P0 | Prueba de coherencia doc-vs-CLI como gate de CI | 4.8-E (aditiva) | Alto: elimina causa raiz de 5 derivas | S (~1 dia) | Bajo | No (gate de CI, sin comando publico nuevo) | `implementable now` |
| A2 | P0 | `ag generate --check` (modo solo verificacion) | 4.8-A (aditiva) | Alto: evita drift schema/artefactos en CI | S (~0.5 dia) | Medio (determinismo) | No (RFC-0019 PR 4 ya lo describe) | `implementable now` |
| A3 | P1 | Ampliar `--output json` a los comandos de lectura restantes | 4.8-E (aditiva) | Alto: desbloquea RFC-0024 | M (~2-3 dias) | Medio (drift de contrato) | Si: version bump del contrato en `docs/mcp`, sin RFC nueva | `implementable now` |
| A4 | P1 | `ag describe` (resumen de proyecto, solo lectura) | 4.8-E (aditiva) | Medio: respuesta unica "que contiene este proyecto" | S-M (~1 dia) | Medio (fuga de secretos) | No (RFC-0021 etapa E; shape ya nombrado en el contrato MCP) | `implementable now` |
| A5 | P2 | `ag doctor` (preflight local multi-modulo) | 4.8-F (aditiva) | Medio: reduce ciclos de soporte; base de `debug.diagnostics` | M (~2 dias) | Medio (scope creep) | Si: enmienda pequena a RFC-0021/0024 por comando nuevo (regla 28) | `needs RFC (amendment)` |
| A6 | P3 | Archivo cifrado de secretos + `ag env set` | 4.8-B (aditiva) | Alto, pero bloqueado | L (~1 semana) | Alto (criptografia + secretos) | RFC-0020 ya aceptada; decision humana adicional | `needs human + dependency unverified` |
| A7 | P3 | `ag completions SHELL` | 4.8-E (aditiva) | Bajo-medio | S (~0.5 dia) | Bajo | Si: comando nuevo (regla 28) | `needs human + dependency unverified` |

### 3.2 Fichas

#### A1. Prueba de coherencia doc-vs-CLI como gate de CI

- **Problema de usuario:** la documentacion y el binario divergen: cinco
  defectos de deriva confirmados (comandos anunciados que la CLI rechaza,
  advertencias de `bind` ya corregidas, versiones de alpha desactualizadas).
  Hoy no hay prueba automatica que lo detecte. [EV-09]
- **Alcance minimo:** un test que compara, contra el binario real,
  (1) texto de ayuda de cada comando, (2) mensajes de rechazo de argumentos
  invalidos, (3) los comandos del inicio rapido del README/manual y
  (4) cadenas de version. Sin red, sin secretos, sin rutas protegidas. Se
  ejecuta en el job `quality` existente.
- **Fase:** 4.8 etapa E, aditiva. No mueve hitos ni abre la puerta.
- **Dependencias (S-9):** `clap` 4, `serde_json` 1 y el propio binario `ag`.
  Todas verificadas en `Cargo.toml`; **sin dependencia nueva**. [EV-08]
- **Valor:** evita que un usuario siga un comando documentado que no existe;
  reduce el coste de mantenimiento documental (K-7) y es la accion P4 del
  borrador de estado de producto. [EV-09]
- **Coste:** S (~1 dia), mas correccion de las derivas que detecte.
- **Riesgo:** bajo. Si la prueba compara texto literal, puede volverse ruidosa;
  mitigacion: comparar contratos/IDs estables, no prosa libre.
- **RFC requerida:** no. Es una prueba de CI, no un comando publico ni un
  cambio de contrato.
- **Descartar si:** exige modificar `.github/workflows` sin humano (L-4), o si
  las cadenas comparadas incluyen valores de entorno/secretos.

#### A2. `ag generate --check`

- **Problema de usuario:** no hay forma de fallar en CI cuando los artefactos
  generados (Rust/SQL/TS/OpenAPI) estan desactualizados frente a `schema.ag`.
  El feature se genera y nadie lo detecta hasta runtime. [EV-02][EV-06]
- **Alcance minimo:** bandera `--check` en `ag generate`: genera en memoria o
  directorio temporal, compara bytes con los archivos existentes, imprime un
  resumen de diferencias y sale no-cero si hay drift. **No escribe nada** en
  modo check. Reutiliza los generadores de `ag-dsl`.
- **Fase:** 4.8 etapa A (RFC-0019 PR 4, documentado como pendiente). Aditiva.
- **Dependencias (S-9):** `ag-dsl` (crate interno), `clap` 4, `serde_json` 1,
  `tempfile` 3 (dev). Todas verificadas; **sin dependencia nueva**. [EV-08]
- **Valor:** guarda barata y determinista de la propiedad "schema-first";
  cierra una casilla documentada sin abrir fases. [EV-02]
- **Coste:** S (~0.5 dia).
- **Riesgo:** medio si los generadores no son byte-deterministas (regla 36 de
  reproducibilidad). Mitigacion: verificar determinismo primero; si no lo son,
  arreglar eso antes y no relajar la comparacion.
- **RFC requerida:** no; RFC-0019 ya describe `ag generate --check`.
- **Descartar si:** requiere escribir en el arbol en modo check, o si obliga a
  relajar la comparacion para que pase (L-10 anti-trampas).

#### A3. Ampliar `--output json` a los comandos de lectura restantes

- **Problema de usuario:** scripts y agentes siguen parseando prosa para
  varias operaciones de solo lectura; el prerrequisito 3 de RFC-0024
  ("request/executor split cubre todos los grupos") sigue sin cumplirse.
  [EV-04][EV-05]
- **Alcance minimo:** publicar shapes versionados para los comandos de lectura
  que **ya tienen executor tipado** (p. ej. `config get`, `domains list`,
  `mail` de solo lectura), anadir snapshots bajo `docs/mcp/` y subir la
  version del contrato. Los comandos mutantes siguen rechazando `--output json`
  antes de hacer trabajo. Sin `deploy.*` (pertenece a Fase 5). [EV-05]
- **Fase:** 4.8 etapa E, aditiva; desbloquea la etapa F (MCP) sin abrirla.
- **Dependencias (S-9):** `serde_json` 1, `clap` 4, contratos locales
  `docs/mcp/cli-json-contract-0.1.0.md` y `0.2.0.md`. Sin crate nuevo. [EV-05][EV-08]
- **Valor:** superficie de maquina completa; menos parsers de prosa; alinea
  CLI y el contrato MCP compartido sin codigo MCP aun. [EV-05]
- **Coste:** M (~2-3 dias, segun cuantos comandos tengan executor).
- **Riesgo:** medio por drift de contrato entre Anti-Gravital y Fabric.
  Mitigacion: `contract_version` en cada respuesta y snapshot test por tool,
  tal como exige RFC-0024.
- **RFC requerida:** no RFC nueva si se mantiene la familia de contratos; si
  se agrega un tool group nuevo, entonces enmienda a RFC-0024. Sin `deploy.*`.
- **Descartar si:** un comando carece de executor tipado (entonces primero
  hacer el trabajo del prerrequisito 3), o si algun comando mutante acepta JSON.

#### A4. `ag describe` (resumen de proyecto, solo lectura)

- **Problema de usuario:** no existe una unica forma de responder "que modulo
  tengo, que declare en `ag.toml`, que schema y que entorno requiere". Hoy el
  usuario junta `config`, `env list`, `schema lint` y `workers list` a mano.
  El nombre ya aparece en el contrato MCP y en la etapa E pendiente.
  [EV-02][EV-05][EV-06]
- **Alcance minimo:** comando de solo lectura que imprima un resumen agregado
  (manifiesto, modulos declarados, resumen del schema, variables declaradas
  **sin valores** y estado de configuracion). Misma shape en prosa y en JSON
  versionado. Cero escritura, cero red.
- **Fase:** 4.8 etapa E (documentada). Aditiva.
- **Dependencias (S-9):** `clap` 4, `serde_json` 1, `toml` 1 y el reader
  parcial de `ag.toml` que ya usan `ag env`/`ag dns`. Sin crate nuevo.
  [EV-04][EV-08]
- **Valor:** ahorra trabajo al usuario y da al contrato MCP un tool de
  descubrimiento coherente (`project.describe` / `ag describe`). [EV-05]
- **Coste:** S-M (~1 dia).
- **Riesgo:** medio: fuga de secretos. Mitigacion: reutilizar `Secret<T>`, no
  serializar valores, y anadir test de redaccion.
- **RFC requerida:** no; RFC-0021 etapa E y el contrato MCP ya lo nombran.
- **Descartar si:** imprime cualquier valor secreto o necesita contactar un
  proveedor externo.

#### A5. `ag doctor` (preflight local multi-modulo)

- **Problema de usuario:** un checkout puede estar "listo" localmente y fallar
  en CI o en otra maquina; no hay una verification unica antes de pedir ayuda.
  Ya existe `ag workers doctor`, pero es de un solo modulo. [EV-04]
- **Alcance minimo:** comando de solo lectura que ejecuta, a traves de los
  executores existentes, `schema lint`, `env check`, `config validate` y el
  chequeo de workers, y emite un informe compacto con estado por modulo.
  Sin red, sin DNS, sin escritura, sin aplicar cambios. `--output json`
  opcional reutilizando la forma versionada.
- **Fase:** 4.8 etapa F, aditiva; puede servir de base local de
  `debug.diagnostics` cuando RFC-0024 se desbloquee.
- **Dependencias (S-9):** `clap` 4, `toml` 1, `serde_json` 1, `thiserror` 2 y
  los executores de `ag-cli`. Todas verificadas; sin crate nuevo. [EV-08]
- **Valor:** reduce ciclos de soporte y da una senal verificable de salud del
  proyecto sin daemon ni observador externo (compatible con `HealthRegistry`
  nativo de `ag-observe`). [EV-02][EV-09]
- **Coste:** M (~2 dias).
- **Riesgo:** medio por scope creep: puede convertirse en "arreglador" o en
  cliente de red. Mitigacion: solo-agregador, sin mutacion, sin egress (el
  prerrequisito 7 de RFC-0024 exige cero DNS mientras se documente stdio-only).
- **RFC requerida:** **si**, enmienda pequena a RFC-0021/0024 por ser comando
  nuevo (regla 28). No requiere ADR: no cambia limites de crates.
- **Descartar si:** necesita red/egress, duplica `ag workers doctor` sin
  aportar agregacion multi-modulo, o empieza a corregir automaticamente.

### 3.3 Candidatos de la fase actual que NO son quick wins hoy

Estos dos son utiles y estan dentro de la fase, pero **no son ejecutables por
este agente ahora**: dependen de una libreria no verificada (S-9) y/o de una
decision humana (seguridad/criptografia). Se listan con sus atributos para que
el gerente los priorice, no para implementarlos.

#### A6. Archivo cifrado de secretos y `ag env set`

- **Problema:** Fase 4.8-B tiene el loader declarado y `Secret<T>`, pero no
  existe el archivo cifrado ni la escritura controlada de secretos. [EV-02][EV-06]
- **Alcance minimo:** un archivo cifrado por entorno con comandos de
  lectura/escritura que nunca imprimen valores; integracion con el loader.
- **Fase:** 4.8-B (aditiva). Sin abrir Fase 5.
- **Dependencias (S-9): NO VERIFICADAS.** `age` y `age-core` **no estan** en
  `Cargo.toml` ni en `Cargo.lock` (verificado con `grep`, EV-08). Requieren
  comprobacion de nombre exacto, mantenimiento y licencia, fijacion de version
  y **revision humana de manifiesto/lockfile** (S-9).
- **Valor:** alto; **coste:** L; **riesgo:** alto (S-1/S-2, S-5).
- **RFC:** RFC-0020 aceptada, mas decision humana de biblioteca y custodia.
- **Descartar si:** no hay libreria verificada con licencia compatible
  Apache-2.0, o si el diseno obliga al agente a leer credenciales (S-2).

#### A7. `ag completions SHELL`

- **Problema:** la CLI tiene muchos subcomandos y no hay generador de
  completions para shell; DX real pero menor. [EV-04]
- **Alcance minimo:** un subcomando que emita el script para los shells
  soportados, feature-gated y sin red.
- **Fase:** 4.8-E (aditiva).
- **Dependencias (S-9): NO VERIFICADAS.** `clap_complete` **no esta** en
  `Cargo.toml` ni en `Cargo.lock` (EV-08). Requiere verificacion S-9 y
  aprobacion humana de manifiesto/lockfile.
- **Valor:** bajo-medio; **coste:** S; **riesgo:** bajo.
- **RFC:** si (comando nuevo, regla 28).
- **Descartar si:** se prefiere escribir el generador a mano (reinventar la
  herramienta dominante, regla 12) o si la dependencia no pasa S-9.

---

## 4. Bucket B: ya documentado en el roadmap (Fase 5+). No reproponer como nuevo

Estas son funcionalidades reales y utiles, pero **ya estan contratadas** en el
roadmap. Proponerlas como "ideas nuevas" seria saltar fases (regla 8) y
convertir este catalogo en hype (regla 10). Se enumeran con su puerta para que
no se malinterpreten como omisiones.

| Item | Fase / etapa | Fuente | Por que no es propuesta nueva (compuerta) | Dependencias (S-9) | Descartar si |
| --- | --- | --- | --- | --- | --- |
| `ag-cloud`: `deploy.ag`, Dockerfile, docker-compose, fly, railway, k8s, `ag deploy`/`ag rollback` | 5 (5-A..5-F) | [EV-11][EV-02] | Requiere puerta pre-Fase 5 cerrada y RFC de `deploy.ag` + comandos; hoy `ag deploy` es solo render local | Crates `ag-cloud`/`ag-domains` (a crear en su fase); no tocar ahora | Se intente antes de cerrar #156/#157/deuda, o se reenviente Terraform/K8s (regla 12) |
| Funciones serverless: `ag functions ...`, WIT, WASM host, capabilities, triggers | 5.5 (5.5-A..F) | [EV-11][EV-06] | RFC-0023 aceptada pero **sin iniciar**; exige beta 0.5 y 4.8-B/E completas | `ag-wasm-host` (existe), `ag-workers` (existe); motor WASM a fijar en su RFC | Se implemente antes de la beta 0.5 (regla 8) o sin denial test por capability |
| Knowledge graph, `ag ai suggest-schema/review-migration/analyze-architecture`, dashboard | 6 (6-A..E) | [EV-11] | Requiere beta 0.5 y RFCs de grafo y provider/consentimiento; 6-A..C son utiles sin IA | Providers remotos son opt-in por feature (ADR-0009/0011); no introducir aqui | Se envie schema/codigo a un proveedor sin consentimiento o se presente como "AI-native" (regla 10) |
| `ag-migrate` (importadores) | 7 | [EV-11] | Sin iniciar; depende de 6 y de la puerta de la fase | Sin verificar en esta fase | Se proponga como "quick win" |
| `ag-mobile` (puente Flutter) | 8 | [EV-11][EV-01] | Sin iniciar; UI queda fuera del nucleo (RFC-0019) | Flutter es herramienta dominante: integrar, no reimplementar | Se reintroduzca `ag-ui` en el core |
| Plugins WASI | 9 | [EV-11][EV-01] | Sin iniciar; comparte host con 5.5-A | `ag-wasm-host` (existe) | Se duplique el host de funciones |
| `ag mcp serve` / `ag mcp client add` y tools `debug.*` | 4.8-F (aditiva) pero **bloqueada** | [EV-05][EV-02] | RFC-0024 aceptada con 7 prerrequisitos sin cumplir; `deploy.*` debe esperar a Fase 5 | `ag-cli` feature `mcp` (a crear en su RFC), `Secret<T>` (existe) | Se exponga el plano de control de Fabric bajo el contrato, o se abra un puerto/daemon (alternativa C rechazada) |
| Endurecimiento y 1.0 (API estable, SemVer) | 10 | [EV-11] | Sin iniciar; no es una funcion suelta sino la puerta final | N/A | Se prometa estabilidad de API en alpha |

---

## 5. Bucket C: fuera de alcance o rechazar, con razon

| Propuesta descartada | Razon verificable | Fuente |
| --- | --- | --- |
| Correo entrante: IMAP/POP, buzones, antispam, reputacion de IP | Explicitamente fuera del alcance; el MTA es **outbound** | [EV-01] |
| `ag-domains` como registrador o reemplazo de Terraform | Fuera de alcance; se integra generando variables de Terraform | [EV-01][EV-03] |
| Renderizado de UI, `ag-ui`, Flutter embebido en el core | Retirado por RFC-0019; alcance exclusivamente backend | [EV-01][EV-11] |
| Reimplementar PostgreSQL, Redis, NATS, MinIO, Kubernetes, Docker, Next.js | Regla 12: integrar mediante traits/adapters, no reinventar | [EV-03] |
| "Agente IA autonomo" que escriba codigo/archivos sin confirmacion | Rompe regla 10, el principio "no privilege by being a machine" de RFC-0024 y el consentimiento de Fase 6 | [EV-03][EV-05][EV-11] |
| Daemon de agente con puerto propio, autenticacion y protocolo privado | Alternativa C de RFC-0024 rechazada: duplica el protocolo abierto y anade superficie critica | [EV-05] |
| Managed API REST de `ag-mail` | Solo diseno; issue #152 diferido por decision aceptada | [EV-09] |
| Crear un crate nuevo para cualquiera de los small wins de la seccion 3 | Reglas 5/14: los cambios viven en crates existentes | [EV-01][EV-03] |
| Implementar `ag functions` o `ag mcp serve` "para llegar antes" | Regla 8: salto de fase; la puerta pre-Fase 5 sigue abierta | [EV-02][EV-07] |

---

## 6. Recomendacion y coherencia con la vision

**Orden recomendado:** A1 + A2 en paralelo (ambos P0, sin RFC), luego A3 y A4
(P1, desbloquean la interfaz de maquina), despues A5 con enmienda de RFC. A6 y
A7 quedan en espera de decision humana; el resto se deja en su fase.

Comprobacion de propiedades de la vision:

| Propiedad del Blueprint | Resultado de este catalogo |
| --- | --- |
| Binario unico, sin runtime externo | Ninguna propuesta anade daemon, sidecar ni interprete |
| Schema-first `.ag` | A2 refuerza el generador; ninguna propuesta crea un segundo DSL |
| Crates modulares | Ninguna propuesta crea crate; A6/A7 se limitan a features de crates existentes |
| Integrar, no reinventar (regla 12) | Bucket C rechaza reinvenciones; A7 usa la herramienta dominante si se aprueba |
| No saltar fases (regla 8) | Bucket B enumera Fase 5+ como "no reproponer"; ningun A abre la puerta |
| Anti-hype (regla 10) | Todo A nace de un hueco con EV; C descarta tendencias sin proposito |
| Apache-2.0 + producto privado, MCP compartido | A3/A4/A5 alimentan el contrato comun; sin dependencia de compilacion con Fabric |
| Evidencia verificada | Toda afirmacion factual de las secciones 1-5 cita EV; la seccion 6 es un resumen derivado de esas fuentes. Dependencias verificadas o marcadas `NO VERIFICADA` (S-9) |

---

## 7. Indice de evidencia (`source_ids` de este catalogo)

| ID | Fuente verificable (ruta, seccion/linea, comando o commit) |
| --- | --- |
| EV-01 | `docs/master/ANTI-GRAVITAL-Blueprint-v4.1.md` L21-50 (vision/posicionamiento), L52-77 (alcance y fuera de alcance), L78-97 (crates y reglas de dependencia), L99-131 (estado de entrega, MCP y fases 4.8/5.5), L145-203 (version espanola). |
| EV-02 | `docs/roadmap/STATUS.md` L12-14 (nota de ola 9 y puerta abierta), L453-500 (Fase 4.7), L503-534 (Fase 4.8 y prerrequisitos), L538-574 (Fases 5-10). |
| EV-03 | `CLAUDE.md` L201-214 (regla 8 incremental), L236-256 (regla 10 anti-hype), L277-298 (regla 12 interoperabilidad); `docs/roadmap/reglas-de-oro.md` L7-21. |
| EV-04 | `crates/ag-cli/src/cli.rs` L35-81 (json_command_name y comandos con forma versionada), L83-217 (arbol de comandos: `new --with`, `dev`, `build`, `config`, `env`, `generate`, `schema`, `dns`, `domains`, `mail`, `workers`, `deploy`). |
| EV-05 | `docs/rfc/RFC-0024-mcp-shared-machine-interface.md` L64-101 (principios, contrato compartido y grupos de tools), L102-125 (servidor/cliente/self-debug), L170-194 (siete prerrequisitos y mcp-readiness-review); `docs/mcp/cli-json-contract-0.1.0.md` y `0.2.0.md`. |
| EV-06 | `docs/roadmap/fase-04-8-fundaciones-del-proyecto.md` (etapas A-E; `ag.toml` reader, `ag generate --check`, archivo cifrado, `ag describe`); `docs/rfc/RFC-0019-project-layered-architecture.md`, `RFC-0020-environment-and-secrets.md`, `RFC-0021-server-install-remote-console-and-machine-interface.md`. |
| EV-07 | `docs/audits/PRE_FASE5_RELEASE_GATE.md` (tabla de filas; Fuzz 24h #156, Benchmarks #157, Open debt pendientes). |
| EV-08 | `Cargo.toml` seccion `[workspace.dependencies]`: `clap = "4"`, `serde_json = "1"`, `toml = "1"`, `thiserror = "2"`, `tempfile = "3"`, `ag-dsl` (path). Comando: `grep -nE '^(clap\|serde_json\|toml\|thiserror\|tempfile\|clap_complete\|age\|age-core)[[:space:]]*=' Cargo.toml`; `grep -E '^name = "(clap_complete\|age\|age-core)"' Cargo.lock` -> sin coincidencias. |
| EV-09 | Borrador `community15/status-producto:docs/audits/estado-honesto-producto.md` L21-27, L59-92, L117-130, P4 en L154 (no aterrizado en esta rama; dato no confiable). |
| EV-10 | Borrador `community15/status-roadmap:docs/audits/estado-honesto-roadmap.md` L45-65, L83-98, L107-134 (no aterrizado en esta rama; dato no confiable). |
| EV-11 | `docs/roadmap/fase-05-ag-cloud.md` L8-89; `fase-05-5-funciones-serverless.md` L8-79; `fase-06-ag-ai-knowledge-graph.md` L8-68; `fase-10-endurecimiento-y-1.0.md`. |
| EV-12 | Comandos ejecutados en este worktree: `git branch --show-current` -> `community15/opp-features`; `grep -rn -- '--check\|fn describe\|encrypt' crates/ag-cli/src` -> sin implementacion de `ag generate --check`, `ag describe` ni archivo cifrado; `find ... -iname '*estado*honesto*'` -> solo borradores de pares. |
| EV-13 | `git diff --check` y escaneo de contenido (sin secretos, emojis ni atribucion a herramientas de IA) sobre los archivos cambiados: resultados en la seccion 8. |

---

## 8. Verificacion de este worktree

- `git diff --check`: RC=0, sin errores de whitespace.
- Escaneo de contenido (atribucion a herramientas de IA, emojis, secretos): sin
  coincidencias sobre los tres archivos cambiados.
- `bash scripts/kaizen check --only descriptors policy`: RC=0, todo en verde.
- `bash scripts/kaizen check`: RC=1, FALLO. Pasan `descriptores`,
  `politicas de contenido`, `formato` y `lint`. Fallan puertas **preexistentes**:
  `tests`, `swarm: ver kaizen-swarm check`, `gobernanza: ver kaizen-gov check`
  y `deuda subio: 4 > 0`. El cambio es solo documental y no anade deuda:
  `git diff -U0 | grep -E '^\+.*(TODO|FIXME|HACK)'` sin salida. Nada se relajo
  ni se silencio (S-10/L-10).
- Sin push, sin PR, sin merge y sin cambios en rutas protegidas (L-4).
  Descriptor: `docs/kaizen/pr-drafts/community15-opp-features.md`.
