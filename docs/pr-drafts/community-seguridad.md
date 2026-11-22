# Descriptor de PR

## Resumen

fix(installers): ancla de integridad externa con fallo cerrado y fallback de desarrollo (Refs #141)

## Fase afectada

Fase 0 (Fundaciones y Gobernanza), instrumento de soporte/seguridad. No altera
el alcance del producto, el orden de fases ni la lista de crates. No avanza
ninguna casilla de `docs/roadmap/STATUS.md`: el issue #141 sigue abierto.

## Tipo de cambio

- [x] Correccion de bug (debilidad de diseno del ancla de confianza)
- [x] Seguridad / instaladores
- [x] Documentacion (SECURITY.md, CHANGELOG.md, guia de integridad)
- [ ] Gobernanza (no crea ni modifica RFC/ADR)
- [ ] Nueva feature de producto
- [ ] Cambio que rompe compatibilidad

## Contexto

`checksums/installers.sha256` vive en el mismo repositorio que los scripts que
atestigua, por lo que un checkout manipulado puede manipular tambien la lista.
ADR-0009 regla 4 exige instaladores auditables, pero no existia una forma de
aportar un ancla de confianza externa. No hay clave de release ni canal de
publicacion fuera del repositorio, asi que no se puede cerrar #141 sin inventar
una garantia de firma que no existe.

Se implementa la interfaz del ancla y su cumplimiento:

- `AG_INSTALLER_CHECKSUMS` apunta a un manifiesto SHA-256 obtenido fuera del
  checkout. Con la variable definida, `install.sh` y `install.ps1` verifican sus
  propios bytes antes de compilar o instalar y abortan si el archivo falta, no
  contiene la entrada del script o no coincide (fallo cerrado).
- Sin la variable, avisan y conservan el comportamiento anterior; ningun archivo
  del repositorio se trata como ancla de confianza.
- `tools/check-installers.sh` respeta la misma variable: el manifiesto externo
  gana y falla cerrado si no esta disponible; la lista local queda como fallback
  de desarrollo documentado para mantener verde la paridad en CI.
- `install.sh` admite `sha256sum` (Linux) y `shasum -a 256` (macOS).

Queda pendiente para cerrar #141: publicar los checksums por un canal externo al
repositorio o firmar los instaladores con una clave publica en SECURITY.md, y
verificar el flujo completo en una maquina limpia.

## Documentos relacionados

- `docs/security/INSTALLATION_INTEGRITY.md` (ancla, variables, limites)
- `SECURITY.md` (seccion "Integridad del instalador")
- `CHANGELOG.md` (seccion [Unreleased])
- `ADR-0009` regla 4 (instaladores auditables con verificacion de integridad)
- `CLAUDE.md` reglas 16, 19, 23, 26, 28
- Issue #141 (permanece abierto)

## Plan de prueba

```sh
bash -n install.sh
bash -n tools/check-installers.sh
bash tools/check-installers.sh
sha256sum install.sh install.ps1 > /tmp/ag-external.sha256
AG_INSTALLER_CHECKSUMS=/tmp/ag-external.sha256 bash tools/check-installers.sh
AG_INSTALLER_CHECKSUMS=/tmp/ag-absent.sha256 bash tools/check-installers.sh  # debe fallar
```

`tools/check-installers.sh` incluye pruebas nuevas para: aviso con la variable
sin definir, manifiesto externo valido que verifica y continua, manifiesto
ausente que aborta antes de invocar Cargo, y hash modificado que aborta antes
de invocar Cargo.

Limitacion de la verificacion local: no hay `pwsh`/`powershell` en el entorno,
por lo que `install.ps1` solo recibio comprobacion estatica (balance de
delimitadores y presencia de las llamadas clave); la ejecucion real en Windows
sigue pendiente.

## Criterios de salida que avanza

Ninguno. No se marca ninguna casilla de `docs/roadmap/STATUS.md`. Se avanza la
condicion tecnica del issue #141 relativa a "mecanismo verificable con ancla
externa y fallo cerrado", pero no las condiciones de publicacion/firma, por lo
que el issue no se cierra.

## Cierre de issues

Closes: none

- Refs #141 (avanza con el ancla configurable y el fallo cerrado; no cierra:
  faltan el canal de publicacion externo o la firma y la verificacion E2E)

## Checklist

- [x] Pertenece a la fase correcta (gobernanza y seguridad de Fase 0).
- [x] Respeta la documentacion y no anade alcance, crates ni capacidades nuevas.
- [x] No anade dependencias; no aplica cargo fmt/clippy/test/audit/deny.
- [x] Documentacion afectada actualizada en el mismo commit.
- [x] Sin emojis, sin atribucion de herramientas, mensaje de commit corto.
