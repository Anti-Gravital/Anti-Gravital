# Descriptor de PR

## Resumen

Asistente interactivo de `ag new` (RFC-0018 etapa A), RFC-0018 a RFC-0023 aceptadas, Hoja de Ruta ampliada (Fases 4.6-4.8, 5.5 y etapas 5-10), Rust 1.99.1 y refresco de dependencias

## Fase afectada

Hoja de Ruta completa (documento maestro). Introduce las Fases 4.7 y 4.8
aditivas como propuestas. El unico cambio de codigo es el refresco de toolchain
y dependencias (sin funcionalidad nueva); no avanza ninguna puerta.

## Tipo de cambio

- [x] Correccion de bug (`Cargo.lock` desincronizado de los manifiestos)
- [x] Documentacion (maestro Hoja-de-Ruta EN+ES, derivados, README, RFC)
- [x] Gobernanza (limpieza de descriptores de PRs ya fusionadas)
- [x] Nueva feature (asistente interactivo de `ag new`, feature `tui`)
- [x] Mantenimiento (toolchain Rust 1.99.1 y dependencias)
- [x] Cambio que rompe compatibilidad (pre-release): MSRV 1.99.1; rutas de
  axum 0.8 con capturas `{nombre}`

## Contexto

Esta rama entrega tres cosas pedidas por el BDFL: una auditoria incremental del
repositorio desde la de junio (issue #142), la ampliacion y el detalle de la
Hoja de Ruta, y el diseno documentado de una interfaz interactiva de terminal
para crear y configurar proyectos. Siguiendo las reglas 5, 22 y 28, la interfaz
se entrega como RFC y como fase de la Hoja de Ruta; no se escribe codigo de la
TUI hasta que la RFC se decida.

### Etapa A de la TUI implementada (RFC-0018)

Las RFC-0018 a RFC-0023 fueron aceptadas por el BDFL el 2026-11-10; cada una
registra en su seccion 9 la resolucion de sus preguntas abiertas. Sobre esa
aceptacion esta rama implementa la etapa A de RFC-0018:

- `crates/ag-cli/src/main.rs` (1829 lineas) dividido en `cli.rs`,
  `commands/` (un modulo por grupo de comandos), sin cambio de comportamiento.
- `request.rs`: `NewProjectRequest` y `Template`; lo construyen tanto el
  parser de flags como el asistente, y lo ejecuta un unico `execute_new`.
- `interactive.rs`: decide si la sesion puede preguntar (terminal real, sin
  `CI`, sin `AG_NO_TUI`, `TERM` distinto de `dumb`, sin `--no-interactive`).
- `tui/` (feature `tui`, activa por defecto): asistente a pantalla completa
  de tres pasos (nombre con validacion en vivo, template, revision), tema
  monocromo con `NO_COLOR`, prompts por linea bajo 80x24, guarda que restaura
  la terminal al salir, ante error, `Ctrl-C` y panic, y todos los textos en
  `tui/text.rs`.
- `ag new`: `NAME` pasa a ser opcional; flag global `--no-interactive`; al
  terminar un flujo interactivo se imprime el comando equivalente.
- Dependencia nueva `ratatui` 0.30 (opcional); se retira `dialoguer`.
- Diagrama `docs/diagrams/cli-interactive-flow.md`; documentacion de la CLI,
  README (EN+ES), manual y knowledge graph actualizados.

Etapas B (`ag config`) y C (pantalla de inicio) no estan implementadas.

### RFC-0018 — experiencia interactiva de terminal

`docs/rfc/RFC-0018-interactive-terminal-experience.md` (estado: aceptada).
Define una TUI a pantalla completa dentro de `ag-cli`, tras la feature `tui`,
como front-end de los comandos existentes:

- Paridad de flags obligatoria: toda decision interactiva tiene su equivalente
  no interactivo, y pantalla y flags producen la misma peticion tipada para un
  unico ejecutor.
- Nunca interactiva por sorpresa: solo con terminal real, sin `CI`, sin
  `AG_NO_TUI`, sin `--no-interactive`. Los scripts actuales no cambian.
- Tres etapas: A (fundacion y asistente de `ag new`), B (`ag config` y
  `ag new --with`), C (pantalla de inicio de `ag` sin argumentos).
- Tres preguntas abiertas para la decision: idioma de la interfaz, posicion en
  las puertas, comportamiento de `ag` sin argumentos.


### RFC-0019 a RFC-0023 — arquitectura, entorno, operacion, reglas y funciones

Todas aceptadas el 2026-11-10, sin codigo todavia. Quedan abiertas dos
preguntas de `RFC-0021` (alcance de `ag setup`, custodia de la llave de
release) hasta su etapa C.

- `RFC-0019`: arquitectura por capas de los proyectos generados (dominio,
  aplicacion, infraestructura con la base de datos como modulo acotado,
  interfaz, raiz de composicion) forzada por Cargo; manifiesto `ag.toml`;
  alcance de interoperabilidad estrictamente backend (Docker, Kubernetes,
  Terraform, gRPC, Go, WebAssembly, Flutter). Propone retirar `ag-ui`.
- `RFC-0020`: variables de entorno declaradas, carga tipada fail-fast, tipo
  `Secret<T>`, archivo de secretos cifrado por entorno, comandos `ag env`.
- `RFC-0021`: instalacion con un comando desde binarios estaticos firmados
  (cierra el diseno del issue #141), `ag setup`, consola remota sobre SSH
  (`ag connect`) e interfaz de maquina (`--output json`, `ag describe`,
  plan/apply) para scripts y agentes automatizados.
- `RFC-0022`: guardarrailes de mantenibilidad, cada regla con el mecanismo de
  CI que la hace cumplir.
- `RFC-0023`: funciones serverless como componentes WebAssembly aislados en
  `ag-wasm-host`, declaradas en `schema.ag`, con triggers HTTP, schedule, cola
  y eventos, capacidades denegadas por defecto y limites por invocacion. Nueva
  Fase 5.5 aditiva en la Hoja de Ruta, posterior a la beta 0.5.

### Toolchain y dependencias

- Rust 1.95.0 a 1.99.1 (toolchain y MSRV) en todos los lugares que lo
  declaran, con checksums de instaladores regenerados.
- `cargo update` completo y 21 actualizaciones mayores (lista en el CHANGELOG),
  incluida la migracion a `axum` 0.8: capturas de ruta `{nombre}` en crates,
  ejemplos, benchmark y en el router que emite `ag generate`.
- Seis actualizaciones mayores no se aplican porque requieren migracion
  deliberada de codigo criptografico, TLS o de protocolo: issues #175 a #180.
- Los templates de `ag new` fijan `ag-core` por commit con `axum` 0.7 y siguen
  compilando; su pin debe subirse tras fusionar (ver CHANGELOG).

### Enmienda de la Hoja de Ruta (EN+ES)

- Nueva seccion **Fase 4.6**: la fila existia en la tabla resumen pero la fase
  no tenia seccion. Dos pistas (correo `RFC-0009`, workers `RFC-0012`) con
  estado por etapa verificado contra el codigo, criterios de salida y riesgos.
  Resuelve la ambiguedad de la etiqueta "4.6-D".
- Nueva seccion **Fase 4.7** (aditiva, propuesta): criterios de entrada,
  entregables por etapa, criterios de salida y riesgos.
- Nueva seccion **Fase 4.8** (aditiva, propuesta): fundaciones del proyecto
  en cinco etapas (capas, entorno, instalacion, consola remota, interfaz de
  maquina) con la declaracion de alcance backend.
- **Fases 5 a 10**: bloque de etapas que ordena los entregables ya existentes
  en pasos fusionables, y lista de RFC previas por la regla 28. Las etapas no
  anaden alcance.
- **Fase 5**: el criterio de entrada nombra el gate pre-Fase 5 y sus issues
  abiertos (#156, #157); el target auto-alojado se entrega antes que los
  alojados (`ADR-0009` regla 5); las etapas A y B de la Fase 4.7 pasan a ser
  criterio de salida de la v0.5.
- Correcciones: URL del repositorio (Fase 0), conteo de importadores (Fase 7,
  seis y no cinco), target `wasm32-wasip1` (Fase 10).
- `tools/split-masters.sh`: rangos recalculados; derivados de `docs/roadmap/`
  regenerados, con archivos nuevos para las Fases 4.6 y 4.7.
- `docs/master/VERSION.md` y `.github/workflows/docs.yml`: hash y tamano
  nuevos del maestro; el comando de verificacion local incluye el Blueprint
  markdown, que faltaba.

### Correcciones de la auditoria aplicadas en esta rama

- `Cargo.lock` no contenia el arbol de dependencias de la feature `avif` de
  `ag-storage` (PR #164). Un build limpio lo reescribia (343 lineas) y
  `cargo audit` inspeccionaba un lockfile incompleto.
- `cargo audit` fallaba en `main` con 15 vulnerabilidades por avisos publicados
  desde junio. Trece se resuelven con actualizaciones del lockfile compatibles
  por semver (`rustls` 0.23.46, `rustls-webpki` 0.103.16, `quinn-proto`
  0.11.19, `h2` 0.4.20, `webpki-roots`/`webpki-root-certs` 1.0.11, `tokio`
  1.54.1). Las dos restantes (`crypto-bigint`, `libsqlite3-sys`) no tienen
  correccion compatible y quedan en el issue #173; las filas "Audit" y "Deny"
  del gate pre-Fase 5 pasan de `pass` a `fail` con esa evidencia.
- `docs/modules/ag-cli/README.md` declaraba "seis comandos" y solo las Fases 2
  y 3; reescrito en ingles con la superficie real (ocho comandos).
- README raiz: la fila espanola de la Fase 4.6 habia divergido de la inglesa.
- Seis descriptores en `docs/pr-drafts/` correspondian a PRs ya fusionadas
  ; eliminados segun la regla de ciclo de vida.

### Hallazgos registrados como Issues (no resueltos aqui)

- #173 `cargo audit` en rojo: `crypto-bigint` y `libsqlite3-sys` sin correccion
  compatible (p1, seguridad).
- #181 Con `--all-features` se enlazan dos proveedores criptograficos de
  `rustls` y dos tests del MTA entran en panic (preexistente; CI no lo cubre).
- #174 `cargo deny` en rojo: `astral-tokio-tar` sin mantenimiento (ruta de
  dev-dependency).
- #168 CI no detecta un `Cargo.lock` desactualizado (sin `--locked`).
- #169 `ag dev` escucha en `0.0.0.0` por defecto.
- #170 Blueprint v4.1 contradice `ADR-0010` y el `ag-cache` actual.
- #171 CLAUDE.md secciones 3, 4 y 24 aun nombran el PDF v4.0.
- #172 Casillas del maestro para las Fases 0-4 sin marcar frente a STATUS.md.

## Documentos relacionados

- `docs/rfc/RFC-0018-interactive-terminal-experience.md`
- `docs/rfc/RFC-0019-project-layered-architecture.md`
- `docs/rfc/RFC-0020-environment-and-secrets.md`
- `docs/rfc/RFC-0021-server-install-remote-console-and-machine-interface.md`
- `docs/rfc/RFC-0022-maintainability-guardrails.md`
- `docs/rfc/RFC-0023-serverless-functions.md`
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/master/VERSION.md`
- `docs/roadmap/` (derivados, `README.md`, `STATUS.md`, `calendar.md`)
- `ADR-0009`, `ADR-0010`, `ADR-0013`, `RFC-0009`, `RFC-0012`

## Plan de prueba

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p ag-mail --features mta,api,minijinja
cargo build --workspace --locked
sha256sum docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md   # coincide con VERSION.md
bash tools/split-masters.sh && git diff --exit-code docs/roadmap docs/architecture
```

Resultado local (2026-11-10, Rust 1.99.1, Linux x86-64), sobre el commit final:

- `cargo fmt --all -- --check`: limpio.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; tambien con
  `--all-features`.
- `cargo test --workspace --all-targets`: 851 pasados, 0 fallidos, 10 ignorados.
- `cargo test -p ag-mail --features mta,api,minijinja`: 125 pasados.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: exit 0.
- `ag generate` sobre `examples/ecommerce-api/schema.ag`: salida identica a la
  versionada (`diff -rq`).
- Regeneracion de derivados sin diff; `tools/check-installers.sh` pasa.
- `cargo test -p ag-cli`: 52 tests unitarios y 6 de regresion con el binario
  real; tambien con `--no-default-features`.
- Asistente ejercitado en una pseudo-terminal real: 80x24 (flujo completo,
  nombre invalido, seleccion, revision, creacion), 60x15 (modo por linea),
  `NO_COLOR` (cero secuencias de color), `Esc` y `Ctrl-C` (nada escrito y
  terminal restaurada), `--no-interactive` y `AG_NO_TUI` (sin pantalla).
- No medido: aumento de tamano del binario release por la feature `tui`
  (criterio de salida de la fase, pendiente).
- NO pasa: `cargo test --workspace --all-features` (2 tests del MTA, issue
  #181, preexistente en `main`); `cargo audit` (2 vulnerabilidades, issue #173,
  frente a 15 antes de esta rama); `cargo deny` (issues #173 y #174).

Un test (`ag-edge` `tls_edge`) falla solo si hay un proxy HTTPS en el entorno;
pasa con las variables de proxy sin definir.

## Criterios de salida que avanza

Fase 4.7: los dos criterios de entrada, los ocho entregables de la etapa A y
los cinco transversales (para las pantallas existentes), y el criterio de
salida de paridad. Ver `docs/roadmap/STATUS.md`.

## Cierre de issues

Closes: none

- Refs #142 (auditoria previa; esta rama es su continuacion incremental)
- Refs #168
- Refs #173
- Refs #174
- Refs #175
- Refs #176
- Refs #177
- Refs #178
- Refs #179
- Refs #180
- Refs #181
- Refs #141
- Refs #169
- Refs #170
- Refs #171
- Refs #172

## Checklist

- [x] Pertenece a la fase correcta (documentacion y RFC antes que codigo).
- [x] Respeta la documentacion; el maestro se modifica con RFC y registro en
  `VERSION.md`.
- [x] Compila, pasa tests, fmt y clippy.
- [ ] Pasa audit y deny: NO. Quedan avisos sin correccion compatible (issues
  #173 y #174).
- [x] README sincronizado en el mismo cambio.
- [x] Una dependencia nueva (`ratatui`, opcional, justificada en `RFC-0018`
  seccion 4.8) y una retirada (`dialoguer`). Cambios de version en el CHANGELOG.
