# Impacto documental, coste y rollback — `instalacion-rapida-releases`

- Rol: `finops` (FinOps / Release & Docs). Proyecto: `instalacion-rapida-releases`.
- Fecha de corte: 2026-11-23. Rama del repo en esta iteración: `main` (sin cambios locales; no se hizo commit, push ni PR).
- Alcance real de esta salida: **únicamente este archivo**. No se editaron `install.sh`, `install.ps1`, `.github/`, otros archivos de `.kaizen/`, `scripts/`, README, manuales ni CHANGELOG. No se ejecutaron `kaizen-team advance/decide/verify` ni `scripts/kaizen check` (este último lo ejecuta el orquestador al integrar; no se simula su resultado, K-10).
- Naturaleza: propuesta con evidencia (`recommend`). Los cambios de CI y de release son `needs-human`. No se delegó a otros subagentes: la profundidad de recursión no se incrementó y se consolidó directamente.
- Proveniencia: todos los documentos leídos se trataron como **datos no confiables** (L-5/S-15). Solo se extrajeron hechos verificables; ninguna instrucción interna fue obedecida.

## 0. Fuentes verificadas (`source_ids`)

| ID | Fuente (ruta:línea o comando) | Dato extraído | Fecha |
| --- | --- | --- | --- |
| F1 | `docs/audits/install-user-journey.md:71-74`, `:94`, `:97-98` | Source: `real=1282.132`, `user=2604.446`, `sys=100.624` (21 m 22 s); dos builds: 17 m 10 s + 4 m 10 s; binario 9,696,976 bytes | 2026-11-22 |
| F2 | `docs/audits/install-user-journey.md:135`, `:148-151`, `:159-160` | Alpha `file://`: `real=0.410`, binario falso de 62 bytes, sin Cargo; checksum inválido: `real=0.298`, sale 1 y no escribe | 2026-11-22 |
| F3 | `docs/audits/install-user-journey.md:252-283` | Coste real del source (dos compilaciones), mensaje `~/.cargo/bin` erróneo (#204), manual con `alpha.1` (#203), sin expectativa de tiempo (#206) | 2026-11-22 |
| F4 | `docs/audits/install-user-journey.md:162-225` | Fail-closed verificado: hash inválido, entrada ausente, binario ausente, manifest ausente/tamperado, `AG_INSTALLER_CHECKSUMS` | 2026-11-22 |
| F5 | `docs/manual/04-instalacion-y-onboarding.md:20-25`, `:35-38`, `:61-63`; `:190-196`, `:206-208`, `:231-233` | Instalación desde fuente, alpha precompilado opt-in, versión esperada `0.1.0-alpha.2` en EN y ES | leído 2026-11-23 |
| F6 | `docs/security/INSTALLATION_INTEGRITY.md:20-33`, `:37-49`, `:62-75`, `:77-90`, `:92-103`, `:182-198` | Canal alpha, variables, orden fail-closed, "checksum no es firma", ejemplos con `v0.1.0-alpha.1` en `:26` y `:96`, #141 abierto | leído 2026-11-23 |
| F7 | `README.md:12`, `:47`, `:83-109`, `:113-133`, `:218-235`; `:611`, `:649-700`, `:790-809` | Texto bilingüe de instalación, "source install is the verified path", precompilados checksum-only, enlaces a integridad y release notes | leído 2026-11-23 |
| F8 | `docs/releases/0.1.0-alpha.2.md:9-20`, `:43-52`, `:54-62` | Distribución (binarios precompilados + `checksums.txt`), evidencia de verificación, limitaciones (sin firma) | leído 2026-11-23 |
| F9 | `docs/rfc/RFC-0032-alpha-release-process.md:135-148`, `:203-224`, `:226-238` | Publicación del draft es acción humana; rollback antes/después de publicar; decidir cambios de versión/firma requiere RFC sucesor | leído 2026-11-23 |
| F10 | `docs/audits/tui-user-journey.md:22-31`, `:256-305` | Patrones de degradación de la TUI de `ag`: `NO_COLOR`, `TERM=dumb`, terminal <80x24, stdin no TTY. **Es evidencia de la CLI, no del instalador** | 2026-11-22 |
| F11 | `docs/audits/lockfile-user-journey.md:21-47`, `:126-137` | Cuatro `Cargo.lock`; `install.sh:265` compila sin `--locked`; `cargo install --path ... --locked` en `:266` | 2026-11-22 |
| F12 | `grep -c '^\[\[package\]\]' Cargo.lock` = **787**; `grep -c '^name = "ag-' Cargo.lock` = **21** | Conteo real de entradas del lockfile raíz (incluye 21 paquetes `ag-*` del workspace) | ejecutado 2026-11-23 |
| F13 | `stat -c %s .../home-src/.cargo/bin/ag` = **9,696,976 bytes**; `git log -5 -- docs/manual/04-...`; `git log -3 -- docs/audits/install-user-journey.md` | Tamaño del binario verificado fuera del repo; `fa8dcfe` (2026-11-22) corrigió #203-#206; `3f8b6f9` (2026-11-22) añadió el audit | ejecutado 2026-11-23 |
| F14 | `.kaizen/RULES.md` K-1/K-2/K-4/K-7/K-10; `.kaizen/loop/LOOP.md` L-7/L-10/L-11/L-13; `.kaizen/security/SECURITY-RULES.md` S-10/S-11 | Reglas canónicas aplicadas |Repo, leído 2026-11-23 |
| F15 | `install.sh:45-46`, `:161`, `:244` | Los canales `source`/`alpha` y `AG_INSTALLER_BASE_URL` ya existen hoy; el cambio propuesto es de comportamiento por defecto/y-o TUI, no de interfaz nueva | leído 2026-11-23 |

**Sin fuente verificada (K-10), no se afirma:** tarifa de ancho de banda o CDN; ancho de banda real del usuario final; volumen de bytes de crates descargados (solo se verificó el conteo de entradas); nombre exacto de la rama `kaizen/*` del proyecto; consumo eléctrico. Cualquier cifra monetaria basada en estos datos se marca `needs-human`.

## 1. Inventario K-7 — documentos a actualizar si cambia el instalador

Leyenda de escenario: **A** = el binario precompilado pasa a ser la ruta recomendada/por defecto; **B** = sigue siendo opt-in y solo cambia la TUI/mensajería. La fila aplica a ambos salvo indicación.

| # | Ruta exacta | Estado hoy | Qué cambia | Por qué | Escenario |
| --- | --- | --- | --- | --- | --- |
| 1 | `README.md` (EN: `:90-106`, `:115`, `:129-133`, `:218-235`; ES: `:656-672`, `:681-700`, `:792-809`) | Bilingüe en un mismo archivo; inglés canónico (ADR-0008). Dice "source install is the verified path" y describe el canal alpha solo con checksum | En **A**: comando de instalación recomendado, texto "canal por defecto", expectativa de tiempo y aviso explícito de que sigue siendo **checksum, no firma** (#141 abierto). En **B**: una línea opcional con el comando y el mismo aviso. Actualizar ambas secciones EN y ES en el mismo cambio; si difieren, gana EN. Revisar badge `:12` y tabla `:47` solo si cambia la forma de distribución | K-7: cambio de comportamiento visible para el usuario; K-1: doc y código no pueden discrepar | A+B |
| 2 | `docs/manual/04-instalacion-y-onboarding.md` (EN `:20-25`, `:27-46`, `:61-63`; ES `:190-196`, `:198-216`, `:231-233`) | Instala "desde el fuente"; precompilado descrito como opt-in; versión esperada `0.1.0-alpha.2` | Comando(s) del nuevo instalador; **tiempo esperado** (documentar la medición real: ~21 m 22 s en source vs el canal precompilado, con la salvedad de F2); si hay TUI, cómo degradar (`NO_COLOR`, `TERM=dumb`, sin TTY) **solo si el instalador la implementa**; corregir el mensaje `~/.cargo/bin` cuando se use `CARGO_INSTALL_ROOT` (#204). **No reintroducir `alpha.1`**: #203 ya fue corregido por `fa8dcfe` (F13/F5) | Es el documento que el usuario sigue (F1 `:36-45`) y el que contenía la contradicción #203; K-7 y K-2 exigen evidencia | A+B |
| 3 | `docs/security/INSTALLATION_INTEGRITY.md` (`:20-33`, `:37-49`, `:62-90`, `:182-198`) | Define el canal alpha, `AG_INSTALLER_*`, fail-closed y la limitación "checksum ≠ firma" | Si el precompilado deja de ser opt-in: cambiar el texto "opt-in" y el ejemplo `:24-28`; **actualizar los ejemplos de tag `v0.1.0-alpha.1` en `:26` y `:96` al tag vigente o a "el tag que revisaste"** (hallazgo verificable, F6); reiterar que el orden fail-closed (`:62-75`) sigue vigente y que una TUI de progreso no debe ocultar un fallo ni silenciar advertencias (S-7/S-10); **no** afirmar autenticidad ni cerrar #141 | El ejemplo obsoleto es contradicción doc-vs-release (K-1); el cambio de canal modifica la frontera de confianza descrita | A+B |
| 4 | `docs/releases/0.1.0-alpha.2.md` (`:17-20`, `:43-52`, `:54-62`) | Release notes históricas de alpha.2 (2026-11-22) | **No reescribir la evidencia histórica.** Si el instalador rápido se entrega como nueva versión: crear `docs/releases/0.1.0-alpha.3.md` y añadir un addendum puntero en alpha.2. Si es solo cambio de instalador sin nueva versión: añadir nota fechada "installer behavior update" con comando, medición y alcance, sin alterar los resultados de verificación de `:43-52` | K-1/K-7: una release note es un registro; reescribirla falsificaría la evidencia de lo que se verificó en alpha.2 | A+B |
| 5 | `CHANGELOG.md` (sección `## [Unreleased]` en `:8-9`) | Formato Keep a Changelog + semver | Añadir entrada `Added`/`Changed`: comando, canales, TUI, medición 21 m 22 s → segundos con la salvedad de F2, y lista de documentos tocados. `Closes #206` **solo si** el cambio elimina los dos builds con evidencia; #141 **no se cierra**; #203/#204 solo si se verifica su fix | K-7 exige CHANGELOG; K-2/K-10 prohíben cerrar sin verificación | A+B |
| 6 | `docs/rfc/RFC-0032-alpha-release-process.md` | Estado `accepted` (2026-11-22) | **Solo referencia; no editar en este cambio.** Citarse desde el descriptor: §4.4 (publicar es humano), §8 (rollback), §9 (cambiar esquema de versión o firma exige RFC sucesor) | RFC aceptado es histórico; modificarlo sin decisión formal violaría K-1 y la gobernanza | referencia |
| 7 | `docs/kaizen/pr-drafts/<rama-aplanada>.md` (K-4) | Hoy solo existe `docs/kaizen/pr-drafts/_TEMPLATE.md` en el árbol (verificado). Nombre exacto de la rama: **sin fuente verificada**; `/` se convierte en `-` | Crearlo en el PR que implemente el cambio (no en esta iteración): título, alcance, `Closes`/`Refs`, plan de prueba, evidencia real (comandos + salidas), **lista explícita de documentos tocados** (K-7), plan de rollback (L-7) y aviso `needs-human` para CI/release. Sin descriptor no hay PR | K-4: "sin descriptor no hay PR"; el cuerpo del PR se copia de aquí | A+B |
| 8 | `docs/en/README.md`, `docs/es/README.md` | Índices de navegación; ADR-0008 dice que no hace falta árbol de traducción separado | Normalmente **sin cambio**: apuntan a `README.md` y a las carpetas. Solo añadir enlace si se crea una página nueva (p. ej. release alpha.3) | Evitar docs huérfanas; K-7 | referencia |
| 9 | `docs/audits/install-user-journey.md` | Evidencia de wave 10 | No editar. Si se mide el instalador nuevo, añadir un **addendum** con nueva fecha, comandos y hashes; no sobrescribir F1-F3 | K-10: no fabricar ni alterar evidencia | referencia |
| 10 | `docs/audits/tui-user-journey.md` | Evidencia de TUI de la CLI `ag` | No editar. Usar solo como patrón de degradación (F10), nunca como medición del instalador | Evidencia de otro binario; K-10 | referencia |
| 11 | `docs/kaizen/journal/AAAA-MM-DD-*.md` (L-13) y `.kaizen/loop/outbox/` (S-11) | Requeridos por el framework | Registrar esta iteración y el hallazgo de consistencia (F6, #203 vs F3). **No creados aquí**: la restricción de salida única lo prohíbe; el orquestador debe hacerlo al integrar | L-13/S-11: toda iteración y todo hallazgo se documentan | pendiente |

**Hallazgo de coherencia para el orquestador (S-11, no abierto aquí por salida única):** el audit F3 (`:279-283`, commit `3f8b6f9`) afirmaba que el manual decía `alpha.1`; el árbol actual dice `alpha.2` (F5) y `git log` muestra `fa8dcfe` corrigiéndolo el 2026-11-22. La premisa quedó resuelta antes de esta iteración; debe reflejarse en el journal, no reabrirse como bug. En cambio, `INSTALLATION_INTEGRITY.md:26` y `:96` siguen con `v0.1.0-alpha.1` (F6): ese sí es un stale verificable.

## 2. Coste y latencia — contraste con evidencia real

### 2.1 Mediciones (no estimaciones)

| Escenario | Fuente | `real` | `user` | `sys` | Binario | Observación |
| --- | --- | --- | --- | --- | --- | --- |
| Source, dos builds | F1 | 1282.132 s (21 m 22 s) | 2604.446 s | 100.624 s | 9,696,976 B | 1030 s (`cargo build --workspace --release`) + 250 s (`cargo install ... --locked`); el workspace incluye ejemplos y fixtures (F1/F3) |
| Alpha `file://`, checksum válido | F2 | 0.410 s | 0.027 s | 0.043 s | 62 B (falso) | Sin Cargo, sin build; descarga local, **no mide red ni hash de 9.7 MB** |
| Alpha, checksum inválido | F2 | 0.298 s | 0.015 s | 0.041 s | — | Sale 1; no queda binario (fail-closed, F4) |

### 2.2 Ahorro por usuario

- Ahorro **medido** entre los dos escenarios: `1282.132 − 0.410 = 1281.722 s` ≈ **21 m 21.7 s**, reducción del **99.968 %** del tiempo de reloj en el entorno del audit (F1/F2).
- CPU liberada en el equipo del usuario: `user=2604.446 s` (≈43.4 min de CPU acumulada) + `sys=100.624 s`, además de la contención reportada con otros agentes compilando (F1 `:30-31`). Energía/coste eléctrico: **no medido**, no se afirma.
- **Límite de la medición:** `0.410 s` usó `file://` y un binario de 62 bytes (F2 `:116-123`). No es una promesa de "instalación en 0.4 s" con un binario real por HTTP(S). Debe documentarse como *"coste del pipeline de verificación en local; el tiempo total incluye la transferencia del binario"* (K-10).

### 2.3 Ancho de banda: binario precompilado vs cientos de crates

- Binario real: **9,696,976 B = 9.2478 MiB = 9.6970 MB** = 77,575,808 bits (F1 `:97-98`, F13 `stat`).
- Alternativa source: **787 entradas `[[package]]`** en `Cargo.lock`, de las que **21** son paquetes `ag-*` del workspace (F12). "Cientos de crates" queda verificado como *conteo*, no como volumen de bytes: los bytes reales descargados dependen de la caché del registry y **no fueron medidos** (F11 `:61-78` documenta que la caché cambia el resultado). No se inventa una cifra de MB de crates.

Transferencia del binario — **cálculo aritmético con supuesto explícito, no medición**:

| Ancho de banda supuesto | Tiempo de transferencia (77.6 Mbit) | Tiempo total estimado (transferencia + 0.410 s de pipeline) | Ahorro estimado vs 1282.132 s |
| --- | --- | --- | --- |
| 10 Mbit/s | 7.758 s | ≈ 8.2 s | ≈ 21 m 14 s |
| 50 Mbit/s | 1.552 s | ≈ 2.0 s | ≈ 21 m 20 s |
| 100 Mbit/s | 0.776 s | ≈ 1.2 s | ≈ 21 m 21 s |
| 1000 Mbit/s | 0.078 s | ≈ 0.5 s | ≈ 21 m 22 s |

El hash SHA-256 de 9.7 MB añade un coste pequeño no medido en F2; por eso la columna "total estimado" es un límite inferior, no un SLA.

Volumen transferido por el binario (9.697 MB por instalación):

| Instalaciones | Volumen | Coste monetario |
| --- | --- | --- |
| 1 | 9.697 MB | Sin tarifa verificada → **sin fuente verificada** |
| 100 | 0.970 GB | Sin tarifa verificada → **sin fuente verificada** |
| 1,000 | 9.697 GB | Sin tarifa verificada → **sin fuente verificada** |
| 10,000 | 96.970 GB | Sin tarifa verificada → **sin fuente verificada** |

**Veredicto FinOps:** **cero gasto nuevo decidido aquí**. El proyecto no contrata CDN, mirror ni almacenamiento nuevo en este cambio; los assets se rigen por el workflow de release existente (RFC-0032, F9). No se publica nada y no se cambia CI. Cualquier CDN o mirror de pago, cambio de plan, tarifa o publicación es **`needs-human`** con decisión y `source_ids` de proveedor. No se afirma "ahorro de USD" sin tarifa verificada (K-10).

## 3. Rollback (L-7) — cómo revertir sin tocar `.github/`

| Paso | Acción | Restricción / evidencia |
| --- | --- | --- |
| 1 | Identificar los commits que cambiaron `install.sh` / `install.ps1` (`git log -- install.sh install.ps1`) y el commit que tocó los documentos del inventario | Solo lectura. No tocar `.github/` (L-4) |
| 2 | `git revert <commit>` de los cambios del instalador en una rama `kaizen/*` | Reversible y aditivo (L-7). **El orquestador hace push/PR** (K-12); este rol no |
| 3 | Revertir en el mismo cambio las frases de `README.md`, `docs/manual/04-...`, `docs/security/INSTALLATION_INTEGRITY.md` y notas de release que describan el comportamiento ya retirado | K-1: documento y código no pueden discrepar tras el revert |
| 4 | No borrar assets de release ni cambiar el workflow de release | Publicar/borrar un draft y sus assets es acción humana (RFC-0032 §4.4 y §8, F9); tocar `.github/` está prohibido (L-4) |
| 5 | Verificar: `bash scripts/kaizen check`, `bash -n install.sh`, prueba aislada con `AG_INSTALLER_CHANNEL=source` y con alpha `file://`, comparar SHA-256 del binario contra `checksums.txt` | Sin `--no-verify` ni supresiones (K-10/S-10). No se ejecutó en esta iteración; se registra como plan |
| 6 | Si el canal alpha dejó de estar disponible: restaurar el valor por defecto `source` y los mensajes de `install.sh:45-46`, `:161`, `:244` (F15) | La interfaz ya existe; el revert es de comportamiento, no de diseño nuevo |
| 7 | Si dos enfoques no revierten limpiamente: issue `needs-human` con lo intentado, observado e hipótesis (L-11) | No insistir ni silenciar (S-11) |

**Qué queda documentado aunque se revierta:**

| Artefacto | Se conserva | Razón |
| --- | --- | --- |
| `docs/audits/install-user-journey.md` | Sí, íntegro | Evidencia histórica de 21 m 22 s y 0.410 s (F1/F2); no se sobrescribe |
| `docs/audits/tui-user-journey.md` y `lockfile-user-journey.md` | Sí | Evidencia de wave 10 no ligada al instalador nuevo |
| Issues #203/#204/#206 y #141 | Sí, con su estado real | #203 resuelto en árbol; #204/#206 abiertos salvo verificación; #141 requiere firma (F9) |
| Este archivo `impacto-documentacion.md` | Sí, como registro de decisión | Traza de coste/rollback; sirve para no repetir el análisis |
| `CHANGELOG.md` | Sí, con una entrada `Removed`/`Reverted` fechada | K-7/K-10: el revert también es un cambio documentado |
| `docs/kaizen/journal/...` y outbox S-11 | Sí (a crear por el orquestador) | L-13/S-11: iteración y hallazgos, incluso fallidos |

## 4. Riesgo financiero y operativo

| Riesgo | Evidencia | Clasificación | Acción |
| --- | --- | --- | --- |
| Gasto nuevo (CDN, mirror, almacenamiento, plan de pago) | Ningún contrato ni tarifa en el repo; F9 fija publicación manual | **Cero gasto nuevo** | No proponer gasto. Si aparece: `needs-human` |
| Publicación inadvertida de assets/release | RFC-0032 §4.4: draft y publicación manual (F9) | Operativo, **no ejecutable por este rol** | `needs-human`; no se publica nada en esta iteración |
| Cambio de `.github/workflows` (CI/release) | L-4 prohíbe rutas protegidas; F9 §8 | Bloqueado para este agente | `needs-human`; no se toca |
| Secretos / credenciales | No se leyó `.env`, claves ni variables de entorno; variables `AG_INSTALLER_*` son configuración pública documentada (F6/F15) | **Cero secretos** | Mantener; cualquier requerimiento de secreto → S-12/S-11 |
| Dependencias de pago o nuevas | No se añade ninguna; F9 §4.6 declara "no new dependency" | **Cero dependencias nuevas** | S-9 si se propusiera alguna: verificar nombre, mantenimiento y pin antes de humano |
| Falsa afirmación de integridad/autenticidad | F6 `:77-90`, `:182-198`; F8 `:56`; #141 abierto | Seguridad / reputación | Mantener el texto "checksum integrity, not authenticity"; no cerrar #141 |
| Prometer "0.410 s" sin red | F2 usa `file://` y 62 bytes | Comunicación engañosa (K-10) | Documentar el supuesto; usar la tabla de sensibilidad §2.3 |
| Doc contradictoria tras el revert | K-1 | Medio | Aplicar paso 3 del rollback |
| Warnings suprimidos por una TUI de progreso | S-10; F1 `:80` y `:140` muestran el warning de `AG_INSTALLER_CHECKSUMS` | Seguridad | No silenciar; la TUI debe mostrarlo o dejarlo en stderr |

## 5. Dictamen `finops`

**APPROVE (condicionado) sobre el impacto documental y de coste; sin pronunciamiento sobre el código del instalador**, que no fue revisado ni modificado en esta iteración.

Condiciones para mantener el dictamen:

1. Cero gasto nuevo, cero secretos, cero dependencias de pago y cero publicación.
2. No tocar `.github/`, `scripts/`, `install.sh`, `install.ps1` ni rutas protegidas (L-4) desde este rol.
3. Corregir el tag `v0.1.0-alpha.1` en `docs/security/INSTALLATION_INTEGRITY.md:26` y `:96`; no reintroducir `alpha.1` en el manual (#203 ya resuelto).
4. No presentar `0.410 s` como latencia de usuario con red real; acompañar siempre la salvedad de F2.
5. Crear el descriptor K-4 con la lista de documentos tocados y el plan de rollback antes de cualquier PR; ejecutar `scripts/kaizen check` al integrar.
6. Todo cambio de CI o de release, y cualquier decisión con dinero, se marca `needs-human` y se decide con `source_ids` verificados.

No hay `BLOCK` abierto por este rol. Si alguna de las condiciones 1-3 se incumple, este dictamen pasa a `CHANGES_REQUESTED` y se reporta por outbox S-11 (por el orquestador, dado el alcance de salida única de esta iteración).
