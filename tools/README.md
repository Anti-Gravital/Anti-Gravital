# tools/

Scripts y utilidades de soporte del repositorio. No forman parte del
producto distribuido; existen para regenerar derivados de la
documentacion, verificar integridad y mantener consistencia.

## Scripts vigentes

- `split-masters.sh`: regenera los archivos verbatim bajo
  `docs/architecture/` y `docs/roadmap/` a partir de los maestros en
  `docs/master/`. Idempotente.
- `scaffold-crates.sh`: regenera los archivos esqueleto de los 15
  crates del workspace. Idempotente. Se usa una sola vez al
  inicializar el repositorio.
- `scaffold-docs.sh`: regenera los README de `docs/modules/` y las
  vistas verbatim bajo `docs/security/`, `docs/governance/`,
  `docs/benchmarks/` y `docs/dsl/`. Idempotente.

## Reglas

- Los scripts viven aqui mientras se necesiten para mantener el
  repositorio. Cuando un script se vuelve permanente y critico,
  evaluar migrarlo a un crate dedicado bajo `crates/` previa RFC.
- Sin secretos. Sin credenciales. Sin acceso a recursos externos.
- Compatibles con bash en Linux y macOS.
- Sin emojis en su salida.
