# Descriptor de PR

## Resumen

docs(audits): infrastructure review of issues #112, #149 and #141 (Refs #112, #149, #141)

## Fase afectada

Fase 0 (Fundaciones y Gobernanza), registro de auditoria de infraestructura. No
altera el alcance del producto, el orden de fases ni la lista de crates. No avanza
ninguna casilla de `docs/roadmap/STATUS.md`: ninguno de los tres issues se cierra.

## Tipo de cambio

- [ ] Correccion de bug
- [ ] Seguridad / instaladores
- [x] Documentacion (auditoria, indice de docs y CHANGELOG)
- [ ] Gobernanza (no crea ni modifica RFC/ADR)
- [ ] Nueva feature de producto
- [ ] Cambio que rompe compatibilidad

## Contexto

Revision de infraestructura de tres issues resueltos por otros roles:

- #112: el patron producer-only de `ag-workers` esta documentado y ejemplificado,
  pero `ag-edge` no tiene consumidor concreto de encolado durable. Se confirma el
  bloqueo y se corrigen tres detalles del plan futuro: `producer` es un feature
  marcador sin `cfg` en el codigo, no existe un check de ciclos en CI, y el
  arbol de features debe verificarse con y sin el feature opcional.
- #149: en las versiones fijadas (`opentelemetry-otlp` 0.33.1,
  `opentelemetry-proto` 0.33.1) el código protobuf se distribuye generado y no
  hay build script; una compilacion de prueba con `PROTOC=/nonexistent/protoc`
  tuvo exito. No es necesario hacer `protoc` obligatorio. Se propone habilitar
  OTLP/HTTP tras un feature opcional, coordinando el transporte con #177.
- #141: se reviso la interfaz fail-closed `AG_INSTALLER_CHECKSUMS` del rol de
  seguridad. Se acepta la interfaz; para cerrar el issue faltan la publicacion
  de un manifiesto y una firma con clave publica fuera del repositorio.

## Documentos relacionados

- `docs/audits/infrastructure-review-112-149-141.md` (nuevo)
- `docs/INDEX.md`, `CHANGELOG.md`
- `ADR-0009` (reglas 2, 4 y 5), `ADR-0011`, `ADR-0012`
- `RFC-0012` (secciones 3.3, 4, 5 S6 y 7)
- Issues #112, #149 y #141 (permanecen abiertos)

## Plan de prueba

```sh
cargo tree -p ag-workers --no-default-features --features producer -e normal --offline
cargo check --locked --offline -p ag-workers --no-default-features --features producer
cargo tree -p ag-edge -e features --offline
git diff --check
```

Ademas se revisa localmente que el documento nuevo no contenga atribucion de
herramientas ni marcas prohibidas, segun los jobs `prohibited-content` de
`.github/workflows/docs.yml`.

La comprobacion de OTLP se realizo en un crate desechable fuera del repositorio:
`opentelemetry-otlp = "=0.33.1"` con `default-features = false` y
`features = ["http-proto", "reqwest-client", "trace"]`, con
`PROTOC=/nonexistent/protoc`, `cargo check --offline` en verde.

## Criterios de salida que avanza

Ninguno. No se marca ninguna casilla de `docs/roadmap/STATUS.md`. La auditoria
avanza las condiciones de verificacion de los tres issues sin cerrarlos.

## Cierre de issues

Closes: none

- Refs #112 (revision; el issue sigue bloqueado hasta que exista un consumidor)
- Refs #149 (revision; la implementacion la posee otro rol)
- Refs #141 (revision de la interfaz; faltan publicacion y firma)

## Checklist

- [x] Pertenece a la fase correcta (gobernanza y auditoria de Fase 0).
- [x] Respeta la documentacion y no anade alcance, crates ni capacidades nuevas.
- [x] No anade dependencias ni codigo Rust; no aplica cargo fmt/clippy/test/audit/deny.
- [x] Documentacion afectada actualizada en el mismo commit.
- [x] Sin emojis, sin atribucion de herramientas, mensaje de commit corto.
