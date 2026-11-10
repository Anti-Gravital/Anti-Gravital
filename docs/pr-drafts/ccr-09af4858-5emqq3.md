# Descriptor de PR

## Resumen

Enmienda de la Hoja de Ruta: secciones de Fase 4.6 y 4.7, etapas y RFC previas en Fases 5-10; RFC-0018 (experiencia interactiva de terminal); correccion de Cargo.lock

## Fase afectada

Hoja de Ruta completa (documento maestro). Introduce la Fase 4.7 aditiva como
propuesta. No implementa codigo funcional y no avanza ninguna puerta.

## Tipo de cambio

- [x] Correccion de bug (`Cargo.lock` desincronizado de los manifiestos)
- [x] Documentacion (maestro Hoja-de-Ruta EN+ES, derivados, README, RFC)
- [x] Gobernanza (limpieza de descriptores de PRs ya fusionadas)
- [ ] Nueva feature
- [ ] Cambio que rompe compatibilidad

## Contexto

Esta rama entrega tres cosas pedidas por el BDFL: una auditoria incremental del
repositorio desde la de junio (issue #142), la ampliacion y el detalle de la
Hoja de Ruta, y el diseno documentado de una interfaz interactiva de terminal
para crear y configurar proyectos. Siguiendo las reglas 5, 22 y 28, la interfaz
se entrega como RFC y como fase de la Hoja de Ruta; no se escribe codigo de la
TUI hasta que la RFC se decida.

### RFC-0018 — experiencia interactiva de terminal

`docs/rfc/RFC-0018-interactive-terminal-experience.md` (estado: propuesta).
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

Fusionar esta PR registra la decision de la RFC (seccion 9).

### Enmienda de la Hoja de Ruta (EN+ES)

- Nueva seccion **Fase 4.6**: la fila existia en la tabla resumen pero la fase
  no tenia seccion. Dos pistas (correo `RFC-0009`, workers `RFC-0012`) con
  estado por etapa verificado contra el codigo, criterios de salida y riesgos.
  Resuelve la ambiguedad de la etiqueta "4.6-D".
- Nueva seccion **Fase 4.7** (aditiva, propuesta): criterios de entrada,
  entregables por etapa, criterios de salida y riesgos.
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
- `docs/modules/ag-cli/README.md` declaraba "seis comandos" y solo las Fases 2
  y 3; reescrito en ingles con la superficie real (ocho comandos).
- README raiz: la fila espanola de la Fase 4.6 habia divergido de la inglesa.
- Seis descriptores en `docs/pr-drafts/` correspondian a PRs ya fusionadas
  ; eliminados segun la regla de ciclo de vida.

### Hallazgos registrados como Issues (no resueltos aqui)

- #168 CI no detecta un `Cargo.lock` desactualizado (sin `--locked`).
- #169 `ag dev` escucha en `0.0.0.0` por defecto.
- #170 Blueprint v4.1 contradice `ADR-0010` y el `ag-cache` actual.
- #171 CLAUDE.md secciones 3, 4 y 24 aun nombran el PDF v4.0.
- #172 Casillas del maestro para las Fases 0-4 sin marcar frente a STATUS.md.

## Documentos relacionados

- `docs/rfc/RFC-0018-interactive-terminal-experience.md`
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/master/VERSION.md`
- `docs/roadmap/` (derivados, `README.md`, `STATUS.md`, `calendar.md`)
- `ADR-0009`, `ADR-0010`, `ADR-0013`, `RFC-0009`, `RFC-0012`

## Plan de prueba

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --locked
sha256sum docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md   # coincide con VERSION.md
bash tools/split-masters.sh && git diff --exit-code docs/roadmap docs/architecture
```

Resultado local (2026-11-10, Rust 1.95.0, Linux x86-64): fmt limpio; clippy
exit 0; tests 820 pasados, 0 fallidos, 14 ignorados; regeneracion de derivados
sin diff.

## Criterios de salida que avanza

Ninguno. La PR anade la seccion "Fase 4.7" a `docs/roadmap/STATUS.md` con todas
sus casillas sin marcar.

## Cierre de issues

Closes: none

- Refs #142 (auditoria previa; esta rama es su continuacion incremental)
- Refs #168
- Refs #169
- Refs #170
- Refs #171
- Refs #172

## Checklist

- [x] Pertenece a la fase correcta (documentacion y RFC antes que codigo).
- [x] Respeta la documentacion; el maestro se modifica con RFC y registro en
  `VERSION.md`.
- [x] Compila, pasa tests, fmt y clippy.
- [x] README sincronizado en el mismo cambio.
- [x] Sin dependencias nuevas ni cambios de API.
