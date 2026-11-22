# Seis plantillas de infraestructura para el catalogo (ola 6)

## Resumen

Anadir mail-service, dns-manager, object-storage-api, monitoring-service,
microservices-starter y cli-tool como plantillas andamiables y verificadas offline.

## Fase afectada

Ola 6 del catalogo de plantillas comunitarias. No altera fases del roadmap ni
el comportamiento de `ag new` para las plantillas existentes; el registro y el
scaffolder los gestiona el rol `catalog`.

## Tipo de cambio

- [x] Documentacion y catalogo de plantillas (`templates/<nombre>/`).
- [x] Codigo nuevo (plantillas Rust, sin cambios en crates compartidos).
- [ ] Cambio de arquitectura o de DSL.
- [ ] Cambio de seguridad o de dependencias del workspace.
- [ ] Cambio de roadmap o de targets de rendimiento.

## Documentos relacionados

- `CLAUDE.md` (reglas 1-7, 14-16, 19, 26, 27, 28 y 32).
- `templates/README.md` (convencion de plantillas y fijacion por `rev`).
- `docs/pr-drafts/README.md` (contrato de descriptores).

## Plan de prueba

Comandos ejecutados desde la raiz del worktree con el entorno de Cargo
compartido:

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
ROOT=templates
BASE=/tmp/ag-infra-verify
# para cada plantilla: copiar todos los archivos salvo template.toml a
# $BASE/<nombre>, sustituyendo {{name}} por el nombre de proyecto
cd $BASE/<nombre> && cargo check --offline
```

Nombres de proyecto usados: `mymail`, `mydns`, `mystore`, `mymonitor`,
`myservices` y `mycli`.

## Resultado de la verificacion

| Plantilla | Comando | Resultado |
| --- | --- | --- |
| mail-service | `cargo check --offline` | OK (exit 0) |
| dns-manager | `cargo check --offline` | OK (exit 0) |
| object-storage-api | `cargo check --offline` | OK (exit 0) |
| monitoring-service | `cargo check --offline` | OK (exit 0) |
| microservices-starter | `cargo check --offline` | OK (exit 0) |
| cli-tool | `cargo check --offline` | OK (exit 0) |

Sin warnings ni errores de compilacion en las seis. La integracion real con
`ag-observe` queda detras del feature opcional `ag-observe` porque el rev
fijado requiere crates OpenTelemetry ausentes en la cache offline; el default
expone los mismos endpoints con contadores atomicos.

## Criterios de salida que avanza

- Catalogo de al menos 20 plantillas: se entregan seis plantillas de
  infraestructura listas para registrar.
- Toda plantilla debe andamiarse y compilar offline: verificado 6/6.
- Toda plantilla declare necesidades externas reales sin exigirlas para
  compilar: declaradas en `template.toml` y en cada `README.md`.

## Cierre de issues

Closes: none

## Checklist

- [x] Pertenece a la fase correcta (ola 6, catalogo).
- [x] Respeta la documentacion y no toca archivos compartidos de la CLI.
- [x] No rompe arquitectura ni anade dependencias al workspace.
- [x] Compila y pasa `cargo check --offline` en las seis plantillas.
- [x] Documentacion actualizada (README por plantilla y CHANGELOG).
